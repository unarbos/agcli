<!-- orchestrate handoff
task: fix-p0-btcli-sudo-stake-burn
branch: orch/agcli-parity/fix-p0-btcli-sudo-stake-burn
agentId: bc-5d04a096-252a-4f2d-8913-0b5e194bae7c
runId: run-601efab3-cd3e-4985-9778-07ebc0f55c21
resultStatus: finished
finishedAt: 2026-05-29T06:39:08.034Z
-->

## Status
success

## Branch
`orch/agcli-parity/fix-p0-btcli-sudo-stake-burn`

## What I did
- Implemented `SubtensorModule.add_stake_burn` support in `src/chain/extrinsics.rs` via new `Client::add_stake_burn(...)` using the runtime’s 4-argument signature (`hotkey, netuid, amount, limit`).
- Added an equivalent CLI path in `src/cli/admin_cmds.rs`:
  - `agcli admin raw --call stake-burn --args '[<netuid>,<amount_tao>]'`
  - alias `--call add_stake_burn` also works
  - parses TAO amount, converts to RAO, and submits chain write-path.
- Added localnet parity write-path test `tests/parity_admin.rs::parity_stake_burn` that:
  - runs against real Docker subtensor localnet
  - executes agcli stake-burn path
  - asserts chain-effect delta (subnet burn and/or balance delta).
- Updated `docs/parity/matrix.json` row `btcli.sudo.stake-burn` from `GAP` to `COVERED_E2E` and attached parity test reference.
- Updated `docs/commands/admin.md` to document the new raw-call alias behavior.
- Committed, pushed the branch, and opened a draft PR against `main`.

## Measurements
- `docs/parity/matrix.json :: btcli.sudo.stake-burn agcli_status: GAP → COVERED_E2E`
- `cargo test --features e2e --test parity_admin -- --nocapture: 0 passed / 1 failed → 1 passed / 0 failed`
- `draft PRs for orch/agcli-parity/fix-p0-btcli-sudo-stake-burn: 0 → 1`

## Verification
unit-test-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Implemented as the scoped “or equivalent” route under existing `admin raw` surface, since adding a brand-new `admin stake-burn` clap subcommand would require touching `src/cli/mod.rs` (outside the requested edit scope).
- Localnet verification was run with the existing harness-backed local chain setup; explicit scaffold variant A config files were not present in this branch snapshot.
- The repo had many unrelated pre-existing modified files; I staged/committed only scoped files for this task.
- Environment setup required in-VM bootstrap (Rust stable update + Docker install/daemon/image pull) to execute the required localnet parity test.

## Suggested follow-ups
- Add a first-class `agcli admin stake-burn --netuid --amount` subcommand in CLI definitions (`src/cli/mod.rs`) so users don’t rely on `admin raw`.
- Consider flipping/covering the related shared-dispatchable SDK rows (`sdk.async_subtensor.add_stake_burn`, `sdk.subtensor.add_stake_burn`) once their parity policy is scheduled.
- Add/restore explicit scaffold variant files (A-E) in-repo so parity workers can bind tests to named scaffold configs directly.
- Run an env-setup agent in Cursor web to bake this toolchain bootstrap into cloud defaults. Suggested prompt:
  - `Prepare cloud agent parity environment: Rust stable >=1.96, Docker daemon with vfs storage driver and pre-pulled ghcr.io/opentensor/subtensor-localnet:devnet-ready, plus startup validation for docker socket access so e2e localnet tests run without manual setup.`