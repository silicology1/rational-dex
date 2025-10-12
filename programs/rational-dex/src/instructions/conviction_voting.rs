/// Conviction voting is used to assign reputation scores to accounts. Each account can receive a score between 0 and 5, and the final reputation is determined by the median of all votes, weighted by conviction.
use crate::state::conviction_state::{Proposal, VoteRecord};
use anchor_lang::prelude::*;

pub fn create_proposal_handler(ctx: Context<CreateProposal>, evidence: String) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    proposal.author = ctx.accounts.author.key();
    proposal.evidence = evidence;
    proposal.votes_account_count = 0;
    proposal.final_score = None;
    Ok(())
}

// pub fn conviction_vote_handler(
//     ctx: Context<CastVote>,
//     _proposer: Pubkey,
//     score: u8,
//     conviction: u8,
// ) -> Result<()> {
//     require!(score <= 5, VotingError::InvalidScore);
//     require!(conviction <= 6, VotingError::InvalidConviction);

//     let voter = &ctx.accounts.voter;
//     let proposal = &mut ctx.accounts.proposal;

//     let weight = conviction_weight(conviction)?;
//     let vote = VoteRecord {
//         voter: voter.key(),
//         score,
//         conviction,
//         weight,
//     };

//     // Update or insert vote
//     if let Some(existing) = proposal.votes.iter_mut().find(|v| v.voter == voter.key()) {
//         *existing = vote;
//     } else {
//         proposal.votes.push(vote);
//     }

//     Ok(())
// }

// /// Finalize the proposal and compute median
// pub fn finalize_handler(ctx: Context<FinalizeProposal>, _proposer: Pubkey) -> Result<()> {
//     let proposal = &mut ctx.accounts.proposal;
//     require!(!proposal.votes.is_empty(), VotingError::NoVotes);

//     let mut scores: Vec<u8> = proposal
//         .votes
//         .iter()
//         .flat_map(|v| std::iter::repeat(v.score).take(v.weight as usize))
//         .collect();

//     scores.sort_by(|a, b| a.partial_cmp(b).unwrap());
//     let mid = scores.len() / 2;
//     let median = if scores.len() % 2 == 0 {
//         (scores[mid - 1] + scores[mid]) / 2
//     } else {
//         scores[mid]
//     };

//     proposal.final_score = Some(median);
//     proposal.score_updated_at = Some(Clock::get()?.unix_timestamp);

//     Ok(())
// }

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub author: Signer<'info>,
    #[account(init, payer = author, space = 8 + Proposal::INIT_SPACE, seeds = [b"proposal",author.key().as_ref()], bump)]
    pub proposal: Account<'info, Proposal>,
    pub system_program: Program<'info, System>,
}

// #[derive(Accounts)]
// #[instruction(proposer: Pubkey)]
// pub struct CastVote<'info> {
//     #[account(mut,seeds = [b"proposal",proposer.as_ref()], bump
//     )]
//     pub proposal: Account<'info, Proposal>,
//     #[account(mut)]
//     pub voter: Signer<'info>,
// }

// #[derive(Accounts)]
// #[instruction(proposer: Pubkey)]
// pub struct FinalizeProposal<'info> {
//     #[account(mut,seeds = [b"proposal",proposer.as_ref()], bump
//     )]
//     pub proposal: Account<'info, Proposal>,
//     pub authority: Signer<'info>,
// }

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

pub fn conviction_weight(conviction: u8) -> Result<u8> {
    require!(conviction <= 6, VotingError::InvalidConviction);
    Ok(conviction)
}
