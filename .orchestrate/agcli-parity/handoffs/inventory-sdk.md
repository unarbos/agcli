<!-- orchestrate handoff
task: inventory-sdk
branch: orch/agcli-parity/inventory-sdk
agentId: bc-fc17e35f-aa4f-4d38-bf09-695cf4632743
runId: run-77022913-52c0-43b2-aae4-1671024e1760
resultStatus: finished
finishedAt: 2026-05-28T16:39:49.642Z
-->

## Status
success

## Branch
`orch/agcli-parity/inventory-sdk`

## What I did
- Read `.orchestrate/agcli-parity/discovery.md` and the upstream bootstrap handoffs first, then worked only within the allowed output scope.
- Created `docs/parity/inventory-sdk.json` as an exhaustive chain-relevant SDK inventory from the live installed `bittensor` package (`.venv`, v10.3.2), covering:
  - `bittensor.core.subtensor.Subtensor` chain helpers
  - `bittensor.core.async_subtensor.AsyncSubtensor` chain helpers
  - sync/async extrinsic-submitter pairs with dispatchable mappings
  - read helpers using `query`, `query_map`, and `query_runtime_api`
  - source file line references for every row
- Created `docs/parity/inventory-sdk.md` grouped into:
  - extrinsic submitter helpers (write)
  - read helpers (storage query helpers)
  - runtime-api wrappers
  - explicit out-of-scope helper categories with short justifications.
- Corrected classification edge cases so wrapper methods that submit extrinsics indirectly are still marked `write` (notably `add_stake_multiple`, `unstake_multiple`, `serve_axon` in sync+async).
- Ran required JSON validation checks and spot-checked method presence against live classes.

## Measurements
- `inventory-sdk rows: 0 → 278`
- `write rows: 0 → 118`
- `write rows with non-null extrinsic: 0 → 118`
- `runtime-api wrapper rows (read): 0 → 30`
- `jq -e 'type == "array" and length > 30' docs/parity/inventory-sdk.json: false → true`
- `jq -e 'all(.[]; .id and .module and .function and .read_or_write and (.read_or_write != "write" or .extrinsic != null))' docs/parity/inventory-sdk.json: false → true`

## Verification
type-check-only

## Notes, concerns, deviations, findings, thoughts, feedback
- In the installed SDK version (`bittensor==10.3.2`), there is no `SubtensorInterface` class exposed from `bittensor.core.subtensor`; inventory is based on `Subtensor` + `AsyncSubtensor` public methods.
- The prompt mentions `core.chain_data/runtime_api`; in this installed version there is no standalone `runtime_api` module path under `bittensor/core/chain_data/`. Runtime API wrappers are implemented via `Subtensor` / `AsyncSubtensor` methods and are captured as `runtime_api:*` storage-key entries.
- Event/storage annotations are cross-referenced to local subtensor pallet sources where available and represented as expected-chain-effect targets for parity-matrix use.

## Suggested follow-ups
- Add a verifier pass that spot-checks a random sample of `events_expected` and `storage_keys` against chain metadata/runtime traces on localnet for higher confidence before matrix lock-in.
- If parity scope later expands beyond Subtensor classes, run a second inventory pass over `Metagraph` and other chain-reading SDK surfaces to decide whether they should become matrix rows or remain explicitly N/A.