# balance — Free TAO balance

Query **free** TAO balance for an account (default wallet coldkey or `--address`). Optional **historical** query at a block height, or **watch** mode with an optional **threshold** alert.

**Discoverability:** `agcli balance --help`; Tier 1 in [`docs/llm.txt`](../llm.txt) maps “check my balance” → `agcli balance`; `agcli explain` Phase 6 lists the command with the e2e log name; this file is linked from the command table in `llm.txt`.

## Usage

### One-shot (latest state)

```bash
agcli balance
agcli balance --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli --output json balance --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

### Historical (`--at-block`)

```bash
agcli balance --at-block 4000000
agcli balance --at-block 4000000 --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli --network archive balance --at-block 3500000 --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

Pruned nodes only retain recent state (~256 blocks). Older heights need an **archive** endpoint (`--network archive` or `--endpoint <archive-ws>`). Pruned-state errors from storage are wrapped with a hint that mentions `agcli balance --at-block … --network archive` (see [`annotate_at_block_error`](https://github.com/unarbos/agcli/blob/main/src/chain/mod.rs) in `src/chain/mod.rs`).

### Watch mode

```bash
agcli balance --watch              # poll every 60s (default interval)
agcli balance --watch 30           # poll every 30s
agcli balance --watch --threshold 10.0   # annotate when free balance drops below 10 τ
agcli --output json balance --watch 30 --threshold 1.0 --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

Watch mode runs until **Ctrl+C**. A failed poll prints a warning, waits, and retries (same spirit as the implementation loop in `commands.rs`).

## Read path (RPC / storage)

Order matches [`Commands::Balance`](https://github.com/unarbos/agcli/blob/main/src/cli/commands.rs) in `src/cli/commands.rs` (201–325):

1. **`validate_threshold`** when `--threshold` is set (`src/cli/helpers.rs`).
2. **`connect`** (global network / endpoint).
3. **`resolve_and_validate_coldkey_address`** — optional `validate_ss58` on `--address`, else coldkey from wallet (`src/cli/helpers.rs`).
4. **If `--at-block`:** `get_block_hash(block)` → `get_balance_at_block(address, hash)` (`src/chain/mod.rs` — `System::Account` at pinned hash).
5. **Else if `--watch`:** loop: `get_balance_ss58` each interval; optional `below_threshold` in JSON or human suffix.
6. **Else:** `get_balance_ss58` (latest block inside `get_balance`).

## JSON shapes

**One-shot**

```json
{"address":"5G…","balance_rao":1234567890,"balance_tao":1.23456789}
```

**`--at-block`**

```json
{"address":"5G…","block":4000000,"block_hash":"0x…","balance_rao":…,"balance_tao":…}
```

**`--watch` + `--output json`** (one object per successful poll)

```json
{"address":"5G…","balance_rao":…,"balance_tao":…,"below_threshold":false,"timestamp":"2024-01-01T00:00:00Z"}
```

## Exit codes

| Code | When |
|------|------|
| **0** | Successful one-shot or historical query; watch mode until Ctrl+C (including continued run after transient RPC errors in the loop). |
| **2** | Clap / invalid global flags (unknown option, bad value where clap parses). |
| **10** | Network / WebSocket failure (e.g. cannot connect, connection reset) — typical for failed `connect` or hard RPC errors on one-shot paths. |
| **12** | Validation: invalid **`--address`** (SS58), or other input classified as validation in [`src/error.rs`](https://github.com/unarbos/agcli/blob/main/src/error.rs). |
| **15** | Timeout (when a timeout applies to the operation). |
| **1** | Generic / other: e.g. negative or non-finite **`--threshold`**, **`--at-block`** hash lookup failure (`Block N not found`), unresolved coldkey when no **`--address`** (wallet empty / missing), or uncategorized errors. |

Classification follows [`agcli::error::classify`](https://github.com/unarbos/agcli/blob/main/src/error.rs). **`--threshold`** validation failures include a [`hint`](https://github.com/unarbos/agcli/blob/main/src/error.rs) for exit **12**.

## Common issues

| Symptom | Cause | What to do |
|---------|-------|------------|
| Pruned / unknown block / state discarded on **`--at-block`** | Full node without archive history | Use **`--network archive`** or an archive **`--endpoint`** (see [`explain` archive topic](https://github.com/unarbos/agcli/blob/main/src/utils/explain.rs) and `agcli explain archive`). |
| Cannot resolve coldkey | No **`--address`** and default wallet has no coldkey | Pass **`--address`**, or create a wallet: `agcli wallet create` |
| Invalid SS58 | Bad checksum or prefix | Fix address; local devnets often use prefix **42**. |

## E2E

Log lines **`balance_preflight`** in Phase 20 [`test_balance_preflight`](https://github.com/unarbos/agcli/blob/main/tests/e2e_test.rs): **`get_balance_ss58`** (Alice), then **`get_block_number`** → **`get_block_hash`** → **`get_balance_at_block`** at head — same RPC sequence as the CLI’s one-shot and **`--at-block`** branches (watch mode is long-running and is covered by `cli_test` parsers).

## Source code

**Handler:** [`src/cli/commands.rs`](https://github.com/unarbos/agcli/blob/main/src/cli/commands.rs) — `Commands::Balance`.

**Chain:** [`src/chain/mod.rs`](https://github.com/unarbos/agcli/blob/main/src/chain/mod.rs) — `get_balance`, `get_balance_ss58`, `get_block_hash`, `get_balance_at_block`.

## Related commands

- `agcli transfer` / `transfer-all` / `transfer-keep-alive` — move TAO
- `agcli stake list` — staked positions (separate from free balance)
- `agcli view portfolio` — free balance plus all stakes
- `agcli view account` — full account view
- `agcli diff portfolio` — balance (and stake) delta between two blocks
- `agcli doctor` — connectivity smoke test after install
