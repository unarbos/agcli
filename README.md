# agcli — Rust CLI + SDK for Bittensor

A fast, safe Rust toolkit for interacting with the [Bittensor](https://bittensor.com) network.
Covers everything: wallet management, staking, transfers, subnet operations, weight setting, registration, metagraph queries, and more.

## Features

| Category | Capabilities |
|---|---|
| **Wallet** | Create, import (mnemonic/seed), encrypt/decrypt coldkeys, manage multiple hotkeys |
| **Staking** | Add/remove stake, move between subnets, swap between hotkeys, limit orders, auto-stake, claim root dividends |
| **Transfers** | Send TAO between accounts |
| **Subnets** | List subnets, view metagraph, register neurons (burn/POW), create subnets, hyperparameters |
| **Weights** | Set weights, commit-reveal (v2 and v3/TLE), batch operations |
| **Delegates** | View delegates, manage take rates, childkey delegation |
| **Identity** | Set/view on-chain identity for accounts and subnets |
| **Queries** | Portfolio view, neuron info, dynamic pricing, subnet state, block info |
| **Key Swaps** | Hotkey swap, coldkey swap (scheduled/announced), dispute |
| **Root** | Root registration, root weights, root claims |

## Quick Start

### Install

```bash
cargo install --git https://github.com/unconst/agcli
```

### CLI Usage

```bash
# Check balance
agcli balance --address 5Gx...

# Create a wallet
agcli wallet create --name my_wallet

# List all subnets
agcli subnet list

# View metagraph
agcli subnet metagraph 1

# Add stake
agcli stake add 10.0 --netuid 1 --hotkey 5Hx...

# Transfer TAO
agcli transfer 5Dest... 1.5

# Set weights
agcli weights set --netuid 1 "0:100,1:200,2:300"

# View portfolio
agcli view portfolio
```

### SDK Usage (as library)

Add to your `Cargo.toml`:
```toml
[dependencies]
agcli = { git = "https://github.com/unconst/agcli", default-features = false, features = ["sdk-only"] }
```

```rust
use agcli::{Client, Wallet, Balance};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect to finney
    let client = Client::connect("wss://entrypoint-finney.opentensor.ai:443").await?;

    // Open wallet
    let mut wallet = Wallet::open("~/.bittensor/wallets/default")?;

    // Check balance
    let balance = client.get_balance(&wallet.coldkey_public()).await?;
    println!("Balance: {}", balance.display_tao());

    // Get block number
    let block = client.get_block_number().await?;
    println!("Block: {}", block);

    Ok(())
}
```

## Architecture

```
agcli/
├── src/
│   ├── lib.rs           # Library root, re-exports
│   ├── main.rs          # CLI entry point
│   ├── chain/           # Substrate RPC client
│   │   ├── connection.rs  # JSON-RPC transport
│   │   ├── storage.rs     # Storage queries
│   │   └── mod.rs         # High-level Client
│   ├── wallet/          # Wallet management
│   │   ├── keypair.rs     # SR25519 key generation
│   │   ├── keyfile.rs     # Encrypted file I/O
│   │   └── mod.rs         # Wallet abstraction
│   ├── types/           # Core data types
│   │   ├── balance.rs     # TAO/Alpha balances
│   │   ├── network.rs     # Network presets
│   │   └── chain_data.rs  # Decoded chain structures
│   ├── extrinsics/      # Transaction construction
│   │   ├── staking.rs     # Stake operations
│   │   ├── transfer.rs    # Transfers
│   │   ├── registration.rs # Registration
│   │   ├── weights.rs     # Weight setting
│   │   ├── subnet.rs      # Subnet management
│   │   ├── identity.rs    # Identity
│   │   └── swap.rs        # Key swaps
│   ├── queries/         # Composed query helpers
│   │   ├── portfolio.rs   # Full portfolio view
│   │   ├── metagraph.rs   # Metagraph fetch
│   │   └── subnet.rs      # Subnet queries
│   ├── utils/           # Shared utilities
│   │   ├── format.rs      # Display formatting
│   │   └── pow.rs         # POW solver
│   └── cli/             # CLI definitions
│       ├── mod.rs         # Clap definitions
│       └── commands.rs    # Command handlers
├── docs/
│   ├── llm.txt          # Agent-friendly docs
│   └── tutorials/
│       ├── getting-started.md
│       ├── staking-guide.md
│       └── subnet-builder.md
├── Cargo.toml
└── README.md
```

## Bittensor Concepts

- **TAO**: Native token. 1 TAO = 1,000,000,000 RAO.
- **Subnets**: Independent networks (netuid 0-N) each running their own incentive mechanism.
- **Coldkey**: Offline signing key for high-value ops (staking, transfers). Encrypted on disk.
- **Hotkey**: Online key for automated ops (weights, serving). Stored plaintext.
- **Neurons**: Registered entities on a subnet (miners and validators).
- **Weights**: Validators set weights to score miners, determining emission distribution.
- **Dynamic TAO**: Each subnet has its own alpha token. Staking buys alpha; emission is in alpha.
- **Root Network** (SN0): Special subnet governing emission distribution across all subnets.
- **Commit-Reveal**: Weight privacy scheme. Commit a hash, reveal later.
- **Take**: Percentage of delegated emissions that a validator keeps.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `AGCLI_NETWORK` | `finney` | Network preset |
| `AGCLI_ENDPOINT` | — | Custom WS endpoint |
| `AGCLI_WALLET_DIR` | `~/.bittensor/wallets` | Wallet directory |
| `AGCLI_WALLET` | `default` | Active wallet name |
| `AGCLI_HOTKEY` | `default` | Active hotkey name |

## License

MIT
