<!-- orchestrate handoff
task: phase4-gap-triage
branch: orch/agcli-parity/phase4-gap-triage
agentId: bc-ce3415f4-5422-437d-ad8b-1afc03c2e349
runId: run-fc6be5bb-cf35-4ea2-93ed-6e1698b3e432
resultStatus: finished
finishedAt: 2026-05-28T20:36:33.487Z
-->

## Status
success

## Branch
`orch/agcli-parity/phase4-gap-triage`

## What I did
- Added `docs/parity/gap-triage.md` with all required five sections:
  - Executive summary (status counts, top gaps, top CLI-only drift items)
  - GAP rows grouped by P0/P1/P2 with per-row implementation metadata
  - Behavior-drift rows from Phase 3 with cited worker handoffs + fix scope
  - Incomplete Phase 3 categories with failure hypotheses + narrowed respawn scopes
  - Ordered follow-up orchestrate plan with worker-level sequencing
- Updated `docs/parity/matrix.json` so **every `GAP` row** has `triage_tier: P0|P1|P2` embedded in `notes`, without changing `agcli_status` values.
- Verified acceptance checks locally (`file exists`, `>=5` section headers, and `jq` GAP-tier assertion).

## Measurements
- `docs/parity/gap-triage.md exists: 0 → 1`
- `top-level sections (grep '^## '): 0 → 5`
- `GAP rows with P-tier in notes: false → true`
- `GAP triage tier distribution: P0=0/P1=0/P2=0 → P0=1/P1=19/P2=45`
- `agcli_status value changes in matrix: 0 == 0`

## Verification
not-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- This branch snapshot did not contain `docs/parity/matrix.json`; I sourced it from `origin/orch/agcli-parity/parity-matrix` and then applied only the requested triage-note updates.
- Phase 3 produced only **two confirmed** chain-effect divergences (`associate-hotkey`, `regen-coldkey`) in available handoffs; remaining “top 5” list entries are explicitly labeled as high-risk pending diagnosis due incomplete category runs.
- No source/test/docs files outside the allowed scope were modified.

## Suggested follow-ups
- Publish the env-setup worker first (Rust>=1.95, Docker prewarmed localnet image, uv + .venv with btcli/bittensor).
- Re-run the 7 incomplete Phase 3 categories with the narrowed per-category scopes documented in `docs/parity/gap-triage.md`.
- Land `P0` first (`btcli.sudo.stake-burn`) as a single-row PR with localnet write-path verification.
- Land `P1` governance + SDK power-surface rows as second wave, then re-run Phase 5 and Phase 6.