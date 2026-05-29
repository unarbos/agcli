<!-- orchestrate handoff
task: parity-balance-transfer
branch: orch/phase3-parity-suite/parity-balance-transfer
agentId: bc-2827650e-3aaf-462d-a7ce-718f8039c6c5
runId: run-3a32847e-ec0d-457a-aff9-76a0798e0b4b
resultStatus: finished
finishedAt: 2026-05-28T18:11:49.339Z
-->

## Status
success

## Branch
`orch/phase3-parity-suite/parity-balance-transfer`

## What I did
- Added `tests/parity/mod.rs` with `#[cfg(feature = "e2e")]` wiring and harness reuse (`tests/e2e_modules/harness.rs`).
- Added `tests/parity/balance_transfer.rs` implementing real localnet parity coverage for:
  - btcli wallet transfer modes (`--allow-death`, `--all --allow-death`, keep-alive default transfer)
  - SDK equivalents (`Client::transfer`, `Client::transfer_all`, `Client::transfer_keep_alive`)
  - agcli equivalents (`transfer`, `transfer-all`, `transfer-keep-alive`)
  - balance assertions via `agcli balance --output json`
  - UX assertions for `--batch --output json --dry-run`, structured JSON validation errors, and success exit codes.
- Added `docs/parity/matrix.json` from the upstream parity-matrix baseline and updated this category’s rows to `COVERED_E2E` with `parity_test` set:
  - `btcli.wallet.balance`
  - `btcli.wallet.transfer`
  - `sdk.async_subtensor.get_balance`
  - `sdk.async_subtensor.transfer`
  - `sdk.subtensor.get_balance`
  - `sdk.subtensor.transfer`
- Verified with the required command:
  - `cargo test --features e2e --test parity_balance_transfer -- --nocapture`
  - (used a temporary local `tests/parity_balance_transfer.rs` entry shim only for execution, then removed it so final diff stays within allowed paths).

## Measurements
- `cargo test --features e2e --test parity_balance_transfer -- --nocapture: 0 passing → 1 passing`
- `matrix rows in this category set to COVERED_E2E: 0 → 6`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- This branch did not contain pre-existing `tests/parity.rs` / `tests/parity_balance_transfer.rs` entry wiring, so the required test command was executed via a temporary local shim file and that shim was deleted before commit.
- `examples/scaffold-variants/` was not present on this base branch, so the parity test boots fresh localnet containers directly (harness-style) with category-specific container names:
  - `agcli_parity_balance_transfer_btcli`
  - `agcli_parity_balance_transfer_agcli`
  - `agcli_parity_balance_transfer_sdk`
  - `agcli_parity_balance_transfer_ux`
- No source files under `src/**` were modified.

## Suggested follow-ups
- Add the permanent parity entry target (`tests/parity.rs` / `tests/parity_balance_transfer.rs`) in the integration branch so `cargo test --test parity_balance_transfer` works without a temporary shim.
- If strict scaffold-variant boot is required for this category, merge scaffold variant artifacts into the integration branch and switch `boot_chain` to variant-config boot path.