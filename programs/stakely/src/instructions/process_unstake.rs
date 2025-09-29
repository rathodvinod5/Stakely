use anchor_lang::prelude::*;
use anchor_lang::system_program::transfer;
use solana_program::program::invoke_signed;
use solana_program::system_instruction;

use crate::states::{ Pool, UnstakeTicket };
use crate::errors::{ CustomErrors };
  
// Admin function: process an UnstakeTicket (transfer lamports when available)
// - The admin or keeper will call this when sufficient liquid lamports exist in Reserve
pub fn process_unstake(ctx: Context<ProcessUnstake>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let unstake_ticket = &mut ctx.accounts.unstake_ticket;
   
    require!(!unstake_ticket.released, CustomErrors::FundAlreadyReleased);

    // let reserve_lamports = ctx.accounts.reserve.get_lamports();
    let reserve_lamports = **ctx.accounts.reserve.to_account_info().lamports.borrow();
    // let requested_lamports = unstake_ticket.requested_lamports
    //     .try_into()
    //     .map_err(|_| error!(CustomErrors::MathOverflow))?;
    let requested_lamports = unstake_ticket.requested_lamports
            .try_into()
            .map_err(|_| error!(CustomErrors::MathOverflow))?;

    require!(requested_lamports >= reserve_lamports, CustomErrors::InsufficientBalance);

    // Transfer lamports from reserve PDA to user
    // Since reserve is owned by system program and PDA controlled by program, use invoke_signed with reserve seeds
    let seeds = [b"pool".as_ref(), &[pool.bump]];
    let signers_seeds = &[&seeds[..]];
    let system_program = &ctx.accounts.system_program;

    let instructions = system_instruction::transfer(
        &ctx.accounts.reserve.key(), 
        &unstake_ticket.requester.key(), 
        requested_lamports
    );
    let _ = invoke_signed(
        &instructions, 
        &[
            ctx.accounts.reserve.to_account_info(),
            ctx.accounts.requester.to_account_info(),
            system_program.to_account_info()
        ], 
        signers_seeds
    );

    unstake_ticket.released = true;

    Ok(())
}

#[derive(Accounts)]
pub struct ProcessUnstake<'info> {
    #[account(mut)]
    pub requester: AccountInfo<'info>,

    #[account(mut, has_one = reserve)]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"pool_reserve"],
        bump
    )]
    pub reserve: AccountInfo<'info>,

    #[account(
        mut,
        close = requester,
        constraint = !unstake_ticket.released @ CustomErrors::FundAlreadyReleased,
        seeds = [b"unstake_ticket", pool.key().as_ref(), &pool.unstaked_count.to_le_bytes()],
        bump
    )]
    pub unstake_ticket: Account<'info, UnstakeTicket>,

    pub system_program: Program<'info, System>
}