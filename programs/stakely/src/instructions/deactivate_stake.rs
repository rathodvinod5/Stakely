// use anchor_lang::prelude::*;
// use solana_program::{ 
//     stake::instruction::deactivate_stake,
//     program::invoke_signed,
// };

// use crate::states::{ Pool, StakeEntry, StakeStatus };
// use crate::errors::CustomErrors;

// // Admin: deactivate a stake account and withdraw its lamports back to reserve (keeper operation)
// // - This will call stake::deactivate_stake then after deactivation and when lamports released, withdraw to reserve
// pub fn deactivate_stake_account(ctx: Context<DeactivateStakeAccount>) -> Result<()> {
//     let pool = &mut ctx.accounts.pool;
//     let stake_entry = &mut ctx.accounts.stake_entry;
//     let stake_account = &ctx.accounts.stake_account;

//     require!(stake_entry.pool == pool.key(), CustomErrors::KeyMismatch);

//     let deactivate_ix = deactivate_stake(
//         &stake_account.key(), 
//         &pool.key()
//     );
//     let seeds = &[b"pool".as_ref(), &[pool.bump]];
//     let signer_seeds = &[&seeds[..]];

//     let result = invoke_signed(
//         &deactivate_ix, 
//         &[
//             stake_account.to_account_info(),
//             ctx.accounts.clock.to_account_info(),
//             ctx.accounts.stake_program.to_account_info(),
//         ], 
//         signer_seeds
//     );

//     if !pool.deactivating_stakes.contains(&stake_account.key()) {
//         pool.deactivating_stakes.push(stake_account.key());
//     }

//     stake_entry.status = StakeStatus::Deactivating;

//     Ok(())
// }

// #[derive(Accounts)]
// pub struct DeactivateStakeAccount<'info> {
//     #[account(mut)]
//     pub admin: Signer<'info>,

//     #[account(mut, has_one = admin)]
//     pub pool: Account<'info, Pool>,

//     #[account(mut)]
//     pub stake_account: AccountInfo<'info>,

//     #[account(mut, has_one = pool.key())]
//     pub stake_entry: Account<'info, StakeEntry>,

//     #[account(address = solana_program::stake::program::ID)]
//     pub stake_program: AccountInfo<'info>,

//     pub clock: Sysvar<'info, Clock>
// }