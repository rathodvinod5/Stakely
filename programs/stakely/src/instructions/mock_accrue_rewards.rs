use anchor_lang::{
    prelude::*,
    solana_program::{
        system_instruction::{ transfer },
        program:: { invoke }
    },
};

use crate::errors::{ CustomErrors };
use crate::states::{ Pool };

// Admin/keeper: mock_accrue_rewards
// - The admin first transfers lamports to the pool reserve (off-chain client does SystemProgram::transfer to PDA)
// - Then calls this instruction with amount to account for those lamports as "rewards" (adds to total_staked)
pub fn mock_accrue_rewards(ctx: Context<MockAccrueRewards>, reward_lamports: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let reserve = &ctx.accounts.reserve;

    // let reserve_lamports = reserve.lamports();
    // require!(reserve_lamports >= reward_lamports, CustomErrors::InsufficientBalance);
    require!(pool.admin == ctx.accounts.admin.key(), CustomErrors::NotTheOwner);

    let transfer_instr = transfer(
        &ctx.accounts.admin.key(), 
        &reserve.key(), 
        reward_lamports
    );
    let system_program = &ctx.accounts.system_program;
    let _ = invoke(
        &transfer_instr, 
        &[
            ctx.accounts.admin.to_account_info(),
            reserve.to_account_info(),
            system_program.to_account_info()
        ]
    );

    pool.total_staked = pool.total_staked
        .checked_add(reward_lamports as u128)
        .ok_or(CustomErrors::MathOverflow)?;

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