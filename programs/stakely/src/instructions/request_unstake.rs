use anchor_lang::prelude::*;

// Admin function: process an UnstakeTicket (transfer lamports when available)
// - The admin or keeper will call this when sufficient liquid lamports exist in Reserve
pub fn request_unstake() -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct RequestUnstake {

}
