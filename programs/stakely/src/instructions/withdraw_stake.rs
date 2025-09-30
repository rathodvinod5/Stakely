use anchor_lang::prelude::*;

use crate::states::{ Pool, StakeEntry };

pub fn withdraw_stake(ctx: Context<WithdrawStakeAmount>) -> Result<()> {
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