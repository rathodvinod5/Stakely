use anchor_lang::prelude::*;
// use solana_stake_interface::state::Stake;
// use solana_program::stake::state::Stake;

use crate::states::{ Pool, StakeEntry, StakeStatus };
use crate::errors::{ CustomErrors };

pub fn withdraw_stake(ctx: Context<WithdrawStakeAmount>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let stake_account = &ctx.accounts.stake_account;
    let stake_entry = &mut ctx.accounts.stake_entry;
    let reserve_account = &ctx.accounts.reserve;

    require!(pool.deactivating_stakes.contains(&stake_account.key()), CustomErrors::InvalidStakeAccount);
    require!(stake_entry.status == StakeStatus::Deactivating, CustomErrors::StakeNotDeactivated);

    let withdraw_ix = solana_program::stake::instruction::withdraw(
        &stake_account.key(), 
        &pool.key(), 
        &ctx.accounts.reserve.key(), 
        stake_entry.deposited_lamports.try_into().unwrap_or(u64::MAX),
        None, // No additional signers
    );

    let seeds = &[b"pool".as_ref(), &[pool.bump]];
    let signer_seeds = &[&seeds[..]];

    let result = solana_program::program::invoke_signed(
        &withdraw_ix, 
        &[
            stake_account.to_account_info(),
            reserve_account.to_account_info(),
            pool.to_account_info(),
            ctx.accounts.stake_program.to_account_info(),
            ctx.accounts.system_program.to_account_info()
        ], 
        signer_seeds
    );

    pool.deactivating_stakes.retain(|public_key| public_key != &stake_account.key());
    stake_entry.status = StakeStatus::Deactive;

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawStakeAmount<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, has_one = admin)]
    pub pool: Account<'info, Pool>,

    #[account(mut, address = pool.reserve)]
    pub reserve: AccountInfo<'info>,

    #[account(mut)]
    pub stake_account: AccountInfo<'info>,

    #[account(mut, has_one = pool)]
    pub stake_entry: Account<'info, StakeEntry>,

    #[account(address = solana_program::stake::program::ID)]
    pub stake_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>
}