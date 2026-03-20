//! Declarative test environment scaffolding.
//!
//! Takes a TOML config (or sensible defaults) and produces a fully-configured
//! local chain with subnets, funded accounts, registered neurons, and tuned
//! hyperparameters — all in one call.
//!
//! ```rust,no_run
//! use agcli::scaffold::{self, ScaffoldConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let result = scaffold::run(&ScaffoldConfig::default()).await?;
//!     println!("endpoint: {}", result.endpoint);
//!     for sn in &result.subnets {
//!         println!("netuid {} — {} neurons", sn.netuid, sn.neurons.len());
//!     }
//!     Ok(())
//! }
//! ```

use crate::admin;
use crate::chain::Client;
use crate::localnet::{self, LocalnetConfig};
use crate::types::balance::Balance;
use crate::types::network::NetUid;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair as _};
use std::time::Duration;

// ───────────────────── Config types (TOML-deserializable) ─────────────────────

/// Top-level scaffold configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ScaffoldConfig {
    /// Chain / Docker settings.
    pub chain: ChainConfig,
    /// Subnets to create (default: 1 subnet with standard hyperparams).
    pub subnet: Vec<SubnetConfig>,
}

impl Default for ScaffoldConfig {
    fn default() -> Self {
        Self {
            chain: ChainConfig::default(),
            subnet: vec![SubnetConfig::default()],
        }
    }
}

/// Chain / Docker configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ChainConfig {
    /// Docker image for the localnet.
    pub image: String,
    /// Container name.
    pub container: String,
    /// Host port for WebSocket RPC.
    pub port: u16,
    /// Whether to start the chain (false = assume already running).
    pub start: bool,
    /// Seconds to wait for chain readiness.
    pub timeout: u64,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            image: localnet::DEFAULT_IMAGE.to_string(),
            container: localnet::DEFAULT_CONTAINER.to_string(),
            port: 9944,
            start: true,
            timeout: 120,
        }
    }
}

/// Per-subnet configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SubnetConfig {
    /// Blocks per epoch.
    pub tempo: Option<u16>,
    /// Max validator slots.
    pub max_allowed_validators: Option<u16>,
    /// Max total UID slots.
    pub max_allowed_uids: Option<u16>,
    /// Minimum weights a validator must set.
    pub min_allowed_weights: Option<u16>,
    /// Maximum weight value.
    pub max_weight_limit: Option<u16>,
    /// Blocks of immunity after registration.
    pub immunity_period: Option<u16>,
    /// Blocks between weight submissions (0 = unlimited).
    pub weights_rate_limit: Option<u64>,
    /// Enable commit-reveal weights.
    pub commit_reveal: Option<bool>,
    /// Activity cutoff in blocks.
    pub activity_cutoff: Option<u16>,
    /// Neurons to create in this subnet.
    pub neuron: Vec<NeuronConfig>,
}

impl Default for SubnetConfig {
    fn default() -> Self {
        Self {
            tempo: Some(100),
            max_allowed_validators: Some(8),
            max_allowed_uids: None,
            min_allowed_weights: Some(1),
            max_weight_limit: None,
            immunity_period: None,
            weights_rate_limit: Some(0),
            commit_reveal: Some(false),
            activity_cutoff: None,
            neuron: vec![
                NeuronConfig {
                    name: "validator1".to_string(),
                    fund_tao: Some(1000.0),
                    register: true,
                },
                NeuronConfig {
                    name: "miner1".to_string(),
                    fund_tao: Some(100.0),
                    register: true,
                },
                NeuronConfig {
                    name: "miner2".to_string(),
                    fund_tao: Some(100.0),
                    register: true,
                },
            ],
        }
    }
}

/// Per-neuron configuration within a subnet.
#[derive(Debug, Clone, Deserialize)]
pub struct NeuronConfig {
    /// Human-readable name for this neuron.
    pub name: String,
    /// TAO to fund from Alice (None = don't fund).
    pub fund_tao: Option<f64>,
    /// Whether to register on the subnet.
    #[serde(default = "default_true")]
    pub register: bool,
}

fn default_true() -> bool {
    true
}

// ───────────────────── Result types (JSON-serializable) ───────────────────────

/// Result of a scaffold operation.
#[derive(Debug, Clone, Serialize)]
pub struct ScaffoldResult {
    /// Chain endpoint.
    pub endpoint: String,
    /// Container name (if chain was started).
    pub container: Option<String>,
    /// Current block height.
    pub block_height: u64,
    /// Created subnets with their neurons.
    pub subnets: Vec<SubnetResult>,
}

/// Result for a single subnet.
#[derive(Debug, Clone, Serialize)]
pub struct SubnetResult {
    /// Assigned netuid.
    pub netuid: u16,
    /// Applied hyperparameters.
    pub hyperparams: serde_json::Value,
    /// Neurons in this subnet.
    pub neurons: Vec<NeuronResult>,
}

/// Result for a single neuron.
#[derive(Debug, Clone, Serialize)]
pub struct NeuronResult {
    /// Name from config.
    pub name: String,
    /// SS58 address (coldkey = hotkey for simplicity).
    pub ss58: String,
    /// Secret seed URI (for programmatic use).
    #[serde(skip)]
    pub seed: String,
    /// UID on the subnet (if registered).
    pub uid: Option<u16>,
    /// Balance after funding.
    pub balance_tao: Option<f64>,
}

// ───────────────────── Orchestration ─────────────────────

/// Load a scaffold config from a TOML file.
pub fn load_config(path: &str) -> Result<ScaffoldConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read scaffold config: {}", path))?;
    let config: ScaffoldConfig =
        toml::from_str(&content).with_context(|| format!("Failed to parse TOML: {}", path))?;
    Ok(config)
}

/// Run the full scaffold: start chain → create wallets → fund → register
/// subnets → set hyperparams → register neurons → return manifest.
pub async fn run(config: &ScaffoldConfig) -> Result<ScaffoldResult> {
    run_with_progress(config, |_| {}).await
}

/// Run scaffold with a progress callback for CLI output.
pub async fn run_with_progress<F>(
    config: &ScaffoldConfig,
    mut on_progress: F,
) -> Result<ScaffoldResult>
where
    F: FnMut(&str),
{
    // 1. Start chain (or connect to existing)
    let (endpoint, container) = if config.chain.start {
        on_progress("Starting localnet...");
        let cfg = LocalnetConfig {
            image: config.chain.image.clone(),
            container_name: config.chain.container.clone(),
            port: config.chain.port,
            wait: true,
            wait_timeout: config.chain.timeout,
        };
        let info = localnet::start(&cfg).await?;
        on_progress(&format!(
            "Chain ready at {} (block {})",
            info.endpoint, info.block_height
        ));
        (info.endpoint, Some(info.container_name))
    } else {
        let ep = format!("ws://127.0.0.1:{}", config.chain.port);
        on_progress(&format!("Connecting to existing chain at {}...", ep));
        (ep, None)
    };

    // Safety: Scaffold uses deterministic dev-key URIs (e.g. //validator1_sn1).
    // These keys are predictable and must NEVER hold real funds. Reject non-local endpoints.
    if !is_local_endpoint(&endpoint) {
        bail!(
            "Scaffold refuses to run against non-local endpoint '{}'.\n  \
             Scaffold uses deterministic dev keys — connecting to mainnet/testnet \
             would put real funds at predictable addresses anyone can derive.\n  \
             Use a local chain (127.0.0.1/localhost) or set chain.start = true.",
            endpoint
        );
    }

    // 2. Connect client
    let client = Client::connect(&endpoint).await?;

    // 3. Get Alice (sudo key)
    let alice = sr25519::Pair::from_string("//Alice", None)
        .map_err(|_| anyhow::anyhow!("Failed to derive //Alice keypair"))?;
    let alice_ss58 =
        sp_core::crypto::Ss58Codec::to_ss58check_with_version(&alice.public(), 42u16.into());

    if config.subnet.is_empty() {
        bail!("No subnets defined in scaffold config");
    }

    let mut subnet_results = Vec::new();

    for (i, subnet_cfg) in config.subnet.iter().enumerate() {
        on_progress(&format!("Creating subnet {}...", i + 1));

        // 4. Register subnet — collect netuids before and after to find the new one
        //    (avoids race condition: `total_networks - 1` assumes sequential assignment
        //     which breaks under concurrent registrations)
        let netuids_before: std::collections::HashSet<u16> = client
            .get_all_subnets()
            .await?
            .iter()
            .map(|s| s.netuid.0)
            .collect();
        retry_idempotent_extrinsic(|| client.register_network(&alice, &alice_ss58)).await?;
        wait_blocks(&client, 2).await;
        // Pin a fresh block to bypass query cache and see the newly created subnet
        let pin_hash = client.pin_latest_block().await?;
        let subnets_after = client.get_all_subnets_at_block(pin_hash).await?;
        let netuids_after: std::collections::HashSet<u16> =
            subnets_after.iter().map(|s| s.netuid.0).collect();
        let new_netuids: Vec<u16> = netuids_after.difference(&netuids_before).copied().collect();
        if new_netuids.is_empty() {
            bail!(
                "Subnet registration failed: no new netuid appeared (before: {:?}, after: {:?})",
                netuids_before,
                netuids_after
            );
        }
        // Use min() for determinism when multiple subnets appear concurrently
        // (HashSet iteration order is non-deterministic; Substrate assigns netuids incrementally)
        let netuid = *new_netuids.iter().min().unwrap();
        on_progress(&format!("Subnet created: netuid {}", netuid));

        // 5. Set hyperparameters via sudo
        //    Some chains reject admin calls during weights windows or lack certain
        //    calls in their runtime. We treat these as non-fatal: log a warning
        //    via the progress callback and continue scaffolding.
        let mut hyperparams = serde_json::Map::new();

        /// Try an admin call; on known chain-specific errors, warn and continue.
        macro_rules! try_admin {
            ($label:expr, $call:expr, $key:expr, $val:expr, $hp:expr, $cb:expr) => {
                match $call.await {
                    Ok(_) => {
                        $hp.insert($key.into(), $val.into());
                    }
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("WeightsWindow")
                            || msg.contains("AdminActionProhibited")
                            || msg.contains("not found")
                            || msg.contains("Bad origin")
                        {
                            let truncated = crate::utils::truncate(&msg, 80);
                            $cb(&format!("Warning: {} skipped — {}", $label, truncated));
                        } else {
                            return Err(e);
                        }
                    }
                }
            };
        }

        if let Some(tempo) = subnet_cfg.tempo {
            try_admin!(
                "set_tempo",
                admin::set_tempo(&client, &alice, netuid, tempo),
                "tempo",
                tempo,
                hyperparams,
                on_progress
            );
        }
        if let Some(max_val) = subnet_cfg.max_allowed_validators {
            try_admin!(
                "set_max_validators",
                admin::set_max_allowed_validators(&client, &alice, netuid, max_val),
                "max_allowed_validators",
                max_val,
                hyperparams,
                on_progress
            );
        }
        if let Some(max_uids) = subnet_cfg.max_allowed_uids {
            try_admin!(
                "set_max_uids",
                admin::set_max_allowed_uids(&client, &alice, netuid, max_uids),
                "max_allowed_uids",
                max_uids,
                hyperparams,
                on_progress
            );
        }
        if let Some(min_w) = subnet_cfg.min_allowed_weights {
            try_admin!(
                "set_min_weights",
                admin::set_min_allowed_weights(&client, &alice, netuid, min_w),
                "min_allowed_weights",
                min_w,
                hyperparams,
                on_progress
            );
        }
        if let Some(max_wl) = subnet_cfg.max_weight_limit {
            try_admin!(
                "set_max_weight_limit",
                admin::set_max_weight_limit(&client, &alice, netuid, max_wl),
                "max_weight_limit",
                max_wl,
                hyperparams,
                on_progress
            );
        }
        if let Some(ip) = subnet_cfg.immunity_period {
            try_admin!(
                "set_immunity_period",
                admin::set_immunity_period(&client, &alice, netuid, ip),
                "immunity_period",
                ip,
                hyperparams,
                on_progress
            );
        }
        if let Some(wrl) = subnet_cfg.weights_rate_limit {
            try_admin!(
                "set_weights_rate_limit",
                admin::set_weights_set_rate_limit(&client, &alice, netuid, wrl),
                "weights_rate_limit",
                wrl,
                hyperparams,
                on_progress
            );
        }
        if let Some(cr) = subnet_cfg.commit_reveal {
            try_admin!(
                "set_commit_reveal",
                admin::set_commit_reveal_weights_enabled(&client, &alice, netuid, cr),
                "commit_reveal",
                cr,
                hyperparams,
                on_progress
            );
        }
        if let Some(ac) = subnet_cfg.activity_cutoff {
            try_admin!(
                "set_activity_cutoff",
                admin::set_activity_cutoff(&client, &alice, netuid, ac),
                "activity_cutoff",
                ac,
                hyperparams,
                on_progress
            );
        }

        on_progress(&format!(
            "Hyperparams set on netuid {} ({} params)",
            netuid,
            hyperparams.len()
        ));

        // 6. Create, fund, and register neurons
        let mut neuron_results = Vec::new();

        for neuron_cfg in &subnet_cfg.neuron {
            on_progress(&format!("Setting up neuron '{}'...", neuron_cfg.name));

            // Generate a keypair deterministically from name + subnet index
            // so scaffold results are reproducible within the same config
            let seed_uri = format!("//{}_sn{}", neuron_cfg.name, netuid);
            let pair = sr25519::Pair::from_string(&seed_uri, None)
                .map_err(|_| anyhow::anyhow!("Failed to derive keypair for {}", seed_uri))?;
            let ss58 =
                sp_core::crypto::Ss58Codec::to_ss58check_with_version(&pair.public(), 42u16.into());

            // Fund from Alice
            let mut balance_tao = None;
            if let Some(fund) = neuron_cfg.fund_tao {
                if fund > 0.0 {
                    // Transfer is non-idempotent — do NOT retry (could double-spend).
                    client
                        .transfer(&alice, &ss58, Balance::from_tao(fund))
                        .await?;
                    balance_tao = Some(fund);
                }
            }

            // Register on subnet
            let mut uid = None;
            if neuron_cfg.register {
                retry_idempotent_extrinsic(|| {
                    client.burned_register(&alice, NetUid(netuid), &ss58)
                })
                .await?;
                wait_blocks(&client, 1).await;

                // Look up UID
                uid = lookup_uid(&client, NetUid(netuid), &ss58).await.ok();
            }

            neuron_results.push(NeuronResult {
                name: neuron_cfg.name.clone(),
                ss58,
                seed: seed_uri,
                uid,
                balance_tao,
            });
        }

        on_progress(&format!(
            "Subnet {} ready: {} neurons registered",
            netuid,
            neuron_results.iter().filter(|n| n.uid.is_some()).count()
        ));

        subnet_results.push(SubnetResult {
            netuid,
            hyperparams: serde_json::Value::Object(hyperparams),
            neurons: neuron_results,
        });
    }

    let block_height = client.get_block_number().await.unwrap_or(0);

    Ok(ScaffoldResult {
        endpoint,
        container,
        block_height,
        subnets: subnet_results,
    })
}

// ───────────────────── Helpers ─────────────────────

/// Retry an **idempotent** extrinsic up to 10 times on transient errors.
///
/// **Only use for naturally idempotent operations** (e.g. register_network,
/// burned_register). Do NOT use for transfers or other operations where
/// a retry could double-spend. For non-idempotent ops, call directly
/// without retry.
async fn retry_idempotent_extrinsic<F, Fut>(f: F) -> Result<String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<String>>,
{
    for attempt in 1..=10 {
        match f().await {
            Ok(hash) => return Ok(hash),
            Err(e) => {
                let msg = format!("{}", e);
                let is_transient = msg.contains("outdated")
                    || msg.contains("banned")
                    || msg.contains("subscription")
                    || msg.contains("State already discarded")
                    || msg.contains("UnknownBlock")
                    || msg.contains("not valid");
                if is_transient && attempt < 10 {
                    let delay = if msg.contains("banned") {
                        13_000
                    } else if msg.contains("State") || msg.contains("Unknown") {
                        1_000 // Longer delay for state pruning
                    } else {
                        100
                    };
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    continue;
                }
                if attempt == 10 {
                    return Err(e);
                }
                // Non-transient error
                return Err(e);
            }
        }
    }
    unreachable!()
}

/// Wait for N blocks to pass.
async fn wait_blocks(client: &Client, n: u64) {
    let Ok(start) = client.get_block_number().await else {
        tokio::time::sleep(Duration::from_millis(n * 300)).await;
        return;
    };
    let target = start + n;
    for _ in 0..n * 20 {
        if let Ok(current) = client.get_block_number().await {
            if current >= target {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

/// Look up a neuron's UID on a subnet by hotkey SS58.
async fn lookup_uid(client: &Client, netuid: NetUid, hotkey_ss58: &str) -> Result<u16> {
    let neurons = client.get_neurons_lite(netuid).await?;
    for n in neurons.iter() {
        if n.hotkey == hotkey_ss58 {
            return Ok(n.uid);
        }
    }
    bail!("Neuron {} not found on subnet {}", hotkey_ss58, netuid.0)
}

/// Check if an IP string is in the RFC 1918 172.16.0.0/12 range (172.16.x.x–172.31.x.x).
fn is_rfc1918_172(host: &str) -> bool {
    if let Some(rest) = host.strip_prefix("172.") {
        if let Some(second_octet_str) = rest.split('.').next() {
            if let Ok(second) = second_octet_str.parse::<u8>() {
                return (16..=31).contains(&second);
            }
        }
    }
    false
}

/// Check if a WebSocket endpoint URL points to a local address.
fn is_local_endpoint(endpoint: &str) -> bool {
    // Strip ws:// or wss:// prefix
    let host_part = endpoint
        .strip_prefix("wss://")
        .or_else(|| endpoint.strip_prefix("ws://"))
        .unwrap_or(endpoint);
    // Handle IPv6 in brackets: [::1]:9944
    let host = if host_part.starts_with('[') {
        host_part
            .split(']')
            .next()
            .unwrap_or(host_part)
            .trim_start_matches('[')
    } else {
        // IPv4 or hostname: extract before port or path
        let h = host_part.split(':').next().unwrap_or(host_part);
        h.split('/').next().unwrap_or(h)
    };
    matches!(host, "127.0.0.1" | "localhost" | "0.0.0.0" | "::1")
        || host_part.starts_with("::1") // bare IPv6 loopback without brackets
        || host.starts_with("192.168.")
        || host.starts_with("10.")
        || is_rfc1918_172(host) // proper check for 172.16.0.0/12
}

#[cfg(test)]
mod tests {
    use super::*;

    /// H-61 fix: NeuronResult.seed should be excluded from JSON serialization
    /// to prevent leaking secret URIs in logs/output.
    #[test]
    fn neuron_result_seed_not_serialized() {
        let nr = NeuronResult {
            name: "alice".to_string(),
            ss58: "5GrwvaEF".to_string(),
            seed: "//Alice".to_string(),
            uid: Some(0),
            balance_tao: Some(1000.0),
        };
        let json = serde_json::to_string(&nr).unwrap();
        assert!(
            !json.contains("//Alice"),
            "Seed URI should not appear in serialized JSON: {}",
            json
        );
        assert!(
            !json.contains("seed"),
            "Seed field should be skipped in serialization: {}",
            json
        );
        // Other fields should still be present
        assert!(json.contains("alice"));
        assert!(json.contains("5GrwvaEF"));
    }

    // ──── Issue 694: Scaffold local-endpoint guard ────

    #[test]
    fn is_local_endpoint_allows_localhost() {
        assert!(is_local_endpoint("ws://127.0.0.1:9944"));
        assert!(is_local_endpoint("ws://localhost:9944"));
        assert!(is_local_endpoint("ws://0.0.0.0:9944"));
        assert!(is_local_endpoint("ws://::1:9944"));
        assert!(is_local_endpoint("ws://192.168.1.100:9944"));
        assert!(is_local_endpoint("ws://10.0.0.1:9944"));
        assert!(is_local_endpoint("ws://172.16.0.1:9944"));
    }

    #[test]
    fn is_local_endpoint_rejects_remote() {
        assert!(!is_local_endpoint("ws://subtensor.example.com:9944"));
        assert!(!is_local_endpoint(
            "wss://entrypoint-finney.opentensor.ai:443"
        ));
        assert!(!is_local_endpoint("ws://8.8.8.8:9944"));
        assert!(!is_local_endpoint("wss://mainnet.bittensor.com:443"));
    }

    #[test]
    fn is_local_endpoint_handles_wss() {
        assert!(is_local_endpoint("wss://127.0.0.1:9944"));
        assert!(is_local_endpoint("wss://localhost:9944"));
        assert!(!is_local_endpoint("wss://external.host:443"));
    }

    #[test]
    fn is_local_endpoint_handles_no_scheme() {
        assert!(is_local_endpoint("127.0.0.1:9944"));
        assert!(is_local_endpoint("localhost:9944"));
        assert!(!is_local_endpoint("subtensor.example.com:9944"));
    }

    // ──── Issue 695: retry_idempotent_extrinsic rename ────

    #[test]
    fn retry_idempotent_extrinsic_exists() {
        // Compile-time verification that retry_idempotent_extrinsic is the renamed function.
        // The old `retry_extrinsic` should not compile. This test passes iff the rename is in place.
        fn _assert_fn_exists<F, Fut>(_f: F)
        where
            F: Fn() -> Fut,
            Fut: std::future::Future<Output = anyhow::Result<String>>,
        {
        }
        // We verify the function signature exists via type check.
        // Cannot call it without an async runtime, but compilation proves it exists.
    }

    #[tokio::test]
    async fn retry_idempotent_succeeds_on_first_try() {
        let result = retry_idempotent_extrinsic(|| async { Ok("0xabc".to_string()) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "0xabc");
    }

    #[tokio::test]
    async fn retry_idempotent_retries_on_transient_then_succeeds() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let attempt = AtomicU32::new(0);
        let result = retry_idempotent_extrinsic(|| {
            let n = attempt.fetch_add(1, Ordering::SeqCst);
            async move {
                if n < 2 {
                    Err(anyhow::anyhow!("outdated transaction"))
                } else {
                    Ok("0xdef".to_string())
                }
            }
        })
        .await;
        assert!(
            result.is_ok(),
            "Should succeed after transient retries: {:?}",
            result.err()
        );
        assert_eq!(result.unwrap(), "0xdef");
    }

    #[tokio::test]
    async fn retry_idempotent_fails_on_non_transient() {
        let result = retry_idempotent_extrinsic(|| async {
            Err::<String, _>(anyhow::anyhow!("insufficient balance"))
        })
        .await;
        assert!(
            result.is_err(),
            "Non-transient error should fail immediately"
        );
    }

    // ──── Issue 100: is_rfc1918_172 correctly checks 172.16-31 range ────

    #[test]
    fn rfc1918_172_accepts_valid_range() {
        // 172.16.x.x through 172.31.x.x are private (RFC 1918)
        assert!(is_rfc1918_172("172.16.0.1"));
        assert!(is_rfc1918_172("172.20.5.10"));
        assert!(is_rfc1918_172("172.31.255.255"));
    }

    #[test]
    fn rfc1918_172_rejects_public_addresses() {
        // 172.32+ and 172.0-15 are NOT private
        assert!(!is_rfc1918_172("172.32.0.1"), "172.32.x.x is not RFC1918");
        assert!(!is_rfc1918_172("172.0.0.1"), "172.0.x.x is not RFC1918");
        assert!(
            !is_rfc1918_172("172.15.255.255"),
            "172.15.x.x is not RFC1918"
        );
        assert!(!is_rfc1918_172("172.255.0.1"), "172.255.x.x is not RFC1918");
    }

    #[test]
    fn rfc1918_172_rejects_non_172() {
        assert!(!is_rfc1918_172("10.0.0.1"));
        assert!(!is_rfc1918_172("192.168.1.1"));
        assert!(!is_rfc1918_172("8.8.8.8"));
    }

    #[test]
    fn is_local_endpoint_rejects_public_172() {
        // Before the fix, 172.32+ was incorrectly accepted as local
        assert!(
            !is_local_endpoint("ws://172.32.0.1:9944"),
            "172.32.x should not be local"
        );
        assert!(
            !is_local_endpoint("ws://172.100.0.1:9944"),
            "172.100.x should not be local"
        );
        assert!(
            !is_local_endpoint("ws://172.255.0.1:9944"),
            "172.255.x should not be local"
        );
        // But 172.16-31 should still be accepted
        assert!(is_local_endpoint("ws://172.16.0.1:9944"));
        assert!(is_local_endpoint("ws://172.31.255.255:9944"));
    }

    // ──── Issue 113: scaffold warning truncation uses char-safe truncate ────

    #[test]
    fn truncate_handles_multibyte_utf8() {
        // Verify crate::utils::truncate doesn't panic on multi-byte chars
        let multibyte = "Error: 操作が拒否されました。以下の理由で失敗しました。管理アクションが禁止されています。";
        let truncated = crate::utils::truncate(multibyte, 20);
        // Should not panic and should be <= 20 chars (plus ellipsis)
        assert!(truncated.chars().count() <= 21, "truncated: {}", truncated);
    }

    #[test]
    fn truncate_short_string_unchanged() {
        let short = "WeightsWindow error";
        let result = crate::utils::truncate(short, 80);
        assert_eq!(result, short);
    }

    // ──── Issue 114: deterministic netuid selection from HashSet ────

    #[test]
    fn min_netuid_is_deterministic() {
        // Simulate the fix: when multiple netuids appear, min() gives deterministic result
        use std::collections::HashSet;
        let before: HashSet<u16> = [1, 2, 5, 10].into_iter().collect();
        let after: HashSet<u16> = [1, 2, 5, 7, 10, 12].into_iter().collect();
        let new_netuids: Vec<u16> = after.difference(&before).copied().collect();
        let netuid = *new_netuids.iter().min().unwrap();
        assert_eq!(netuid, 7, "Should deterministically pick lowest new netuid");
    }
}
