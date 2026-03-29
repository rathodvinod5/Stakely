use anchor_lang::{
    prelude::*,
    solana_program::{
        system_instruction::{ transfer },
        program:: { invoke }
    },
};
use anchor_spl::token::{Mint, Token};
// use anchor_spl::associated_token::AssociatedToken;
// use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};


use crate::states::Pool;
use crate::errors::CustomErrors;

// Initialize the pool:
// - Creates Pool account (PDA)
// - Creates LST mint (SPL) with pool as mint authority (PDA)
// - Creates reserve PDA (System account) to hold liquid lamports
pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {

    let pool = &mut ctx.accounts.pool;
    let lst_mint = &ctx.accounts.lst_mint;
    pool.admin = ctx.accounts.admin.key();
    pool.reserve_account = ctx.accounts.reserve_account.key();
    pool.lst_decimals = lst_mint.decimals;
    pool.lst_mint = lst_mint.key();
    pool.total_staked = 0 as u128;
    pool.total_lst_minted = 0 as u128;
    pool.staked_count = 0;
    pool.unstaked_count = 0;
    pool.bump = ctx.bumps.pool;
    pool.reserve_bump = ctx.bumps.reserve_account;
    // pool.deactivating_stake_accounts = Vec::new();
    // pool.deactivating_stake_accounts = vec![]; // Vec::<Pubkey>::new();

//    msg!("====> entered initialize_pool");
//     let pool = &mut ctx.accounts.pool;
//     let lst_mint = &ctx.accounts.lst_mint;
//     let admin = &ctx.accounts.admin;
//     let reserve_account = &ctx.accounts.reserve_account;

//     // manually derive and validate reserve PDA
//     let (expected_reserve_pda, reserve_bump) = Pubkey::find_program_address(
//         &[b"pool-reserve", pool.key().as_ref()],
//         ctx.program_id,
//     );
//     require!(
//         reserve_account.key() == expected_reserve_pda,
//         CustomErrors::KeyMismatch
//     );
//     msg!("Reserve PDA validated: {}", reserve_account.key());

//     // fund reserve account with rent exempt minimum
//     let rent_exempt = Rent::get()?.minimum_balance(0);
//     let reserve_balance = reserve_account.lamports();
//     if reserve_balance < rent_exempt {
//         invoke(
//             &transfer(
//                 &admin.key(),
//                 &reserve_account.key(),
//                 rent_exempt - reserve_balance,
//             ),
//             &[
//                 admin.to_account_info(),
//                 reserve_account.to_account_info(),
//                 ctx.accounts.system_program.to_account_info(),
//             ],
//         )?;
//         msg!("Reserve account funded: {} lamports", rent_exempt);
//     }

//     // store pool state
//     pool.admin = admin.key();
//     pool.reserve_account = reserve_account.key();
//     pool.lst_decimals = lst_mint.decimals;
//     pool.lst_mint = lst_mint.key();
//     pool.total_staked = 0;
//     pool.total_lst_minted = 0;
//     pool.staked_count = 0;
//     pool.unstaked_count = 0;
//     pool.bump = ctx.bumps.pool;
//     pool.reserve_bump = reserve_bump;               // ← use manually derived bump

//     msg!("Pool initialized successfully");
//     msg!("Pool PDA: {}", pool.key());
//     msg!("Reserve PDA: {}", reserve_account.key());
//     msg!("Reserve bump: {}", pool.reserve_bump);

    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {

    #[account(mut, signer)]
    pub admin: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + Pool::INIT_SPACE,
        seeds = [b"pool", lst_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = admin,
        space = 8,
        seeds = [b"pool-reserve", pool.key().as_ref()],
        bump
    )]
    pub reserve_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub lst_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
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

