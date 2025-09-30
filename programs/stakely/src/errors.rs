use anchor_lang::prelude::*;

#[error_code]
pub enum CustomErrors {
    #[msg("Math Overflow")]
    MathOverflow,
    #[msg("Not the owner")]
    NotTheOwner,
    #[msg("Insufficient balance")]
    InsufficientBalance,
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
    #[msg("")]
    KeyMismatch
}