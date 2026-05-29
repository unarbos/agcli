<!-- orchestrate handoff
task: fix-p1-senate
branch: orch/agcli-parity/fix-p1-senate
agentId: bc-93bf7d89-8a7f-41fd-be01-2b20e7db995f
runId: run-14ce55f4-98b3-4ac3-b785-00523243b7f7
resultStatus: finished
finishedAt: 2026-05-29T07:29:31.347Z
-->

## Status
success

## Branch
`orch/agcli-parity/fix-p1-senate`

## What I did
- Implemented senate membership query support in `src/chain/queries.rs`:
  - Added `Client::list_senate_members()` with pallet fallback order: `SenateMembers` → `Triumvirate` → `Senate`.
  - Added robust missing-pallet/missing-storage handling and account extraction/normalization into SS58 strings.
- Implemented agcli senate list path in `src/cli/admin_cmds.rs`:
  - Added read-only alias handling for `agcli admin raw --call senate --args '[]'` (plus `senate_members`, `senate-members`, `triumvirate` aliases).
  - Added JSON output contract (`count`, `members`) and text rendering.
  - Added unit tests for alias recognition and empty-args validation.
- Added localnet-backed smoke coverage in `src/chain/queries.rs`:
  - `chain::queries::tests::senate_members_query_localnet_smoke` starts real Docker localnet and validates senate member output shape.
- Updated parity/docs:
  - Flipped `docs/parity/matrix.json` row `btcli.sudo.senate` to `COVERED_E2E` with parity test reference.
  - Updated `docs/commands/admin.md` with the senate raw alias behavior and output schema.
- Committed, pushed branch, and opened a draft PR against `main`.

## Measurements
- `docs/parity/matrix.json :: btcli.sudo.senate agcli_status: GAP → COVERED_E2E`
- `chain::queries::tests::senate_members_query_localnet_smoke: 0 passing → 1 passing`
- `draft PRs for orch/agcli-parity/fix-p1-senate: 0 → 1`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Full command from verification plan was run:
  - `cargo test --features e2e -- --nocapture`
  - Result: `990 passed; 1 failed` with unrelated pre-existing failure `events::tests::pretty_subtensor_variants_are_exhaustive`.
- Targeted senate tests passed, including localnet smoke against Docker.
- Repository had many unrelated modified files in working tree; I staged/committed only the scoped senate files.

## Suggested follow-ups
- Add first-class clap surface in `src/cli/mod.rs` for governance parity (`agcli admin senate list`) so users do not rely on `admin raw` aliases.
- Close remaining governance P1 GAP rows with the same pattern:
  - `btcli.sudo.proposals`
  - `btcli.sudo.senate-vote`