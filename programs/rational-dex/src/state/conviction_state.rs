use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub author: Pubkey,
    #[max_len(32)]
    pub evidence: String,
    pub final_score: Option<u8>,
    pub score_updated_at: Option<i64>,
}

#[account]
#[derive(InitSpace)]
pub struct AuthorState {
    pub author: Pubkey,
    pub proposal_count: u64,
    pub last_final_score: Option<u8>,
    pub score_updated_at: Option<i64>,
}

#[account]
#[derive(InitSpace)]
pub struct Scores {
    pub counts: [u64; 11], // 0–10 scores
}

#[account]
#[derive(InitSpace)]
pub struct Voter {
    pub voted: bool,
    pub conviction: u8,
    pub locked_amount: u64,
    pub unlock_time: i64, // Unix timestamp when tokens can be unlocked
}
