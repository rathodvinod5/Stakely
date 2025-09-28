use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::states::{ Pool, StakeEntry, UnstakeTicket };

// Admin function: process an UnstakeTicket (transfer lamports when available)
// - The admin or keeper will call this when sufficient liquid lamports exist in Reserve
pub fn request_unstake() -> Result<()> {
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
