<!-- orchestrate handoff
task: parity-misc
branch: orch/phase3-parity-suite/parity-misc
agentId: bc-e2595969-6753-43b8-8b66-cd3de72fb78b
runId: run-a9cc69d3-1239-424b-ad04-67ba8e05b1c3
resultStatus: finished
finishedAt: 2026-05-28T18:17:09.726Z
-->

## Status
partial

## Branch
`orch/phase3-parity-suite/parity-misc`

## What I did
- Added `tests/parity/mod.rs`:
  - Wired shared parity harness import (`tests/e2e_modules/harness.rs`) behind `#[cfg(feature = "e2e")]`.
  - Declared `pub mod misc;`.
- Added `tests/parity/misc.rs`:
  - Implemented misc parity scaffolding with per-scenario container isolation (`agcli_parity_misc_*`) and teardown.
  - Implemented localnet bootstrap via scaffold config + wallet derivation, using harness helpers (`ensure_local_chain`, `ensure_alive`, `wait_blocks`, `dev_pair`).
  - Implemented chain-effect parity scenarios:
    - `misc_crowdloan_create_parity_btcli_sdk_agcli`
    - `misc_swap_coldkey_parity_btcli_sdk_agcli`
  - Implemented UX/CLI behavior checks for misc groups in:
    - `misc_group_ux_claims_json_errors`
    - includes JSON dry-run / structured JSON error assertions across utils, evm, contracts, drand, liquidity, safe-mode.
- Added `docs/parity/matrix.json` (seeded from latest phase3 matrix branch) and updated covered rows to `COVERED_E2E` with `parity_test` pointers:
  - `btcli.crowd.create`
  - `sdk.async_subtensor.create_crowdloan`
  - `sdk.subtensor.create_crowdloan`
  - `btcli.wallet.swap-coldkey`
  - `sdk.async_subtensor.announce_coldkey_swap`
  - `sdk.subtensor.announce_coldkey_swap`
- Upgraded Rust toolchain on the worker VM to stable `1.96.0` to unblock modern dependency resolution.
- Committed and pushed all changes to the required branch.

## Measurements
- `matrix rows with parity_test starting tests/parity/misc.rs:: 0 → 6`
- `COVERED_E2E rows tied to tests/parity/misc.rs:: 0 → 6`
- `tokio tests in tests/parity/misc.rs: 0 → 3`

## Verification
not-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- The required verifier command fails in this repo state because there is no `parity_misc` test target:
  - `cargo test --features e2e --test parity_misc -- --nocapture`
  - error: `no test target named 'parity_misc'`.
- This branch only allowed modifying:
  - `tests/parity/misc.rs`
  - `tests/parity/mod.rs`
  - `docs/parity/matrix.json`
  so I did not add `tests/parity_misc.rs` or update `Cargo.toml` test entries (both would be required to make `--test parity_misc` executable in this checkout).
- I did not open a PR (per worker instructions).
- No UI changes; no recording artifact applicable.

## Suggested follow-ups
- Add parity test target wiring so the requested verifier command is runnable:
  - either `tests/parity_misc.rs` thin entrypoint, or
  - explicit `[[test]] name = "parity_misc"` in `Cargo.toml`.
- Re-run:
  - `cargo test --features e2e --test parity_misc -- --nocapture`
  in an environment with Docker + `.venv` containing `bittensor` and `bittensor-cli`.
- Expand misc matrix/test coverage for remaining category rows still `COVERED_CLI_ONLY`/`GAP` (notably liquidity/drand/evm/contracts/safe-mode parity mappings).