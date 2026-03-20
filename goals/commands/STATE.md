## Step 46 (2026-03-20)

**Focus:** **`agcli stake list`** — docs (install/discoverability), Phase 20 RPC parity preflight, validation **hint**, tracker row.

**Done:**
- **Docs:** Expanded **`docs/commands/stake.md`** with a top **`stake list`** section — `--help` / **`llm.txt`** / **`explain`**, usage (latest + **`--at-block`** + archive), read path aligned with **`StakeCommands::List`** in **`stake_cmds.rs`**, JSON vs CSV vs human empty states, exit table (**0** / **2** / **10** / **12** / **15** / **1**), e2e + **`test_stake_queries`** cross-ref; trimmed duplicate **`### stake list`** blurb to a link; simplified **Source Code** line-no soup.
- **E2E:** **`tests/e2e_test.rs`** Phase 20 — **`test_stake_list_preflight`** after **`test_transfer_preflight`**: **`validate_ss58`**(Alice, **`stake list --address`**) + **`get_stake_for_coldkey`** + **`get_block_hash`**(head) + **`get_stake_for_coldkey_at_block`**.
- **Discoverability:** **`docs/llm.txt`** Tier 1 + Stake table + detailed bullet for **`stake list`**; **`src/utils/explain.rs`** Phase 6 line + TIPS bullet.
- **Errors:** **`src/error.rs`** — VALIDATION **`hint`** branch for **`stake list --address`**; unit test **`classify_stake_list_address_validation_hint`**.
- **Tracker:** **`goals/commands/COMMANDS_TESTED.md`** — row for **`stake list`**.

**Left:** Step 47 — next command (e.g. **`stake add`** preflight/docs parity or **`view portfolio`**); optional full **`e2e_local_chain`** Docker run.

**Tests run:** `cargo fmt --all`; `cargo test -p agcli error::tests --lib`; `cargo test -p agcli --test e2e_test e2e_local_chain --no-run`; `cargo test -p agcli utils::explain::tests --lib` — **passed**.
