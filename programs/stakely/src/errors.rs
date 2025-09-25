use anchor_lang::prelude::*;

#[error_code]
pub enum CustomErrors {
    #[msg("No the owner")]
    NotTheOwner,
    #[msg("Insufficient balance")]
    InsufficientBalance,
    #[msg("Reserve out of balance")]
    ReserveOutOfBalance
}