use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;

pub mod states;
pub mod errors;

declare_id!("BhHah6xCWoHUvE2gyqba5vSTTyRYHzyjjTsqXX55H4AV");

#[program]
pub mod stakely {
    use crate::instructions::initialize_pool;

    use super::*;

    pub fn initialize(ctx: Context<InitializePool>, lst_decimals: u8) -> Result<()> {
        initialize_pool(ctx, lst_decimals);
        Ok(())
    }

    pub fn deposit_and_delegate(ctx: Context<DepositAndDelegate>, stake_amount: u64) -> Result<()> {
        deposit_and_delegate(ctx, stake_amount);
        Ok(())
    }

    pub fn request_unstake(ctx: Context<RequestUnstake>) -> Result<()> {
        request_unstake(ctx);
        Ok(())
    }

    pub fn process_unstake(ctx: Context<ProcessUnstake>) -> Result<()> {
        Ok(())
    }

    pub fn mock_accrue_rewards(ctx: Context<MockAccrueRewards>) -> Result<()> {
        mock_accrue_rewards(ctx);
        Ok(())
    }

    pub fn deactivate_and_withdraw(ctx: Context<DeactivateAndWithdraw>) -> Result<()> {
        deactivate_and_withdraw(ctx);
        Ok(())
    }
}
