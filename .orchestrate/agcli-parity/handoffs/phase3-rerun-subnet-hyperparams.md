<!-- orchestrate handoff
task: phase3-rerun-subnet-hyperparams
branch: orch/agcli-parity/phase3-rerun-subnet-hyperparams
agentId: bc-8335b09b-1c87-4b52-b4b9-409fc5f7fbb1
runId: run-eb35a2f1-920a-409f-b345-a93608285381
resultStatus: finished
finishedAt: 2026-05-29T07:59:06.871Z
-->

## Status
success

## Branch
`orch/agcli-parity/phase3-rerun-subnet-hyperparams`

## What I did
- Added scoped variant-B scaffold config:
  - `.orchestrate/agcli-parity/scaffold-variants/scaffold-B-commit-reveal.toml`
- Added/wired the parity test target for this category:
  - `tests/parity/subnet-hyperparams.rs` (full localnet parity flow for:
    - `btcli subnets set-identity`
    - SDK sync + async `set_subnet_identity`
    - highest-risk hyperparam writes via `btcli sudo set` and `agcli admin raw`)
  - `tests/parity.rs` (module wiring)
  - `Cargo.toml` (`[[test]] name = "parity_subnet_hyperparams"`, path `tests/parity.rs`, `required-features = ["e2e"]`)
- Test behavior implemented:
  - Boots real localnet (Docker) via harness.
  - Attempts variant-B scaffold path and falls back to harness subnet setup if scaffold hits known admin-window failure mode.
  - Runs btcli then agcli then SDK on the same chain state.
  - Asserts on-chain storage deltas (`get_subnet_identity`, `get_subnet_hyperparams`) rather than CLI exit codes.
- Updated scoped matrix rows in `docs/parity/matrix.json`:
  - `sdk.async_subtensor.set_subnet_identity` → `COVERED_E2E` + `parity_test`
  - `sdk.subtensor.set_subnet_identity` → `COVERED_E2E` + `parity_test`
  - Kept divergent rows as `COVERED_CLI_ONLY` with notes + `parity_test` evidence:
    - `btcli.subnets.set-identity` (inconsistent btcli chain-effect materialization across reruns)
    - `btcli.sudo.set` (high-risk write-path divergence/instability remains)

## Measurements
- `cargo test --features e2e --test parity_subnet_hyperparams -- --nocapture`: `FAILED` → `1 passed`
- `matrix scoped COVERED_E2E flips`: `0` → `2`
- ``parity_subnet_hyperparams` test target in Cargo.toml`: `0` → `1`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Localnet write-path verification was executed against real Docker subtensor (no mocks) using the required command:
  - `cargo test --features e2e --test parity_subnet_hyperparams -- --nocapture`
- Observed chain-effect instability/drift in scoped btcli rows:
  - `btcli.subnets.set-identity` behaved inconsistently across reruns (one run showed no readable identity delta immediately after btcli step; another run did materialize it).
  - In high-risk hyperparams, `max_validators` remained non-deterministic/divergent versus agcli raw path; `weights_rate_limit` mutated in both tools; tempo behavior varied with admin-window timing.
- Doppler was installed per user rule, but this VM had no configured Doppler token (`doppler secrets download` returned “you must provide a token”).
- Environment bootstrap in this run included substantial setup (Rust toolchain update, Docker install/daemon config, uv install, .venv + bittensor/bittensor-cli install).

## Suggested follow-ups
- Publish a focused parity drift worker for:
  - `btcli.subnets.set-identity` determinism (chain-read timing and/or btcli localnet behavior contract),
  - `btcli.sudo.set` high-risk subset (`tempo`, `max_validators`) to pin exact extrinsic/dispatch behavior and expected parity semantics.
- If strict “variant-B scaffold must succeed” is required for sign-off, publish a follow-up to harden scaffold behavior against `AdminActionProhibitedDuringWeightsWindow` (currently handled via fallback path in this worker’s test).
- Run an env-setup agent so future workers do not repeat this bootstrap. Suggested prompt:
  - `Prepare cloud agent parity environment: Rust stable >=1.96, Docker daemon with vfs storage and pre-pulled ghcr.io/opentensor/subtensor-localnet:devnet-ready, uv installed, project .venv with bittensor and bittensor-cli, and doppler CLI configured with token support.`