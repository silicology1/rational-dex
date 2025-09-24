use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VoteRound {
    pub authority: Pubkey,
    pub bump: u8,
    pub commit_window_seconds: u64,
    pub reveal_window_seconds: u64,
    pub start_timestamp: u64,
    pub stage: VoteStage,
    pub total_weight: u64,
    pub finalized: bool,
    pub start_seed: [u8; 8], // store seed used to derive PDA so we can use it in signer seeds
}

#[account]
#[derive(InitSpace)]
pub struct VoteAccount {
    pub vote_round: Pubkey,
    pub voter: Pubkey,
    pub hash: [u8; 32],
    pub revealed: bool,
    pub stake_amount: u64,
    pub lock_seconds: u64,
    pub revealed_price: u64,
    pub weight: u64,
    pub nonce: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum VoteStage {
    Commit,
    Reveal,
}
