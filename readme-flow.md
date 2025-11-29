1.  What we’re building (reminder)
    A native-SOL → LST pool that:

    - Accepts SOL deposits (user-created stake accounts), delegates to validators,

      - Mints LST (liquid staking tokens) to users,

      - Tracks delegated stake via StakeEntrys,

      - Supports request_unstake → deactivation → withdraw → redeem flows,

      - Has an admin/keeper role for withdrawals and reward accrual,

      - Includes dev/testing helpers (mock_accrue_rewards, optional process_unstake automation).

2.  Instruction set we designed
    (Your list — real app + helpers)

    - initialize_pool — create Pool PDA, LST mint, reserve PDA, pool signer; set initial state.

    - deposit_and_delegate — user funds stake account (client creates & initializes), program records StakeEntry, mints LST, delegates or records for off-chain keeper to batch-delegate.

    - request_unstake — user burns LST (or marks desire), creates UnstakeTicket (or marks their StakeEntry), and triggers stake deactivation.

    - process_unstake — helper/optional keeper routine to check tickets and use reserve if available (we discussed this as optional).

    - mock_accrue_rewards — admin-only helper to transfer lamports to reserve and bump pool.total_staked (testing).

    - deactivate_stake (we split into two) — admin/keeper deactivates a stake account and records it in pool.deactivating_stakes.

    - withdraw_stake — admin/keeper withdraws lamports from a fully deactivated stake account into reserve (drains + closes stake account).

    - redeem / process_unstake payout — transfer SOL from reserve to user after ticket processed (burned LST already handled).

3.  Correct production sequence (what happens after request_unstake)

    - request_unstake (user) → burns LST (or marks), creates ticket, calls stake deactivation (or admin/keeper does deactivate_stake), and pool marks stake as deactivating.

    - Wait for epoch boundary (stake must become inactive).

    - withdraw_stake (admin/keeper) → verify stake is inactive, call stake program withdraw via invoke_signed with pool PDA, move funds into reserve PDA, mark StakeEntry as Withdrawn (remove from deactivating list).

    - process_unstake or redeem (program/admin) → fulfill UnstakeTicket(s) by transferring lamports from reserve to the user(s) (or batch payouts). Ticket closed/marked released.

    Notes:

    - If reserve had enough liquidity at step 1, process_unstake could pay out immediately without deactivation/withdraw.

    - mock_accrue_rewards is only for tests/dev to simulate yield and must both move lamports into reserve and update pool.total_staked.

4.  Safety / important checks we agreed on

    - Always verify the stake account is owned by Stake Program and has enough lamports (rent + stake amount).

    - Verify stake_amount parameter matches actual lamports in the stake account (prevent minting LST for missing funds).

    - withdraw_stake must check stake state (via StakeState decode + Clock) to ensure deactivation epoch passed; otherwise return clear error.

    - Use invoke_signed for any CPI that requires the Pool PDA to sign (mint_to, withdraw, etc.).

    - Use has_one constraints to enforce relationships (ticket → pool, pool → reserve, stake_entry → pool).
