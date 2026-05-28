<!-- orchestrate handoff
task: parity-governance
branch: orch/phase3-parity-suite/parity-governance
agentId: bc-3dde5eb9-f71a-4e9e-9dfb-a4d9bff43deb
runId: run-b942054e-0e15-4e0d-b495-b528956f431f
resultStatus: finished
finishedAt: 2026-05-28T18:03:56.800Z
-->

## Status
partial

## Branch
`orch/phase3-parity-suite/parity-governance`

## What I did
- `tests/parity/governance.rs`
  - Added a new e2e governance parity suite with localnet scaffold bootstrapping, per-scenario fresh container teardown, and pre/post on-chain snapshots via `Client`.
  - Implemented chain-delta parity assertions for:
    - proxy add (`btcli` vs SDK vs `agcli`)
    - proxy remove (`btcli` vs SDK vs `agcli`)
    - sudo-set-tempo wrapper (`btcli sudo set` vs `agcli admin raw`)
  - Added agcli UX assertions for success/invalid flows:
    - `--batch --output json --dry-run` acceptance
    - structured JSON errors on bad input
    - expected exit code behavior
  - Added additional UX checks for `agcli multisig`, `agcli scheduler`, and `agcli preimage` dry-run/error behavior in the same governance harness.
- `tests/parity/mod.rs`
  - Wired in `pub mod governance;` under `#[cfg(feature = "e2e")]`.
- `docs/parity/matrix.json`
  - Flipped covered governance rows from `COVERED_CLI_ONLY` to `COVERED_E2E` and populated `parity_test` for:
    - `btcli.proxy.add`
    - `btcli.proxy.remove`
    - `btcli.sudo.set`
    - `sdk.async_subtensor.add_proxy`
    - `sdk.async_subtensor.remove_proxy`
    - `sdk.subtensor.add_proxy`
    - `sdk.subtensor.remove_proxy`

## Measurements
- `tests/parity/governance.rs LOC: 0 → 1025`
- `governance matrix target rows marked COVERED_E2E: 0 → 7`

## Verification
not-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- The requested verification command failed in this environment before test compilation:
  - `cargo test --locked --features e2e --test parity_governance -- --nocapture`
  - blocker: Cargo 1.83 cannot parse crates requiring `edition2024` (e.g. `time-core 0.1.8`), so dependency resolution aborts.
- This branch baseline did not contain `tests/parity/*` or `docs/parity/matrix.json`; I brought in `tests/parity/mod.rs` + `docs/parity/matrix.json` baseline content from `origin/orch/phase3-parity-suite/parity-stake-basic` and then applied governance-only updates.
- btcli/SDK inventories on this repo snapshot only provide clear governance parity equivalents for proxy and sudo-wrapper rows; there were no matrix rows for btcli/sd k multisig/scheduler/preimage. I still added agcli UX assertions for those command groups inside governance tests.

## Suggested follow-ups
- Upgrade Rust/Cargo in the parity runner image (or pin dependency graph) so `cargo test --features e2e --test parity_governance -- --nocapture` can run.
- If planner wants strict matrix parity for multisig/scheduler/preimage against btcli/SDK, publish follow-up tasks to add/confirm equivalent inventory rows first, then flip those rows to `COVERED_E2E` with dedicated delta assertions.