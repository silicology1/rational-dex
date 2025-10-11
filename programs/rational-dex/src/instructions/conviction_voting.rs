use crate::state::conviction_state::{Proposal, VoteRecord};
use anchor_lang::prelude::*;

pub fn create_proposal_handle(ctx: Context<CreateProposal>, title: String) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    proposal.author = ctx.accounts.author.key();
    proposal.evidence = title;
    proposal.votes = Vec::new();
    proposal.final_score = None;
    Ok(())
}

pub fn conviction_vote_handler(
    ctx: Context<CastVote>,
    _proposer: Pubkey,
    score: u8,
    conviction: u8,
) -> Result<()> {
    require!(score <= 5, VotingError::InvalidScore);
    require!(conviction <= 6, VotingError::InvalidConviction);

    let voter = &ctx.accounts.voter;
    let proposal = &mut ctx.accounts.proposal;

    let weight = conviction_weight(conviction)?;
    let vote = VoteRecord {
        voter: voter.key(),
        score,
        conviction,
        weight,
    };

    // Update or insert vote
    if let Some(existing) = proposal.votes.iter_mut().find(|v| v.voter == voter.key()) {
        *existing = vote;
    } else {
        proposal.votes.push(vote);
    }

    Ok(())
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub author: Signer<'info>,
    #[account(init, payer = author, space = 8 + Proposal::INIT_SPACE, seeds = [b"proposal",author.key().as_ref()], bump)]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(proposer: Pubkey)]
pub struct CastVote<'info> {
    #[account(mut,seeds = [b"proposal",proposer.as_ref()], bump
    )]
    pub proposal: Account<'info, Proposal>,
    #[account(mut)]
    pub voter: Signer<'info>,
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
}

/// Map conviction to vote weight multiplier
fn conviction_weight(conviction: u8) -> Result<f64> {
    let weight = match conviction {
        0 => 0.1,
        1 => 1.0,
        2 => 2.0,
        3 => 3.0,
        4 => 4.0,
        5 => 5.0,
        6 => 6.0,
        _ => return err!(VotingError::InvalidConviction),
    };
    Ok(weight)
}
