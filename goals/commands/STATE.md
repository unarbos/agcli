## Step 45 (2026-03-20)

**Focus:** **`agcli transfer`** / **`transfer-all`** / **`transfer-keep-alive`** — docs accuracy, e2e preflight, exit/hints, discoverability.

**Done:**
- **Docs:** Rewrote **`docs/commands/transfer.md`** — discoverability (`--help`, `llm.txt`, `explain`), correct pallets (**`transfer_allow_death`** vs **`transfer_keep_alive`** vs **`transfer_all`**), read path aligned with **`commands.rs`**, JSON (**`print_tx_result`**), **`--dry-run`**, exit table (**0** / **2** / **10** / **11** / **12** / **13** / **15** / **1**), e2e Phase 20 + Phase 2 + Phase 16 references.
- **E2E:** **`tests/e2e_test.rs`** Phase 20 — **`test_transfer_preflight`** after **`test_balance_preflight`**: **`validate_ss58`**(Bob) + **`validate_amount`** + **`get_balance_ss58`** (Alice/Bob).
- **Discoverability:** **`docs/llm.txt`** Tier 1 + Transfer table row; **`src/utils/explain.rs`** Phase 6 line + TIPS bullet.
- **Errors:** **`src/error.rs`** — validation heuristics **`transfer amount`** + **`invalid destination`** → **12**; VALIDATION **`hint`** branches for both; unit tests **`classify_transfer_*`**.
- **Tracker:** **`goals/commands/COMMANDS_TESTED.md`** — row for transfer commands.

**Left:** Step 46 — next untracked command (e.g. stake subcommand or view); run full **`e2e_local_chain`** in Docker when convenient.

**Tests run:** `cargo fmt --all`; `cargo test -p agcli error::tests --lib`; `cargo test -p agcli --test e2e_test e2e_local_chain --no-run`; `cargo test -p agcli utils::explain::tests --lib` — **passed**.
