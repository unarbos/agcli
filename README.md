# agcli — Rust CLI + SDK for Bittensor

[![CI](https://github.com/unarbos/agcli/actions/workflows/ci.yml/badge.svg)](https://github.com/unarbos/agcli/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/unarbos/agcli/graph/badge.svg)](https://codecov.io/gh/unarbos/agcli)

Fast, safe Rust toolkit for the [Bittensor](https://bittensor.com) network.

The **CI** badge reflects the [main workflow](https://github.com/unarbos/agcli/actions/workflows/ci.yml) only: formatting, Clippy, `cargo test` on the library plus `wallet_test` / `cli_weights`, coverage upload, and release build. It does **not** include Docker or live-chain e2e tests (those run in a separate [E2E workflow](https://github.com/unarbos/agcli/actions/workflows/e2e.yml)).

## Coverage

Codecov and the **coverage** job use the same scope as CI tests above ([`cargo llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) on the library + `wallet_test` / `cli_weights` only — no e2e). Use the Codecov badge above or open **Coverage** in the [CI workflow](https://github.com/unarbos/agcli/actions/workflows/ci.yml) for the latest tree, PR diffs, and file-level line coverage. Wallets, staking, transfers, subnets, weights, metagraph queries, Dynamic TAO, monitoring, and more.

## Install

```bash
cargo install --git https://github.com/unconst/agcli
```

## Quick Examples

```bash
# Check balance
agcli balance --address 5Gx...

# List subnets as JSON
agcli --output json subnet list

# Stake TAO with slippage protection
agcli stake add --amount 10 --netuid 1 --max-slippage 2.0 --password p --yes

# Atomic commit-reveal weights
agcli weights commit-reveal --netuid 1 --weights "0:100,1:200" --wait

# Live subnet monitoring (JSON streaming)
agcli subnet monitor --netuid 97 --json

# Local development — zero cost, instant feedback
agcli localnet scaffold
```

Every command supports `--output json|csv`, `--yes` (skip prompts), `--batch` (hard-error mode), and `--dry-run` (preview). Full non-interactive operation for AI agents.

## Documentation

| Resource | Description |
|----------|-------------|
| **[docs/why-agcli.md](docs/why-agcli.md)** | Why agcli? Comparison with btcli and the Bittensor Python SDK |
| **[docs/llm.txt](docs/llm.txt)** | Agent/LLM reference — quick-ref card + full command reference |
| **[docs/commands/](docs/commands/)** | Per-command deep dives — on-chain behavior, pallet refs, storage keys, events, errors |
| **[docs/tutorials/](docs/tutorials/)** | Step-by-step guides: [getting started](docs/tutorials/getting-started.md), [staking](docs/tutorials/staking-guide.md), [validator](docs/tutorials/validator-guide.md), [subnet builder](docs/tutorials/subnet-builder.md), [agent automation](docs/tutorials/agent-automation.md) |
| **[docs/faq.md](docs/faq.md)** | Beyond agcli — miners, Yuma math, picking subnets, validator↔miner protocols, subnet codebases |
| **[docs/hyperparameters.md](docs/hyperparameters.md)** | Complete reference for all ~32 sudo-settable subnet hyperparameters — what each does, defaults, interactions |
| **[docs/philosophy.md](docs/philosophy.md)** | Subnet design philosophy, incentive patterns, trust model |

## SDK Usage

```toml
[dependencies]
agcli = { git = "https://github.com/unconst/agcli", default-features = false, features = ["sdk-only"] }
```

```rust
use agcli::{Client, Wallet, Balance};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::connect("wss://entrypoint-finney.opentensor.ai:443").await?;
    let balance = client.get_balance_ss58("5Gx...").await?;
    let subnets = client.get_all_subnets().await?;
    let metagraph = client.get_metagraph(1.into()).await?;
    Ok(())
}
```

## Architecture

```
agcli/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # SDK re-exports (Client, Wallet, Balance, Config)
│   ├── config.rs            # Persistent config (~/.agcli/config.toml)
│   ├── error.rs             # Error classification + exit codes
│   ├── events.rs            # Real-time block/event subscription
│   ├── live.rs              # Live polling with delta tracking
│   ├── chain/
│   │   ├── mod.rs           # Client: connection, retry, 40+ queries + extrinsics
│   │   ├── queries.rs       # Chain query methods
│   │   ├── extrinsics.rs    # Transaction builders
│   │   └── rpc_types.rs     # Type conversions
│   ├── cli/
│   │   ├── mod.rs           # Clap parser: 20 command groups, 90+ subcommands
│   │   ├── commands.rs      # Main dispatcher
│   │   ├── helpers.rs       # Shared CLI helpers
│   │   ├── subnet_cmds.rs   # Subnet operations
│   │   ├── view_cmds.rs     # View/query handlers
│   │   ├── stake_cmds.rs    # Staking operations
│   │   ├── weights_cmds.rs  # Weight setting + commit-reveal
│   │   ├── wallet_cmds.rs   # Wallet management
│   │   ├── block_cmds.rs    # Block explorer
│   │   ├── network_cmds.rs  # Network queries + commitment commands
│   │   ├── localnet_cmds.rs # Local chain lifecycle + scaffold
│   │   ├── admin_cmds.rs    # AdminUtils sudo hyperparam setters
│   │   └── system_cmds.rs   # Config, proxy, delegate, identity
│   ├── localnet.rs           # SDK: Docker chain start/stop/status/reset/logs
│   ├── admin.rs              # SDK: 13 AdminUtils functions + raw_admin_call
│   ├── scaffold.rs           # SDK: Declarative test environment orchestration
│   ├── wallet/              # Key management (Python wallet compat)
│   ├── types/               # Balance, NeuronInfo, SubnetInfo, etc.
│   ├── queries/             # Cache layer (Moka + disk)
│   ├── extrinsics/          # Weight hashing, MEV shield
│   └── utils/               # Explain, format, POW solver
├── docs/
│   ├── llm.txt              # Agent-optimized reference
│   ├── commands/             # 24 per-command docs
│   └── tutorials/            # 5 step-by-step guides
├── examples/
│   └── scaffold.toml         # Example scaffold configuration
├── tests/                    # Integration tests (+ `cli_test_modules/`, `helpers_test_modules/`, `e2e_modules/`)
├── build.rs                  # Compile-time chain metadata fetch
└── Cargo.toml
```

## Building

Requires Rust 1.75+ and network access (fetches chain metadata at build time):

```bash
git clone https://github.com/unconst/agcli && cd agcli && cargo build --release
```

## License

MIT
