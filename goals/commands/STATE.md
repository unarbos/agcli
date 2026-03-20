## Step 48 (2026-03-20)

**Focus:** **`agcli stake remove`** — docs (install + read path + exits), Phase 20 RPC/validation preflight, **`error.rs`** (`unstake amount` + hint order vs `stake amount`), discoverability, tracker row.

**Done:**
- **Docs:** **`## stake remove — Unstake alpha to free TAO (wallet)`** in **`docs/commands/stake.md`** — discoverability, **`cargo install`** examples, read path aligned with **`StakeCommands::Remove`** (**`validate_netuid`** → **`validate_amount`(`unstake amount`)** → **`unlock`** → optional **`check_slippage`** sell path **`try_join`(`current_alpha_price`, `sim_swap_alpha_for_tao`)** → **`remove_stake_mev`**), exit table (**0** / **2** / **10** / **11** / **12** / **13** / **15** / **1**), e2e cross-ref; **`### stake remove`** → anchor + on-chain one-liner.
- **E2E:** **`tests/e2e_test.rs`** Phase 20 — **`test_stake_remove_preflight`** after **`test_stake_add_preflight`**: **`validate_netuid`**(1) + **`validate_amount`**(**`unstake amount`**) + **`try_join!(current_alpha_price, sim_swap_alpha_for_tao)`**.
- **Discoverability:** **`docs/llm.txt`** Tier 1 Unstake line + Stake table + Staking bullet; **`src/utils/explain.rs`** Phase 6 line + TIPS bullet.
- **Errors:** **`src/error.rs`** — validation **12** for **`unstake amount`**; **`hint`**: check **`unstake amount` before `stake amount`** (substring trap); unit test **`classify_unstake_amount_validation_hint`**.
- **Tracker:** **`goals/commands/COMMANDS_TESTED.md`** — row for **`stake remove`**.

**Left:** Step 49 — e.g. **`view portfolio`** or **`stake move`**; optional full **`e2e_local_chain`** Docker run.

**Tests run:** `cargo fmt --all`; `cargo test -p agcli error::tests --lib`; `cargo test -p agcli utils::explain::tests --lib`; `cargo test -p agcli --test e2e_test e2e_local_chain` **`--no-run`** — **passed**.

**Git:** Commit + **`git push origin main`** this step (project paths + tracker + STATE).
