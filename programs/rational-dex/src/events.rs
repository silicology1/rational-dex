use anchor_lang::prelude::*;

#[event]
pub struct OfferCreated {
    pub offer_id: u64,
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub offered_amount: u64,
    pub wanted_amount: u64,
}

#[event]
pub struct OfferTaken {
    pub offer_id: u64,
    pub taker: Pubkey,
    pub amount_taken: u64,
    pub remaining_amount: u64,
}

#[event]
pub struct OfferCancelled {
    pub offer_id: u64,
    pub maker: Pubkey,
    pub refunded_amount: u64,
}
