pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("423RnyowFFTqfPRsKAWPEvprwTvcTG3jpHFAKrqPdiwv");

#[program]
pub mod temp {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_pool(ctx: Context<InitializePool>, fee_num: u64, fee_den: u64) -> Result<()> {
        pool::initialize_pool_handler(ctx, fee_num, fee_den)
    }
}
