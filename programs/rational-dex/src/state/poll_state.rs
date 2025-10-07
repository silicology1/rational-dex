use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PollAccount {
    /// PDA bump seed
    pub bump: u8,
    /// Encrypted vote counters: [yes_count, no_count] as 32-byte ciphertexts
    pub vote_state: [[u8; 32]; 7],
    /// Unique identifier for this poll
    pub id: u32,
    /// Public key of the poll creator (only they can reveal results)
    pub authority: Pubkey,
    /// Cryptographic nonce for the encrypted vote counters
    pub nonce: u128,
    /// The poll question (max 50 characters)
    pub price: u64,

    pub mint0: Pubkey,
    pub mint1: Pubkey,
}
