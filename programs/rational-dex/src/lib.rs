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
}
