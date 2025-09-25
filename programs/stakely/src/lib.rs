use anchor_lang::prelude::*;

declare_id!("BhHah6xCWoHUvE2gyqba5vSTTyRYHzyjjTsqXX55H4AV");

#[program]
pub mod stakely {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
