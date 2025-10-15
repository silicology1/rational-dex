/// Conviction voting is used to assign reputation scores to accounts. Each account can receive a score between 0 and 5, and the final reputation is determined by the median of all votes, weighted by conviction.
use crate::conviction_state::MAX_VOTES;
use crate::state::conviction_state::{
    AuthorState, Proposal, ProposalAccounts, Score, Voter, Weight,
};
use anchor_lang::prelude::*;

pub fn initialize_proposal_handler(
    ctx: Context<InitializeProposal>,
    evidence: String,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let author_state = &mut ctx.accounts.author_state;
    let proposal_accounts = &mut ctx.accounts.proposal_accounts;

    // ✅ Ownership checks
    require!(
        ctx.accounts.score.to_account_info().owner == ctx.program_id,
        VotingError::InvalidAccountOwner
    );
    require!(
        ctx.accounts.weight.to_account_info().owner == ctx.program_id,
        VotingError::InvalidAccountOwner
    );
    require!(
        ctx.accounts.voter.to_account_info().owner == ctx.program_id,
        VotingError::InvalidAccountOwner
    );

    // ✅ Initialize proposal data
    proposal.author = ctx.accounts.author.key();
    proposal.evidence = evidence;
    proposal.final_score = None;
    proposal.score_updated_at = None;

    // ✅ Record linked accounts
    proposal_accounts.proposal = proposal.key();
    proposal_accounts.score_account = ctx.accounts.score.key();
    proposal_accounts.weight_account = ctx.accounts.weight.key();
    proposal_accounts.voter_account = ctx.accounts.voter.key();

    // ✅ Increment author’s proposal count
    author_state.proposal_count = author_state
        .proposal_count
        .checked_add(1)
        .ok_or(VotingError::OverflowError)?; // return a custom error

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeProposal<'info> {
    #[account(mut)]
    pub author: Signer<'info>,

    // Stores author metadata and proposal counter
    #[account(
        init_if_needed,
        payer = author,
        space = 8 + AuthorState::INIT_SPACE,
        seeds = [b"author_state", author.key().as_ref()],
        bump
    )]
    pub author_state: Account<'info, AuthorState>,

    // Each proposal has unique index under the same author
    #[account(
        init,
        payer = author,
        space = 8 + Proposal::INIT_SPACE,
        seeds = [
            b"proposal",
            author.key().as_ref(),
            &author_state.proposal_count.to_le_bytes()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    // Record that links the proposal and its related PDAs
    #[account(
        init,
        payer = author,
        space = 8 + ProposalAccounts::INIT_SPACE,
        seeds = [
            b"proposal_accounts",
            proposal.key().as_ref()
        ],
        bump
    )]
    pub proposal_accounts: Account<'info, ProposalAccounts>,

    // Large zero-copy accounts (allocated separately)
    #[account(zero)]
    pub score: AccountLoader<'info, Score>,

    #[account(zero)]
    pub weight: AccountLoader<'info, Weight>,

    #[account(zero)]
    pub voter: AccountLoader<'info, Voter>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposer: Pubkey)]
pub struct ConvictionCastVote<'info> {
    #[account(mut, seeds = [b"proposal", proposer.key().as_ref()], bump)]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub score: AccountLoader<'info, Score>,
    #[account(mut)]
    pub weight: AccountLoader<'info, Weight>,
    #[account(mut)]
    pub voter: AccountLoader<'info, Voter>,
    #[account(mut)]
    pub voter_signer: Signer<'info>,
}

pub fn conviction_vote_handler(
    ctx: Context<ConvictionCastVote>,
    _proposer: Pubkey,
    score_value: u8,
    conviction: u8,
) -> Result<()> {
    require!(score_value <= 5, VotingError::InvalidScore);
    require!(conviction <= 6, VotingError::InvalidConviction);

    let mut scores = ctx.accounts.score.load_mut()?;
    let mut weights = ctx.accounts.weight.load_mut()?;
    let mut voters = ctx.accounts.voter.load_mut()?;

    let voter_pk = ctx.accounts.voter_signer.key();
    let weight_val = conviction_weight(conviction)?;

    // Find if voter already voted
    let mut index: Option<usize> = None;
    for i in 0..MAX_VOTES {
        if voters.data[i] == voter_pk {
            index = Some(i);
            break;
        }
        if voters.data[i] == Pubkey::default() && index.is_none() {
            // first empty slot
            index = Some(i);
        }
    }

    require!(index.is_some(), VotingError::NoSpaceLeft);
    let i = index.unwrap();

    voters.data[i] = voter_pk;
    scores.data[i] = score_value;
    weights.data[i] = weight_val;

    Ok(())
}

#[derive(Accounts)]
#[instruction(proposer: Pubkey)]
pub struct FinalizeProposal<'info> {
    #[account(mut, seeds = [b"proposal", proposer.key().as_ref()], bump)]
    pub proposal: Account<'info, Proposal>,
    #[account()]
    pub score: AccountLoader<'info, Score>,
    #[account()]
    pub weight: AccountLoader<'info, Weight>,
    #[account()]
    pub voter: AccountLoader<'info, Voter>,
    #[account(mut)]
    pub user: Signer<'info>,
}

pub fn finalize_handler(ctx: Context<FinalizeProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;

    let scores = ctx.accounts.score.load()?;
    let weights = ctx.accounts.weight.load()?;
    let voters = ctx.accounts.voter.load()?;

    let mut weighted_scores = vec![];

    for i in 0..MAX_VOTES {
        if voters.data[i] != Pubkey::default() {
            for _ in 0..weights.data[i] {
                weighted_scores.push(scores.data[i]);
            }
        }
    }

    require!(!weighted_scores.is_empty(), VotingError::NoVotes);

    weighted_scores.sort();
    let mid = weighted_scores.len() / 2;
    let median = if weighted_scores.len() % 2 == 0 {
        (weighted_scores[mid - 1] + weighted_scores[mid]) / 2
    } else {
        weighted_scores[mid]
    };

    proposal.final_score = Some(median);
    proposal.score_updated_at = Some(Clock::get()?.unix_timestamp);

    Ok(())
}

#[error_code]
pub enum VotingError {
    #[msg("Invalid score (must be between 0–5).")]
    InvalidScore,
    #[msg("Invalid conviction (must be between 0–6).")]
    InvalidConviction,
    #[msg("No votes yet.")]
    NoVotes,
    #[msg("Already delegating.")]
    AlreadyDelegating,
    #[msg("Unauthorized.")]
    Unauthorized,
    #[msg("No space left.")]
    NoSpaceLeft,
    #[msg("Invalid account owner.")]
    InvalidAccountOwner,
    #[msg("Overflow error.")]
    OverflowError,
}

pub fn conviction_weight(conviction: u8) -> Result<u8> {
    require!(conviction <= 6, VotingError::InvalidConviction);
    Ok(conviction)
}
