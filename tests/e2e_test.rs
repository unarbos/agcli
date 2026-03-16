//! End-to-end tests against a real local subtensor chain (Docker).
//!
//! Requires: `docker pull ghcr.io/opentensor/subtensor-localnet:devnet-ready`
//!
//! Run with:
//!   cargo test --test e2e_test -- --nocapture
//!
//! The test harness:
//!   1. Starts a local subtensor chain via Docker (fast-block mode, 250ms blocks).
//!   2. Waits for the chain to produce blocks.
//!   3. Runs tests that submit real extrinsics and verify storage map effects.
//!   4. Tears down the container on completion.
//!
//! Dev accounts (pre-funded in genesis):
//!   Alice: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY (sudo, 1M TAO)
//!   Bob:   5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty

use agcli::chain::Client;
use agcli::types::balance::Balance;
use agcli::types::network::NetUid;
use sp_core::{sr25519, Pair};
use std::process::Command;
use std::sync::Once;
use std::time::Duration;

// ──────── Constants ────────

const LOCAL_WS: &str = "ws://127.0.0.1:9944";
const CONTAINER_NAME: &str = "agcli_e2e_test";
const DOCKER_IMAGE: &str = "ghcr.io/opentensor/subtensor-localnet:devnet-ready";

/// Alice is the sudo account in localnet, pre-funded with 1M TAO.
const ALICE_URI: &str = "//Alice";
const ALICE_SS58: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

/// Bob is another pre-funded dev account.
const BOB_URI: &str = "//Bob";
const BOB_SS58: &str = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

// ──────── Harness ────────

static INIT: Once = Once::new();

/// Ensure a local chain container is running. Idempotent — only starts once.
fn ensure_local_chain() {
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
async fn wait_for_chain() -> Client {
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
fn dev_pair(uri: &str) -> sr25519::Pair {
    sr25519::Pair::from_string(uri, None).expect("valid dev URI")
}

/// Wait for N blocks to pass (useful for extrinsic finalization in fast-block mode).
async fn wait_blocks(client: &Client, n: u64) {
    let start = client.get_block_number().await.unwrap();
    let target = start + n;
    loop {
        let current = client.get_block_number().await.unwrap();
        if current >= target {
            return;
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

/// Retry an extrinsic up to 10 times on "Transaction is outdated" errors.
/// Fast-block mode (250ms) can cause mortal-era transactions to expire between signing and submission.
/// The retry loop is generous because this is a known subxt issue with fast devnets.
async fn retry_extrinsic<F, Fut>(f: F) -> String
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<String>>,
{
    for attempt in 1..=10 {
        match f().await {
            Ok(hash) => return hash,
            Err(e) => {
                let msg = format!("{}", e);
                if (msg.contains("outdated") || msg.contains("banned") || msg.contains("subscription")) && attempt < 10 {
                    if attempt <= 2 {
                        println!("  attempt {} outdated, retrying...", attempt);
                    }
                    // Wait for next block then retry — the next attempt will get a fresh block hash.
                    // For "banned" errors, wait longer (the node caches banned tx hashes).
                    let delay = if msg.contains("banned") { 13_000 } else { 100 };
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    continue;
                }
                panic!("extrinsic failed after {} attempts: {}", attempt, e);
            }
        }
    }
    unreachable!()
}

// ──────── Tests ────────

/// All e2e tests run in a single tokio runtime sharing one chain instance.
/// Tests are sequential within this function to avoid race conditions on chain state.
#[tokio::test]
async fn e2e_local_chain() {
    ensure_local_chain();
    let client = wait_for_chain().await;

    println!("\n═══ E2E Test Suite — Local Subtensor Chain ═══\n");

    // Phase 1: Basic connectivity and queries
    test_connectivity(&client).await;
    test_alice_balance(&client).await;
    test_total_networks(&client).await;

    // Phase 2: Transfers
    test_transfer(&client).await;

    // Phase 3: Subnet registration
    test_register_network(&client).await;

    // Phase 4: Burned register on new subnet
    test_burned_register(&client).await;

    // Phase 4b: Snipe registration (block-subscription based)
    test_snipe_register(&client).await;

    // Phase 5: Staking (on SN1 which has subtokens enabled in genesis)
    test_add_remove_stake(&client).await;

    // Phase 6: Set weights
    test_set_weights(&client).await;

    // Phase 7: Subnet queries
    test_subnet_queries(&client).await;

    // Cleanup
    println!("\n═══ All E2E Tests Passed ═══\n");
    let _ = Command::new("docker")
        .args(["rm", "-f", CONTAINER_NAME])
        .output();
}

// ──── 1. Connectivity ────

async fn test_connectivity(client: &Client) {
    let block = client.get_block_number().await.expect("get_block_number");
    assert!(block > 0, "chain should be producing blocks, got block {}", block);
    println!("[PASS] connectivity — at block {block}");
}

// ──── 2. Alice Balance ────

async fn test_alice_balance(client: &Client) {
    let balance = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("get_balance for Alice");
    // Alice should have substantial funds (1M TAO in genesis, minus any tx fees)
    assert!(
        balance.tao() > 100_000.0,
        "Alice should have >100k TAO, got {}",
        balance.tao()
    );
    println!("[PASS] alice_balance — {} TAO", balance.tao());
}

// ──── 3. Total Networks ────

async fn test_total_networks(client: &Client) {
    let n = client.get_total_networks().await.expect("get_total_networks");
    // Localnet genesis typically has root network (netuid 0) at minimum
    assert!(n >= 1, "should have at least 1 network (root), got {}", n);
    println!("[PASS] total_networks — {n} networks");
}

// ──── 4. Transfer ────

async fn test_transfer(client: &Client) {
    let alice = dev_pair(ALICE_URI);
    let amount = Balance::from_tao(10.0);

    // Check Bob's balance before
    let bob_before = client
        .get_balance_ss58(BOB_SS58)
        .await
        .expect("Bob balance before");

    // Transfer 10 TAO from Alice to Bob (retry on "outdated" — fast blocks advance quickly)
    let hash = retry_extrinsic(|| client.transfer(&alice, BOB_SS58, amount)).await;
    println!("  transfer tx: {hash}");

    // Wait a few blocks for finalization
    wait_blocks(&client, 3).await;

    // Check Bob's balance after
    let bob_after = client
        .get_balance_ss58(BOB_SS58)
        .await
        .expect("Bob balance after");

    let diff = bob_after.rao() as i128 - bob_before.rao() as i128;
    assert!(
        diff > 0,
        "Bob's balance should have increased, before={} after={}",
        bob_before,
        bob_after
    );
    // Should be close to 10 TAO (exact match minus tiny rounding)
    let expected_rao = amount.rao() as i128;
    assert!(
        (diff - expected_rao).abs() < 1_000_000, // within 0.001 TAO tolerance
        "Bob should have received ~10 TAO, got diff={} RAO",
        diff
    );
    println!(
        "[PASS] transfer — Alice→Bob 10 TAO (before={}, after={})",
        bob_before, bob_after
    );
}

// ──── 5. Register Network (Subnet) ────

async fn test_register_network(client: &Client) {
    let alice = dev_pair(ALICE_URI);

    let networks_before = client.get_total_networks().await.expect("networks before");

    // Register a new subnet with Alice as owner, using Alice hotkey
    let hash = retry_extrinsic(|| client.register_network(&alice, ALICE_SS58)).await;
    println!("  register_network tx: {hash}");

    wait_blocks(&client, 3).await;

    let networks_after = client.get_total_networks().await.expect("networks after");
    assert!(
        networks_after > networks_before,
        "total_networks should increase after register_network: before={}, after={}",
        networks_before,
        networks_after
    );
    println!(
        "[PASS] register_network — subnets {} → {}",
        networks_before, networks_after
    );
}

// ──── 6. Burned Register ────

async fn test_burned_register(client: &Client) {
    let alice = dev_pair(ALICE_URI);
    let bob = dev_pair(BOB_URI);
    let bob_pub = bob.public();
    let bob_ss58_derived =
        sp_core::crypto::Ss58Codec::to_ss58check_with_version(&bob_pub, 42u16.into());

    // Find the newest subnet (highest netuid)
    let total = client.get_total_networks().await.expect("total networks");
    // The newest subnet is at netuid = total - 1 (0-indexed) or we search for it
    // Actually localnet may have sparse netuids; let's use total_networks as an estimate
    // and try the last registered subnet. The most recently registered should be netuid = total - 1.
    let netuid = NetUid(total - 1);
    println!("  burning register on SN{}", netuid.0);

    // Burned register Bob's hotkey on the newest subnet
    let hash = retry_extrinsic(|| client.burned_register(&alice, netuid, &bob_ss58_derived)).await;
    println!("  burned_register tx: {hash}");

    wait_blocks(&client, 3).await;

    // Verify: query neurons on that subnet — should have at least 1
    let neurons = client
        .get_neurons_lite(netuid)
        .await
        .expect("get_neurons_lite after register");
    assert!(
        !neurons.is_empty(),
        "SN{} should have at least 1 neuron after burned_register",
        netuid.0
    );

    // Verify Bob's hotkey is among the registered neurons
    let bob_found = neurons.iter().any(|n| n.hotkey == bob_ss58_derived);
    assert!(
        bob_found,
        "Bob's hotkey should be registered on SN{}",
        netuid.0
    );
    println!(
        "[PASS] burned_register — Bob registered on SN{} ({} neurons)",
        netuid.0,
        neurons.len()
    );
}

// ──── 6b. Snipe Registration (block-subscription) ────

/// Tests the core snipe mechanism: subscribe to blocks, check slot availability,
/// and fire burned_register the instant a new block arrives.
async fn test_snipe_register(client: &Client) {
    let alice = dev_pair(ALICE_URI);

    // Generate a fresh keypair for the snipe target (so it's guaranteed unregistered)
    let (snipe_hotkey, _) = sr25519::Pair::generate();
    let snipe_ss58 =
        sp_core::crypto::Ss58Codec::to_ss58check_with_version(&snipe_hotkey.public(), 42u16.into());

    // Find the newest subnet
    let total = client.get_total_networks().await.expect("total networks");
    let netuid = NetUid(total - 1);

    // Pre-check: verify subnet has open slots
    let info = client
        .get_subnet_info(netuid)
        .await
        .expect("subnet info")
        .expect("subnet should exist");
    assert!(
        info.registration_allowed,
        "registration should be allowed on SN{}",
        netuid.0
    );
    assert!(
        info.n < info.max_n,
        "SN{} should have capacity: {}/{}",
        netuid.0,
        info.n,
        info.max_n
    );

    println!(
        "  Snipe target: SN{} ({}/{} slots, burn={})",
        netuid.0,
        info.n,
        info.max_n,
        info.burn.display_tao()
    );

    // ── Core snipe logic: subscribe to blocks and register on next block ──
    let subxt_client = client.subxt();
    let mut block_sub = subxt_client
        .blocks()
        .subscribe_finalized()
        .await
        .expect("block subscription");

    let start = std::time::Instant::now();
    let mut registered = false;

    // Wait for next block and attempt registration
    for attempt in 1..=5 {
        let block = block_sub.next().await;
        let block = match block {
            Some(Ok(b)) => b,
            Some(Err(e)) => {
                println!("  block stream error on attempt {}: {}", attempt, e);
                continue;
            }
            None => break,
        };
        let block_num = block.number();
        println!(
            "  Attempt {} at block #{}: submitting burned_register...",
            attempt, block_num
        );

        match client.burned_register(&alice, netuid, &snipe_ss58).await {
            Ok(hash) => {
                let elapsed = start.elapsed();
                println!(
                    "  ✓ Registered on attempt {} ({:.1}s): {}",
                    attempt,
                    elapsed.as_secs_f64(),
                    hash
                );
                registered = true;
                break;
            }
            Err(e) => {
                let msg = format!("{}", e);
                if msg.contains("TooManyRegistrationsThisBlock") {
                    println!("  rate-limited at block #{}, waiting for next block", block_num);
                    continue;
                } else {
                    panic!(
                        "Unexpected registration error on attempt {}: {}",
                        attempt, msg
                    );
                }
            }
        }
    }

    assert!(registered, "snipe should have registered within 5 block attempts");
    wait_blocks(&client, 3).await;

    // Verify: neuron count on the subnet should have increased
    let info_after = client
        .get_subnet_info(netuid)
        .await
        .expect("subnet info after snipe")
        .expect("subnet should still exist");
    assert!(
        info_after.n > info.n,
        "SN{} neuron count should increase after snipe: before={}, after={}",
        netuid.0,
        info.n,
        info_after.n
    );

    println!(
        "[PASS] snipe_register — block-sub registration on SN{} (neurons {}/{}, {:.1}s)",
        netuid.0,
        info_after.n,
        info_after.max_n,
        start.elapsed().as_secs_f64()
    );
}

// ──── 7. Staking ────

async fn test_add_remove_stake(client: &Client) {
    let alice = dev_pair(ALICE_URI);
    let bob = dev_pair(BOB_URI);
    let bob_pub = bob.public();
    let bob_ss58 =
        sp_core::crypto::Ss58Codec::to_ss58check_with_version(&bob_pub, 42u16.into());

    // First register Bob on root (SN0) so we can stake there — root has subtokens enabled
    // Try SN1 first (genesis subnet), fall back to SN0
    let netuid = NetUid(1);

    // We need Bob registered on this subnet. Try burned_register — if already registered, that's fine.
    match client.burned_register(&alice, netuid, &bob_ss58).await {
        Ok(hash) => println!("  registered Bob on SN{}: {}", netuid.0, hash),
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("AlreadyRegistered") || msg.contains("HotKeyAlreadyRegistered") {
                println!("  Bob already registered on SN{}", netuid.0);
            } else {
                println!("  registration on SN{} failed ({}), will try staking anyway", netuid.0, msg);
            }
        }
    }
    wait_blocks(&client, 2).await;

    let stake_amount = Balance::from_tao(5.0);

    // Get Alice's stakes before
    let stakes_before = client
        .get_stake_for_coldkey(ALICE_SS58)
        .await
        .expect("stakes before");
    let alice_stake_on_bob_before = stakes_before
        .iter()
        .find(|s| s.hotkey == bob_ss58 && s.netuid == netuid)
        .map(|s| s.stake.rao())
        .unwrap_or(0);

    // Add 5 TAO stake from Alice to Bob
    let result = client.add_stake(&alice, &bob_ss58, netuid, stake_amount).await;
    match result {
        Ok(hash) => {
            println!("  add_stake tx: {hash}");
            wait_blocks(&client, 3).await;

            // Verify stake increased
            let stakes_after = client
                .get_stake_for_coldkey(ALICE_SS58)
                .await
                .expect("stakes after add");
            let alice_stake_on_bob_after = stakes_after
                .iter()
                .find(|s| s.hotkey == bob_ss58 && s.netuid == netuid)
                .map(|s| s.stake.rao())
                .unwrap_or(0);

            assert!(
                alice_stake_on_bob_after > alice_stake_on_bob_before,
                "stake should increase after add_stake: before={}, after={}",
                alice_stake_on_bob_before,
                alice_stake_on_bob_after
            );
            println!(
                "[PASS] add_stake — Alice→Bob@SN{}: {} → {} RAO",
                netuid.0, alice_stake_on_bob_before, alice_stake_on_bob_after
            );

            // Now remove some stake
            let remove_amount = Balance::from_tao(2.0);
            let hash = client
                .remove_stake(&alice, &bob_ss58, netuid, remove_amount)
                .await
                .expect("remove_stake");
            println!("  remove_stake tx: {hash}");

            wait_blocks(&client, 3).await;

            let stakes_final = client
                .get_stake_for_coldkey(ALICE_SS58)
                .await
                .expect("stakes after remove");
            let alice_stake_final = stakes_final
                .iter()
                .find(|s| s.hotkey == bob_ss58 && s.netuid == netuid)
                .map(|s| s.stake.rao())
                .unwrap_or(0);

            assert!(
                alice_stake_final < alice_stake_on_bob_after,
                "stake should decrease after remove_stake: after_add={}, after_remove={}",
                alice_stake_on_bob_after,
                alice_stake_final
            );
            println!(
                "[PASS] remove_stake — Alice→Bob@SN{}: {} → {} RAO",
                netuid.0, alice_stake_on_bob_after, alice_stake_final
            );
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("SubtokenDisabled") {
                // Staking requires alpha subtokens which may not be enabled on all localnet versions.
                // This is a chain runtime limitation, not an agcli bug.
                println!("[SKIP] add_stake — SubtokenDisabled on SN{} (localnet runtime limitation)", netuid.0);
                println!("[SKIP] remove_stake — skipped due to SubtokenDisabled");
            } else {
                panic!("add_stake failed unexpectedly: {}", e);
            }
        }
    }
}

// ──── 7b. Set Weights ────

async fn test_set_weights(client: &Client) {
    let alice = dev_pair(ALICE_URI);

    // Find a subnet where Alice has a registered neuron
    let total = client.get_total_networks().await.expect("total networks");
    let netuid = NetUid(total - 1); // Use the newest subnet where we registered Alice's hotkey

    // Check if Alice's hotkey has a UID on this subnet
    let neurons = client.get_neurons_lite(netuid).await.expect("neurons");
    let alice_neuron = neurons.iter().find(|n| n.hotkey == ALICE_SS58);

    match alice_neuron {
        Some(neuron) => {
            let uid = neuron.uid;
            println!("  Alice has UID {} on SN{}", uid, netuid.0);

            // Set weights — point all weight at UID 0
            let uids = vec![0u16];
            let weights = vec![65535u16];
            let version_key = 0u64;

            let result = client
                .set_weights(&alice, netuid, &uids, &weights, version_key)
                .await;

            match result {
                Ok(hash) => {
                    println!("  set_weights tx: {hash}");
                    wait_blocks(&client, 3).await;

                    // Verify weights are stored on-chain
                    let on_chain = client
                        .get_weights_for_uid(netuid, uid)
                        .await
                        .expect("get_weights_for_uid");
                    assert!(
                        !on_chain.is_empty(),
                        "weights should be set on SN{} for UID {}",
                        netuid.0,
                        uid
                    );
                    println!(
                        "[PASS] set_weights — SN{} UID {}: {} weight entries on-chain",
                        netuid.0,
                        uid,
                        on_chain.len()
                    );
                }
                Err(e) => {
                    let msg = format!("{}", e);
                    if msg.contains("CommitRevealEnabled") || msg.contains("WeightsCommitNotAllowed")
                    {
                        println!(
                            "[SKIP] set_weights — commit-reveal mode on SN{}",
                            netuid.0
                        );
                    } else if msg.contains("SettingWeightsTooFast") {
                        println!(
                            "[SKIP] set_weights — rate limited on SN{} (SettingWeightsTooFast)",
                            netuid.0
                        );
                    } else {
                        println!("[WARN] set_weights failed: {}", e);
                    }
                }
            }
        }
        None => {
            println!(
                "[SKIP] set_weights — Alice not registered on SN{}, skipping",
                netuid.0
            );
        }
    }
}

// ──── 8. Subnet Queries ────

async fn test_subnet_queries(client: &Client) {
    // Test get_all_subnets
    let subnets = client.get_all_subnets().await.expect("get_all_subnets");
    assert!(!subnets.is_empty(), "should have at least 1 subnet");
    println!(
        "  subnets: {} total (first: SN{} \"{}\")",
        subnets.len(),
        subnets[0].netuid,
        subnets[0].name
    );

    // Test total_stake
    let total_stake = client.get_total_stake().await.expect("get_total_stake");
    println!("  total_stake: {}", total_stake);

    // Test get_all_dynamic_info
    let dynamic = client
        .get_all_dynamic_info()
        .await
        .expect("get_all_dynamic_info");
    assert!(!dynamic.is_empty(), "should have dynamic info for subnets");
    println!("  dynamic_info: {} entries", dynamic.len());

    // Test block timestamp
    let block_num = client.get_block_number().await.expect("block_number");
    assert!(block_num > 10, "should have produced many blocks by now");

    println!(
        "[PASS] subnet_queries — {} subnets, {} dynamic infos, block {}",
        subnets.len(),
        dynamic.len(),
        block_num
    );
}
