use anchor_lang::prelude::*;
use anchor_spl::token:: { Mint, Token };

use crate::states::{ Pool };

// Initialize the pool:
// - Creates Pool account (PDA)
// - Creates LST mint (SPL) with pool as mint authority (PDA)
// - Creates reserve PDA (System account) to hold liquid lamports
pub fn initialize_pool(ctx: Context<InitializePool>, lst_decimals: u8) -> Result<()> {
    // Initialize the mint to be controlled by pool PDA
    // (We use CPI outside - the Anchor macro already ensured mint account exists).
    let pool = &mut ctx.accounts.pool;
    let decimals = ctx.accounts.lst_mint.decimals;

    pool.admin = ctx.accounts.admin.key();
    pool.lst_mint = ctx.accounts.lst_mint.key();
    pool.reserve = ctx.accounts.reserve.key();
    pool.bump = ctx.bumps.pool;
    // pool.lst_decimals = lst_decimals;
    pool.lst_decimals = decimals;
    pool.total_staked = 0u128;
    pool.total_lst_minted = 0u128;
    pool.staked_count = 0;
    pool.unstaked_count = 0;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut, signer)]
    pub admin: AccountInfo<'info>,

    #[account(mut)]
    pub lst_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = admin,
        space = 8,
        seeds = [b"pool_reserve"],
        bump
    )]
    pub reserve: AccountInfo<'info>,

    #[account(
        seeds = [b"pool"],
        bump
    )]
    pub pool_signer: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_account: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}