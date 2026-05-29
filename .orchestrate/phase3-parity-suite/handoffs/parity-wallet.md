<!-- orchestrate handoff
task: parity-wallet
branch: orch/phase3-parity-suite/parity-wallet
agentId: bc-7d54098f-c996-410f-95ba-2ad873bd6e42
runId: run-c64b0c54-8286-498c-84d2-6a3ca337f2e1
resultStatus: finished
finishedAt: 2026-05-28T18:07:22.966Z
-->

## Status
success

## Branch
`orch/phase3-parity-suite/parity-wallet`

## What I did
- Added wallet parity test scaffolding in `tests/parity/mod.rs` with `#[cfg(feature = "e2e")]`, wiring the existing e2e harness and wallet category module.
- Implemented `tests/parity/wallet.rs` with two real localnet parity tests:
  - `wallet_filesystem_and_sdk_workflows_parity`
    - Covers create, list, show, regen-hotkey, derive, dev-key, show-mnemonic flows across btcli/SDK/agcli where equivalents exist.
    - Uses localnet boot/teardown, `Client` pre/post snapshots, and compares chain deltas.
    - Asserts agcli UX contract: accepts `--batch --output json --dry-run` (where used), returns success exit codes, emits structured JSON errors on bad inputs.
  - `wallet_associate_hotkey_and_check_swap_equivalence`
    - Covers associate-hotkey and check-swap with btcli + SDK vs agcli.
    - Verifies check-swap equivalence and captures the associate-hotkey chain-effect divergence (btcli no-op vs agcli fee-spending extrinsic on baseline localnet).
- Updated `docs/parity/matrix.json` wallet rows:
  - Set covered rows to `COVERED_E2E` with `parity_test`.
  - Kept diverging rows as `COVERED_CLI_ONLY` with explicit divergence notes (`associate-hotkey`, `regen-coldkey`).
- Ran the required verification command exactly:
  - `cargo test --features e2e --test parity_wallet -- --nocapture`
  - Result: passing (2 tests).

## Measurements
- `tests/parity/*.rs files for wallet category: 0 → 2`
- `wallet parity tests in parity_wallet target: 0 → 2 passing`
- `docs/parity/matrix.json wallet-category rows: 0 → 11`
- `docs/parity/matrix.json COVERED_E2E rows (wallet scope): 0 → 9`
- `docs/parity/matrix.json COVERED_CLI_ONLY rows (wallet scope): 0 → 2`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- This branch did not contain preexisting `tests/parity.rs`, `tests/parity_wallet.rs`, or `docs/parity/matrix.json` from earlier phases. I created only the task-allowed files permanently.
- To run the mandated command `cargo test --features e2e --test parity_wallet -- --nocapture`, I used a temporary local `tests/parity_wallet.rs` entrypoint during verification and removed it before commit, so final committed paths remain within scope.
- `associate-hotkey` divergence is real in localnet baseline:
  - btcli reports already-associated and does not spend fee.
  - agcli submits an extrinsic and consumes fee.
  - Matrix row remains `COVERED_CLI_ONLY` with this explicit note.
- `btcli wallet regen-coldkey` showed non-parity behavior in non-interactive runs (`success=false` JSON while exiting 0), so that row remains `COVERED_CLI_ONLY` with notes.
- Environment setup required substantial bootstrapping on this VM (Rust toolchain update, Docker install/daemon startup, Python venv + btcli + bittensor). For future workers, this should be baked into cloud env setup.

## Suggested follow-ups
- Add/restore canonical `tests/parity_wallet.rs` and `tests/parity.rs` entrypoints on the integration branch so workers do not need temporary harness entry files for category test execution.
- Decide intended behavior for `agcli wallet associate-hotkey` when hotkey is already associated, then either align with btcli or document intentional difference in parity report.
- Investigate btcli `regen-coldkey` non-interactive exit semantics vs JSON success field and decide whether parity should follow btcli behavior or keep stricter agcli exit/error contract.
- Run an env-setup agent to persist toolchain/bootstrap for future cloud agents. Suggested prompt:
  - `Update cloud agent environment for agcli parity work: install Rust stable >=1.95, docker.io with vfs daemon config and running dockerd, pre-pull ghcr.io/opentensor/subtensor-localnet:devnet-ready, install uv, create project .venv with bittensor-cli and bittensor, and ensure TMP is set to /tmp for cargo/cc builds.`