use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::{errors::CustomErrors, states::{ Pool, StakeEntry, StakeStatus, UnstakeTicket }};

// Admin function: process an UnstakeTicket (transfer lamports when available)
// - The admin or keeper will call this when sufficient liquid lamports exist in Reserve
pub fn request_unstake(ctx: Context<RequestUnstake>, unstake_token_lst_amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let lst_mint = &ctx.accounts.lst_mint;
    let user_token_ata = &ctx.accounts.user_token_ata;
    let unstake_ticket = &mut ctx.accounts.unstake_ticket;

    let unstake_ata_lst_balance = user_token_ata.amount;
    require!(
        unstake_token_lst_amount > 0,
        CustomErrors::InvalidUnstakeAmount
    );
    require!(
        unstake_ata_lst_balance >= unstake_token_lst_amount,
        CustomErrors::InsufficientUserTokenBalance
    );
    require!(
        pool.total_lst_minted >= unstake_token_lst_amount.into(),
        CustomErrors::InvalidStakeAmount
    );
    require!(
        ctx.accounts.stake_entry.stake_status == StakeStatus::Active,
        CustomErrors::NoActiveStakes
    );

    let account_info = Burn {
        mint: lst_mint.to_account_info(),
        from: user_token_ata.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let system_program = ctx.accounts.system_program.to_account_info();
    let cpi_context = CpiContext::new(system_program, account_info);
    let _ = token::burn(cpi_context, unstake_token_lst_amount);

    let unstake_ata_lst_balance_u128 = unstake_ata_lst_balance as u128;
    let lamports_eq = unstake_ata_lst_balance_u128
        .checked_mul(pool.total_staked)
        .ok_or(CustomErrors::MathOverflow)?
        .checked_div(pool.total_lst_minted)
        .ok_or(CustomErrors::MathOverflow)?;

    unstake_ticket.pool = pool.key();
    unstake_ticket.requester = ctx.accounts.user.key();
    unstake_ticket.requested_amount = lamports_eq;
    unstake_ticket.is_released = false;
    unstake_ticket.index = pool.unstaked_count;

    // please clarify this again, because we need to perform subtraction, not addition
    pool.total_staked = pool
        .total_staked
        .checked_sub(lamports_eq)
        .ok_or(CustomErrors::MathOverflow)?;

    // please clarify this again, because we need to perform subtraction, not addition
    pool.total_lst_minted = pool
        .total_lst_minted
        .checked_sub(unstake_ata_lst_balance_u128)
        .ok_or(CustomErrors::MathOverflow)?;

    pool.unstaked_count = pool
        .unstaked_count
        .checked_add(1)
        .unwrap_or(pool.unstaked_count);

    Ok(())
}

#[derive(Accounts)]
pub struct RequestUnstake<'info> {
     #[account(mut, signer)]
    pub user: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = lst_mint,
        associated_token::authority = user,
    )]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        has_one = lst_mint
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        address = pool.lst_mint
    )]
    pub lst_mint: Account<'info, Mint>,

    #[account(
        mut,
        has_one = pool,
        constraint = pool.key() == stake_entry.pool,
    )]
    pub stake_entry: Account<'info, StakeEntry>,

    #[account(
        init,
        payer = user,
        space = 8 + UnstakeTicket::INIT_SPACE,
        seeds = [b"unstake-ticket", pool.key().as_ref(), &pool.unstaked_count.to_le_bytes()],
        bump
    )]
    pub unstake_ticket: Account<'info, UnstakeTicket>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}
