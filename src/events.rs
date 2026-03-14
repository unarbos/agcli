//! Event and block subscription — real-time chain event streaming.
//!
//! Uses subxt's block subscription to watch for new blocks and decode
//! relevant SubtensorModule events (stakes, transfers, registrations, etc.).

use anyhow::Result;
use subxt::OnlineClient;

use crate::SubtensorConfig;

/// Categories of events to filter for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventFilter {
    /// All events
    All,
    /// Staking events (add/remove/move/swap stake)
    Staking,
    /// Registration events (neuron/subnet registration)
    Registration,
    /// Transfer events
    Transfer,
    /// Weight events (set/commit/reveal)
    Weights,
    /// Subnet events (hyperparams, identity)
    Subnet,
}

impl EventFilter {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "staking" | "stake" => Self::Staking,
            "registration" | "register" | "reg" => Self::Registration,
            "transfer" | "transfers" => Self::Transfer,
            "weights" | "weight" => Self::Weights,
            "subnet" | "subnets" => Self::Subnet,
            _ => Self::All,
        }
    }

    fn matches_pallet(&self, pallet: &str) -> bool {
        match self {
            Self::All => true,
            Self::Staking => pallet == "SubtensorModule"
                && ["StakeAdded", "StakeRemoved", "StakeMoved"].iter().any(|_| true),
            Self::Registration => pallet == "SubtensorModule",
            Self::Transfer => pallet == "Balances",
            Self::Weights => pallet == "SubtensorModule",
            Self::Subnet => pallet == "SubtensorModule",
        }
    }
}

/// A decoded chain event for display.
#[derive(Debug)]
pub struct ChainEvent {
    pub block_number: u64,
    pub block_hash: String,
    pub pallet: String,
    pub variant: String,
    pub fields: String,
}

impl std::fmt::Display for ChainEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{} {}::{} {}",
            self.block_number, self.pallet, self.variant, self.fields
        )
    }
}

/// Subscribe to new blocks and stream events matching the filter.
pub async fn subscribe_events(
    client: &OnlineClient<SubtensorConfig>,
    filter: EventFilter,
    json_output: bool,
) -> Result<()> {
    let mut block_sub = client.blocks().subscribe_finalized().await?;

    println!("Subscribed to finalized blocks (filter: {:?}). Ctrl+C to stop.\n", filter);

    while let Some(block_result) = block_sub.next().await {
        let block = block_result?;
        let block_number = block.number() as u64;
        let block_hash = format!("{:?}", block.hash());

        let events = block.events().await?;
        for event in events.iter() {
            let event = event?;
            let pallet = event.pallet_name().to_string();
            let variant = event.variant_name().to_string();

            if !filter.matches_pallet(&pallet) {
                continue;
            }

            let fields = format!("{:?}", event.field_values()?);

            let ce = ChainEvent {
                block_number,
                block_hash: block_hash.clone(),
                pallet: pallet.clone(),
                variant: variant.clone(),
                fields: truncate(&fields, 200),
            };

            if json_output {
                println!(
                    "{}",
                    serde_json::json!({
                        "block": block_number,
                        "hash": block_hash,
                        "pallet": pallet,
                        "event": variant,
                        "fields": fields,
                    })
                );
            } else {
                println!("{}", ce);
            }
        }
    }
    Ok(())
}

/// Subscribe to new blocks only (no event decoding).
pub async fn subscribe_blocks(
    client: &OnlineClient<SubtensorConfig>,
    json_output: bool,
) -> Result<()> {
    let mut block_sub = client.blocks().subscribe_finalized().await?;

    println!("Subscribed to finalized blocks. Ctrl+C to stop.\n");

    while let Some(block_result) = block_sub.next().await {
        let block = block_result?;
        let number = block.number() as u64;
        let hash = format!("{:?}", block.hash());
        let extrinsic_count = block.extrinsics().await?.len();

        if json_output {
            println!(
                "{}",
                serde_json::json!({
                    "block": number,
                    "hash": hash,
                    "extrinsics": extrinsic_count,
                })
            );
        } else {
            println!("Block #{} hash={} extrinsics={}", number, hash, extrinsic_count);
        }
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
