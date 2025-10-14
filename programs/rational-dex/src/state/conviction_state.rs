use anchor_lang::prelude::*;

pub const MAX_VOTES: usize = 10_485_752; // ~10 MB

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub author: Pubkey,
    #[max_len(32)]
    pub evidence: String,
    pub votes_account_count: u8, // number of (Score, Weight, Voter) chunks
    pub final_score: Option<u8>,
    pub score_updated_at: Option<i64>,
}

#[account(zero_copy)]
pub struct Score {
    pub data: [u8; MAX_VOTES],
}

#[account(zero_copy)]
pub struct Weight {
    pub data: [u8; MAX_VOTES],
}

#[account(zero_copy)]
pub struct Voter {
    pub data: [Pubkey; MAX_VOTES],
}
