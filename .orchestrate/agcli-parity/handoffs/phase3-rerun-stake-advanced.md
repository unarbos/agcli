<!-- orchestrate handoff
task: phase3-rerun-stake-advanced
branch: orch/agcli-parity/phase3-rerun-stake-advanced
agentId: bc-21049e31-a47d-4cef-b8ab-dc134dc24cde
runId: run-25cf619a-7cfa-416f-a1a4-3331ba26532c
resultStatus: finished
finishedAt: 2026-05-29T11:16:53.579Z
-->

## Status
partial

## Branch
`orch/agcli-parity/phase3-rerun-stake-advanced`

## What I did
- Added/wired the scoped parity test target for this category:
  - `tests/parity.rs` (entrypoint + harness wiring + `stake-advanced` module include)
  - `tests/parity/stake-advanced.rs` (full localnet variant-C parity harness for `move/swap/transfer_stake` across btcli, SDK sync, SDK async, agcli; chain snapshot/delta assertions; scaffold fallback handling; retries for transient SDK tx-pool bans)
  - `Cargo.toml` (`[[test]] name = "parity_stake_advanced", path = "tests/parity.rs", required-features = ["e2e"]`)
- Repeatedly executed the required verifier command on localnet:
  - `cargo test --features e2e --test parity_stake_advanced -- --nocapture`
- Debugged and mitigated multiple blockers encountered during reruns:
  - cleaned stale concurrent parity processes/containers that were contending on local resources
  - hardened SDK sync/async retries for transient `Transaction is temporarily banned` behavior
  - isolated agcli wallet setup from btcli wallet format incompatibility by using agcli `wallet dev`
  - added an agcli `AmountTooLow` retry path (TAO-scale then RAO-scale fallback) for stake ops
- Updated scoped matrix rows in `docs/parity/matrix.json` (9 rows: btcli move/swap/transfer + sdk sync/async move/swap/transfer):
  - kept them `COVERED_CLI_ONLY`
  - set `parity_test` to `tests/parity/stake-advanced.rs::stake_advanced_parity_btcli_sdk_agcli`
  - added explicit divergence/blocker notes with the localnet evidence (`SubtensorModule::AmountTooLow` on agcli `stake move`)

## Measurements
(none)

## Verification
not-verified

## Notes, concerns, deviations, findings, thoughts, feedback
- The required localnet parity command was run end-to-end multiple times; latest clean run reproducibly fails at agcli stage:
  - btcli sequence: `move/swap/transfer_stake` completed
  - SDK sync sequence: `move/swap/transfer_stake` completed
  - SDK async sequence: `move/swap/transfer_stake` completed
  - agcli sequence: fails immediately at `stake move` with:
    - `SubtensorModule::AmountTooLow`
    - exit code `13` from agcli
    - test exits with `__PARITY_EXIT:101`
- Because agcli fails on first advanced stake op, swap/transfer parity cannot be fully asserted against agcli in this scope; rows were intentionally left `COVERED_CLI_ONLY` with notes.
- Variant C scaffold continued to hit `sudo_set_tempo` dispatch failures; test exercised the built-in manual multi-subnet fallback path.
- Doppler CLI was installed per rule, but no token was available in this VM (`doppler secrets get ...` returned “you must provide a token”), so no secrets could be fetched from Doppler here.

## Suggested follow-ups
- Publish a focused gap-closure worker for agcli `stake move` on localnet variant C:
  - investigate why agcli returns `SubtensorModule::AmountTooLow` for the same scenario where btcli + SDK succeed
  - validate amount encoding/units and signer/hotkey resolution path in agcli stake move
- After fixing agcli move, rerun:
  - `cargo test --features e2e --test parity_stake_advanced -- --nocapture`
  - then flip the 9 scoped rows from `COVERED_CLI_ONLY` to `COVERED_E2E` if deltas match end-to-end.