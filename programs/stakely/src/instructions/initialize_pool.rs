use anchor_lang::prelude::*;
use anchor_spl::token:: { Mint, Token };

use crate::states::{ Pool };

// Initialize the pool:
// - Creates Pool account (PDA)
// - Creates LST mint (SPL) with pool as mint authority (PDA)
// - Creates reserve PDA (System account) to hold liquid lamports
pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut, signer)]
    pub admin: AccountInfo<'info>, // AccountInfo indicates it can be wallet, PDA or DAO

    // CHECK: mint created by client and passed in. We'll set pool.lst_mint to it.
    #[account(mut)]
    pub lst_mint: Account<'info, Mint>,

    // Pool PDA - created off-chain with seeds ["pool"].
    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool"],
        bump
    )]
    pub pool: Account<'info, Pool>,

    // Reserve PDA: program-derived system account to hold lamports for liquidity
    // We'll create it as a system account with seeds ["pool_reserve"]
    #[account(
        init,
        payer = admin,
        space = 8,
        seeds = [b"pool_reserve"],
        bump
    )]
    pub reserve: AccountInfo<'info>,

    // This is the signer PDA used to mint tokens (pool PDA)
    // CHECK: we use pool as signer for the mint CPI
    // In CPI we pass pool_signer as authority
    #[account(seeds = [b"pool"], bump)]
    pub pool_signer: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>
}