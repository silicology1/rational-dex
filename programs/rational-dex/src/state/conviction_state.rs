use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Proposal {
    pub author: Pubkey,
    #[max_len(32)]
    pub evidence: String,
    pub votes_account_count: u8, // number of ProposalVotes PDAs
    pub final_score: Option<u8>,
    pub score_updated_at: Option<i64>,
}

#[account]
#[derive(InitSpace)]
pub struct ProposalVotes {
    #[max_len(250)] // each chunk max 250 votes
    pub votes: Vec<VoteRecord>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, InitSpace)]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub score: u8,
    pub conviction: u8,
    pub weight: u8,
}
