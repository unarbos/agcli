# Local Testing Guide

Develop and test subnets locally with zero cost and instant feedback. No TAO needed, no testnet delays.

## Prerequisites

- Docker installed and running
- agcli built: `cargo build --release`

## Quick Start

One command gives you a fully-configured test environment with a local chain, funded accounts, and registered neurons:

```bash
agcli localnet scaffold
```

This starts a local subtensor chain (fast-block mode, 250ms blocks) and returns a JSON manifest with:
- WebSocket endpoint (`ws://127.0.0.1:9944`)
- 1 subnet with tuned hyperparameters (tempo=100, weights_rate_limit=0, commit_reveal=false)
- 3 funded neurons: `validator1` (1000 TAO), `miner1` (100 TAO), `miner2` (100 TAO)
- Deterministic keypairs derived from neuron names (e.g., `//validator1_sn1` — reproducible across runs, same keys every time)

## Step-by-Step

### 1. Start the chain and scaffold a subnet

```bash
# Default scaffold: 1 subnet, 3 neurons
agcli localnet scaffold

# Or start just the chain (no subnet setup)
agcli localnet start
```

### 2. Connect and inspect

```bash
# Check chain status
agcli localnet status

# View your subnet
agcli subnet list --network local

# View the metagraph
agcli subnet metagraph 1 --network local

# Check account balances
agcli balance --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY --network local
```

### 3. Test your validator/miner logic

```bash
# Register another neuron
agcli subnet register-neuron 1 --network local

# Set weights as a validator
agcli weights set --netuid 1 "0:100,1:200" --network local

# Serve an axon endpoint
agcli serve axon --netuid 1 --ip 127.0.0.1 --port 8091 --network local

# View hyperparameters
agcli subnet hyperparams 1 --network local
```

### 4. Use the scaffold JSON output

The scaffold command prints a JSON manifest you can use in scripts:

```bash
# Capture output for scripting
OUTPUT=$(agcli localnet scaffold 2>/dev/null)

# Extract neuron SS58 addresses with jq
VALIDATOR=$(echo "$OUTPUT" | jq -r '.subnets[0].neurons[0].ss58')
MINER=$(echo "$OUTPUT" | jq -r '.subnets[0].neurons[1].ss58')
NETUID=$(echo "$OUTPUT" | jq -r '.subnets[0].netuid')

# Now use them in subsequent commands
agcli balance --address "$VALIDATOR" --network local
agcli weights set --netuid "$NETUID" "0:100,1:200" --network local
```

### 5. Use the Rust SDK

```rust
use agcli::chain::Client;
use agcli::types::balance::Balance;
use agcli::types::network::NetUid;
use sp_core::{sr25519, Pair};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to local chain
    let client = Client::connect("ws://127.0.0.1:9944").await?;

    // Use scaffold keypairs (deterministic from name)
    let validator = sr25519::Pair::from_string("//validator1_sn1", None)?;
    let miner = sr25519::Pair::from_string("//miner1_sn1", None)?;

    // Query subnet state
    let neurons = client.get_neurons_lite(NetUid(1)).await?;
    println!("Neurons on SN1: {}", neurons.len());

    // Set weights
    let uids = vec![0u16, 1u16];
    let weights = vec![32768u16, 32768u16];
    client.set_weights(&validator, NetUid(1), &uids, &weights, 0).await?;

    // Query metagraph
    let mg = client.get_metagraph(NetUid(1)).await?;
    println!("Metagraph: n={}, block={}", mg.n, mg.block);

    Ok(())
}
```

### 6. Clean up

```bash
agcli localnet stop
```

## Custom Configurations

Create a `scaffold.toml` to customize your test environment:

```toml
[chain]
image = "ghcr.io/opentensor/subtensor-localnet:devnet-ready"
port = 9944
start = true
timeout = 120

# Subnet with fast tempo for rapid testing
[[subnet]]
tempo = 50
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

[[subnet.neuron]]
name = "miner2"
fund_tao = 100.0
register = true
```

Then run:

```bash
agcli localnet scaffold --config scaffold.toml
```

### Multiple Subnets

Add multiple `[[subnet]]` blocks to test cross-subnet operations:

```toml
[[subnet]]
tempo = 100
[[subnet.neuron]]
name = "val_sn1"
fund_tao = 500.0
register = true

[[subnet]]
tempo = 200
[[subnet.neuron]]
name = "val_sn2"
fund_tao = 500.0
register = true
```

## Dev Accounts

Two accounts are pre-funded in the genesis block:

| Account | SS58 | Seed | TAO | Notes |
|---------|------|------|-----|-------|
| Alice | `5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY` | `//Alice` | 1,000,000 | Sudo key |
| Bob | `5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty` | `//Bob` | 1,000,000 | Standard dev account |

Alice has sudo privileges and can call AdminUtils to configure subnet hyperparameters.

**Scaffold-created neurons** use deterministic keypairs formatted as `//{name}_sn{netuid}` (e.g., `//validator1_sn1`). This means every scaffold run produces the same keys — your test scripts stay stable across sessions. You can derive the keypair in Rust with `sr25519::Pair::from_string("//validator1_sn1", None)?`.

## Chain Details

- **Block time**: 250ms (fast-block mode, ~240x faster than mainnet)
- **Validators**: 3 nodes in the Docker container
- **Docker image**: `ghcr.io/opentensor/subtensor-localnet:devnet-ready`
- **WebSocket**: `ws://127.0.0.1:9944` (default)

### Known Limitations

Some runtime features behave differently on localnet:

| Feature | Status | Workaround |
|---------|--------|------------|
| Staking (subtoken) | May return `SubtokenDisabled` | Use a newer localnet image, or test staking on testnet |
| Commit-reveal weights | Enabled by default | Disable via `agcli admin set-commit-reveal --netuid N --enabled false` or scaffold config |
| State pruning | Fast blocks prune old state quickly | Query recent blocks only; use `--at-block` with recent block numbers |
| Crowdloan pallet | Not available | Test crowdloan features on testnet |

## Testing Workflow

A typical development cycle:

1. **Start**: `agcli localnet scaffold` (once per session)
2. **Develop**: Write your validator/miner code against `ws://127.0.0.1:9944`
3. **Iterate**: Changes take effect in <1 second (250ms blocks)
4. **Reset**: `agcli localnet reset` to wipe state and start fresh
5. **Graduate**: Once working locally, deploy to testnet (`--network test`)

## Troubleshooting

**"Docker not installed or not running"** — Install Docker and start the daemon.

**"Container failed to start"** — Pull the image first: `docker pull ghcr.io/opentensor/subtensor-localnet:devnet-ready`

**"Chain did not become ready"** — The container started but isn't producing blocks. Check logs: `agcli localnet logs`

**Port conflict** — Another service is using port 9944. Use `--port 9955` to pick a different port.

**"Transaction is outdated"** — Fast-block mode can cause mortal-era transactions to expire. Retry the transaction.

## Related

- [Localnet Commands](../commands/localnet.md) — Full CLI reference
- [Subnet Builder Guide](subnet-builder.md) — Production subnet operations
- [Getting Started](getting-started.md) — CLI basics and wallet setup
