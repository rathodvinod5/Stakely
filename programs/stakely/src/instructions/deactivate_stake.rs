use anchor_lang::prelude::*;

use crate::states::{ Pool, StakeEntry };

// Admin: deactivate a stake account and withdraw its lamports back to reserve (keeper operation)
// - This will call stake::deactivate_stake then after deactivation and when lamports released, withdraw to reserve
pub fn deactivate_stake_account(ctx: Context<DeactivateStakeAccount>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct DeactivateStakeAccount<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, has_one = admin)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub stake_account: AccountInfo<'info>,

    #[account(mut, has_one = pool.key())]
    pub stake_entry: Account<'info, StakeEntry>,

    #[account(address = solana_program::stake::program::ID)]
    pub system_program: AccountInfo<'info>,
    
    pub clock: Sysvar<'info, Clock>
}