## Step 51 (2026-03-20)

**Focus:** **`agcli stake swap`** — docs (install + read path + exits, fix wrong `--from-hotkey`/`--to-hotkey` stub), Phase 20 preflight (same order as **`StakeCommands::Swap`**), **`error.rs`** (`swap amount` + hint), discoverability, tracker row; **`mod.rs`** help text aligned with **`stake move`** (alpha amount).

**Done:**
- **Docs:** **`## stake swap — Swap alpha between subnets (same hotkey)`** in **`docs/commands/stake.md`** — mirrors **`move`** pre-wallet order (**`validate_netuid`×2** → same-SN bail → **`validate_amount`(`swap amount`)** → **`check_spending_limit(to)`** → unlock → **`swap_stake_mev`**), exit table, e2e cross-ref; **`### stake swap`** → anchor + on-chain note (**`swap_stake`** vs **`move_stake`**).
- **E2E:** **`tests/e2e_test.rs`** Phase 20 — **`test_stake_swap_preflight`** after **`test_stake_move_preflight`**: same RPC sequence with **`swap amount`**.
- **Discoverability:** **`docs/llm.txt`** Tier 1 “Swap alpha SN→SN (AMM)” + Stake table + staking bullet; **`src/utils/explain.rs`** Phase 6 line + TIPS + **`owner_workflow_mentions_stake_swap_preflight`**.
- **Errors:** **`src/error.rs`** — validation **12** substring **`swap amount`**; hint → **`stake.md`** / **`stake swap --help`**; **`classify_swap_amount_validation_hint`**.
- **CLI help:** **`src/cli/mod.rs`** — **`Swap`** amount doc comment (**alpha**, notes **`Balance::from_tao`** parity with **`move`**).
- **Tracker:** **`goals/commands/COMMANDS_TESTED.md`** — row for **`stake swap`**.

**Left:** Step 52 — e.g. **`stake unstake-all`** or **`stake swap-limit`**; optional full **`e2e_local_chain`** Docker run.

**Tests run:** `cargo fmt --all`; `cargo test -p agcli error::tests::classify_swap_amount_validation_hint --lib`; `cargo test -p agcli utils::explain::tests::owner_workflow_mentions_stake_swap_preflight --lib`; `cargo test -p agcli --test e2e_test e2e_local_chain` **`--no-run`** — **passed**.

**Git:** Commit + push **project files only** (excludes **`.github/workflows/ci.yml`**, **`tests/wallet_test.rs`**).
