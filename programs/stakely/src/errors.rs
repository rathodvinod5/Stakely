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
    #[msg("Insufficient stake amount")]
    InsufficientStakeAmount
}