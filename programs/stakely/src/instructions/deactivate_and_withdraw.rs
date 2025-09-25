use anchor_lang::prelude::*;

// Admin: deactivate a stake account and withdraw its lamports back to reserve (keeper operation)
// - This will call stake::deactivate_stake then after deactivation and when lamports released, withdraw to reserve
pub fn deactivate_and_withdraw(ctx: Context<DeactivateAndWithdraw>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct DeactivateAndWithdraw {

}