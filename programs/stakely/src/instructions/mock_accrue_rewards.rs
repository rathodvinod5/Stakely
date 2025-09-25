use anchor_lang::prelude::*;

// Admin/keeper: mock_accrue_rewards
// - The admin first transfers lamports to the pool reserve (off-chain client does SystemProgram::transfer to PDA)
// - Then calls this instruction with amount to account for those lamports as "rewards" (adds to total_staked)
pub fn mock_accrue_rewards(ctx: Context<MockAccrueRewards>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct MockAccrueRewards {

}