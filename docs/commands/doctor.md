# doctor — Install & connectivity smoke test

Run **`agcli doctor`** after installing the binary to confirm your build talks to the right network, the RPC endpoint answers, and your default wallet directory looks sane. No subcommands — one diagnostic panel.

**Discoverability:** `agcli doctor --help`; `agcli explain` Phase 6 cheat sheet lists **`agcli doctor`** with the e2e log name; [`docs/llm.txt`](../llm.txt) Tier 1 + command table link here.

## Usage

```bash
agcli doctor
agcli doctor --network test
agcli doctor --endpoint ws://127.0.0.1:9944 --output json
agcli --wallet mywallet --wallet-dir ~/.bittensor/wallets doctor
```

Uses global flags only (`--network`, `--endpoint`, `--wallet-dir`, `--wallet`, `--output`, etc.).

## What it checks

Order matches [`handle_doctor`](https://github.com/unarbos/agcli/blob/main/src/cli/system_cmds.rs) in `src/cli/system_cmds.rs`:

1. **Version** — build label (`agcli v…` from `CARGO_PKG_VERSION`). Always OK.
2. **Network** — resolved network name and how many WebSocket URLs are configured. Always OK.
3. **Connection** — `Client::connect_network(network)`; OK or FAIL with error text (unreachable host, TLS, wrong URL, etc.).
4. **Block height** — `get_block_number` when connected; OK or FAIL.
5. **Subnets** — `get_total_networks` when connected; OK or FAIL.
6. **Latency (3 pings)** — three sequential `get_block_number` calls; reports avg/min/max ms. FAIL if every ping errors; partial failures are noted in the detail line.
7. **Disk cache** — [`disk_cache::list_keys`](https://github.com/unarbos/agcli/blob/main/src/queries/disk_cache.rs) + entry count / size under [`disk_cache::path`](https://github.com/unarbos/agcli/blob/main/src/queries/disk_cache.rs) (`~/.agcli/cache` by default). Informational; treated as OK even when empty.
8. **Wallet** — opens `{wallet_dir}/{wallet_name}` (defaults: `~/.bittensor/wallets` / `default`, tildes expanded). OK if a coldkey is present; FAIL if coldkey missing or the wallet path cannot be opened as expected.

## Human output

```
agcli doctor
------------------------------------------------------------
  [  OK] Version              agcli v…
  [  OK] Network              …
  [  OK] Connection           OK (Nms)
  ...
------------------------------------------------------------
  All checks passed.
```

Failed rows show `[FAIL]`; the footer reports how many checks failed.

## JSON output

`--output json`:

```json
{
  "doctor": [
    { "check": "Version", "detail": "…", "ok": true },
    …
  ]
}
```

## Exit codes

**The process exits `0` whenever `doctor` finishes**, even if some rows are FAIL — failures are visible in the table or JSON `ok: false`, not via a non-zero exit. That matches [`handle_doctor`](https://github.com/unarbos/agcli/blob/main/src/cli/system_cmds.rs), which always returns `Ok(())`.

For automation that must detect RPC failure, parse JSON and inspect `Connection`, `Block height`, or `Latency (3 pings)` entries.

Other commands still use the normal map in [`src/error.rs`](https://github.com/unarbos/agcli/blob/main/src/error.rs) (**1** generic, **10** network, **12** validation, **15** timeout, etc.). **`doctor` is intentionally non-fatal** so a single run always produces a full report.

Invalid global flags are handled by clap (typically exit **2**).

## E2E

Log line **`doctor_preflight`** in Phase 20 `test_doctor_preflight` (`tests/e2e_test.rs`) mirrors the post-connect RPC bundle: `get_block_number`, `get_total_networks`, three `get_block_number` pings, plus disk cache key count/path (same helpers as the CLI). Wallet state is environment-specific and is documented above rather than asserted in CI.

## Source code

**Handler:** [`src/cli/system_cmds.rs`](https://github.com/unarbos/agcli/blob/main/src/cli/system_cmds.rs) — `handle_doctor()`.

**Dispatch:** [`src/cli/commands.rs`](https://github.com/unarbos/agcli/blob/main/src/cli/commands.rs) — `Commands::Doctor`.

## Related commands

- `agcli utils latency` — dedicated round-trip benchmark
- `agcli balance` — free balance for an address
- `agcli config show` — persisted defaults (`network`, `wallet`, …)
