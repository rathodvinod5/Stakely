use anchor_lang::prelude::*;
// use solana_program::stake;
// use solana_stake_interface as stake;
use anchor_lang::solana_program::stake;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
// use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;

use crate::states::{ Pool, StakeEntry, StakeStatus };
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
    // require!(stake_account.owner == &Pubkey::from(crate::program::Stakely::id()), CustomErrors::NotTheOwner);


    let rent_exempt = Rent::get()?.minimum_balance(std::mem::size_of::<stake::state::StakeStateV2>());
    require!((stake_amount > 0) || (actual_stake_amount >= rent_exempt + stake_amount), CustomErrors::InsufficientStakeAmount);

    // System create account: create a new account owned by stake program
    // The client must include the `stake_account` as a signer (it will be a new ephemeral Keypair)
    // But in Anchor, to create a stake account, we usually create it off-chain and include it as signer.
    // For simpler flows we rely on the client to create the stake account Keypair and fund it with lamports.
    // (Anchor can't `create_account` into stake program with signer generated here.)
    //
    // For demonstration, ensure stake account has been created & initialized by client.
    // We'll perform the delegate CPI to stake program now:

    // Initialize stake account: we call stake::instruction::initialize (stake_authority = pool PDA)
    // But to keep simple, we set staker/withdrawer to pool PDA (so program can manage)
    // let stake_authority = pool.key();
    // let withdraw_authority = pool.key();

    // Note: we expect the stake account was created and contains `stake_amount` lamports.
    // Caller must create & fund it before calling this instruction (typical pattern).
    // Now delegate it:
    // let vote_pubkey = ctx.accounts.validator_vote.key();
    // let ix_delegate = stake::instruction::delegate_stake(
    //     &ctx.accounts.stake_account.key(),
    //     &stake_authority,
    //     &vote_pubkey,
    // );

    
    // transfer sol from stake_account to reserve account
    // **stake_account.try_borrow_mut_lamports()? -= stake_amount;
    // **reserve_account.try_borrow_mut_lamports()? += stake_amount;

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


    // We need to sign for authority: since stake_account's staker is pool PDA, delegate requires
    // pool PDA to sign as stake authority. But pool PDA is a program derived address and not a signer.
    // To perform delegation we must make the stake account's staker be a real keypair OR use stake_authorize to set authority.
    //
    // To keep this Anchor-only example workable, we'll assume the client created the stake account
    // with staker/withdrawer set to the client (depositor) and authorized to the pool via `stake_authorize` CPI.
    // This is complex in a single TX; to keep the example in scope, we'll instead:
    // - Accept that the stake account is already delegated by the client OR
    // - Use a simpler flow: deposit -> move lamports into pool reserve (liquidity) and mint LST,
    //   then off-chain keepers create stake accounts and delegate them from the reserve to validators.
    //
    // For this code example: we'll mint LST proportional to stake_amount and create a StakeEntry record.

    // Mint LST to user proportional to current exchange rate
    // Compute amount of LST to mint:
    let mut lst_mint_amount = 0u128;
    if pool.total_staked == 0 && pool.total_lst_minted == 0 {
        // First deposit: 1 LST per SOL (scaled by decimals)
        // lst = stake_amount * 10^lst_decimals
        lst_mint_amount = (stake_amount as u128)
            .checked_mul(10u128.pow(pool.lst_decimals as u32))
            .unwrap();
    } else {
        // lst = amount * (total_lst_supply / total_staked) or
        // lst_to_mint = stake_amount * total_lst_minted / total_staked
        lst_mint_amount = (stake_amount as u128)
            .checked_mul(pool.total_lst_minted).unwrap()
            .checked_div(pool.total_staked).unwrap();
    }

    // create instructions for mint lst_tokens to user_ata
    let accounts = MintTo { 
        mint: ctx.accounts.lst_mint.to_account_info(),
        to: ctx.accounts.user_token_ata.to_account_info(),
        authority: pool.to_account_info(),
    };
    let token_program = ctx.accounts.token_program.to_account_info();
    let seeds = &[b"pool".as_ref(), pool.lst_mint.as_ref(), &[pool.bump]];
    let signer_seeds = &[&seeds[..]];
    let mint_context = CpiContext::new_with_signer(token_program, accounts, signer_seeds);

    // safe to cast to u64 because decimals scale may make it big; ensure it fits
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
    stake_entry.status = StakeStatus::Active;
    stake_entry.index = pool.staked_count;

    // update pool.stake_count
    pool.staked_count = pool.staked_count.checked_add(1).unwrap_or(pool.staked_count);

    Ok(())
}

#[derive(Accounts)]
pub struct DepositAndDelegate<'info> {

    /// CHECK: This is the user depositing and delegating stake
    #[account(mut, signer)]
    pub user: AccountInfo<'info>,

    #[account(mut)]
    pub pool: Account<'info, Pool>,

    /// CHECK: This is the stake account to be delegated
    #[account(mut)]
    pub stake_account: AccountInfo<'info>,

    /// CHECK: This is the reserve account holding lamports
    #[account(mut)]
    pub reserve_account: AccountInfo<'info>,

    /// CHECK: This is the validator vote account to delegate stake to
    #[account(mut)]
    pub validator_vote: AccountInfo<'info>,

    #[account(
        init,
        payer = user,
        space = StakeEntry::LEN,
        seeds = [b"stake_entry", stake_account.key().as_ref()],
        bump
    )]
    pub stake_entry: Account<'info, StakeEntry>,

    #[account(mut)]
    pub lst_mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_token_ata: Account<'info, TokenAccount>,

    /// CHECK: This is the pool signer PDA
    // #[account(
    //     seeds = [b"pool", lst_mint.key().as_ref()],
    //     bump = pool.bump
    // )]
    // pub pool_signer: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}