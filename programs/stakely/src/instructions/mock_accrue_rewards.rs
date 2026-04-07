use anchor_lang::{
    prelude::*,
    solana_program::{
        system_instruction::{ transfer },
        program:: { invoke }
    },
};

use crate::errors::{ CustomErrors };
use crate::states::{ Pool };

// Admin/keeper: mock_accrue_rewards
// - The admin first transfers lamports to the pool reserve (off-chain client does SystemProgram::transfer to PDA)
// - Then calls this instruction with amount to account for those lamports as "rewards" (adds to total_staked)
pub fn mock_accrue_rewards(ctx: Context<MockAccrueRewards>, reward_amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let admin = &ctx.accounts.admin;
    let reserve_account = &ctx.accounts.reserve_account;

    require_keys_eq!(pool.admin.key(), admin.key(), CustomErrors::NotTheOwner);
    require!(reward_amount > 0, CustomErrors::InsufficientBalance);

    let admin_lamports = admin.to_account_info().lamports();
    require!(
        admin_lamports >= reward_amount,
        CustomErrors::InsufficientBalance
    );

    let instruction = transfer(&admin.key(), &reserve_account.key(), reward_amount);
    let account_infos = &[
        admin.to_account_info(),
        reserve_account.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ];

    let _ = invoke(&instruction, account_infos)?;

    pool.total_staked = pool.total_staked.checked_add(reward_amount.into()).unwrap();

    Ok(())
}

#[derive(Accounts)]
pub struct MockAccrueRewards<'info> {
    #[account(
        mut, 
        signer,
        constraint = admin.key() == pool.admin @ CustomErrors::NotTheOwner
    )]
    pub admin: AccountInfo<'info>,

    #[account(
        mut,
        has_one = reserve_account,
        has_one = admin
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"pool-reserve", pool.key().as_ref()],
        bump
    )]
    pub reserve_account: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}