use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub admin: Pubkey,
    pub reserve: Pubkey,
    pub lst_mint: Pubkey,
    pub bump: u8,
    pub total_staked: u128,
    pub total_lst_mint: u128,
    pub staked_count: u64,
    pub unstaked_count: u64,
    pub lst_decimals: u8,
}