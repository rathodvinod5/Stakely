use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction:: { transfer };
use anchor_lang::solana_program::program::{ invoke_signed };
// use anchor_spl::token::{self, token::{self, Mint} };

use anchor_spl::{
    self,
    token::{self, Token },
};


use crate::states::{ Pool, UnstakeTicket };
use crate::errors::{ CustomErrors };
  
// Admin function: process an UnstakeTicket (transfer lamports when available)
// - The admin or keeper will call this when sufficient liquid lamports exist in Reserve
pub fn process_unstake(ctx: Context<ProcessUnstake>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let reserve_account = &mut ctx.accounts.reserve_account;
    let requester = &mut ctx.accounts.requester;
    let unstake_ticket = &mut ctx.accounts.unstake_ticket;

    // let reserve_lamports = **reserve_account.to_account_info().lamports.borrow();
    let reserve_lamports = reserve_account.lamports();
    let requested_lamports = unstake_ticket
        .requested_amount
        .try_into()
        .map_err(|_| error!(CustomErrors::MathOverflow))?;

    require!(
        reserve_lamports >= requested_lamports,
        CustomErrors::InsufficientBalance
    );
    require!(
        !unstake_ticket.is_released,
        CustomErrors::FundAlreadyReleased
    );
    let pool_key = pool.key();
    let seeds = &[
        b"pool-reserve".as_ref(),
        pool_key.as_ref(),
        &[pool.reserve_bump],
    ];
    let signers_seeds = &[&seeds[..]];
    let instruction = transfer(&reserve_account.key(), &requester.key(), requested_lamports);
    let account_infos = &[
        reserve_account.to_account_info(),
        requester.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
    ];
    let _ = invoke_signed(&instruction, account_infos, signers_seeds);

    // no sense of changing it's param, because by the end of this instruction
    // this account will be closed and the rent will be refunded to the admin
    // unstake_ticket.is_released = true;

    Ok(())
}

#[derive(Accounts)]
pub struct ProcessUnstake<'info> {
    #[account(mut)]
    pub requester: AccountInfo<'info>,

    #[account(
        mut,
        has_one = reserve_account,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"pool-reserve", pool.key().as_ref()],
        bump = pool.reserve_bump
    )]
    pub reserve_account: UncheckedAccount<'info>,

    #[account(
        mut,
        close = requester,
        // constraint = !unstake_ticket.is_released @ CustomErrors::FundAlreadyReleased,
        // seeds = [b"unstake-ticket", pool.key().as_ref(), &pool.unstaked_count.to_le_bytes()],
        // bump
    )]
    pub unstake_ticket: Account<'info, UnstakeTicket>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}