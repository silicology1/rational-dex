use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub author: Pubkey,
    #[max_len(32)]
    pub evidence: String,
    #[max_len(1000)]
    pub votes: Vec<VoteRecord>,
    pub final_score: Option<f64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, InitSpace)]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub score: u8,
    pub conviction: u8,
    pub weight: f64,
}
