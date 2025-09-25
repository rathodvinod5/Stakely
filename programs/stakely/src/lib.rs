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

    pub fn initialize(ctx: Context<InitializePool>) -> Result<()> {
        initialize_pool(ctx);
        Ok(())
    }

    pub fn deposit(ctx: Context<DepositAndDelegate>) -> Result<()> {
        deposit_and_delegate(ctx);
        Ok(())
    }
}
