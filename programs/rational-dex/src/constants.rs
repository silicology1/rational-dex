use anchor_lang::prelude::*;
use arcium_anchor::comp_def_offset;

#[constant]
pub const SEED: &str = "anchor";

pub const ANCHOR_DISCRIMINATOR: usize = 8;

pub const COMP_DEF_OFFSET_INIT_VOTE_STATS: u32 = comp_def_offset("init_vote_stats");
pub const COMP_DEF_OFFSET_VOTE: u32 = comp_def_offset("vote");
pub const COMP_DEF_OFFSET_REVEAL: u32 = comp_def_offset("reveal_result");
