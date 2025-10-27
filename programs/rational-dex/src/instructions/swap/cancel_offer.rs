use crate::error::DexError;
use crate::events::OfferCancelled;
use crate::state::swap_state::Offer;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
    TransferChecked,
};

#[derive(Accounts)]
pub struct CancelOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = token_mint_a,
        seeds = [b"offer", maker.key().as_ref(), offer.id.to_le_bytes().as_ref()],
        bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
}

pub fn cancel_offer(context: Context<CancelOffer>) -> Result<()> {
    let offer = &context.accounts.offer;
    let remaining = offer.remaining_amount;

    require!(remaining > 0, DexError::InvalidAmount);

    // PDA signer seeds
    let seeds = &[
        b"offer",
        context.accounts.maker.key.as_ref(),
        &offer.id.to_le_bytes()[..],
        &[offer.bump],
    ];
    let signer_seeds = [&seeds[..]];

    // Move remaining tokens back to maker
    let accounts = TransferChecked {
        from: context.accounts.vault.to_account_info(),
        to: context.accounts.maker_token_account_a.to_account_info(),
        mint: context.accounts.token_mint_a.to_account_info(),
        authority: offer.to_account_info(),
    };
    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );
    transfer_checked(
        cpi_context,
        remaining,
        context.accounts.token_mint_a.decimals,
    )?;

    // Close the vault account
    let accounts = CloseAccount {
        account: context.accounts.vault.to_account_info(),
        destination: context.accounts.maker.to_account_info(),
        authority: context.accounts.offer.to_account_info(),
    };
    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );
    close_account(cpi_context)?;

    emit!(OfferCancelled {
        offer_id: offer.id,
        maker: context.accounts.maker.key(),
        refunded_amount: remaining,
    });

    Ok(())
}
