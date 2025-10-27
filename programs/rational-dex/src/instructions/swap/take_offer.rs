use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
        TransferChecked,
    },
};

use crate::events::OfferTaken;

use crate::error::DexError;
use crate::state::swap_state::{Offer, Price};

use super::{compute_token_b_wanted, transfer_tokens};

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    pub token_mint_a: InterfaceAccount<'info, Mint>,

    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = token_mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [b"price", token_mint_a.key().as_ref()],
        bump
    )]
    pub price_of_token_a: Account<'info, Price>,

    #[account(
        seeds = [b"price", token_mint_b.key().as_ref()],
        bump
    )]
    pub price_of_token_b: Account<'info, Price>,

    #[account(
        mut, // Why mut because we will close the account when offer is taken
        // close = maker, // Refund the sol to maker
        has_one = maker,
        has_one = token_mint_a,
        has_one = token_mint_b,
        seeds = [b"offer", maker.key().as_ref(), offer.id.to_le_bytes().as_ref()],
        bump = offer.bump
    )]
    offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn send_wanted_tokens_to_maker(context: Context<TakeOffer>, taker_amount: u64) -> Result<()> {
    let offer = &mut context.accounts.offer;

    // Basic checks
    require!(taker_amount > 0, DexError::InvalidAmount);
    require!(
        taker_amount <= offer.remaining_amount,
        DexError::InsufficientOffer
    );

    // Compute how many token B the taker must send
    let token_b_required = compute_token_b_wanted(
        taker_amount,
        context.accounts.price_of_token_a.price, // you'll need to pass these price accounts into TakeOffer if required
        context.accounts.price_of_token_b.price,
    )?;

    let result = transfer_tokens(
        &context.accounts.taker_token_account_b, // from
        &context.accounts.maker_token_account_b, // to
        &token_b_required,                       // amount
        &context.accounts.token_mint_b,
        &context.accounts.taker,
        &context.accounts.token_program,
    );

    //  Update remaining_amount
    offer.remaining_amount = offer
        .remaining_amount
        .checked_sub(taker_amount)
        .ok_or(DexError::Overflow)?;

    emit!(OfferTaken {
        offer_id: offer.id,
        taker: context.accounts.taker.key(),
        amount_taken: taker_amount,
        remaining_amount: offer.remaining_amount,
    });

    result
}

pub fn withdraw_and_close_vault(context: Context<TakeOffer>, taker_amount: u64) -> Result<()> {
    let offer = &mut context.accounts.offer;

    let seeds = &[
        b"offer",
        context.accounts.maker.to_account_info().key.as_ref(),
        &offer.id.to_le_bytes()[..],
        &[offer.bump],
    ];
    let signer_seeds = [&seeds[..]];

    let accounts = TransferChecked {
        from: context.accounts.vault.to_account_info(),
        to: context.accounts.taker_token_account_a.to_account_info(),
        mint: context.accounts.token_mint_a.to_account_info(),
        authority: offer.to_account_info(),
    };
    // Signer is offer account
    let cpi_context = CpiContext::new_with_signer(
        context.accounts.token_program.to_account_info(),
        accounts,
        &signer_seeds,
    );

    // We move the tokens from the vault to the taker's account
    transfer_checked(
        cpi_context,
        taker_amount, // amount that we gonna transfer
        context.accounts.token_mint_a.decimals,
    )?;

    // If fully filled — close vault
    if offer.remaining_amount == 0 {
        let accounts = CloseAccount {
            account: context.accounts.vault.to_account_info(),
            destination: context.accounts.taker.to_account_info(), // We refund any lamport to the taker
            authority: context.accounts.offer.to_account_info(),
        };

        let cpi_context = CpiContext::new_with_signer(
            context.accounts.token_program.to_account_info(),
            accounts,
            &signer_seeds,
        );

        close_account(cpi_context)?;
    }

    Ok(())
}
