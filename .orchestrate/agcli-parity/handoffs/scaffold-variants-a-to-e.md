<!-- orchestrate handoff
task: scaffold-variants-a-to-e
branch: orch/agcli-parity/scaffold-variants-a-to-e
agentId: bc-1e305c08-9cec-4203-947a-fd41e3dec5b5
runId: run-df55502e-463e-4d98-92bf-4232d8d633dc
resultStatus: finished
finishedAt: 2026-05-28T17:01:26.387Z
-->

## Status
partial

## Branch
`orch/agcli-parity/scaffold-variants-a-to-e`

## What I did
- Read `.orchestrate/agcli-parity/discovery.md` first, then created the scaffold variants directory and added:
  - `examples/scaffold-variants/scaffold-A-baseline.toml`
  - `examples/scaffold-variants/scaffold-B-commit-reveal.toml`
  - `examples/scaffold-variants/scaffold-C-multi-subnet.toml`
  - `examples/scaffold-variants/scaffold-D-mechanisms.toml`
  - `examples/scaffold-variants/scaffold-E-user-liquidity.toml`
- Added the required single-line header comment at the top of each TOML (container, port, expected subnet/neuron counts, and post-scaffold call guidance where schema fields are missing).
- Verified canonical field/call names from `src/scaffold.rs` and `subtensor/pallets/admin-utils/src/lib.rs`:
  - `commit_reveal` is exposed in scaffold TOML.
  - `reveal_period_epochs` is not exposed directly in scaffold TOML; documented post-scaffold `sudo_set_commit_reveal_weights_interval`.
  - `mechanism_count` is not exposed directly in scaffold TOML; documented post-scaffold `sudo_set_mechanism_count`.
  - `toggle_user_liquidity` is not exposed in scaffold TOML; documented post-scaffold `agcli liquidity toggle`.
  - Confirmed minimum reveal period constant in subtensor is `1` epoch.
- Successfully booted and captured expected JSON for:
  - `examples/scaffold-variants/scaffold-A-expected.json`
  - `examples/scaffold-variants/scaffold-B-expected.json`
  - `examples/scaffold-variants/scaffold-C-expected.json`
- Committed and pushed:
  - commit `51114f0`
  - branch `orch/agcli-parity/scaffold-variants-a-to-e`

## Measurements
- `scaffold variant TOMLs created (A..E): 0 → 5`
- `expected JSON files passing jq shape check: 0 → 3`
- `variants booted successfully via scaffold (A..E): 0 → 3`

## Verification
not-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- Variants A/B/C succeeded only after handling transient scaffold failures caused by:
  - `AdminUtils.sudo_set_tempo` rejected with `AdminActionProhibitedDuringWeightsWindow` (module error index 109).
- Variants D/E could not be completed to successful scaffold JSON in this run:
  - repeated `sudo_set_tempo` freeze-window failures,
  - subsequent `CannotAffordLockCost` after repeated subnet-create attempts on same chain instance.
- I removed invalid/partial D/E expected output artifacts so only valid expected JSON remains.
- D and E TOMLs still contain the required post-scaffold command documentation in their headers.
- No source/test/docs paths outside allowed scope were modified.

## Suggested follow-ups
- Patch `agcli localnet scaffold` to tolerate admin freeze window (e.g., wait/retry around `sudo_set_tempo` and related admin writes), then rerun D/E.
- Add a scaffold option to skip hyperparam writes (or specifically skip tempo) so variant bootstrap can proceed deterministically.
- After scaffold retry fix, regenerate:
  - `examples/scaffold-variants/scaffold-D-expected.json`
  - `examples/scaffold-variants/scaffold-E-expected.json`
- Consider adding a preflight in scaffold that detects/avoids `CannotAffordLockCost` in retry scenarios (fresh chain reset or lock-cost check).