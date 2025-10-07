use crate::state::poll_state::PollAccount;
use crate::{constants::COMP_DEF_OFFSET_VOTE, error::ErrorCode, SignerAccount, ID, ID_CONST};
use arcium_client::idl::arcium::types::CallbackAccount;

use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

pub fn init_vote_comp_def_handler(ctx: Context<InitVoteCompDef>) -> Result<()> {
    init_comp_def(ctx.accounts, true, 0, None, None)?;
    Ok(())
}

pub fn vote_handler(
    ctx: Context<Vote>,
    computation_offset: u64,
    _id: u32,
    vote: [u8; 32],
    vote_encryption_pubkey: [u8; 32],
    vote_nonce: u128,
) -> Result<()> {
    let args = vec![
        Argument::ArcisPubkey(vote_encryption_pubkey),
        Argument::PlaintextU128(vote_nonce),
        Argument::EncryptedBool(vote),
        Argument::PlaintextU128(ctx.accounts.poll_acc.nonce),
        Argument::Account(
            ctx.accounts.poll_acc.key(),
            // Offset calculation: 8 bytes (discriminator) + 1 byte (bump)
            8 + 1,
            32 * 2, // 2 vote counters (yes/no), each stored as 32-byte ciphertext
        ),
    ];

    ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        None,
        vec![VoteCallback::callback_ix(&[CallbackAccount {
            pubkey: ctx.accounts.poll_acc.key(),
            is_writable: true,
        }])],
    )?;
    Ok(())
}

pub fn vote_callback_handler(
    ctx: Context<VoteCallback>,
    output: ComputationOutputs<VoteOutput>,
) -> Result<()> {
    let o = match output {
        ComputationOutputs::Success(VoteOutput { field_0 }) => field_0,
        _ => return Err(ErrorCode::AbortedComputation.into()),
    };

    ctx.accounts.poll_acc.vote_state = o.ciphertexts;
    ctx.accounts.poll_acc.nonce = o.nonce;

    let clock = Clock::get()?;
    let current_timestamp = clock.unix_timestamp;

    emit!(VoteEvent {
        timestamp: current_timestamp,
    });

    Ok(())
}

#[queue_computation_accounts("vote", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, _id: u32)]
pub struct Vote<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init_if_needed,
        space = 9,
        payer = payer,
        seeds = [&SIGN_PDA_SEED],
        bump,
        address = derive_sign_pda!(),
    )]
    pub sign_pda_account: Account<'info, SignerAccount>,
    #[account(
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Account<'info, MXEAccount>,
    #[account(
        mut,
        address = derive_mempool_pda!()
    )]
    /// CHECK: mempool_account, checked by the arcium program
    pub mempool_account: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_execpool_pda!()
    )]
    /// CHECK: executing_pool, checked by the arcium program
    pub executing_pool: UncheckedAccount<'info>,
    #[account(
        mut,
        address = derive_comp_pda!(computation_offset)
    )]
    /// CHECK: computation_account, checked by the arcium program.
    pub computation_account: UncheckedAccount<'info>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_VOTE)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(
        mut,
        address = derive_cluster_pda!(mxe_account)
    )]
    pub cluster_account: Account<'info, Cluster>,
    #[account(
        mut,
        address = ARCIUM_FEE_POOL_ACCOUNT_ADDRESS,
    )]
    pub pool_account: Account<'info, FeePool>,
    #[account(
        address = ARCIUM_CLOCK_ACCOUNT_ADDRESS,
    )]
    pub clock_account: Account<'info, ClockAccount>,
    pub system_program: Program<'info, System>,
    pub arcium_program: Program<'info, Arcium>,
    /// CHECK: Poll authority pubkey
    #[account(
        address = poll_acc.authority,
    )]
    pub authority: UncheckedAccount<'info>,
    #[account(
        seeds = [b"poll", authority.key().as_ref(), _id.to_le_bytes().as_ref()],
        bump = poll_acc.bump,
        has_one = authority
    )]
    pub poll_acc: Account<'info, PollAccount>,
}

#[init_computation_definition_accounts("vote", payer)]
#[derive(Accounts)]
pub struct InitVoteCompDef<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        address = derive_mxe_pda!()
    )]
    pub mxe_account: Box<Account<'info, MXEAccount>>,
    #[account(mut)]
    /// CHECK: comp_def_account, checked by arcium program.
    /// Can't check it here as it's not initialized yet.
    pub comp_def_account: UncheckedAccount<'info>,
    pub arcium_program: Program<'info, Arcium>,
    pub system_program: Program<'info, System>,
}

#[callback_accounts("vote")]
#[derive(Accounts)]
pub struct VoteCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_VOTE)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    #[account(mut)]
    pub poll_acc: Account<'info, PollAccount>,
}

#[event]
pub struct VoteEvent {
    pub timestamp: i64,
}
