<!-- orchestrate handoff
task: fix-p1-sdk-power-surface
branch: orch/agcli-parity/fix-p1-sdk-power-surface
agentId: bc-570b476d-89a1-4393-b2f2-040ff116a141
runId: run-26f359c0-81f1-4b25-8150-3756e3f10ad1
resultStatus: finished
finishedAt: 2026-05-29T08:17:02.028Z
-->

## Status
success

## Branch
`orch/agcli-parity/fix-p1-sdk-power-surface`

## What I did
- Added new SDK parity command surfaces in CLI definitions and dispatch:
  - `agcli utils compose-call`
  - `agcli view runtime-api call`
  - `agcli utils validate-extrinsic-params`
- Added `src/cli/sdk_cmds.rs` with concrete implementations:
  - dynamic call composition to SCALE call-data hex (`handle_compose_call`)
  - dynamic runtime API execution via subxt runtime API payloads (`handle_runtime_api_call`)
  - nonce/era/tip preflight validation with mortal-era period checks (`handle_validate_extrinsic_params`)
  - focused unit tests for JSON arg parsing and mortal-era validation rules
- Wired handlers into existing command routing:
  - `src/cli/system_cmds.rs` for new `utils` commands
  - `src/cli/view_cmds.rs` for `view runtime-api call`
  - `src/cli/commands.rs` so `utils compose-call` gets chain connection
- Updated parity matrix rows in `docs/parity/matrix.json` for both async+sync SDK rows:
  - `sdk.async_subtensor.compose_call`
  - `sdk.subtensor.compose_call`
  - `sdk.async_subtensor.query_runtime_api`
  - `sdk.subtensor.query_runtime_api`
  - `sdk.async_subtensor.validate_extrinsic_params`
  - `sdk.subtensor.validate_extrinsic_params`
  - flipped from `GAP` to `COVERED_CLI_ONLY` with agcli invocations/notes
- Pushed branch and opened draft PR against `main`: https://github.com/unarbos/agcli/pull/13

## Measurements
- `matrix targeted SDK GAP rows (compose_call/query_runtime_api/validate_extrinsic_params, async+sync): 6 → 0`
- `matrix targeted SDK COVERED_CLI_ONLY rows (same scope): 0 → 6`
- `draft PRs for branch orch/agcli-parity/fix-p1-sdk-power-surface: 0 → 1`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Required verifier build passes:
  - `SKIP_METADATA_FETCH=1 cargo build --release --bin agcli`
- Targeted unit tests for new helper/validation behavior pass:
  - `SKIP_METADATA_FETCH=1 cargo test --lib sdk_cmds::tests:: -- --nocapture` (4 passed)
- CLI surface wiring verified with help contracts and one live command contract:
  - `agcli utils compose-call --help`
  - `agcli view runtime-api call --help`
  - `agcli utils validate-extrinsic-params --help`
  - `agcli utils validate-extrinsic-params --nonce 1 --era mortal --mortality-blocks 64 --tip 0.1 --output json`
- Matrix rows were moved to `COVERED_CLI_ONLY` rather than `COVERED_E2E`; localnet parity test target wiring under `tests/parity/` is not present in this branch snapshot.

## Suggested follow-ups
- Add localnet parity tests for these three capability families under the Phase 3/4 parity harness and flip the six rows to `COVERED_E2E`.
- Add btcli/SDK side-by-side runtime API validation flows once parity test target wiring (`tests/parity/*` + test target registration) is available.