use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub admin: Pubkey,
    pub reserve: Pubkey,
    pub lst_mint: Pubkey,
    pub bump: u8,
    pub total_staked: u128,
    pub total_list_mint: u128,
    pub staked_count: u64,
    pub unstaked_count: u64,
    pub lst_decimals: u8,
}

#[account]
#[derive(InitSpace)]
pub struct StakeEntry {
    pub pool: Pubkey,
    pub validator_voter: Pubkey,
    pub stake_account: Pubkey,
    pub deposited_lamports: u128,
    pub index: u64,
}

pub enum StakeStatus {
    Active,
    Deactive
}