# stake — Staking Operations

Lock TAO on subnets behind hotkeys to earn emission rewards. Staking converts TAO into subnet-specific alpha tokens via the AMM pool.

## stake list — Positions per coldkey (read-only)

List **alpha stake positions** for a coldkey (default wallet coldkey or `--address`). Optional **historical** snapshot at a block height. No extrinsic; no wallet unlock unless the default coldkey must be read from disk.

**Discoverability:** `agcli stake list --help`; Tier 1 in [`docs/llm.txt`](../llm.txt) maps “View all stakes” → `agcli --output json stake list`; `agcli explain` Phase 6 lists the command with the e2e log name; this file is linked from the command table in `llm.txt`.

### Latest state

```bash
agcli stake list
agcli stake list --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli --output json stake list --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli --output csv stake list
```

With `--output json`, the CLI prints a **JSON array** of stake rows (serialized `StakeInfo`: `netuid`, `hotkey`, `coldkey`, `stake`, `alpha_stake`, …). With `--output csv`, the header is `netuid,hotkey,stake_rao,alpha_raw`. An empty portfolio yields an empty array/CSV body or the human line `No stakes found for …`.

### Historical (`--at-block`)

```bash
agcli stake list --at-block 4000000
agcli stake list --at-block 4000000 --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
agcli --network archive stake list --at-block 3500000 --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

Pruned nodes only retain recent state. Older heights need an **archive** endpoint (`--network archive` or `--endpoint <archive-ws>`). Runtime API failures at a pinned block are wrapped with **`annotate_at_block_error`** (same family of hints as `agcli balance --at-block` — see `src/chain/mod.rs` / `get_stake_for_coldkey_at_block` in `src/chain/queries.rs`).

## Read path (RPC / runtime API)

Order matches [`StakeCommands::List`](https://github.com/unconst/agcli/blob/main/src/cli/stake_cmds.rs) in `src/cli/stake_cmds.rs` (handler `handle_stake`, `List` branch):

1. **`connect`** (global network / endpoint) — same as other stake subcommands (`src/cli/commands.rs` dispatch).
2. **`resolve_and_validate_coldkey_address`** — if `--address` is set, **`validate_ss58(..., "stake list --address")`**; else coldkey from wallet (`src/cli/helpers.rs`). Empty / unresolved coldkey bails before RPC (same pattern as `agcli balance`).
3. **If `--at-block`:** `get_block_hash(block)` → **`get_stake_for_coldkey_at_block(&addr, hash)`** (`src/chain/queries.rs`).
4. **Else:** **`get_stake_for_coldkey(&addr)`** (latest via runtime API at head).
5. **Render:** `render_rows` — human table, JSON array, or CSV (`src/cli/helpers.rs`).

## Exit codes

| Code | When |
|------|------|
| **0** | Successful query (including **empty** stake list). |
| **2** | Clap / invalid global flags. |
| **10** | Network / WebSocket failure on `connect` or hard RPC errors. |
| **12** | Validation: invalid **`--address`** (SS58) and other input classified as validation in [`src/error.rs`](https://github.com/unconst/agcli/blob/main/src/error.rs). |
| **15** | Timeout when applicable. |
| **1** | Generic: e.g. **`Block N not found`** for **`--at-block`**, could not resolve coldkey when no **`--address`** and wallet has no coldkey, or uncategorized errors. |

Invalid **`--address`** messages include the label **`stake list --address`** — [`classify`](https://github.com/unconst/agcli/blob/main/src/error.rs) treats that substring as validation **12**; [`hint`](https://github.com/unconst/agcli/blob/main/src/error.rs) points at **`docs/commands/stake.md`**.

## E2E

Log lines **`stake_list_preflight`** in Phase 20 [`test_stake_list_preflight`](https://github.com/unconst/agcli/blob/main/tests/e2e_test.rs): **`validate_ss58`** with label **`stake list --address`** (explicit-address path), **`get_stake_for_coldkey`**, then **`get_block_number`** → **`get_block_hash`** → **`get_stake_for_coldkey_at_block`** at head — same RPC sequence as the CLI’s latest and **`--at-block`** branches. Deeper stake RPC coverage remains in Phase 5 **`test_stake_queries`**.

## Related

- `agcli balance` — free TAO (not staked)
- `agcli view portfolio` — balance + stakes + pricing
- `agcli diff portfolio` — stake map at two blocks

---

## stake add — Stake TAO on a subnet (wallet)

Lock **free TAO** from the wallet **coldkey** into **alpha** on a subnet for a chosen **hotkey** (default wallet hotkey or `--hotkey-address`). Uses the subnet AMM (`swap_tao_for_alpha`).

**Discoverability:** `agcli stake add --help`; Tier 1 in [`docs/llm.txt`](../llm.txt); `agcli explain` Phase 6 lists the e2e log name; this page is linked from the Stake row in `llm.txt`.

### After `cargo install`

```bash
agcli stake add --amount 10.0 --netuid 1 --password p --yes
agcli stake add --amount 1.0 --netuid 1 --hotkey-address 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty --password p --yes
agcli stake add --amount 5.0 --netuid 1 --max-slippage 2.0 --password p --yes
```

Global flags (`--network`, `--endpoint`, `--wallet-dir`, `--wallet`, `--mev`, …) apply like other write commands.

## Read path (validation → RPC preflight → submit)

Order matches [`StakeCommands::Add`](https://github.com/unconst/agcli/blob/main/src/cli/stake_cmds.rs) in `src/cli/stake_cmds.rs` (`handle_stake`, `Add` branch, lines 101–145):

1. **`connect`** (from `commands.rs` dispatch, same as other stake subcommands).
2. **`validate_netuid(netuid)`** — **SN0** rejected before wallet (`src/cli/helpers.rs`).
3. **`validate_amount(amount, "stake amount")`** — positive, finite TAO.
4. **`check_spending_limit(netuid, amount)`** — optional per-subnet / global caps from `agcli config` (`src/cli/helpers.rs`).
5. **`unlock_and_resolve`** — coldkey + hotkey SS58 (`src/cli/helpers.rs`).
6. **Balance (+ optional slippage) preflight:** if **`--max-slippage`** is set, **`try_join!(get_balance(&coldkey_pub), check_slippage(...))`**; otherwise **`get_balance(&coldkey_pub)`** alone. If free TAO &lt; amount, bail with **Insufficient balance** and a pointer to `agcli balance`. **`check_slippage`** (buy path) uses **`current_alpha_price`** + **`sim_swap_tao_for_alpha`** (runtime APIs in `src/chain/queries.rs`); aborts if estimated slippage exceeds the cap (or warns above ~2% when within cap).
7. **`add_stake_mev`** — extrinsic via `stake_op` (human **Tx:** line on success). **Note:** success output is **not** shaped by global `--output json` today (table/JSON apply to read-only stake commands such as **`stake list`**).

## Exit codes

| Code | When |
|------|------|
| **0** | Stake extrinsic submitted and finalized path OK. |
| **2** | Clap / invalid global flags. |
| **10** | Network / WebSocket failure on **`connect`** or hard RPC errors during preflight. |
| **11** | Auth: wallet / password / hotkey resolution (`unlock_and_resolve`). |
| **12** | Validation: invalid **`--netuid`** (e.g. **0**), invalid **`--amount`** (**`stake amount`** label in errors), **spending limit exceeded** (local config), and other messages classified in [`src/error.rs`](https://github.com/unconst/agcli/blob/main/src/error.rs). |
| **13** | Chain / client guardrails: **insufficient** free TAO before submit; **slippage** over **`--max-slippage`** (message contains **maximum allowed**); dispatch errors (**`NotEnoughBalanceToStake`**, **`HotKeyAccountNotExists`**, **`StakingRateLimitExceeded`**, **`InsufficientLiquidity`**, …). |
| **15** | Timeout when applicable. |
| **1** | Uncategorized errors. |

Invalid **`--amount`** messages use the **`stake amount`** label — **`classify`** → **12** with a **`hint`** pointing at **`docs/commands/stake.md`**.

## E2E

Log lines **`stake_add_preflight`** in Phase 20 [`test_stake_add_preflight`](https://github.com/unconst/agcli/blob/main/tests/e2e_test.rs): **`validate_netuid(1)`**, **`validate_amount`** with label **`stake amount`**, **`check_spending_limit`**, **`get_balance_ss58`**(Alice) ≥ amount, then **`try_join!(current_alpha_price, sim_swap_tao_for_alpha)`** — same RPC inputs as the **`--max-slippage`** branch’s **`check_slippage`** buy path. Full **`add_stake`** / **`remove_stake`** extrinsics remain in Phase 8 **`test_add_remove_stake`**.

## Subcommands

### stake add

See **[stake add](#stake-add--stake-tao-on-a-subnet-wallet)** (read path, slippage, exit codes, e2e).

**On-chain**: `SubtensorModule::add_stake(origin, hotkey, netuid, amount_staked)` — withdraw TAO from coldkey → `stake_into_subnet()` → AMM `swap_tao_for_alpha()` → alpha shares on **`Alpha(hotkey, coldkey, netuid)`**; events **`StakeAdded`**.

### stake remove
Unstake alpha from a subnet. Converts alpha → TAO via the AMM pool.

```bash
agcli stake remove --amount 5.0 --netuid 1 [--hotkey-address SS58] [--max-slippage 2.0]
```

**On-chain**: `SubtensorModule::remove_stake(origin, hotkey, netuid, amount_unstaked)`
- Flow: decrease alpha shares → `unstake_from_subnet()` → `swap_alpha_for_tao()` via AMM → deposit TAO to coldkey
- Events: `StakeRemoved(coldkey, hotkey, tao_amount, alpha_amount, netuid, block)`
- Errors: `NotEnoughStakeToWithdraw`, `StakingRateLimitExceeded`, `InsufficientLiquidity`

### stake list
See **[stake list](#stake-list--positions-per-coldkey-read-only)** (read path, JSON/CSV, `--at-block`, exit codes, e2e).

### stake move
Move alpha between subnets (same hotkey). Sells alpha on source subnet, buys on destination.

```bash
agcli stake move --amount 5.0 --from 1 --to 2 [--hotkey-address SS58]
```

**On-chain**: `SubtensorModule::move_stake(origin, hotkey, origin_netuid, destination_netuid, alpha_amount)`
- Events: `StakeMoved(coldkey, origin_hotkey, origin_netuid, dest_hotkey, dest_netuid, tao_equivalent)`
- Two AMM operations: `swap_alpha_for_tao()` on source, `swap_tao_for_alpha()` on destination
- All move/swap/transfer operations funnel through `transition_stake_internal()`

### stake swap
Swap stake between hotkeys on the same subnet.

```bash
agcli stake swap --amount 5.0 --netuid 1 --from-hotkey HK1 --to-hotkey HK2
```

**On-chain**: `SubtensorModule::swap_stake(origin, from_hotkey, from_netuid, to_netuid, alpha_amount)`

### stake unstake-all
Unstake all alpha from a hotkey across all subnets.

```bash
agcli stake unstake-all [--hotkey-address SS58]
```

### stake add-limit / remove-limit / swap-limit
Limit orders for staking operations. Execute when AMM price reaches target.

```bash
agcli stake add-limit --amount 10.0 --netuid 1 --price 0.5 [--partial] [--hotkey-address SS58]
agcli stake remove-limit --amount 5.0 --netuid 1 --price 0.8 [--partial] [--hotkey-address SS58]
agcli stake swap-limit --amount 5.0 --from 1 --to 2 --price 0.5 [--partial] [--hotkey-address SS58]
```

### stake childkey-take
Set the childkey take percentage for a hotkey on a subnet.

```bash
agcli stake childkey-take --take 10.0 --netuid 1 [--hotkey-address SS58]
```

**On-chain**: `SubtensorModule::set_childkey_take(origin, hotkey, netuid, take)` where take is u16 (pct * 65535 / 100)
- Errors: `InvalidChildkeyTake`, `TxChildkeyTakeRateLimitExceeded`

### stake set-children
Delegate weight to child hotkeys on a subnet.

```bash
agcli stake set-children --netuid 1 --children "0.5:5Child1...,0.3:5Child2..."
```

**On-chain**: `SubtensorModule::set_children(origin, hotkey, netuid, children)` → `do_schedule_children()`
- Children are NOT applied immediately — they go into `PendingChildKeys` with a cooldown period
- Events: `SetChildrenScheduled(hotkey, netuid, cooldown_block, children)`
- Errors: `InvalidChild`, `DuplicateChild`, `ProportionOverflow`, `TooManyChildren` (max 5), `ChildParentInconsistency` (bipartite separation enforced), `NotEnoughStakeToSetChildkeys`

### stake remove-stake-full-limit
Remove ALL stake for a hotkey/subnet pair, optionally with a price limit.

```bash
agcli stake remove --amount MAX --netuid 1 [--hotkey-address SS58]
```

**On-chain**: `SubtensorModule::remove_stake_full_limit(origin, hotkey, netuid, limit_price)`
- If `limit_price` is set, uses limit order logic; otherwise unstakes everything at market.

### stake recycle-alpha
Recycle alpha tokens back to TAO (burns alpha, reduces `SubnetAlphaOut` — increases alpha price).

```bash
agcli stake recycle-alpha --amount 100.0 --netuid 1 [--hotkey-address SS58]
```

### stake burn-alpha
Permanently burn alpha tokens. Unlike recycle, does NOT reduce `SubnetAlphaOut` (pool ratio unchanged).

```bash
agcli stake burn-alpha --amount 50.0 --netuid 1 [--hotkey-address SS58]
```

### stake unstake-all-alpha
Unstake all alpha across all subnets for a hotkey.

```bash
agcli stake unstake-all-alpha [--hotkey-address SS58]
```

### stake claim-root
Claim root network dividends for a specific subnet.

```bash
agcli stake claim-root --netuid 1
```

**On-chain**: `SubtensorModule::claim_root_dividends(origin, hotkey, netuid)`

### stake process-claim
Batch claim root dividends across multiple subnets.

```bash
agcli stake process-claim [--hotkey-address SS58] [--netuids "1,2,3"]
```

Iterates over all subnets where the hotkey has stake and calls `claim_root_dividends` for each.

### stake set-auto
Set automatic staking destination for a subnet.

```bash
agcli stake set-auto --netuid 1 [--hotkey-address SS58]
```

### stake show-auto
Show auto-stake destinations for a coldkey.

```bash
agcli stake show-auto [--address SS58]
```

### stake set-claim
Set how root emissions are handled (swap to TAO, keep as alpha, or keep for specific subnets).

```bash
agcli stake set-claim --claim-type swap|keep|keep-subnets [--subnets "1,2,3"]
```

### stake transfer-stake
Transfer stake to a different coldkey owner.

```bash
agcli stake transfer-stake --dest 5Dest... --amount 10.0 --from 1 --to 2 [--hotkey-address SS58]
```

**On-chain**: `SubtensorModule::transfer_stake(origin, destination_coldkey, hotkey, origin_netuid, destination_netuid, alpha_amount)`

### stake wizard
Interactive or fully-scripted staking wizard.

```bash
agcli stake wizard [--netuid 1] [--amount 5.0] [--hotkey-address SS58] [--password PW] [--yes]
```

## Global Flags That Affect Staking
- `--mev` — Encrypt staking extrinsic via MEV shield (ML-KEM-768)
- `--dry-run` — Show what would be submitted without broadcasting
- `--output json` — Machine-readable JSON output
- `--batch` / `--yes` — Non-interactive mode

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `NotEnoughBalanceToStake` | Coldkey balance < stake amount | Check `agcli balance` |
| `StakingRateLimitExceeded` | Too many stake ops in short time | Wait and retry |
| `NotEnoughStakeToWithdraw` | Unstake amount > staked amount | Check `agcli stake list` |
| `HotKeyAccountNotExists` | Hotkey not registered on chain | Register hotkey first |
| `TooManyChildren` | >5 children set | Reduce child count |
| `AmountTooLow` | Stake amount below minimum | Increase amount |

## Source Code
**agcli handler**: [`src/cli/stake_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/stake_cmds.rs) — `handle_stake()` (`StakeCommands::List` is the read-only entry above; other variants follow in the same file).

**Subtensor pallet**:
- [`staking/add_stake.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/staking/add_stake.rs) — `add_stake` extrinsic + AMM swap
- [`staking/remove_stake.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/staking/remove_stake.rs) — `remove_stake` + unstake flow
- [`staking/move_stake.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/staking/move_stake.rs) — `move_stake`, `swap_stake`, `transfer_stake`
- [`staking/set_children.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/staking/set_children.rs) — `set_children`, `set_childkey_take`
- [`staking/recycle_alpha.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/staking/recycle_alpha.rs) — `recycle_alpha`, burn operations
- [`staking/claim_root.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/staking/claim_root.rs) — `claim_root_dividends`
- [`staking/stake_utils.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/staking/stake_utils.rs) — AMM: `swap_tao_for_alpha()`, `swap_alpha_for_tao()`
- [`macros/dispatches.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/dispatches.rs) — all dispatch entry points
- [`macros/events.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/events.rs) — event definitions
- [`macros/errors.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/macros/errors.rs) — error definitions

## Related Commands
- `agcli balance` — Check balance before staking
- `agcli view portfolio` — See all stakes and positions
- `agcli subnet show --netuid N` — Check subnet AMM pool depth
- `agcli view swap-sim --netuid N --tao X` — Simulate swap before staking
- `agcli explain --topic stake-weight` — Min stake for weight setting
