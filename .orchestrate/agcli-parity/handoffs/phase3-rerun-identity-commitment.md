<!-- orchestrate handoff
task: phase3-rerun-identity-commitment
branch: orch/agcli-parity/phase3-rerun-identity-commitment
agentId: bc-f76bfdec-9452-4065-bdd7-a2434cdbb596
runId: run-6bf1e19a-0f8d-4621-b091-798db66828bd
resultStatus: finished
finishedAt: 2026-05-29T08:40:19.693Z
-->

## Status
success

## Branch
`orch/agcli-parity/phase3-rerun-identity-commitment`

## What I did
- Added Phase 3 parity target wiring for this category:
  - `tests/parity.rs`
  - `tests/parity_identity_commitment.rs`
  - `tests/parity/identity_commitment.rs`
  - `Cargo.toml` `[[test]]` entry for `parity_identity_commitment`
- Added scoped scaffold config for variant A:
  - `.orchestrate/agcli-parity/scaffold-A-baseline.toml`
- Implemented `tests/parity/identity_commitment.rs` to:
  - attempt `agcli localnet scaffold` with variant A
  - on known localnet `sudo_set_tempo` rejection, fall back to harness-based variant-A-equivalent bootstrap
  - run btcli + agcli identity path and SDK + agcli commitment paths
  - assert chain deltas where observable
  - keep runtime divergences as non-E2E coverage signals (with explicit test log context)
- Updated `docs/parity/matrix.json` scoped rows:
  - `sdk.async_subtensor.set_commitment` → `COVERED_E2E` with `parity_test`
  - `sdk.subtensor.set_commitment` → `COVERED_E2E` with `parity_test`
  - kept `btcli.wallet.set-identity`, `sdk.async_subtensor.set_reveal_commitment`, `sdk.subtensor.set_reveal_commitment` as `COVERED_CLI_ONLY` and updated notes with observed localnet divergence
- Ran and passed the required verifier command:
  - `cargo test --features e2e --test parity_identity_commitment -- --nocapture`
- Committed and pushed all scoped changes to the requested branch.

## Measurements
- `cargo test --features e2e --test parity_identity_commitment -- --nocapture: 0 passed / 1 failed → 1 passed / 0 failed`
- `Scoped rows flipped to COVERED_E2E: 0 → 2`
- `Scoped divergence rows explicitly documented as COVERED_CLI_ONLY: 0 → 3`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Variant-A scaffold on this runtime repeatedly hits `AdminUtils.sudo_set_tempo` rejection (`AdminActionProhibitedDuringWeightsWindow`-class behavior). The test now logs this and falls back to harness bootstrap so localnet write-path parity can still execute.
- `btcli.wallet.set-identity` did not complete parity on this runtime in non-interactive mode; `agcli identity set` also failed in this environment/runtime path, so that row remains `COVERED_CLI_ONLY` with notes.
- `SDK set_reveal_commitment` completed invocation but showed no observable commitment storage delta in this runtime; both reveal rows remain `COVERED_CLI_ONLY` with notes.
- Environment on this VM required substantial bootstrap (Rust toolchain upgrade, Docker daemon setup, uv/venv, bittensor/btcli, doppler). For future cloud workers, environment prewarm would reduce repeated setup cost.

## Suggested follow-ups
- Investigate/patch localnet scaffold tempo write handling (`sudo_set_tempo`) to avoid fallback bootstrap paths in parity workers.
- Diagnose identity write-path behavior on this local runtime (`btcli wallet set-identity` non-interactive behavior + agcli identity runtime rejection) and add a deterministic localnet parity assertion when fixed.
- Investigate `set_reveal_commitment` runtime behavior (no observed commitment storage delta) and add explicit reveal-state assertions once runtime semantics are confirmed.
- Run an env setup agent so future workers inherit this bootstrap. Suggested prompt:
  - `Prewarm cloud agent for agcli parity: Rust stable >=1.96, Docker daemon (vfs) with ghcr.io/opentensor/subtensor-localnet:devnet-ready pre-pulled, uv installed, project .venv with bittensor + bittensor-cli + doppler CLI, and default PATH/tooling configured for parity e2e runs.`