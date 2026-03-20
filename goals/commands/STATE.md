## Step 50 (2026-03-20)

**Focus:** **`agcli stake move`** — docs (install + read path + exits), Phase 20 preflight (**validation + `check_spending_limit` on `--to`**), **`error.rs`** (`move amount` + hint), discoverability, tracker row.

**Done:**
- **Docs:** **`## stake move — Move alpha between subnets (same hotkey)`** in **`docs/commands/stake.md`** — discoverability, **`cargo install`** examples, read path vs **`StakeCommands::Move`** (**`validate_netuid`×2** → same-SN bail → **`validate_amount`(`move amount`)** → **`check_spending_limit(to, …)`** → unlock → **`move_stake_mev`**; no balance/slippage pre-read), exit table (**0** / **2** / **10** / **11** / **12** / **13** / **15** / **1** for **`from == to`**), e2e cross-ref; **`### stake move`** → anchor + on-chain one-liner.
- **E2E:** **`tests/e2e_test.rs`** Phase 20 — **`test_stake_move_preflight`** after **`test_stake_remove_preflight`**: **`validate_netuid(1)`**, **`validate_netuid(2)`**, **`validate_amount`**(**`move amount`**), **`check_spending_limit(2, amount)`**.
- **Discoverability:** **`docs/llm.txt`** Tier 1 “Move alpha SN→SN” + Stake table + detailed bullet; **`src/utils/explain.rs`** Phase 6 line + TIPS bullet + unit test **`owner_workflow_mentions_stake_move_preflight`**.
- **Errors:** **`src/error.rs`** — validation **12** substring **`move amount`**; **`hint`** → **`stake.md`** / **`stake move --help`**; unit test **`classify_move_amount_validation_hint`**.
- **Tracker:** **`goals/commands/COMMANDS_TESTED.md`** — row for **`stake move`**.

**Left:** Step 51 — e.g. **`stake swap`**; optional full **`e2e_local_chain`** Docker run.

**Tests run:** `cargo fmt --all`; `cargo test -p agcli error::tests::classify_move_amount_validation_hint --lib`; `cargo test -p agcli utils::explain::tests::owner_workflow_mentions_stake_move_preflight --lib`; `cargo test -p agcli --test e2e_test e2e_local_chain` **`--no-run`** — **passed**.

**Git:** Commit + push **project files only** for this step (excludes local edits to **`.github/workflows/ci.yml`** and **`tests/wallet_test.rs`** left unstaged).
