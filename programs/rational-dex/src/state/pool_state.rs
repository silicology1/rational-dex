use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub authority: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub mint_lp: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub bump: u8,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
    pub total_lp_supply: u64,
    pub consensus_price: u64,
    pub last_price_timestamp: u64,
}
