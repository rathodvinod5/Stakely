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
    #[max_len(1024)]
    pub deactivating_stakes: Vec<Pubkey>,
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


#[account]
// #[derive(
//     AnchorSerialize,
//     AnchorDeserialize,
//     Clone,
//     Copy,
//     PartialEq,
//     Eq,
// )]
pub struct StakeEntry {
    pub pool: Pubkey,
    pub stake_account: Pubkey,
    pub validator_voter: Pubkey,
    pub deposited_lamports: u128,
    pub status: StakeStatus,
    pub index: u64,
}

// Optional but recommended: explicit space calculation
impl StakeEntry {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 16 + 1 + 8;
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StakeStatus {
    Active = 0,
    Deactivating = 1,
    Deactive = 2,
}

// Tell Anchor how many bytes this enum will occupy when calculating account space
impl anchor_lang::Space for StakeStatus {
    // Anchor expects this associated constant for size calculation
    const INIT_SPACE: usize = 1;
}