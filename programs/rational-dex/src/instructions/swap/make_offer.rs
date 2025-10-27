use crate::state::swap_state::{Offer, Price};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::ANCHOR_DISCRIMINATOR;

use super::{compute_token_b_wanted, transfer_tokens};
use crate::events::OfferCreated;

#[derive(Accounts)]
#[instruction(id: u64)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub token_mint_a: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub token_mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = ANCHOR_DISCRIMINATOR + Offer::INIT_SPACE,
        seeds = [b"offer", maker.key().as_ref(), id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

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
        init, // No init_if_needed as we don't want someone to reuse vault
        payer = maker,
        associated_token::mint = token_mint_a,
        associated_token::authority = offer, // offer account is the authority rather than a user. offer account will sign things to move in and out of the vault, which we will see later
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn send_offered_tokens_to_vault(
    context: &Context<MakeOffer>,
    token_a_offered_amount: u64,
) -> Result<()> {
    transfer_tokens(
        &context.accounts.maker_token_account_a, //from
        &context.accounts.vault,                 //to
        &token_a_offered_amount,                 //amount
        &context.accounts.token_mint_a,          //mint
        &context.accounts.maker,                 //authority
        &context.accounts.token_program,         //token program
    )
}

pub fn save_offer(context: Context<MakeOffer>, id: u64, token_a_offered_amount: u64) -> Result<()> {
    let token_b_wanted_amount = compute_token_b_wanted(
        token_a_offered_amount,
        context.accounts.price_of_token_a.price,
        context.accounts.price_of_token_b.price,
    )?;
    context.accounts.offer.set_inner(Offer {
        id,
        maker: context.accounts.maker.key(),
        token_mint_a: context.accounts.token_mint_a.key(),
        token_mint_b: context.accounts.token_mint_b.key(),
        token_b_wanted_amount,
        remaining_amount: token_a_offered_amount,
        bump: context.bumps.offer,
    });

    emit!(OfferCreated {
        offer_id: id,
        maker: context.accounts.maker.key(),
        token_mint_a: context.accounts.token_mint_a.key(),
        token_mint_b: context.accounts.token_mint_b.key(),
        offered_amount: token_a_offered_amount,
        wanted_amount: token_b_wanted_amount,
    });
    Ok(())
}
