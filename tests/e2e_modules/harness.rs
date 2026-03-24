pub use agcli::chain::Client;
pub use agcli::cli::helpers::{
    check_spending_limit, safe_rao, validate_amount, validate_netuid, validate_ss58,
};
pub use agcli::extrinsics::compute_weight_commit_hash;
pub use agcli::queries::subnet::list_subnets;
pub use agcli::types::balance::Balance;
pub use agcli::types::chain_data::{AxonInfo, SubnetIdentity};
pub use agcli::types::network::NetUid;
pub use sp_core::{sr25519, Pair};
pub use std::process::Command;
pub use std::sync::Once;
pub use std::time::Duration;
// StreamExt is needed for .next() on block subscriptions
#[allow(unused_imports)]
pub use futures::StreamExt;
// ──────── Constants ────────

pub const LOCAL_WS: &str = "ws://127.0.0.1:9944";
pub const CONTAINER_NAME: &str = "agcli_e2e_test";
pub const DOCKER_IMAGE: &str = "ghcr.io/opentensor/subtensor-localnet:devnet-ready";

/// Alice is the sudo account in localnet, pre-funded with 1M TAO.
pub const ALICE_URI: &str = "//Alice";
pub const ALICE_SS58: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

/// Bob is another pre-funded dev account.
pub const BOB_URI: &str = "//Bob";
pub const BOB_SS58: &str = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

// ──────── Harness ────────

pub static INIT: Once = Once::new();

/// Ensure a local chain container is running. Idempotent — only starts once.
pub fn ensure_local_chain() {
    INIT.call_once(|| {
        // Kill any stale containers using our port
        let _ = Command::new("docker").args(["rm", "-f", CONTAINER_NAME]).output();
        // Also kill any other container that might be on port 9944
        let _ = Command::new("bash")
            .args(["-c", "docker ps -q --filter publish=9944 | xargs -r docker rm -f"])
            .output();

        // Brief pause for port release
        std::thread::sleep(Duration::from_secs(1));

        // Start fresh container in fast-block mode (250ms blocks).
        let output = Command::new("docker")
            .args([
                "run", "--rm", "-d",
                "--name", CONTAINER_NAME,
                "-p", "9944:9944",
                "-p", "9945:9945",
                DOCKER_IMAGE,
            ])
            .output()
            .expect("Failed to run Docker — is Docker installed and running?");

        assert!(
            output.status.success(),
            "Docker container failed to start:\n  stdout: {}\n  stderr: {}\n  Pull image: docker pull {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
            DOCKER_IMAGE
        );
    });
}

/// Wait for the chain to produce blocks and be connectable.
pub async fn wait_for_chain() -> Client {
    let max_attempts = 30;
    for attempt in 1..=max_attempts {
        match Client::connect(LOCAL_WS).await {
            Ok(client) => {
                // Verify blocks are being produced
                match client.get_block_number().await {
                    Ok(block) if block > 0 => {
                        println!("[harness] connected at block {block}");
                        return client;
                    }
                    _ => {}
                }
            }
            Err(_) => {}
        }
        if attempt == max_attempts {
            panic!("Chain did not become ready after {} attempts", max_attempts);
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    unreachable!()
}

/// Derive an sr25519 keypair from a dev URI like "//Alice".
pub fn dev_pair(uri: &str) -> sr25519::Pair {
    sr25519::Pair::from_string(uri, None).expect("valid dev URI")
}

/// Convert a public key to SS58 with prefix 42.
pub fn to_ss58(pub_key: &sr25519::Public) -> String {
    sp_core::crypto::Ss58Codec::to_ss58check_with_version(pub_key, 42u16.into())
}

/// Reconnect if the WebSocket connection is dead.
/// After reconnecting, validates that the chain block number is reasonable
/// (>100) to avoid stale/syncing node connections.
pub async fn ensure_alive(client: &mut Client) {
    if client.is_alive().await {
        return;
    }
    // After a chain restart, 250ms blocks take ~25s to reach block 100.
    // We allow up to 60 attempts × 3s = 180s to reconnect and wait for the chain to stabilize.
    for attempt in 1..=60u64 {
        match client.reconnect().await {
            Ok(()) => {
                let block = client.get_block_number().await.unwrap_or(0);
                if block < 100 {
                    if attempt <= 3 || attempt % 10 == 0 {
                        println!(
                            "  [reconnect] connected but block {} is too low (attempt {}), waiting...",
                            block, attempt
                        );
                    }
                    tokio::time::sleep(Duration::from_millis(3000)).await;
                    continue;
                }
                println!("  [reconnect] restored at block {}", block);
                return;
            }
            Err(_) => {
                if attempt == 60 {
                    println!("  [reconnect] WARNING: could not reconnect after 60 attempts");
                    return;
                }
                tokio::time::sleep(Duration::from_millis(1000 + 500 * attempt.min(10))).await;
            }
        }
    }
}

/// Ensure Alice is registered on a subnet and the subnet has basic config
/// (commit-reveal off, rate limits zeroed, validator permits available).
/// Call this at the start of any test that needs Alice to interact with a subnet.
/// Handles chain restarts gracefully by registering Alice if she's not present.
pub async fn ensure_alice_on_subnet(client: &mut Client, netuid: NetUid) -> u16 {
    let alice = dev_pair(ALICE_URI);
    let alice_ss58 = to_ss58(&alice.public());

    // Check if Alice is registered (retry on connection failure)
    let mut neurons = Vec::new();
    for _ in 0..5 {
        ensure_alive(client).await;
        match client.get_neurons_lite(netuid).await {
            Ok(n) => {
                neurons = n.to_vec();
                break;
            }
            Err(_) => {
                let _ = client.reconnect().await;
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        }
    }
    let uid = if let Some(n) = neurons.iter().find(|n| n.hotkey == ALICE_SS58) {
        n.uid
    } else {
        // Register Alice with retries for transient errors
        for reg_attempt in 1..=20u32 {
            ensure_alive(client).await;
            match client.burned_register(&alice, netuid, &alice_ss58).await {
                Ok(_) => break,
                Err(e) => {
                    let msg = format!("{e}");
                    if msg.contains("AlreadyRegistered") {
                        break;
                    }
                    if is_retryable(&msg) && reg_attempt < 20 {
                        if is_conn_dead(&msg) {
                            ensure_alive(client).await;
                        } else if needs_fresh_conn(&msg) {
                            let _ = client.reconnect().await;
                        }
                        tokio::time::sleep(Duration::from_millis(retry_delay_ms(&msg))).await;
                        continue;
                    }
                    println!(
                        "  [WARN] could not register Alice on SN{}: {}",
                        netuid.0, msg
                    );
                    break;
                }
            }
        }
        wait_blocks(client, 5).await;
        // Retry neuron lookup with reconnect
        let mut found_uid = None;
        for _ in 0..5 {
            ensure_alive(client).await;
            if let Ok(neurons2) = client.get_neurons_lite(netuid).await {
                if let Some(n) = neurons2.iter().find(|n| n.hotkey == ALICE_SS58) {
                    found_uid = Some(n.uid);
                    break;
                }
            }
            wait_blocks(client, 3).await;
        }
        match found_uid {
            Some(uid) => uid,
            None => return 0, // Fallback — chain may have restarted
        }
    };

    // Sudo config with retries — WeightsWindow can block admin calls
    {
        use subxt::dynamic::Value;
        let sudo_calls: &[(&str, Vec<subxt::dynamic::Value>)] = &[
            (
                "sudo_set_commit_reveal_weights_enabled",
                vec![Value::u128(netuid.0 as u128), Value::bool(false)],
            ),
            (
                "sudo_set_weights_set_rate_limit",
                vec![Value::u128(netuid.0 as u128), Value::u128(0)],
            ),
            (
                "sudo_set_max_allowed_validators",
                vec![Value::u128(netuid.0 as u128), Value::u128(256)],
            ),
            (
                "sudo_set_serving_rate_limit",
                vec![Value::u128(netuid.0 as u128), Value::u128(0)],
            ),
        ];
        for (call_name, fields) in sudo_calls {
            for attempt in 1..=5u32 {
                ensure_alive(client).await;
                match sudo_admin_call(client, &alice, call_name, fields.clone()).await {
                    Ok(_) => break,
                    Err(e) => {
                        if is_retryable(&e) && attempt < 5 {
                            tokio::time::sleep(Duration::from_millis(retry_delay_ms(&e))).await;
                            continue;
                        }
                        break; // best-effort, move on
                    }
                }
            }
            wait_blocks(client, 1).await;
        }
    }
    uid
}

pub fn is_retryable(msg: &str) -> bool {
    msg.contains("outdated")
        || msg.contains("banned")
        || msg.contains("subscription")
        || msg.contains("restart")
        || msg.contains("connection")
        || msg.contains("closed")
        || msg.contains("Custom error")
        || msg.contains("CommitRevealEnabled")
        || msg.contains("WeightsWindow")
        || msg.contains("Prohibited")
        || msg.contains("NeuronNoValidatorPermit")
        || msg.contains("NotRegistered")
        || msg.contains("HotKeyNotRegisteredInSubNet")
        || msg.contains("dispatch failed")
}

pub fn is_conn_dead(msg: &str) -> bool {
    msg.contains("closed") || msg.contains("restart") || msg.contains("connection")
}

/// Stale state (expired mortal era or tx pool priority) — needs fresh connection.
pub fn needs_fresh_conn(msg: &str) -> bool {
    msg.contains("outdated") || msg.contains("Custom error")
}

pub fn retry_delay_ms(msg: &str) -> u64 {
    if msg.contains("banned") {
        13_000
    } else if msg.contains("CommitRevealEnabled")
        || msg.contains("WeightsWindow")
        || msg.contains("Prohibited")
        || msg.contains("NeuronNoValidatorPermit")
        || msg.contains("dispatch failed")
    {
        1_500
    } else if msg.contains("Custom error") {
        1_500
    } else if msg.contains("subscription") || msg.contains("closed") {
        2_000
    } else if msg.contains("outdated") {
        1_500
    } else {
        500
    }
}

/// Wait for N blocks to pass (useful for extrinsic finalization in fast-block mode).
/// Tolerates transient RPC errors (connection drops) by retrying with backoff.
pub async fn wait_blocks(client: &mut Client, n: u64) {
    let start = match client.get_block_number().await {
        Ok(b) => b,
        Err(_) => {
            // RPC glitch — just sleep for estimated block time and return
            tokio::time::sleep(Duration::from_millis(n * 300)).await;
            return;
        }
    };
    let target = start + n;
    let mut failures = 0u32;
    loop {
        match client.get_block_number().await {
            Ok(current) if current >= target => return,
            Ok(_) => {
                failures = 0;
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            Err(_) => {
                failures += 1;
                if failures > 10 {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

/// Retry an extrinsic up to 20 times, reconnecting on connection errors.
/// Usage: retry_extrinsic!(client, client.transfer(...))
macro_rules! retry_extrinsic {
    ($client:expr, $call:expr) => {{
        let mut __re_result: String = String::new();
        let mut __re_done = false;
        for __re_attempt in 1u32..=20 {
            match $call.await {
                Ok(hash) => {
                    __re_result = hash;
                    __re_done = true;
                    break;
                }
                Err(e) => {
                    let msg = format!("{}", e);
                    if is_retryable(&msg) && __re_attempt < 20 {
                        if __re_attempt <= 3 {
                            println!("  attempt {} transient error, retrying...", __re_attempt);
                        }
                        if is_conn_dead(&msg) {
                            ensure_alive($client).await;
                        } else if needs_fresh_conn(&msg) {
                            let _ = $client.reconnect().await;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms(&msg)))
                            .await;
                        continue;
                    }
                    panic!("extrinsic failed after {} attempts: {}", __re_attempt, e);
                }
            }
        }
        assert!(__re_done, "retry_extrinsic: unreachable");
        __re_result
    }};
}

/// Retry an extrinsic that might fail, returning Ok(hash) or Err(msg).
/// Usage: try_extrinsic!(client, client.transfer(...))
macro_rules! try_extrinsic {
    ($client:expr, $call:expr) => {{
        let mut __te_result: Result<String, String> = Err("max retries".to_string());
        for __te_attempt in 1u32..=20 {
            match $call.await {
                Ok(hash) => {
                    __te_result = Ok(hash);
                    break;
                }
                Err(e) => {
                    let msg = format!("{}", e);
                    if is_retryable(&msg) && __te_attempt < 20 {
                        if is_conn_dead(&msg) {
                            ensure_alive($client).await;
                        } else if needs_fresh_conn(&msg) {
                            let _ = $client.reconnect().await;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms(&msg)))
                            .await;
                        continue;
                    }
                    __te_result = Err(msg);
                    break;
                }
            }
        }
        __te_result
    }};
}

/// Submit a sudo call via AdminUtils pallet using Sudo.sudo() wrapping.
/// Uses the checked variant that inspects the Sudid event for inner dispatch errors.
/// Alice must be the sudo key. Returns Ok(hash) or Err(message).
pub async fn sudo_admin_call(
    client: &mut Client,
    alice: &sr25519::Pair,
    call: &str,
    fields: Vec<subxt::dynamic::Value>,
) -> Result<String, String> {
    let mut result: Result<String, String> = Err("max retries".to_string());
    for attempt in 1u32..=8 {
        match client
            .submit_sudo_raw_call_checked(alice, "AdminUtils", call, fields.clone())
            .await
        {
            Ok(hash) => {
                result = Ok(hash);
                break;
            }
            Err(e) => {
                let msg = format!("{}", e);
                if is_retryable(&msg) && attempt < 8 {
                    if is_conn_dead(&msg) {
                        ensure_alive(client).await;
                    } else if needs_fresh_conn(&msg) {
                        let _ = client.reconnect().await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms(&msg)))
                        .await;
                    continue;
                }
                result = Err(msg);
                break;
            }
        }
    }
    result
}
// ──── 5b. Chain Setup (sudo config) ────

/// Configure a single subnet for testing — enable subtokens, disable commit-reveal,
/// zero out per-subnet rate limits. Uses sudo (Alice).
pub async fn setup_subnet(client: &mut Client, alice: &sr25519::Pair, sn: NetUid) {
    ensure_alive(client).await;
    use subxt::dynamic::Value;

    /// Reconnect client if dead, retry sudo call up to `max` times with wait between attempts.
    async fn robust_sudo(
        client: &mut Client,
        alice: &sr25519::Pair,
        call: &str,
        fields: Vec<subxt::dynamic::Value>,
        max: u32,
    ) -> Result<String, String> {
        for attempt in 1..=max {
            ensure_alive(client).await;
            let result = sudo_admin_call(client, alice, call, fields.clone()).await;
            match &result {
                Ok(_) => return result,
                Err(e)
                    if e.contains("dispatch failed")
                        || e.contains("WeightsWindow")
                        || e.contains("Prohibited")
                        || e.contains("connection")
                        || e.contains("closed")
                        || e.contains("restart")
                        || e.contains("outdated") =>
                {
                    if attempt <= 3 {
                        println!("    {} attempt {}/{}: {}", call, attempt, max, e);
                    }
                    wait_blocks(client, 2).await;
                    continue;
                }
                _ => return result,
            }
        }
        Err(format!("{call}: max retries exhausted"))
    }

    println!("── Setup SN{} ──", sn.0);

    // Enable registration on the subnet
    match robust_sudo(
        client,
        alice,
        "sudo_set_network_registration_allowed",
        vec![Value::u128(sn.0 as u128), Value::bool(true)],
        5,
    )
    .await
    {
        Ok(hash) => println!("  registration_allowed SN{}: {hash}", sn.0),
        Err(e) => println!("  [WARN] registration_allowed SN{}: {}", sn.0, e),
    }
    wait_blocks(client, 2).await;

    // Enable subtokens
    match robust_sudo(
        client,
        alice,
        "sudo_set_subtoken_enabled",
        vec![Value::u128(sn.0 as u128), Value::bool(true)],
        5,
    )
    .await
    {
        Ok(hash) => println!("  subtoken_enabled SN{}: {hash}", sn.0),
        Err(e) => println!("  [WARN] subtoken SN{}: {}", sn.0, e),
    }
    wait_blocks(client, 2).await;

    // Disable commit-reveal weights
    match robust_sudo(
        client,
        alice,
        "sudo_set_commit_reveal_weights_enabled",
        vec![Value::u128(sn.0 as u128), Value::bool(false)],
        5,
    )
    .await
    {
        Ok(hash) => println!("  commit-reveal off SN{}: {hash}", sn.0),
        Err(e) => println!("  [WARN] commit-reveal SN{}: {}", sn.0, e),
    }
    wait_blocks(client, 2).await;

    // Zero out per-subnet rate limits
    for (name, desc) in &[
        ("sudo_set_weights_set_rate_limit", "weights rate limit"),
        ("sudo_set_serving_rate_limit", "serving rate limit"),
    ] {
        match robust_sudo(
            client,
            alice,
            name,
            vec![Value::u128(sn.0 as u128), Value::u128(0)],
            5,
        )
        .await
        {
            Ok(hash) => println!("  zero {} SN{}: {hash}", desc, sn.0),
            Err(e) => println!("  [WARN] {} SN{}: {}", desc, sn.0, e),
        }
        wait_blocks(client, 2).await;
    }

    // Set max validators to 256 so all UIDs can have validator permits
    match robust_sudo(
        client,
        alice,
        "sudo_set_max_allowed_validators",
        vec![Value::u128(sn.0 as u128), Value::u128(256)],
        5,
    )
    .await
    {
        Ok(hash) => println!("  max validators 256 SN{}: {hash}", sn.0),
        Err(e) => println!("  [WARN] max validators SN{}: {}", sn.0, e),
    }
    wait_blocks(client, 2).await;

    // Raise target registrations per interval so snipe tests aren't rate-limited
    let _ = robust_sudo(
        client,
        alice,
        "sudo_set_target_registrations_per_interval",
        vec![Value::u128(sn.0 as u128), Value::u128(100)],
        5,
    )
    .await;
    wait_blocks(client, 2).await;

    // Lower difficulty to ease registration
    let _ = robust_sudo(
        client,
        alice,
        "sudo_set_difficulty",
        vec![Value::u128(sn.0 as u128), Value::u128(1)],
        5,
    )
    .await;
    wait_blocks(client, 2).await;

    // Set min burn for snipe guard test
    let _ = robust_sudo(
        client,
        alice,
        "sudo_set_min_burn",
        vec![Value::u128(sn.0 as u128), Value::u128(1_000_000_000)],
        5,
    )
    .await;

    wait_blocks(client, 2).await;
    println!("[PASS] setup SN{}", sn.0);
}

/// Set global (non-per-subnet) rate limits to zero.
pub async fn setup_global_rate_limits(client: &mut Client, alice: &sr25519::Pair) {
    ensure_alive(client).await;
    use subxt::dynamic::Value;

    println!("── Global rate limits ──");

    // Reconnect helper for a single sudo call with reconnect
    async fn robust_global_sudo(
        client: &mut Client,
        alice: &sr25519::Pair,
        call: &str,
        fields: Vec<subxt::dynamic::Value>,
    ) -> Result<String, String> {
        for attempt in 1..=5u32 {
            if !client.is_alive().await {
                for r in 1..=5u64 {
                    match client.reconnect().await {
                        Ok(()) => break,
                        Err(_) if r < 5 => tokio::time::sleep(Duration::from_millis(500 * r)).await,
                        Err(e) => return Err(format!("reconnect failed: {e}")),
                    }
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            let result = sudo_admin_call(client, alice, call, fields.clone()).await;
            match &result {
                Ok(_) => return result,
                Err(e)
                    if e.contains("connection")
                        || e.contains("closed")
                        || e.contains("restart")
                        || e.contains("outdated") =>
                {
                    if attempt <= 3 {
                        println!("    {} attempt {}/5: {}", call, attempt, e);
                    }
                    wait_blocks(client, 3).await;
                    continue;
                }
                _ => return result,
            }
        }
        Err(format!("{call}: max retries exhausted"))
    }

    match robust_global_sudo(
        client,
        alice,
        "sudo_set_tx_rate_limit",
        vec![Value::u128(0)],
    )
    .await
    {
        Ok(hash) => println!("  zero tx rate limit: {hash}"),
        Err(e) => println!("  [WARN] tx rate limit: {}", e),
    }

    wait_blocks(client, 2).await;

    match robust_global_sudo(
        client,
        alice,
        "sudo_set_tx_delegate_take_rate_limit",
        vec![Value::u128(0)],
    )
    .await
    {
        Ok(hash) => println!("  zero delegate take rate limit: {hash}"),
        Err(e) => println!("  [WARN] delegate take rate limit: {}", e),
    }

    wait_blocks(client, 2).await;
    println!("[PASS] global rate limits zeroed");
}
/// Same salt → `u16` encoding as `agcli weights reveal` (byte pairs, little-endian per pair).
pub fn salt_bytes_to_reveal_vec(salt: &[u8]) -> Vec<u16> {
    salt.chunks(2)
        .map(|chunk| {
            let b0 = chunk[0] as u16;
            let b1 = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
            (b1 << 8) | b0
        })
        .collect()
}

pub async fn sudo_set_commit_reveal_weights_or_fail(
    client: &mut Client,
    netuid: NetUid,
    enabled: bool,
) {
    let alice = dev_pair(ALICE_URI);
    use subxt::dynamic::Value;
    let mut ok = false;
    for attempt in 1..=20u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_commit_reveal_weights_enabled",
            vec![Value::u128(netuid.0 as u128), Value::bool(enabled)],
        )
        .await
        {
            Ok(hash) => {
                println!(
                    "  cr sudo {} SN{}: {hash}",
                    if enabled { "on" } else { "off" },
                    netuid.0
                );
                ok = true;
                break;
            }
            Err(e)
                if (e.contains("dispatch failed")
                    || e.contains("Module")
                    || e.contains("SymbolAlreadyInUse")
                    || is_retryable(&e))
                    && attempt < 20 =>
            {
                if attempt <= 3 {
                    println!("  cr sudo retry {attempt}: {e}");
                }
                wait_blocks(client, 10).await;
            }
            Err(e) => panic!(
                "sudo_set_commit_reveal_weights_enabled({enabled}) on SN{}: {e}",
                netuid.0
            ),
        }
    }
    assert!(
        ok,
        "could not set commit-reveal={enabled} on SN{} after 20 attempts",
        netuid.0
    );
    wait_blocks(client, 3).await;
}
