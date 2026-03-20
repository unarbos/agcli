# diff — Historical state comparison

Compare read-only chain snapshots at **two block heights** (balance, subnet metrics, network totals, or metagraph deltas). Pair with **`agcli block`** to pick heights, then drill in with **`diff`**.

**Discoverability:** `agcli diff --help`; `agcli explain --topic diff` (alias topic **`compare`**) lists examples and archive notes.

**Flags:** Every subcommand uses **`--block1`** and **`--block2`** (both required, `u32`). There is no `--from-block` / `--to-block` spelling.

For heights outside your node’s state window, use **`--network archive`** (or an archive **`--endpoint`**) — same pruning semantics as **`agcli balance --at-block`** (see errors below).

## Subcommands

### diff portfolio

Compare coldkey **free balance** and **aggregate stake positions** between two blocks (default address: wallet coldkey; override with **`--address`**).

```bash
agcli diff portfolio --block1 100 --block2 200
agcli diff portfolio --block1 100 --block2 200 --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli diff portfolio --block1 100 --block2 200 --output json
```

**Read path** (matches [`handle_diff`](https://github.com/unconst/agcli/blob/main/src/cli/block_cmds.rs) `DiffCommands::Portfolio`): `try_join!`(`get_block_hash(block1)`, `get_block_hash(block2)`) → `try_join!`(`get_balance_at_block`, `get_stake_for_coldkey_at_block` for each hash).

**JSON** (`--output json`): `address`, `block1`, `block2`, `balance_tao`, `balance_diff_tao`, `total_stake_tao`, `stake_diff_tao`, `total_tao`, `total_diff_tao`, stake position counts.

**Errors / exit codes**

- Missing **`--address`** when no default wallet / empty resolved address → bail **1** (message suggests **`--address`**).
- Invalid SS58 → **12** (validation).
- Pruned / unknown block state → message may include archive hint; classified per [`src/error.rs`](https://github.com/unconst/agcli/blob/main/src/error.rs) (often **1** with context, or **10** / **15** for transport).
- Otherwise same read-only RPC classification as other block-scoped queries: network **10**, timeout **15**, generic **1**.

**E2E:** Log **`diff_portfolio_preflight`** in Phase 20 `test_diff_queries` (`tests/e2e_test.rs`).

### diff subnet

Compare **dynamic subnet** fields (TAO in pool, price, emission, tempo, owner hotkey) between two blocks.

```bash
agcli diff subnet --netuid 1 --block1 100 --block2 200
agcli diff subnet --netuid 1 --block1 100 --block2 200 --output json
```

**Read path** (matches `DiffCommands::Subnet`): `try_join!`(`get_block_hash(block1)`, `get_block_hash(block2)`) → `try_join!`(`get_dynamic_info_at_block(netuid, hash1)`, same for `hash2`). If either snapshot returns **`None`**, the CLI bails with **`Subnet {netuid} not found at block {N}`** (treated as validation **12** — see below).

**JSON:** `netuid`, `name` (from newer block’s info), `block1`, `block2`, `tao_in`, `price`, `emission` arrays and diffs.

**Errors / exit codes**

- **`Subnet N not found at block B`** (no dynamic info at that height) → **12** (`subnet` + `not found` in [`classify`](https://github.com/unconst/agcli/blob/main/src/error.rs)); hint lists **`agcli diff subnet`** / **`agcli diff metagraph`** among other `--netuid` readers.
- Missing **`--netuid`** → clap **2**.
- Pruned state / RPC → **1** / **10** / **15** like other historical reads.

**E2E:** Log **`diff_subnet_preflight`** in Phase 20 `test_diff_queries`.

### diff network

Compare **total issuance**, **total stake**, implied **staking ratio**, and **subnet count** between two blocks.

```bash
agcli diff network --block1 100 --block2 200
agcli diff network --block1 100 --block2 200 --output json
```

**Read path** (matches `DiffCommands::Network`): `try_join!`(`get_block_hash` ×2) → one `try_join!` of six calls: `get_total_issuance_at_block` ×2, `get_total_stake_at_block` ×2, `get_all_subnets_at_block` ×2.

**JSON:** `block1`, `block2`, `total_issuance_tao`, `total_stake_tao`, `staking_ratio_pct`, `subnet_count`.

**Errors / exit codes:** Historical RPC / pruning / network same as **`diff portfolio`** (**1** / **10** / **15**). No subnet **12** path.

**E2E:** Log **`diff_network_preflight`** in Phase 20 `test_diff_queries`.

### diff metagraph

Load **lite metagraph** at both heights and print **neurons that changed** (stake / emission / incentive deltas above thresholds, hotkey replacement, or **new** UIDs).

```bash
agcli diff metagraph --netuid 1 --block1 100 --block2 200
agcli diff metagraph --netuid 1 --block1 100 --block2 200 --output json
```

**Read path** (matches `DiffCommands::Metagraph`): `try_join!`(`get_block_hash` ×2) → `try_join!`(`get_neurons_lite_at_block(netuid, hash1)`, same for `hash2`). Diff logic is local (HashMap by UID); output lists only changed or new neurons.

**JSON:** `netuid`, `block1`, `block2`, `neurons_block1`, `neurons_block2`, `changed`, `diffs` (array of change records).

**Errors / exit codes:** RPC / pruning like other diff readers. Empty metagraphs are valid (**0** success with empty human table or `"changed": 0` in JSON).

**E2E:** Log **`diff_metagraph_preflight`** in Phase 20 `test_diff_queries`.

## Source code

**agcli handler:** [`src/cli/block_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/block_cmds.rs) — `handle_diff()`: `Portfolio` ~L176, `Subnet` ~L279, `Network` ~L382, `Metagraph` ~L465.

**On-chain:** read-only storage / runtime APIs at pinned block hashes (no extrinsics).

## Related commands

- `agcli block latest` / `block info` / `block range` — Pick safe block heights before diffing.
- `agcli subnet metagraph --diff` — Live head vs pinned block (different UX than **`diff metagraph`**).
- `agcli subnet cache-diff` — Compare **cached** metagraph files, not arbitrary on-chain heights.
- `agcli explain --topic diff` — Conceptual overview and examples.
