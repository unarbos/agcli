# subscribe — Real-Time Block & Event Streaming

Watch finalized blocks or decode and print Subtensor events as they finalize. Useful for monitoring, alerting, and integration pipelines. **No wallet** is required — only a WebSocket-capable RPC endpoint.

**Discoverability:** `agcli subscribe --help` / `agcli subscribe blocks --help` / `agcli subscribe events --help`. `agcli explain` mentions **`subscribe events`** in the quick-start tips; see also `docs/llm.txt` (Subscribe row).

## Subcommands

### subscribe blocks

Stream each **finalized** block (number, hash, extrinsic count). Runs until **Ctrl+C**; reconnects automatically if the WebSocket drops (with backoff, up to five attempts per failure before exiting).

```bash
agcli subscribe blocks
agcli subscribe blocks --output json
```

**Read path** (matches [`subscribe_blocks`](https://github.com/unarbos/agcli/blob/main/src/events.rs) / [`handle_subscribe`](https://github.com/unarbos/agcli/blob/main/src/cli/network_cmds.rs) `SubscribeCommands::Blocks`): `blocks().subscribe_finalized()` → for each block: `extrinsics().await` for count (human table) or JSON lines with `block`, `hash`, `extrinsics`.

**JSON** (`--output json`): one JSON object per line per block (`block`, `hash`, `extrinsics`). Gap warnings use `warning: "gap_detected"` with `missed_from` / `missed_to` / `missed_count`.

**Errors / exit codes:** Invalid global flags → clap **2**. Repeated subscription failures after retries → bail with transport message → typically **10** (network) or **15** (timeout) per [`src/error.rs`](https://github.com/unarbos/agcli/blob/main/src/error.rs). Other uncategorized failures → **1**.

**E2E:** Phase 26 `test_subscribe_blocks` in `tests/e2e_test.rs` — reads several finalized blocks via the same `subscribe_finalized` entry path.

### subscribe events

Stream **decoded** runtime events from finalized blocks, with optional category, `--netuid`, and `--account` filters. Same long-running / Ctrl+C / reconnect behavior as **`subscribe blocks`**.

```bash
agcli subscribe events
agcli subscribe events --filter staking
agcli subscribe events --filter all --netuid 1
agcli subscribe events --filter transfer --account 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty
agcli subscribe events --output json --filter weights
```

**Validation** (before subscribing — matches [`handle_subscribe`](https://github.com/unarbos/agcli/blob/main/src/cli/network_cmds.rs)): `validate_event_filter` on `--filter`; for `--account`, `validate_ss58` (invalid / empty address → exit **12**).

**Read path** (matches [`subscribe_events_inner`](https://github.com/unarbos/agcli/blob/main/src/events.rs)): `blocks().subscribe_finalized()` → per block `events().await` → iterate events → `EventFilter::matches` on pallet/variant → optional structured `--netuid` / `--account` filtering via field extraction → print line (human) or JSON (`block`, `hash`, `pallet`, `event`, `fields`).

**Errors / exit codes:** Unknown `--filter` → **12** (`Invalid event filter`). Bad `--account` → **12** (invalid address text). Clap **2** for missing/invalid global flags. Persistent subscription / stream failures after retries → **10** / **15** / **1** like **`subscribe blocks`**. Undecodable events in a block are skipped with a warning (process keeps running, exit **0** until you interrupt).

## Filter categories (`--filter`)

Values are case-insensitive. Aliases match [`EventFilter` `FromStr`](https://github.com/unarbos/agcli/blob/main/src/events.rs) and [`validate_event_filter`](https://github.com/unarbos/agcli/blob/main/src/cli/helpers.rs).

| Filter (aliases) | What is shown |
|------------------|---------------|
| `all` | Every decoded event |
| `staking` (`stake`) | Subtensor stake add/remove/move/swap/recycle/burn/root claim/auto-stake, … |
| `registration` (`register`, `reg`) | Neuron/subnet registration, PoW register, bulk register, … |
| `transfer` (`transfers`) | `Balances` pallet transfer-related events |
| `weights` (`weight`) | Weights set/commit/reveal, CR/timelock batches, commit-reveal config, … |
| `subnet` (`subnets`) | Hyperparams, identity, network add/remove, tempo, dissolve schedule, lease, symbol, … |
| `delegation` (`delegate`, `delegates`) | Delegate added, take changes, children |
| `keys` (`key`) | Hotkey/coldkey swap, EVM associate, chain identity |
| `swap` (`dex`, `liquidity`) | Swap pallet liquidity / swap / fees |
| `governance` (`gov`, `sudo`, `safemode`) | Safe mode, sudo, scheduler, proxy, multisig |
| `crowdloan` (`crowdloans`, `fund`) | Crowdloan pallet lifecycle events |

`--netuid N` keeps only events whose **decoded fields** include that netuid (events without a netuid field are dropped when this flag is set). `--account SS58` keeps only events that mention that address in composite fields.

## Source code

- **CLI handler:** [`src/cli/network_cmds.rs`](https://github.com/unarbos/agcli/blob/main/src/cli/network_cmds.rs) — `handle_subscribe()` (~L366).
- **Streaming logic:** [`src/events.rs`](https://github.com/unarbos/agcli/blob/main/src/events.rs) — `subscribe_blocks`, `subscribe_events_filtered`, filters, gap detection.
- **Filter validation:** [`src/cli/helpers.rs`](https://github.com/unarbos/agcli/blob/main/src/cli/helpers.rs) — `validate_event_filter`.

## Related commands

- `agcli subnet monitor` — Higher-level subnet monitoring with summaries
- `agcli subnet watch` — Tempo / weight-window focused TUI-style watch
- `agcli block latest` — One-shot head snapshot (no WebSocket stream)
