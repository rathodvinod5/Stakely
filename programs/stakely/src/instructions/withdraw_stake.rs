use anchor_lang::{prelude::*, system_program::{self, transfer}};
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

pub fn withdraw_stake_account(ctx: Context<WithdrawStakeAmount>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let stake_account = &ctx.accounts.stake_account;
    let stake_entry = &mut ctx.accounts.stake_entry;
    // let system_program = &ctx.accounts.system_program;

    // let clock: Clock = Clock::from_account_info(&ctx.accounts.clock_sysvar)?;

     // validate stake account matches stake entry
    require!(
        stake_account.key() == stake_entry.stake_account,
        CustomErrors::InvalidStakeAccount
    );

    // ========================== not required ==========================
    // // --- Load stake state ---
    // let stake_state: StakeStateV2 = StakeStateV2::deserialize(&mut &ctx.accounts.stake_account.data.borrow()[..])
    //     .map_err(|_| error!(CustomErrors::InvalidStakeState))?;

    // // --- Ensure stake is fully deactivated ---
    // match stake_state {
    //     StakeStateV2::Stake(_, stake, _stake_flags) => {
    //         require!(
    //             stake.delegation.deactivation_epoch != u64::MAX
    //                 && stake.delegation.deactivation_epoch <= clock.epoch,
    //             CustomErrors::StakeNotYetDeactivated
    //         );
    //     }
    //     _ => return Err(error!(CustomErrors::InvalidStakeState)),
    // }

    // require!(stake_entry.stake_status == StakeStatus::Deactivating, CustomErrors::StakeNotYetDeactivated);

    // // let all_lamports = stake_account.lamports();
    // // let withdraw_ix = withdraw(
    // //     &stake_account.key(),
    // //     &pool.key(),
    // //     &ctx.accounts.reserve.key(),
    // //     all_lamports,               // ← withdraw all lamports
    // //     None
    // // );

    // let withdraw_ix = withdraw( //solana_program::stake::instruction::withdraw
    //     &stake_account.key(), 
    //     &pool.key(), 
    //     &ctx.accounts.reserve_account.key(), 
    //     stake_entry.deposited_lamports.try_into().unwrap_or(u64::MAX), 
    //     None
    // );
    // let seeds = &[b"pool", pool.lst_mint.as_ref(), &[pool.bump]];
    // let signer_seeds = &[&seeds[..]];
    // let stake_program = &ctx.accounts.stake_program;
    // let result = invoke_signed( //solana_program::program::invoke_signed
    //     &withdraw_ix, 
    //     &[
    //         stake_account.to_account_info(),
    //         ctx.accounts.reserve_account.to_account_info(),
    //         ctx.accounts.clock_sysvar.to_account_info(),
    //         pool.to_account_info(),
    //         stake_program.to_account_info(),
    //         ctx.accounts.system_program.to_account_info()
    //     ], 
    //     signer_seeds
    // )?;
    // stake_entry.stake_status = StakeStatus::Deactive;
    // msg!("Stake withdrawn: {} lamports to reserve", all_lamports);
    // =====================================================================


    require!(
        stake_entry.stake_status == StakeStatus::Deactivating,
        CustomErrors::StakeNotYetDeactivated
    );

   require!(
        stake_entry.stake_status == StakeStatus::Deactivating,
        CustomErrors::StakeNotYetDeactivated
    );

    let stake_lamports = stake_account.lamports();
    require!(stake_lamports > 0, CustomErrors::InsufficientBalance);

    // pool PDA has withdraw authority over stake account
    // so we use pool PDA seeds to sign the transfer CPI
    let pool_key = pool.lst_mint;
    let seeds = &[
        b"pool".as_ref(),
        pool_key.as_ref(),
        &[pool.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // CPI to system program to transfer lamports
    // pool PDA signs as the withdraw authority
    let cpi_context = CpiContext::new_with_signer(
        ctx.accounts.system_program.to_account_info(),
        system_program::Transfer {
            from: ctx.accounts.stake_account.to_account_info(),
            to: ctx.accounts.reserve_account.to_account_info(),
        },
        signer_seeds,
    );

    stake_entry.stake_status = StakeStatus::Deactive;

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawStakeAmount<'info> {
    #[account(
        mut,
        constraint = admin.key() == pool.admin @ CustomErrors::NotTheOwner
    )]
    pub admin: Signer<'info>,

    #[account(
        mut, 
        has_one = admin @ CustomErrors::NotTheOwner,
        has_one = reserve_account @ CustomErrors::KeyMismatch
    )]
    pub pool: Account<'info, Pool>,

    #[account(mut, address = pool.reserve_account)]
    pub reserve_account: AccountInfo<'info>,

    #[account(mut)]
    pub stake_account: AccountInfo<'info>,

    #[account(mut, has_one = pool)]
    pub stake_entry: Account<'info, StakeEntry>,

    /// CHECK: validated via address constraint
    #[account(address = solana_program::stake::program::ID)]
    pub stake_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    // pub clock: Sysvar<'info, Clock>

    // // System clock (needed to check epoch/deactivation)
    // #[account(address = solana_program::sysvar::clock::id())]
    // pub clock_sysvar: AccountInfo<'info>,
}