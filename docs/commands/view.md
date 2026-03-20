# view — Query & Analytics Commands

Read-only commands for querying chain state, analytics, and account information. No wallet unlock required for most commands.

## view portfolio — Full coldkey portfolio (read-only)

Aggregates **free TAO**, **total staked** (TAO equivalent), and **per-subnet positions** (alpha, hotkey, subnet name, price) for a coldkey. Uses the default wallet coldkey or **`--address`**. Supports **`--at-block`**, **`--live`**, and global **`--output json|csv`**. No hot/cold unlock — only reads the default coldkey from disk when **`--address`** is omitted.

**Discoverability:** `agcli view portfolio --help`; Tier 1 in [`docs/llm.txt`](../llm.txt); `agcli explain --topic ow` (Phase 6) references the e2e log name; View row in `llm.txt` → this file.

### After `cargo install`

```bash
cargo install --git https://github.com/unconst/agcli
agcli view portfolio
agcli view portfolio --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli --output json view portfolio --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli --output csv view portfolio
agcli view portfolio --at-block 100
agcli --network archive view portfolio --at-block 3500000 --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli --live 30 view portfolio
```

### Read path (RPC / runtime API)

Order matches [`ViewCommands::Portfolio`](https://github.com/unconst/agcli/blob/main/src/cli/view_cmds.rs) in `src/cli/view_cmds.rs` (`handle_view`, `Portfolio` branch):

1. **`connect`** (global network / endpoint — same as other view commands).
2. **`resolve_and_validate_coldkey_address`** — if **`--address`** is set, **`validate_ss58(..., "portfolio --address")`**; else coldkey from wallet (`src/cli/helpers.rs`). Unresolved / empty coldkey bails before RPC (same pattern as `agcli balance` / `agcli stake list`).
3. **If `--at-block`:** **`get_block_hash(block)`** → **`try_join!(get_balance_at_block(addr, hash), get_stake_for_coldkey_at_block(addr, hash))`** — compact JSON (see below); no dynamic-info merge on this path.
4. **Else if `--live`:** polling loop calling **`fetch_portfolio`** each interval (`src/queries/portfolio.rs`, `src/live.rs`).
5. **Else (latest):** **`handle_portfolio`** → **`fetch_portfolio`**: **`pin_latest_block`** → **`try_join!(get_balance_at_hash, get_stake_for_coldkey_at_block, get_all_dynamic_info_at_block)`** — dynamic info fills subnet name and price; if that RPC fails, the code logs a warning and treats dynamic data as empty (`src/queries/portfolio.rs`).

### JSON shapes

**Latest** (`--output json`) — serialized [`Portfolio`](https://github.com/unconst/agcli/blob/main/src/queries/portfolio.rs): `coldkey_ss58`, `free_balance`, `total_staked`, `positions` (`netuid`, `subnet_name`, `hotkey_ss58`, `alpha_stake`, `tao_equivalent`, `price`). Field names/types follow `serde` on `Balance` and the struct definitions in the crate.

**`--at-block`** — object built in `handle_portfolio_at_block`: `address`, `block`, `free_balance_rao` / `free_balance_tao`, `total_staked_rao` / `total_staked_tao`, `stakes` (`hotkey`, `netuid`, `stake_rao`, `stake_tao`).

### Exit codes

| Code | When |
|------|------|
| **0** | Successful query (including **empty** positions / stakes). |
| **2** | Clap / invalid global flags. |
| **10** | Network / WebSocket failure on `connect` or hard RPC errors. |
| **12** | Validation: invalid **`--address`** (SS58) per [`classify`](https://github.com/unconst/agcli/blob/main/src/error.rs). |
| **15** | Timeout when applicable. |
| **1** | Generic: e.g. **`Block N not found`** for **`--at-block`**, could not resolve coldkey when no **`--address`**, pruned state at a historical height, or uncategorized errors. |

Messages for bad **`--address`** include **`portfolio --address`** — [`hint`](https://github.com/unconst/agcli/blob/main/src/error.rs) points at **`docs/commands/view.md`**.

### E2E

Log lines **`view_portfolio_preflight`** in Phase 20 [`test_view_portfolio_preflight`](https://github.com/unconst/agcli/blob/main/tests/e2e_test.rs): **`validate_ss58`** with label **`portfolio --address`**, **`pin_latest_block`** → **`try_join!(get_balance_at_hash, get_stake_for_coldkey_at_block, get_all_dynamic_info_at_block)`**, then head **`get_block_hash`** + **`try_join!(get_balance_at_block, get_stake_for_coldkey_at_block)`** — mirrors latest **`fetch_portfolio`** and **`--at-block`**. Broader view RPC checks remain in Phase 21 **`test_view_queries`**.

### Related

- `agcli balance` — free TAO only
- `agcli stake list` — stakes only (no dynamic price/name merge)
- `agcli diff portfolio` — compare two block heights

---

## Subcommands

### view portfolio

See **[view portfolio](#view-portfolio--full-coldkey-portfolio-read-only)** (install examples, read path, JSON, **`--at-block`**, **`--live`**, exit **12** for bad **`--address`**, e2e).

**Read-only:** `System::Account`, stake storage, dynamic info (latest path). No extrinsic.

### view network
Network-wide overview: total issuance, total stake, emission rate, block height, active subnets.

```bash
agcli view network [--at-block N]
```

### view dynamic
Dynamic TAO info for all subnets: prices, AMM pool depths, volumes, emissions.

```bash
agcli view dynamic [--at-block N]
# CSV: netuid,name,price,tao_in,alpha_in,alpha_out,emission
```

### view neuron
Full detail for a single neuron: stake, weights, bonds, axon, rank, trust, consensus.

```bash
agcli view neuron --netuid 1 --uid 0 [--at-block N]
```

### view validators
Top validators by stake across subnets.

```bash
agcli view validators [--netuid N] [--limit 50] [--at-block N]
```

### view history
Recent extrinsics for an account (via Subscan API).

```bash
agcli view history [--address SS58] [--limit 20]
```

Fetches from `https://bittensor.api.subscan.io/api/v2/scan/extrinsics`.

### view account
Comprehensive account explorer: balance, stakes, identities, proxy delegations, recent history.

```bash
agcli view account [--address SS58] [--at-block N]
```

### view subnet-analytics
Detailed subnet analysis: neuron distribution, weight patterns, emission concentration.

```bash
agcli view subnet-analytics --netuid 1
```

### view staking-analytics
APY estimates and yield analysis for staked positions.

```bash
agcli view staking-analytics [--address SS58]
```

### view swap-sim
Simulate an AMM swap without executing. Shows expected output, fees, and price impact.

```bash
agcli view swap-sim --netuid 1 --tao 10.0
agcli view swap-sim --netuid 1 --alpha 1000.0
# JSON: {"alpha_out", "tao_fee", "alpha_fee", "price_impact_pct"}
```

Use this before `stake add` to estimate slippage.

### view nominations
Show who delegates to a specific validator.

```bash
agcli view nominations --hotkey-address SS58
```

## Live Mode
Any view command can be polled continuously with `--live`:

```bash
agcli --live view dynamic          # poll with delta tracking
agcli --live 30 view portfolio     # poll every 30s
agcli --live subnet metagraph --netuid 1  # track neuron changes
```

## Historical Queries
Most view commands support `--at-block N`:

```bash
agcli --network archive view portfolio --at-block 3000000
agcli --network archive view dynamic --at-block 3500000
```

Requires an archive node for blocks beyond ~256 block pruning window.

## Source Code
**agcli handler**: [`src/cli/view_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/view_cmds.rs) — `handle_view()` at L9, subcommands: Portfolio L17, Network L27, Dynamic L28, Neuron L37, Validators L42, History L47, Account L51, SubnetAnalytics L55, StakingAnalytics L58, SwapSim L62, Nominations L65. Audit: `handle_audit()` at L1287.

**On-chain**: read-only queries against `System::Account`, `SubtensorModule` storage maps (Stake, Alpha, DynamicInfo, SubnetHyperparams, Metagraph, etc.). History uses [Subscan API](https://bittensor.api.subscan.io/api/v2/scan/extrinsics).

## Related Commands
- `agcli balance` — Simple balance check
- `agcli stake list` — Stake positions only
- `agcli subnet metagraph` — Full metagraph data
- `agcli explain --topic amm` — How Dynamic TAO AMM works
