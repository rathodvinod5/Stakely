use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount};

use crate::{errors::CustomErrors, states::{ Pool, StakeEntry, StakeStatus, UnstakeTicket }};

// Admin function: process an UnstakeTicket (transfer lamports when available)
// - The admin or keeper will call this when sufficient liquid lamports exist in Reserve
pub fn request_unstake(ctx: Context<RequestUnstake>, unstake_token_lst_amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // Check LST balance
    let user_token_lst_balance = ctx.accounts.user_token_ata.amount;

    require!(user_token_lst_balance >= unstake_token_lst_amount, CustomErrors::InsufficientStakeAmount);
    // Check user stake (optional but safer)
    require!(ctx.accounts.stake_entry.status == StakeStatus::Active, CustomErrors::NoActiveStake);

    // Burn the unstake_token_lst_amount passed in the params
    let accounts = Burn {
        mint: ctx.accounts.lst_mint.to_account_info(),
        from: ctx.accounts.user_token_ata.to_account_info(),
        authority: ctx.accounts.user.to_account_info()
    };
    let token_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(token_program, accounts);
    let _ = token::burn(cpi_ctx, unstake_token_lst_amount);

    require!(pool.total_lst_minted > 0, CustomErrors::EmptyPool);

    // compute equivalent lamports the user should receive (pro rata)
    let unstake_token_lst_amount_u128 = unstake_token_lst_amount as u128;
    let lamports_eq = unstake_token_lst_amount_u128
        .checked_mul(pool.total_staked).ok_or(CustomErrors::MathOverflow)?
        .checked_div(pool.total_lst_minted).ok_or(CustomErrors::MathOverflow)?;
    
    // Update pool supply/accounting immediately (reflect burned LST)
    pool.total_lst_minted = pool.total_lst_minted
        .checked_add(unstake_token_lst_amount_u128).ok_or(CustomErrors::MathOverflow)?;
    pool.total_staked = pool.total_staked
        .checked_add(lamports_eq).ok_or(CustomErrors::MathOverflow)?;

    // Create unstake ticket
    let unstak_ticket = &mut ctx.accounts.unstake_ticket;
    unstak_ticket.pool = pool.key();
    unstak_ticket.requester = ctx.accounts.user.key();
    unstak_ticket.requested_lamports = lamports_eq;
    unstak_ticket.released = false;
    unstak_ticket.index = pool.unstaked_count;

    pool.unstaked_count = pool.unstaked_count.checked_add(1).unwrap_or(pool.unstaked_count);

    // The pool must later gather lamports from stake accounts / reserve to fulfill the ticket
    // Process_unstake will transfer lamports to the requester when available.
    Ok(())
}

#[derive(Accounts)]
pub struct RequestUnstake<'info> {
    #[account(mut, signer)]
    pub user: AccountInfo<'info>,

    // #[account(
    //     mut,
    //     seeds = [b"pool"],
    //     bump = pool.bump
    // )]
    #[account(mut)]
    pub pool: Account<'info, Pool>,

    // User’s LST token account
    #[account(
        mut,
        address = pool.lst_mint
    )]
    pub lst_mint: Account<'info, Mint>,

    // User’s LST token account
    #[account(
        mut,
        associated_token::mint = lst_mint,
        associated_token::authority = user,
    )]
    pub user_token_ata: Account<'info, TokenAccount>,

    // The stake entry associated with this user
    #[account(
        mut,
        constraint = stake_entry.pool == pool.key()
    )]
    pub stake_entry: Account<'info, StakeEntry>,

    // Unstake ticket created for this request
    #[account(
        init,
        payer = user,
        space = 8 + UnstakeTicket::INIT_SPACE,
        seeds = [b"unstake_ticket", pool.key().as_ref(), &pool.unstaked_count.to_le_bytes()],
        bump
    )]
    pub unstake_ticket: Account<'info, UnstakeTicket>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
}
