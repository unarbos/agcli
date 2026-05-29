<!-- orchestrate handoff
task: parity-matrix
branch: orch/agcli-parity/parity-matrix
agentId: bc-31285050-3318-4760-b826-51840cd5e50e
runId: run-3dc4083d-4fe9-4f40-ab9e-2e0c9eeed2c8
resultStatus: finished
finishedAt: 2026-05-28T17:03:09.439Z
-->

## Status
success

## Branch
`orch/agcli-parity/parity-matrix`

## What I did
- Created `docs/parity/matrix.json` with one row for every upstream inventory row plus agcli-unique rows from `docs/why-agcli.md`.
- Mapped btcli and SDK rows to agcli invocations, assigned `agcli_status` per policy, populated `agcli_flag_diffs`, and set `parity_test` to `null` for all rows in this phase.
- Added cross-link notes for rows that share the same normalized dispatchable across btcli/SDK sources.
- Added 26 `agcli-unique` rows (all cited to locations in `docs/why-agcli.md`) with `COVERED_UNIQUE`.
- Created `docs/parity/matrix.md` grouped exactly as requested:
  - Phase 3 candidates: `COVERED_CLI_ONLY` + `COVERED_UNIQUE`
  - Phase 4 candidates: `GAP`
  - Informational: `COVERED_E2E` + `N/A`
- Ran jq validations for schema/status correctness, per-source row coverage, and N/A note requirements.
- Committed and pushed the branch.

## Measurements
- `docs/parity/matrix.json rows: 0 → 397`
- `matrix btcli-source rows: 0 → 93`
- `matrix sdk-source rows: 0 → 278`
- `matrix agcli-unique rows: 0 → 26`
- `jq status-enum validity: false → true`
- `jq N/A rows all have non-empty notes: false → true`
- `docs/parity/matrix.md Phase 3 candidates rows: 0 → 328`
- `docs/parity/matrix.md Phase 4 candidates rows: 0 → 65`

## Verification
type-check-only

## Notes, concerns, deviations, findings, thoughts, feedback
- `docs/parity/inventory-btcli.json` and `docs/parity/inventory-sdk.json` were not present in this branch checkout. I treated upstream outputs as canonical by reading them directly from `origin/orch/agcli-parity/inventory-btcli` and `origin/orch/agcli-parity/inventory-sdk` via `git show`.
- I did not modify either upstream inventory file, and only wrote the two allowed deliverables.
- I intentionally set `COVERED_E2E` to none in this phase to avoid claiming localnet parity coverage before Phase 3 tests are attached.
- `N/A` was used only for Axon runtime scoped rows (btcli axon set/reset and SDK serve_axon sync/async), with explicit scope justification in notes.

## Suggested follow-ups
- Manually spot-check at least 5 `GAP` rows against `src/cli/mod.rs` and `docs/commands/*` to confirm there is truly no equivalent surface.
- In Phase 3, prioritize parity tests for high-impact `COVERED_CLI_ONLY` write rows first (stake, transfer, proxy, crowdloan, subnet admin).
- In Phase 4, evaluate whether some SDK `GAP` read helpers should be closed via new agcli read surfaces versus explicitly reclassified if deemed non-operator internals.