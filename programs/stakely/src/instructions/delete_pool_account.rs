use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint };
use crate::{
    errors::CustomErrors, 
    states::{ Pool }
};

pub fn delete_pool_account(ctx: Context<DeletePoolAccount>) -> Result<()> {
    let reserve = &ctx.accounts.reserve_account;
    let admin = &ctx.accounts.admin;

    // -----------------------------------------
    // 1. Drain lamports from RESERVE → authority
    // -----------------------------------------
    let lamports = reserve.lamports();
    if lamports > 0 {
        **reserve.try_borrow_mut_lamports()? -= lamports;
        **admin.try_borrow_mut_lamports()? += lamports;
    }

    // ---------------------------------------------------
    // 2. Zero out reserve data + mark as system-owned
    // ---------------------------------------------------
    reserve.realloc(0, false)?;
    reserve.assign(&solana_program::system_program::ID);

    // pool_account auto-closes via `close = authority`
    Ok(())
}


#[derive(Accounts)]
pub struct DeletePoolAccount<'info> {
    // #[account(mut, signer)]
    // pub admin: UncheckedAccount<'info>,

    // #[account(
    //     mut,
    //     has_one = admin,
    //     has_one = lst_mint,
    //     close = admin,
    //     seeds = [b"pool", lst_mint.key().as_ref()],
    //     bump = pool.bump
    // )]
    // pub pool: Account<'info, Pool>,
    /// CHECK: Admin must sign AND must match pool_account.admin
    #[account(
        mut,
        signer,
        constraint = admin.key() == pool.admin @ CustomErrors::NotTheOwner,
    )]
    pub admin: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"pool", lst_mint.key().as_ref()],
        bump = pool.bump,
        close = admin,
        has_one = admin @ CustomErrors::NotTheOwner,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"pool_reserve", pool.key().as_ref()],
        bump,
        owner = crate::ID
    )]
    pub reserve_account: UncheckedAccount<'info>,

    #[account(
        mut,
        address = pool.lst_mint, 
    )]
    pub lst_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
}