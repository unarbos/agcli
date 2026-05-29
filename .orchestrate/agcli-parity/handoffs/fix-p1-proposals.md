<!-- orchestrate handoff
task: fix-p1-proposals
branch: orch/agcli-parity/fix-p1-proposals
agentId: bc-1d2a6248-4c4d-47c3-82aa-b745ae18bc27
runId: run-e5e8872f-906b-40c6-a71e-ce517b0c9b8a
resultStatus: finished
finishedAt: 2026-05-29T07:09:02.694Z
-->

## Status
success

## Branch
`orch/agcli-parity/fix-p1-proposals`

## What I did
- Implemented governance proposal listing support in `src/cli/admin_cmds.rs` via:
  - `agcli admin raw --call proposals --args '[]' --output json`
  - bypassed `AdminUtils` raw-call validation for this read-only alias
  - added human-readable text rendering + structured JSON output (`count`, `proposals`)
- Added chain query support in `src/chain/queries.rs`:
  - new `Client::list_senate_proposals()` that reads governance proposal storage
  - reads `Proposals`, `ProposalOf`, and `Voting`
  - includes fallback across governance pallet names (`Triumvirate` then `Senate`) for runtime compatibility
  - normalizes proposal hashes and returns vote summary fields (`index`, `threshold`, `end`, `ayes_count`, `nays_count`)
- Added localnet e2e-gated smoke test in `src/chain/queries.rs`:
  - `chain::queries::tests::senate_proposals_query_localnet_smoke`
  - boots real Docker localnet, queries proposals, validates response shape
- Updated docs:
  - `docs/commands/admin.md` to document the new proposals alias behavior and output contract
- Updated parity matrix:
  - `docs/parity/matrix.json` row `btcli.sudo.proposals` flipped to `COVERED_E2E` with parity test reference
- Committed, pushed, and opened a draft PR (base `main`, head `orch/agcli-parity/fix-p1-proposals`).

## Measurements
- `docs/parity/matrix.json :: btcli.sudo.proposals agcli_status: GAP → COVERED_E2E`
- `senate_proposals_query_localnet_smoke: 0 passing → 1 passing`
- `agcli admin raw --call proposals --args '[]' on localnet: exit 1 → exit 0`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Localnet in this environment does not expose an active governance proposals pallet with entries by default; validated behavior is a successful, structured empty response (`{"count":0,"proposals":[]}`) rather than an error.
- To keep the command robust across runtime variants, the implementation falls back between `Triumvirate` and `Senate` pallet names.
- Workspace had many unrelated pre-existing modified files; only scoped files were staged/committed.
- Environment bootstrap was required in-VM (Rust toolchain upgrade + Docker install/daemon/image pull) to perform real localnet validation.

## Suggested follow-ups
- Add first-class CLI surface in `src/cli/mod.rs` (out of this task’s allowed paths), e.g.:
  - `agcli admin senate proposals`
  - `agcli admin senate list`
  - `agcli admin senate vote`
- Close remaining governance P1 GAP rows (`btcli.sudo.senate`, `btcli.sudo.senate-vote`) with the same one-row-per-PR approach.
- Run a Cursor env-setup agent so future workers don’t repeat toolchain/bootstrap work. Suggested prompt:
  - `Prepare cloud agent parity environment: Rust stable >=1.96, Docker daemon with socket access and vfs storage driver, pre-pull ghcr.io/opentensor/subtensor-localnet:devnet-ready, and verify localnet e2e readiness on startup.`