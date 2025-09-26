use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::states::{ Pool, StakeEntry };

// Deposit and delegate:
// - Creates a new stake account (system create + initialize)
// - Delegates stake account to the given validator_vote_pubkey
// - Records a StakeEntry account (PDA) to track it
// - Mints LST to depositor proportional to pool state
//
// NOTE: Caller must fund the transaction with the lamports they want to stake.
pub fn deposit_and_delegate(ctx: Context<DepositAndDelegate>) -> Result<()> {
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