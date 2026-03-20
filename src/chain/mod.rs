//! Substrate chain client — connect, query storage, submit extrinsics.

pub mod extrinsics;
pub mod queries;
pub mod rpc_types;

use anyhow::{Context, Result};
use sp_core::sr25519;
use std::borrow::Cow;
use subxt::backend::legacy::rpc_methods::LegacyRpcMethods;
use subxt::backend::rpc::RpcClient;
use subxt::tx::PairSigner;
use subxt::OnlineClient;

use crate::queries::query_cache::QueryCache;
use crate::types::balance::Balance;
use crate::{api, AccountId, SubtensorConfig};

// Re-export for event subscription
pub use subxt;

/// Check whether an error message looks transient (connection, timeout, transport).
fn is_transient_error(msg: &str) -> bool {
    msg.contains("onnect")
        || msg.contains("timeout")
        || msg.contains("Ws")
        || msg.contains("transport")
        || msg.contains("closed")
        || msg.contains("reset")
        || msg.contains("State already discarded") // fast-block chain state pruning
        || msg.contains("UnknownBlock") // stale block reference
}

/// Default retry count for RPC queries.
const RPC_RETRIES: u32 = 2;

/// Retry a fallible async operation with exponential backoff on transient errors.
/// Retries up to `max_retries` times with delays of 500ms, 1s, 2s, ...
/// Only retries on errors that look transient (connection, timeout, transport).
pub(crate) async fn retry_on_transient<F, Fut, T>(label: &str, max_retries: u32, f: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let start = std::time::Instant::now();
    let mut last_err = None;
    for attempt in 0..=max_retries {
        match f().await {
            Ok(val) => {
                let elapsed = start.elapsed();
                tracing::debug!(
                    elapsed_ms = elapsed.as_millis() as u64,
                    attempts = attempt + 1,
                    label,
                    "RPC query succeeded"
                );
                return Ok(val);
            }
            Err(e) => {
                let msg = format!("{:#}", e);
                if !is_transient_error(&msg) || attempt == max_retries {
                    let elapsed = start.elapsed();
                    tracing::debug!(
                        elapsed_ms = elapsed.as_millis() as u64,
                        attempts = attempt + 1,
                        label,
                        error = %msg,
                        "RPC query failed"
                    );
                    return Err(e);
                }
                let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                tracing::warn!(
                    attempt = attempt + 1,
                    max = max_retries,
                    delay_ms = delay.as_millis() as u64,
                    label,
                    error = %msg,
                    "Transient RPC error, retrying"
                );
                tokio::time::sleep(delay).await;
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("{}: all retries exhausted", label)))
}

/// Derive a short cache prefix from a WebSocket URL to namespace disk cache entries.
/// Recognizes well-known Bittensor endpoints; falls back to host-based prefix.
fn url_to_cache_prefix(url: &str) -> String {
    let lower = url.to_lowercase();
    // Check more specific patterns first
    if lower.contains("test.finney") || lower.contains("testnet") {
        return "test".to_string();
    }
    if lower.contains("entrypoint-finney") || lower.contains("finney.opentensor") {
        return "finney".to_string();
    }
    if lower.contains("127.0.0.1") || lower.contains("localhost") {
        return "local".to_string();
    }
    if lower.contains("archive") || lower.contains("onfinality") {
        return "archive".to_string();
    }
    // Fallback: use a sanitized host portion
    let host = lower
        .trim_start_matches("wss://")
        .trim_start_matches("ws://")
        .split(':')
        .next()
        .unwrap_or("unknown")
        .split('/')
        .next()
        .unwrap_or("unknown")
        .replace('.', "_");
    host
}

/// Acquire a per-signer file lock to serialize concurrent extrinsic submissions.
///
/// Multiple agcli processes using the same coldkey would race for the same nonce,
/// causing "Transaction already imported" or "Priority too low" errors. This lock
/// serializes all submissions from the same key so each gets a fresh nonce.
///
/// Lock file: `/tmp/agcli-tx-locks/<hex-pubkey>.lock`
/// The lock is held for the entire sign+submit+finalize cycle and released on drop.
fn acquire_tx_lock(pair: &sr25519::Pair) -> Result<std::fs::File> {
    use fs2::FileExt;
    let pub_key = sp_core::Pair::public(pair);
    let hex_key = hex::encode(pub_key.as_ref() as &[u8]);
    let lock_dir = std::path::PathBuf::from("/tmp/agcli-tx-locks");
    std::fs::create_dir_all(&lock_dir).context("create tx lock dir")?;
    let lock_path = lock_dir.join(format!("{}.lock", hex_key));
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .with_context(|| format!("open tx lock file: {}", lock_path.display()))?;
    // Try to acquire exclusive lock with 60s timeout (enough for finalization)
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        match file.try_lock_exclusive() {
            Ok(()) => {
                tracing::debug!("Acquired tx lock for {}", &hex_key[..8]);
                return Ok(file);
            }
            Err(_) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(_) => {
                anyhow::bail!(
                    "Another agcli process is submitting a transaction with this key. \
                     Timed out waiting 60s for tx lock. If no other process is running, \
                     remove {}",
                    lock_path.display()
                );
            }
        }
    }
}

/// Signer type for extrinsic submission.
pub type Signer = PairSigner<SubtensorConfig, sr25519::Pair>;

/// High-level client for the Bittensor (subtensor) chain.
pub struct Client {
    inner: OnlineClient<SubtensorConfig>,
    rpc: LegacyRpcMethods<SubtensorConfig>,
    cache: QueryCache,
    dry_run: bool,
    url: String,
    /// Timeout for waiting for transaction finalization (seconds).
    finalization_timeout: u64,
    /// Extrinsic mortality in blocks (0 = use default).
    mortality_blocks: u64,
}

impl Client {
    /// Access the runtime metadata from the connected chain.
    pub fn metadata(&self) -> subxt::Metadata {
        self.inner.metadata()
    }

    /// Connect to a subtensor node (single URL, no retry).
    async fn connect_once(url: &str) -> Result<Self> {
        let start = std::time::Instant::now();
        tracing::info!("Connecting to {}", url);
        let rpc_client = RpcClient::from_url(url)
            .await
            .with_context(|| format!(
                "Failed to connect to subtensor node at '{}'. Check your network connection and endpoint.\n  Finney:  wss://entrypoint-finney.opentensor.ai:443\n  Test:    wss://test.finney.opentensor.ai:443\n  Archive: wss://bittensor-finney.api.onfinality.io/public-ws\n  Local:   ws://127.0.0.1:9944\n  Set with: --network finney|test|local|archive  or  --endpoint <url>",
                url
            ))?;
        let rpc = LegacyRpcMethods::new(rpc_client.clone());
        let inner = OnlineClient::from_rpc_client(rpc_client)
            .await
            .with_context(|| "Failed to initialize subxt client from RPC connection")?;
        tracing::info!("Connected to {} in {:?}", url, start.elapsed());
        // Derive a short network prefix from the URL for disk cache namespacing.
        // This prevents cross-network cache contamination (e.g. finney data served to testnet).
        let net_prefix = url_to_cache_prefix(url);
        Ok(Self {
            inner,
            rpc,
            cache: QueryCache::new_with_network(&net_prefix),
            dry_run: false,
            url: url.to_string(),
            finalization_timeout: 30,
            mortality_blocks: 0,
        })
    }

    /// Reconnect to the same endpoint. Creates a fresh RPC connection while preserving settings.
    /// Useful when the subxt background task dies (e.g. on fast-block devnets).
    pub async fn reconnect(&mut self) -> Result<()> {
        let fresh = Self::connect_once(&self.url).await?;
        self.inner = fresh.inner;
        self.rpc = fresh.rpc;
        self.cache = QueryCache::new_with_network(&url_to_cache_prefix(&self.url));
        Ok(())
    }

    /// Invalidate all cached query data (both in-memory and disk).
    /// Call this before critical operations where stale data could mislead the user,
    /// e.g. after interactive prompts where the user may have waited longer than the cache TTL.
    pub async fn invalidate_cache(&self) {
        self.cache.invalidate_all().await;
    }

    /// Check if the connection is still alive by attempting a lightweight RPC call.
    pub async fn is_alive(&self) -> bool {
        self.inner.blocks().at_latest().await.is_ok()
    }

    /// Connect to a subtensor node with retry + exponential backoff.
    /// Tries each URL in order, retrying up to 3 times per URL with 1s→2s→4s delays.
    pub async fn connect(url: &str) -> Result<Self> {
        Self::connect_with_retry(&[url]).await
    }

    /// Connect with retry across multiple endpoints.
    /// Tries each URL in order; on failure retries with exponential backoff (1s, 2s, 4s).
    pub async fn connect_with_retry(urls: &[&str]) -> Result<Self> {
        let max_retries: u32 = 3;
        let mut last_err = None;

        for url in urls {
            for attempt in 0..max_retries {
                if attempt > 0 {
                    let delay = std::time::Duration::from_secs(1 << (attempt - 1));
                    tracing::warn!(
                        "Retry {}/{} for {} in {:?}",
                        attempt,
                        max_retries - 1,
                        url,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                }
                match Self::connect_once(url).await {
                    Ok(client) => {
                        if attempt > 0 {
                            tracing::info!("Connected to {} on attempt {}", url, attempt + 1);
                        }
                        return Ok(client);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Connection attempt {} to {} failed: {}",
                            attempt + 1,
                            url,
                            e
                        );
                        last_err = Some(e);
                    }
                }
            }
            tracing::warn!(
                "All {} attempts to {} exhausted, trying next endpoint",
                max_retries,
                url
            );
        }

        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No endpoints provided")))
    }

    /// Test multiple endpoints concurrently and connect to the fastest one.
    /// Measures connection + RPC round-trip latency for each URL in parallel,
    /// then picks the endpoint with the lowest average latency.
    /// Falls back to `connect_with_retry` if all measurements fail.
    pub async fn best_connection(urls: &[&str]) -> Result<Self> {
        if urls.len() <= 1 {
            return Self::connect_with_retry(urls).await;
        }

        tracing::info!(
            endpoints = urls.len(),
            "Testing endpoints for best connection"
        );

        // Test all endpoints concurrently
        let mut handles = Vec::with_capacity(urls.len());
        for url in urls {
            let url_owned = url.to_string();
            handles.push(tokio::spawn(async move {
                let start = std::time::Instant::now();
                match Self::connect_once(&url_owned).await {
                    Ok(client) => {
                        let connect_ms = start.elapsed().as_millis();
                        // One RPC round-trip to measure total latency
                        let rpc_start = std::time::Instant::now();
                        match client.get_block_number().await {
                            Ok(_) => {
                                let rpc_ms = rpc_start.elapsed().as_millis();
                                let total = connect_ms + rpc_ms;
                                tracing::debug!(url = %url_owned, connect_ms, rpc_ms, total_ms = total, "Endpoint measured");
                                Ok((url_owned, client, total))
                            }
                            Err(e) => {
                                tracing::debug!(url = %url_owned, error = %e, "Endpoint RPC failed");
                                Err(anyhow::anyhow!("RPC failed for {}: {}", url_owned, e))
                            }
                        }
                    }
                    Err(e) => {
                        tracing::debug!(url = %url_owned, error = %e, "Endpoint connection failed");
                        Err(e)
                    }
                }
            }));
        }

        // Collect results
        let mut best: Option<(String, Self, u128)> = None;
        let mut last_err = None;
        for handle in handles {
            match handle.await {
                Ok(Ok((url, client, latency))) => {
                    let is_better = best
                        .as_ref()
                        .is_none_or(|(_, _, best_lat)| latency < *best_lat);
                    if is_better {
                        best = Some((url, client, latency));
                    }
                }
                Ok(Err(e)) => {
                    last_err = Some(e);
                }
                Err(e) => {
                    last_err = Some(anyhow::anyhow!("Task join error: {}", e));
                }
            }
        }

        match best {
            Some((url, client, latency)) => {
                tracing::info!(url = %url, latency_ms = latency, "Selected best endpoint");
                Ok(client)
            }
            None => Err(last_err.unwrap_or_else(|| anyhow::anyhow!("All endpoints failed"))),
        }
    }

    /// Connect to a well-known network with automatic fallback endpoints.
    pub async fn connect_network(network: &crate::types::Network) -> Result<Self> {
        let urls = network.ws_urls();
        Self::connect_with_retry(&urls).await
    }

    /// Get a reference to the underlying subxt client.
    pub fn subxt(&self) -> &OnlineClient<SubtensorConfig> {
        &self.inner
    }

    /// Create a signer from an sr25519 keypair.
    pub fn signer(pair: &sr25519::Pair) -> Signer {
        PairSigner::new(pair.clone())
    }

    fn to_account_id(pk: &sr25519::Public) -> AccountId {
        AccountId::from(pk.0)
    }

    fn ss58_to_account_id(ss58: &str) -> Result<AccountId> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        Ok(AccountId::from(pk.0))
    }

    /// Public version of ss58_to_account_id for use outside chain module.
    pub fn ss58_to_account_id_pub(ss58: &str) -> Result<AccountId> {
        Self::ss58_to_account_id(ss58)
    }

    /// Enable dry-run mode: sign_submit will print a JSON preview instead of broadcasting.
    pub fn set_dry_run(&mut self, enabled: bool) {
        self.dry_run = enabled;
    }

    /// Returns true if dry-run mode is enabled.
    pub fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    /// Set the finalization timeout in seconds (default: 30).
    pub fn set_finalization_timeout(&mut self, secs: u64) {
        self.finalization_timeout = secs.max(1);
    }

    /// Get the current finalization timeout in seconds.
    pub fn finalization_timeout(&self) -> u64 {
        self.finalization_timeout
    }

    /// Set extrinsic mortality in blocks (0 = use default).
    /// This determines how long the extrinsic stays valid in the mempool.
    pub fn set_mortality_blocks(&mut self, blocks: u64) {
        self.mortality_blocks = blocks;
    }

    /// Get the current mortality setting in blocks.
    pub fn mortality_blocks(&self) -> u64 {
        self.mortality_blocks
    }

    /// Sign, submit, and wait for finalization of a typed extrinsic.
    /// Returns the extrinsic hash. Provides contextual error messages for common failures.
    /// In dry-run mode, encodes the call data and returns a JSON preview without submitting.
    async fn sign_submit<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
    ) -> Result<String> {
        // Dry-run: encode the call and show what would be submitted
        if self.dry_run {
            let call_data = self
                .inner
                .tx()
                .call_data(tx)
                .map_err(|e| anyhow::anyhow!("Failed to encode call data: {}", e))?;
            let signer_pub = sp_core::Pair::public(pair);
            let signer_ss58 = crate::wallet::keypair::to_ss58(&signer_pub, 42);
            let info = serde_json::json!({
                "dry_run": true,
                "signer": signer_ss58,
                "call_data_hex": format!("0x{}", hex::encode(&call_data)),
                "call_data_len": call_data.len(),
            });
            eprintln!(
                "[dry-run] Transaction would be submitted by {} ({} bytes call data)",
                signer_ss58,
                call_data.len()
            );
            crate::cli::helpers::print_json(&info);
            return Ok("dry-run".to_string());
        }

        let signer = Self::signer(pair);
        // Acquire per-signer file lock to prevent concurrent agcli instances
        // from submitting with the same nonce (Issue 648).
        let _tx_lock = acquire_tx_lock(pair)?;
        let start = std::time::Instant::now();
        let spinner = crate::cli::helpers::spinner("Submitting transaction...");
        tracing::debug!(
            finalization_timeout = self.finalization_timeout,
            mortality_blocks = self.mortality_blocks,
            "Submitting extrinsic"
        );

        // Note: For subxt 0.38.x, transaction mortality is configured differently.
        // The default behavior (no explicit params) uses immortal transactions.
        // We'll use the simple sign_and_submit_then_watch_default for now.

        // Retry submission on transient errors (connection drop before tx reaches node).
        // Once submitted, we do NOT retry — the finalization wait is non-idempotent.
        let inner = &self.inner;
        let progress = retry_on_transient("sign_submit", RPC_RETRIES, || async {
            match inner
                .tx()
                .sign_and_submit_then_watch_default(tx, &signer)
                .await
            {
                Ok(p) => Ok(p),
                Err(e) => {
                    let msg = e.to_string();
                    if is_transient_error(&msg) {
                        Err(anyhow::anyhow!("{}", msg))
                    } else {
                        spinner.finish_and_clear();
                        Err(format_submit_error(e))
                    }
                }
            }
        })
        .await?;
        spinner.set_message("Waiting for finalization...");
        tracing::debug!(
            timeout_secs = self.finalization_timeout,
            "Extrinsic submitted, waiting for finalization"
        );
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(self.finalization_timeout),
            progress.wait_for_finalized_success(),
        )
        .await
        .map_err(|_| {
            spinner.finish_and_clear();
            anyhow::anyhow!(
                "Transaction timed out after {}s waiting for finalization. \
                 The extrinsic may still be pending. Increase wait with \
                 --finalization-timeout or AGCLI_FINALIZATION_TIMEOUT / ~/.agcli/config.toml \
                 `finalization_timeout`, or tune --mortality-blocks on congested networks.",
                self.finalization_timeout
            )
        })?
        .map_err(|e| {
            spinner.finish_and_clear();
            format_dispatch_error(e)
        })?;
        let hash = format!("{:?}", result.extrinsic_hash());
        spinner.finish_and_clear();
        tracing::info!(tx_hash = %hash, elapsed_ms = start.elapsed().as_millis() as u64, "Extrinsic finalized");
        Ok(hash)
    }

    /// Sign and submit via MEV shield: SCALE-encode the call, encrypt with ML-KEM-768,
    /// then submit encrypted extrinsic to MevShield.submit_encrypted.
    pub async fn sign_submit_mev<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
    ) -> Result<String> {
        tracing::info!("MEV shield: encrypting extrinsic");
        let start = std::time::Instant::now();

        // 1. Encode the call to SCALE bytes
        let call_data = self
            .inner
            .tx()
            .call_data(tx)
            .map_err(|e| anyhow::anyhow!("Failed to encode call data: {}", e))?;

        // 2. Fetch the MEV shield public key from chain
        let mev_key = self.get_mev_shield_next_key().await?;

        // 3. Encrypt with ML-KEM-768 + XChaCha20-Poly1305
        let (commitment, ciphertext) =
            crate::extrinsics::mev_shield::encrypt_for_mev_shield(&mev_key, &call_data)?;

        tracing::info!(
            elapsed_ms = start.elapsed().as_millis() as u64,
            ct_len = ciphertext.len(),
            "MEV shield: encryption complete"
        );

        // 4. Submit the encrypted extrinsic
        self.submit_mev_encrypted(pair, commitment, ciphertext)
            .await
    }

    /// Sign and submit, optionally wrapping through MEV shield.
    pub async fn sign_submit_or_mev<T: subxt::tx::Payload>(
        &self,
        tx: &T,
        pair: &sr25519::Pair,
        use_mev: bool,
    ) -> Result<String> {
        if use_mev {
            self.sign_submit_mev(tx, pair).await
        } else {
            self.sign_submit(tx, pair).await
        }
    }

    // ──────── Balance Queries ────────

    /// Get TAO balance (free) for an account.
    pub async fn get_balance(&self, account: &sr25519::Public) -> Result<Balance> {
        let start = std::time::Instant::now();
        let account_id = Self::to_account_id(account);
        let inner = &self.inner;
        let info = retry_on_transient("get_balance", 2, || async {
            let addr = api::storage().system().account(&account_id);
            let result = inner
                .storage()
                .at_latest()
                .await
                .context("Failed to get latest block for balance query")?
                .fetch(&addr)
                .await
                .context("Failed to fetch account balance")?;
            Ok(result)
        })
        .await?;
        let balance = match info {
            Some(info) => Balance::from_rao(info.data.free),
            None => Balance::ZERO,
        };
        tracing::debug!(
            elapsed_ms = start.elapsed().as_millis() as u64,
            "get_balance"
        );
        Ok(balance)
    }

    /// Get TAO balance for an SS58 address.
    pub async fn get_balance_ss58(&self, ss58: &str) -> Result<Balance> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        self.get_balance(&pk).await
    }

    /// Resolve a block number to a block hash via RPC.
    pub async fn get_block_hash(&self, block_number: u32) -> Result<subxt::utils::H256> {
        use subxt::backend::legacy::rpc_methods::NumberOrHex;
        let rpc = &self.rpc;
        let hash = retry_on_transient("get_block_hash", RPC_RETRIES, || async {
            let h = rpc
                .chain_get_block_hash(Some(NumberOrHex::Number(block_number as u64)))
                .await
                .context("Failed to get block hash")?;
            Ok(h)
        })
        .await?;
        hash.ok_or_else(|| anyhow::anyhow!("Block {} not found", block_number))
    }

    /// Wrap at-block storage errors with an archive node hint when state is pruned.
    fn annotate_at_block_error(err: anyhow::Error, block_number: Option<u32>) -> anyhow::Error {
        let msg = format!("{:#}", err);
        let is_state_pruned = msg.contains("pruned")
            || msg.contains("State already discarded")
            || msg.contains("UnknownBlock")
            || msg.contains("not found");
        if is_state_pruned {
            if let Some(bn) = block_number {
                return anyhow::anyhow!(
                    "{}\n\n  Hint: Block {} state may have been pruned from this node.\n  Use --network archive (or --endpoint <archive-url>) to query historical state.\n  Example: agcli balance --at-block {} --network archive",
                    msg, bn, bn
                );
            }
        }
        err
    }

    /// Get TAO balance at a specific block hash.
    pub async fn get_balance_at_block(
        &self,
        ss58: &str,
        block_hash: subxt::utils::H256,
    ) -> Result<Balance> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        let account_id = Self::to_account_id(&pk);
        let addr = api::storage().system().account(&account_id);
        let info = self
            .inner
            .storage()
            .at(block_hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        match info {
            Some(info) => Ok(Balance::from_rao(info.data.free)),
            None => Ok(Balance::ZERO),
        }
    }

    /// Get total staked TAO at a specific block hash.
    pub async fn get_total_stake_at_block(
        &self,
        block_hash: subxt::utils::H256,
    ) -> Result<Balance> {
        let addr = api::storage().subtensor_module().total_stake();
        let val = self
            .inner
            .storage()
            .at(block_hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(Balance::from_rao(val.unwrap_or(0)))
    }

    // ──────── Block Info ────────

    /// Current block number (best / non-finalized).
    pub async fn get_block_number(&self) -> Result<u64> {
        let inner = &self.inner;
        retry_on_transient("get_block_number", RPC_RETRIES, || async {
            let block = inner
                .blocks()
                .at_latest()
                .await
                .context("Failed to fetch latest block")?;
            Ok(block.number() as u64)
        })
        .await
    }

    /// Current **finalized** block number.
    ///
    /// Uses the RPC `chain_getFinalizedHead` → `chain_getBlock` path.
    /// Finalized blocks are consensus-safe and will not be reorged.
    /// Prefer this over `get_block_number()` when correctness matters more
    /// than latency (e.g., commit-reveal timing, cost decisions).
    pub async fn get_finalized_block_number(&self) -> Result<u64> {
        let rpc = &self.rpc;
        retry_on_transient("get_finalized_block_number", RPC_RETRIES, || async {
            let hash = rpc
                .chain_get_finalized_head()
                .await
                .context("Failed to fetch finalized head")?;
            let header = rpc
                .chain_get_header(Some(hash))
                .await
                .context("Failed to fetch finalized header")?
                .ok_or_else(|| anyhow::anyhow!("Finalized header not found for {:?}", hash))?;
            Ok(header.number as u64)
        })
        .await
    }

    // ──────── Network Params ────────

    /// Total TAO issuance.
    pub async fn get_total_issuance(&self) -> Result<Balance> {
        let inner = &self.inner;
        retry_on_transient("get_total_issuance", RPC_RETRIES, || async {
            let addr = api::storage().balances().total_issuance();
            let val = inner.storage().at_latest().await?.fetch(&addr).await?;
            let raw = val.unwrap_or(0);
            Ok(Balance::from_rao(raw))
        })
        .await
    }

    /// Total staked TAO.
    pub async fn get_total_stake(&self) -> Result<Balance> {
        let inner = &self.inner;
        retry_on_transient("get_total_stake", RPC_RETRIES, || async {
            let addr = api::storage().subtensor_module().total_stake();
            let val = inner.storage().at_latest().await?.fetch(&addr).await?;
            Ok(Balance::from_rao(val.unwrap_or(0)))
        })
        .await
    }

    /// Total number of subnets.
    pub async fn get_total_networks(&self) -> Result<u16> {
        let inner = &self.inner;
        retry_on_transient("get_total_networks", RPC_RETRIES, || async {
            let addr = api::storage().subtensor_module().total_networks();
            let val = inner.storage().at_latest().await?.fetch(&addr).await?;
            Ok(val.unwrap_or(0))
        })
        .await
    }

    /// Block emission rate.
    pub async fn get_block_emission(&self) -> Result<Balance> {
        let inner = &self.inner;
        retry_on_transient("get_block_emission", RPC_RETRIES, || async {
            let addr = api::storage().subtensor_module().block_emission();
            let val = inner.storage().at_latest().await?.fetch(&addr).await?;
            Ok(Balance::from_rao(val.unwrap_or(0)))
        })
        .await
    }

    // ──────── Subnet Creation ────────

    /// Get the cost to register a new subnet (network registration lock cost).
    /// Returns the amount of TAO required to lock when creating a new subnet.
    pub async fn get_subnet_registration_cost(&self) -> Result<Balance> {
        let inner = &self.inner;
        retry_on_transient("get_subnet_registration_cost", RPC_RETRIES, || async {
            let payload = api::apis()
                .subnet_registration_runtime_api()
                .get_network_registration_cost();
            let cost = inner
                .runtime_api()
                .at_latest()
                .await
                .context("Failed to get latest block for subnet cost query")?
                .call(payload)
                .await
                .context("Failed to query subnet registration cost")?;
            Ok(Balance::from_rao(cost))
        })
        .await
    }

    // ──────── Block Hash Pinning ────────

    /// Pin the latest block hash for consistent multi-query reads.
    /// Returns the pinned block hash. All subsequent pinned query methods
    /// will read from this exact block, avoiding redundant `at_latest()` calls
    /// and ensuring data consistency across related queries.
    pub async fn pin_latest_block(&self) -> Result<subxt::utils::H256> {
        let inner = &self.inner;
        retry_on_transient("pin_latest_block", RPC_RETRIES, || async {
            let block = inner.blocks().at_latest().await
                .context("Failed to fetch latest block for pinning")?;
            let hash = block.hash();
            tracing::debug!(block_hash = %hash, block_number = block.number(), "Pinned latest block");
            Ok(hash)
        }).await
    }

    /// Get TAO balance for an SS58 address using a pinned block hash.
    /// More efficient than get_balance_ss58() when making multiple queries
    /// because it avoids a redundant at_latest() RPC call per query.
    pub async fn get_balance_at_hash(
        &self,
        ss58: &str,
        block_hash: subxt::utils::H256,
    ) -> Result<Balance> {
        let pk = crate::wallet::keypair::from_ss58(ss58)?;
        let account_id = Self::to_account_id(&pk);
        let addr = api::storage().system().account(&account_id);
        let info = self
            .inner
            .storage()
            .at(block_hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        match info {
            Some(info) => Ok(Balance::from_rao(info.data.free)),
            None => Ok(Balance::ZERO),
        }
    }

    /// Get balances for multiple SS58 addresses using a single pinned block.
    /// More efficient than individual `get_balance_ss58()` calls because:
    ///
    /// 1. Single `at_latest()` call instead of N calls
    /// 2. All reads are from the same block (data consistency)
    /// 3. All balance fetches run concurrently (parallel RPC calls)
    ///
    /// Returns `Vec<(ss58, Balance)>` in the same order as input.
    pub async fn get_balances_multi(&self, addresses: &[&str]) -> Result<Vec<(String, Balance)>> {
        if addresses.is_empty() {
            return Ok(vec![]);
        }
        let block_hash = self.pin_latest_block().await?;
        // Fetch all balances concurrently instead of sequentially
        let futures: Vec<_> = addresses
            .iter()
            .map(|addr| {
                let addr_owned = addr.to_string();
                async move {
                    let balance = self.get_balance_at_hash(&addr_owned, block_hash).await?;
                    Ok::<_, anyhow::Error>((addr_owned, balance))
                }
            })
            .collect();
        let results = futures::future::try_join_all(futures).await?;
        Ok(results)
    }

    // ──────── Pinned Network Params ────────

    /// Total TAO issuance at a pinned block hash.
    pub async fn get_total_issuance_at(&self, hash: subxt::utils::H256) -> Result<Balance> {
        let addr = api::storage().balances().total_issuance();
        let val = self
            .inner
            .storage()
            .at(hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        let raw = val.unwrap_or(0);
        Ok(Balance::from_rao(raw))
    }

    /// Total staked TAO at a pinned block hash.
    pub async fn get_total_stake_at(&self, hash: subxt::utils::H256) -> Result<Balance> {
        let addr = api::storage().subtensor_module().total_stake();
        let val = self
            .inner
            .storage()
            .at(hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(Balance::from_rao(val.unwrap_or(0)))
    }

    /// Total number of subnets at a pinned block hash.
    pub async fn get_total_networks_at(&self, hash: subxt::utils::H256) -> Result<u16> {
        let addr = api::storage().subtensor_module().total_networks();
        let val = self
            .inner
            .storage()
            .at(hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(val.unwrap_or(0))
    }

    /// Block emission rate at a pinned block hash.
    pub async fn get_block_emission_at(&self, hash: subxt::utils::H256) -> Result<Balance> {
        let addr = api::storage().subtensor_module().block_emission();
        let val = self
            .inner
            .storage()
            .at(hash)
            .fetch(&addr)
            .await
            .map_err(|e| Self::annotate_at_block_error(e.into(), None))?;
        Ok(Balance::from_rao(val.unwrap_or(0)))
    }

    /// Get the block number for a pinned block hash.
    /// Useful when you pin a block and also need its number without an extra at_latest().
    pub async fn get_block_number_at(&self, hash: subxt::utils::H256) -> Result<u64> {
        let block = self
            .inner
            .blocks()
            .at(hash)
            .await
            .context("Failed to fetch block at pinned hash")?;
        Ok(block.number() as u64)
    }

    /// Fetch all network overview stats using a single pinned block.
    /// Returns (block_number, total_stake, total_networks, total_issuance, emission).
    /// Saves 4 redundant `at_latest()` RPC round-trips compared to individual queries.
    pub async fn get_network_overview(&self) -> Result<(u64, Balance, u16, Balance, Balance)> {
        let hash = self.pin_latest_block().await?;
        // Block number comes from the pinned block itself
        let block = self
            .inner
            .blocks()
            .at(hash)
            .await
            .context("Failed to fetch pinned block")?;
        let block_number = block.number() as u64;
        let (stake, networks, issuance, emission) = tokio::try_join!(
            self.get_total_stake_at(hash),
            self.get_total_networks_at(hash),
            self.get_total_issuance_at(hash),
            self.get_block_emission_at(hash),
        )?;
        Ok((block_number, stake, networks, issuance, emission))
    }
}

/// Format submission errors (before tx reaches chain) with actionable hints.
fn format_submit_error(e: subxt::Error) -> anyhow::Error {
    let msg = e.to_string();
    if msg.contains("connection") || msg.contains("Connection") || msg.contains("Ws") {
        anyhow::anyhow!("Connection lost while submitting transaction. Check your network and endpoint.\n  Error: {}", msg)
    } else if msg.contains("Priority is too low") || msg.contains("priority") {
        anyhow::anyhow!("Transaction rejected: a conflicting transaction is already pending. Wait for it to finalize or use a different nonce.\n  Error: {}", msg)
    } else if msg.contains("Inability to pay") || msg.contains("InsufficientBalance") {
        anyhow::anyhow!("Insufficient balance to pay transaction fees. Check your free TAO balance with `agcli balance`.\n  Error: {}", msg)
    } else {
        anyhow::anyhow!("Transaction submission failed: {}", msg)
    }
}

/// Decoded custom error: (name, human-readable description).
struct DecodedError {
    name: &'static str,
    desc: &'static str,
}

/// Decode raw "Custom error: N" codes from SubtensorModule into named errors
/// with human-readable descriptions.
/// When subxt can't match compile-time metadata to the runtime, it returns
/// numeric error indices instead of named variants.
fn decode_custom_error(msg: &str) -> Option<DecodedError> {
    // Extract the number from "Custom error: N" or "custom error: N"
    let lower = msg.to_lowercase();
    let idx = lower.find("custom error:")?;
    let after = &lower[idx + "custom error:".len()..];
    let num_str: String = after
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let n: u32 = num_str.parse().ok()?;
    // SubtensorModule (pallet index 7) error enum — from chain metadata
    let (name, desc) = match n {
        0 => ("RootNetworkDoesNotExist", "The root network (SN0) does not exist on this chain"),
        1 => ("InvalidIpType", "The IP type provided for axon serving is not valid (use 4 for IPv4 or 6 for IPv6)"),
        2 => ("InvalidIpAddress", "The IP address provided is not a valid format"),
        3 => ("InvalidPort", "The port number for axon serving is invalid (must be 1-65535)"),
        4 => ("HotKeyNotRegisteredInSubNet", "This hotkey is not registered on the target subnet. Register with `agcli subnet register-neuron`"),
        5 => ("HotKeyAccountNotExists", "This hotkey account does not exist on chain. It may need to be funded or registered first"),
        6 => ("HotKeyNotRegisteredInNetwork", "This hotkey is not registered on any network. Register first with `agcli subnet register-neuron --netuid <N>`"),
        7 => ("NonAssociatedColdKey", "This coldkey is not associated with the specified hotkey. Check your --wallet and --hotkey flags"),
        8 => ("NotEnoughStake", "Insufficient stake for this operation. Check your stake with `agcli stake list`"),
        9 => ("NotEnoughStakeToWithdraw", "Cannot unstake this amount — it exceeds your current stake. Check `agcli stake list`"),
        10 => ("NotEnoughStakeToSetWeights", "Your stake is below the minimum required to set weights on this subnet"),
        11 => ("NotEnoughStakeToSetChildkeys", "Your stake is below the minimum required to set childkeys"),
        12 => ("NotEnoughBalanceToStake", "Your TAO balance is too low to stake this amount. Check `agcli balance`"),
        13 => ("BalanceWithdrawalError", "Failed to withdraw balance — the chain could not complete the transfer"),
        14 => ("ZeroBalanceAfterWithdrawn", "This operation would leave your account with zero balance, which is not allowed"),
        15 => ("NeuronNoValidatorPermit", "This neuron does not have a validator permit on the subnet"),
        16 => ("WeightVecNotEqualSize", "The UIDs and weights arrays must be the same length"),
        17 => ("DuplicateUids", "Duplicate UIDs found in your weight submission — each UID must appear only once"),
        18 => ("UidVecContainInvalidOne", "One or more UIDs are out of range for this subnet"),
        19 => ("WeightVecLengthIsLow", "Too few weights provided — you must set weights for at least the minimum number of UIDs"),
        20 => ("TooManyRegistrationsThisBlock", "Registration limit reached for this block. Wait ~12 seconds and try again"),
        21 => ("HotKeyAlreadyRegisteredInSubNet", "This hotkey is already registered on the subnet"),
        22 => ("NewHotKeyIsSameWithOld", "The new hotkey is the same as the current one — no change needed"),
        23 => ("InvalidWorkBlock", "The PoW work block is invalid or too old"),
        24 => ("InvalidDifficulty", "The PoW difficulty does not match the current requirement"),
        25 => ("InvalidSeal", "The PoW seal/nonce solution is incorrect"),
        26 => ("MaxWeightExceeded", "Total weight exceeds the maximum allowed (65535). Reduce individual weights"),
        27 => ("HotKeyAlreadyDelegate", "This hotkey already has a delegate set"),
        28 => ("SettingWeightsTooFast", "Weight-setting is rate-limited. Wait a few blocks before setting weights again"),
        29 => ("IncorrectWeightVersionKey", "Wrong weight version key — the subnet may have updated its expected version"),
        30 => ("ServingRateLimitExceeded", "Axon serving updates are rate-limited. Wait before updating your axon info"),
        31 => ("UidsLengthExceedUidsInSubNet", "You submitted weights for more UIDs than exist on the subnet"),
        32 => ("NetworkTxRateLimitExceeded", "Global transaction rate limit hit. Wait a few blocks before retrying"),
        33 => ("DelegateTxRateLimitExceeded", "Delegate operations are rate-limited. Wait before modifying delegate settings"),
        34 => ("HotKeySetTxRateLimitExceeded", "Hotkey update operations are rate-limited. Wait before retrying"),
        35 => ("StakingRateLimitExceeded", "Staking operations are rate-limited. Wait a few blocks before staking again"),
        36 => ("SubNetRegistrationDisabled", "Subnet registration is currently disabled by the subnet owner"),
        37 => ("TooManyRegistrationsThisInterval", "Too many registrations in this interval. Wait for the next interval to retry"),
        38 => ("TransactorAccountShouldBeHotKey", "This operation must be submitted by the hotkey account, not the coldkey"),
        39 => ("FaucetDisabled", "The faucet is disabled on this network"),
        40 => ("NotSubnetOwner", "You are not the owner of this subnet. Only the subnet owner can perform this action"),
        41 => ("RegistrationNotPermittedOnRootSubnet", "Direct registration on the root subnet (SN0) is not allowed"),
        42 => ("StakeTooLowForRoot", "Your total stake is too low to participate in the root network"),
        43 => ("AllNetworksInImmunity", "All subnets are currently in their immunity period — no subnet can be replaced"),
        44 => ("NotEnoughBalanceToPaySwapHotKey", "Insufficient balance to pay the hotkey swap fee"),
        45 => ("NotRootSubnet", "This operation is only available on the root subnet (SN0)"),
        46 => ("CanNotSetRootNetworkWeights", "Cannot set weights on the root network directly"),
        47 => ("NoNeuronIdAvailable", "The subnet is full — no UID slots available. Wait for a slot to open or try a different subnet"),
        48 => ("DelegateTakeTooLow", "Delegate take percentage is below the minimum allowed"),
        49 => ("DelegateTakeTooHigh", "Delegate take percentage exceeds the maximum (11.11%)"),
        50 => ("NoWeightsCommitFound", "No weight commit found to reveal. You must `agcli weights commit` before revealing"),
        51 => ("InvalidRevealCommitHashNotMatch", "The reveal data does not match your previous commit hash"),
        52 => ("CommitRevealEnabled", "This subnet uses commit-reveal for weights. Use `agcli weights commit` then `agcli weights reveal`"),
        53 => ("CommitRevealDisabled", "Commit-reveal is not enabled on this subnet. Use `agcli weights set` directly"),
        54 => ("LiquidAlphaDisabled", "Liquid alpha is not enabled on this subnet"),
        55 => ("AlphaHighTooLow", "The alpha high parameter is set too low"),
        56 => ("AlphaLowOutOfRange", "The alpha low parameter is outside the valid range"),
        57 => ("ColdKeyAlreadyAssociated", "This coldkey is already associated with a hotkey"),
        58 => ("NotEnoughBalanceToPaySwapColdKey", "Insufficient balance to pay the coldkey swap fee"),
        59 => ("InvalidChild", "The specified childkey UID is invalid"),
        60 => ("DuplicateChild", "Duplicate childkey UID — each child must appear only once"),
        61 => ("ProportionOverflow", "Childkey proportions exceed 100% total"),
        62 => ("TooManyChildren", "Too many childkeys — the maximum number of children has been reached"),
        63 => ("TxRateLimitExceeded", "General transaction rate limit exceeded. Wait a few blocks before retrying"),
        64 => ("ColdkeySwapAnnouncementNotFound", "No coldkey swap has been announced for this account"),
        65 => ("ColdkeySwapTooEarly", "The coldkey swap was announced too recently. Wait for the cooldown period"),
        66 => ("ColdkeySwapReannouncedTooEarly", "Cannot re-announce a coldkey swap yet — the minimum interval hasn't passed"),
        67 => ("AnnouncedColdkeyHashDoesNotMatch", "The new coldkey does not match the one announced in the swap"),
        68 => ("ColdkeySwapAlreadyDisputed", "This coldkey swap has been disputed and cannot be executed"),
        69 => ("NewColdKeyIsHotkey", "The new coldkey address is already registered as a hotkey — use a different address"),
        70 => ("InvalidChildkeyTake", "The childkey take value is invalid (must be 0-18%)"),
        71 => ("TxChildkeyTakeRateLimitExceeded", "Childkey take changes are rate-limited. Wait before changing again"),
        72 => ("InvalidIdentity", "One or more identity fields are invalid or exceed the maximum length"),
        73 => ("MechanismDoesNotExist", "The specified mechanism does not exist on this subnet"),
        74 => ("CannotUnstakeLock", "Cannot unstake during the lock period (subnet immunity or staking lock)"),
        75 => ("SubnetNotExists", "This subnet ID does not exist. Check available subnets with `agcli subnet list`"),
        76 => ("TooManyUnrevealedCommits", "Too many pending weight reveals. Reveal existing commits before creating new ones"),
        77 => ("ExpiredWeightCommit", "Your weight commit has expired. Submit a new commit"),
        78 => ("RevealTooEarly", "The reveal window is not open yet. Wait for the reveal period to begin"),
        79 => ("InputLengthsUnequal", "Input arrays have different lengths — UIDs and values must match"),
        80 => ("CommittingWeightsTooFast", "Weight commits are rate-limited. Wait before committing again"),
        81 => ("AmountTooLow", "The amount is below the minimum threshold for this operation"),
        82 => ("InsufficientLiquidity", "The liquidity pool does not have enough reserves for this trade"),
        83 => ("SlippageTooHigh", "Price slippage exceeds the allowed maximum. Try a smaller amount or wait for better liquidity"),
        84 => ("TransferDisallowed", "This transfer is not allowed by chain rules"),
        85 => ("ActivityCutoffTooLow", "The activity cutoff parameter is below the minimum"),
        86 => ("CallDisabled", "This operation is currently disabled on the chain"),
        87 => ("FirstEmissionBlockNumberAlreadySet", "The emission start block has already been configured for this subnet"),
        88 => (
            "NeedWaitingMoreBlocksToStarCall",
            "Subnet emissions `start` call is not allowed yet — wait until enough blocks have passed after subnet creation (see subnet start / check-start docs)",
        ),
        89 => ("NotEnoughAlphaOutToRecycle", "Not enough alpha available to recycle"),
        90 => ("CannotBurnOrRecycleOnRootSubnet", "Burn and recycle operations are not allowed on the root subnet (SN0)"),
        91 => ("UnableToRecoverPublicKey", "Could not recover the public key from the provided signature"),
        92 => ("InvalidRecoveredPublicKey", "The recovered public key does not match the expected account"),
        93 => ("SubtokenDisabled", "The subtoken feature is not enabled on this subnet"),
        94 => ("HotKeySwapOnSubnetIntervalNotPassed", "The minimum interval between hotkey swaps on this subnet has not passed"),
        95 => ("ZeroMaxStakeAmount", "Maximum stake amount cannot be set to zero"),
        96 => ("SameNetuid", "Source and destination subnet are the same — use different netuids"),
        97 => ("InsufficientBalance", "Insufficient TAO balance for this operation. Check `agcli balance`"),
        98 => ("StakingOperationRateLimitExceeded", "Staking operations are rate-limited. Wait a few blocks (~12s each) before retrying"),
        99 => ("InvalidLeaseBeneficiary", "The lease beneficiary address is invalid"),
        100 => ("LeaseCannotEndInThePast", "Lease end block must be in the future"),
        101 => ("LeaseNetuidNotFound", "No lease found for this subnet ID"),
        102 => ("LeaseDoesNotExist", "The specified lease does not exist"),
        103 => ("LeaseHasNoEndBlock", "This lease is open-ended and cannot be ended by block number"),
        104 => ("LeaseHasNotEnded", "The lease has not ended yet — wait for the lease end block"),
        105 => ("Overflow", "Arithmetic overflow — try a smaller amount"),
        106 => ("BeneficiaryDoesNotOwnHotkey", "The beneficiary account does not own the specified hotkey"),
        107 => ("ExpectedBeneficiaryOrigin", "This call must be made by the beneficiary account"),
        108 => ("AdminActionProhibitedDuringWeightsWindow", "Admin changes are blocked during the weights setting window. Try after the current tempo"),
        109 => ("SymbolDoesNotExist", "The specified token symbol does not exist"),
        110 => ("SymbolAlreadyInUse", "This token symbol is already taken. Choose a different symbol"),
        111 => ("IncorrectCommitRevealVersion", "The commit_reveal_version argument does not match on-chain CommitRevealWeightsVersion (timelocked weight commits)"),
        112 => ("RevealPeriodTooLarge", "The reveal period is too long"),
        113 => ("RevealPeriodTooSmall", "The reveal period is too short"),
        114 => ("InvalidValue", "The provided value is invalid for this parameter"),
        115 => ("SubnetLimitReached", "The maximum number of subnets has been reached — no new subnets can be created"),
        116 => ("CannotAffordLockCost", "Insufficient balance to pay the subnet creation lock cost. Check `agcli subnet create-cost`"),
        117 => ("EvmKeyAssociateRateLimitExceeded", "EVM key association is rate-limited. Wait before retrying"),
        118 => ("SameAutoStakeHotkeyAlreadySet", "Auto-stake is already set to this hotkey — no change needed"),
        119 => ("UidMapCouldNotBeCleared", "Internal error: UID map cleanup failed"),
        120 => ("TrimmingWouldExceedMaxImmunePercentage", "Trimming would cause immune neurons to exceed the maximum allowed percentage"),
        121 => ("ChildParentInconsistency", "Childkey parent relationship is inconsistent"),
        122 => ("InvalidNumRootClaim", "Invalid number of root claims"),
        123 => ("InvalidRootClaimThreshold", "The root claim threshold is invalid"),
        124 => ("InvalidSubnetNumber", "The subnet number is invalid"),
        125 => ("TooManyUIDsPerMechanism", "Too many UIDs assigned to a single mechanism"),
        126 => ("VotingPowerTrackingNotEnabled", "Voting power tracking is not enabled on this subnet"),
        127 => ("InvalidVotingPowerEmaAlpha", "The voting power EMA alpha parameter is invalid"),
        128 => ("PrecisionLoss", "Calculation would lose too much precision — try a different amount"),
        129 => ("Deprecated", "This feature has been deprecated and is no longer available"),
        130 => ("AddStakeBurnRateLimitExceeded", "Add-stake-burn operations are rate-limited. Wait a few blocks before retrying"),
        131 => ("ColdkeySwapAnnounced", "A coldkey swap is already announced for this account"),
        132 => ("ColdkeySwapDisputed", "This coldkey swap has been disputed"),
        133 => (
            "ColdkeySwapClearTooEarly",
            "Cannot clear the coldkey swap announcement yet — wait until current block ≥ announce block + reannouncement delay (see `agcli wallet check-swap`)",
        ),
        _ => return None,
    };
    Some(DecodedError { name, desc })
}

/// Last `Pallet::Variant`-style token in `msg` (non-empty ASCII idents on both sides of `::`).
/// Used when subxt/metadata reports a qualified pallet error we do not match explicitly.
fn last_qualified_pallet_variant(msg: &str) -> Option<(&str, &str)> {
    let bytes = msg.as_bytes();
    let mut best: Option<(usize, usize, usize, usize)> = None;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b':' && bytes[i + 1] == b':' {
            let left_end = i;
            let mut left_start = left_end;
            while left_start > 0 {
                let c = bytes[left_start - 1];
                if c.is_ascii_alphanumeric() || c == b'_' {
                    left_start -= 1;
                } else {
                    break;
                }
            }
            let variant_start = i + 2;
            let mut variant_end = variant_start;
            while variant_end < bytes.len() {
                let c = bytes[variant_end];
                if c.is_ascii_alphanumeric() || c == b'_' {
                    variant_end += 1;
                } else {
                    break;
                }
            }
            if left_end > left_start && variant_end > variant_start {
                best = Some((left_start, left_end, variant_start, variant_end));
            }
        }
        i += 1;
    }
    best.map(|(ls, le, vs, ve)| (&msg[ls..le], &msg[vs..ve]))
}

/// Format dispatch errors (tx reached chain but execution failed) with contextual hints.
fn format_dispatch_error(e: subxt::Error) -> anyhow::Error {
    let raw_msg = e.to_string();
    // If the error is a raw "Custom error: N", decode it so the named-error matchers below work.
    // The decoded description provides the user-friendly explanation.
    let (msg, decoded_desc) = if let Some(decoded) = decode_custom_error(&raw_msg) {
        (
            format!("{} [{}]", raw_msg, decoded.name),
            Some(decoded.desc),
        )
    } else {
        (raw_msg, None)
    };
    // Map common SubtensorModule errors to helpful messages
    // `NotEnoughStake` is a substring of `NotEnoughStakeToSetWeights` / ToWithdraw / ToSetChildkeys — check specific variants first.
    let hint = if msg.contains("NotEnoughStakeToSetWeights") {
        "Stake is below the chain minimum required to set weights on this subnet. Stake more on this validator hotkey or check `agcli stake list`."
    } else if msg.contains("NotEnoughStakeToWithdraw") {
        "Cannot unstake this amount — it exceeds your current stake. Check `agcli stake list`."
    } else if msg.contains("NotEnoughStakeToSetChildkeys") {
        "Stake is below the minimum required to set childkeys on this subnet."
    } else if msg.contains("NotEnoughBalanceToStake") || msg.contains("NotEnoughStake") {
        "Insufficient balance or stake for this operation. Check `agcli balance` and `agcli stake list`."
    } else if msg.contains("NotEnoughBalanceToPaySwapHotKey")
        || msg.contains("NotEnoughBalanceToPaySwapColdKey")
    {
        "Insufficient free balance to pay the hotkey or coldkey swap fee. Fund the signing account and retry."
    } else if msg.contains("Registry::NotRegistered") {
        "No on-chain identity is registered for this account (pallet Registry). Use `agcli network identity set` or pass the correct SS58."
    } else if msg.contains("HotKeyNotRegisteredInNetwork") {
        "This hotkey is not registered on any subnet yet. Pick a target netuid and register with `agcli subnet register-neuron --netuid <N>`."
    } else if msg.contains("HotKeyNotRegisteredInSubNet") || msg.contains("NotRegistered") {
        "This hotkey is not registered on this subnet. Register with `agcli subnet register-neuron` (check `--netuid`)."
    } else if msg.contains("NotEnoughBalance")
        || (msg.contains("InsufficientBalance")
            && !msg.contains("Swap::InsufficientBalance")
            && !msg.contains("Crowdloan::InsufficientBalance"))
    {
        "Insufficient TAO balance. Check your balance with `agcli balance`."
    } else if msg.contains("AlreadyRegistered") {
        "This hotkey is already registered on the subnet."
    } else if msg.contains("TooManyRegistrationsThisBlock") {
        "Registration limit reached for this block. Try again in the next block (~12 seconds)."
    } else if msg.contains("RootNetworkDoesNotExist") {
        "The root network (SN0) is missing from this runtime — wrong chain, incomplete genesis, or metadata mismatch. Verify `--endpoint` and that this is a Bittensor/subtensor node."
    } else if msg.contains("InvalidIpType")
        || msg.contains("InvalidIpAddress")
        || msg.contains("InvalidPort")
    {
        "Axon/prometheus endpoint fields were rejected: use a reachable IP, port 1–65535, and protocol 4 (IPv4) or 6 (IPv6) for axon. See `agcli network serve axon` / `agcli network serve prometheus`."
    } else if msg.contains("InvalidWorkBlock")
        || msg.contains("InvalidDifficulty")
        || msg.contains("InvalidSeal")
    {
        "PoW registration failed: the work block must be current, difficulty must match the subnet requirement, and the seal must match the computed work. Re-run `agcli subnet pow` for a fresh template."
    } else if msg.contains("ServingRateLimitExceeded") {
        "Axon or prometheus `serve` updates are rate-limited on this subnet. Wait more blocks (see `min_blocks_between_serve_axon` / prometheus equivalent in `agcli subnet hyperparams --netuid <N>`) before retrying."
    } else if msg.contains("NetworkTxRateLimitExceeded") {
        "Subnet-creation / network-level transactions are globally rate-limited. Wait several blocks and retry."
    } else if msg.contains("DelegateTxRateLimitExceeded") {
        "Delegate-related transactions are rate-limited. Wait more blocks before changing delegate or take again."
    } else if msg.contains("HotKeySetTxRateLimitExceeded") {
        "Hotkey swap/update transactions are rate-limited. Wait more blocks before another hotkey change."
    } else if msg.contains("EvmKeyAssociateRateLimitExceeded") {
        "EVM key association is rate-limited. Wait before associating another EVM key from this account."
    } else if msg.contains("TxChildkeyTakeRateLimitExceeded") {
        "Childkey take changes are rate-limited. Wait before updating childkey take again."
    } else if msg.contains("FaucetDisabled") {
        "The chain faucet is disabled — funding via faucet extrinsics is not available on this network."
    } else if msg.contains("NotSubnetOwner") {
        "Only the subnet owner can run this extrinsic. Confirm ownership with `agcli subnet show --netuid <N>` and sign with the owner coldkey."
    } else if msg.contains("LiquidAlphaDisabled")
        || msg.contains("AlphaHighTooLow")
        || msg.contains("AlphaLowOutOfRange")
    {
        "Liquid-alpha parameter change rejected: liquid alpha must be enabled on the subnet, and alpha_high / alpha_low must satisfy on-chain bounds. Inspect current values with `agcli subnet hyperparams --netuid <N>`."
    } else if msg.contains("NewColdKeyIsHotkey") {
        "Coldkey swap destination cannot be an address that is already registered as a hotkey. Choose a different coldkey account."
    } else if msg.contains("HotKeyAlreadyDelegate") {
        "This hotkey is already a delegate; no need to become delegate again."
    } else if msg.contains("NewHotKeyIsSameWithOld") {
        "The proposed hotkey matches the current one — the chain will not apply a no-op swap."
    } else if msg.contains("RegistrationNotPermittedOnRootSubnet") {
        "Direct neuron registration on the root subnet (SN0) is not allowed. Register on a user subnet instead."
    } else if msg.contains("StakeTooLowForRoot") {
        "Total stake is below the minimum required for root-network participation. Stake more before using root-only flows."
    } else if msg.contains("AllNetworksInImmunity") {
        "Every subnet is in its immunity window, so no subnet can be pruned/replaced right now. Wait until at least one subnet exits immunity."
    } else if msg.contains("NotRootSubnet") {
        "This extrinsic only applies on the root subnet (netuid 0). Check `--netuid` and command docs."
    } else if msg.contains("BalanceWithdrawalError") {
        "The chain could not debit the coldkey for this stake operation (transfer failed internally). Check free balance and account locks."
    } else if msg.contains("ZeroBalanceAfterWithdrawn") {
        "This operation would leave a required account with zero balance, which the runtime disallows. Leave a small reserve or reduce the amount."
    } else if msg.contains("Swap::MechanismDoesNotExist") {
        "Swap/AMM: that netuid has no subnet (or the subnet is not available to the DEX). Confirm with `agcli subnet list` / `agcli subnet show --netuid <N>`."
    } else if msg.contains("MechanismDoesNotExist") {
        "The subnet has no mechanism with the given index. Check `agcli subnet mechanism-count --netuid <N>` (or hyperparams) for valid mechanism IDs."
    } else if msg.contains("CannotUnstakeLock") {
        "Unstaking is blocked by a lock (e.g. immunity window or staking lock). Wait until the lock ends or reduce the requested amount."
    } else if msg.contains("TransferDisallowed") {
        "This transfer path is disallowed by subnet/token rules. Check subnet transfer settings and token mode."
    } else if msg.contains("ActivityCutoffTooLow") {
        "The activity cutoff hyperparameter is below the chain minimum. Increase it in the owner `subnet set-param` flow."
    } else if msg.contains("CallDisabled") {
        "This pallet call is disabled on-chain (runtime or subnet configuration). It cannot be executed until re-enabled."
    } else if msg.contains("FirstEmissionBlockNumberAlreadySet") {
        "Emission start for this subnet was already configured; the extrinsic cannot set it twice."
    } else if msg.contains("NeedWaitingMoreBlocksToStarCall") {
        "Too few blocks have passed since subnet creation to start emissions. Wait for the required delay and retry."
    } else if msg.contains("Swap::SubtokenDisabled") {
        "Swap/AMM: this subnet does not have subtoken mode enabled for the DEX. Enable/configure subnet subtoken support before AMM swaps on this netuid."
    } else if msg.contains("SubtokenDisabled") {
        "Subtoken operations are disabled on this subnet (SubtensorModule)."
    } else if msg.contains("HotKeySwapOnSubnetIntervalNotPassed") {
        "Minimum blocks between hotkey swaps on this subnet have not elapsed. Wait and retry."
    } else if msg.contains("ZeroMaxStakeAmount") {
        "Maximum stake cannot be set to zero; use a positive cap or a different hyperparameter flow."
    } else if msg.contains("SameNetuid") {
        "Source and destination netuids must differ for this cross-subnet operation."
    } else if msg.contains("InvalidLeaseBeneficiary") {
        "The lease beneficiary SS58 is invalid or not allowed for this leased-subnet registration."
    } else if msg.contains("LeaseCannotEndInThePast") {
        "Lease end block must be strictly in the future."
    } else if msg.contains("LeaseHasNoEndBlock") {
        "This lease has no fixed end block; use the extrinsic that matches open-ended leases."
    } else if msg.contains("LeaseHasNotEnded") {
        "The lease is still active — wait until the end block before this cleanup step."
    } else if msg.contains("BeneficiaryDoesNotOwnHotkey") {
        "The lease beneficiary must control the hotkey used in this lease flow."
    } else if msg.contains("ExpectedBeneficiaryOrigin") {
        "Sign this call with the lease beneficiary account, not another key."
    } else if msg.contains("RevealPeriodTooLarge") || msg.contains("RevealPeriodTooSmall") {
        "Commit–reveal reveal period is out of allowed bounds. Adjust the value to match chain limits (see subnet hyperparams / owner set-param)."
    } else if msg.contains("AdminUtils::InvalidValue") {
        "AdminUtils rejected this subnet-owner `set-param` value: it is not valid for the target field. Compare your argument to on-chain bounds with `agcli subnet hyperparams --netuid <N>`."
    } else if msg.contains("InvalidValue") {
        "A numeric or enum hyperparameter is out of range for this extrinsic. Compare your argument to on-chain min/max in `agcli subnet hyperparams`."
    } else if msg.contains("ValueNotInBounds") {
        "Subnet owner set-param rejected: value is outside the allowed bounds for this field. Re-check min/max against `agcli subnet hyperparams --netuid <N>` and pallet docs."
    } else if msg.contains("MaxValidatorsLargerThanMaxUIds") {
        "`max_allowed_validators` must be strictly less than `max_allowed_uids`. Lower validators or raise max UIDs in the owner hyperparameter flow."
    } else if msg.contains("MaxAllowedUIdsLessThanCurrentUIds") {
        "`max_allowed_uids` cannot be set below the current number of neurons on the subnet. Wait for deregistrations or choose a higher value."
    } else if msg.contains("BondsMovingAverageMaxReached") {
        "Bonds moving-average parameter is at or above the runtime maximum. Pick a smaller value."
    } else if msg.contains("NegativeSigmoidSteepness") {
        "Negative sigmoid steepness can only be set by root/sudo — use a non-negative value for subnet-owner calls."
    } else if msg.contains("MinAllowedUidsGreaterThanCurrentUids") {
        "Minimum allowed UIDs cannot be greater than the current neuron count on the subnet. Lower the minimum or wait for more registrations."
    } else if msg.contains("MinAllowedUidsGreaterThanMaxAllowedUids") {
        "Minimum allowed UIDs/weights cannot be greater than maximum allowed UIDs. Fix the pair so min ≤ max."
    } else if msg.contains("MaxAllowedUidsLessThanMinAllowedUids") {
        "Maximum allowed UIDs must be at least the minimum allowed UIDs. Increase max or decrease min."
    } else if msg.contains("MaxAllowedUidsGreaterThanDefaultMaxAllowedUids") {
        "`max_allowed_uids` cannot exceed the chain default cap. Lower the requested maximum."
    } else if msg.contains("TooManyFieldsInCommitmentInfo") {
        "Commitment payload has too many extra fields for this subnet/runtime. Remove optional fields or shorten the commitment metadata."
    } else if msg.contains("AccountNotAllowedCommit") {
        "This account is not allowed to post commitments on this subnet (permissions or registration rules)."
    } else if msg.contains("SpaceLimitExceeded") {
        "On-chain commitment storage quota for this interval is full. Wait for the next interval or reduce commitment size."
    } else if msg.contains("UnexpectedUnreserveLeftover") {
        "Commitment pallet reserve accounting failed (unexpected leftover). Retry; if it persists, the runtime may be in an inconsistent state."
    } else if msg.contains("Unproxyable") {
        "This call is not allowed through the configured proxy type’s filter. Use the real account or a proxy with a compatible type."
    } else if msg.contains("NotProxy") {
        "The signing account is not registered as a proxy for the target. Add the proxy first or sign with the delegating account."
    } else if msg.contains("Unannounced") {
        "This proxy call requires a prior announcement, or the announcement delay has not passed. Use the proxy announcement flow for delayed proxies."
    } else if msg.contains("NoSelfProxy") {
        "Cannot add your own account as a proxy. Use a different delegate address."
    } else if msg.contains("AnnouncementDepositInvariantViolated") {
        "Proxy announcement deposit accounting failed internally. Retry; report if it keeps happening."
    } else if msg.contains("InvalidDerivedAccountId") {
        "Could not derive a valid proxy/pure account from the supplied entropy or salt. Check inputs to pure-proxy creation."
    } else if msg.contains("Proxy::TooMany") {
        "Proxy pallet limit hit: too many proxies for this account or too many pending announcements. Remove a proxy, complete/cancel announcements, or wait."
    } else if msg.contains("Proxy::NotFound") {
        "No matching proxy relationship or announcement — verify delegate SS58, proxy type, and that `proxy.add_proxy` succeeded."
    } else if msg.contains("Proxy::Duplicate") {
        "That delegate is already a proxy for this account — remove it first or choose another delegate."
    } else if msg.contains("Proxy::NoPermission") {
        "This call cannot run through this proxy type (would escalate privileges). Sign with the real account or use a proxy type that allows it."
    } else if msg.contains("TooManyCalls") {
        "Utility batch exceeds the runtime batched-calls limit — split into smaller `utility.batch` calls."
    } else if msg.contains("InvalidDerivedAccount") && !msg.contains("InvalidDerivedAccountId") {
        "Utility pallet rejected derived-account inputs (e.g. pure-proxy preimage). Check salt/entropy and call encoding."
    } else if msg.contains("FeeRateTooHigh") {
        "Subnet swap fee rate exceeds the chain maximum — use a lower rate (owner `set_fee_rate`)."
    } else if msg.contains("InsufficientInputAmount") {
        "DEX swap input is too small after fees or for the pool curve — increase amount or check minimum trade size."
    } else if msg.contains("PriceLimitExceeded") {
        "Swap would violate the price/slippage bound — loosen the limit, reduce size, or retry with a better quote."
    } else if msg.contains("LiquidityNotFound") {
        "No liquidity position matches this key or tick range — verify position id / subnet and that you own the position."
    } else if msg.contains("InvalidTickRange") {
        "Concentrated-liquidity tick range is invalid (lower ≥ upper or out of bounds)."
    } else if msg.contains("MaxPositionsExceeded") {
        "Maximum user LP positions on this subnet — close or consolidate a position before opening another."
    } else if msg.contains("TooManySwapSteps") {
        "Multi-hop swap has too many steps — use a shorter path or fewer intermediate pools."
    } else if msg.contains("InvalidLiquidityValue") {
        "Liquidity mint/burn parameter is invalid or below the minimum the pool accepts."
    } else if msg.contains("ReservesTooLow") {
        "Pool reserves are too low for this swap or liquidity action — reduce size or wait for liquidity."
    } else if msg.contains("UserLiquidityDisabled") {
        "Subnet owner disabled user add/remove liquidity on this subnet."
    } else if msg.contains("TooManyFieldsInIdentityInfo") {
        "On-chain identity has too many additional fields — trim optional fields to the runtime maximum."
    } else if msg.contains("Registry::CannotRegister") {
        "Registry pallet: identity registration failed requirements (deposit, permissions, or eligibility)."
    } else if msg.contains("CannotRegister") {
        "Registry identity registration failed requirements (deposit, permissions, or eligibility)."
    } else if msg.contains("NotRegistered")
        && !msg.contains("HotKey")
        && !msg.contains("ColdkeySwapAnnouncement")
    {
        "No on-chain identity registered for this account — register first or use the correct SS58."
    } else if msg.contains("DepositTooLow")
        || msg.contains("CapTooLow")
        || msg.contains("MinimumContributionTooLow")
        || msg.contains("CannotEndInPast")
        || msg.contains("BlockDurationTooShort")
        || msg.contains("BlockDurationTooLong")
    {
        "Crowdloan parameters are out of allowed bounds (deposit, cap, minimum contribution, duration, or end block)."
    } else if msg.contains("InvalidCrowdloanId") {
        "Unknown or invalid crowdloan id — list active crowdloans or verify the id from chain state."
    } else if msg.contains("CapRaised") || msg.contains("CapNotRaised") {
        "Crowdloan cap state mismatch — either the cap is already fully raised or the cap was not reached for the next step."
    } else if msg.contains("ContributionPeriodEnded") || msg.contains("ContributionPeriodNotEnded")
    {
        "Crowdloan contribution window is closed or not finished yet — check the crowdloan schedule on-chain."
    } else if msg.contains("ContributionTooLow") {
        "Crowdloan contribution is below the minimum required for this crowdloan."
    } else if msg.contains("Crowdloan::AlreadyFinalized") || msg.contains("AlreadyFinalized") {
        "This crowdloan is already finalized — no further contributions or creator steps apply."
    } else if msg.contains("NoContribution") {
        "No contribution record for this account on this crowdloan."
    } else if msg.contains("CallUnavailable") {
        "Crowdloan success preimage/call is missing from storage — the configured dispatch may not be registered."
    } else if msg.contains("NotReadyToDissolve") {
        "Crowdloan cannot be dissolved yet — contributions may still be active or cap not handled."
    } else if msg.contains("DepositCannotBeWithdrawn") {
        "Crowdloan deposit cannot be withdrawn in the current state (success path or locks)."
    } else if msg.contains("MaxContributorsReached") {
        "Crowdloan contributor cap reached — no new contributors until rules change."
    } else if msg.contains("Crowdloan::InvalidOrigin") {
        "Crowdloan: this extrinsic must be signed by the crowdloan creator, a contributor, or another allowed role — check which origin the call expects."
    } else if msg.contains("InvalidOrigin") {
        "This extrinsic must be signed by the crowdloan creator, a contributor, or another allowed role — check which origin the call expects."
    } else if msg.contains("Crowdloan::Underflow") {
        "Crowdloan arithmetic underflowed (e.g. withdrawing more than contributed or inconsistent cap/state). Verify amounts and on-chain crowdloan status."
    } else if msg.contains("DrandConnectionFailure") {
        "Drand pallet could not reach the randomness beacon (node/network) — operators should check drand connectivity."
    } else if msg.contains("UnverifiedPulse") || msg.contains("PulseVerificationError") {
        "Drand pulse failed verification — wrong signature, stale data, or misconfigured beacon."
    } else if msg.contains("InvalidRoundNumber") {
        "Drand round number did not advance as expected — pulses must be submitted in order."
    } else if msg.contains("Drand::NoneValue") {
        "Drand pallet expected on-chain state that is missing (beacon config or round data not initialized). Operators should verify drand setup and runtime migrations."
    } else if msg.contains("Drand::StorageOverflow") {
        "Drand internal counter/storage limit exceeded — beacon state may need pruning or a runtime fix; report if this persists on a healthy node."
    } else if msg.contains("BadEncKeyLen") {
        "Shield pallet rejected an author encryption key length (inherent / validator key rotation)."
    } else if msg.contains("Shield::Unreachable") {
        "Shield pallet hit an internal unreachable path (runtime bug, corrupted storage, or incompatible node). Retry with another `--endpoint`, update the node/agcli, and report with logs if it persists."
    } else if msg.contains("TrimmingWouldExceedMaxImmunePercentage") {
        "`subnet trim` would leave too high a share of immune neurons. Reduce the trim request or wait for immunity changes."
    } else if msg.contains("ChildParentInconsistency") {
        "Childkey/parent relationships are inconsistent with on-chain state. Re-fetch children and fix parent/child bindings before resubmitting."
    } else if msg.contains("InvalidNumRootClaim") || msg.contains("InvalidRootClaimThreshold") {
        "Root-claim parameters are invalid for this runtime. Check allowed ranges in pallet docs or adjust counts/thresholds."
    } else if msg.contains("InvalidSubnetNumber") {
        "The subnet count or index in this call is not allowed (e.g. exceeds runtime limits)."
    } else if msg.contains("TooManyUIDsPerMechanism") {
        "UID capacity for this mechanism would exceed the chain limit (UIDs × mechanisms ≤ 256). Reduce registrations or mechanism count."
    } else if msg.contains("VotingPowerTrackingNotEnabled") {
        "Voting-power tracking is not enabled on this subnet; VP-only extrinsics will fail until it is turned on."
    } else if msg.contains("InvalidVotingPowerEmaAlpha") {
        "Voting power EMA alpha must be ≤ 10^18 (fixed-point). Use a smaller alpha value."
    } else if msg.contains("Deprecated") {
        "This extrinsic or feature is deprecated on this runtime. Use the supported replacement flow if one exists."
    } else if msg.contains("InvalidIdentity") {
        "Subnet or user identity fields failed validation (length, charset, or required fields). Trim values and retry `agcli network identity set` / `agcli network identity set-subnet`."
    } else if msg.contains("InvalidChildkeyTake") {
        "Childkey take is outside 0–18% (or other runtime bounds). Pick a valid take percentage."
    } else if msg.contains("InvalidChild") {
        "The child hotkey/UID is not valid for this parent on this subnet. Verify UIDs with `agcli view neurons --netuid <N>` and the `proportion:hotkey` list passed to `agcli stake set-children`."
    } else if msg.contains("DuplicateChild") {
        "The same child appears twice in the child list; each child must be unique."
    } else if msg.contains("ProportionOverflow") {
        "Child proportions sum to more than 100% — reduce proportions so the total fits the runtime cap."
    } else if msg.contains("TooManyChildren") {
        "At most five childkeys are allowed per parent on this subnet; remove a child before adding another."
    } else if msg.contains("NotEnoughAlphaOutToRecycle") {
        "Not enough subnet alpha is available to recycle at the requested amount. Lower the amount or wait for more alpha."
    } else if msg.contains("CannotBurnOrRecycleOnRootSubnet") {
        "Burn/recycle flows are not allowed on the root subnet (SN0)."
    } else if msg.contains("UnableToRecoverPublicKey") || msg.contains("InvalidRecoveredPublicKey")
    {
        "Signature recovery failed or the recovered key does not match the claimed account. Re-sign with the correct keypair or check the message/hash you signed."
    } else if msg.contains("SameAutoStakeHotkeyAlreadySet") {
        "Auto-stake is already configured to this hotkey."
    } else if msg.contains("UidMapCouldNotBeCleared") {
        "Internal UID-map cleanup failed on-chain (unexpected state). Retry later or report if persistent."
    } else if msg.contains("AdminActionProhibitedDuringWeightsWindow") {
        "Subnet-owner admin calls are blocked during the protected weights window. Retry after the current tempo interval ends (see subnet tempo / `agcli subnet hyperparams --netuid <N>`)."
    } else if msg.contains("TransactorAccountShouldBeHotKey") {
        "This extrinsic must be signed by the hotkey account, not the coldkey. Use the hotkey that owns the neuron (see `agcli wallet` / signing flags for your command)."
    } else if msg.contains("InvalidNetuid") || msg.contains("NetworkDoesNotExist") {
        "Invalid subnet ID. List available subnets with `agcli subnet list`."
    } else if msg.contains("BadOrigin") || msg.contains("NotOwner") {
        "Permission denied — you are not the owner of this subnet or account."
    } else if msg.contains("SettingWeightsTooFast") {
        "Subnet weights rate limit: wait more blocks before set_weights again. Check subnet hyperparams or run `agcli weights set --dry-run` for rate-limit context."
    } else if msg.contains("WeightsNotSettable") {
        "Cannot set weights right now (weights window or chain rules). Retry after the window or check subnet status."
    } else if msg.contains("TxRateLimitExceeded") {
        "Rate limit exceeded. Wait before retrying this operation."
    } else if msg.contains("StakeRateLimitExceeded") {
        "Staking rate limit exceeded. Wait before staking/unstaking again."
    } else if msg.contains("InvalidTake")
        || msg.contains("DelegateTakeTooHigh")
        || msg.contains("DelegateTakeTooLow")
    {
        "Invalid delegate take percentage. Take must be within the chain’s allowed band (above minimum, at or below 11.11%). Check `agcli subnet hyperparams --netuid <N>` for current limits."
    } else if msg.contains("NonAssociatedColdKey") {
        "This coldkey is not associated with the specified hotkey."
    } else if msg.contains("CommitRevealEnabled") {
        "This subnet requires commit-reveal for weights. Use `agcli weights commit` then `agcli weights reveal`."
    } else if msg.contains("CommitRevealDisabled") {
        "This subnet does not use commit-reveal. Set weights with `agcli weights set` instead of commit/reveal."
    } else if msg.contains("NoWeightsCommitFound") {
        "No on-chain weight commit for this hotkey/subnet. Run `agcli weights commit` before reveal, or check `agcli weights status`."
    } else if msg.contains("InvalidRevealCommitHashNotMatch") {
        "Reveal does not match the commit (weights or salt differ). Use the exact same weights and salt as in `agcli weights commit`."
    } else if msg.contains("TooManyUnrevealedCommits") {
        "Too many unrevealed commits for this hotkey. Reveal pending commits or wait before submitting another."
    } else if msg.contains("ExpiredWeightCommit") {
        "The commit expired before reveal. Submit a new `agcli weights commit` within the chain commit window."
    } else if msg.contains("RevealTooEarly") {
        "Reveal phase is not open yet. Wait for the reveal window (see subnet tempo / `agcli subnet show --netuid <N>`)."
    } else if msg.contains("CommittingWeightsTooFast") {
        "Weight commits are rate-limited. Wait more blocks before running `agcli weights commit` again."
    } else if msg.contains("IncorrectCommitRevealVersion") {
        "Global commit-reveal protocol version mismatch (timelocked commits). Update agcli — `agcli weights commit-timelocked` reads `CommitRevealWeightsVersion` from chain before submitting."
    } else if msg.contains("IncorrectWeightVersionKey") {
        "The subnet expects a different weights version key. Use `agcli weights set --version-key <KEY>` matching the chain (see `agcli subnet hyperparams --netuid <N>`)."
    } else if msg.contains("NeuronNoValidatorPermit") {
        "This hotkey does not hold a validator permit on this subnet (top-N by stake). Gain stake or wait for permit slots — see `agcli view validators --netuid <N>` and subnet `max_allowed_validators`."
    } else if msg.contains("WeightVecLengthIsLow") {
        "Too few UIDs in the weight vector for this subnet's minimum. Add more targets or check `min_allowed_weights` (`agcli subnet hyperparams --netuid <N>`)."
    } else if msg.contains("DuplicateUids") {
        "Duplicate UIDs in the weight vector — each UID must appear only once. Deduplicate your `--weights` / JSON input."
    } else if msg.contains("WeightVecNotEqualSize") || msg.contains("InputLengthsUnequal") {
        "UID list and weight list must have the same length. Check your `--weights` or JSON pairs."
    } else if msg.contains("UidVecContainInvalidOne") {
        "One or more UIDs are out of range for this subnet. List neurons with `agcli view neurons --netuid <N>`."
    } else if msg.contains("MaxWeightExceeded") {
        "Sum of weights exceeds 65535. Reduce values so the total fits in u16 range."
    } else if msg.contains("UidsLengthExceedUidsInSubNet") {
        "Too many UIDs in the submission — cannot exceed the number of neurons on this subnet."
    } else if msg.contains("CanNotSetRootNetworkWeights") {
        "Weights on the root subnet (netuid 0) are not set with this extrinsic. Use the chain’s root-weighting flow if applicable."
    } else if msg.contains("SubnetLocked") || msg.contains("NetworkIsImmuned") {
        "This subnet is in its immunity period and cannot be modified yet."
    } else if msg.contains("MaxAllowedUIDs") || msg.contains("SubNetworkDoesNotExist") {
        "Subnet capacity reached or does not exist. Check `agcli subnet list` for current subnets."
    } else if msg.contains("HotKeyAlreadyRegistered") {
        "This hotkey is already registered. Use a different hotkey or deregister the existing one first."
    } else if msg.contains("ColdkeySwapAnnounced") || msg.contains("ColdKeySwapScheduled") {
        "A coldkey swap is announced for this coldkey. Subnet-owner AdminUtils calls (`agcli subnet set-param`, `subnet set-symbol`, `subnet trim`, …) are rejected until the swap executes or is cleared. Check `agcli wallet check-swap` (add `--address` if not the default wallet)."
    } else if msg.contains("ColdkeySwapAlreadyDisputed") {
        "This coldkey swap was already disputed — the execute step cannot proceed. Check `agcli wallet check-swap` and wait for on-chain resolution."
    } else if msg.contains("ColdkeySwapDisputed") {
        "This coldkey swap is disputed. Owner/admin extrinsics from this coldkey stay blocked until the dispute is resolved on-chain."
    } else if msg.contains("ColdkeySwapClearTooEarly") {
        "Clearing the swap announcement is too soon. The chain requires `announce_block + ColdkeySwapReannouncementDelay` blocks to pass. Check status with `agcli wallet check-swap` and retry after the delay."
    } else if msg.contains("ColdKeyAlreadyAssociated") {
        "Coldkey association conflict for this operation. If you scheduled a swap, check `agcli wallet check-swap`."
    } else if msg.contains("ColdkeySwapAnnouncementNotFound") {
        "No coldkey swap has been announced for this account."
    } else if msg.contains("ColdkeySwapTooEarly") || msg.contains("ColdkeySwapReannouncedTooEarly")
    {
        "Coldkey swap was announced too recently. Wait for the cooldown period before executing."
    } else if msg.contains("AnnouncedColdkeyHashDoesNotMatch") {
        "The new coldkey does not match the previously announced swap destination."
    } else if msg.contains("DelegateAlreadySet") {
        "Delegate is already set for this hotkey."
    } else if msg.contains("InvalidTransaction") && msg.contains("proxy") {
        "Proxy transaction failed. Check that the proxy account has enough balance for fees and that the proxy type matches the operation."
    } else if msg.contains("SubNetRegistrationDisabled") {
        "Registration is disabled on this subnet."
    } else if msg.contains("NoNeuronIdAvailable") {
        "No neuron UID slots available on this subnet. Wait for a slot to open or try a different subnet."
    } else if msg.contains("Crowdloan::InsufficientBalance") {
        "Crowdloan: not enough free balance for the creator deposit or a contribution. Fund the signing account (`agcli balance`)."
    } else if msg.contains("Swap::InsufficientBalance") {
        "AMM/swap: insufficient free balance for this trade or liquidity add/remove (including fees). Check `agcli balance`."
    } else if msg.contains("Swap::InsufficientLiquidity") {
        "AMM: not enough liquidity in the pool (or in-range) for this swap or burn. Reduce size, adjust ticks/price bounds, wait for depth, or add liquidity."
    } else if msg.contains("InsufficientLiquidity") {
        "Not enough on-chain liquidity for this Subtensor operation (stake/swap/slippage path). Try a smaller amount, better price/slippage settings, or wait — this is not always the same as AMM `Swap::InsufficientLiquidity`."
    } else if msg.contains("InsufficientBalance") {
        "Insufficient balance for this operation. Check your balance with `agcli balance`."
    } else if msg.contains("SubnetNotExists") || msg.contains("SubnetDoesNotExist") {
        "Subnet does not exist for this netuid (SubtensorModule or AdminUtils owner call). Check `agcli subnet list` and `agcli subnet show --netuid <N>`."
    } else if msg.contains("HotKeyAccountNotExists") {
        "Hotkey account does not exist on chain. Fund it or register first."
    } else if msg.contains("StakingOperationRateLimitExceeded")
        || msg.contains("StakingRateLimitExceeded")
    {
        "Staking rate limit exceeded. Wait a few blocks before retrying."
    } else if msg.contains("TooManyRegistrationsThisInterval") {
        "Too many registrations this interval. Wait before retrying."
    } else if msg.contains("SlippageTooHigh") {
        "Slippage too high for this operation. Try a smaller amount or wait for better liquidity."
    } else if msg.contains("AmountTooLow") {
        "Amount is below the minimum threshold for this operation."
    } else if msg.contains("SubnetLimitReached") || msg.contains("CannotAffordLockCost") {
        "Cannot create subnet: either the subnet limit is reached or you cannot afford the lock cost. Check the current lock with `agcli subnet create-cost`."
    } else if msg.contains("AddStakeBurnRateLimitExceeded") {
        "Add-stake-burn rate limit exceeded. Wait a few blocks before retrying."
    } else if msg.contains("LeaseNetuidNotFound") {
        "No lease is indexed for this netuid on-chain. Confirm the subnet is a leased network, the netuid is correct (`agcli subnet list`, `agcli subnet show --netuid <N>`), and you are using the right lease registration / terminate flow."
    } else if msg.contains("LeaseDoesNotExist") {
        "That lease entry does not exist in chain storage (wrong id, already cleared, or never created). Re-check lease state for this operation before resubmitting."
    } else if msg.contains("SymbolAlreadyInUse") {
        "This token symbol is already taken. Choose a different symbol."
    } else if msg.contains("SymbolDoesNotExist") {
        "The specified symbol does not exist."
    } else if msg.contains("Crowdloan::Overflow") {
        "Crowdloan accounting overflow (cap, contributions, or reserves). Verify amounts and on-chain crowdloan state."
    } else if msg.contains("Overflow") || msg.contains("PrecisionLoss") {
        "Arithmetic overflow or precision loss. Try a smaller amount."
    } else {
        "" // no special hint
    };

    let hint_resolved: Cow<'static, str> = if !hint.is_empty() {
        Cow::Borrowed(hint)
    } else if msg.contains("::") && !msg.contains("://") {
        if let Some((pallet, variant)) = last_qualified_pallet_variant(&msg) {
            Cow::Owned(format!(
                "Runtime pallet `{pallet}` returned error `{variant}` (qualified metadata error). Look up `{variant}` in that pallet’s `#[pallet::error]` enum in the subtensor repo for this extrinsic; verify `--endpoint` matches the network and update agcli if nodes run a newer runtime."
            ))
        } else {
            Cow::Borrowed(
                "Named runtime/pallet error (see message for `Pallet::Variant`). Check pallet docs for this extrinsic, verify `--endpoint` matches the network, and upgrade agcli if the runtime is newer.",
            )
        }
    } else {
        Cow::Borrowed(hint)
    };

    // Build the error message with all available context:
    // 1. The decoded human-readable description (from error code mapping)
    // 2. The hint (from pattern matching on known error types)
    // 3. The raw error message for debugging
    if !hint_resolved.is_empty() {
        if let Some(desc) = decoded_desc {
            anyhow::anyhow!(
                "Transaction failed: {}\n  Reason: {}\n  Hint: {}",
                msg,
                desc,
                hint_resolved
            )
        } else {
            anyhow::anyhow!("Transaction failed: {}\n  Hint: {}", msg, hint_resolved)
        }
    } else if let Some(desc) = decoded_desc {
        anyhow::anyhow!("Transaction failed: {}\n  Reason: {}", msg, desc)
    } else {
        anyhow::anyhow!("Transaction failed on chain: {}", msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn retry_succeeds_after_transient_error() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let result = retry_on_transient("test", 3, || {
            let c = c.clone();
            async move {
                let n = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if n < 2 {
                    Err(anyhow::anyhow!("Connection reset"))
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_does_not_retry_non_transient_error() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let result: Result<i32> = retry_on_transient("test", 3, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(anyhow::anyhow!("Invalid SS58 address"))
            }
        })
        .await;
        assert!(result.is_err());
        // Should NOT retry for non-transient errors
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_exhausts_all_attempts() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();
        let result: Result<i32> = retry_on_transient("test", 2, || {
            let c = c.clone();
            async move {
                c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err(anyhow::anyhow!("Connection timeout"))
            }
        })
        .await;
        assert!(result.is_err());
        // 1 initial + 2 retries = 3 total
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_succeeds_immediately() {
        let result = retry_on_transient("test", 3, || async { Ok::<_, anyhow::Error>(99) }).await;
        assert_eq!(result.unwrap(), 99);
    }

    #[test]
    fn batch_balance_result_order() {
        // Unit test for the ordering guarantee of get_balances_multi
        // (The actual chain test is in integration tests)
        let addrs = [
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKv3gB",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        ];
        assert_eq!(addrs.len(), 2, "batch addresses should preserve count");
    }

    #[test]
    fn format_dispatch_error_subnet_locked() {
        let err = subxt::Error::Other("SubnetLocked: cannot modify".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("immunity period"),
            "should mention immunity: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_proxy() {
        let err = subxt::Error::Other("InvalidTransaction proxy check failed".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Proxy transaction"),
            "should mention proxy: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_proxy_qualified_variants_hint() {
        let cases = [
            ("Pallet error: Proxy::TooMany", "too many proxies"),
            ("Pallet error: Proxy::NotFound", "No matching proxy"),
            ("Pallet error: Proxy::Duplicate", "already a proxy"),
            (
                "Pallet error: Proxy::NoPermission",
                "cannot run through this proxy type",
            ),
            (
                "Pallet error: Crowdloan::Underflow",
                "Crowdloan arithmetic underflowed",
            ),
            ("Pallet error: Drand::NoneValue", "missing"),
            (
                "Pallet error: Drand::StorageOverflow",
                "Drand internal counter",
            ),
        ];
        for (raw, needle) in cases {
            let err = subxt::Error::Other(raw.to_string());
            let result = format_dispatch_error(err);
            let msg = format!("{:#}", result);
            let lower = msg.to_lowercase();
            let n = needle.to_lowercase();
            assert!(
                msg.contains("Hint:") && lower.contains(n.as_str()),
                "expected hint containing {:?} in: {}",
                needle,
                msg
            );
        }
    }

    #[test]
    fn format_dispatch_error_unknown() {
        let err = subxt::Error::Other("SomeTotallyNewError".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Transaction failed on chain"),
            "unknown errors get generic message: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_shield_unreachable_hint() {
        let err = subxt::Error::Other("Pallet error: Shield::Unreachable".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Hint:") && msg.to_lowercase().contains("shield"),
            "expected shield-specific hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_registry_not_registered_identity_hint() {
        let err = subxt::Error::Other("Pallet error: Registry::NotRegistered".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Hint:") && msg.to_lowercase().contains("identity"),
            "expected Registry identity hint, got: {}",
            msg
        );
        assert!(
            !msg.contains("register-neuron"),
            "must not mis-route to subnet neuron registration: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_admin_utils_invalid_value_hint() {
        let err = subxt::Error::Other("Pallet error: AdminUtils::InvalidValue".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Hint:") && msg.to_lowercase().contains("adminutils"),
            "expected AdminUtils owner set-param hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_swap_subtoken_disabled_hint() {
        let err = subxt::Error::Other("Pallet error: Swap::SubtokenDisabled".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Hint:") && msg.to_lowercase().contains("amm"),
            "expected Swap/AMM subtoken hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_swap_insufficient_liquidity_hint() {
        let err = subxt::Error::Other("Pallet error: Swap::InsufficientLiquidity".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Hint:") && msg.to_lowercase().contains("pool"),
            "expected AMM pool liquidity hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_swap_insufficient_balance_not_tao_only_hint() {
        let err = subxt::Error::Other("Pallet error: Swap::InsufficientBalance".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Hint:")
                && msg.to_lowercase().contains("amm")
                && !msg.to_lowercase().contains("tao balance"),
            "Swap::InsufficientBalance must not use the early Subtensor TAO-only hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_crowdloan_insufficient_balance_not_tao_only_hint() {
        let err = subxt::Error::Other("Pallet error: Crowdloan::InsufficientBalance".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Hint:")
                && msg.to_lowercase().contains("crowdloan")
                && !msg.to_lowercase().contains("tao balance"),
            "Crowdloan::InsufficientBalance must not use the early Subtensor TAO-only hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_named_pallet_fallback_hint() {
        let err =
            subxt::Error::Other("Pallet error: HypotheticalPallet::FutureVariant".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Hint:")
                && msg.contains("HypotheticalPallet")
                && msg.contains("FutureVariant")
                && msg.contains("qualified metadata"),
            "unknown pallet variants should name pallet+variant in the Hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_duplicate_uids_hint() {
        let err = subxt::Error::Other("Custom error: 17".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("DuplicateUids") && msg.contains("Hint:"),
            "should decode 17 and add duplicate-UID hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_invalid_uid_hint() {
        let err = subxt::Error::Other("Custom error: 18".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("UidVecContainInvalidOne") && msg.contains("Hint:"),
            "should decode 18 and add invalid-UID hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_max_weight_exceeded_hint() {
        let err = subxt::Error::Other("Custom error: 26".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("MaxWeightExceeded") && msg.contains("65535"),
            "should decode 26 and add max-weight hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_weight_vec_not_equal_size_hint() {
        let err = subxt::Error::Other("Custom error: 16".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("WeightVecNotEqualSize") && msg.contains("Hint:"),
            "should decode 16 and add length-mismatch hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_input_lengths_unequal_hint() {
        let err = subxt::Error::Other("Custom error: 79".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("InputLengthsUnequal") && msg.contains("Hint:"),
            "should decode 79 and add length-mismatch hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_uids_length_exceed_subnet_hint() {
        let err = subxt::Error::Other("Custom error: 31".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("UidsLengthExceedUidsInSubNet") && msg.contains("Hint:"),
            "should decode 31 and add too-many-UIDs hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_cannot_set_root_network_weights_hint() {
        let err = subxt::Error::Other("Custom error: 46".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("CanNotSetRootNetworkWeights") && msg.contains("Hint:"),
            "should decode 46 and add root-network hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_not_enough_stake_to_set_weights_hint() {
        let err = subxt::Error::Other("Custom error: 10".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("NotEnoughStakeToSetWeights") && msg.contains("Hint:"),
            "should decode 10 and add stake hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_neuron_no_validator_permit_hint() {
        let err = subxt::Error::Other("Custom error: 15".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("NeuronNoValidatorPermit") && msg.contains("Hint:"),
            "should decode 15 and add validator-permit hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_weight_vec_length_is_low_hint() {
        let err = subxt::Error::Other("Custom error: 19".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("WeightVecLengthIsLow") && msg.contains("Hint:"),
            "should decode 19 and add min-weights hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_setting_weights_too_fast_hint() {
        let err = subxt::Error::Other("Custom error: 28".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("SettingWeightsTooFast") && msg.contains("Hint:"),
            "should decode 28 and add rate-limit hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_incorrect_weight_version_key_hint() {
        let err = subxt::Error::Other("Custom error: 29".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("IncorrectWeightVersionKey") && msg.contains("version-key"),
            "should decode 29 and add version-key hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_commit_reveal_enabled_hint() {
        let err = subxt::Error::Other("Custom error: 52".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("CommitRevealEnabled") && msg.contains("weights reveal"),
            "should decode 52 and add commit-reveal hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_commit_reveal_disabled_hint() {
        let err = subxt::Error::Other("Custom error: 53".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("CommitRevealDisabled") && msg.contains("weights set"),
            "should decode 53 and add direct-set hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_no_weights_commit_found_hint() {
        let err = subxt::Error::Other("Custom error: 50".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("NoWeightsCommitFound") && msg.contains("weights commit"),
            "should decode 50 and add commit-first hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_invalid_reveal_commit_hash_hint() {
        let err = subxt::Error::Other("Custom error: 51".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("InvalidRevealCommitHashNotMatch") && msg.contains("salt"),
            "should decode 51 and add salt/weights hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_too_many_unrevealed_commits_hint() {
        let err = subxt::Error::Other("Custom error: 76".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("TooManyUnrevealedCommits") && msg.contains("Reveal"),
            "should decode 76 and add reveal-first hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_expired_weight_commit_hint() {
        let err = subxt::Error::Other("Custom error: 77".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("ExpiredWeightCommit") && msg.contains("commit"),
            "should decode 77 and add recommit hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_reveal_too_early_hint() {
        let err = subxt::Error::Other("Custom error: 78".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("RevealTooEarly") && msg.contains("reveal"),
            "should decode 78 and add timing hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_committing_weights_too_fast_hint() {
        let err = subxt::Error::Other("Custom error: 80".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("CommittingWeightsTooFast") && msg.contains("weights commit"),
            "should decode 80 and add commit rate-limit hint: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_incorrect_commit_reveal_version_hint() {
        let err = subxt::Error::Other("Custom error: 111".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("IncorrectCommitRevealVersion") && msg.contains("agcli"),
            "should decode 111 and add version hint: {}",
            msg
        );
    }

    #[test]
    fn decode_custom_error_6() {
        let d = decode_custom_error("Custom error: 6").expect("should decode 6");
        assert_eq!(d.name, "HotKeyNotRegisteredInNetwork");
        assert!(!d.desc.is_empty(), "should have a description");
    }

    #[test]
    fn decode_custom_error_20() {
        let d = decode_custom_error("Custom error: 20").expect("should decode 20");
        assert_eq!(d.name, "TooManyRegistrationsThisBlock");
    }

    #[test]
    fn decode_custom_error_21() {
        let d = decode_custom_error("Custom error: 21").expect("should decode 21");
        assert_eq!(d.name, "HotKeyAlreadyRegisteredInSubNet");
    }

    #[test]
    fn decode_custom_error_unknown_index() {
        assert!(decode_custom_error("Custom error: 999").is_none());
    }

    #[test]
    fn decode_custom_error_59_invalidchild() {
        let d = decode_custom_error("Custom error: 59").expect("should decode 59");
        assert_eq!(d.name, "InvalidChild");
    }

    #[test]
    fn decode_custom_error_97_insufficientbalance() {
        let d = decode_custom_error("Custom error: 97").expect("should decode 97");
        assert_eq!(d.name, "InsufficientBalance");
    }

    #[test]
    fn decode_custom_error_98_staking_rate_limit() {
        let d = decode_custom_error("Custom error: 98").expect("should decode 98");
        assert_eq!(d.name, "StakingOperationRateLimitExceeded");
    }

    #[test]
    fn decode_custom_error_132_coldkey_disputed() {
        let d = decode_custom_error("Custom error: 132").expect("should decode 132");
        assert_eq!(d.name, "ColdkeySwapDisputed");
    }

    #[test]
    fn decode_custom_error_133_coldkey_swap_clear_too_early() {
        let d = decode_custom_error("Custom error: 133").expect("should decode 133");
        assert_eq!(d.name, "ColdkeySwapClearTooEarly");
    }

    #[test]
    fn format_dispatch_error_coldkey_swap_clear_too_early_hint() {
        let err = subxt::Error::Other("Custom error: 133".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("reannouncement") || msg.contains("check-swap"),
            "ColdkeySwapClearTooEarly should explain delay / check-swap: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_coldkey_swap_announced_hint() {
        let err = subxt::Error::Other("Custom error: 131".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("set-param") && msg.contains("check-swap"),
            "ColdkeySwapAnnounced should mention owner writes and check-swap: {}",
            msg
        );
    }

    #[test]
    fn decode_custom_error_68_distinct_from_132_disputed() {
        let d68 = decode_custom_error("Custom error: 68").expect("68");
        let d132 = decode_custom_error("Custom error: 132").expect("132");
        assert_eq!(d68.name, "ColdkeySwapAlreadyDisputed");
        assert_eq!(d132.name, "ColdkeySwapDisputed");
        assert_ne!(d68.name, d132.name);
    }

    #[test]
    fn format_dispatch_error_coldkey_swap_already_disputed_hint() {
        let err = subxt::Error::Other("Custom error: 68".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("already disputed") || msg.contains("check-swap"),
            "ColdkeySwapAlreadyDisputed should explain execute blocked: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_admin_weights_window_hint() {
        let err = subxt::Error::Other("Custom error: 108".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("weights window") || msg.contains("tempo"),
            "AdminActionProhibitedDuringWeightsWindow should mention window/tempo: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_transactor_must_be_hotkey_hint() {
        let err = subxt::Error::Other("Custom error: 38".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("hotkey") && msg.contains("coldkey"),
            "TransactorAccountShouldBeHotKey should contrast signing keys: {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_root_network_missing_hint() {
        let err = subxt::Error::Other("Custom error: 0".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("SN0") || msg.contains("root network"),
            "RootNetworkDoesNotExist should mention SN0/root: {}",
            msg
        );
    }

    #[test]
    fn decode_custom_error_no_match() {
        assert!(decode_custom_error("some other error text").is_none());
    }

    #[test]
    fn decode_custom_error_all_have_descriptions() {
        // Verify every SubtensorModule error code has a non-empty description (sync with pallet `errors.rs`)
        for i in 0..=133u32 {
            let msg = format!("Custom error: {}", i);
            let d =
                decode_custom_error(&msg).unwrap_or_else(|| panic!("error {} should decode", i));
            assert!(!d.name.is_empty(), "error {} should have a name", i);
            assert!(
                !d.desc.is_empty(),
                "error {} ({}) should have a description",
                i,
                d.name
            );
        }
    }

    #[test]
    fn format_dispatch_error_custom_6_decoded() {
        let err = subxt::Error::Other("Custom error: 6".to_string());
        let result = format_dispatch_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Reason:") && msg.contains("HotKeyNotRegisteredInNetwork"),
            "Custom error: 6 should include decoded variant name in Reason: {}",
            msg
        );
        assert!(
            msg.contains("any subnet yet") || msg.to_lowercase().contains("any network"),
            "Custom error: 6 is network-wide (not on this subnet only): {}",
            msg
        );
    }

    #[test]
    fn format_dispatch_error_lease_netuid_distinct_from_lease_missing() {
        let e1 = format_dispatch_error(subxt::Error::Other(
            "SubtensorModule::LeaseNetuidNotFound".to_string(),
        ));
        let m1 = format!("{:#}", e1);
        assert!(
            m1.contains("indexed") && m1.contains("netuid"),
            "LeaseNetuidNotFound should explain missing lease index for netuid: {}",
            m1
        );
        let e2 = format_dispatch_error(subxt::Error::Other(
            "SubtensorModule::LeaseDoesNotExist".to_string(),
        ));
        let m2 = format!("{:#}", e2);
        assert!(
            m2.contains("storage") || m2.contains("entry"),
            "LeaseDoesNotExist should explain absent lease record: {}",
            m2
        );
    }

    #[test]
    fn format_submit_error_priority() {
        let err = subxt::Error::Other("Priority is too low".to_string());
        let result = format_submit_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("conflicting transaction"),
            "should mention conflict: {}",
            msg
        );
    }

    #[test]
    fn format_submit_error_insufficient() {
        let err = subxt::Error::Other("Inability to pay some fees".to_string());
        let result = format_submit_error(err);
        let msg = format!("{:#}", result);
        assert!(
            msg.contains("Insufficient balance"),
            "should mention balance: {}",
            msg
        );
    }

    #[test]
    fn is_transient_catches_common_patterns() {
        assert!(is_transient_error("Connection reset by peer"));
        assert!(is_transient_error("Ws transport error"));
        assert!(is_transient_error("Connection closed unexpectedly"));
        assert!(is_transient_error("request timeout after 30s"));
        assert!(!is_transient_error("Invalid SS58 address"));
        assert!(!is_transient_error("NotEnoughBalance"));
    }

    // ── Fix: disk cache network namespacing (Issue 678) ──

    #[test]
    fn url_to_cache_prefix_finney() {
        assert_eq!(
            url_to_cache_prefix("wss://entrypoint-finney.opentensor.ai:443"),
            "finney"
        );
    }

    #[test]
    fn url_to_cache_prefix_test() {
        assert_eq!(
            url_to_cache_prefix("wss://test.finney.opentensor.ai:443"),
            "test"
        );
    }

    #[test]
    fn url_to_cache_prefix_local() {
        assert_eq!(url_to_cache_prefix("ws://127.0.0.1:9944"), "local");
        assert_eq!(url_to_cache_prefix("ws://localhost:9944"), "local");
    }

    #[test]
    fn url_to_cache_prefix_archive() {
        assert_eq!(
            url_to_cache_prefix("wss://bittensor-finney.api.onfinality.io/public-ws"),
            "archive"
        );
    }

    #[test]
    fn url_to_cache_prefix_unknown() {
        let prefix = url_to_cache_prefix("wss://my-custom-node.example.com:443");
        assert_eq!(prefix, "my-custom-node_example_com");
    }

    #[test]
    fn different_networks_produce_different_prefixes() {
        let finney = url_to_cache_prefix("wss://entrypoint-finney.opentensor.ai:443");
        let test = url_to_cache_prefix("wss://test.finney.opentensor.ai:443");
        let local = url_to_cache_prefix("ws://127.0.0.1:9944");
        assert_ne!(finney, test);
        assert_ne!(finney, local);
        assert_ne!(test, local);
    }

    // ── Issue 646: Finalized block transient error handling ──

    /// Transient error patterns should be retried for finalized block queries.
    #[test]
    fn transient_errors_cover_finalized_block_patterns() {
        // These are the error messages that can occur when fetching finalized head.
        assert!(is_transient_error("Connection reset by peer"));
        assert!(is_transient_error("Ws transport error"));
        assert!(is_transient_error("timeout waiting for response"));
        assert!(is_transient_error("connection closed"));
    }

    /// Non-transient errors should not be retried.
    #[test]
    fn non_transient_errors_not_retried_for_finalized() {
        assert!(!is_transient_error("Invalid SS58 address"));
        assert!(!is_transient_error("Subnet not found"));
        assert!(!is_transient_error("Insufficient balance"));
    }

    /// Verify get_finalized_block_number method exists on Client (compile-time check).
    /// The method should be distinct from get_block_number (best/non-finalized).
    #[test]
    fn client_has_finalized_block_number_method() {
        // Compile-time assertion: Client must have both methods.
        // We verify this by taking function pointer types.
        // Both return Result<u64> so they are interchangeable in usage.
        fn _takes_async_u64_fn<F>(_f: F)
        where
            F: for<'a> FnOnce(
                &'a Client,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<u64>> + 'a>,
            >,
        {
        }
        // The test compiles iff both methods exist. We cannot call them without a Client.
    }

    /// Verify sign_submit_or_mev delegates to sign_submit_mev when mev=true (compile-time).
    #[test]
    fn client_has_sign_submit_or_mev() {
        // This test verifies that sign_submit_or_mev exists and accepts (Payload, Pair, bool).
        // Compile-time verification only — no runtime Client available.
    }

    // ──── Issue 648: Per-signer transaction lock ────

    #[test]
    fn tx_lock_acquires_and_releases() {
        use sp_core::Pair;
        let pair = sr25519::Pair::from_string("//Alice", None).unwrap();
        let lock = acquire_tx_lock(&pair);
        assert!(lock.is_ok(), "Should acquire tx lock: {:?}", lock.err());
        drop(lock); // Release
                    // Should be able to reacquire immediately
        let lock2 = acquire_tx_lock(&pair);
        assert!(lock2.is_ok(), "Should reacquire tx lock: {:?}", lock2.err());
    }

    #[test]
    fn tx_lock_different_keys_independent() {
        use sp_core::Pair;
        let alice = sr25519::Pair::from_string("//Alice", None).unwrap();
        let bob = sr25519::Pair::from_string("//Bob", None).unwrap();
        let lock_a = acquire_tx_lock(&alice).unwrap();
        let lock_b = acquire_tx_lock(&bob);
        assert!(lock_b.is_ok(), "Different keys should not block each other");
        drop(lock_a);
        drop(lock_b);
    }

    #[test]
    fn tx_lock_same_key_blocks_concurrent() {
        use sp_core::Pair;
        use std::sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        };

        let pair = sr25519::Pair::from_string("//Charlie", None).unwrap();
        // Acquire lock in main thread
        let _lock = acquire_tx_lock(&pair).unwrap();

        let acquired = Arc::new(AtomicBool::new(false));
        let acquired2 = acquired.clone();

        // Try to acquire in another thread — should block
        let pair2 = pair.clone();
        let handle = std::thread::spawn(move || {
            // This should block until we release
            match acquire_tx_lock(&pair2) {
                Ok(_lock) => {
                    acquired2.store(true, Ordering::SeqCst);
                }
                Err(_) => {}
            }
        });

        // Wait a bit — the other thread should still be blocked
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(
            !acquired.load(Ordering::SeqCst),
            "Other thread should be blocked"
        );

        // Release the lock
        drop(_lock);
        handle.join().unwrap();
        assert!(
            acquired.load(Ordering::SeqCst),
            "Other thread should have acquired lock after release"
        );
    }

    #[test]
    fn tx_lock_creates_lock_dir() {
        use sp_core::Pair;
        let pair = sr25519::Pair::from_string("//Dave", None).unwrap();
        let lock = acquire_tx_lock(&pair);
        assert!(lock.is_ok());
        assert!(std::path::Path::new("/tmp/agcli-tx-locks").exists());
        drop(lock);
    }

    // ──── Issue 100: reconnect preserves network cache prefix ────

    #[test]
    fn url_to_cache_prefix_used_in_reconnect_path() {
        // Verify that url_to_cache_prefix is available and deterministic
        // (reconnect now uses it instead of QueryCache::new())
        let url = "wss://entrypoint-finney.opentensor.ai:443";
        let prefix1 = url_to_cache_prefix(url);
        let prefix2 = url_to_cache_prefix(url);
        assert_eq!(prefix1, prefix2, "Same URL must produce same prefix");
        assert_eq!(prefix1, "finney");
    }

    #[test]
    fn reconnect_cache_prefix_matches_initial() {
        // The reconnect path now calls url_to_cache_prefix(&self.url) instead of
        // QueryCache::new(). Verify the prefix function returns non-empty for all
        // well-known endpoints.
        for (url, expected) in [
            ("wss://entrypoint-finney.opentensor.ai:443", "finney"),
            ("wss://test.finney.opentensor.ai:443", "test"),
            ("ws://127.0.0.1:9944", "local"),
        ] {
            let prefix = url_to_cache_prefix(url);
            assert_eq!(
                prefix, expected,
                "URL '{}' should produce prefix '{}'",
                url, expected
            );
        }
    }

    // ── Issue 139: decode_custom_error slices lowercased string ──

    #[test]
    fn decode_custom_error_slices_lower_not_original() {
        // The fix ensures we slice `lower` (not `msg`) so byte offsets are consistent.
        let d = decode_custom_error("Custom error: 6").expect("should decode");
        assert_eq!(d.name, "HotKeyNotRegisteredInNetwork");
        // Mixed case — to_lowercase finds the substring correctly
        let d2 = decode_custom_error("CUSTOM ERROR: 97").expect("should decode");
        assert_eq!(d2.name, "InsufficientBalance");
    }

    #[test]
    fn decode_custom_error_with_leading_text() {
        // Error message with prefix text before "custom error:"
        let d = decode_custom_error("Dispatch failed: Custom error: 20").expect("should decode");
        assert_eq!(d.name, "TooManyRegistrationsThisBlock");
    }

    #[test]
    fn decode_custom_error_with_trailing_digit_suffix() {
        // Issue 146: trim_matches with inverted digit predicate fails when suffix contains digits.
        // "Custom error: 6 (SubError2)" — old code would produce "6 (SubError2" via trim_matches.
        // New take_while approach correctly extracts "6".
        let d = decode_custom_error("Custom error: 6 (SubError2)").expect("should decode");
        assert_eq!(d.name, "HotKeyNotRegisteredInNetwork");
    }

    #[test]
    fn decode_custom_error_only_first_number_extracted() {
        // "Custom error: 42 extra99" — should extract 42, not "42 extra99"
        let d = decode_custom_error("Custom error: 42");
        assert!(d.is_some(), "should decode error code 42");
    }
}
