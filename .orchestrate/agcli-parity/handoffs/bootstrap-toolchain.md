<!-- orchestrate handoff
task: bootstrap-toolchain
branch: orch/agcli-parity/bootstrap-toolchain
agentId: bc-6de2c6a8-6510-44c9-b3cd-fe2676c58f49
runId: run-9432f8a8-d04d-41cc-95ee-146ad0eba9de
resultStatus: finished
finishedAt: 2026-05-28T16:15:20.930Z
-->

## Status
partial

## Branch
`orch/agcli-parity/bootstrap-toolchain`

## What I did
- Read `.orchestrate/agcli-parity/discovery.md` first and followed the VM bootstrap recipe.
- Installed Rust stable and set default toolchain; verified `rustc 1.95.0`.
- Initialized the `subtensor` submodule with depth 1 and verified `subtensor/Cargo.toml` exists; captured submodule SHA.
- Installed Docker (`docker.io`), configured `/etc/docker/daemon.json` with `{"features":{"containerd-snapshotter":false},"storage-driver":"vfs"}`, started `dockerd` in background writing to `/tmp/dockerd.log`, and verified `sudo docker run --rm hello-world`.
- Pulled `ghcr.io/opentensor/subtensor-localnet:devnet-ready` and captured first RepoDigest (`sha256:10848ffc...`).
- Installed Python venv support (`python3-venv`), created `.venv`, installed `bittensor` and `bittensor-cli`, and captured `pip show` + `btcli --version`.
- Built release binary with `SKIP_METADATA_FETCH=1 cargo build --release --bin agcli`; verified `target/release/agcli --version`; captured agcli git SHA.
- Ran scaffold smoke attempts (`--output json`, ports 9970–9974, default and config variants). All attempts failed with:
  - `code: 13`
  - `Sudo inner dispatch failed for AdminUtils.sudo_set_tempo`
  - module error index `109` (`AdminActionProhibitedDuringWeightsWindow`)
  - therefore could not produce success JSON with `endpoint`, `block_height`, `subnets[0].neurons[0].ss58`.
- Wrote required files:
  - `docs/parity/versions.json`
  - `docs/parity/versions.md`
- Appended one verified line at tail of `.orchestrate/agcli-parity/discovery.md` under `## Verified at`.
- Committed and pushed:
  - commit: `ea8b553`
  - branch pushed: `orch/agcli-parity/bootstrap-toolchain`

## Measurements
- `rustc version: 1.95.0 >= 1.89`
- `subtensor/Cargo.toml exists: 0 → 1`
- `docker hello-world exit_code: 1 → 0`
- `localnet image digest captured entries: 0 → 1`
- `btcli version: 0 → 9.21.2`
- `bittensor version: 0 → 10.3.2`
- `SKIP_METADATA_FETCH=1 cargo build --release --bin agcli exit_code: 1 → 0`
- `target/release/agcli --version exit_code: 1 → 0`
- `scaffold success JSON required fields present: 0 == 0`

## Verification
type-check-only

## Notes, concerns, deviations, findings, thoughts, feedback
- Only unmet acceptance criterion is scaffold success JSON (step 7). Reproduced repeatedly on fresh ports and container resets.
- Failure is deterministic in this environment/branch/image combination: `AdminActionProhibitedDuringWeightsWindow` during `AdminUtils.sudo_set_tempo`.
- I still captured all other required manifests/versions and appended discovery verification line.
- `.venv/` is present locally as untracked workspace artifact and was not committed.

## Suggested follow-ups
- Investigate and patch `agcli localnet scaffold` to handle/retry `sudo_set_tempo` when chain is in protected weights window (or defer admin writes until window allows).
- Re-run step 7 after scaffold fix and update/bootstrap handoff status from partial to success.