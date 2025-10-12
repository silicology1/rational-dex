pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;
pub use constants::*;
pub use instructions::*;
#[allow(unused_imports)]
pub use state::*;

declare_id!("EEL1Q3J9MjPxTWagTKE39jpUVBjUg7q283ztTVzbveDz");

#[arcium_program]
pub mod rational_dex {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
    pub fn initialize_pool(ctx: Context<InitializePool>, fee_num: u64, fee_den: u64) -> Result<()> {
        pool::initialize_pool_handler(ctx, fee_num, fee_den)
    }

    pub fn init_vote_stats_comp_def(ctx: Context<InitVoteStatsCompDef>) -> Result<()> {
        init_vote_stats_comp_def_handle(ctx)
    }

    #[arcium_callback(encrypted_ix = "init_vote_stats")]
    pub fn init_vote_stats_callback(
        ctx: Context<InitVoteStatsCallback>,
        output: ComputationOutputs<InitVoteStatsOutput>,
    ) -> Result<()> {
        init_vote_stats_callback_handler(ctx, output)
    }

    pub fn init_vote_comp_def(ctx: Context<InitVoteCompDef>) -> Result<()> {
        init_vote_comp_def_handler(ctx)
    }

    pub fn vote(
        ctx: Context<Vote>,
        computation_offset: u64,
        _id: u32,
        vote: [u8; 32],
        vote_encryption_pubkey: [u8; 32],
        vote_nonce: u128,
    ) -> Result<()> {
        vote_handler(
            ctx,
            computation_offset,
            _id,
            vote,
            vote_encryption_pubkey,
            vote_nonce,
        )
    }

    #[arcium_callback(encrypted_ix = "vote")]
    pub fn vote_callback(
        ctx: Context<VoteCallback>,
        output: ComputationOutputs<VoteOutput>,
    ) -> Result<()> {
        vote_callback_handler(ctx, output)
    }
    pub fn init_reveal_result_comp_def(ctx: Context<InitRevealResultCompDef>) -> Result<()> {
        init_reveal_result_comp_def_handler(ctx)
    }

    pub fn reveal_result(
        ctx: Context<RevealVotingResult>,
        computation_offset: u64,
        id: u32,
    ) -> Result<()> {
        reveal_result_handler(ctx, computation_offset, id)
    }
    #[arcium_callback(encrypted_ix = "reveal_result")]
    pub fn reveal_result_callback(
        ctx: Context<RevealResultCallback>,
        output: ComputationOutputs<RevealResultOutput>,
    ) -> Result<()> {
        reveal_result_callback_handler(ctx, output)
    }

    // Conviction Voting Instructions

    pub fn create_proposal(ctx: Context<CreateProposal>, evidence: String) -> Result<()> {
        create_proposal_handler(ctx, evidence)
    }

    // pub fn conviction_vote(
    //     ctx: Context<CastVote>,
    //     _proposer: Pubkey,
    //     score: u8,
    //     conviction: u8,
    // ) -> Result<()> {
    //     conviction_vote_handler(ctx, _proposer, score, conviction)
    // }

    // pub fn finalize(ctx: Context<FinalizeProposal>, _proposer: Pubkey) -> Result<()> {
    //     finalize_handler(ctx, _proposer)
    // }
}
