use crate::state::poll_state::PollAccount;
use crate::{
    constants::COMP_DEF_OFFSET_INIT_VOTE_STATS, error::ErrorCode, SignerAccount, ID, ID_CONST,
};
use arcium_client::idl::arcium::types::CallbackAccount;

use anchor_lang::prelude::*;
use arcium_anchor::prelude::*;

pub fn init_vote_stats_comp_def_handle(ctx: Context<InitVoteStatsCompDef>) -> Result<()> {
    init_comp_def(ctx.accounts, true, 0, None, None)?;
    Ok(())
}

pub fn create_new_poll(
    ctx: Context<CreateNewPoll>,
    computation_offset: u64,
    id: u32,
    price: u64,
    mint0: Pubkey,
    mint1: Pubkey,
    nonce: u128,
) -> Result<()> {
    msg!("Creating a new poll");

    // Initialize the poll account with the provided parameters
    ctx.accounts.poll_acc.price = price;
    ctx.accounts.poll_acc.mint0 = mint0;
    ctx.accounts.poll_acc.mint1 = mint1;
    ctx.accounts.poll_acc.bump = ctx.bumps.poll_acc;
    ctx.accounts.poll_acc.id = id;
    ctx.accounts.poll_acc.authority = ctx.accounts.payer.key();
    ctx.accounts.poll_acc.nonce = nonce;
    ctx.accounts.poll_acc.vote_state = [[0; 32]; 7];

    let args = vec![Argument::PlaintextU128(nonce)];

    ctx.accounts.sign_pda_account.bump = ctx.bumps.sign_pda_account;

    // Initialize encrypted vote counters (yes/no) through MPC
    queue_computation(
        ctx.accounts,
        computation_offset,
        args,
        None,
        vec![InitVoteStatsCallback::callback_ix(&[CallbackAccount {
            pubkey: ctx.accounts.poll_acc.key(),
            is_writable: true,
        }])],
    )?;

    Ok(())
}

#[init_computation_definition_accounts("init_vote_stats", payer)]
#[derive(Accounts)]
pub struct InitVoteStatsCompDef<'info> {
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

#[queue_computation_accounts("init_vote_stats", payer)]
#[derive(Accounts)]
#[instruction(computation_offset: u64, id: u32)]
pub struct CreateNewPoll<'info> {
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
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_VOTE_STATS)
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
    #[account(
        init,
        payer = payer,
        space = 8 + PollAccount::INIT_SPACE,
        seeds = [b"poll", payer.key().as_ref(), id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll_acc: Account<'info, PollAccount>,
}

#[callback_accounts("init_vote_stats")]
#[derive(Accounts)]
pub struct InitVoteStatsCallback<'info> {
    pub arcium_program: Program<'info, Arcium>,
    #[account(
        address = derive_comp_def_pda!(COMP_DEF_OFFSET_INIT_VOTE_STATS)
    )]
    pub comp_def_account: Account<'info, ComputationDefinitionAccount>,
    #[account(address = ::anchor_lang::solana_program::sysvar::instructions::ID)]
    /// CHECK: instructions_sysvar, checked by the account constraint
    pub instructions_sysvar: AccountInfo<'info>,
    /// CHECK: poll_acc, checked by the callback account key passed in queue_computation
    #[account(mut)]
    pub poll_acc: Account<'info, PollAccount>,
}

pub fn init_vote_stats_callback_handler(
    ctx: Context<InitVoteStatsCallback>,
    output: ComputationOutputs<InitVoteStatsOutput>,
) -> Result<()> {
    let o = match output {
        ComputationOutputs::Success(InitVoteStatsOutput { field_0 }) => field_0,
        _ => return Err(ErrorCode::AbortedComputation.into()),
    };

    ctx.accounts.poll_acc.vote_state = o.ciphertexts;
    ctx.accounts.poll_acc.nonce = o.nonce;

    Ok(())
}
