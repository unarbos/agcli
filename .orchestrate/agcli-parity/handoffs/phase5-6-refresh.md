<!-- orchestrate handoff
task: phase5-6-refresh
branch: orch/agcli-parity/phase5-6-refresh
agentId: bc-0aed5030-91cb-41d8-9bea-e5953590b4cd
runId: run-72dd841b-40b4-4243-a13b-9d5073c6cc7d
resultStatus: finished
finishedAt: 2026-05-29T08:45:46.165Z
-->

## Status
success

## Branch
`orch/agcli-parity/phase5-6-refresh`

## What I did
- Read required context first:
  - `.orchestrate/agcli-parity/discovery.md`
  - `docs/parity/gap-triage.md`
- Updated `docs/parity/matrix.json` to reflect the documented follow-up cycle outcomes from the two upstream handoffs:
  - Flipped `sdk.async_subtensor.set_commitment` and `sdk.subtensor.set_commitment` to `COVERED_E2E` with `parity_test` set.
  - Kept identity/reveal rows as `COVERED_CLI_ONLY` with explicit divergence notes.
  - Flipped six SDK power-surface rows (`compose_call`, `query_runtime_api`, `validate_extrinsic_params` for async+sync) from `GAP` to `COVERED_CLI_ONLY` and recorded agcli invocations/notes.
- Added `.github/workflows/parity-localnet.yml`:
  - Pulls localnet Docker image.
  - Builds agcli release binary.
  - Discovers parity targets from `tests/parity*.rs`, Cargo `[[test]]` entries, and `tests/parity/*.rs` conventions.
  - Executes every discovered parity target with `cargo test --features e2e --test <target> -- --nocapture`.
  - Fails fast if no parity targets are discoverable (prevents false-green CI).
- Added `docs/parity/parity-report.md` with refreshed, explicit matrix counts and an honest DoD checklist against current counts.
- Ran required verifier command:
  - `python3 -c 'import yaml; yaml.safe_load(open(".github/workflows/parity-localnet.yml"))'`

## Measurements
- `matrix COVERED_E2E rows: 0 → 2`
- `matrix GAP rows: 65 → 59`
- `matrix COVERED_CLI_ONLY rows: 302 → 306`
- `matrix rows with parity_test: 0 → 2`
- `parity-localnet workflow files: 0 → 1`

## Verification
type-check-only

## Notes, concerns, deviations, findings, thoughts, feedback
- This branch snapshot did not contain existing `tests/parity*.rs` targets, so workflow target discovery currently fails fast by design until parity targets are present in-branch.
- `docs/parity/parity-report.md` is generated from current matrix state and explicitly marks mission DoD items as not yet satisfied.
- No PR was opened, per instruction.

## Suggested follow-ups
- Merge/sync the parity test target files (`tests/parity*.rs` and supporting `tests/parity/*`) into the integration branch so `parity-localnet` CI runs end-to-end instead of failing at discovery.
- Re-run full parity matrix aggregation after additional Phase 3/4 merges to keep `parity-report.md` aligned with latest chain-verified coverage.