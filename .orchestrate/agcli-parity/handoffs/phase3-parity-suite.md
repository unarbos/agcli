<!-- orchestrate handoff
task: phase3-parity-suite
branch: cursor/agcli-parity-orchestrate-88b1
agentId: bc-c24eb220-8a00-413e-8807-35c30e25dd80
runId: run-00c7f948-3d48-485b-8bff-276884cfbf86
resultStatus: finished
finishedAt: 2026-05-28T18:51:11.813Z
-->

## Status
partial

## Branch
`cursor/agcli-parity-orchestrate-88b1`

## What my subtree did
- Created `.orchestrate/phase3-parity-suite/plan.json` with 12 category workers, category-specific path fences, maxAttempts=3, and the required parity scaffolding pattern embedded in each worker scoped goal.
- Ran the orchestrate loop repeatedly, handled crash recovery, and respawned failed workers up to retry budget.
- Collected terminal outcomes for all 12 categories:
  - **Handed off:** `parity-balance-transfer`, `parity-wallet`, `parity-stake-basic`, `parity-governance`, `parity-misc`
  - **Error after retries:** `parity-stake-advanced`, `parity-subnet-register`, `parity-subnet-hyperparams`, `parity-weights`, `parity-network-readonly`
  - **Cancelled after prolonged stalled running state:** `parity-root`, `parity-identity-commitment`
- Aggregated worker-reported outputs:
  - `parity-balance-transfer` reported success, unit-test-verified, and flipped 6 matrix rows to `COVERED_E2E`.
  - `parity-wallet` reported success, unit-test-verified, flipped 9 wallet rows to `COVERED_E2E`, and left 2 wallet rows `COVERED_CLI_ONLY` with divergence notes.
  - `parity-stake-basic`, `parity-governance`, and `parity-misc` reported partial completion with matrix updates, but not-verified due inability to run required `--test parity_<category>` targets in this repo state.
- Captured all failure handoffs for non-completing categories for Phase 4 triage.

## Verification
not-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- **High**
  - `parity-wallet` found real chain-effect drift: `associate-hotkey` behavior diverges (btcli no-op vs agcli fee-spending extrinsic on baseline localnet). Worker kept row `COVERED_CLI_ONLY`.
  - Multiple categories could not complete due repeated worker termination with `failureMode: unknown` and no SDK error signal (`stake-advanced`, `subnet-register`, `subnet-hyperparams`, `weights`, `network-readonly`), plus stalled long-running states (`root`, `identity-commitment`).
  - Requested verification commands `cargo test --features e2e --test parity_<category>` were missing as concrete test targets in this branch for several workers, blocking acceptance-level verification.
- **Medium**
  - Wallet worker reported `regen-coldkey` non-interactive drift: btcli produced `success=false` JSON while exiting 0; row left `COVERED_CLI_ONLY`.
  - Governance worker hit environment/toolchain blocker (`Cargo 1.83` could not resolve `edition2024` dependencies), so governance tests were authored but not execution-verified.
- **Low**
  - Several workers reported repo/layout mismatch at start (`tests/parity/*` and matrix paths absent in their initial snapshot), then attempted branch alignment.
  - Initial model choices requested in prompt (`gpt-5.5-high-fast`, `gpt-5.5-high`) were rejected by catalog as invalid in this runtime. I switched worker models to `gpt-5.3-codex-high-fast` to get spawns moving.
  - `syncStateToGit` had to be disabled in this sub-workspace due repeated git-sync push rejection noise; local state/handoffs remained authoritative.

## Suggested follow-ups
- Publish a **recovery subplanner** for the 7 non-success categories with narrower scopes and explicit bootstrap checks before edits:
  - `stake-advanced`, `subnet-register`, `subnet-hyperparams`, `weights`, `network-readonly`, `root`, `identity-commitment`
- Add explicit integration-test targets so required commands are runnable without temporary shims:
  - `tests/parity_<category>.rs` thin entries or `[[test]]` target wiring in `Cargo.toml`.
- Run merge workers to integrate successful category branches (`balance-transfer`, `wallet`, `stake-basic`, `governance`, `misc`) into an integration branch, then re-run per-category verification.
- Prioritize Phase 4 fixes for wallet divergences already surfaced:
  - `associate-hotkey` chain-effect mismatch
  - `regen-coldkey` non-interactive success/exit semantics mismatch
- Run an env setup agent for parity workloads so future workers avoid repeated VM drift. Suggested prompt:
  - `Prepare cloud agent parity environment: Rust stable >=1.95, Docker with working daemon and pre-pulled ghcr.io/opentensor/subtensor-localnet:devnet-ready, uv installed, project .venv with bittensor and bittensor-cli, and cargo-compatible toolchain for edition2024 dependencies.`