//! Event and block subscription — real-time chain event streaming.
//!
//! Uses subxt's block subscription to watch for new blocks and decode
//! relevant SubtensorModule events (stakes, transfers, registrations, etc.).

use anyhow::Result;
use subxt::ext::scale_value::{Composite, Primitive, ValueDef};
use subxt::OnlineClient;

use crate::utils::truncate;
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

impl std::str::FromStr for EventFilter {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "staking" | "stake" => Self::Staking,
            "registration" | "register" | "reg" => Self::Registration,
            "transfer" | "transfers" => Self::Transfer,
            "weights" | "weight" => Self::Weights,
            "subnet" | "subnets" => Self::Subnet,
            _ => Self::All,
        })
    }
}

/// Known staking-related event variant names.
const STAKING_VARIANTS: &[&str] = &[
    "StakeAdded",
    "StakeRemoved",
    "StakeMoved",
    "StakeSwapped",
    "AllStakeRemoved",
];

/// Known registration-related event variant names.
const REGISTRATION_VARIANTS: &[&str] = &[
    "NeuronRegistered",
    "BurnedRegister",
    "SubnetRegistered",
    "PowRegistered",
];

/// Known weight-related event variant names.
const WEIGHT_VARIANTS: &[&str] = &[
    "WeightsSet",
    "WeightsCommitted",
    "WeightsRevealed",
    "WeightsBatchRevealed",
];

/// Known subnet management event variant names.
const SUBNET_VARIANTS: &[&str] = &[
    "SubnetHyperparamsSet",
    "SubnetIdentitySet",
    "SubnetIdentityRemoved",
    "NetworkAdded",
    "NetworkRemoved",
    "TempoSet",
];

impl EventFilter {
    fn matches(&self, pallet: &str, variant: &str) -> bool {
        match self {
            Self::All => true,
            Self::Staking => pallet == "SubtensorModule" && STAKING_VARIANTS.contains(&variant),
            Self::Registration => {
                pallet == "SubtensorModule" && REGISTRATION_VARIANTS.contains(&variant)
            }
            Self::Transfer => pallet == "Balances",
            Self::Weights => pallet == "SubtensorModule" && WEIGHT_VARIANTS.contains(&variant),
            Self::Subnet => pallet == "SubtensorModule" && SUBNET_VARIANTS.contains(&variant),
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

/// Extract a u16 netuid value from a Composite field (named or unnamed).
///
/// Handles both `Named([("netuid", U128(n)), ...])` and `Unnamed([U128(n), ...])`
/// by checking named fields first, then looking for u16-range values in unnamed composites.
fn extract_netuid<T: Clone>(composite: &Composite<T>) -> Option<u16> {
    match composite {
        Composite::Named(fields) => {
            for (name, val) in fields {
                if name == "netuid" {
                    if let ValueDef::Primitive(Primitive::U128(n)) = &val.value {
                        if *n <= u16::MAX as u128 {
                            return Some(*n as u16);
                        }
                        return None; // out of u16 range — not a valid netuid
                    }
                }
            }
            None
        }
        Composite::Unnamed(_) => None,
    }
}

/// Extract SS58 account addresses from a Composite field.
///
/// Walks named fields looking for 32-byte AccountId composites (common Substrate pattern).
/// Returns all found SS58 addresses.
fn extract_accounts<T: Clone>(composite: &Composite<T>) -> Vec<String> {
    let mut accounts = Vec::new();
    match composite {
        Composite::Named(fields) => {
            for (_name, val) in fields {
                extract_accounts_from_value(&val.value, &mut accounts);
            }
        }
        Composite::Unnamed(fields) => {
            for val in fields {
                extract_accounts_from_value(&val.value, &mut accounts);
            }
        }
    }
    accounts
}

/// Recursively extract SS58 addresses from a ValueDef.
fn extract_accounts_from_value<T: Clone>(val: &ValueDef<T>, out: &mut Vec<String>) {
    match val {
        ValueDef::Composite(inner) => {
            // Check if this composite is a 32-byte AccountId
            if let Some(ss58) = try_composite_as_ss58(inner) {
                out.push(ss58);
            } else {
                // Recurse into sub-fields
                match inner {
                    Composite::Named(fields) => {
                        for (_, v) in fields {
                            extract_accounts_from_value(&v.value, out);
                        }
                    }
                    Composite::Unnamed(fields) => {
                        for v in fields {
                            extract_accounts_from_value(&v.value, out);
                        }
                    }
                }
            }
        }
        ValueDef::Variant(variant) => {
            // Recurse into variant fields (e.g., Some(account))
            match &variant.values {
                Composite::Named(fields) => {
                    for (_, v) in fields {
                        extract_accounts_from_value(&v.value, out);
                    }
                }
                Composite::Unnamed(fields) => {
                    for v in fields {
                        extract_accounts_from_value(&v.value, out);
                    }
                }
            }
        }
        _ => {}
    }
}

/// Try to interpret a Composite as a 32-byte AccountId and return its SS58 address.
fn try_composite_as_ss58<T: Clone>(composite: &Composite<T>) -> Option<String> {
    let fields = match composite {
        Composite::Unnamed(fields) if fields.len() == 32 => fields,
        _ => return None,
    };
    let mut bytes = [0u8; 32];
    for (i, field) in fields.iter().enumerate() {
        match &field.value {
            ValueDef::Primitive(Primitive::U128(n)) if *n <= 255 => {
                bytes[i] = *n as u8;
            }
            _ => return None,
        }
    }
    let public = sp_core::sr25519::Public::from_raw(bytes);
    Some(crate::wallet::keypair::to_ss58(&public, 42))
}

/// Convert a Composite to structured JSON for output.
fn composite_to_json<T: Clone>(composite: &Composite<T>) -> serde_json::Value {
    match composite {
        Composite::Named(fields) => {
            let map: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(map)
        }
        Composite::Unnamed(fields) => {
            // Check if this looks like raw bytes (AccountId32, etc.)
            if fields.len() == 32 || fields.len() == 64 {
                let bytes: Vec<u8> = fields
                    .iter()
                    .filter_map(|v| match &v.value {
                        ValueDef::Primitive(Primitive::U128(n)) if *n <= 255 => Some(*n as u8),
                        _ => None,
                    })
                    .collect();
                if bytes.len() == fields.len() {
                    return serde_json::Value::String(format!("0x{}", hex::encode(&bytes)));
                }
            }
            let arr: Vec<serde_json::Value> = fields.iter().map(value_to_json).collect();
            serde_json::Value::Array(arr)
        }
    }
}

/// Convert a SCALE Value to a serde_json::Value for structured JSON output.
fn value_to_json<T: Clone>(val: &subxt::ext::scale_value::Value<T>) -> serde_json::Value {
    match &val.value {
        ValueDef::Primitive(p) => match p {
            Primitive::Bool(b) => serde_json::Value::Bool(*b),
            Primitive::Char(c) => serde_json::Value::String(c.to_string()),
            Primitive::U128(n) => {
                if *n <= u64::MAX as u128 {
                    serde_json::json!(*n as u64)
                } else {
                    serde_json::Value::String(n.to_string())
                }
            }
            Primitive::I128(n) => {
                if *n >= i64::MIN as i128 && *n <= i64::MAX as i128 {
                    serde_json::json!(*n as i64)
                } else {
                    serde_json::Value::String(n.to_string())
                }
            }
            Primitive::U256(n) => serde_json::Value::String(format!("{:?}", n)),
            Primitive::I256(n) => serde_json::Value::String(format!("{:?}", n)),
            Primitive::String(s) => serde_json::Value::String(s.clone()),
        },
        ValueDef::Composite(composite) => composite_to_json(composite),
        ValueDef::Variant(variant) => {
            let inner = composite_to_json(&variant.values);
            serde_json::json!({ &variant.name: inner })
        }
        ValueDef::BitSequence(bits) => serde_json::Value::String(format!("bits({})", bits.len())),
    }
}

/// Compute the gap (number of missed blocks) between the last processed block and the current one.
/// Returns 0 if there is no gap or if `last_processed` is None (first block).
pub(crate) fn compute_block_gap(last_processed: Option<u64>, current: u64) -> u64 {
    match last_processed {
        Some(last) => current.saturating_sub(last + 1),
        None => 0,
    }
}

/// Subscribe to new blocks and stream events matching the filter.
pub async fn subscribe_events(
    client: &OnlineClient<SubtensorConfig>,
    filter: EventFilter,
    json_output: bool,
) -> Result<()> {
    subscribe_events_filtered(client, filter, json_output, None, None).await
}

/// Maximum consecutive reconnection attempts before giving up.
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Subscribe to events with optional netuid and account filters.
/// Auto-reconnects on WebSocket drops with exponential backoff.
pub async fn subscribe_events_filtered(
    client: &OnlineClient<SubtensorConfig>,
    filter: EventFilter,
    json_output: bool,
    netuid_filter: Option<u16>,
    account_filter: Option<&str>,
) -> Result<()> {
    if !json_output {
        let mut desc = format!("filter: {:?}", filter);
        if let Some(n) = netuid_filter {
            use std::fmt::Write;
            let _ = write!(desc, ", netuid={}", n);
        }
        if let Some(a) = account_filter {
            use std::fmt::Write;
            let _ = write!(desc, ", account={}", crate::utils::short_ss58(a));
        }
        println!(
            "Subscribed to finalized blocks ({}). Ctrl+C to stop.\n",
            desc
        );
    }

    // Run the subscription loop with Ctrl+C handling (Issue 721)
    tokio::select! {
        result = subscribe_events_inner(client, filter, json_output, netuid_filter, account_filter) => result,
        _ = tokio::signal::ctrl_c() => {
            if !json_output {
                eprintln!("\nInterrupted. Closing event subscription.");
            }
            Ok(())
        }
    }
}

/// Inner event subscription loop, factored out for Ctrl+C wrapping.
async fn subscribe_events_inner(
    client: &OnlineClient<SubtensorConfig>,
    filter: EventFilter,
    json_output: bool,
    netuid_filter: Option<u16>,
    account_filter: Option<&str>,
) -> Result<()> {
    let mut reconnect_attempts = 0u32;
    let mut last_processed_block: Option<u64> = None;
    loop {
        let sub_result = client.blocks().subscribe_finalized().await;
        let mut block_sub = match sub_result {
            Ok(s) => {
                if reconnect_attempts > 0 {
                    tracing::info!(
                        "Event subscription reconnected after {} attempts",
                        reconnect_attempts
                    );
                    if !json_output {
                        eprintln!("Reconnected to block stream.");
                    }
                }
                reconnect_attempts = 0;
                s
            }
            Err(e) => {
                reconnect_attempts += 1;
                if reconnect_attempts > MAX_RECONNECT_ATTEMPTS {
                    return Err(anyhow::anyhow!(
                        "Event subscription failed after {} reconnection attempts: {}",
                        MAX_RECONNECT_ATTEMPTS,
                        e
                    ));
                }
                let delay = std::time::Duration::from_secs(1 << reconnect_attempts.min(5));
                tracing::warn!(
                    attempt = reconnect_attempts,
                    delay_secs = delay.as_secs(),
                    error = %e,
                    "Event subscription failed, reconnecting"
                );
                if !json_output {
                    eprintln!(
                        "Warning: subscription failed ({}), retrying in {}s...",
                        e,
                        delay.as_secs()
                    );
                }
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        while let Some(block_result) = block_sub.next().await {
            let block = match block_result {
                Ok(b) => b,
                Err(e) => {
                    let msg = format!("{}", e);
                    // Transient stream errors: log and break to reconnect
                    tracing::warn!(error = %msg, "Block stream error, will reconnect");
                    if !json_output {
                        eprintln!(
                            "Warning: block stream interrupted ({}), reconnecting...",
                            msg
                        );
                    }
                    break; // break inner loop to reconnect
                }
            };
            let block_number = block.number() as u64;

            // Detect and warn about gaps in block sequence (Issue 644)
            if let Some(last) = last_processed_block {
                let gap = block_number.saturating_sub(last + 1);
                if gap > 0 {
                    tracing::warn!(
                        last_block = last,
                        current_block = block_number,
                        missed_blocks = gap,
                        "Gap detected: {} blocks missed between #{} and #{}",
                        gap, last, block_number
                    );
                    if !json_output {
                        eprintln!(
                            "Warning: {} block(s) missed (#{} to #{}) — events in those blocks were not captured",
                            gap, last + 1, block_number - 1
                        );
                    } else {
                        println!(
                            "{}",
                            serde_json::json!({
                                "warning": "gap_detected",
                                "missed_from": last + 1,
                                "missed_to": block_number - 1,
                                "missed_count": gap,
                            })
                        );
                    }
                }
            }
            last_processed_block = Some(block_number);
            let block_hash = format!("{:?}", block.hash());

            let events = match block.events().await {
                Ok(ev) => ev,
                Err(e) => {
                    tracing::warn!(block = block_number, error = %e, "Failed to decode events, skipping block");
                    if !json_output {
                        eprintln!(
                            "Warning: failed to decode events in block #{}: {}",
                            block_number, e
                        );
                    }
                    continue;
                }
            };
            for event in events.iter() {
                let event = match event {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::debug!(error = %e, "Skipping undecodable event");
                        continue;
                    }
                };
                let pallet = event.pallet_name().to_string();
                let variant = event.variant_name().to_string();

                if !filter.matches(&pallet, &variant) {
                    continue;
                }

                let field_values = match event.field_values() {
                    Ok(fv) => fv,
                    Err(e) => {
                        tracing::debug!(pallet = %pallet, variant = %variant, error = %e, "Failed to decode event fields");
                        continue;
                    }
                };

                // Structured netuid filtering — extract netuid from composite fields only
                if let Some(target_netuid) = netuid_filter {
                    match extract_netuid(&field_values) {
                        Some(found) if found == target_netuid => { /* match */ }
                        Some(_) => continue,  // different netuid
                        None => continue,     // no netuid field — skip (not a netuid-bearing event)
                    }
                }

                // Structured account filtering — extract SS58 addresses from composite fields
                if let Some(target_account) = account_filter {
                    let accounts = extract_accounts(&field_values);
                    if !accounts.iter().any(|a| a == target_account) {
                        continue;
                    }
                }

                if json_output {
                    let structured_fields = composite_to_json(&field_values);
                    println!(
                        "{}",
                        serde_json::json!({
                            "block": block_number,
                            "hash": block_hash,
                            "pallet": pallet,
                            "event": variant,
                            "fields": structured_fields,
                        })
                    );
                } else {
                    let fields_str = format!("{:?}", field_values);
                    let ce = ChainEvent {
                        block_number,
                        block_hash: block_hash.clone(),
                        pallet: pallet.clone(),
                        variant: variant.clone(),
                        fields: truncate(&fields_str, 200),
                    };
                    println!("{}", ce);
                }
            }
        }
        // Inner loop exited — block_sub stream ended or errored. Reconnect.
        reconnect_attempts += 1;
        if reconnect_attempts > MAX_RECONNECT_ATTEMPTS {
            return Err(anyhow::anyhow!(
                "Event subscription ended after {} consecutive reconnection failures",
                MAX_RECONNECT_ATTEMPTS
            ));
        }
        let delay = std::time::Duration::from_secs(1 << reconnect_attempts.min(5));
        tracing::warn!(
            attempt = reconnect_attempts,
            delay_secs = delay.as_secs(),
            "Block stream ended, reconnecting"
        );
        if !json_output {
            eprintln!(
                "Block stream ended, reconnecting in {}s...",
                delay.as_secs()
            );
        }
        tokio::time::sleep(delay).await;
    }
}

/// Subscribe to new blocks only (no event decoding).
/// Auto-reconnects on WebSocket drops with exponential backoff.
pub async fn subscribe_blocks(
    client: &OnlineClient<SubtensorConfig>,
    json_output: bool,
) -> Result<()> {
    if !json_output {
        println!("Subscribed to finalized blocks. Ctrl+C to stop.\n");
    }

    // Run the block subscription loop with Ctrl+C handling (Issue 721)
    tokio::select! {
        result = subscribe_blocks_inner(client, json_output) => result,
        _ = tokio::signal::ctrl_c() => {
            if !json_output {
                eprintln!("\nInterrupted. Closing block subscription.");
            }
            Ok(())
        }
    }
}

/// Inner block subscription loop, factored out for Ctrl+C wrapping.
async fn subscribe_blocks_inner(
    client: &OnlineClient<SubtensorConfig>,
    json_output: bool,
) -> Result<()> {
    let mut reconnect_attempts = 0u32;
    let mut last_processed_block: Option<u64> = None;
    loop {
        let sub_result = client.blocks().subscribe_finalized().await;
        let mut block_sub = match sub_result {
            Ok(s) => {
                if reconnect_attempts > 0 && !json_output {
                    eprintln!("Reconnected to block stream.");
                }
                reconnect_attempts = 0;
                s
            }
            Err(e) => {
                reconnect_attempts += 1;
                if reconnect_attempts > MAX_RECONNECT_ATTEMPTS {
                    return Err(anyhow::anyhow!(
                        "Block subscription failed after {} reconnection attempts: {}",
                        MAX_RECONNECT_ATTEMPTS,
                        e
                    ));
                }
                let delay = std::time::Duration::from_secs(1 << reconnect_attempts.min(5));
                if !json_output {
                    eprintln!(
                        "Warning: subscription failed ({}), retrying in {}s...",
                        e,
                        delay.as_secs()
                    );
                }
                tokio::time::sleep(delay).await;
                continue;
            }
        };

        while let Some(block_result) = block_sub.next().await {
            let block = match block_result {
                Ok(b) => b,
                Err(e) => {
                    if !json_output {
                        eprintln!("Warning: block stream interrupted ({}), reconnecting...", e);
                    }
                    break;
                }
            };
            let number = block.number() as u64;

            // Detect and warn about gaps in block sequence (Issue 644)
            if let Some(last) = last_processed_block {
                let gap = number.saturating_sub(last + 1);
                if gap > 0 {
                    tracing::warn!(
                        last_block = last,
                        current_block = number,
                        missed_blocks = gap,
                        "Gap detected in block stream"
                    );
                    if !json_output {
                        eprintln!(
                            "Warning: {} block(s) missed (#{} to #{})",
                            gap, last + 1, number - 1
                        );
                    } else {
                        println!(
                            "{}",
                            serde_json::json!({
                                "warning": "gap_detected",
                                "missed_from": last + 1,
                                "missed_to": number - 1,
                                "missed_count": gap,
                            })
                        );
                    }
                }
            }
            last_processed_block = Some(number);
            let hash = format!("{:?}", block.hash());
            let extrinsic_count = match block.extrinsics().await {
                Ok(exts) => exts.len(),
                Err(_) => 0,
            };

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
                println!(
                    "Block #{} hash={} extrinsics={}",
                    number, hash, extrinsic_count
                );
            }
        }

        reconnect_attempts += 1;
        if reconnect_attempts > MAX_RECONNECT_ATTEMPTS {
            return Err(anyhow::anyhow!(
                "Block subscription ended after {} consecutive reconnection failures",
                MAX_RECONNECT_ATTEMPTS
            ));
        }
        let delay = std::time::Duration::from_secs(1 << reconnect_attempts.min(5));
        if !json_output {
            eprintln!(
                "Block stream ended, reconnecting in {}s...",
                delay.as_secs()
            );
        }
        tokio::time::sleep(delay).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ========== EventFilter::from_str tests ==========

    #[test]
    fn from_str_staking() {
        assert_eq!(EventFilter::from_str("staking").unwrap(), EventFilter::Staking);
    }

    #[test]
    fn from_str_stake() {
        assert_eq!(EventFilter::from_str("stake").unwrap(), EventFilter::Staking);
    }

    #[test]
    fn from_str_registration() {
        assert_eq!(EventFilter::from_str("registration").unwrap(), EventFilter::Registration);
    }

    #[test]
    fn from_str_register() {
        assert_eq!(EventFilter::from_str("register").unwrap(), EventFilter::Registration);
    }

    #[test]
    fn from_str_reg() {
        assert_eq!(EventFilter::from_str("reg").unwrap(), EventFilter::Registration);
    }

    #[test]
    fn from_str_transfer() {
        assert_eq!(EventFilter::from_str("transfer").unwrap(), EventFilter::Transfer);
    }

    #[test]
    fn from_str_transfers() {
        assert_eq!(EventFilter::from_str("transfers").unwrap(), EventFilter::Transfer);
    }

    #[test]
    fn from_str_weights() {
        assert_eq!(EventFilter::from_str("weights").unwrap(), EventFilter::Weights);
    }

    #[test]
    fn from_str_weight() {
        assert_eq!(EventFilter::from_str("weight").unwrap(), EventFilter::Weights);
    }

    #[test]
    fn from_str_subnet() {
        assert_eq!(EventFilter::from_str("subnet").unwrap(), EventFilter::Subnet);
    }

    #[test]
    fn from_str_subnets() {
        assert_eq!(EventFilter::from_str("subnets").unwrap(), EventFilter::Subnet);
    }

    #[test]
    fn from_str_unknown_falls_back_to_all() {
        assert_eq!(EventFilter::from_str("anything_else").unwrap(), EventFilter::All);
    }

    #[test]
    fn from_str_empty_falls_back_to_all() {
        assert_eq!(EventFilter::from_str("").unwrap(), EventFilter::All);
    }

    #[test]
    fn from_str_case_insensitive_staking() {
        assert_eq!(EventFilter::from_str("STAKING").unwrap(), EventFilter::Staking);
    }

    #[test]
    fn from_str_case_insensitive_transfer() {
        assert_eq!(EventFilter::from_str("Transfer").unwrap(), EventFilter::Transfer);
    }

    #[test]
    fn from_str_case_insensitive_weights() {
        assert_eq!(EventFilter::from_str("WEIGHTS").unwrap(), EventFilter::Weights);
    }

    #[test]
    fn from_str_case_insensitive_subnet() {
        assert_eq!(EventFilter::from_str("SUBNET").unwrap(), EventFilter::Subnet);
    }

    #[test]
    fn from_str_case_insensitive_registration() {
        assert_eq!(EventFilter::from_str("Registration").unwrap(), EventFilter::Registration);
    }

    // ========== EventFilter::matches() tests ==========

    // -- All variant matches everything --

    #[test]
    fn all_matches_any_pallet_and_variant() {
        assert!(EventFilter::All.matches("SubtensorModule", "StakeAdded"));
        assert!(EventFilter::All.matches("Balances", "Transfer"));
        assert!(EventFilter::All.matches("System", "ExtrinsicSuccess"));
        assert!(EventFilter::All.matches("Whatever", "Anything"));
    }

    // -- Staking variant --

    #[test]
    fn staking_matches_stake_added() {
        assert!(EventFilter::Staking.matches("SubtensorModule", "StakeAdded"));
    }

    #[test]
    fn staking_matches_stake_removed() {
        assert!(EventFilter::Staking.matches("SubtensorModule", "StakeRemoved"));
    }

    #[test]
    fn staking_matches_stake_moved() {
        assert!(EventFilter::Staking.matches("SubtensorModule", "StakeMoved"));
    }

    #[test]
    fn staking_matches_stake_swapped() {
        assert!(EventFilter::Staking.matches("SubtensorModule", "StakeSwapped"));
    }

    #[test]
    fn staking_matches_all_stake_removed() {
        assert!(EventFilter::Staking.matches("SubtensorModule", "AllStakeRemoved"));
    }

    #[test]
    fn staking_rejects_wrong_pallet() {
        assert!(!EventFilter::Staking.matches("Balances", "StakeAdded"));
    }

    #[test]
    fn staking_rejects_wrong_variant() {
        assert!(!EventFilter::Staking.matches("SubtensorModule", "NeuronRegistered"));
    }

    #[test]
    fn staking_rejects_transfer() {
        assert!(!EventFilter::Staking.matches("Balances", "Transfer"));
    }

    // -- Registration variant --

    #[test]
    fn registration_matches_neuron_registered() {
        assert!(EventFilter::Registration.matches("SubtensorModule", "NeuronRegistered"));
    }

    #[test]
    fn registration_matches_burned_register() {
        assert!(EventFilter::Registration.matches("SubtensorModule", "BurnedRegister"));
    }

    #[test]
    fn registration_matches_subnet_registered() {
        assert!(EventFilter::Registration.matches("SubtensorModule", "SubnetRegistered"));
    }

    #[test]
    fn registration_matches_pow_registered() {
        assert!(EventFilter::Registration.matches("SubtensorModule", "PowRegistered"));
    }

    #[test]
    fn registration_rejects_wrong_pallet() {
        assert!(!EventFilter::Registration.matches("Balances", "NeuronRegistered"));
    }

    #[test]
    fn registration_rejects_wrong_variant() {
        assert!(!EventFilter::Registration.matches("SubtensorModule", "StakeAdded"));
    }

    // -- Transfer variant --

    #[test]
    fn transfer_matches_balances_pallet() {
        assert!(EventFilter::Transfer.matches("Balances", "Transfer"));
    }

    #[test]
    fn transfer_matches_any_balances_variant() {
        assert!(EventFilter::Transfer.matches("Balances", "Deposit"));
        assert!(EventFilter::Transfer.matches("Balances", "Withdraw"));
        assert!(EventFilter::Transfer.matches("Balances", "Endowed"));
    }

    #[test]
    fn transfer_rejects_subtensor_module() {
        assert!(!EventFilter::Transfer.matches("SubtensorModule", "Transfer"));
    }

    #[test]
    fn transfer_rejects_system_pallet() {
        assert!(!EventFilter::Transfer.matches("System", "Transfer"));
    }

    // -- Weights variant --

    #[test]
    fn weights_matches_weights_set() {
        assert!(EventFilter::Weights.matches("SubtensorModule", "WeightsSet"));
    }

    #[test]
    fn weights_matches_weights_committed() {
        assert!(EventFilter::Weights.matches("SubtensorModule", "WeightsCommitted"));
    }

    #[test]
    fn weights_matches_weights_revealed() {
        assert!(EventFilter::Weights.matches("SubtensorModule", "WeightsRevealed"));
    }

    #[test]
    fn weights_matches_weights_batch_revealed() {
        assert!(EventFilter::Weights.matches("SubtensorModule", "WeightsBatchRevealed"));
    }

    #[test]
    fn weights_rejects_wrong_pallet() {
        assert!(!EventFilter::Weights.matches("Balances", "WeightsSet"));
    }

    #[test]
    fn weights_rejects_wrong_variant() {
        assert!(!EventFilter::Weights.matches("SubtensorModule", "StakeAdded"));
    }

    // -- Subnet variant --

    #[test]
    fn subnet_matches_hyperparams_set() {
        assert!(EventFilter::Subnet.matches("SubtensorModule", "SubnetHyperparamsSet"));
    }

    #[test]
    fn subnet_matches_identity_set() {
        assert!(EventFilter::Subnet.matches("SubtensorModule", "SubnetIdentitySet"));
    }

    #[test]
    fn subnet_matches_identity_removed() {
        assert!(EventFilter::Subnet.matches("SubtensorModule", "SubnetIdentityRemoved"));
    }

    #[test]
    fn subnet_matches_network_added() {
        assert!(EventFilter::Subnet.matches("SubtensorModule", "NetworkAdded"));
    }

    #[test]
    fn subnet_matches_network_removed() {
        assert!(EventFilter::Subnet.matches("SubtensorModule", "NetworkRemoved"));
    }

    #[test]
    fn subnet_matches_tempo_set() {
        assert!(EventFilter::Subnet.matches("SubtensorModule", "TempoSet"));
    }

    #[test]
    fn subnet_rejects_wrong_pallet() {
        assert!(!EventFilter::Subnet.matches("Balances", "NetworkAdded"));
    }

    #[test]
    fn subnet_rejects_wrong_variant() {
        assert!(!EventFilter::Subnet.matches("SubtensorModule", "StakeAdded"));
    }

    // -- Cross-filter isolation --

    #[test]
    fn staking_does_not_match_weight_events() {
        assert!(!EventFilter::Staking.matches("SubtensorModule", "WeightsSet"));
    }

    #[test]
    fn weights_does_not_match_staking_events() {
        assert!(!EventFilter::Weights.matches("SubtensorModule", "StakeAdded"));
    }

    #[test]
    fn registration_does_not_match_subnet_events() {
        assert!(!EventFilter::Registration.matches("SubtensorModule", "NetworkAdded"));
    }

    #[test]
    fn subnet_does_not_match_registration_events() {
        assert!(!EventFilter::Subnet.matches("SubtensorModule", "NeuronRegistered"));
    }

    // ========== ChainEvent Display tests ==========

    #[test]
    fn chain_event_display_format() {
        let event = ChainEvent {
            block_number: 12345,
            block_hash: "0xabc".to_string(),
            pallet: "SubtensorModule".to_string(),
            variant: "StakeAdded".to_string(),
            fields: "{amount: 100}".to_string(),
        };
        let display = format!("{}", event);
        assert_eq!(display, "#12345 SubtensorModule::StakeAdded {amount: 100}");
    }

    #[test]
    fn chain_event_display_zero_block() {
        let event = ChainEvent {
            block_number: 0,
            block_hash: "0x000".to_string(),
            pallet: "Balances".to_string(),
            variant: "Transfer".to_string(),
            fields: "{}".to_string(),
        };
        let display = format!("{}", event);
        assert_eq!(display, "#0 Balances::Transfer {}");
    }

    #[test]
    fn chain_event_display_empty_fields() {
        let event = ChainEvent {
            block_number: 999,
            block_hash: "0xdef".to_string(),
            pallet: "System".to_string(),
            variant: "ExtrinsicSuccess".to_string(),
            fields: "".to_string(),
        };
        let display = format!("{}", event);
        assert_eq!(display, "#999 System::ExtrinsicSuccess ");
    }

    #[test]
    fn chain_event_display_large_block_number() {
        let event = ChainEvent {
            block_number: 4_000_000,
            block_hash: "0xfff".to_string(),
            pallet: "SubtensorModule".to_string(),
            variant: "WeightsCommitted".to_string(),
            fields: "{netuid: 1}".to_string(),
        };
        let display = format!("{}", event);
        assert!(display.starts_with("#4000000 "));
        assert!(display.contains("SubtensorModule::WeightsCommitted"));
    }

    #[test]
    fn chain_event_display_contains_block_number() {
        let event = ChainEvent {
            block_number: 42,
            block_hash: "0x1".to_string(),
            pallet: "P".to_string(),
            variant: "V".to_string(),
            fields: "F".to_string(),
        };
        let display = format!("{}", event);
        assert!(display.contains("#42"));
    }

    #[test]
    fn chain_event_display_contains_pallet_and_variant() {
        let event = ChainEvent {
            block_number: 1,
            block_hash: "0x2".to_string(),
            pallet: "MyPallet".to_string(),
            variant: "MyEvent".to_string(),
            fields: "data".to_string(),
        };
        let display = format!("{}", event);
        assert!(display.contains("MyPallet::MyEvent"));
    }

    // ========== EventFilter Debug derive ==========

    #[test]
    fn event_filter_debug() {
        assert_eq!(format!("{:?}", EventFilter::All), "All");
        assert_eq!(format!("{:?}", EventFilter::Staking), "Staking");
        assert_eq!(format!("{:?}", EventFilter::Registration), "Registration");
        assert_eq!(format!("{:?}", EventFilter::Transfer), "Transfer");
        assert_eq!(format!("{:?}", EventFilter::Weights), "Weights");
        assert_eq!(format!("{:?}", EventFilter::Subnet), "Subnet");
    }

    // ========== EventFilter Clone + Copy + PartialEq ==========

    #[test]
    fn event_filter_clone_and_eq() {
        let f = EventFilter::Staking;
        let f2 = f;  // Copy
        let f3 = f.clone();
        assert_eq!(f, f2);
        assert_eq!(f, f3);
    }

    #[test]
    fn event_filter_inequality() {
        assert_ne!(EventFilter::Staking, EventFilter::Transfer);
        assert_ne!(EventFilter::All, EventFilter::Subnet);
        assert_ne!(EventFilter::Weights, EventFilter::Registration);
    }

    // ========== Issue 718/719: Structured extraction tests ==========

    /// Helper: build a Named composite with a netuid field.
    fn make_named_with_netuid(netuid: u128) -> Composite<()> {
        Composite::Named(vec![
            ("netuid".to_string(), subxt::ext::scale_value::Value::u128(netuid)),
        ])
    }

    /// Helper: build a Named composite without a netuid field.
    fn make_named_no_netuid() -> Composite<()> {
        Composite::Named(vec![
            ("amount".to_string(), subxt::ext::scale_value::Value::u128(1000)),
        ])
    }

    #[test]
    fn extract_netuid_named_match() {
        let composite = make_named_with_netuid(42);
        assert_eq!(extract_netuid(&composite), Some(42));
    }

    #[test]
    fn extract_netuid_named_no_field() {
        let composite = make_named_no_netuid();
        assert_eq!(extract_netuid(&composite), None);
    }

    #[test]
    fn extract_netuid_unnamed_returns_none() {
        // Issue 719: Unnamed composites should NOT match by accident
        let composite = Composite::Unnamed(vec![
            subxt::ext::scale_value::Value::u128(42),
        ]);
        assert_eq!(extract_netuid(&composite), None, "Unnamed(42) must not match as netuid");
    }

    #[test]
    fn extract_accounts_from_32_byte_composite() {
        // Build a 32-byte unnamed composite (AccountId pattern)
        let bytes: Vec<subxt::ext::scale_value::Value<()>> = (0u8..32)
            .map(|b| subxt::ext::scale_value::Value::u128(b as u128))
            .collect();
        let account_composite = Composite::Unnamed(bytes);
        let outer = Composite::Named(vec![
            ("who".to_string(), subxt::ext::scale_value::Value {
                value: ValueDef::Composite(account_composite),
                context: (),
            }),
        ]);
        let accounts = extract_accounts(&outer);
        assert_eq!(accounts.len(), 1, "Should extract exactly one account");
        assert!(accounts[0].starts_with("5"), "Should be a valid SS58 address: {}", accounts[0]);
    }

    #[test]
    fn extract_accounts_no_account_fields() {
        // Issue 718: A composite with no 32-byte fields should return empty
        let composite = Composite::Named(vec![
            ("amount".to_string(), subxt::ext::scale_value::Value::u128(1000)),
            ("netuid".to_string(), subxt::ext::scale_value::Value::u128(1)),
        ]);
        let accounts = extract_accounts(&composite);
        assert!(accounts.is_empty(), "Should not find any accounts");
    }

    #[test]
    fn extract_accounts_multiple_accounts() {
        // Build two different 32-byte AccountIds
        let bytes_a: Vec<subxt::ext::scale_value::Value<()>> = vec![1u8; 32]
            .into_iter()
            .map(|b| subxt::ext::scale_value::Value::u128(b as u128))
            .collect();
        let bytes_b: Vec<subxt::ext::scale_value::Value<()>> = vec![2u8; 32]
            .into_iter()
            .map(|b| subxt::ext::scale_value::Value::u128(b as u128))
            .collect();
        let outer = Composite::Named(vec![
            ("from".to_string(), subxt::ext::scale_value::Value {
                value: ValueDef::Composite(Composite::Unnamed(bytes_a)),
                context: (),
            }),
            ("to".to_string(), subxt::ext::scale_value::Value {
                value: ValueDef::Composite(Composite::Unnamed(bytes_b)),
                context: (),
            }),
        ]);
        let accounts = extract_accounts(&outer);
        assert_eq!(accounts.len(), 2, "Should find two accounts");
        assert_ne!(accounts[0], accounts[1], "Different bytes should produce different SS58");
    }

    #[test]
    fn try_composite_as_ss58_rejects_non_32_byte() {
        // A 16-byte composite should not be interpreted as an account
        let bytes: Vec<subxt::ext::scale_value::Value<()>> = (0u8..16)
            .map(|b| subxt::ext::scale_value::Value::u128(b as u128))
            .collect();
        let composite = Composite::Unnamed(bytes);
        assert!(try_composite_as_ss58(&composite).is_none());
    }

    // ========== Issue 644: Block gap detection tests ==========

    #[test]
    fn compute_block_gap_no_previous() {
        // First block ever — no gap possible
        assert_eq!(compute_block_gap(None, 100), 0);
    }

    #[test]
    fn compute_block_gap_consecutive() {
        // Consecutive blocks — no gap
        assert_eq!(compute_block_gap(Some(99), 100), 0);
    }

    #[test]
    fn compute_block_gap_one_missed() {
        // Block 101 follows 99 — missed block 100
        assert_eq!(compute_block_gap(Some(99), 101), 1);
    }

    #[test]
    fn compute_block_gap_many_missed() {
        // Block 200 follows 100 — missed blocks 101..199
        assert_eq!(compute_block_gap(Some(100), 200), 99);
    }

    #[test]
    fn compute_block_gap_same_block() {
        // Same block number (shouldn't happen, but handle gracefully)
        assert_eq!(compute_block_gap(Some(100), 100), 0);
    }

    #[test]
    fn compute_block_gap_overflow_protection() {
        // Current < last (shouldn't happen, saturating_sub protects)
        assert_eq!(compute_block_gap(Some(200), 100), 0);
    }

    // ──── Issue 100: extract_netuid rejects out-of-u16-range values ────

    #[test]
    fn extract_netuid_valid_value() {
        use subxt::ext::scale_value::Composite;
        let composite = Composite::Named(vec![
            ("netuid".to_string(), subxt::ext::scale_value::Value::u128(42)),
        ]);
        assert_eq!(extract_netuid(&composite), Some(42));
    }

    #[test]
    fn extract_netuid_max_u16() {
        use subxt::ext::scale_value::Composite;
        let composite = Composite::Named(vec![
            ("netuid".to_string(), subxt::ext::scale_value::Value::u128(65535)),
        ]);
        assert_eq!(extract_netuid(&composite), Some(65535));
    }

    #[test]
    fn extract_netuid_rejects_above_u16() {
        use subxt::ext::scale_value::Composite;
        // Value 65536 is above u16::MAX — should return None, not silently truncate to 0
        let composite = Composite::Named(vec![
            ("netuid".to_string(), subxt::ext::scale_value::Value::u128(65536)),
        ]);
        assert_eq!(extract_netuid(&composite), None);
    }

    #[test]
    fn extract_netuid_rejects_large_value() {
        use subxt::ext::scale_value::Composite;
        // 0x0001_0001 would truncate to 1 without the guard
        let composite = Composite::Named(vec![
            ("netuid".to_string(), subxt::ext::scale_value::Value::u128(0x0001_0001)),
        ]);
        assert_eq!(extract_netuid(&composite), None);
    }
}
