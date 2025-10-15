use anchor_lang::prelude::*;

pub const MAX_VOTES: usize = 10_485_752; // ~10 MB

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

#[account]
#[derive(InitSpace)]
pub struct ProposalAccounts {
    pub proposal: Pubkey,
    pub score_account: Pubkey,
    pub weight_account: Pubkey,
    pub voter_account: Pubkey,
}
