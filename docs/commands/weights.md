# weights — Weight Setting Operations

Validators set weights to score miners on a subnet. Weights determine how emissions are distributed. Supports direct set, two-phase commit-reveal, and atomic commit-reveal workflows.

## From a binary-only install

- **`agcli explain --topic weights`** — built-in cheat sheet (aliases: `settingweights`, `setweights`).
- **`agcli explain --topic weights --full`** — prints this file when `docs/commands/` is next to the binary or when run from the repo (same resolution as other `--full` topics).
- **`agcli weights --help`** — subcommand list and flags.

## Subcommands

### weights show
Read-only: list validators on a subnet who have set weights and their targets (optionally filter to one hotkey). **No wallet** — only RPC reads.

```bash
agcli weights show --netuid 1 [--hotkey-address <SS58>] [--limit N] [--output json]
```

**Discoverability:** `agcli weights --help`, `agcli explain --topic weights`.

**Unknown netuid:** If hyperparams are absent at latest head (no subnet), agcli bails with the same message as `subnet show` / `weights set` — **exit code 12** (validation). RPC failure when fetching hyperparams logs a warning and continues (`require_subnet_exists_for_weights_cmd`, same rule as `weights commit` / `reveal` / `status`).

**Read path / e2e:** Local: `validate_netuid`; optional `validate_ss58` for `--hotkey-address`; optional `validate_view_limit` for `--limit`. Latest-head **`get_subnet_hyperparams`** for subnet existence (same helper as other weight commands). Then either: (1) **`get_neurons_lite`** → find UID for the hotkey → **`get_weights_for_uid`**, or (2) **`get_all_weights`** + **`get_neurons_lite`**, keep only UIDs whose weight vector is non-empty, apply `--limit` to that validator list, pretty-print or JSON. **E2E:** Phase 37 `test_all_weights` logs **`weights_show_preflight`** (the three hyperparams branches above), then exercises **`get_all_weights`**, the non-empty filter, **`get_neurons_lite`**, **`get_weights_for_uid`** for the first listed validator and (when Alice has a non-empty row) the **`--hotkey-address`** path for Alice. **Source:** `WeightCommands::Show` / `handle_weights_show` in `src/cli/weights_cmds.rs`.

**Hotkey not on subnet:** `Hotkey … not found on SN…` after metagraph lookup — treated as a normal CLI error (**exit 1**, not exit 12).

**JSON:** Top-level `netuid`, `validators_with_weights`, `entries` with `uid`, `hotkey`, `num_weights`, `weights` per validator; single-hotkey mode returns one object with `weights` array.

### weights set
Directly set weights on a subnet. Cannot be used when commit-reveal is enabled.

```bash
agcli weights set --netuid 1 --weights "0:100,1:200,2:50" [--version-key 0] [--dry-run]
```

**Discoverability:** `agcli weights --help`, `agcli explain --topic weights` (aliases include `settingweights`, `setweights`).

**Unknown netuid:** If there is no subnet on-chain (hyperparams absent), agcli **bails before** unlocking the wallet, with the same message as `agcli subnet show` / `subnet hyperparams` — **exit code 12** (validation). A bad `--at-block` on read-only commands is different; here the check uses the latest head.

**Read path / e2e:** Latest-head `get_subnet_hyperparams` (same inner read as `subnet hyperparams` for existence + CR + rate limit). Local parse: `validate_netuid`, `validate_weight_input`, `resolve_weights` (`uid:weight`, JSON, `-`, `@file`). **E2E:** `e2e_test::test_set_weights` logs **`weights_set_preflight`** with `commit_reveal_weights_enabled`, `weights_rate_limit`, `min_allowed_weights`, then submits `set_weights` (Phase 7). **Source:** `WeightCommands::Set` in `src/cli/weights_cmds.rs` (`handle_weights`).

**`--dry-run`:** Still unlocks the wallet so agcli can SS58-check stake-weight and warn when below ~1000τ; JSON includes `stake_sufficient`, `commit_reveal_enabled`, `weights_rate_limit_blocks`, and parsed `weights`. No extrinsic is submitted.

**On-chain**: `SubtensorModule::set_weights(origin, netuid, dests, weights, version_key)`
- Storage writes: `Weights` map for the hotkey's UID
- Events: `WeightsSet(netuid, uid)`
- Pre-checks: hotkey registered, sufficient stake (>=1000τ alpha), rate limit, version key match, commit-reveal disabled
- Errors: `NotEnoughStakeToSetWeights`, `SettingWeightsTooFast`, `CommitRevealEnabled`, `IncorrectWeightVersionKey`, `WeightVecLengthIsLow` (too few UIDs vs `min_allowed_weights`), `WeightVecNotEqualSize`, `UidVecContainInvalidOne`

**Dry-run output** (JSON):
```json
{"dry_run": true, "netuid": 1, "num_weights": 3, "version_key": 0,
 "stake_sufficient": true, "commit_reveal_enabled": false,
 "weights_rate_limit_blocks": 100, "weights": [{"uid": 0, "weight": 100}]}
```

### weights commit
Commit a blake2 hash of weights (phase 1 of commit-reveal). Save the salt for reveal.

```bash
agcli weights commit --netuid 1 --weights "0:100,1:200" [--salt "mysecret"]
```

**Discoverability:** `agcli weights --help`, `agcli explain --topic weights`, **Commit-Reveal Flow** below.

**Unknown netuid:** If there is no subnet on-chain, agcli **bails before** unlocking the wallet — same message and **exit code 12** (validation) as `subnet show` / `weights set`. If hyperparams cannot be fetched (RPC error), agcli warns and continues (same `require_subnet_exists_for_weights_cmd` rule as `weights reveal` / `weights status`).

**Read path / e2e:** Latest-head `get_subnet_hyperparams` for existence only (`require_subnet_exists_for_weights_cmd`). Local: `validate_netuid`, `validate_weight_input`, then after unlock `resolve_weights` and **blake2b-256** `compute_weight_commit_hash(uids, weights, salt_bytes)` (omit `--salt` → agcli prints a generated 32-char alphanumeric salt). Unlike **`weights set`**, this path does **not** run the pre-submit stake-weight or commit-reveal “use commit-reveal instead” warnings; rely on **`subnet hyperparams`** / on-chain errors if commit-reveal is off. **E2E:** Phase 17 `test_commit_weights` logs **`weights_commit_preflight`** (`commit_reveal_weights_enabled`, `weights_rate_limit`) then `commit_weights`. **Source:** `WeightCommands::Commit` in `src/cli/weights_cmds.rs`.

**On-chain**: `SubtensorModule::commit_crv3_weights(origin, netuid, commit_hash)`
- Hash: blake2b-256 of (uids, weights, salt)
- Events: `CRV3WeightsCommitted(account, netuid, hash)`
- Errors: `CommittingWeightsTooFast`, `CommitRevealDisabled`, `TooManyUnrevealedCommits`

### weights reveal
Reveal previously committed weights (phase 2 of commit-reveal).

```bash
agcli weights reveal --netuid 1 --weights "0:100,1:200" --salt "mysecret" [--version-key 0]
```

**Discoverability:** `agcli weights --help`, `agcli explain --topic weights` / `commit-reveal`, **Commit-Reveal Flow** below.

**Unknown netuid:** Same pre-check as `weights commit` — **exit 12** before wallet when the subnet is missing at latest head; RPC failure logs a warning and continues (`require_subnet_exists_for_weights_cmd`).

**Read path / e2e:** Latest-head `get_subnet_hyperparams` for existence only (same as commit). Local: `validate_netuid`, `validate_weight_input`, non-empty `--salt`, `resolve_weights`. After unlock, the CLI encodes `--salt` as **little-endian u16 pairs** (two UTF-8 bytes per `u16`, pad the last pair with a zero high byte) — must match what you used at commit time. **E2E:** Phase 17 `test_reveal_weights_rejected_without_prior_commit` logs **`weights_reveal_preflight`** then `reveal_weights` (plus later reveal tests for `RevealTooEarly`, hash mismatch, expiry). **Source:** `WeightCommands::Reveal` in `src/cli/weights_cmds.rs`.

**On-chain**: `SubtensorModule::reveal_crv3_weights(origin, netuid, uids, values, salt, version_key)`
- Events: `CRV3WeightsRevealed(netuid, account)`
- Errors: `NoWeightsCommitFound`, `InvalidRevealCommitHashNotMatch`, `ExpiredWeightCommit`, `RevealTooEarly`

### weights commit-reveal
Atomic: commit, wait for reveal window, then auto-reveal in a single command.

```bash
agcli weights commit-reveal --netuid 1 --weights "0:100,1:200" [--version-key 0] [--wait]
```

**Discoverability:** `agcli weights --help`, `agcli explain --topic weights` / `commit-reveal`, `subnet hyperparams --netuid N` for `commit_reveal_weights_*` and `tempo`.

**Unknown netuid:** If hyperparams are absent at latest head (no subnet), agcli **bails before** unlocking the wallet — same **“Subnet N not found”** text and **exit code 12** (validation) as `subnet show` / `weights set`.

**RPC / hyperparams:** This path **requires** hyperparams for CR on/off, `commit_reveal_weights_interval`, and `tempo` (reveal wait). Unlike **`weights commit`**, **`weights reveal`**, **`weights status`**, and **`weights show`**, a hyperparams **RPC error** does **not** warn-and-continue: agcli returns an error with a connectivity hint (**often exit 10** if the root cause is the endpoint).

**Read path / e2e:** Latest-head `get_subnet_hyperparams` before wallet (inline in `WeightCommands::CommitReveal`, not `require_subnet_exists_for_weights_cmd`). Local: `validate_netuid`, `validate_weight_input`, `resolve_weights`. **E2E:** Phase 17 `test_commit_weights_rejected_when_commit_reveal_disabled` logs **`weights_commit_reveal_preflight`** (`commit_reveal_weights_enabled`, `commit_reveal_weights_interval`, `tempo`, `weights_rate_limit`) before exercising `commit_weights` with CR off. **Source:** `WeightCommands::CommitReveal` in `src/cli/weights_cmds.rs`.

**Behavior**:
1. Fetches hyperparams (see above).
2. If **commit-reveal disabled:** prints a warning and submits **`set_weights`** with the same parsed vector (same extrinsic family as **`weights set`**; stake / rate-limit / `CommitRevealEnabled` rules apply on-chain).
3. If **enabled:** generates a random **32-character alphanumeric** salt, **blake2b-256** commit hash (`compute_weight_commit_hash`), **`commit_weights`**, then polls finalized head every **12s** until `commit_finalized_block + commit_reveal_weights_interval × tempo` blocks, then **`reveal_weights`** with salt encoded as **little-endian u16 pairs** (two UTF-8 bytes per `u16`, same as **`weights reveal`**).
4. **`--wait`:** After reveal, prints a JSON summary (`status`, `commit_tx`, `reveal_tx`, block numbers, `num_weights`).

**On-chain (CR enabled):** `commit_crv3_weights` then `reveal_crv3_weights` — same error families as separate **`weights commit`** / **`weights reveal`** (`CommittingWeightsTooFast`, `TooManyUnrevealedCommits`, `RevealTooEarly`, hash mismatch, expiry, etc.).

### weights status
Check **your** hotkey’s pending **commit-reveal** weight commits on a subnet: commit hash, commit block, reveal window, and a human-readable phase (**WAITING** until the reveal window opens, **READY TO REVEAL** inside the window, **EXPIRED** after `last_reveal`). Also prints current head block, whether commit-reveal is enabled, and `reveal_period_epochs`.

**Discoverability:** `agcli weights --help` → `status`; `agcli explain --topic weights` (Phase 6 cheat line). Full detail: this file.

```bash
agcli weights status --netuid 1
```

Uses the default wallet / hotkey from global flags (same as `weights commit` / `reveal`).

**Unknown netuid:** Latest-head **`require_subnet_exists_for_weights_cmd`** (`get_subnet_hyperparams`) — **exit 12** before the wallet when hyperparams are absent (no subnet). Hyperparams **RPC error** → **warn and continue** (same rule as `weights commit` / `reveal` / `show`), then the command may still fail when loading commits if the endpoint is unusable.

**Read path / e2e:** After preflight: **`try_join!`** of `get_weight_commits`, `get_block_number`, `get_subnet_hyperparams`, `get_reveal_period_epochs` (see `WeightCommands::Status` in `src/cli/weights_cmds.rs`). **E2E:** Phase 17 `test_reveal_weights_rejected_without_prior_commit` logs **`weights_status_preflight`** with the same RPC bundle (after **`weights_reveal_preflight`**) before calling `reveal_weights`. Cross-check all hotkeys on the subnet with **`agcli subnet commits --netuid N`**.

**Errors:** No extrinsic is submitted; failures are wallet unlock / RPC / storage read issues (classified per `src/error.rs`), not on-chain dispatch codes from this command.

**Source map:** `WeightCommands::Status` → `handle_weights` in [`src/cli/weights_cmds.rs`](https://github.com/unarbos/agcli/blob/main/src/cli/weights_cmds.rs); storage query `get_weight_commits` in [`src/chain/extrinsics.rs`](https://github.com/unarbos/agcli/blob/main/src/chain/extrinsics.rs).

### weights commit-timelocked
Commit weights under **drand** timelock: the chain stores a hash tied to a **`--round`**; decryption/reveal is driven by drand when that round is available (see **Timelocked Weights** below for extrinsic/events). Same weight string and Blake2 commit hashing as **`weights commit`** (optional **`--salt`**, random salt printed if omitted — keep it for any follow-on reveal flow your network documents).

**Discoverability:** `agcli weights --help` → `commit-timelocked`; `agcli explain --topic weights` (Phase 6 cheat line). Full detail: this file.

```bash
agcli weights commit-timelocked --netuid 1 --weights "0:100,1:200" --round 12345
agcli weights commit-timelocked --netuid 1 --weights "0:100" --round 12345 --salt mysalt
```

Uses the default wallet / hotkey from global flags (same as `weights commit`).

**Read path:** Latest-head **`require_subnet_exists_for_weights_cmd`** (`get_subnet_hyperparams`) before the wallet — same **unknown subnet → exit 12** and **hyperparams RPC error → warn and continue** as **`weights commit`**. After unlock, the SDK’s **`commit_timelocked_weights`** loads **`CommitRevealWeightsVersion`** via **`get_commit_reveal_weights_version`** and passes it as `commit_reveal_version` on the extrinsic.

**Unknown netuid:** **Exit 12** before wallet when hyperparams are absent at latest head (same bail text as `weights commit` / `subnet show`).

**RPC:** Hyperparams preflight failure does not block (warn + continue). If **`get_commit_reveal_weights_version`** fails before submit, you get a normal RPC/storage error (often exit **10**), not dispatch **111**.

**On-chain error `IncorrectCommitRevealVersion` (dispatch 111):** Only **`commit_timelocked_weights`** checks `commit_reveal_version` against storage. Current agcli reads the live version before submit; **111** usually means a mismatched binary/metadata or a manually crafted call.

**Read path / e2e:** Phase 17 `test_commit_timelocked_weights_rejected_when_incorrect_commit_reveal_version` logs **`weights_commit_timelocked_preflight`** (hyperparams branches + **`get_commit_reveal_weights_version`**) before an intentional wrong-version raw call expecting **111**.

### weights set-mechanism
Set weights for a **single mechanism** (`mechanism_id`: **0** = Yuma, **1** = Oracle) without commit-reveal. Same `uid:weight` string / JSON / `-` / `@file` rules as **`weights set`**.

```bash
agcli weights set-mechanism --netuid 1 --mechanism-id 0 --weights "0:100,1:200" [--version-key 0]
```

**Discoverability:** `agcli weights --help` → `set-mechanism`; `agcli explain --topic weights` (Phase 6 cheat line). Full detail: this file.

**Unknown netuid:** Latest-head **`require_subnet_exists_for_weights_cmd`** (`get_subnet_hyperparams`) — **exit 12** before wallet when hyperparams are absent (no subnet). Hyperparams **RPC error** → **warn and continue** (same rule as **`weights commit`** / **`reveal`** / **`show`**).

**Read path / e2e:** Local: `validate_netuid`, `validate_weight_input`, `resolve_weights`. Then **`require_subnet_exists_for_weights_cmd`** before the wallet; **`set_mechanism_weights`** after unlock. **E2E:** Phase 5 `test_set_mechanism_weights` logs **`weights_set_mechanism_preflight`** (same hyperparams branches as the CLI helper) then submits **`set_mechanism_weights`** (mechanism **0**, same vector shape as **`test_set_weights`**). **Source:** `WeightCommands::SetMechanism` in `src/cli/weights_cmds.rs`.

**`--dry-run`:** JSON only (`dry_run`, `netuid`, `mechanism_id`, `mechanism` name, `num_weights`, `version_key`) — no wallet unlock and no extrinsic (unlike **`weights set`** dry-run).

**On-chain:** `SubtensorModule::set_mechanism_weights(origin, netuid, mecid, dests, weights, version_key)` — mechanism-specific weight matrix; errors overlap **`weights set`** (stake, rate limit, version key, UID validity, commit-reveal mode) where the pallet enforces the same rules.

### weights commit-mechanism
Commit a **blake2b-256** hash for one mechanism’s weight vector (commit-reveal path for mechanism-specific weights). You supply the hash only; compute it offline the same way as **`weights commit`**: same `uids` and `weights` ordering, same raw **salt bytes**, **`compute_weight_commit_hash`** / pallet rules (see **`weights commit`**).

```bash
agcli weights commit-mechanism --netuid 1 --mechanism-id 0 --hash 0x0123abcd...   # 32 bytes = 64 hex chars
```

**Discoverability:** `agcli weights --help` → `commit-mechanism`; `agcli explain --topic weights` (Phase 6 cheat line). Full detail: this file.

**Unknown netuid:** Latest-head **`require_subnet_exists_for_weights_cmd`** — **exit 12** before wallet when hyperparams are absent. Hyperparams **RPC error** → **warn and continue** (same as **`weights commit`** / **`set-mechanism`**).

**Read path / e2e:** Local: `validate_netuid`; **`--hash`** must decode to **exactly 32 bytes** (optional `0x` prefix); then **`require_subnet_exists_for_weights_cmd`** before the wallet; **`commit_mechanism_weights`** after unlock. **E2E:** Phase 5 `test_commit_mechanism_weights` logs **`weights_commit_mechanism_preflight`** (same hyperparams branches as the CLI helper) then submits **`commit_mechanism_weights`** (mechanism **0**, hash from the same UID/weight vector + salt shape as **`test_set_mechanism_weights`**). Phase 5 then runs **`test_reveal_mechanism_weights`** with the matching reveal vector and salt encoding. **Source:** `WeightCommands::CommitMechanism` in `src/cli/weights_cmds.rs`.

**Contrast with `weights commit`:** Global CR commit takes **`--weights`** (and optional **`--salt`**) and hashes inside the CLI; mechanism commit takes a precomputed **`--hash`** only — there is no `--salt` flag on this subcommand.

**On-chain:** `SubtensorModule::commit_mechanism_weights(origin, netuid, mecid, commit_hash)` — errors overlap **`weights commit`** (`CommitRevealDisabled`, `CommittingWeightsTooFast`, `TooManyUnrevealedCommits`, …) where the pallet applies the same rules to mechanism commits. Pair with **`weights reveal-mechanism`** for the reveal step.

### weights reveal-mechanism
Reveal mechanism-specific weights after **`weights commit-mechanism`**: submit the **same** `uid:weight` vector, **`--version-key`**, and **`--salt`** string you used when building the commit hash. Salt is encoded exactly like **`weights reveal`**: UTF-8 bytes taken in pairs, each pair forms a **little-endian `u16`** (if the string has odd length, the high byte of the last `u16` is **0**).

```bash
agcli weights reveal-mechanism --netuid 1 --mechanism-id 0 --weights "0:65535" --salt 'e2e-mech-commit' [--version-key 0]
```

**Discoverability:** `agcli weights --help` → `reveal-mechanism`; `agcli explain --topic weights` (Phase 6 cheat line). Full detail: this file.

**Unknown netuid:** Latest-head **`require_subnet_exists_for_weights_cmd`** — **exit 12** before wallet when hyperparams are absent. Hyperparams **RPC error** → **warn and continue** (same as **`weights commit`** / **`reveal`** / **`set-mechanism`**).

**Read path / e2e:** Local: `validate_netuid`, `validate_weight_input`, `resolve_weights`; salt → `Vec<u16>` as in `WeightCommands::RevealMechanism`; then **`require_subnet_exists_for_weights_cmd`** before the wallet; **`reveal_mechanism_weights`** after unlock. **E2E:** Phase 5 `test_reveal_mechanism_weights` logs **`weights_reveal_mechanism_preflight`** (same three hyperparams branches as the CLI helper) then submits **`reveal_mechanism_weights`** with the same UID/weight vector, mechanism **0**, and salt encoding as **`test_commit_mechanism_weights`** (shared `MECH_CR_SALT_STR` in `e2e_test.rs`). **Source:** `WeightCommands::RevealMechanism` in `src/cli/weights_cmds.rs`.

**Contrast with `weights reveal`:** Same salt encoding and stake/CR error families on-chain where the pallet matches global reveal; this extrinsic is **per mechanism** (`--mechanism-id`).

**On-chain:** `SubtensorModule::reveal_mechanism_weights(origin, netuid, mecid, uids, values, salt, version_key)` — pair with **`commit_mechanism_weights`**; errors overlap **`weights reveal`** (`NoWeightsCommitFound`, `InvalidRevealCommitHashNotMatch`, `RevealTooEarly`, `ExpiredWeightCommit`, …) where applicable.

## Advanced: Mechanism Weights
Subnets can have multiple mechanisms (indexed by MechId). Each mechanism has its own weight matrix. The storage index is `netuid * MAX_MECHANISMS + mecid`.

On-chain extrinsics:
- `set_mechanism_weights(origin, netuid, mecid, dests, weights, version_key)` — CLI: **`weights set-mechanism`**
- `commit_mechanism_weights(origin, netuid, mecid, commit_hash)` — CLI: **`weights commit-mechanism`** (`--hash`); see **`### weights commit-mechanism`** above
- `reveal_mechanism_weights(origin, netuid, mecid, uids, values, salt, version_key)` — CLI: **`weights reveal-mechanism`** (`--weights`, `--salt`, `--version-key`); see **`### weights reveal-mechanism`** above

**Unknown netuid:** **`weights commit-mechanism`** and **`weights reveal-mechanism`** use the same latest-head **`require_subnet_exists_for_weights_cmd`** as **`weights set-mechanism`** — **exit 12** before wallet when the subnet is absent; RPC failure → warn and continue.

## Advanced: Timelocked Weights (Drand)
Weights can be committed with drand-based timelock encryption — auto-decrypted when the specified drand round arrives, without requiring a reveal transaction.

On-chain: `commit_timelocked_weights(origin, netuid, commit, reveal_round, commit_reveal_version)`
- Events: `TimelockedWeightsCommitted(account, netuid, hash, reveal_round)`
- Storage: `TimelockedWeightCommits`

**Unknown netuid:** **Exit 12** before wallet (same as other signing weight commands). **111 / `IncorrectCommitRevealVersion`:** only this extrinsic compares `commit_reveal_version` to storage; see **`weights commit-timelocked`** above.

## Advanced: Batch Weight Operations
Set/commit/reveal weights across multiple subnets in a single extrinsic:
- `batch_set_weights(origin, netuids, weights, version_keys)`
- `batch_commit_weights(origin, netuids, commit_hashes)`
- `batch_reveal_weights(origin, netuid, uids_list, values_list, salts_list, version_keys)`

Events: `BatchWeightsCompleted`, `BatchCompletedWithErrors`, `BatchWeightItemFailed`

## Weight Format
Weights are comma-separated `uid:weight` pairs where:
- `uid` = neuron UID (u16, must exist in metagraph)
- `weight` = weight value (u16, 0-65535)

Weights are normalized on-chain to sum to 1.0 (u16::MAX).

## Commit-Reveal Flow
```
1. agcli weights commit --netuid N --weights "..." [--salt S]
   → saves salt (print to stdout)
2. Wait for reveal window (commit_reveal_period blocks after commit)
3. agcli weights reveal --netuid N --weights "..." --salt S
   → must match exact same weights and salt
```

Or use `agcli weights commit-reveal` to do both automatically.

## Extrinsic finalization timeouts
After submit, agcli waits for inclusion/finalization (default **30s**). If the chain stops producing blocks or the RPC lags, you may see: `Transaction timed out after Ns waiting for finalization` with a **Hint** to raise `--finalization-timeout`, set `AGCLI_FINALIZATION_TIMEOUT` or `finalization_timeout` in `~/.agcli/config.toml`, or tune `--mortality-blocks`. This is the same path exercised by `e2e_test` when the chain is paused (local/CI).

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `NotEnoughStakeToSetWeights` | Hotkey alpha < ~1000τ on subnet | Stake more on this subnet |
| `SettingWeightsTooFast` | Rate limit not expired | Wait `weights_rate_limit` blocks |
| `CommitRevealEnabled` | Used `set` when CR is on | Use `commit-reveal` instead |
| `CommitRevealDisabled` | Used `commit` when CR is off | Use `set` instead |
| `InvalidRevealCommitHashNotMatch` | Wrong weights or salt on reveal | Use exact same values from commit |
| `ExpiredWeightCommit` | Reveal window passed | Re-commit and reveal sooner |
| `RevealTooEarly` | Reveal window not open yet | Wait for reveal window |
| `UidVecContainInvalidOne` | UID not in metagraph | Check `agcli subnet metagraph` |
| `WeightVecLengthIsLow` | Fewer UIDs than `min_allowed_weights` | Add targets or check `agcli subnet hyperparams --netuid N` (`min_allowed_weights`) |
| `IncorrectCommitRevealVersion` (**111**) | `commit_reveal_version` ≠ on-chain version (timelocked commits) | Update agcli; CLI loads chain version before submit |
| Finalization timeout | No new finalized blocks within `--finalization-timeout` | Increase timeout / fix RPC; see **Extrinsic finalization timeouts** above |

## Source Code
**agcli handler**: [`src/cli/weights_cmds.rs`](https://github.com/unarbos/agcli/blob/main/src/cli/weights_cmds.rs) — `handle_weights()`; subcommands include `Show`, `Set`, `Commit`, `Reveal`, **`CommitReveal`** (`commit-reveal`), **`Status`** (`status`), **`CommitTimelocked`** (`commit-timelocked`), **`SetMechanism`** (`set-mechanism`), **`CommitMechanism`** (`commit-mechanism`), **`RevealMechanism`** (`reveal-mechanism`)

**Subtensor pallet**:
- [`subnets/weights.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/weights.rs) — `set_weights`, `commit_crv3_weights`, `reveal_crv3_weights`, mechanism weights, timelocked weights, batch weight operations
- [`macros/dispatches.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/dispatches.rs) — dispatch entry points for all weight extrinsics
- [`macros/events.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/events.rs) — WeightsSet, CRV3WeightsCommitted, CRV3WeightsRevealed, TimelockedWeightsCommitted, BatchWeightsCompleted
- [`macros/errors.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/errors.rs) — weight-related error definitions

## Related Commands
- `agcli subnet hyperparams --netuid N` — Check weights_rate_limit, commit_reveal settings
- `agcli subnet watch --netuid N` — Live tempo countdown and weight window status
- `agcli subnet commits --netuid N` — See pending commits
- `agcli explain --topic commit-reveal` — How commit-reveal works
- `agcli explain --topic rate-limits` — Weight rate limit details
- `agcli explain --topic yuma` — How weights feed into consensus
