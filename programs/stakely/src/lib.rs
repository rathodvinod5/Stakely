use anchor_lang::prelude::*;

pub mod instructions;
use instructions::*;

pub mod states;
pub mod errors;

declare_id!("BhHah6xCWoHUvE2gyqba5vSTTyRYHzyjjTsqXX55H4AV");

#[program]
pub mod stakely {

    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        let _ = instructions::initialize_pool(ctx);
        Ok(())
    }

    pub fn deposit_and_delegate(ctx: Context<DepositAndDelegate>, stake_amount: u64) -> Result<()> {
        let _ = instructions::deposit_and_delegate(ctx, stake_amount);
        Ok(())
    }

    pub fn request_unstake(ctx: Context<RequestUnstake>, unstake_token_lst_amount: u64) -> Result<()> {
        let _ = instructions::request_unstake(ctx, unstake_token_lst_amount);
        Ok(())
    }

    pub fn process_unstake(ctx: Context<ProcessUnstake>) -> Result<()> {
        let _ = instructions::process_unstake(ctx);
        Ok(())
    }

    pub fn mock_accrue_rewards(ctx: Context<MockAccrueRewards>, reawrd_lamprts: u64) -> Result<()> {
        let _ = instructions::mock_accrue_rewards(ctx, reawrd_lamprts);
        Ok(())
    }

    pub fn deactivate_stake_account(ctx: Context<DeactivateStakeAccount>) -> Result<()> {
        let _ = instructions::deactivate_stake_account(ctx);
        Ok(())
    }
}
