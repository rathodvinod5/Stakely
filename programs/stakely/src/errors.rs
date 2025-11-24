use anchor_lang::prelude::*;

#[error_code]
pub enum CustomErrors {
    #[msg("Math Overflow")]
    MathOverflow,
    #[msg("Not the owner")]
    NotTheOwner,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Insufficient token record")]
    InsufficientTokensRecord,
    #[msg("Reserve out of balance")]
    ReserveOutOfBalance,
    #[msg("Insufficient user token balance")]
    InsufficientUserTokenBalance,
    #[msg("Insufficient stake amount")]
    InsufficientStakeAmount,
    #[msg("No active stake")]
    NoActiveStake,
    #[msg("Empty pool")]
    EmptyPool,
    #[msg("Fund already released!")]
    FundAlreadyReleased,
    #[msg("Key mismatch")]
    KeyMismatch,
    #[msg("Invalid stake account")]
    InvalidStakeAccount,
    #[msg("Stake account not deactivated!")]
    StakeNotYetDeactivated,
    #[msg("Invalid stake state")]
    InvalidStakeState
}