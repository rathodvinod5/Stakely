use anchor_lang::prelude::*;

use crate::states::{ Pool };
// Admin/keeper: mock_accrue_rewards
// - The admin first transfers lamports to the pool reserve (off-chain client does SystemProgram::transfer to PDA)
// - Then calls this instruction with amount to account for those lamports as "rewards" (adds to total_staked)
pub fn mock_accrue_rewards(ctx: Context<MockAccrueRewards>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct MockAccrueRewards<'info> {
    #[account(mut, signer)]
    pub admin: AccountInfo<'info>,

    #[account(mut, has_one = reserve)]
    pub pool: Account<'info, Pool>,

    #[account(mut, seeds = [b"pool_reserve"], bump)]
    pub reserve: AccountInfo<'info>,

    pub system_program: Program<'info, System>
}