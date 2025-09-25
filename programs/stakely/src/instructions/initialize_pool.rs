use anchor_lang::prelude::*;

// Initialize the pool:
// - Creates Pool account (PDA)
// - Creates LST mint (SPL) with pool as mint authority (PDA)
// - Creates reserve PDA (System account) to hold liquid lamports
pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    pub authority: Signer<'info>
}