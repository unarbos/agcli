# block — Block Explorer

Query finalized block information. Useful for debugging, auditing, and historical analysis.

**Discoverability:** `agcli block --help`; `agcli explain --topic archive` lists **`block latest`** / **`block info`** / **`block range`** with archive context.

No wallet or coldkey is required — read-only RPC.

## Subcommands

### block latest

Show the latest finalized block: height, hash, extrinsic count, and timestamp (when present).

```bash
agcli block latest
agcli block latest --output json
```

**Read path** (matches [`handle_block`](https://github.com/unconst/agcli/blob/main/src/cli/block_cmds.rs) `BlockCommands::Latest`): `get_block_number` → `u32` conversion → `get_block_hash` → `try_join!`(`get_block_extrinsic_count`, `get_block_timestamp`).

**JSON** (`--output json`): `block_number` (u64), `block_hash`, `extrinsic_count`, optional `timestamp_ms` and RFC3339 `timestamp` when the node returns a timestamp inherent.

**Errors / exit codes:** RPC or transport failures classify as network (**10**), timeouts as **15**, and other failures per [`src/error.rs`](https://github.com/unconst/agcli/blob/main/src/error.rs). If the chain head ever exceeded `u32::MAX`, the CLI would bail before hash lookup (exit **1** — not expected on normal networks).

**E2E:** Log line **`block_latest_preflight`** in Phase 20 `test_block_queries` (`tests/e2e_test.rs`) mirrors the Latest branch RPC order above.

### block info

Show details for a specific block (header fields, extrinsic count, timestamp when present).

```bash
agcli block info --number 4000000
agcli block info --number 4000000 --output json
```

**Read path** (matches [`handle_block`](https://github.com/unconst/agcli/blob/main/src/cli/block_cmds.rs) `BlockCommands::Info`): `get_block_hash(number)` → `try_join!`(`get_block_header`, `get_block_extrinsic_count`, `get_block_timestamp`).

**JSON** (`--output json`): `block_number`, `block_hash`, `parent_hash`, `state_root`, `extrinsic_count`, optional `timestamp_ms` and RFC3339 `timestamp`.

**Errors / exit codes:** Same read-only RPC classification as **`block latest`** — network **10**, timeout **15**, other failures **1** per [`src/error.rs`](https://github.com/unconst/agcli/blob/main/src/error.rs). Invalid `--number` / missing flag surface as clap validation (exit **2**).

**E2E:** Log line **`block_info_preflight`** in Phase 20 `test_block_queries` (`tests/e2e_test.rs`) mirrors the Info branch RPC order above and cross-checks head against **`block_latest_preflight`**.

### block range

Query a range of blocks (max 1000). Good for scanning metadata before using **`agcli diff`** on specific heights.

```bash
agcli block range --from 3999900 --to 4000000
agcli block range --from 3999900 --to 4000000 --output json
```

**Read path** (matches `BlockCommands::Range`): local checks (`--from` ≤ `--to`, span ≤ 1000) → `futures::future::try_join_all` over `get_block_hash` for each height → `try_join_all` of per-hash `try_join!`(`get_block_extrinsic_count`, `get_block_timestamp`).

**Output:** Table or JSON rows with block height, hash, timestamp string, extrinsic count (same column semantics as the CLI `render_rows` path).

**Validation:** `--from` must be ≤ `--to`; span must be ≤ 1000 blocks — otherwise the CLI bails with exit **1** (message explains the constraint).

**Errors / exit codes:** RPC failures while fetching hashes or per-block details classify like other block commands (**10** / **15** / **1**). Range validation failures are **1** (not **12** — no subnet).

**E2E:** Log line **`block_range_preflight`** in Phase 20 `test_block_queries` mirrors the two-stage concurrent batching above on a short tail range (last three blocks).

## Source Code

**agcli handler**: [`src/cli/block_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/block_cmds.rs) — `handle_block()`: `Info` ~L15, `Range` ~L54, `Latest` ~L127.

**On-chain**: read-only queries using subxt block APIs (`get_block`, `get_block_hash`).

## Related Commands

- `agcli diff` — Compare chain state between two blocks
- `agcli subscribe blocks` — Watch blocks in real-time
- `agcli --network archive block info --number N` — Query historical blocks
