/// Conviction voting is used to assign reputation scores to accounts. Each account can receive a score between 0 and 10.
use crate::state::conviction_state::{AuthorState, Proposal, Scores, Voter};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};
pub fn initialize_proposal_handler(
    ctx: Context<InitializeProposal>,
    evidence: String,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let author_state = &mut ctx.accounts.author_state;
    // ✅ Initialize proposal data
    proposal.author = ctx.accounts.author.key();
    proposal.evidence = evidence;
    proposal.final_score = None;
    proposal.score_updated_at = None;

    // ✅ Increment author’s proposal count
    author_state.proposal_count = author_state
        .proposal_count
        .checked_add(1)
        .ok_or(VotingError::OverflowError)?; // return a custom error

    Ok(())
}

pub fn conviction_vote_handler(
    ctx: Context<VoteProposal>,
    _proposal_count: u64,
    score: u8,
    conviction: u8,
) -> Result<()> {
    let scores = &mut ctx.accounts.scores;
    let voter_account = &mut ctx.accounts.voter_account;

    // ✅ Validate score range
    require!(score <= 10, VotingError::InvalidScore);

    // ✅ Get conviction weight
    let weight = conviction_weight(conviction)? as u64;

    // ✅ Check if voter has already voted
    require!(!voter_account.voted, VotingError::AlreadyVoted);

    // Initialize score it scores.count is empty:
    if scores.counts.is_empty() {
        scores.counts = [0u64; 11]; // initialize all score counts
    }

    // ✅ Multiply score by conviction weight
    let effective_vote = (score as u64)
        .checked_mul(weight)
        .ok_or(VotingError::OverflowError)?;

    // ✅ Increment the weighted vote
    scores.counts[score as usize] = scores.counts[score as usize]
        .checked_add(effective_vote)
        .ok_or(VotingError::OverflowError)?;
    // ✅ Mark voter as voted
    voter_account.voted = true;

    // ✅ Lock tokens based on conviction
    let lock_amount: u64 = 10_000_000 * weight; // example: conviction, 10 tokens per conviction weight
    let decimals = ctx.accounts.mint.decimals;
    let lock_duration = 60 * 60 * 24 * (weight as i64) * 10; // days = weight × 10 day

    voter_account.conviction = conviction;
    voter_account.locked_amount = lock_amount;
    voter_account.unlock_time = Clock::get()?.unix_timestamp + lock_duration;

    // SPL TransferChecked ensures mint & decimals consistency
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.voter_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.vault_token_account.to_account_info(),
                authority: ctx.accounts.voter.to_account_info(),
            },
        ),
        lock_amount,
        decimals,
    )?;

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
            &author_state.proposal_count.to_string().as_bytes()
        ],
        bump
    )]
    pub proposal: Account<'info, Proposal>,

    // Record that links the proposal and its related PDAs
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposal_count: u64)]
pub struct VoteProposal<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    // ✅ SPL Token mint (your native token)
    #[account(mint::token_program = token_program)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
          mut,
          associated_token::mint = mint,
          associated_token::authority = voter,
          associated_token::token_program = token_program,
      )]
    pub voter_token_account: InterfaceAccount<'info, TokenAccount>,

    // ✅ PDA vault to hold locked tokens
    #[account(
            init_if_needed, // double check it, init_if_needed or just init
            payer = voter,
            associated_token::mint = mint,
            associated_token::authority = scores,
            associated_token::token_program = token_program,
        )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(init_if_needed, payer = voter, space = 8 + Scores::INIT_SPACE,
        seeds = [
            b"scores",
            proposal_count.to_string().as_bytes(),
        ],
        bump
    )]
    pub scores: Account<'info, Scores>,

    #[account(
        init_if_needed,
        payer = voter,
        space = 8 + Voter::INIT_SPACE,
        seeds = [
            b"voter",
           proposal_count.to_string().as_bytes(),
           voter.key().as_ref()
        ],
        bump
    )]
    pub voter_account: Account<'info, Voter>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[error_code]
pub enum VotingError {
    #[msg("Invalid score (must be between 0–10).")]
    InvalidScore,
    #[msg("Invalid conviction (must be between 0–6).")]
    InvalidConviction,
    #[msg("No votes yet.")]
    NoVotes,
    #[msg("Already Voted.")]
    AlreadyVoted,
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
