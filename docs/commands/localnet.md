# localnet — Local Chain Management

Start, stop, and manage Docker-based subtensor chains for development and testing. Includes one-command test environment scaffolding.

## Chain Lifecycle

### localnet start
Pull and run a subtensor Docker container. Waits for blocks to be produced before returning.

```bash
agcli localnet start [--image TAG] [--port 9944] [--container NAME] [--wait false] [--timeout 120]
# JSON: {"status", "container_name", "container_id", "image", "endpoint", "port", "block_height", "dev_accounts"}
```

Default image: `ghcr.io/opentensor/subtensor-localnet:devnet-ready` (fast-block mode, 250ms blocks, 3 validators).

Returns dev accounts (Alice with 1M TAO + sudo, Bob with 1M TAO) and WebSocket endpoint.

Kills any stale container on the same port before starting. Requires Docker installed and running.

### localnet stop
Stop and remove the container.

```bash
agcli localnet stop [--container NAME]
# JSON: {"status": "stopped", "container_name"}
```

### localnet status
Query running state, block height, and container metadata.

```bash
agcli localnet status [--container NAME] [--port 9944]
# JSON: {"running", "container_name", "container_id", "image", "endpoint", "block_height", "uptime"}
```

### localnet reset
Wipe state and restart the container fresh.

```bash
agcli localnet reset [--image TAG] [--port 9944] [--container NAME] [--timeout 120]
# JSON: {"status": "reset", "container_name", "container_id", "endpoint", "block_height"}
```

### localnet logs
Show container logs.

```bash
agcli localnet logs [--container NAME] [--tail 100]
```

## Scaffold

### localnet scaffold
One command that produces a fully-configured test environment: starts chain, creates wallets, funds accounts from Alice, registers subnets, sets hyperparameters via sudo AdminUtils, registers neurons, and returns a JSON manifest.

```bash
# Use sensible defaults: 1 subnet, 3 neurons (validator1@1000 TAO, miner1@100 TAO, miner2@100 TAO)
agcli localnet scaffold

# Custom config
agcli localnet scaffold --config scaffold.toml --output json

# CLI overrides
agcli localnet scaffold --image ghcr.io/opentensor/subtensor-localnet:v1.2.0 --port 9955

# Connect to existing chain (skip Docker start)
agcli localnet scaffold --no-start --port 9944
```

**Flags:**
| Flag | Description |
|------|-------------|
| `--config PATH` | TOML config file (default: built-in defaults) |
| `--image TAG` | Override Docker image |
| `--port N` | Override host port (default: 9944) |
| `--no-start` | Skip starting chain, connect to existing endpoint |

**Default environment** (no config file needed):
- 1 subnet: tempo=100, max_validators=8, min_weights=1, weights_rate_limit=0, commit_reveal=false
- 3 neurons: `validator1` (1000 TAO), `miner1` (100 TAO), `miner2` (100 TAO)
- All neurons registered on the subnet with deterministic keypairs

**JSON output:**
```json
{
  "endpoint": "ws://127.0.0.1:9944",
  "container": "agcli_localnet",
  "block_height": 42,
  "subnets": [{
    "netuid": 1,
    "hyperparams": {"tempo": 100, "max_allowed_validators": 8, "min_allowed_weights": 1, "weights_rate_limit": 0, "commit_reveal": false},
    "neurons": [
      {"name": "validator1", "ss58": "5G...", "seed": "//validator1_sn1", "uid": 0, "balance_tao": 1000.0},
      {"name": "miner1", "ss58": "5F...", "seed": "//miner1_sn1", "uid": 1, "balance_tao": 100.0},
      {"name": "miner2", "ss58": "5H...", "seed": "//miner2_sn1", "uid": 2, "balance_tao": 100.0}
    ]
  }]
}
```

**Scaffold config (TOML):**
```toml
[chain]
image = "ghcr.io/opentensor/subtensor-localnet:devnet-ready"
port = 9944
start = true
timeout = 120

[[subnet]]
tempo = 100
max_allowed_validators = 8
min_allowed_weights = 1
weights_rate_limit = 0
commit_reveal = false

[[subnet.neuron]]
name = "validator1"
fund_tao = 1000.0
register = true

[[subnet.neuron]]
name = "miner1"
fund_tao = 100.0
register = true
```

See `examples/scaffold.toml` for a complete example.

**Orchestration flow:**
1. Start chain (or connect to existing if `--no-start`)
2. Register subnet from Alice (sudo)
3. Set hyperparameters via AdminUtils sudo calls
4. For each neuron: generate deterministic keypair from name → fund from Alice → burn register → look up UID
5. Return JSON manifest with all addresses, seeds, UIDs, balances

Neuron keypairs are deterministic: derived from `//{name}_sn{netuid}` URI, so scaffold results are reproducible for the same config.

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `Failed to run Docker` | Docker not installed or not running | Install Docker and start the daemon |
| `Container failed to start` | Image not found or port conflict | `docker pull <image>` or change `--port` |
| `Chain did not become ready` | Container started but not producing blocks | Check `agcli localnet logs`, increase `--timeout` |
| `Container not found` | Stop/status on non-existent container | Check `docker ps` or start first |

## Source Code
**agcli handler**: [`src/cli/localnet_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/localnet_cmds.rs) — Start L11, Stop L71, Status L86, Reset L114, Logs L152, Scaffold L159

**SDK**: [`src/localnet.rs`](https://github.com/unconst/agcli/blob/main/src/localnet.rs) — `start()` L114, `stop()` L180, `status()` L201, `reset()` L252, `logs()` L258

**Scaffold SDK**: [`src/scaffold.rs`](https://github.com/unconst/agcli/blob/main/src/scaffold.rs) — `ScaffoldConfig` L36, `run()` L209, `run_with_progress()` L214

## Related Commands
- `agcli admin set-tempo` — Set subnet hyperparameters via sudo
- `agcli admin list` — Show all available AdminUtils parameters
- `agcli subnet register` — Register a new subnet
- `agcli subnet register-neuron` — Register a neuron on a subnet
- `agcli transfer` — Fund accounts with TAO
