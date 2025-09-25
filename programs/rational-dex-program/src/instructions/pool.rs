use crate::state::pool_state::Pool;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use sha2::{Digest, Sha256};

pub fn initialize_pool_handler(
    ctx: Context<InitializePool>,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.authority = *ctx.accounts.authority.key;
    pool.vault_a = *ctx.accounts.vault_a.to_account_info().key;
    pool.vault_b = *ctx.accounts.vault_b.to_account_info().key;
    pool.mint_lp = *ctx.accounts.mint_lp.to_account_info().key;
    pool.mint_a = *ctx.accounts.mint_a.to_account_info().key;
    pool.mint_b = *ctx.accounts.mint_b.to_account_info().key;
    pool.fee_numerator = fee_numerator;
    pool.fee_denominator = fee_denominator;
    pool.total_lp_supply = 0;
    pool.consensus_price = 0;
    pool.last_price_timestamp = 0;
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, payer = authority, space = 8 + Pool::INIT_SPACE, seeds = [b"pool"], bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut)]
    pub vault_a: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub vault_b: InterfaceAccount<'info, TokenAccount>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub mint_lp: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}
