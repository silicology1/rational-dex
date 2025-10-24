use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Offer {
    pub id: u64,
    // Maker is who make the offer
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub token_b_wanted_amount: u64, // Its calculated automatically from the token A amount and the swap rate
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Price {
    pub token_mint: Pubkey,
    pub price: u64,
    pub last_updated: u64,
    pub bump: u8,
}
