## Step 49 (2026-03-20)

**Focus:** **`agcli view portfolio`** — docs (install + read path + JSON + exits), Phase 20 RPC preflight, **`error.rs`** (`portfolio --address` + hint), discoverability, tracker row.

**Done:**
- **Docs:** **`## view portfolio — Full coldkey portfolio (read-only)`** in **`docs/commands/view.md`** — discoverability, **`cargo install`** examples, read path aligned with **`ViewCommands::Portfolio`** (**`resolve_and_validate_coldkey_address`** → latest **`pin_latest_block`** + **`try_join`(`get_balance_at_hash`, `get_stake_for_coldkey_at_block`, `get_all_dynamic_info_at_block`)** vs **`--at-block`** **`get_block_hash`** + **`try_join`(`get_balance_at_block`, `get_stake_for_coldkey_at_block`)**), **`--live`** note, JSON shapes (latest **`Portfolio`** vs at-block object), exit table (**0** / **2** / **10** / **12** / **15** / **1**), e2e cross-ref; **`### view portfolio`** → anchor + read-only one-liner.
- **E2E:** **`tests/e2e_test.rs`** Phase 20 — **`test_view_portfolio_preflight`** after **`test_stake_remove_preflight`**: **`validate_ss58`**(Alice, **`portfolio --address`**) + pinned-head **`fetch_portfolio`** bundle + head **`--at-block`** bundle.
- **Discoverability:** **`docs/llm.txt`** Tier 1 Full portfolio line + e2e/doc pointer; **`src/utils/explain.rs`** Phase 6 line + TIPS bullet + unit test **`owner_workflow_mentions_view_portfolio_preflight`**.
- **Errors:** **`src/error.rs`** — validation **12** substring **`portfolio --address`**; **`hint`** → **`view.md`**; unit test **`classify_portfolio_address_validation_hint`**.
- **Tracker:** **`goals/commands/COMMANDS_TESTED.md`** — row for **`view portfolio`**.

**Left:** Step 50 — e.g. **`stake move`**; optional full **`e2e_local_chain`** Docker run.

**Tests run:** `cargo fmt --all`; `cargo test -p agcli error::tests::classify_portfolio_address_validation_hint --lib`; `cargo test -p agcli utils::explain::tests::owner_workflow_mentions_view_portfolio_preflight --lib`; `cargo test -p agcli --test e2e_test e2e_local_chain` **`--no-run`** — **passed**.

**Git:** Commit + **`git push origin main`** this step (project paths + tracker + STATE).
