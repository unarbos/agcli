<!-- orchestrate handoff
task: env-setup-followup
branch: orch/agcli-parity/env-setup-followup
agentId: bc-05080370-3025-46a0-ac7d-3e8f9c29ddc6
runId: run-7a3a816a-880a-4925-a856-28162e6eea66
resultStatus: finished
finishedAt: 2026-05-29T06:15:38.560Z
-->

## Status
success

## Branch
`orch/agcli-parity/env-setup-followup`

## What I did
- Read required context files first:
  - `.orchestrate/agcli-parity/discovery.md`
  - `docs/parity/gap-triage.md`
- Completed environment bootstrap steps:
  - `rustup install stable && rustup default stable`
  - `git submodule update --init --depth=1 -- subtensor`
  - Installed Docker (`docker.io`), configured daemon with `storage-driver: vfs`, started daemon, and pulled `ghcr.io/opentensor/subtensor-localnet:devnet-ready`.
  - Installed `uv`, created `.venv`, activated it, installed `bittensor` + `bittensor-cli`.
  - Built release binary with `SKIP_METADATA_FETCH=1 cargo build --release --bin agcli`.
- Ran scaffold A smoke attempts on port `9971` and validated JSON output format with `jq`.
- Updated scoped docs only:
  - Added `docs/parity/versions.json` with measured toolchain/image/package versions and scaffold smoke metadata.
  - Appended a new verified-at line to `.orchestrate/agcli-parity/discovery.md`.
- Committed and pushed to requested branch (no PR opened).

## Measurements
- `rustc version: 1.96.0 >= 1.95.0`
- `docker daemon status: running == running`
- `docker storage driver: vfs == vfs`
- `.venv package versions: bittensor-cli 9.22.0, bittensor 10.4.0 == installed`
- `SKIP_METADATA_FETCH=1 cargo build --release --bin agcli exit_code: 0 == 0`
- `scaffold A smoke output JSON validity (jq): true == true`

## Verification
type-check-only

## Notes, concerns, deviations, findings, thoughts, feedback
- `docs/parity/versions.json` did not exist on this branch snapshot, so I created it with current measured values.
- `docs/parity/versions.md` also does not exist on this branch snapshot; left unchanged.
- Scaffold A on port `9971` consistently returned runtime error JSON (code `13`) rather than a success payload. Early failures showed `sudo_set_tempo` pallet module error tuple `(index 7, error 109)`; repeated retries later hit `SubtensorModule::CannotAffordLockCost` on the same running chain state.
- Even with runtime failure, the smoke output was valid JSON and was validated with `jq`.

## Suggested follow-ups
- Sync in the intended Phase 0 scaffold variant files (`examples/scaffold-variants/`) so workers can run exact A-E configs rather than fallback/default scaffolding.
- Investigate `localnet scaffold` write-path behavior on `devnet-ready` for `sudo_set_tempo` rejection (`index 7, error 109`) and confirm whether this maps to `AdminActionProhibitedDuringWeightsWindow` on current runtime.
- Reset/replace chain state between repeated scaffold attempts to avoid lock-cost escalation causing `CannotAffordLockCost`.