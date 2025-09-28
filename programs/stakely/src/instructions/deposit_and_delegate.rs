use anchor_lang::prelude::*;
// use anchor_spl::token_2022::MintTo;
// use solana_program::stake;
use solana_stake_interface as stake;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

use crate::states::{ Pool, StakeEntry };
use crate::errors::CustomErrors;

// Deposit and delegate:
// - Creates a new stake account (system create + initialize)
// - Delegates stake account to the given validator_vote_pubkey
// - Records a StakeEntry account (PDA) to track it
// - Mints LST to depositor proportional to pool state
//
// NOTE: Caller must fund the transaction with the lamports they want to stake.
pub fn deposit_and_delegate(ctx: Context<DepositAndDelegate>, stake_amount: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let stake_account = &mut ctx.accounts.stake_account;
    let reserve_account = &mut ctx.accounts.reserve_account;

    let actual_stake_amount = stake_account.lamports();

    require!(stake_account.owner == &Pubkey::from(solana_program::stake::program::ID), CustomErrors::NotTheOwner);

    let rent_exempt = Rent::get()?.minimum_balance(std::mem::size_of::<stake::state::StakeStateV2>());
    require!((stake_amount > 0) || (actual_stake_amount >= rent_exempt + stake_amount), CustomErrors::InsufficientStakeAmount);

    // transfer sol from stake_account to reserve account
    **stake_account.try_borrow_mut_lamports()? -= stake_amount;
    **reserve_account.try_borrow_mut_lamports()? += stake_amount;

    // ALTERNATE WAY TO TRANSFER SOL
    // =================================================================
    // let from_lamports = **stake_account.try_borrow_mut_lamports()?;
    // let to_lamports   = **reserve_account.try_borrow_mut_lamports()?;

    // // safe checked math, using u128 to avoid overflow
    // let amount_u128 = (stake_amount as u128);
    // let new_from = (from_lamports as u128).checked_sub(amount_u128)
    //     .ok_or(CustomErrors::InsufficientStakeAmount)? as u64;
    // let new_to = (to_lamports as u128).checked_add(amount_u128)
    //     .ok_or(CustomErrors::MathOverflow)? as u64;

    // // write back
    // **stake_account.try_borrow_mut_lamports()? = new_from;
    // **reserve_account.try_borrow_mut_lamports()?   = new_to;
    // ===================================================================
    // END OF ALTERNATIVE WAY


    // OR ANOTHER ALTERNATE WAY
    // ===================================================================
    // let amount = stake_amount.checked_mul(LAMPORTS_PER_SOL).ok_or(CustomErrors::MathOverflow)?;
    // **reserve_account.try_borrow_mut_lamports()? = (**reserve_account.try_borrow_mut_lamports()? as u128)
    // .checked_add(amount as u128)
    // .ok_or(CustomErrors::MathOverflow)? as u64;
    // **reserve_account.try_borrow_mut_lamports()? = (**reserve_account.try_borrow_mut_lamports()? as u128)
    // .checked_add(amount as u128)
    // .ok_or(CustomErrors::MathOverflow)? as u64;
    // ===================================================================
    // END OF ALTERNATIVE WAY


    // Calculate lst to mint to user ata
    let mut lst_mint_amount = 0u128;
    if pool.total_staked == 0 && pool.total_lst_minted == 0 {
        lst_mint_amount = (stake_amount as u128)
            .checked_mul(10u128.pow(pool.lst_decimals as u32))
            .unwrap();
    } else {
        lst_mint_amount = (stake_amount as u128)
            .checked_mul(pool.total_lst_minted).unwrap()
            .checked_div(pool.total_staked).unwrap();
    }

    // create instructions for mint lst_tokens to user_ata
    let accounts = MintTo { 
        mint: ctx.accounts.lst_mint.to_account_info(),
        to: ctx.accounts.user_token_ata.to_account_info(),
        authority: ctx.accounts.pool_signer.to_account_info(),
    };
    let token_program = ctx.accounts.token_program.to_account_info();
    let seeds = &[b"pool".as_ref(), &[pool.bump]];
    let signer_seeds = &[&seeds[..]];
    let mint_context = CpiContext::new_with_signer(token_program, accounts, signer_seeds);
    let mint_amount = lst_mint_amount
        .try_into()
        .map_err(|_| error!(CustomErrors::MathOverflow))?;

    let _ = token::mint_to(mint_context, mint_amount);

    // update the params of pool
    pool.total_staked = pool.total_staked.checked_add(stake_amount as u128).unwrap();
    pool.total_lst_minted = pool.total_lst_minted.checked_add(mint_amount as u128).unwrap();

    // create a stake entry
    let stake_entry = &mut ctx.accounts.stake_entry;
    stake_entry.pool = pool.key();
    stake_entry.validator_voter = ctx.accounts.validator_vote.key();
    stake_entry.stake_account = stake_account.key();
    stake_entry.deposited_lamports = stake_amount as u128;
    stake_entry.index = pool.staked_count;

    // update pool.stake_count
    pool.staked_count = pool.staked_count.checked_add(1).unwrap_or(pool.staked_count);

    Ok(())
}

#[derive(Accounts)]
pub struct DepositAndDelegate<'info> {
    #[account(mut, signer)]
    pub user: AccountInfo<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    #[account(mut)]
    pub stake_account: AccountInfo<'info>,

    #[account(mut)]
    pub reserve_account: AccountInfo<'info>,

    #[account(mut)]
    pub validator_vote: AccountInfo<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + StakeEntry::INIT_SPACE,
        seeds = [b"stake_entry", stake_account.key().as_ref()],
        bump
    )]
    pub stake_entry: Account<'info, StakeEntry>,

    #[account(mut)]
    pub lst_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"pool"],
        bump = pool.bump
    )]
    pub pool_signer: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}