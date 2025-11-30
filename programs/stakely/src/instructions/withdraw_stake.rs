use anchor_lang::prelude::*;
use solana_program::{
    stake::{
        state::{ StakeStateV2 },
        instruction:: { withdraw }
    },
    clock::Clock,
    // sysvar::Sysvar,
    program:: { invoke_signed }
};
use crate::states::{ Pool, StakeEntry, StakeStatus };
use crate::errors::{ CustomErrors };

pub fn withdraw_stake(ctx: Context<WithdrawStakeAmount>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let stake_account = &ctx.accounts.stake_account;
    let stake_entry = &mut ctx.accounts.stake_entry;

    let clock: Clock = Clock::from_account_info(&ctx.accounts.clock_sysvar)?;

    // --- Load stake state ---
    let stake_state: StakeStateV2 = StakeStateV2::deserialize(&mut &ctx.accounts.stake_account.data.borrow()[..])
        .map_err(|_| error!(CustomErrors::InvalidStakeState))?;

    // --- Ensure stake is fully deactivated ---
    match stake_state {
        StakeStateV2::Stake(_, stake, _stake_flags) => {
            require!(
                stake.delegation.deactivation_epoch != u64::MAX
                    && stake.delegation.deactivation_epoch <= clock.epoch,
                CustomErrors::StakeNotYetDeactivated
            );
        }
        _ => return Err(error!(CustomErrors::InvalidStakeState)),
    }

    require!(pool.deactivating_stake_accounts.contains(&stake_account.key()), CustomErrors::InvalidStakeAccount);
    require!(stake_entry.status == StakeStatus::Deactivating, CustomErrors::StakeNotYetDeactivated);

    let withdraw_ix = withdraw( //solana_program::stake::instruction::withdraw
        &stake_account.key(), 
        &pool.key(), 
        &ctx.accounts.reserve.key(), 
        stake_entry.deposited_lamports.try_into().unwrap_or(u64::MAX), 
        None
    );
    let seeds = &[b"pool", pool.lst_mint.as_ref(), &[pool.bump]];
    let signer_seeds = &[&seeds[..]];
    let stake_program = &ctx.accounts.stake_program;
    let result = invoke_signed( //solana_program::program::invoke_signed
        &withdraw_ix, 
        &[
            stake_account.to_account_info(),
            ctx.accounts.reserve.to_account_info(),
            ctx.accounts.clock_sysvar.to_account_info(),
            pool.to_account_info(),
            stake_program.to_account_info(),
            ctx.accounts.system_program.to_account_info()
        ], 
        signer_seeds
    );

    pool.deactivating_stake_accounts.retain(|pub_key| pub_key != &stake_account.key());

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
    // pub clock: Sysvar<'info, Clock>
    /// System clock (needed to check epoch/deactivation)
    #[account(address = solana_program::sysvar::clock::id())]
    pub clock_sysvar: AccountInfo<'info>,
}