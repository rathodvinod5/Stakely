use anchor_lang::prelude::*;

use crate::states::{ Pool, UnstakeTicket };
use crate::errors::{ CustomErrors };
  
// Admin function: process an UnstakeTicket (transfer lamports when available)
// - The admin or keeper will call this when sufficient liquid lamports exist in Reserve
pub fn process_unstake(ctx: Context<ProcessUnstake>) -> Result<()> {
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