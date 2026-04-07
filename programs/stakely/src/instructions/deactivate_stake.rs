use anchor_lang::prelude::*;
use solana_program::{ 
    stake::instruction::deactivate_stake,
    program::invoke_signed,
};

use crate::states::{ Pool, StakeEntry, StakeStatus };
use crate::errors::CustomErrors;

// Admin: deactivate a stake account and withdraw its lamports back to reserve (keeper operation)
// - This will call stake::deactivate_stake then after deactivation and when lamports released, withdraw to reserve
pub fn deactivate_stake_account(ctx: Context<DeactivateStakeAccount>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let stake_entry = &mut ctx.accounts.stake_entry;
    let stake_account = &ctx.accounts.stake_account;

    require!(stake_entry.pool == pool.key(), CustomErrors::KeyMismatch);

    // ========================== not required ==========================
    // because we do not have real validator
    // let deactivate_ix = deactivate_stake(
    //     &stake_account.key(), 
    //     &pool.key()
    // );
    // let seeds = &[b"pool", pool.lst_mint.as_ref(), &[pool.bump]];
    // let signer_seeds = &[&seeds[..]];

    // let result = invoke_signed(
    //     &deactivate_ix, 
    //     &[
    //         stake_account.to_account_info(),
    //         ctx.accounts.clock.to_account_info(),
    //         pool.to_account_info(),
    //         ctx.accounts.stake_program.to_account_info(),
    //     ], 
    //     signer_seeds
    // )?;
    // ====================================================================

    stake_entry.stake_status = StakeStatus::Deactivating;

    msg!("Stake account deactivated: {}", stake_account.key());

    Ok(())
}

#[derive(Accounts)]
pub struct DeactivateStakeAccount<'info> {
    /// CHECK: must match pool.admin
    #[account(
        mut,  
        constraint = admin.key() == pool.admin @ CustomErrors::NotTheOwner
    )]
    pub admin: Signer<'info>,

    #[account(
        mut, 
        has_one = admin @ CustomErrors::NotTheOwner
    )]
    pub pool: Account<'info, Pool>,


    #[account(mut)]
    pub stake_account: UncheckedAccount<'info>,

    #[account(
        mut, 
        has_one = pool,
        constraint = stake_account.key() == stake_entry.stake_account @ CustomErrors::InvalidStakeAccount
    )]
    pub stake_entry: Account<'info, StakeEntry>
}