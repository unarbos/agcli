//! CLI command definitions and handlers.

pub mod commands;

use clap::{Parser, Subcommand};
use crate::types::network::Network;

/// agcli — Rust CLI for the Bittensor network
#[derive(Parser, Debug)]
#[command(name = "agcli", version, about, long_about = None)]
pub struct Cli {
    /// Network to connect to
    #[arg(long, short, default_value = "finney", env = "AGCLI_NETWORK")]
    pub network: String,

    /// Custom chain endpoint (overrides --network)
    #[arg(long, env = "AGCLI_ENDPOINT")]
    pub endpoint: Option<String>,

    /// Wallet directory
    #[arg(long, default_value = "~/.bittensor/wallets", env = "AGCLI_WALLET_DIR")]
    pub wallet_dir: String,

    /// Wallet name
    #[arg(long, short, default_value = "default", env = "AGCLI_WALLET")]
    pub wallet: String,

    /// Hotkey name
    #[arg(long, default_value = "default", env = "AGCLI_HOTKEY")]
    pub hotkey: String,

    /// Output format
    #[arg(long, default_value = "table", value_parser = ["table", "json", "csv"])]
    pub output: String,

    /// Enable live polling mode (interval in seconds, default 12)
    #[arg(long)]
    pub live: Option<Option<u64>>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ──── Wallet ────
    /// Wallet management
    #[command(subcommand)]
    Wallet(WalletCommands),

    // ──── Balance ────
    /// Show account balance
    Balance {
        /// SS58 address (defaults to wallet coldkey)
        #[arg(long)]
        address: Option<String>,
    },

    // ──── Transfer ────
    /// Transfer TAO to another account
    Transfer {
        /// Destination SS58 address
        dest: String,
        /// Amount of TAO to send
        amount: f64,
    },

    // ──── Staking ────
    /// Staking operations
    #[command(subcommand)]
    Stake(StakeCommands),

    // ──── Subnets ────
    /// Subnet operations
    #[command(subcommand)]
    Subnet(SubnetCommands),

    // ──── Weights ────
    /// Weight setting operations
    #[command(subcommand)]
    Weights(WeightCommands),

    // ──── Root ────
    /// Root network operations
    #[command(subcommand)]
    Root(RootCommands),

    // ──── Delegate ────
    /// Delegate operations (take management)
    #[command(subcommand)]
    Delegate(DelegateCommands),

    // ──── Info ────
    /// View detailed information
    #[command(subcommand)]
    View(ViewCommands),

    // ──── Identity ────
    /// On-chain identity operations
    #[command(subcommand)]
    Identity(IdentityCommands),

    // ──── Swap ────
    /// Key swap operations
    #[command(subcommand)]
    Swap(SwapCommands),

    // ──── Subscribe ────
    /// Subscribe to real-time chain events
    #[command(subcommand)]
    Subscribe(SubscribeCommands),
}

#[derive(Subcommand, Debug)]
pub enum SubscribeCommands {
    /// Watch finalized blocks
    Blocks,
    /// Watch chain events (all, staking, registration, transfer, weights, subnet)
    Events {
        /// Event filter category
        #[arg(default_value = "all")]
        filter: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum WalletCommands {
    /// Create a new wallet
    Create {
        /// Wallet name
        #[arg(long, default_value = "default")]
        name: String,
    },
    /// List all wallets
    List,
    /// Show wallet details (keys, addresses)
    Show {
        /// Show all hotkeys
        #[arg(long)]
        all: bool,
    },
    /// Import wallet from mnemonic
    Import {
        /// Wallet name
        #[arg(long, default_value = "default")]
        name: String,
    },
    /// Regenerate coldkey from mnemonic
    RegenColdkey,
    /// Regenerate hotkey from mnemonic
    RegenHotkey {
        /// Hotkey name
        #[arg(long, default_value = "default")]
        name: String,
    },
    /// Create a new hotkey
    NewHotkey {
        /// Hotkey name
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum StakeCommands {
    /// Add stake to a hotkey on a subnet
    Add {
        /// Amount of TAO to stake
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58 (defaults to wallet hotkey)
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Remove stake from a hotkey on a subnet
    Remove {
        /// Amount to unstake
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Show all stakes for current wallet
    List {
        /// Coldkey SS58 address
        #[arg(long)]
        address: Option<String>,
    },
    /// Move stake between subnets
    Move {
        /// Amount of alpha to move
        amount: f64,
        /// Source subnet
        #[arg(long)]
        from: u16,
        /// Destination subnet
        #[arg(long)]
        to: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Swap stake between hotkeys on same subnet
    Swap {
        /// Amount
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Source hotkey
        #[arg(long)]
        from_hotkey: String,
        /// Destination hotkey
        #[arg(long)]
        to_hotkey: String,
    },
    /// Unstake all from a hotkey
    UnstakeAll {
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Claim root dividends
    ClaimRoot {
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
    },
    /// Add stake with a limit price
    AddLimit {
        /// Amount of TAO
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Limit price
        #[arg(long)]
        price: f64,
        /// Allow partial fill
        #[arg(long)]
        partial: bool,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Remove stake with limit price
    RemoveLimit {
        /// Amount of alpha
        amount: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Limit price
        #[arg(long)]
        price: f64,
        /// Allow partial fill
        #[arg(long)]
        partial: bool,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Set childkey take
    ChildkeyTake {
        /// Take percentage (0-18)
        take: f64,
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Set children for hotkey
    SetChildren {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Children as "proportion:hotkey_ss58" pairs, comma-separated
        #[arg(long)]
        children: String,
    },
    /// Full staking wizard (interactive)
    Wizard,
}

#[derive(Subcommand, Debug)]
pub enum SubnetCommands {
    /// List all subnets
    List,
    /// Show detailed info for a subnet
    Show {
        /// Subnet UID
        netuid: u16,
    },
    /// Show subnet hyperparameters
    Hyperparams {
        /// Subnet UID
        netuid: u16,
    },
    /// Show metagraph for a subnet
    Metagraph {
        /// Subnet UID
        netuid: u16,
    },
    /// Register a new subnet
    Register,
    /// Register a neuron on a subnet (burn)
    RegisterNeuron {
        /// Subnet UID
        netuid: u16,
    },
    /// Register via POW
    Pow {
        /// Subnet UID
        netuid: u16,
        /// Number of threads
        #[arg(long, default_value = "4")]
        threads: u32,
    },
}

#[derive(Subcommand, Debug)]
pub enum WeightCommands {
    /// Set weights on a subnet
    Set {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Weights as "uid:weight" pairs, comma-separated
        weights: String,
        /// Version key
        #[arg(long, default_value = "0")]
        version_key: u64,
    },
    /// Commit weights (for commit-reveal enabled subnets)
    Commit {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Weights as "uid:weight" pairs
        weights: String,
        /// Salt (random if not specified)
        #[arg(long)]
        salt: Option<String>,
    },
    /// Reveal previously committed weights
    Reveal {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Weights as "uid:weight" pairs
        weights: String,
        /// Salt used in commit
        salt: String,
        /// Version key
        #[arg(long, default_value = "0")]
        version_key: u64,
    },
}

#[derive(Subcommand, Debug)]
pub enum RootCommands {
    /// Register on root network
    Register,
    /// Set root weights
    Weights {
        /// Weights as "netuid:weight" pairs
        weights: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DelegateCommands {
    /// Show delegate info
    Show {
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// List all delegates
    List,
    /// Decrease take
    DecreaseTake {
        /// New take percentage
        take: f64,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
    /// Increase take
    IncreaseTake {
        /// New take percentage
        take: f64,
        /// Hotkey SS58
        #[arg(long)]
        hotkey: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ViewCommands {
    /// Show full portfolio (all stakes, balances)
    Portfolio {
        /// Coldkey SS58
        #[arg(long)]
        address: Option<String>,
    },
    /// Show neuron details
    Neuron {
        /// Subnet UID
        #[arg(long)]
        netuid: u16,
        /// Neuron UID
        uid: u16,
    },
    /// Show network overview
    Network,
    /// Show Dynamic TAO info for all subnets (prices, pools, volumes)
    Dynamic,
}

#[derive(Subcommand, Debug)]
pub enum IdentityCommands {
    /// Set on-chain identity
    Set {
        /// Name
        #[arg(long)]
        name: String,
        /// URL
        #[arg(long)]
        url: Option<String>,
        /// GitHub
        #[arg(long)]
        github: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
    },
    /// Show identity
    Show {
        /// SS58 address
        address: String,
    },
    /// Set subnet identity (subnet owner only)
    SetSubnet {
        /// Subnet UID
        netuid: u16,
        /// Subnet name
        #[arg(long)]
        name: String,
        /// GitHub
        #[arg(long)]
        github: Option<String>,
        /// URL
        #[arg(long)]
        url: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SwapCommands {
    /// Swap hotkey
    Hotkey {
        /// New hotkey SS58
        new_hotkey: String,
    },
    /// Schedule coldkey swap
    Coldkey {
        /// New coldkey SS58
        new_coldkey: String,
    },
}

impl Cli {
    /// Get live polling interval (None = not live, Some(secs) = live mode).
    pub fn live_interval(&self) -> Option<u64> {
        self.live.map(|opt| opt.unwrap_or(12))
    }

    /// Resolve the network from CLI args.
    pub fn resolve_network(&self) -> Network {
        if let Some(ref endpoint) = self.endpoint {
            Network::Custom(endpoint.clone())
        } else {
            match self.network.as_str() {
                "finney" | "main" => Network::Finney,
                "test" | "testnet" => Network::Test,
                "local" | "localhost" => Network::Local,
                other => Network::Custom(other.to_string()),
            }
        }
    }
}
