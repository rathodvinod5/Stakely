use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Pool {
    pub admin: Pubkey,
    pub reserve: Pubkey,
    pub lst_mint: Pubkey,
    pub bump: u8,
    pub total_staked: u128,
    pub total_lst_minted: u128,
    pub staked_count: u64,
    pub unstaked_count: u64,
    pub lst_decimals: u8,
    #[max_len(1024)] // Set the maximum length as appropriate for your use case
    pub deactivating_stakes: Vec<Pubkey>,
}

#[account]
#[derive(InitSpace)]
pub struct StakeEntry {
    pub pool: Pubkey,
    pub validator_voter: Pubkey,
    pub stake_account: Pubkey,
    pub deposited_lamports: u128,
    pub status: StakeStatus,
    pub index: u64,
}

#[account]
#[derive(InitSpace)]
pub struct UnstakeTicket {
    pub pool: Pubkey,
    pub requester: Pubkey,
    pub requested_lamports: u128,
    pub released: bool,
    pub index: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, PartialEq, Eq, Debug)]
pub enum StakeStatus {
    Active,
    Deactivating,
    Deactive,
}