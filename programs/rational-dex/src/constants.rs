use anchor_lang::prelude::*;
use arcium_anchor::comp_def_offset;

#[constant]
pub const SEED: &str = "anchor";

// Price scaling factor: prices are stored as integer * 1e6
#[constant]
pub const PRICE_SCALE: u64 = 1_000_000;

// Seeds used for PDA derivations
#[constant]
pub const POOL_SEED: &[u8] = b"pool";

#[constant]
pub const VOTE_ROUND_SEED: &[u8] = b"voteround";

#[constant]
pub const VOTE_ACCOUNT_SEED: &[u8] = b"vote";

pub const COMP_DEF_OFFSET_INIT_VOTE_STATS: u32 = comp_def_offset("init_vote_stats");
pub const COMP_DEF_OFFSET_VOTE: u32 = comp_def_offset("vote");
pub const COMP_DEF_OFFSET_REVEAL: u32 = comp_def_offset("reveal_result");
