# transfer — Send TAO (coldkey)

Move **free TAO** from the default wallet **coldkey** to another SS58 account. Three top-level commands share the same handler module; all require **wallet unlock** (`--password` / `AGCLI_PASSWORD`) unless you use **`--dry-run`** after connect.

**Discoverability:** `agcli transfer --help`, `agcli transfer-all --help`, `agcli transfer-keep-alive --help`; Tier 1 in [`docs/llm.txt`](../llm.txt); `agcli explain` Phase 6 lists the e2e log name; this file is linked from the command table in `llm.txt`.

## Commands

### `agcli transfer`

Send a specific **TAO** amount. **On-chain:** `Balances::transfer_allow_death` — the sender account **may be reaped** if the remaining balance would drop below the existential deposit. Prefer **`transfer-keep-alive`** if you must keep the sender alive.

```bash
agcli transfer --dest 5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty --amount 1.0 --password p --yes
agcli --output json transfer --dest 5FHn... --amount 0.001 --password p --yes
agcli --dry-run transfer --dest 5FHn... --amount 1.0 --password p --yes   # encode only, no submit
```

### `agcli transfer-all`

Send **entire free balance** (minus fees). **On-chain:** `Balances::transfer_all(dest, keep_alive)`.

```bash
agcli transfer-all --dest 5FHn... --password p --yes
agcli transfer-all --dest 5FHn... --keep-alive --password p --yes   # leave existential deposit
```

### `agcli transfer-keep-alive`

Send an amount while **keeping the sender above the existential deposit**. **On-chain:** `Balances::transfer_keep_alive`.

```bash
agcli transfer-keep-alive --dest 5FHn... --amount 1.0 --password p --yes
```

## Read path (validation → RPC → submit)

Order matches [`Commands::Transfer`](https://github.com/unconst/agcli/blob/main/src/cli/commands.rs), **`TransferAll`**, and **`TransferKeepAlive`** in `src/cli/commands.rs` (326–469):

1. **`validate_ss58(&dest, "destination")`** (`src/cli/helpers.rs`).
2. **`validate_amount`** for **`transfer`** and **`transfer-keep-alive`** only (`"transfer amount"` label).
3. **`connect`** (global network / endpoint).
4. **`open_wallet`** + **`unlock_coldkey`**.
5. **Preflight balance** ( **`transfer`** and **`transfer-keep-alive`** only ): if the wallet exposes a coldkey SS58, **`get_balance_ss58`**; if current &lt; amount, bail with an insufficient message **before** signing.
6. Optional **Confirm** prompts unless **`--yes`** / batch mode.
7. **`Client::transfer`**, **`transfer_all`**, or **`transfer_keep_alive`** (`src/chain/extrinsics.rs`) → **`sign_submit`** ( **`--dry-run`** encodes call data and returns without submitting — see `src/chain/mod.rs`).

**`transfer-all`** skips the client-side balance compare (the chain handles dust and fees).

## JSON output

Success (normal submit): `print_tx_result` emits a single object on stderr:

```json
{"tx_hash":"0x..."}
```

**`--dry-run`:** `sign_submit` prints a preview object (signer, `call_data_hex`, …) and the JSON result still carries `"tx_hash":"dry-run"` from the helper.

## Exit codes

| Code | When |
|------|------|
| **0** | Transfer submitted; **`--dry-run`** preview OK; user declined confirm (**“Cancelled.”**). |
| **2** | Clap / invalid global flags. |
| **10** | Network / WebSocket failure (e.g. failed **`connect`**). |
| **11** | Auth: wrong password, missing wallet, unlock failure. |
| **12** | Validation: bad **`--dest`** (SS58), invalid **`--amount`** (negative, zero, non-finite), etc. — see [`classify`](https://github.com/unconst/agcli/blob/main/src/error.rs). |
| **13** | Chain: dispatch / extrinsic errors; **client-side** “Insufficient balance: you have … but trying to transfer …” (message contains **insufficient**). |
| **15** | Timeout when applicable. |
| **1** | Other uncategorized errors (e.g. encode failure in dry-run). |

Hints for validation **12** may mention **`docs/commands/transfer.md`** for **`transfer amount`** / **`destination`** messages.

## Fees & existential deposit

- Fees depend on chain configuration (custom fee handler on Bittensor).
- Existential deposit is chain-defined; **`transfer-all --keep-alive`** and **`transfer-keep-alive`** are the safe choices when the sender must stay alive.

## E2E

Log lines **`transfer_preflight`** in Phase 20 [`test_transfer_preflight`](https://github.com/unconst/agcli/blob/main/tests/e2e_test.rs): local **`validate_ss58`** + **`validate_amount`**, then **`get_balance_ss58`** on Alice to mirror the pre-submit check for **`transfer`** / **`transfer-keep-alive`** (destination Bob on localnet). **`transfer-all`** is documented as **no** pre-balance RPC in the handler.

Full extrinsic coverage: Phase 2 **`test_transfer`** (Alice → Bob) and Phase 16 **`test_transfer_all`** in the same file.

## Source code

- **CLI:** [`src/cli/commands.rs`](https://github.com/unconst/agcli/blob/main/src/cli/commands.rs) — `Commands::Transfer`, `TransferAll`, `TransferKeepAlive`.
- **Extrinsics:** [`src/chain/extrinsics.rs`](https://github.com/unconst/agcli/blob/main/src/chain/extrinsics.rs) — `transfer`, `transfer_all`, `transfer_keep_alive`.

## Related commands

- `agcli balance` — free TAO before sending
- `agcli stake transfer-stake` — move **stake** between coldkeys/subnets (different pallet)
- `agcli view history` — recent account activity
- `agcli doctor` — RPC smoke test after install
