<!-- orchestrate handoff
task: phase3-rerun-root
branch: orch/agcli-parity/phase3-rerun-root
agentId: bc-0fda8fc2-dff1-4c7e-a4ea-62b51bb3d82d
runId: run-5f9aabaa-1fa4-44a3-8525-032aaec015b1
resultStatus: finished
finishedAt: 2026-05-29T09:09:55.188Z
-->

## Status
success

## Branch
`orch/agcli-parity/phase3-rerun-root`

## What I did
- Added and wired the `parity_root` test target:
  - `tests/parity/root.rs` (new)
  - `tests/parity_root.rs` (new test binary entrypoint)
  - `tests/parity/mod.rs` (`pub mod root;`)
  - `Cargo.toml` (`[[test]] name = "parity_root"`)
- Implemented narrowed Phase 3 root scope tests in `tests/parity/root.rs`:
  - `root_register_parity_btcli_agcli`
  - `root_identity_parity_btcli_agcli`
  - `root_weights_path_btcli_vs_agcli_commit_reveal`
- Used isolated per-reference localnet scenarios (fresh Docker container each run) and on-chain snapshots before/after operations.
- Added bounded execution safeguards (timeouts around btcli commands and snapshot/finalization waits) to prevent indefinite hangs during parity reruns.
- Updated `docs/parity/matrix.json` for scoped rows:
  - Flipped root-register dispatch rows to `COVERED_E2E` with parity test path:
    - `btcli.subnets.register`
    - `sdk.async_subtensor.root_register`
    - `sdk.subtensor.root_register`
  - Kept root-identity dispatch rows as `COVERED_CLI_ONLY` with explicit divergence notes:
    - `btcli.subnets.set-identity`
    - `sdk.async_subtensor.set_subnet_identity`
    - `sdk.subtensor.set_subnet_identity`

## Measurements
- `cargo test --features e2e --test parity_root -- --nocapture`: `1 failed, 2 passed` → `0 failed, 3 passed`
- `root_register-related rows in docs/parity/matrix.json with agcli_status=COVERED_E2E`: `0` → `3`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Localnet verification run executed successfully with the required command:
  - `SKIP_METADATA_FETCH=1 cargo test --features e2e --test parity_root -- --nocapture`
  - Result: all 3 tests passed.
- Confirmed divergence on root identity path in Variant A/netuid 0:
  - btcli `subnets set-identity` path remained no-op in this scenario
  - agcli `identity set-subnet` mutated on-chain identity
  - Matrix rows were intentionally left `COVERED_CLI_ONLY` with notes.
- Root weights path test is included and validated in `parity_root`, but there is no dedicated `agcli root weights` row in current matrix schema; existing mapped weights rows point to different command surfaces.
- I did not open a PR (per task instruction).

## Suggested follow-ups
- Add explicit matrix row(s) for `agcli root weights` (`SubtensorModule.set_weights` with `netuid=0`) so root-weights evidence can be tracked directly.
- Investigate btcli `subnets set-identity` no-op behavior on root netuid 0 in Variant A and decide whether to normalize behavior or preserve as documented divergence.
- Reconcile/standardize GitHub repo input contract mismatch (`btcli` prefers full URL, `agcli` enforces `owner/repo`) for clearer parity ergonomics.