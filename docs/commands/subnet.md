# subnet — Subnet Operations

Create, manage, monitor, and query subnets on the Bittensor network. Subnets are independent networks identified by a netuid (u16), each with its own metagraph, hyperparameters, and alpha token.

## Query Subcommands

### subnet list
List all active subnets with names, neuron counts, emissions, burn costs, and owner (one row per subnet).

```bash
agcli subnet list [--at-block N]

# Machine-readable (global flag):
agcli --output json subnet list
agcli --output csv subnet list
```

**Discoverability:** After install, run `agcli subnet --help` — **`list`** is the first query subcommand. No wallet and no **`--netuid`** (use this to discover netuids before **`subnet show`**, **`subnet hyperparams`**, etc.). Same prose lives under **`docs/commands/subnet.md`**; `agcli explain --topic subnets` mentions **`subnet list`**.

**Errors:** There is **no** exit **12** for “unknown netuid” — the command does not accept **`--netuid`**. RPC or transport failures, and a missing / not-yet-pruned block for **`--at-block`**, use the usual network / chain error classification (**`Reason:`** text; see **`src/error.rs`**). An empty chain prints an empty table or JSON **`[]`** with exit **0**.

**Read path (latest head):** **`queries::subnet::list_subnets`** (**`src/queries/subnet.rs`**) pins one block with **`pin_latest_block`**, then **`try_join`** of **`get_all_subnets_at_block`** + **`get_all_dynamic_info_at_block`** and merges non-empty **`DynamicInfo`** names (and emission when the subnet row has zero emission) — same join logic as the CLI **`List`** arm without **`--at-block`** in **`src/cli/subnet_cmds.rs`**.

**Historical snapshot:** **`--at-block N`** resolves **`get_block_hash(N)`** then runs the same storage reads at that hash (no wallet).

**E2E:** **`e2e_test::test_subnet_detail_queries`** logs **`subnet_list`** after **`list_subnets`** and asserts the test netuid appears in the list when **`subnet show`** returned a row.

**Source map:** **`SubnetCommands::List`** in **`src/cli/subnet_cmds.rs`**; shared list helper **`list_subnets`** in **`src/queries/subnet.rs`**.

**On-chain:** reads **`NetworksAdded`**, identity / naming-related storage, and **`DynamicInfo`** (exact pallets map to the client helpers above).

### subnet show
Show detailed info for a single subnet including Dynamic TAO pricing.

```bash
agcli subnet show --netuid 1 [--at-block N]
# Alias (same command):
agcli subnet info --netuid 1

# Machine-readable:
agcli --output json subnet show --netuid 1
```

**Discoverability:** After install, run `agcli subnet --help` to list subcommands; `show` is documented there with required `--netuid`. Full prose lives in this file under `docs/commands/` in the repo, or `agcli explain --topic subnets` for concepts.

**Errors:** If the netuid is not on-chain, exits **12** (validation) with message `Subnet N not found` and a hint to run `agcli subnet list`.

**On-chain**: reads `SubnetHyperparams`, `DynamicInfo` (tao_in, alpha_in, alpha_out, price).

### subnet hyperparams
Show all hyperparameters for a subnet (commit-reveal flags, min weights, rate limits, registration, burns, etc.).

```bash
agcli subnet hyperparams --netuid 1

# Historical block (same as subnet show):
agcli subnet hyperparams --netuid 1 --at-block 500000

# Machine-readable:
agcli --output json subnet hyperparams --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `hyperparams` with required `--netuid`. Same `docs/commands/subnet.md` section; `agcli explain --topic hyperparams` for field meanings.

**Errors:** Unknown / inactive netuid (no hyperparams on-chain) exits **12** (validation) with `Subnet N not found` — same as `subnet show`. RPC or missing block for `--at-block` surfaces as network/chain errors with the usual exit codes and `Reason:` text.

Shows: rho, kappa, tempo, immunity_period, min_allowed_weights, weights_rate_limit, commit_reveal_weights, commit_reveal_interval, min/max burn, difficulty, registration_allowed, and related fields.

### subnet metagraph
View the full metagraph (all neurons) or a single UID.

```bash
agcli subnet metagraph --netuid 1 [--uid 0] [--at-block N] [--full] [--save]

# Machine-readable (table rows or JSON depending on global --output):
agcli --output json subnet metagraph --netuid 1
agcli --output csv subnet metagraph --netuid 1

# Live refresh (poll interval seconds via global --live):
agcli --live 30 subnet metagraph --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `metagraph` with required `--netuid`. Full detail here and under `docs/commands/` in the repo; `agcli explain --topic subnets` / `explain --topic weights` for concepts.

**Errors:** Unknown / inactive netuid exits **12** (validation) with `Subnet N not found` — same as `subnet show` and `subnet hyperparams`. Missing `--uid` neuron on a **valid** subnet prints `UID … not found` and exits **0** (query succeeded; entity absent). RPC or bad `--at-block` surfaces as network/chain errors with the usual exit codes.

**Columns (default table):** UID, hotkey (short SS58), stake, rank, trust, incentive, emission, last-update block, validator permit. `--full` adds axon/prometheus fields to CSV and a wider table.

### subnet cache-load / cache-list / cache-diff / cache-prune
Manage **on-disk** metagraph snapshots under `~/.agcli/metagraph/sn<N>/` (created with `agcli subnet metagraph --netuid N --save`). Use these for offline review, diffs against live chain data, and pruning old files.

```bash
agcli subnet cache-list --netuid 1
agcli subnet cache-load --netuid 1 [--block N]
agcli subnet cache-diff --netuid 1 [--from-block A] [--to-block B]
agcli subnet cache-prune --netuid 1 [--keep 10]

# Machine-readable (cache-list / cache-load / cache-diff / cache-prune):
agcli --output json subnet cache-list --netuid 1
agcli --output json subnet cache-load --netuid 1
agcli --output json subnet cache-diff --netuid 1 --from-block 100 --to-block 200
agcli --output json subnet cache-prune --netuid 1 --keep 5
```

**Discoverability:** `agcli subnet --help` lists `cache-load`, `cache-list`, `cache-diff`, and `cache-prune` next to `metagraph`. Pair with **`subnet metagraph --save`** (same doc file). Install the `agcli` binary and run `agcli subnet --help` to see the exact flags.

**Errors:** Unknown / inactive netuid exits **12** (validation) with `Subnet N not found` — same preflight as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet emissions`, `subnet health`, `subnet probe`, `subnet commits`, `subnet watch`, `subnet monitor`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, and `subnet liquidity`. If there is **no** snapshot on disk, `cache-load` / `cache-list` print a tip (or JSON `{"error":…}` / empty `snapshots`) and exit **0**. `cache-diff` exits with an error if a requested **cached** block is missing, or if there is no “from” snapshot when you omit `--from-block` (live “to” side still requires a valid `--netuid`). `cache-prune` exits **0** even when nothing is removed.

**cache-diff behavior:** Omit **`--to-block`** to compare “from” (latest cached by default, or `--from-block`) against the **live** metagraph from chain (RPC). Both sides must refer to the same subnet.

**Source map:** `handle_subnet` match arms for `CacheLoad`, `CacheList`, `CacheDiff`, `CachePrune` in `src/cli/subnet_cmds.rs`.

### subnet probe
HTTP reachability check for neuron axons: issues `GET http://<axon_ip>:<port>/` for each neuron that has a non-zero axon port and a non-placeholder IP (`0.0.0.0` is skipped). Uses a **single pinned latest block** for `get_neurons_lite` and per-UID `get_neuron` so the UID list and axon endpoints come from the same snapshot.

```bash
agcli subnet probe --netuid 1 [--uids "0,1,2"] [--timeout-ms 3000] [--concurrency 32]

# Machine-readable (table rows or JSON array of probe results):
agcli --output json subnet probe --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `probe` with required `--netuid`. Full detail here under `docs/commands/` in the repo.

**Errors:** Unknown netuid exits **12** (validation) with `Subnet N not found` — same as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet emissions`, `subnet health`, `subnet watch`, `subnet monitor`, `subnet liquidity`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, and `subnet commits`. Timeouts and connection failures are reported per row as `timeout` / `refused` / `error: …` with exit **0** (the query completed). If no UIDs match `--uids` or the metagraph is empty, prints `No neurons to probe` (or JSON `{"error":"No neurons to probe","netuid":N}`) and exits **0**.

**JSON / columns:** Each row: `uid`, `hotkey`, `ip`, `port`, `status` (HTTP status code as string, or `timeout` / `refused` / `error: …`), `latency_ms` (present when a response was received), `version` (axon version field).

### subnet watch
Polls the chain on an interval and redraws a **terminal dashboard**: current block, tempo progress bar, weights rate limit, commit-reveal on/off, activity cutoff, and (when dynamic info loads) pool price and emission lines. Uses **latest head** each tick (not a pinned snapshot). Clears the screen with ANSI escape codes between refreshes — use a real terminal or expect noisy output when piped.

```bash
agcli subnet watch --netuid 1 [--interval 12]
```

**Discoverability:** `agcli subnet --help` lists `watch` with required `--netuid` and optional `--interval` (default **12** seconds). Full detail in this file under `docs/commands/` in the repo; `agcli explain` (subnet builder flow) references `subnet watch` for live tempo monitoring.

**Errors:** Unknown / inactive netuid exits **12** (validation) with `Subnet N not found` — same as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet emissions`, `subnet health`, `subnet probe`, `subnet monitor`, `subnet liquidity`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, and `subnet commits`. There is no `--output json` mode (human-oriented TUI). If hyperparams briefly return `None` while the subnet still exists, the CLI prints a warning and keeps polling.

**Source map:** `handle_subnet_watch` in `src/cli/subnet_cmds.rs`.

### subnet monitor
Polls **latest head** on an interval and diffs the metagraph between ticks: prints events for new UIDs, deregistrations, hotkey changes, large emission moves (>20%), incentive shifts, and validators becoming inactive. Human mode writes to stdout; use **`--json`** for one JSON object per line (streaming) suitable for pipes and log aggregation.

```bash
agcli subnet monitor --netuid 1 [--interval 24] [--json]
```

**Discoverability:** `agcli subnet --help` lists `monitor` with required `--netuid`, optional `--interval` (default **24** seconds), and `--json`. This section under `docs/commands/` in the repo; `agcli explain` references `subnet monitor` for structured event streaming.

**Errors:** Unknown / inactive netuid exits **12** (validation) with `Subnet N not found` — same as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet emissions`, `subnet health`, `subnet probe`, `subnet commits`, `subnet liquidity`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, and `subnet watch`. RPC failures use the usual exit codes. **Note:** JSON mode uses the **`--json` flag on the subcommand**, not global `--output json` (which applies to table/JSON commands).

**JSON events:** Each line is a JSON object with `"event"` one of `registration`, `deregistration`, `hotkey_change`, `emission_shift`, `incentive_shift`, `inactive`, plus fields such as `block`, `netuid`, `uid`, `hotkey`, etc., depending on the event.

**Source map:** `handle_subnet_monitor` in `src/cli/subnet_cmds.rs`.

### subnet health
Health dashboard: active vs total neurons, validator/miner counts, zero-emission and stale (weight-update) counts, per-neuron table (stake, incentive, emission, trust), plus price/pool lines and tempo / commit-reveal / rate-limit when hyperparams load. Uses a **single pinned latest block** for neurons, dynamic info, and hyperparams so the snapshot is internally consistent.

```bash
agcli subnet health --netuid 1

# Machine-readable:
agcli --output json subnet health --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `health` with required `--netuid`. This section in `docs/commands/subnet.md`; `agcli explain` references `subnet health` for miner/validator status.

**Errors:** Unknown netuid exits **12** (validation) with `Subnet N not found` — same as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet emissions`, `subnet probe`, `subnet watch`, `subnet monitor`, `subnet liquidity`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, and `subnet commits`. RPC failures use the usual exit codes.

**JSON fields:** `netuid`, `block`, `total_neurons`, `active`, `validators`, `miners`, `zero_emission`, `stale_neurons`, `price`, `commit_reveal`, `neurons` (each: `uid`, `hotkey`, `coldkey`, `active`, `stake_rao`, `rank`, `trust`, `consensus`, `incentive`, `dividends`, `emission`, `validator_permit`, `last_update`, `blocks_since_update`).

### subnet emissions
Per-UID emission breakdown for a subnet. Uses a single pinned latest block for `get_neurons_lite` and dynamic info so the table/JSON snapshot is internally consistent.

```bash
agcli subnet emissions --netuid 1

# Machine-readable:
agcli --output json subnet emissions --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `emissions` with required `--netuid`. This section in `docs/commands/subnet.md`; `agcli explain emission` for how subnet emissions relate to weights and epochs.

**Errors:** Unknown netuid exits **12** (validation) with `Subnet N not found` — same as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet health`, `subnet probe`, `subnet watch`, `subnet monitor`, `subnet liquidity`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, and `subnet commits`. If the dynamic-info runtime call fails, emission/block and tempo lines fall back to defaults (`?` name, 0 τ/block, tempo 360) but the command still succeeds when neurons load.

**JSON fields:** `netuid`, `total_emission_per_block_tao`, `daily_emission_tao` (7200 blocks/day heuristic), `tempo`, `neurons` (each: `uid`, `hotkey`, `emission_raw`, `emission_tao`, `share_pct`, `is_validator`).

### subnet cost
Registration cost, difficulty, min/max burn band, and capacity for a subnet. Queries are pinned to a single latest block (consistent burn/difficulty snapshot).

```bash
agcli subnet cost --netuid 1

# Machine-readable:
agcli --output json subnet cost --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `cost` with required `--netuid`. Detail here under `docs/commands/subnet.md`; `agcli explain --topic subnets` for how burn pricing relates to registration.

**Errors:** Unknown / inactive netuid exits **12** (validation) with `Subnet N not found` — same as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet emissions`, `subnet health`, `subnet probe`, `subnet watch`, `subnet monitor`, `subnet liquidity`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, and `subnet commits`. RPC failures surface with the usual exit codes.

**JSON fields:** `netuid`, `burn_rao` / `burn_tao`, `difficulty`, `neurons`, `max_neurons`, `registration_allowed`, `price` (dynamic), `min_burn`, `max_burn` (from hyperparams when available).

### subnet snipe
Hyper-optimized registration sniper. Subscribes to blocks and fires burn registration the instant each block arrives. Includes pre-flight checks (subnet exists, registration enabled, balance sufficient, burn within budget) and smart error classification.

```bash
# Basic: subscribe to finalized blocks, single hotkey
agcli subnet snipe --netuid 97

# Fast mode: best (non-finalized) blocks for ~50% lower latency
agcli subnet snipe --netuid 97 --fast

# Watch-only: monitor slots and burn cost without registering (no wallet needed)
agcli subnet snipe --netuid 97 --watch

# Watch with alert: highlights "SNIPE WINDOW" when burn ≤ max-cost
agcli subnet snipe --netuid 97 --watch --max-cost 1.5

# Register all hotkeys in the wallet sequentially
agcli subnet snipe --netuid 97 --all-hotkeys

# Full combo: fast + all hotkeys + budget cap + attempt limit
agcli subnet snipe --netuid 97 --fast --all-hotkeys --max-cost 2.0 --max-attempts 50
```

**Discoverability:** Install the `agcli` binary, then `agcli subnet --help` → **`snipe`**. Full flags and behavior are documented here under `docs/commands/subnet.md` in the repo (no separate sub-page).

**Read path / e2e:** `test_subnet_detail_queries` logs **`subnet_snipe_preflight`** — `get_subnet_info` on the test netuid after the same **`require_subnet_exists`** class as the CLI (before block subscription or wallet unlock). Full sniper behavior (register, fast mode, max-cost / max-attempts guards, watch-only) is covered in **`e2e_test` sections 6b–6g** (`test_snipe_*`). Parse coverage: `cli_test` `parse_subnet_snipe_*`.

**Errors:** Unknown / inactive netuid fails **before** streaming or opening a wallet: **`require_subnet_exists`** → **`Subnet N not found`** / `agcli subnet list` → exit **12** (validation), same preflight class as `subnet show`, `subnet check-start`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, `subnet cost`, `subnet emission-split`, `subnet mechanism-count`, etc. After preflight, burn over `--max-cost`, insufficient balance, or disabled registration produce human-readable **bail** messages (typically exit **1** / generic — see `src/error.rs` classification). Successful registration prints a tx hash; `AlreadyRegistered` exits **0**.

**Flags:**
| Flag | Description |
|------|-------------|
| `--netuid N` | Subnet UID to register on (required) |
| `--max-cost TAO` | Maximum burn cost in TAO; aborts if burn exceeds this |
| `--max-attempts N` | Maximum block attempts before giving up |
| `--fast` | Subscribe to best (non-finalized) blocks for lower latency |
| `--watch` | Monitor-only mode, no registration attempts |
| `--all-hotkeys` | Register every hotkey in the wallet sequentially |

**Error handling:**
- `AlreadyRegistered` → exits cleanly (hotkey already on subnet)
- `TooManyRegistrationsThisBlock` → waits for next block (not fixed 12s sleep)
- `MaxAllowedUIDs` → waits for slot to open (pruning)
- `InvalidNetuid` → aborts immediately
- Transient errors → retries on next block
- Block stream disconnection → automatic reconnection

**On-chain**: Uses `SubtensorModule::burned_register(origin, netuid, hotkey)` each block.

**Pre-flight checks**: On-chain subnet existence (same check as `subnet show`), `registration_allowed`, `balance ≥ burn`, `burn ≤ max_cost`.

**JSON:** On successful burn registration with global `--output json`, prints `status`, `netuid`, `hotkey`, `tx_hash`, `attempts`, `elapsed_secs`, `burn_rao`.

**Source map:** `handle_subnet` → `handle_snipe` / `handle_snipe_watch` in `src/cli/subnet_cmds.rs`.

### subnet commits
Lists **pending** weight commits for commit-reveal on a subnet: commit hash, commit block, reveal window, and status (`READY` / `WAITING` / `EXPIRED`). Without `--hotkey-address`, scans **all** hotkeys on the subnet (storage iteration). With `--hotkey-address`, queries that hotkey only. Uses **latest head** for block number, hyperparams, reveal period, and commit storage (not the same pinned snapshot as `subnet health` / `subnet probe`).

```bash
agcli subnet commits --netuid 1 [--hotkey-address SS58]

# Machine-readable:
agcli --output json subnet commits --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `commits` with required `--netuid`. Detail here under `docs/commands/` in the repo; `docs/commands/weights.md` and `agcli explain --topic weights` link to this command for pending commits.

**Errors:** Unknown / inactive netuid exits **12** (validation) with `Subnet N not found` — same as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet emissions`, `subnet health`, `subnet probe`, `subnet watch`, `subnet monitor`, `subnet liquidity`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`. RPC / storage iteration failures use the usual exit codes.

**Commit-reveal off:** If `commit_reveal_weights` is disabled for the subnet, prints a short notice (or JSON with `commit_reveal_enabled: false` and a `message`) and exits **0** — no wallet required.

**JSON (CR enabled):** `netuid`, `block`, `commit_reveal_enabled`, `reveal_period_epochs`, `commits` (each: `hotkey`, `hash`, `commit_block`, `first_reveal`, `last_reveal`, `status`, `blocks_until_action`).

**Table columns:** Hotkey (short SS58), Hash (truncated), Committed block, Reveal window (`first..last`), Status, Blocks until action (or `—`).

**Source map:** `handle_subnet_commits` in `src/cli/subnet_cmds.rs`.

### subnet liquidity
AMM depth dashboard: pool-side **TAO** depth, **alpha** in the pool, spot price, and estimated **slippage %** for fixed TAO trade sizes **0.1 / 1 / 10 / 100** τ (constant-product model). Subnets with **zero** TAO in the pool are omitted from the table and JSON array.

```bash
agcli subnet liquidity
agcli subnet liquidity --netuid 1

# Machine-readable (same global --output as other table commands):
agcli --output json subnet liquidity --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `liquidity` after `watch` and before `monitor`; optional `--netuid` scopes to one subnet, otherwise all subnets with pool depth are ranked by TAO liquidity. Detail here under `docs/commands/subnet.md`; `agcli explain` references `subnet liquidity` next to user liquidity extrinsics (`agcli liquidity …`).

**Errors:** With **`--netuid`**, an unknown / inactive subnet exits **12** (validation) with `Subnet N not found` — same as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet emissions`, `subnet health`, `subnet probe`, `subnet commits`, `subnet watch`, `subnet monitor`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`. Without `--netuid`, the command never fails for “missing netuid” (it scans all dynamic info). RPC failures use the usual exit codes.

**JSON fields (per subnet row):** `netuid`, `name`, `price`, `tao_in`, `alpha_in`, `liquidity_depth_tao`, `slippage_estimates` (each: `trade_tao`, `slippage_pct`).

**Source map:** `handle_subnet_liquidity` in `src/cli/subnet_cmds.rs`.

### subnet emission-split / mechanism-count
Read-only queries for **multi-mechanism** subnets: **`emission-split`** reads `MechanismEmissionSplit` storage (weights per mechanism; **Yuma** / **Oracle** / other IDs are labeled when known). **`mechanism-count`** reads `MechanismCountCurrent` (defaults to **1** on-chain when unset).

```bash
agcli subnet emission-split --netuid 1
agcli subnet mechanism-count --netuid 1

# Machine-readable (global --output json, same as other table-style readers):
agcli --output json subnet emission-split --netuid 1
agcli --output json subnet mechanism-count --netuid 1
```

**Discoverability:** `agcli subnet --help` lists `emission-split` and `mechanism-count` with the other read-only subnet tools (before extrinsics like `register`). Owner-only writers live under **`subnet set-emission-split`** / **`subnet set-mechanism-count`** below.

**Errors:** Unknown / inactive netuid exits **12** (validation) with `Subnet N not found` — same preflight as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet cost`, `subnet emissions`, `subnet health`, `subnet probe`, `subnet commits`, `subnet watch`, `subnet monitor`, `subnet liquidity`, `subnet cache-load`, `subnet cache-list`, `subnet cache-diff`, `subnet cache-prune`, `subnet emission-split`, `subnet mechanism-count`, `subnet check-start`, `subnet start`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`. If no custom split is stored, **`emission-split`** prints a default notice (or JSON with `configured: false`) and exits **0**.

**JSON (`emission-split`, configured):** `netuid`, `configured`, `total_weight`, `split` (each: `mechanism`, `weight`, `pct`). **JSON (`emission-split`, default):** `netuid`, `configured: false`, `message`. **JSON (`mechanism-count`):** `netuid`, `mechanism_count`.

**Source map:** `handle_subnet` arms for `EmissionSplit` / `MechanismCount` in `src/cli/subnet_cmds.rs`; queries in `Client::get_emission_split` / `get_mechanism_count` in `src/chain/queries.rs`.

## Extrinsic Subcommands (Write Operations)

### subnet create-cost
Read-only: current **subnet creation lock** (TAO) required before **`subnet register`** or **`subnet register-leased`**. No **`--netuid`** and **no wallet** — pure RPC.

```bash
agcli subnet create-cost

# Machine-readable (same global --output as other readers):
agcli --output json subnet create-cost
```

**Discoverability:** `agcli subnet --help` lists **`create-cost`** next to **`register`**. After install, run **`agcli subnet create-cost --help`** (no required flags).

**Errors:** There is **no** `require_subnet_exists` or exit **12** for unknown netuid (this query is not subnet-scoped). RPC / runtime API failures surface as network or chain read errors with the usual exit codes and `Reason:` text.

**Read path / e2e:** `Client::get_subnet_registration_cost` → runtime API `get_network_registration_cost` (`src/chain/mod.rs`). **`tests/e2e_test.rs`** prints **`subnet_create_cost`**, then **`subnet_register_plain`** and **`subnet_register_leased`** from the **same** query in **`test_subnet_detail_queries`** (economics operators check before **`subnet register`** / **`subnet register-leased`**).

**JSON:** `cost_rao`, `cost_tao` (matches the human “Lock amount” line).

**Source map:** `SubnetCommands::CreateCost` in `src/cli/subnet_cmds.rs`.

### subnet register
Create a new subnet. Burns the current subnet registration cost (lock cost). Check it first with **`agcli subnet create-cost`**.

```bash
agcli subnet register [--password PW] [--yes]
```

**Discoverability:** `agcli subnet --help` → **`register`**. After install, run **`agcli subnet register --help`** for global flags (`--network`, **`--yes`**, **`--password`**, **`--batch`**). Lock amount: **`agcli subnet create-cost`** (or **`--output json`** on that command for scripts). To attach **metadata in the same extrinsic**, use **`agcli subnet register-with-identity`** (see below).

**Read path / e2e:** The CLI submits **`register_network`** only — it does **not** call **`get_subnet_registration_cost`** before unlock (unlike operators, who should run **`subnet create-cost`** first). **`tests/e2e_test.rs`** logs **`subnet_register_plain`** using the same **`Client::get_subnet_registration_cost`** call as **`subnet_create_cost`** so CI pins the lock economics next to **`subnet register`** / **`subnet register-leased`**.

**Errors:** There is **no** **`--netuid`** and **no** exit **12** for “unknown SN” (this creates a **new** subnet). Wrong password, missing wallet files, or IO errors use normal **auth** / **IO** classification. After submit, **`SubnetLimitReached`**, **`CannotAffordLockCost`**, **`BalanceWithdrawalError`**, **`NetworkTxRateLimitExceeded`**, and related dispatch errors use **`format_dispatch_error`** in **`src/chain/mod.rs`** (decoded names + hints; **`CannotAffordLockCost`** points at **`subnet create-cost`**). Successful submit prints a tx hash.

**On-chain**: `SubtensorModule::register_network(origin, hotkey)` or `register_network_with_identity(origin, hotkey, identity)`
- Storage writes: `SubnetMechanism`, `NetworkRegisteredAt`, `TokenSymbol`, `SubnetTAO`, `SubnetAlphaIn`, `SubnetOwner`, `SubnetOwnerHotkey`, `SubnetLocked`, `SubnetworkN`, `NetworksAdded`, `Tempo`, `TotalNetworks` + all hyperparam defaults
- Events: `NetworkAdded(netuid, mechid)`, optionally `SubnetIdentitySet(netuid)`
- Errors: `SubnetLimitReached`, `CannotAffordLockCost`, `BalanceWithdrawalError`, `NetworkTxRateLimitExceeded`
- Note: Registration cost increases with each new subnet; requires `StartCallDelay` blocks before emissions begin

**Source map:** `SubnetCommands::Register` in **`src/cli/subnet_cmds.rs`**; extrinsic **`Client::register_network`** in **`src/chain/extrinsics.rs`**.

### subnet register-with-identity
Register a **new** subnet and set **subnet identity** fields in one step (same lock cost as **`subnet register`**). Optional string fields default to empty; **`--name`** is required.

```bash
agcli subnet register-with-identity --name "My Subnet" \
  [--github owner/repo] [--url https://example.com] [--contact "..."] \
  [--discord "..."] [--description "..."] [--additional "..."] \
  [--password PW] [--yes]
```

**Discoverability:** `agcli subnet --help` → **`register-with-identity`**. Run **`agcli subnet register-with-identity --help`** after install. Lock amount: **`agcli subnet create-cost`**. To change identity on an **existing** subnet, use **`agcli identity set-subnet --netuid N`** (`docs/commands/identity.md`).

**Read path / e2e:** On-chain identity is read via **`Client::get_subnet_identity`** (`SubnetIdentitiesV3` storage, `src/chain/queries.rs`). **`tests/e2e_test.rs`** logs **`subnet_register_with_identity`** in **`test_subnet_detail_queries`** (same RPC used when enriching **`view account --subnet`** and **`identity set-subnet`** flows).

**Errors:** There is **no** `--netuid` or **`require_subnet_exists`** (this creates a new subnet). **Before** the wallet opens: invalid **`--name`** (empty, too long, control characters) or invalid non-empty **`--github`** / **`--url`** → exit **1** with the same validation messages as **`agcli identity set-subnet`** (`validate_subnet_name`, `validate_github_repo`, `validate_url`). Wallet / password / IO errors use normal classification. On-chain rejections match **`subnet register`** (`InvalidIdentity` when fields violate runtime limits — see `src/chain/mod.rs` dispatch text).

**Source map:** `SubnetCommands::RegisterWithIdentity` → `Client::register_network_with_identity` (`src/cli/subnet_cmds.rs`, `src/chain/extrinsics.rs`).

### subnet register-leased
Register a **leased** subnet (temporary subnet with an optional lease end block). Uses the configured **hotkey** and **coldkey** path (same **`unlock_and_resolve`** as **`subnet register`** / **`subnet register-neuron`**). Pair with **`subnet terminate-lease`** when you need to end the lease early as owner.

```bash
agcli subnet register-leased [--end-block N] [--password PW] [--yes]
```

**Discoverability:** `agcli subnet --help` → **`register-leased`**. Install the binary and run **`agcli subnet register-leased --help`** for **`--end-block`**; lock amount: **`agcli subnet create-cost`**; owner lifecycle: **`agcli explain --topic subnet-owner`**.

**Read path / e2e:** There is **no** `require_subnet_exists` — this command **creates** a new subnet (like **`subnet register`**), so there is no `--netuid` preflight or exit **12** for “unknown SN”. **`tests/e2e_test.rs`** logs **`subnet_create_cost`** and **`subnet_register_leased`** from a **single** **`get_subnet_registration_cost`** call (same RPC as **`agcli subnet create-cost`**) so CI exercises the economics query operators use before a write.

**Errors:** Wallet / password failures use normal **auth** / **IO** classification. On-chain rejections (`SubnetLimitReached`, `CannotAffordLockCost`, balance / rate-limit errors, lease-specific runtime errors) use normal **chain** exit classification after submit (see **`subnet register`** and `src/error.rs` chain hints).

**On-chain**: `SubtensorModule::register_leased_network(origin, hotkey, end_block)`
- Events / errors: runtime-dependent (network added + lease metadata when successful)

**Source map:** `SubnetCommands::RegisterLeased` → `Client::register_leased_network` (`src/cli/subnet_cmds.rs`, `src/chain/extrinsics.rs`).

### subnet register-neuron
Register a neuron on an existing subnet (**burn** registration). Uses the configured **hotkey** (same resolution as **`subnet pow`**, **`weights set`**, etc.).

```bash
agcli subnet register-neuron --netuid 1 [--password PW] [--yes]
```

**Discoverability:** `agcli subnet --help` lists **`register-neuron`** with **`--netuid`**. Install the `agcli` binary and run **`agcli subnet register-neuron --help`**; flow overview in **`agcli explain --topic registration`**.

**Errors:** Unknown / inactive netuid → **`require_subnet_exists`** → **`Subnet N not found`** / `agcli subnet list` → exit **12** (validation), **before** hotkey unlock — same preflight class as **`subnet set-param`**, **`subnet trim`**, **`subnet pow`**, **`subnet dissolve`**, **`subnet root-dissolve`**, **`subnet terminate-lease`**, **`subnet set-mechanism-count`**, **`subnet set-emission-split`**, etc. Insufficient balance, **`SubNetRegistrationDisabled`**, or rate limits surface as normal **chain** exits after submit (see `src/chain/mod.rs` dispatch text).

**Read path:** Current burn price is the **`burn`** field on **`agcli subnet show --netuid N`** (`SubnetInfo`); **`tests/e2e_test.rs`** logs it under **`subnet_register_neuron`** in **`test_subnet_detail_queries`**.

**On-chain**: `SubtensorModule::burned_register(origin, netuid, hotkey)`
- Events: `NeuronRegistered(netuid, uid, hotkey)`
- Errors: `SubNetRegistrationDisabled`, `TooManyRegistrationsThisBlock`, `TooManyRegistrationsThisInterval`

**Source map:** `SubnetCommands::RegisterNeuron` → `Client::burned_register` (`src/cli/subnet_cmds.rs`, `src/chain/extrinsics.rs`).

### subnet pow
Register via proof-of-work (multi-threaded CPU mining). Uses the same **hotkey** path as **`register-neuron`**.

```bash
agcli subnet pow --netuid 1 [--threads 4]
```

**Discoverability:** `agcli subnet --help` → **`pow`**. Run **`agcli subnet pow --help`** for **`--threads`**; registration topic: **`agcli explain --topic registration`**.

**Read path:** After preflight, the CLI calls **`get_block_info_for_pow`** (block number + hash for the work) and **`get_difficulty`** for the subnet; **`tests/e2e_test.rs`** logs them under **`subnet_pow`** in **`test_subnet_detail_queries`**.

**Errors:** Unknown / inactive netuid → **`require_subnet_exists`** → exit **12** (validation) **before** hotkey unlock — same preflight as **`subnet register-neuron`**, **`subnet dissolve`**, **`subnet root-dissolve`**, **`subnet terminate-lease`**, **`subnet set-mechanism-count`**, **`subnet set-emission-split`**, etc. **`validate_threads`** rejects **`--threads 0`** with exit **1**. If no nonce is found within the attempt budget, the CLI prints a message and exits **0** (no tx submitted).

**On-chain**: `SubtensorModule::register(origin, netuid, block, nonce, work, hotkey, coldkey)`

**Source map:** `SubnetCommands::Pow` in `src/cli/subnet_cmds.rs`; `Client::pow_register` in `src/chain/extrinsics.rs`.

### subnet dissolve
Schedule dissolution of a subnet (**owner coldkey** only). This is irreversible once executed on-chain; the CLI asks for confirmation unless **`--yes`** / batch yes-mode.

```bash
agcli subnet dissolve --netuid 1 [--password PW] [--yes]
```

**Discoverability:** `agcli subnet --help` → **`dissolve`**. Install the binary and run **`agcli subnet dissolve --help`** for flags; owner workflow: **`agcli explain --topic subnet-owner`**.

**Read path / e2e:** Preflight uses **`require_subnet_exists`** (same **`get_subnet_info`** check as **`subnet show`**). **`tests/e2e_test.rs`** calls **`require_subnet_exists`** and logs **`subnet_dissolve`** / **`subnet_terminate_lease`** in **`test_subnet_detail_queries`** (same RPC for **`subnet terminate-lease`**).

**Errors:** Unknown / inactive netuid → **`require_subnet_exists`** → **`Subnet N not found`** / `agcli subnet list` → exit **12** (validation), **before** wallet unlock — same class as **`subnet trim`**, **`subnet start`**, **`subnet register-neuron`**, **`subnet pow`**, etc. Declining the confirmation prompt prints **`Cancelled.`** and exits **0**. **`NotSubnetOwner`**, **`SubnetNotExists`**, or other chain rejections use normal **chain** exit classification after submit.

**On-chain**: `SubtensorModule::schedule_dissolve_network(origin, netuid)`
- Events: `DissolveNetworkScheduled(account, netuid, execution_block)`
- Errors: `NotSubnetOwner`, `SubnetNotExists`

**Source map:** `SubnetCommands::Dissolve` → `Client::dissolve_network` (`src/cli/subnet_cmds.rs`, `src/chain/extrinsics.rs`).

### subnet root-dissolve
**Root / sudo only:** immediately remove a subnet (distinct from owner **`subnet dissolve`**). Same **`require_subnet_exists`** preflight **before** wallet unlock; unknown netuid → exit **12** like **`subnet show`**.

```bash
agcli subnet root-dissolve --netuid 1 [--password PW] [--yes]
```

**Source map:** `SubnetCommands::RootDissolve` → `Client::root_dissolve_network`.

### subnet terminate-lease
End a **leased** subnet early (**owner coldkey**). Distinct from **`subnet dissolve`** (scheduled owner dissolve) and **`subnet root-dissolve`** (root-only). Pairs with **`subnet register-leased`**.

```bash
agcli subnet terminate-lease --netuid 1 [--password PW] [--yes]
```

**Discoverability:** `agcli subnet --help` → **`terminate-lease`**. Install the binary and run **`agcli subnet terminate-lease --help`** for flags; owner workflow: **`agcli explain --topic subnet-owner`**.

**Read path / e2e:** **`require_subnet_exists`** runs **before** wallet unlock (same **`get_subnet_info`** check as **`subnet show`** / **`subnet dissolve`**). **`tests/e2e_test.rs`** logs **`subnet_terminate_lease`** next to the dissolve preflight block in **`test_subnet_detail_queries`** (the same test also logs **`subnet_create_cost`** / **`subnet_register_leased`** for the lock-cost RPC used by **`register-leased`** / **`create-cost`**).

**Errors:** Unknown / inactive netuid → **`Subnet N not found`** → exit **12** (validation) **before** wallet unlock — same preflight class as **`subnet dissolve`**, **`subnet root-dissolve`**, **`subnet trim`**, etc. **`NotSubnetOwner`** or other chain rejections after submit use normal **chain** exit classification.

**Source map:** `SubnetCommands::TerminateLease` → `Client::terminate_lease` (`src/cli/subnet_cmds.rs`, `src/chain/extrinsics.rs`).

### subnet start
Start a subnet's emission schedule (owner only).

```bash
agcli subnet start --netuid 1
```

**Discoverability:** `agcli subnet --help` lists **`start`** next to **`check-start`**. Requires `--wallet` / password like other owner extrinsics.

**Errors:** Unknown / inactive netuid is rejected **before** the wallet unlocks: **`require_subnet_exists`** → **`Subnet N not found`** → exit **12** (validation), same as `subnet show` / `subnet check-start` / `subnet set-param` / `subnet set-symbol` / `subnet trim` / `subnet register-neuron` / `subnet pow` / `subnet dissolve` / `subnet root-dissolve` / `subnet terminate-lease` / `subnet set-mechanism-count` / `subnet set-emission-split`. Not owner, call disabled, or chain errors use the usual **chain** exit classification after submit.

**On-chain**: `SubtensorModule::start_call(origin, netuid)` — sets `FirstEmissionBlockNumber`.

### subnet check-start
Read-only: whether emissions are already **active**, neuron count, whether **`subnet start`** is applicable, and **tempo** from hyperparams when available.

```bash
agcli subnet check-start --netuid 1

# Machine-readable (global --output json):
agcli --output json subnet check-start --netuid 1
```

**Discoverability:** `agcli subnet --help` → **`check-start`**. Pair with **`subnet start`** after `subnet register` and neuron bootstrap (see tutorials / `agcli explain --topic subnet-owner`).

**Errors:** Unknown / inactive netuid → **`require_subnet_exists`** → **`Subnet N not found`** / `agcli subnet list` → exit **12** (validation), same preflight as `subnet show`, `subnet hyperparams`, `subnet metagraph`, `subnet snipe`, `subnet set-param`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, etc.

**JSON fields:** `netuid`, `active`, `neurons`, `min_neurons_to_start` (always **1** in CLI), `can_start` (`!active && neurons > 0`), `tempo` (from hyperparams when `Some`).

**Human output:** Already active → short message; **0** neurons → register first; else hints **`agcli subnet start --netuid N`**.

**Source map:** `SubnetCommands::CheckStart` in `src/cli/subnet_cmds.rs`.

### subnet set-param
Set a subnet hyperparameter via `AdminUtils` (subnet owner for owner-allowed params; some fields are root/sudo-only — see [`docs/hyperparameters.md`](../hyperparameters.md)). About 31 configurable params.

```bash
agcli subnet set-param --netuid 1 --param tempo --value 100
agcli subnet set-param --netuid 1 --param list  # show all settable params (table or JSON)

# Machine-readable parameter catalog:
agcli --output json subnet set-param --netuid 1 --param list
```

**Discoverability:** `agcli subnet --help` lists `set-param` with required `--netuid` and `--param`. Install the binary and run `--help`; full prose here and in `docs/hyperparameters.md`; `agcli explain --topic subnet-owner` for the owner workflow.

**Errors:** Unknown / inactive netuid is rejected **before** `--param list`, the confirmation prompt, or wallet unlock: **`require_subnet_exists`** → **`Subnet N not found`** / `agcli subnet list` → exit **12** (validation), same preflight class as `subnet show`, `subnet hyperparams`, `subnet check-start`, `subnet start`, `subnet set-symbol`, `subnet trim`, `subnet register-neuron`, `subnet pow`, `subnet dissolve`, `subnet root-dissolve`, `subnet terminate-lease`, `subnet set-mechanism-count`, `subnet set-emission-split`, etc. Unknown `--param` name (other than `list` / `help`) exits **1** with suggestions. Missing `--value` for a real parameter exits **1**. On-chain rejections (`NotSubnetOwner`, sudo-only param, **`ColdkeySwapAnnounced`** while a coldkey swap is pending — see `agcli wallet check-swap`, etc.) use normal **chain** exit classification after submit (dispatch text + hints in `src/chain/mod.rs` / `src/error.rs`).

**JSON:** Global `--output json` applies to **`--param list`** (`parameters` array with `name`, `type`, `scope`, `description`). Successful writes use the same tx JSON shape as other extrinsics (`print_tx_result`).

Settable params include tempo, max_allowed_uids, immunity_period, max_allowed_validators, min_burn, max_burn, difficulty, weights_rate_limit, commit-reveal fields, liquid alpha, bonds, and more — see `list` output or `hyperparameters.md`.

**Source map:** `SubnetCommands::SetParam` → `handle_subnet_set_param` in `src/cli/subnet_cmds.rs`.

### subnet set-symbol
Set the subnet alpha token symbol on-chain (subnet owner only). The CLI validates `--symbol` locally (non-empty ASCII, max length **32**) before any RPC preflight or wallet unlock.

```bash
agcli subnet set-symbol --netuid 1 --symbol ALPHA
```

**Discoverability:** `agcli subnet --help` lists **`set-symbol`** with **`--netuid`** and **`--symbol`**. Install the binary and run **`agcli subnet set-symbol --help`**; owner workflow context in **`agcli explain --topic subnet-owner`**.

**Errors:** Unknown / inactive netuid → **`require_subnet_exists`** → **`Subnet N not found`** / `agcli subnet list` → exit **12** (validation), **after** symbol validation and **before** wallet unlock — same preflight class as **`subnet set-param`**, **`subnet start`**, **`subnet trim`**, **`subnet register-neuron`**, **`subnet pow`**, **`subnet dissolve`**, **`subnet root-dissolve`**, **`subnet terminate-lease`**, **`subnet set-mechanism-count`**, **`subnet set-emission-split`**, etc. Empty / non-ASCII / too-long **`--symbol`** exits **1** (validation) with a short tip from `validate_symbol`. On-chain rejections (**`NotSubnetOwner`**, **`SymbolAlreadyInUse`**, etc.) use normal **chain** exit classification after submit (see `src/chain/mod.rs` dispatch error text).

**Read path:** `agcli subnet show --netuid N` prints **`Symbol`** from runtime subnet info; storage query **`TokenSymbol`** matches the string this extrinsic sets — see **`get_token_symbol`** in **`tests/e2e_test.rs`** (`test_subnet_detail_queries`).

**On-chain:** `SubtensorModule::update_symbol(origin, netuid, symbol)`

**Source map:** `SubnetCommands::SetSymbol` in `src/cli/subnet_cmds.rs`; extrinsic `set_subnet_symbol` in `src/chain/extrinsics.rs`.

### subnet trim
Lower the subnet’s **max allowed UIDs** on-chain (subnet owner only). Neurons above the new cap can be pruned by the runtime; the CLI submits `SubtensorModule::sudo_set_max_allowed_uids` after an interactive confirm (skipped with **`--yes`**).

```bash
agcli subnet trim --netuid 1 --max-uids 256
```

**Discoverability:** `agcli subnet --help` lists **`trim`** with **`--netuid`** and **`--max-uids`**. Install the binary and run **`agcli subnet trim --help`**; owner context in **`agcli explain --topic subnet-owner`**. The same cap can be set via **`agcli subnet set-param --netuid N --param max_allowed_uids --value …`** (see [`hyperparameters.md`](../hyperparameters.md)).

**Errors:** Unknown / inactive netuid → **`require_subnet_exists`** → **`Subnet N not found`** / `agcli subnet list` → exit **12** (validation), **before** wallet unlock — same preflight class as **`subnet set-param`**, **`subnet set-symbol`**, **`subnet start`**, **`subnet register-neuron`**, **`subnet pow`**, **`subnet dissolve`**, **`subnet root-dissolve`**, **`subnet terminate-lease`**, **`subnet set-mechanism-count`**, **`subnet set-emission-split`**, etc. Declining the confirmation prompt prints **`Cancelled.`** and exits **0**. On-chain rejections (**`NotSubnetOwner`**, **`TrimmingWouldExceedMaxImmunePercentage`** (see `src/chain/mod.rs` dispatch text), etc.) use normal **chain** exit classification after submit.

**Read path:** `agcli subnet show --netuid N` reports **`max_n`** (metagraph capacity); it maps from on-chain **`max_allowed_uids`** — compare with **`subnet_show.max_n`** in **`tests/e2e_test.rs`** (`test_subnet_detail_queries`).

**On-chain:** `SubtensorModule::sudo_set_max_allowed_uids(origin, netuid, max)` (via `submit_raw_call` in the CLI).

**Source map:** `SubnetCommands::Trim` in `src/cli/subnet_cmds.rs`.

### subnet set-mechanism-count
Set how many **emission mechanisms** the subnet uses (subnet owner only). Verify the current value with **`agcli subnet mechanism-count --netuid N`** (same storage read the CLI uses after preflight).

```bash
agcli subnet set-mechanism-count --netuid 1 --count 2
```

**Discoverability:** `agcli subnet --help` lists **`set-mechanism-count`** next to the read-only **`mechanism-count`** / **`emission-split`** tools. After install, **`agcli subnet set-mechanism-count --help`** shows **`--netuid`**, **`--count`**, and global wallet/network flags. Owner context: **`agcli explain --topic subnet-owner`** (Phase 6).

**Errors:** Unknown / inactive netuid → **`require_subnet_exists`** → **`Subnet N not found`** / `agcli subnet list` → exit **12** (validation), **before** wallet unlock — same preflight class as **`subnet set-param`**, **`subnet set-symbol`**, **`subnet trim`**, **`subnet start`**, **`subnet register-neuron`**, **`subnet pow`**, **`subnet dissolve`**, **`subnet root-dissolve`**, **`subnet terminate-lease`**, **`subnet set-emission-split`**, etc. On-chain rejections (**`NotSubnetOwner`**, invalid mechanism count for the runtime, etc.) use normal **chain** exit classification after submit.

**Read path / e2e:** Readers use **`Client::get_mechanism_count`** after **`require_subnet_exists`** (`src/chain/queries.rs`). **`tests/e2e_test.rs`** logs **`subnet_mechanism_count`**, then **`subnet_owner_mechanism_writes`** — documents that owner **`set-mechanism-count`** / **`set-emission-split`** share the same **`get_subnet_info`** preflight before unlock (no extrinsic submit in e2e).

**On-chain:** `SubtensorModule::sudo_set_mechanism_count(origin, netuid, count)` (via `submit_raw_call`).

**Source map:** `SubnetCommands::SetMechanismCount` in **`src/cli/subnet_cmds.rs`**.

### subnet set-emission-split
Set **comma-separated u16 weights** across mechanisms (subnet owner only). The CLI parses and validates **`--weights`** locally (**non-empty**, **sum > 0**, each **≤ u16::MAX**) before any RPC preflight; then **`require_subnet_exists`** runs **before** wallet unlock. Interactive **Proceed?** is skipped with global **`--yes`**. Preview the live split with **`agcli subnet emission-split --netuid N`**.

```bash
agcli subnet set-emission-split --netuid 1 --weights "50,50"
agcli subnet set-emission-split --netuid 1 --weights "70,20,10" --yes
```

**Discoverability:** `agcli subnet --help` → **`set-emission-split`**. **`agcli subnet set-emission-split --help`** documents **`--weights`** and confirm behavior.

**Errors:** Malformed **`--weights`** (non-numeric token, empty segment) → exit **1** with a short parse tip. **`validate_emission_weights`** failures (zero total, empty list) → exit **1** — **before** subnet preflight or wallet. Unknown / inactive netuid → **`require_subnet_exists`** → exit **12** (validation) **after** local weight validation — same **`Subnet N not found`** class as **`subnet set-mechanism-count`**, **`subnet set-param`**, **`subnet trim`**, etc. Declining **Proceed?** prints **`Cancelled.`** and exits **0**. On-chain rejections use normal **chain** exits after submit.

**Read path / e2e:** **`Client::get_emission_split`** backs **`subnet emission-split`**; e2e logs **`subnet_emission_split`** alongside **`subnet_owner_mechanism_writes`** (see **`set-mechanism-count`** above).

**On-chain:** `SubtensorModule::sudo_set_mechanism_emission_split(origin, netuid, weights)` (via `submit_raw_call`).

**Source map:** `SubnetCommands::SetEmissionSplit` in **`src/cli/subnet_cmds.rs`**; **`validate_emission_weights`** in **`src/cli/helpers.rs`**.

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `SubnetNotExists` | Invalid netuid | Check `agcli subnet list` |
| `NotSubnetOwner` | Not the subnet owner | Use owner coldkey |
| `SubnetLimitReached` | Max subnet count reached | Wait for a subnet to be pruned |
| `TooManyRegistrationsThisBlock` | Registration flood | Wait 1+ blocks |
| `SubNetRegistrationDisabled` | Subnet has registration off | Check hyperparams |

## Source Code
**agcli handler**: [`src/cli/subnet_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/subnet_cmds.rs) — `handle_subnet()` at L9, subcommands: List L17, Show L115, Hyperparams L182, Metagraph L264, Register L472, RegisterNeuron L480, Pow L492, Dissolve L540, Watch L561, Monitor L567, Health (handle_subnet_health) L2071, Emissions (handle_subnet_emissions) L2191, Cost (handle_subnet_cost) L2298, Probe (handle_subnet_probe) L2388, Commits L721, SetParam L724, SetSymbol L738, Trim L768, Start L821, CheckStart L796, EmissionSplit L748, MechanismCount L836, SetMechanismCount L846, SetEmissionSplit L864, CacheLoad L577, CacheList L635, CacheDiff L660, CachePrune L704, Liquidity (handle_subnet_liquidity) L696, Snipe L959 (handle_snipe L1064, handle_snipe_watch L1310)

**Subtensor pallet**:
- [`subnets/registration.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/registration.rs) — `register_network`, `burned_register`, `register` (PoW)
- [`subnets/subnet.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/subnet.rs) — `schedule_dissolve_network`, `start_call`, subnet lifecycle
- [`subnets/mechanism.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/mechanism.rs) — emission mechanisms, mechanism counts
- [`subnets/symbols.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/symbols.rs) — `update_symbol`
- [`subnets/uids.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/uids.rs) — UID management, trim
- [`subnets/serving.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/serving.rs) — axon serving
- [`subnets/weights.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/subnets/weights.rs) — weight commit/reveal
- [`macros/dispatches.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/dispatches.rs) — all dispatch entry points
- [`macros/events.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/events.rs) — event definitions
- [`macros/errors.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/errors.rs) — error definitions

## Related Commands
- `agcli stake add --netuid N` — Stake on a subnet
- `agcli weights set --netuid N` — Set weights on a subnet
- `agcli view dynamic` — See all subnet prices and pools
- `agcli explain --topic subnets` — What subnets are
- `agcli explain --topic hyperparams` — Hyperparameters reference
