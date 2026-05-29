<!-- orchestrate handoff
task: phase3-rerun-network-readonly
branch: orch/agcli-parity/phase3-rerun-network-readonly
agentId: bc-addad812-c19e-4dce-b99f-cd2a4e87c789
runId: run-f75fac97-bc50-45ef-878b-66914a9a5297
resultStatus: finished
finishedAt: 2026-05-29T07:49:48.211Z
-->

## Status
success

## Branch
`orch/agcli-parity/phase3-rerun-network-readonly`

## What I did
- Added new parity category implementation at `tests/parity/network-readonly.rs` that:
  - boots localnet for variant A scope (attempts scaffold config first),
  - runs btcli read-only network/view flows (`subnets list/show/get-identity/price`, `view dashboard`),
  - runs agcli equivalents (`subnet list/show`, `view network`),
  - runs both sync + async Bittensor SDK read helpers,
  - asserts no chain-state mutation (balances + subnet count unchanged) across btcli/agcli/SDK runs,
  - asserts cross-tool equivalence on subnet IDs/counts and subnet existence/info expectations.
- Wired test modules:
  - updated `tests/parity/mod.rs` with `network_readonly` module path,
  - added `tests/parity_network_readonly.rs` integration target entrypoint,
  - added `tests/parity.rs` thin parity module file (was missing).
- Added Cargo test target in `Cargo.toml`:
  - `[[test]] name = "parity_network_readonly" path = "tests/parity_network_readonly.rs" required-features = ["e2e"]`.
- Updated `docs/parity/matrix.json` for scoped network/view rows:
  - flipped 15 targeted rows from `COVERED_CLI_ONLY` → `COVERED_E2E`,
  - set `parity_test` to `tests/parity/network-readonly.rs::network_readonly_parity_btcli_sdk_vs_agcli`.

## Measurements
- `targeted network/view matrix rows marked COVERED_E2E: 0 → 15`
- `parity_* integration test targets in Cargo.toml: 5 → 6`
- `cargo test --features e2e --test parity_network_readonly -- --nocapture: 0 passed / 1 failed → 1 passed / 0 failed`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Required verifier command executed successfully:
  - `cargo test --features e2e --test parity_network_readonly -- --nocapture`
  - result: `1 passed; 0 failed`.
- `agcli localnet scaffold` currently fails on this VM/runtime path with `Sudo inner dispatch failed for AdminUtils.sudo_set_tempo` during bootstrap; the new parity test handles this by trying scaffold-variant-A first, then falling back to plain localnet container startup so the readonly parity assertions still run end-to-end.
- No PR was opened (per instruction).
- This run performed substantial environment bootstrap (Rust toolchain update, Docker install/daemon setup, uv install, `.venv` creation, `bittensor` + `bittensor-cli` install, release build).

## Suggested follow-ups
- Investigate/fix `agcli localnet scaffold` failing on `sudo_set_tempo` so parity workers can stay scaffold-only without fallback.
- If strict interpretation requires scaffold manifest-derived keys only, rerun this category after scaffold fix and remove fallback path from `tests/parity/network-readonly.rs`.
- Run an env setup agent in Cursor web so future workers don’t repeat bootstrap; suggested prompt:
  - `Prepare cloud agent parity environment with Rust stable >=1.96, Docker daemon + ghcr.io/opentensor/subtensor-localnet:devnet-ready pre-pulled, uv installed, project .venv with bittensor and bittensor-cli, and working cargo e2e parity defaults.`