## Step 52 (2026-03-20)

**Focus:** **`agcli stake unstake-all`** — full **`stake.md`** section (install, read path vs **`StakeCommands::UnstakeAll`**, exits, e2e), **`error.rs`** **`invalid hotkey-address`** + hint + unit test, Phase 20 **`test_stake_unstake_all_preflight`**, discoverability (**`llm.txt`**, **`explain`** Phase 6 + TIPS + test), **`mod.rs`** help blurb, tracker row.

**Done:**
- **Docs:** **`## stake unstake-all — Unstake all alpha for one hotkey (wallet)`** in **`docs/commands/stake.md`** — **`unlock_and_resolve`** only (optional **`validate_ss58`(`hotkey-address`)**); exit table; e2e cross-ref; **`### stake unstake-all`** → anchor + **`unstake_all`** on-chain note.
- **E2E:** **`tests/e2e_test.rs`** Phase 20 — **`test_stake_unstake_all_preflight`** after **`test_stake_swap_preflight`**: **`validate_ss58(ALICE_SS58, "hotkey-address")`**.
- **Discoverability:** **`docs/llm.txt`** Tier 1 “Unstake all (one hotkey)” + Stake table + Staking bullet; **`src/utils/explain.rs`** Phase 6 line + TIPS + **`owner_workflow_mentions_stake_unstake_all_preflight`**.
- **Errors:** **`src/error.rs`** — validation **12** substring **`invalid hotkey-address`**; hint → **`stake.md`** / **`stake unstake-all --help`**; **`classify_hotkey_address_validation_hint`**.
- **CLI help:** **`src/cli/mod.rs`** — **`UnstakeAll`** doc comment (**`unstake_all`** extrinsic).
- **Tracker:** **`goals/commands/COMMANDS_TESTED.md`** — row for **`stake unstake-all`**.

**Left:** Step 53 — e.g. **`stake swap-limit`** or **`stake unstake-all-alpha`**; optional full **`e2e_local_chain`** Docker run.

**Tests run:** `cargo fmt --all`; `cargo test -p agcli error::tests::classify_hotkey_address_validation_hint --lib`; `cargo test -p agcli utils::explain::tests::owner_workflow_mentions_stake_unstake_all_preflight --lib`; `cargo test -p agcli --features e2e --test e2e_test e2e_local_chain` **`--no-run`** — **passed**.

**Git:** Commit + push **project files only** (excludes **`.github/workflows/ci.yml`**, **`tests/wallet_test.rs`**).
