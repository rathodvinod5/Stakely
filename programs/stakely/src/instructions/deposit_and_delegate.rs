use anchor_lang::prelude::*;

// Deposit and delegate:
// - Creates a new stake account (system create + initialize)
// - Delegates stake account to the given validator_vote_pubkey
// - Records a StakeEntry account (PDA) to track it
// - Mints LST to depositor proportional to pool state
//
// NOTE: Caller must fund the transaction with the lamports they want to stake.
pub fn deposit_and_delegate(ctx: Context<DepositAndDelegate>) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct DepositAndDelegate {

}