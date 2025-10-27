use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::error::DexError;

pub fn transfer_tokens<'info>(
    from: &InterfaceAccount<'info, TokenAccount>,
    to: &InterfaceAccount<'info, TokenAccount>,
    amount: &u64,
    mint: &InterfaceAccount<'info, Mint>,
    authority: &Signer<'info>,
    token_program: &Interface<'info, TokenInterface>,
) -> Result<()> {
    let transfer_accounts_options = TransferChecked {
        from: from.to_account_info(),
        mint: mint.to_account_info(),
        to: to.to_account_info(),
        authority: authority.to_account_info(),
    };

    let cpi_context = CpiContext::new(token_program.to_account_info(), transfer_accounts_options);

    transfer_checked(cpi_context, *amount, mint.decimals)
}

pub fn compute_token_b_wanted(
    token_a_offered_account: u64,
    price_a: u64,
    price_b: u64,
) -> Result<u64> {
    // widen to u128 for safe intermediate multiplication
    let offered = token_a_offered_account;

    if price_b == 0 {
        return Err(DexError::PriceNotSet.into());
    }

    if price_a == 0 {
        return Err(DexError::PriceNotSet.into());
    }

    // multiply, checking overflow
    let product = offered.checked_mul(price_a).ok_or(DexError::Overflow)?;

    // divide, checking overflow (division itself won't overflow, but keep pattern)
    let token_b_wanted = product.checked_div(price_b).ok_or(DexError::Overflow)?;

    Ok(token_b_wanted)
}
