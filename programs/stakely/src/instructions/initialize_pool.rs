use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};
// use anchor_spl::associated_token::AssociatedToken;
// use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};


use crate::states::Pool;

// Initialize the pool:
// - Creates Pool account (PDA)
// - Creates LST mint (SPL) with pool as mint authority (PDA)
// - Creates reserve PDA (System account) to hold liquid lamports
pub fn initialize_pool(ctx: Context<InitializePool>, lst_decimals: u8) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let decimals = ctx.accounts.lst_mint.decimals;

    pool.admin = ctx.accounts.admin.key();
    pool.lst_mint = ctx.accounts.lst_mint.key();
    pool.reserve = ctx.accounts.reserve.key();
    pool.lst_decimals = decimals;
    pool.total_staked = 0 as u128;
    pool.total_lst_minted = 0 as u128;
    pool.staked_count = 0;
    pool.unstaked_count = 0;
    pool.bump = ctx.bumps.pool;
    pool.deactivating_stake_accounts = Vec::new();

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {

    /// CHECK: This is the admin initializing the pool
    #[account(mut, signer)]
    pub admin: AccountInfo<'info>,

    /// CHECK: This is the LST mint account
    #[account(mut)]
    pub lst_mint: Account<'info, Mint>,

    /// CHECK: This is the pool account
    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool", lst_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    /// CHECK: This is a PDA vault account used only to hold SOL. No data is accessed.
    #[account(
        init,
        payer = admin,
        space = 8,
        seeds = [b"pool_reserve", pool.key().as_ref()],
        bump
    )]
    pub reserve: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,

    // pub token_account: InterfaceAccount<'info, TokenAccount>,
    // pub lst_mint: InterfaceAccount<'info, Mint>,
    // pub token_program: Interface<'info, TokenInterface>,
    // // pub associated_token_program: Program<'info, AssociatedToken>,
    // pub system_program: Program<'info, System>,
    // pub rent: Sysvar<'info, Rent>,
}


// step1)  create token
//      - spl-token create-token
// step 2) set mint authority
//      - spl-token authorize <LST_MINT_PUBKEY> mint <POOL_PDA_PUBKEY>
// step 3) set freeze authority (optional but recommended)
//      - spl-token authorize <LST_MINT_PUBKEY> freeze <POOL_PDA_PUBKEY>


// // Query all the pool accounts
// import { utils } from "@coral-xyz/anchor";
// // get discriminator of the Pool account
// const discriminator = utils.sha256.digest("account:Pool").slice(0, 8);

// const pools = await connection.getProgramAccounts(PROGRAM_ID, {
//   filters: [
//     {
//       memcmp: {
//         offset: 0,
//         bytes: Buffer.from(discriminator).toString("hex"),
//       }
//     }
//   ]
// });

