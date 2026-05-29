<!-- orchestrate handoff
task: inventory-btcli
branch: orch/agcli-parity/inventory-btcli
agentId: bc-9f4b6933-ab18-41c7-9e08-b7fabf2b4d0c
runId: run-23b1dba9-0bae-4bab-b262-b25793b72661
resultStatus: finished
finishedAt: 2026-05-28T16:48:47.770Z
-->

## Status
success

## Branch
`orch/agcli-parity/inventory-btcli`

## What I did
- Read `.orchestrate/agcli-parity/discovery.md` and used the provided upstream bootstrap handoff context.
- Recreated `.venv` (it was missing in this workspace), installed pinned reference tools (`bittensor-cli 9.21.2`, `bittensor 10.3.2`), and initialized `subtensor` submodule for pallet cross-reference.
- Enumerated the full btcli command tree from the installed Typer registry (canonical leaves, including hidden `weights` group commands) and captured 93 leaf subcommands.
- Generated `docs/parity/inventory-btcli.json` with one row per leaf, including:
  - `id`, `invocation`, `args`, `read_or_write`, `extrinsic`, `storage_keys`, `events_expected`, `source_file_ref`, `notes`
- Generated `docs/parity/inventory-btcli.md` grouped by top-level command, one-line per row, with per-row links to JSON IDs.
- Validated JSON schema expectations with jq and spot-checked leaf help paths against btcli.

## Measurements
- `inventory-btcli.json rows: 0 → 93`
- `top-level command groups represented: 0 → 12`
- `rows missing required fields or write-extrinsic mapping: 0 == 0`
- `write rows with null extrinsic: 0 == 0`
- `inventory-btcli.md rendered command rows: 0 → 93`
- `help spot-check leaf commands passing: 0 → 5`

## Verification
type-check-only

## Notes, concerns, deviations, findings, thoughts, feedback
- `docs/parity/versions.json` was not present on this branch; inventory was built against the installed btcli in `.venv` (`9.21.2`) as requested in scope.
- `weights` is hidden in `btcli --help` for this version but was still included in inventory to preserve parity mapping coverage.
- `extrinsic`, `storage_keys`, and `events_expected` are best-effort for dynamic dispatch paths (notably `sudo set`, mechanism admin calls, and proxy/crowd wrapper flows); those rows include explicit notes where behavior is action-dependent.
- No out-of-scope files were modified.

## Suggested follow-ups
- Add a small deterministic generator/checker script in parity tooling to prevent inventory drift when btcli updates.
- During Phase 3 localnet parity tests, validate dynamic-command branches (`wallet swap-coldkey`, `stake wizard`, `sudo set` parameter variants) and tighten event/storage mappings using observed receipts.