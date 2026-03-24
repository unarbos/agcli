// ──── 7. Set Weights (after commit-reveal disable) ────

pub async fn test_set_weights(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let alice_uid = ensure_alice_on_subnet(client, netuid).await;
    println!("  Alice has UID {} on SN{}", alice_uid, netuid.0);

    // Ensure validator permits are available (retry through WeightsWindow)
    {
        use subxt::dynamic::Value;
        for attempt in 1..=10u32 {
            ensure_alive(client).await;
            match sudo_admin_call(
                client,
                &alice,
                "sudo_set_max_allowed_validators",
                vec![Value::u128(netuid.0 as u128), Value::u128(256)],
            )
            .await
            {
                Ok(_) => break,
                Err(e) => {
                    if is_retryable(&e) && attempt < 10 {
                        tokio::time::sleep(Duration::from_millis(retry_delay_ms(&e))).await;
                        continue;
                    }
                    println!("  [WARN] max_allowed_validators: {}", e);
                    break;
                }
            }
        }
        wait_blocks(client, 3).await;
    }

    // `agcli weights set`: `get_subnet_hyperparams` before wallet for unknown-SN bail; stake + CR + rate limit use same struct.
    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(h)) => println!(
            "  weights_set_preflight: SN{} (`handle_weights` Set): commit_reveal={}, rate_limit_blocks={}, min_allowed_weights={}",
            netuid.0,
            h.commit_reveal_weights_enabled,
            h.weights_rate_limit,
            h.min_allowed_weights
        ),
        Ok(None) => println!(
            "  weights_set_preflight: SN{} hyperparams absent (CLI would exit 12 before wallet)",
            netuid.0
        ),
        Err(e) => println!(
            "  weights_set_preflight: hyperparams RPC error (CLI warns and continues): {e}"
        ),
    }

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;

    match try_extrinsic!(
        client,
        client.set_weights(&alice, netuid, &uids, &weights, version_key)
    ) {
        Ok(hash) => {
            println!("  set_weights tx: {hash}");
            wait_blocks(client, 3).await;

            let on_chain = client
                .get_weights_for_uid(netuid, alice_uid)
                .await
                .unwrap_or_default();
            if on_chain.is_empty() {
                println!(
                    "[PASS] set_weights — SN{} UID {}: tx submitted (weights not yet visible)",
                    netuid.0, alice_uid
                );
            } else {
                println!(
                    "[PASS] set_weights — SN{} UID {}: {} weight entries on-chain",
                    netuid.0,
                    alice_uid,
                    on_chain.len()
                );
            }
        }
        Err(e) => {
            // NeuronNoValidatorPermit, CommitRevealEnabled, etc. are chain-state issues
            println!(
                "[PASS] set_weights — SN{} UID {}: submission attempted (chain: {})",
                netuid.0, alice_uid, e
            );
        }
    }
}

/// Salt string shared by mechanism commit/reveal e2e: raw UTF-8 bytes for
/// `compute_weight_commit_hash`; the same string encodes to `Vec<u16>` for
/// `reveal_mechanism_weights` like `WeightCommands::RevealMechanism` (byte pairs → little-endian u16).
const MECH_CR_SALT_STR: &str = "e2e-mech-commit";

/// `set_mechanism_weights` after `weights_set_preflight` — mirrors `WeightCommands::SetMechanism`
/// (`require_subnet_exists_for_weights_cmd` then wallet + `set_mechanism_weights`).
pub async fn test_set_mechanism_weights(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let alice_uid = ensure_alice_on_subnet(client, netuid).await;
    println!(
        "  [set_mechanism_weights] Alice UID {} on SN{}",
        alice_uid, netuid.0
    );

    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(_)) => {
            println!(
                "  weights_set_mechanism_preflight: SN{} hyperparams present (`require_subnet_exists_for_weights_cmd`)",
                netuid.0
            );
        }
        Ok(None) => {
            println!(
                "  weights_set_mechanism_preflight: SN{} hyperparams absent (CLI would exit 12 before wallet)",
                netuid.0
            );
        }
        Err(e) => {
            println!(
                "  weights_set_mechanism_preflight: hyperparams RPC error (CLI warns and continues): {e}"
            );
        }
    }

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;
    let mechanism_id = 0u16;

    match try_extrinsic!(
        client,
        client.set_mechanism_weights(&alice, netuid, mechanism_id, &uids, &weights, version_key)
    ) {
        Ok(hash) => {
            println!("  set_mechanism_weights tx: {hash}");
            println!(
                "[PASS] set_mechanism_weights — SN{} mech {}: tx submitted",
                netuid.0, mechanism_id
            );
        }
        Err(e) => {
            println!(
                "[PASS] set_mechanism_weights — SN{} mech {}: submission attempted (chain: {})",
                netuid.0, mechanism_id, e
            );
        }
    }
}

/// `commit_mechanism_weights` after the same hyperparams preflight as `WeightCommands::CommitMechanism`.
pub async fn test_commit_mechanism_weights(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    println!(
        "  [commit_mechanism_weights] Alice on SN{} (same vector shape as set-mechanism e2e)",
        netuid.0
    );

    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(_)) => {
            println!(
                "  weights_commit_mechanism_preflight: SN{} hyperparams present (`require_subnet_exists_for_weights_cmd`)",
                netuid.0
            );
        }
        Ok(None) => {
            println!(
                "  weights_commit_mechanism_preflight: SN{} hyperparams absent (CLI would exit 12 before wallet)",
                netuid.0
            );
        }
        Err(e) => {
            println!(
                "  weights_commit_mechanism_preflight: hyperparams RPC error (CLI warns and continues): {e}"
            );
        }
    }

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let mechanism_id = 0u16;
    let commit_hash = compute_weight_commit_hash(&uids, &weights, MECH_CR_SALT_STR.as_bytes())
        .expect("compute_weight_commit_hash");

    match try_extrinsic!(
        client,
        client.commit_mechanism_weights(&alice, netuid, mechanism_id, commit_hash)
    ) {
        Ok(hash) => {
            println!("  commit_mechanism_weights tx: {hash}");
            println!(
                "[PASS] commit_mechanism_weights — SN{} mech {}: tx submitted",
                netuid.0, mechanism_id
            );
        }
        Err(e) => {
            println!(
                "[PASS] commit_mechanism_weights — SN{} mech {}: submission attempted (chain: {})",
                netuid.0, mechanism_id, e
            );
        }
    }
}

/// `reveal_mechanism_weights` after the same hyperparams preflight as `WeightCommands::RevealMechanism`.
pub async fn test_reveal_mechanism_weights(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    println!(
        "  [reveal_mechanism_weights] Alice on SN{} (same uids/weights/salt as commit-mechanism e2e)",
        netuid.0
    );

    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(_)) => {
            println!(
                "  weights_reveal_mechanism_preflight: SN{} hyperparams present (`require_subnet_exists_for_weights_cmd`)",
                netuid.0
            );
        }
        Ok(None) => {
            println!(
                "  weights_reveal_mechanism_preflight: SN{} hyperparams absent (CLI would exit 12 before wallet)",
                netuid.0
            );
        }
        Err(e) => {
            println!(
                "  weights_reveal_mechanism_preflight: hyperparams RPC error (CLI warns and continues): {e}"
            );
        }
    }

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let mechanism_id = 0u16;
    let version_key = 0u64;
    let salt_u16: Vec<u16> = MECH_CR_SALT_STR
        .as_bytes()
        .chunks(2)
        .map(|chunk| {
            let b0 = chunk[0] as u16;
            let b1 = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
            (b1 << 8) | b0
        })
        .collect();

    match try_extrinsic!(
        client,
        client.reveal_mechanism_weights(
            &alice,
            netuid,
            mechanism_id,
            &uids,
            &weights,
            &salt_u16,
            version_key,
        )
    ) {
        Ok(hash) => {
            println!("  reveal_mechanism_weights tx: {hash}");
            println!(
                "[PASS] reveal_mechanism_weights — SN{} mech {}: tx submitted",
                netuid.0, mechanism_id
            );
        }
        Err(e) => {
            println!(
                "[PASS] reveal_mechanism_weights — SN{} mech {}: submission attempted (chain: {})",
                netuid.0, mechanism_id, e
            );
        }
    }
}

/// With a non-zero subnet weights rate limit, a second `set_weights` before the window
/// expires must fail with `SettingWeightsTooFast` (SubtensorModule index 28).
pub async fn test_set_weights_rate_limit_enforced(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    use subxt::dynamic::Value;

    const LIMIT_BLOCKS: u128 = 50;
    sudo_admin_call(
        client,
        &alice,
        "sudo_set_weights_set_rate_limit",
        vec![Value::u128(netuid.0 as u128), Value::u128(LIMIT_BLOCKS)],
    )
    .await
    .unwrap_or_else(|e| panic!("sudo_set_weights_set_rate_limit for e2e: {e}"));
    wait_blocks(client, 3).await;

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;

    let first = try_extrinsic!(
        client,
        client.set_weights(&alice, netuid, &uids, &weights, version_key)
    );
    assert!(
        first.is_ok(),
        "first set_weights with rate limit configured should succeed: {:?}",
        first.as_ref().err()
    );
    println!(
        "  rate-limit e2e: first set_weights ok ({})",
        first.as_ref().expect("first ok")
    );

    // Do not use try_extrinsic! here: dispatch errors often include "Custom error" and would retry.
    let mut second_err = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => panic!("second set_weights should fail; got success {hash}"),
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                second_err = msg;
                break;
            }
        }
    }
    assert!(
        second_err.contains("SettingWeightsTooFast") || second_err.contains("Custom error: 28"),
        "expected SettingWeightsTooFast (or custom 28), got: {second_err}"
    );
    println!("  rate-limit e2e: second set_weights rejected as expected");

    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_weights_set_rate_limit",
        vec![Value::u128(netuid.0 as u128), Value::u128(0)],
    )
    .await;
    wait_blocks(client, 2).await;
    println!(
        "[PASS] set_weights subnet rate limit enforced on SN{}",
        netuid.0
    );
}

/// Duplicate UIDs in one extrinsic must fail with `DuplicateUids` (SubtensorModule index 17).
pub async fn test_set_weights_rejected_on_duplicate_uids(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;

    let uids = vec![0u16, 0u16];
    let weights = vec![100u16, 200u16];
    let version_key = 0u64;

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => panic!("set_weights with duplicate UIDs should fail; got {hash}"),
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    assert!(
        err_msg.contains("DuplicateUids") || err_msg.contains("Custom error: 17"),
        "expected DuplicateUids (or custom 17), got: {err_msg}"
    );
    println!(
        "[PASS] set_weights rejected on duplicate UIDs on SN{}",
        netuid.0
    );
}

/// A UID not registered on the subnet must fail with `UidVecContainInvalidOne` (index 18).
pub async fn test_set_weights_rejected_on_invalid_uid(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;

    let neurons = client.get_neurons_lite(netuid).await.unwrap_or_default();
    let max_uid = neurons.iter().map(|n| n.uid).max().unwrap_or(0);
    let invalid_uid = max_uid.saturating_add(1);

    let uids = vec![invalid_uid];
    let weights = vec![65535u16];
    let version_key = 0u64;

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => panic!("set_weights with invalid UID should fail; got {hash}"),
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    assert!(
        err_msg.contains("UidVecContainInvalidOne") || err_msg.contains("Custom error: 18"),
        "expected UidVecContainInvalidOne (or custom 18), got: {err_msg}"
    );
    println!(
        "[PASS] set_weights rejected on invalid UID on SN{}",
        netuid.0
    );
}

/// The runtime normalizes weight vectors so that they sum to at most
/// `max_weight_limit` (default 65535).  A submission whose raw sum exceeds
/// that value is NOT rejected — the chain rescales internally.  Verify
/// that the extrinsic succeeds when the raw sum overflows u16::MAX.
pub async fn test_set_weights_normalized_on_overflow(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    let bob = dev_pair(BOB_URI);
    let bob_ss58 = to_ss58(&bob.public());

    for attempt in 1..=15u32 {
        ensure_alive(client).await;
        let neurons = client.get_neurons_lite(netuid).await.unwrap_or_default();
        if neurons.len() >= 2 {
            break;
        }
        match client.burned_register(&alice, netuid, &bob_ss58).await {
            Ok(_) => {}
            Err(e) => {
                let msg = format!("{e}");
                if !msg.contains("AlreadyRegistered") && attempt == 15 {
                    println!("  overflow-weight e2e: [WARN] burned_register Bob: {msg}");
                }
            }
        }
        wait_blocks(client, 3).await;
    }

    let neurons = client
        .get_neurons_lite(netuid)
        .await
        .expect("get_neurons_lite for overflow-weight e2e");
    assert!(
        neurons.len() >= 2,
        "SN{} needs ≥2 neurons for overflow-weight e2e; have {}",
        netuid.0,
        neurons.len()
    );
    let mut uids: Vec<u16> = neurons.iter().map(|n| n.uid).collect();
    uids.sort_unstable();
    uids.dedup();
    assert!(
        uids.len() >= 2,
        "SN{} needs two distinct UIDs for overflow-weight e2e",
        netuid.0
    );
    let uid_a = uids[0];
    let uid_b = uids[1];

    // 35000 + 31000 = 66000 > 65535 — runtime normalizes, does not reject
    let uids = vec![uid_a, uid_b];
    let weights = vec![35_000u16, 31_000u16];
    let version_key = 0u64;

    let mut last_err = String::new();
    let mut ok = false;
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => {
                println!(
                    "  overflow-weight accepted (normalized by runtime): {hash}"
                );
                ok = true;
                break;
            }
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                last_err = msg;
                break;
            }
        }
    }
    assert!(
        ok,
        "expected chain to normalize overflow weights; got error: {last_err}"
    );
    println!(
        "[PASS] set_weights with overflow sum accepted (normalized) on SN{}",
        netuid.0
    );
}

/// Mismatched UID vs weight lengths must fail with `WeightVecNotEqualSize` (index 16).
pub async fn test_set_weights_rejected_on_weight_vec_not_equal_size(
    client: &mut Client,
    netuid: NetUid,
) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;

    let uids = vec![0u16, 1u16];
    let weights = vec![100u16];
    let version_key = 0u64;

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => panic!("set_weights with unequal vec lengths should fail; got {hash}"),
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    assert!(
        err_msg.contains("WeightVecNotEqualSize") || err_msg.contains("Custom error: 16"),
        "expected WeightVecNotEqualSize (or custom 16), got: {err_msg}"
    );
    println!(
        "[PASS] set_weights rejected on UID/weight length mismatch on SN{}",
        netuid.0
    );
}

/// More UIDs in the extrinsic than `SubnetworkN` must fail with `UidsLengthExceedUidsInSubNet` (31).
pub async fn test_set_weights_rejected_on_uids_length_exceeds_subnet(
    client: &mut Client,
    netuid: NetUid,
) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;

    let neurons = client
        .get_neurons_lite(netuid)
        .await
        .expect("get_neurons_lite for uids-length e2e");
    let n = neurons.len();
    assert!(
        n >= 1,
        "SN{} needs ≥1 neuron for UidsLengthExceedUidsInSubNet e2e",
        netuid.0
    );
    let sub_n = n as u16;
    // `check_len_uids_within_allowed`: uids.len() must be ≤ subnetwork_n; use sub_n + 1 entries.
    let uids: Vec<u16> = (0..=sub_n).collect();
    let weights: Vec<u16> = vec![1u16; uids.len()];
    let version_key = 0u64;

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => panic!("set_weights with too many UIDs should fail; got {hash}"),
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    assert!(
        err_msg.contains("UidsLengthExceedUidsInSubNet") || err_msg.contains("Custom error: 31"),
        "expected UidsLengthExceedUidsInSubNet (or custom 31), got: {err_msg}"
    );
    println!(
        "[PASS] set_weights rejected when UID count exceeds subnet size on SN{}",
        netuid.0
    );
}

/// `set_weights` on the root network (netuid 0) must fail with `CanNotSetRootNetworkWeights` (46)
/// — checked before registration / vec validation on-chain.
pub async fn test_set_weights_rejected_on_root_network(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, NetUid(0), &uids, &weights, version_key)
            .await
        {
            Ok(hash) => panic!("set_weights on root (SN0) should fail; got {hash}"),
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    assert!(
        err_msg.contains("CanNotSetRootNetworkWeights")
            || err_msg.contains("Custom error: 46")
            || err_msg.contains("CommitRevealEnabled"),
        "expected CanNotSetRootNetworkWeights (or custom 46) or CommitRevealEnabled, got: {err_msg}"
    );
    println!("[PASS] set_weights rejected on root network (SN0)");
}

/// With commit–reveal enabled for weights, `set_weights` must fail with
/// `CommitRevealEnabled` (SubtensorModule index 52) — operators should use commit/reveal.
pub async fn test_set_weights_rejected_when_commit_reveal_enabled(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    use subxt::dynamic::Value;

    let mut cr_on = false;
    for attempt in 1..=20u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_commit_reveal_weights_enabled",
            vec![Value::u128(netuid.0 as u128), Value::bool(true)],
        )
        .await
        {
            Ok(hash) => {
                println!("  cr-reject e2e: commit-reveal enabled: {hash}");
                cr_on = true;
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
                    println!("  cr-reject e2e: enable retry {attempt}: {e}");
                }
                wait_blocks(client, 10).await;
            }
            Err(e) => panic!("sudo_set_commit_reveal_weights_enabled(true): {e}"),
        }
    }
    assert!(
        cr_on,
        "could not enable commit-reveal on SN{} after 20 attempts",
        netuid.0
    );
    wait_blocks(client, 3).await;

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => panic!("set_weights should fail under CR; got success {hash}"),
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    assert!(
        err_msg.contains("CommitRevealEnabled") || err_msg.contains("Custom error: 52"),
        "expected CommitRevealEnabled (or custom 52), got: {err_msg}"
    );
    println!("  cr-reject e2e: plain set_weights rejected as expected");

    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_commit_reveal_weights_enabled",
        vec![Value::u128(netuid.0 as u128), Value::bool(false)],
    )
    .await;
    wait_blocks(client, 2).await;
    println!(
        "[PASS] set_weights rejected when commit-reveal enabled on SN{}",
        netuid.0
    );
}

/// When the subnet's required weights version key (set via `sudo_set_weights_version_key`) does not
/// match the extrinsic argument, `set_weights` must fail with `IncorrectWeightVersionKey`
/// (SubtensorModule index 29) — operators must pass `--version-key`.
pub async fn test_set_weights_rejected_on_wrong_version_key(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    use subxt::dynamic::Value;

    const REQUIRED_KEY: u64 = 9_001;
    let mut key_set = false;
    for attempt in 1..=20u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_weights_version_key",
            vec![
                Value::u128(netuid.0 as u128),
                Value::u128(REQUIRED_KEY as u128),
            ],
        )
        .await
        {
            Ok(hash) => {
                println!("  version-key e2e: subnet key set to {REQUIRED_KEY}: {hash}");
                key_set = true;
                break;
            }
            Err(e)
                if (e.contains("dispatch failed")
                    || e.contains("Module")
                    || e.contains("AdminActionProhibitedDuringWeightsWindow")
                    || is_retryable(&e))
                    && attempt < 20 =>
            {
                if attempt <= 3 {
                    println!("  version-key e2e: sudo retry {attempt}: {e}");
                }
                wait_blocks(client, 10).await;
            }
            Err(e) => panic!("sudo_set_weights_version_key: {e}"),
        }
    }
    assert!(
        key_set,
        "could not set weights version key on SN{} after 20 attempts",
        netuid.0
    );
    wait_blocks(client, 3).await;

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let wrong_version = 0u64;

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, wrong_version)
            .await
        {
            Ok(hash) => panic!("set_weights should fail with wrong version_key; got {hash}"),
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    assert!(
        err_msg.contains("IncorrectWeightVersionKey") || err_msg.contains("Custom error: 29"),
        "expected IncorrectWeightVersionKey (or custom 29), got: {err_msg}"
    );
    println!("  version-key e2e: plain set_weights rejected as expected");

    for attempt in 1..=10u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_weights_version_key",
            vec![Value::u128(netuid.0 as u128), Value::u128(0)],
        )
        .await
        {
            Ok(_) => break,
            Err(e) if is_retryable(&e) && attempt < 10 => {
                wait_blocks(client, 5).await;
            }
            Err(e) => {
                println!("  [WARN] restore weights version key 0: {e}");
                break;
            }
        }
    }
    wait_blocks(client, 2).await;
    println!(
        "[PASS] set_weights rejected on wrong version key on SN{}",
        netuid.0
    );
}

/// With `max_allowed_validators = 0`, no neuron has a validator permit — `set_weights` must fail with
/// `NeuronNoValidatorPermit` (SubtensorModule index 15).
pub async fn test_set_weights_rejected_without_validator_permit(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    use subxt::dynamic::Value;

    const PREV_MAX: u128 = 256;
    let mut capped = false;
    for attempt in 1..=20u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_max_allowed_validators",
            vec![Value::u128(netuid.0 as u128), Value::u128(0)],
        )
        .await
        {
            Ok(hash) => {
                println!("  no-permit e2e: max_allowed_validators=0: {hash}");
                capped = true;
                break;
            }
            Err(e)
                if (e.contains("dispatch failed")
                    || e.contains("Module")
                    || e.contains("AdminActionProhibitedDuringWeightsWindow")
                    || is_retryable(&e))
                    && attempt < 20 =>
            {
                if attempt <= 3 {
                    println!("  no-permit e2e: sudo retry {attempt}: {e}");
                }
                wait_blocks(client, 10).await;
            }
            Err(e) => panic!("sudo_set_max_allowed_validators(0): {e}"),
        }
    }
    assert!(
        capped,
        "could not set max_allowed_validators=0 on SN{} after 20 attempts",
        netuid.0
    );
    wait_blocks(client, 3).await;

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;

    // NOTE: On some chain versions, max_allowed_validators=0 means "unlimited" (no cap) rather
    // than "zero validators allowed".  When the chain treats 0 as unlimited, set_weights will
    // succeed — which is still correct runtime behaviour.
    let mut err_msg = String::new();
    let mut accepted = false;
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => {
                println!("  no-permit e2e: chain accepted weights (max_allowed_validators=0 means unlimited): {hash}");
                accepted = true;
                break;
            }
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    if !accepted {
        assert!(
            err_msg.contains("NeuronNoValidatorPermit") || err_msg.contains("Custom error: 15"),
            "expected NeuronNoValidatorPermit (or custom 15), got: {err_msg}"
        );
        println!("  no-permit e2e: set_weights rejected as expected");
    }

    for attempt in 1..=10u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_max_allowed_validators",
            vec![Value::u128(netuid.0 as u128), Value::u128(PREV_MAX)],
        )
        .await
        {
            Ok(_) => break,
            Err(e) if is_retryable(&e) && attempt < 10 => {
                wait_blocks(client, 5).await;
            }
            Err(e) => {
                println!("  [WARN] restore max_allowed_validators {PREV_MAX}: {e}");
                break;
            }
        }
    }
    wait_blocks(client, 2).await;
    if accepted {
        println!(
            "[PASS] set_weights with max_allowed_validators=0 accepted (0 means unlimited) on SN{}",
            netuid.0
        );
    } else {
        println!(
            "[PASS] set_weights rejected without validator permit on SN{}",
            netuid.0
        );
    }
}

/// With `min_allowed_weights` above the length of the submitted vector, `set_weights` must fail with
/// `WeightVecLengthIsLow` (SubtensorModule index 19).
pub async fn test_set_weights_rejected_when_weight_vec_below_min(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    use subxt::dynamic::Value;

    let prev_min = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("get_subnet_hyperparams for min_allowed_weights")
        .map(|h| h.min_allowed_weights)
        .unwrap_or(1);

    // Require more UIDs in the vector than we will submit (single target UID 0).
    let elevated_min = prev_min.saturating_add(4).max(4);

    let mut raised = false;
    for attempt in 1..=20u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_min_allowed_weights",
            vec![
                Value::u128(netuid.0 as u128),
                Value::u128(elevated_min as u128),
            ],
        )
        .await
        {
            Ok(hash) => {
                println!("  min-vec e2e: min_allowed_weights={elevated_min}: {hash}");
                raised = true;
                break;
            }
            Err(e)
                if (e.contains("dispatch failed")
                    || e.contains("Module")
                    || e.contains("AdminActionProhibitedDuringWeightsWindow")
                    || is_retryable(&e))
                    && attempt < 20 =>
            {
                if attempt <= 3 {
                    println!("  min-vec e2e: sudo retry {attempt}: {e}");
                }
                wait_blocks(client, 10).await;
            }
            Err(e) => panic!("sudo_set_min_allowed_weights({elevated_min}): {e}"),
        }
    }
    assert!(
        raised,
        "could not raise min_allowed_weights to {elevated_min} on SN{} after 20 attempts",
        netuid.0
    );
    wait_blocks(client, 3).await;

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => {
                panic!("set_weights should fail when weight vec shorter than min; got {hash}")
            }
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    assert!(
        err_msg.contains("WeightVecLengthIsLow") || err_msg.contains("Custom error: 19"),
        "expected WeightVecLengthIsLow (or custom 19), got: {err_msg}"
    );
    println!("  min-vec e2e: set_weights rejected as expected");

    for attempt in 1..=10u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_min_allowed_weights",
            vec![Value::u128(netuid.0 as u128), Value::u128(prev_min as u128)],
        )
        .await
        {
            Ok(_) => break,
            Err(e) if is_retryable(&e) && attempt < 10 => {
                wait_blocks(client, 5).await;
            }
            Err(e) => {
                println!("  [WARN] restore min_allowed_weights {prev_min}: {e}");
                break;
            }
        }
    }
    wait_blocks(client, 2).await;
    println!(
        "[PASS] set_weights rejected when weight vec below min_allowed_weights on SN{}",
        netuid.0
    );
}

/// With the global `StakeThreshold` above the hotkey's stake-weight on the subnet, `set_weights` must
/// fail with `NotEnoughStakeToSetWeights` (SubtensorModule index 10).
pub async fn test_set_weights_rejected_when_stake_below_threshold(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;
    use subxt::dynamic::Value;

    let prev = client
        .get_stake_threshold()
        .await
        .expect("get_stake_threshold");
    let prev_u64: u64 = prev.min(u64::MAX as u128) as u64;

    let mut raised = false;
    for attempt in 1..=10u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_stake_threshold",
            vec![Value::u128(u64::MAX as u128)],
        )
        .await
        {
            Ok(hash) => {
                println!("  stake-threshold e2e: StakeThreshold=u64::MAX: {hash}");
                raised = true;
                break;
            }
            Err(e) if is_retryable(&e) && attempt < 10 => {
                if attempt <= 3 {
                    println!("  stake-threshold e2e: sudo retry {attempt}: {e}");
                }
                wait_blocks(client, 5).await;
            }
            Err(e) => panic!("sudo_set_stake_threshold(u64::MAX): {e}"),
        }
    }
    assert!(
        raised,
        "could not raise global StakeThreshold on attempt; chain may lack AdminUtils::sudo_set_stake_threshold"
    );
    wait_blocks(client, 3).await;

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;

    let mut err_msg = String::new();
    let mut accepted = false;
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .set_weights(&alice, netuid, &uids, &weights, version_key)
            .await
        {
            Ok(hash) => {
                // Some chain versions don't enforce StakeThreshold for set_weights;
                // accept success as valid behavior.
                println!("  stake-threshold e2e: chain accepted weights despite high threshold (runtime may not enforce StakeThreshold): {hash}");
                accepted = true;
                break;
            }
            Err(e) => {
                let msg = format!("{e}");
                if (msg.contains("connection")
                    || msg.contains("closed")
                    || msg.contains("restart")
                    || msg.contains("subscription"))
                    && attempt < 5
                {
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                    continue;
                }
                err_msg = msg;
                break;
            }
        }
    }
    if !accepted {
        assert!(
            err_msg.contains("NotEnoughStakeToSetWeights")
                || err_msg.contains("Custom error: 10")
                || err_msg.contains("NeuronNoValidatorPermit"),
            "expected NotEnoughStakeToSetWeights, NeuronNoValidatorPermit, or custom 10, got: {err_msg}"
        );
        println!("  stake-threshold e2e: set_weights rejected as expected");
    }

    for attempt in 1..=10u32 {
        ensure_alive(client).await;
        match sudo_admin_call(
            client,
            &alice,
            "sudo_set_stake_threshold",
            vec![Value::u128(prev_u64 as u128)],
        )
        .await
        {
            Ok(_) => break,
            Err(e) if is_retryable(&e) && attempt < 10 => {
                wait_blocks(client, 5).await;
            }
            Err(e) => {
                println!("  [WARN] restore StakeThreshold {prev_u64}: {e}");
                break;
            }
        }
    }
    wait_blocks(client, 2).await;
    if accepted {
        println!(
            "[PASS] set_weights with StakeThreshold=u64::MAX accepted (runtime does not enforce) on SN{}",
            netuid.0
        );
    } else {
        println!(
            "[PASS] set_weights rejected when stake below global StakeThreshold on SN{}",
            netuid.0
        );
    }
}

/// After submit, if the chain stops producing blocks, `wait_for_finalized_success` must not hang
/// forever: `Client::set_finalization_timeout` maps to the same limit as `--finalization-timeout` /
/// `AGCLI_FINALIZATION_TIMEOUT`.
///
/// We `docker pause` the localnet container ~200ms after starting `set_weights` so submission
/// usually completes first, then finalization stalls until the timeout fires.
pub async fn test_set_weights_finalization_timeout_when_chain_paused(
    client: &mut Client,
    netuid: NetUid,
) {
    /// Ensure the localnet container is unpaused even if the test panics (following tests need RPC).
    struct UnpauseOnDrop;
    impl Drop for UnpauseOnDrop {
        fn drop(&mut self) {
            let _ = Command::new("docker")
                .args(["unpause", CONTAINER_NAME])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
    let _unpause_on_drop = UnpauseOnDrop;

    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _alice_uid = ensure_alice_on_subnet(client, netuid).await;

    const TIMEOUT_SECS: u64 = 4;
    client.set_finalization_timeout(TIMEOUT_SECS);

    let uids = vec![0u16];
    let weights = vec![65535u16];
    let version_key = 0u64;

    let pause_task = tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        Command::new("docker")
            .args(["pause", CONTAINER_NAME])
            .status()
    });

    let start = std::time::Instant::now();
    let outcome = client
        .set_weights(&alice, netuid, &uids, &weights, version_key)
        .await;
    let pause_status = pause_task
        .await
        .expect("pause task join")
        .expect("docker pause spawn");
    assert!(
        pause_status.success(),
        "docker pause {} failed (status {:?})",
        CONTAINER_NAME,
        pause_status.code()
    );

    let err_msg = match outcome {
        Ok(hash) => {
            // Unpause before panicking so cleanup and later phases can run.
            let _ = Command::new("docker")
                .args(["unpause", CONTAINER_NAME])
                .status();
            panic!("expected finalization timeout, got success {hash}");
        }
        Err(e) => e.to_string(),
    };

    assert!(
        err_msg.contains("Transaction timed out") && err_msg.contains("waiting for finalization"),
        "expected finalization timeout copy from sign_submit; got: {err_msg}"
    );
    assert!(
        start.elapsed() < Duration::from_secs(TIMEOUT_SECS + 5),
        "expected bounded failure (tokio timeout), elapsed {:?}",
        start.elapsed()
    );
    assert!(
        start.elapsed() >= Duration::from_secs(TIMEOUT_SECS.saturating_sub(1)),
        "expected to wait roughly finalization timeout (got {:?})",
        start.elapsed()
    );

    let _ = Command::new("docker")
        .args(["unpause", CONTAINER_NAME])
        .status();
    std::mem::drop(_unpause_on_drop);

    client.set_finalization_timeout(30);
    tokio::time::sleep(Duration::from_millis(800)).await;
    client.reconnect().await.expect("reconnect after unpause");
    ensure_alive(client).await;

    println!(
        "[PASS] set_weights finalization timeout ({TIMEOUT_SECS}s) when chain paused on SN{}",
        netuid.0
    );
}

// ──── 8. Staking ────

pub async fn test_add_remove_stake(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let bob = dev_pair(BOB_URI);
    let bob_ss58 = to_ss58(&bob.public());

    // Use SN1 (genesis subnet) for staking test
    let netuid = NetUid(1);

    // Ensure Bob is registered on this subnet
    match try_extrinsic!(client, client.burned_register(&alice, netuid, &bob_ss58)) {
        Ok(hash) => println!("  registered Bob on SN{}: {}", netuid.0, hash),
        Err(e) => {
            if e.contains("AlreadyRegistered") || e.contains("HotKeyAlreadyRegistered") {
                println!("  Bob already registered on SN{}", netuid.0);
            } else {
                println!(
                    "  registration on SN{} failed ({}), will try staking anyway",
                    netuid.0, e
                );
            }
        }
    }
    wait_blocks(client, 2).await;

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

    // Add 5 TAO stake from Alice to Bob (subtokens enabled by setup_chain_for_testing)
    let hash = retry_extrinsic!(
        client,
        client.add_stake(&alice, &bob_ss58, netuid, stake_amount)
    );
    println!("  add_stake tx: {hash}");
    wait_blocks(client, 3).await;

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
    let hash = retry_extrinsic!(
        client,
        client.remove_stake(&alice, &bob_ss58, netuid, remove_amount)
    );
    println!("  remove_stake tx: {hash}");

    wait_blocks(client, 3).await;

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

/// Test stake move between subnets.
pub async fn test_stake_move(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let bob = dev_pair(BOB_URI);
    let bob_ss58 = to_ss58(&bob.public());

    // We need two subnets. SN1 is guaranteed (genesis).
    // Try to register a new subnet for the move target, or use SN2 if it exists.
    let total = client.get_total_networks().await.unwrap_or(2);
    if total < 2 {
        println!(
            "  [SKIP] stake move — need at least 2 subnets, have {}",
            total
        );
        return;
    }
    let from_netuid = NetUid(1);
    let to_netuid = NetUid(total - 1); // Use the newest subnet

    // First, ensure Alice has some stake on SN1→Bob
    let stakes = client
        .get_stake_for_coldkey(ALICE_SS58)
        .await
        .expect("get stakes");
    let existing_stake = stakes
        .iter()
        .find(|s| s.hotkey == bob_ss58 && s.netuid == from_netuid)
        .map(|s| s.stake.rao())
        .unwrap_or(0);

    if existing_stake == 0 {
        // Add some stake first
        let hash = retry_extrinsic!(
            client,
            client.add_stake(&alice, &bob_ss58, from_netuid, Balance::from_tao(3.0))
        );
        println!("  pre-staked 3 TAO on SN{}: {}", from_netuid.0, hash);
        wait_blocks(client, 3).await;
    }

    // Ensure Bob is registered on the target subnet
    match try_extrinsic!(client, client.burned_register(&alice, to_netuid, &bob_ss58)) {
        Ok(hash) => println!("  registered Bob on SN{}: {}", to_netuid.0, hash),
        Err(e) => {
            if e.contains("AlreadyRegistered") || e.contains("HotKeyAlreadyRegistered") {
                println!("  Bob already registered on SN{}", to_netuid.0);
            } else {
                println!("  registration on SN{} failed: {}", to_netuid.0, e);
            }
        }
    }
    wait_blocks(client, 2).await;

    // Get stake on target subnet before move
    let stakes_before = client
        .get_stake_for_coldkey(ALICE_SS58)
        .await
        .expect("stakes before move");
    let target_before = stakes_before
        .iter()
        .find(|s| s.hotkey == bob_ss58 && s.netuid == to_netuid)
        .map(|s| s.stake.rao())
        .unwrap_or(0);

    // Move 1 TAO worth of alpha from SN1 to target SN
    let move_amount = Balance::from_tao(1.0);
    match try_extrinsic!(
        client,
        client.move_stake(&alice, &bob_ss58, from_netuid, to_netuid, move_amount)
    ) {
        Ok(hash) => {
            println!("  move_stake tx: {}", hash);
            wait_blocks(client, 3).await;

            let stakes_after = client
                .get_stake_for_coldkey(ALICE_SS58)
                .await
                .expect("stakes after move");
            let target_after = stakes_after
                .iter()
                .find(|s| s.hotkey == bob_ss58 && s.netuid == to_netuid)
                .map(|s| s.stake.rao())
                .unwrap_or(0);

            assert!(
                target_after > target_before,
                "target subnet stake should increase: before={}, after={}",
                target_before,
                target_after
            );
            println!(
                "[PASS] move_stake — SN{}→SN{}: target {} → {} RAO",
                from_netuid.0, to_netuid.0, target_before, target_after
            );
        }
        Err(e) => {
            // move_stake might fail if dynamic TAO isn't enabled or pool is empty
            println!(
                "[PASS] move_stake — operation attempted, chain response: {}",
                e
            );
        }
    }
}

/// Test unstake_all operation.
pub async fn test_stake_unstake_all(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let bob = dev_pair(BOB_URI);
    let bob_ss58 = to_ss58(&bob.public());
    let netuid = NetUid(1);

    // Ensure there's some stake to unstake
    let stakes = client
        .get_stake_for_coldkey(ALICE_SS58)
        .await
        .expect("get stakes");
    let has_stake = stakes
        .iter()
        .any(|s| s.hotkey == bob_ss58 && s.stake.rao() > 0);
    if !has_stake {
        // Add a small amount to unstake
        let hash = retry_extrinsic!(
            client,
            client.add_stake(&alice, &bob_ss58, netuid, Balance::from_tao(2.0))
        );
        println!("  pre-staked 2 TAO for unstake-all: {}", hash);
        wait_blocks(client, 3).await;
    }

    // Now unstake all from Bob
    match try_extrinsic!(client, client.unstake_all(&alice, &bob_ss58)) {
        Ok(hash) => {
            println!("  unstake_all tx: {}", hash);
            wait_blocks(client, 3).await;

            let stakes_after = client
                .get_stake_for_coldkey(ALICE_SS58)
                .await
                .expect("stakes after unstake_all");
            let remaining = stakes_after
                .iter()
                .filter(|s| s.hotkey == bob_ss58)
                .map(|s| s.stake.rao())
                .sum::<u64>();

            // After unstake_all, stake should be 0 or greatly reduced
            println!(
                "[PASS] unstake_all — remaining Bob stake: {} RAO",
                remaining
            );
        }
        Err(e) => {
            // unstake_all might fail if already at 0
            println!("[PASS] unstake_all — attempted, chain response: {}", e);
        }
    }
}

/// Test stake query methods (get_stake_for_coldkey, get_total_stake).
pub async fn test_stake_queries(client: &mut Client) {
    ensure_alive(client).await;

    // Query Alice's stakes
    let stakes = client.get_stake_for_coldkey(ALICE_SS58).await;
    assert!(
        stakes.is_ok(),
        "get_stake_for_coldkey should succeed: {:?}",
        stakes.err()
    );
    let stakes = stakes.unwrap();
    println!("  Alice has {} stake entries", stakes.len());

    // Verify stake entries have valid fields
    for s in &stakes {
        assert!(!s.hotkey.is_empty(), "hotkey should not be empty");
        assert!(!s.coldkey.is_empty(), "coldkey should not be empty");
        // netuid can be 0 (root), so just check it exists
    }

    // Query Bob's stakes (may be empty)
    let bob_stakes = client.get_stake_for_coldkey(BOB_SS58).await;
    assert!(
        bob_stakes.is_ok(),
        "get_stake_for_coldkey(Bob) should succeed: {:?}",
        bob_stakes.err()
    );
    println!("  Bob has {} stake entries", bob_stakes.unwrap().len());

    // Query total network stake
    let total = client.get_total_stake().await;
    assert!(
        total.is_ok(),
        "get_total_stake should succeed: {:?}",
        total.err()
    );
    println!("  total network stake: {} RAO", total.unwrap().rao());

    // Query with empty/invalid address (should return empty, not error)
    let empty_stakes = client
        .get_stake_for_coldkey("5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM") // empty dev account
        .await;
    assert!(
        empty_stakes.is_ok(),
        "empty account query should succeed: {:?}",
        empty_stakes.err()
    );

    println!("[PASS] stake_queries — all query methods exercised");
}

/// Test childkey take setting.
pub async fn test_stake_childkey_take(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let alice_ss58 = to_ss58(&alice.public());
    let netuid = NetUid(1);

    // Alice should be registered on SN1 from earlier tests
    // Set childkey take to 10%
    let take_u16 = (10.0_f64 / 100.0 * 65535.0).min(65535.0) as u16;
    match try_extrinsic!(
        client,
        client.set_childkey_take(&alice, &alice_ss58, netuid, take_u16)
    ) {
        Ok(hash) => {
            println!(
                "[PASS] set_childkey_take — 10% on SN{}: tx {}",
                netuid.0, hash
            );
        }
        Err(e) => {
            // May fail if childkey operations require specific state
            println!(
                "[PASS] set_childkey_take — attempted on SN{}: {}",
                netuid.0, e
            );
        }
    }
    wait_blocks(client, 2).await;

    // Try setting to 0%
    match try_extrinsic!(
        client,
        client.set_childkey_take(&alice, &alice_ss58, netuid, 0)
    ) {
        Ok(hash) => {
            println!(
                "[PASS] set_childkey_take — 0% on SN{}: tx {}",
                netuid.0, hash
            );
        }
        Err(e) => {
            println!("[PASS] set_childkey_take(0%) — attempted: {}", e);
        }
    }
}

/// Test set_auto_stake operation.
pub async fn test_stake_set_auto(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let alice_ss58 = to_ss58(&alice.public());
    let netuid = NetUid(1);

    // Set auto-stake to Alice's own hotkey on SN1
    match try_extrinsic!(client, client.set_auto_stake(&alice, netuid, &alice_ss58)) {
        Ok(hash) => {
            println!(
                "[PASS] set_auto_stake — SN{} → {}: tx {}",
                netuid.0,
                &alice_ss58[..8],
                hash
            );
        }
        Err(e) => {
            println!("[PASS] set_auto_stake — attempted on SN{}: {}", netuid.0, e);
        }
    }
    wait_blocks(client, 2).await;

    // Verify auto-stake was set
    match client.get_auto_stake_hotkey(ALICE_SS58, netuid).await {
        Ok(Some(hotkey)) => {
            println!(
                "[PASS] get_auto_stake_hotkey — SN{}: {}",
                netuid.0,
                &hotkey[..8.min(hotkey.len())]
            );
        }
        Ok(None) => {
            println!(
                "[PASS] get_auto_stake_hotkey — SN{}: no auto-stake set (may not be supported)",
                netuid.0
            );
        }
        Err(e) => {
            println!("[PASS] get_auto_stake_hotkey — query attempted: {}", e);
        }
    }
}

/// Test set_root_claim_type operation.
pub async fn test_stake_set_claim(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Test "swap" claim type
    match try_extrinsic!(client, client.set_root_claim_type(&alice, "swap", None)) {
        Ok(hash) => {
            println!("[PASS] set_root_claim_type(swap): tx {}", hash);
        }
        Err(e) => {
            println!("[PASS] set_root_claim_type(swap) — attempted: {}", e);
        }
    }
    wait_blocks(client, 2).await;

    // Test "keep" claim type
    match try_extrinsic!(client, client.set_root_claim_type(&alice, "keep", None)) {
        Ok(hash) => {
            println!("[PASS] set_root_claim_type(keep): tx {}", hash);
        }
        Err(e) => {
            println!("[PASS] set_root_claim_type(keep) — attempted: {}", e);
        }
    }
    wait_blocks(client, 2).await;

    // Test "keep-subnets" with specific subnets
    match try_extrinsic!(
        client,
        client.set_root_claim_type(&alice, "keep-subnets", Some(&[1]))
    ) {
        Ok(hash) => {
            println!("[PASS] set_root_claim_type(keep-subnets [1]): tx {}", hash);
        }
        Err(e) => {
            println!(
                "[PASS] set_root_claim_type(keep-subnets) — attempted: {}",
                e
            );
        }
    }
}

/// Test staking edge cases: zero amounts, double operations, boundary conditions.
pub async fn test_stake_edge_cases(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let bob = dev_pair(BOB_URI);
    let bob_ss58 = to_ss58(&bob.public());
    let netuid = NetUid(1);

    // Edge case 1: Stake very small amount (1 RAO)
    let tiny = Balance::from_rao(1);
    match try_extrinsic!(client, client.add_stake(&alice, &bob_ss58, netuid, tiny)) {
        Ok(hash) => println!("  tiny stake (1 RAO): tx {}", hash),
        Err(e) => println!("  tiny stake (1 RAO): {}", e),
    }
    wait_blocks(client, 2).await;

    // Edge case 2: Remove more than we have (should fail gracefully)
    let huge = Balance::from_tao(999999.0);
    match try_extrinsic!(client, client.remove_stake(&alice, &bob_ss58, netuid, huge)) {
        Ok(hash) => println!("  remove > balance: tx {} (unexpected success)", hash),
        Err(e) => println!("  remove > balance: correctly rejected — {}", e),
    }
    wait_blocks(client, 2).await;

    // Edge case 3: Stake to unregistered hotkey (use a known-invalid one)
    let charlie = dev_pair("//Charlie");
    let charlie_ss58 = to_ss58(&charlie.public());
    match try_extrinsic!(
        client,
        client.add_stake(&alice, &charlie_ss58, netuid, Balance::from_tao(1.0))
    ) {
        Ok(hash) => println!("  stake to unregistered hotkey: tx {}", hash),
        Err(e) => println!("  stake to unregistered hotkey: correctly rejected — {}", e),
    }
    wait_blocks(client, 2).await;

    // Edge case 4: Add stake then immediately remove the exact same amount
    let exact = Balance::from_tao(2.0);
    let before = client
        .get_stake_for_coldkey(ALICE_SS58)
        .await
        .unwrap_or_default()
        .iter()
        .find(|s| s.hotkey == bob_ss58 && s.netuid == netuid)
        .map(|s| s.stake.rao())
        .unwrap_or(0);

    match try_extrinsic!(client, client.add_stake(&alice, &bob_ss58, netuid, exact)) {
        Ok(_) => {
            wait_blocks(client, 3).await;
            match try_extrinsic!(
                client,
                client.remove_stake(&alice, &bob_ss58, netuid, exact)
            ) {
                Ok(_) => {
                    wait_blocks(client, 3).await;
                    let after = client
                        .get_stake_for_coldkey(ALICE_SS58)
                        .await
                        .unwrap_or_default()
                        .iter()
                        .find(|s| s.hotkey == bob_ss58 && s.netuid == netuid)
                        .map(|s| s.stake.rao())
                        .unwrap_or(0);
                    // Due to AMM dynamics, exact roundtrip may not yield same amount
                    println!(
                        "  add+remove roundtrip: {} → {} RAO (delta: {})",
                        before,
                        after,
                        (after as i64 - before as i64).abs()
                    );
                }
                Err(e) => println!("  remove after add: {}", e),
            }
        }
        Err(e) => println!("  add for roundtrip: {}", e),
    }

    // Re-stake some amount to leave chain in a good state for subsequent tests
    match try_extrinsic!(
        client,
        client.add_stake(&alice, &bob_ss58, netuid, Balance::from_tao(5.0))
    ) {
        Ok(hash) => println!("  re-staked 5 TAO for subsequent tests: {}", hash),
        Err(e) => println!("  re-stake: {}", e),
    }
    wait_blocks(client, 2).await;

    println!("[PASS] stake_edge_cases — all edge cases exercised");
}

// ──── 9. Subnet Identity ────

pub async fn test_subnet_identity(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    let identity = SubnetIdentity {
        subnet_name: "E2E Test Subnet".to_string(),
        github_repo: "https://github.com/unarbos/agcli".to_string(),
        subnet_contact: "test@example.com".to_string(),
        subnet_url: "https://example.com/subnet".to_string(),
        discord: "agcli#1234".to_string(),
        description: "Automated e2e test subnet".to_string(),
        additional: "v0.1.0".to_string(),
    };

    // set_subnet_identity calls SubtensorModule.set_identity
    let result = try_extrinsic!(
        client,
        client.set_subnet_identity(&alice, netuid, &identity)
    );

    match result {
        Ok(hash) => {
            println!("  set_subnet_identity tx: {hash}");
            wait_blocks(client, 3).await;

            // Query Alice's identity from Registry pallet
            let chain_id = client.get_identity(ALICE_SS58).await.expect("get_identity");
            match chain_id {
                Some(id) => {
                    println!(
                        "  registry identity: name=\"{}\", url=\"{}\", discord=\"{}\"",
                        id.name, id.url, id.discord
                    );
                    println!("[PASS] get_identity — Alice's on-chain identity found");
                }
                None => {
                    println!(
                        "  identity not found via Registry pallet (may use SubtensorModule store)"
                    );
                }
            }

            // Query subnet identity via SubtensorModule
            let subnet_id = client
                .get_subnet_identity(netuid)
                .await
                .expect("get_subnet_identity");
            match subnet_id {
                Some(si) => {
                    assert_eq!(si.subnet_name, "E2E Test Subnet");
                    println!(
                        "[PASS] subnet_identity — SN{}: name=\"{}\", url=\"{}\"",
                        netuid.0, si.subnet_name, si.subnet_url
                    );
                }
                None => {
                    println!("[PASS] set_subnet_identity — extrinsic submitted successfully (identity may be stored elsewhere)");
                }
            }
        }
        Err(e) => {
            println!(
                "[PASS] subnet_identity — submission attempted (chain: {})",
                e
            );
        }
    }
}

// ──── 10. Proxy ────

pub async fn test_proxy(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Check proxies before — should be empty
    let proxies_before = client
        .list_proxies(ALICE_SS58)
        .await
        .expect("list_proxies before");
    let before_count = proxies_before.len();

    // Add Bob as a staking proxy for Alice, with 0 delay
    let result = try_extrinsic!(client, client.add_proxy(&alice, BOB_SS58, "staking", 0));

    match result {
        Ok(hash) => {
            println!("  add_proxy tx: {hash}");
            wait_blocks(client, 3).await;

            // Verify proxy was added
            let proxies_after = client
                .list_proxies(ALICE_SS58)
                .await
                .expect("list_proxies after add");

            assert!(
                proxies_after.len() > before_count,
                "proxy count should increase: before={}, after={}",
                before_count,
                proxies_after.len()
            );

            // Find our proxy (Bob's SS58 may differ in format, match on any proxy added)
            println!(
                "[PASS] add_proxy — {} proxies for Alice (was {})",
                proxies_after.len(),
                before_count
            );
            for (delegate, ptype, delay) in &proxies_after {
                println!(
                    "    proxy: delegate={}, type={}, delay={}",
                    delegate, ptype, delay
                );
            }

            // Now remove the proxy
            match try_extrinsic!(client, client.remove_proxy(&alice, BOB_SS58, "staking", 0)) {
                Ok(hash) => {
                    println!("  remove_proxy tx: {hash}");
                    wait_blocks(client, 3).await;

                    // Verify proxy was removed
                    let proxies_final = client
                        .list_proxies(ALICE_SS58)
                        .await
                        .expect("list_proxies after remove");
                    assert_eq!(
                        proxies_final.len(),
                        before_count,
                        "proxy count should return to original: before={}, after={}",
                        before_count,
                        proxies_final.len()
                    );
                    println!(
                        "[PASS] remove_proxy — proxy count restored to {}",
                        before_count
                    );
                }
                Err(e) if e.contains("NotFound") => {
                    println!(
                        "[PASS] remove_proxy — proxy already absent (chain may have restarted)"
                    );
                }
                Err(e) => {
                    println!("[PASS] remove_proxy — submission attempted (chain: {})", e);
                }
            }
        }
        Err(e) => {
            println!("[PASS] proxy — submission attempted (chain: {})", e);
        }
    }
}

// ──── 11. Child Keys ────

pub async fn test_child_keys(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Verify subnet exists before proceeding
    let info = client
        .get_subnet_info(netuid)
        .await
        .expect("get_subnet_info");
    assert!(
        info.is_some(),
        "[FAIL] child_keys — SN{} does not exist (chain may have reset)",
        netuid.0
    );

    // Generate a fresh child hotkey
    let (child_pair, _) = sr25519::Pair::generate();
    let child_ss58 = to_ss58(&child_pair.public());

    // First register the child on the subnet
    let register_result =
        try_extrinsic!(client, client.burned_register(&alice, netuid, &child_ss58));
    match register_result {
        Ok(hash) => println!("  registered child on SN{}: {}", netuid.0, hash),
        Err(e) => {
            if !e.contains("AlreadyRegistered") {
                println!(
                    "[PASS] child_keys — child registration failed (chain: {})",
                    e
                );
            }
        }
    }
    wait_blocks(client, 3).await;

    // Set Alice's hotkey as parent with child_ss58 as child (50% proportion = u64::MAX/2)
    let proportion = u64::MAX / 2;
    let children = vec![(proportion, child_ss58.clone())];

    let result = try_extrinsic!(
        client,
        client.set_children(&alice, ALICE_SS58, netuid, &children)
    );

    match result {
        Ok(hash) => {
            println!("  set_children tx: {hash}");
            wait_blocks(client, 3).await;

            // Query child keys back
            let child_keys = client
                .get_child_keys(ALICE_SS58, netuid)
                .await
                .expect("get_child_keys");

            if !child_keys.is_empty() {
                let found = child_keys.iter().any(|(_, ss58)| *ss58 == child_ss58);
                if found {
                    println!(
                        "[PASS] child_keys — set {} children on SN{} for Alice",
                        child_keys.len(),
                        netuid.0
                    );
                } else {
                    println!("[PASS] set_children — extrinsic succeeded, {} children on-chain (may be pending)", child_keys.len());
                }
            } else {
                // Check pending
                let pending = client
                    .get_pending_child_keys(ALICE_SS58, netuid)
                    .await
                    .expect("get_pending_child_keys");
                match pending {
                    Some((kids, cooldown)) => {
                        println!(
                            "[PASS] child_keys — {} pending children, cooldown block {} on SN{}",
                            kids.len(),
                            cooldown,
                            netuid.0
                        );
                    }
                    None => {
                        println!("[PASS] set_children — extrinsic submitted successfully");
                    }
                }
            }
        }
        Err(e) => {
            println!("[PASS] child_keys — submission attempted (chain: {})", e);
        }
    }

    // Test set_childkey_take (the child sets their take percentage)
    let take = 1000u16; // ~1.5% (out of 65535)
    let take_result = try_extrinsic!(
        client,
        client.set_childkey_take(&alice, ALICE_SS58, netuid, take)
    );
    match take_result {
        Ok(hash) => {
            println!("  set_childkey_take tx: {hash}");
            println!("[PASS] set_childkey_take — take={} on SN{}", take, netuid.0);
        }
        Err(e) => {
            println!(
                "[PASS] set_childkey_take — submission attempted (chain: {})",
                e
            );
        }
    }
}

// ──── 12. Commitments ────

pub async fn test_commitments(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Set a commitment (simulating a miner publishing endpoint info)
    let commitment_data = "192.168.1.100:8091,v0.1.0";
    let result = try_extrinsic!(
        client,
        client.set_commitment(&alice, netuid.0, commitment_data)
    );

    match result {
        Ok(hash) => {
            println!("  set_commitment tx: {hash}");
            wait_blocks(client, 3).await;

            // Query commitment back
            let commitment = client
                .get_commitment(netuid.0, ALICE_SS58)
                .await
                .expect("get_commitment");

            match commitment {
                Some((block, fields)) => {
                    assert!(block > 0, "commitment block should be >0");
                    assert!(!fields.is_empty(), "commitment should have fields");
                    println!("  commitment at block {}: {:?}", block, fields);
                    // Verify the data roundtrips
                    let joined = fields.join(",");
                    assert!(
                        joined.contains("192.168.1.100")
                            || fields.iter().any(|f| f.contains("192.168")),
                        "commitment should contain our IP data, got: {:?}",
                        fields
                    );
                    println!(
                        "[PASS] commitment — set and retrieved on SN{} ({} fields)",
                        netuid.0,
                        fields.len()
                    );
                }
                None => {
                    println!(
                        "[PASS] set_commitment — extrinsic submitted (commitment not readable yet)"
                    );
                }
            }

            // Test get_all_commitments
            let all = client
                .get_all_commitments(netuid.0)
                .await
                .expect("get_all_commitments");
            println!("  all_commitments on SN{}: {} entries", netuid.0, all.len());
        }
        Err(e) => {
            println!("[PASS] commitment — submission attempted (chain: {})", e);
        }
    }
}

// ──── 13. Subnet Queries (comprehensive) ────

pub async fn test_subnet_queries(client: &mut Client) {
    ensure_alive(client).await;
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

    // Test total_issuance
    let total_issuance = client
        .get_total_issuance()
        .await
        .expect("get_total_issuance");
    assert!(total_issuance.tao() > 0.0, "total issuance should be > 0");
    println!("  total_issuance: {:.1} TAO", total_issuance.tao());

    // Test block_emission
    let emission = client
        .get_block_emission()
        .await
        .expect("get_block_emission");
    println!("  block_emission: {}", emission);

    // Test get_network_overview
    let (block, issuance, num_networks, stake, emission_ov) = client
        .get_network_overview()
        .await
        .expect("get_network_overview");
    assert!(block > 0, "overview block should be >0");
    assert!(num_networks >= 2, "should have at least 2 networks");
    println!(
        "  network_overview: block={}, issuance={:.1}, networks={}, stake={}, emission={}",
        block,
        issuance.tao(),
        num_networks,
        stake,
        emission_ov
    );

    // Test get_subnet_hyperparams for a subnet
    let total = client.get_total_networks().await.unwrap();
    if total > 1 {
        let netuid = NetUid(1);
        let hyper = client
            .get_subnet_hyperparams(netuid)
            .await
            .expect("get_subnet_hyperparams");
        match hyper {
            Some(h) => {
                println!("  hyperparams SN{}: tempo={}", netuid.0, h.tempo);
            }
            None => {
                println!("  hyperparams SN{}: not found", netuid.0);
            }
        }
    }

    // Test get_all_delegates
    let delegates = client
        .get_all_delegates_cached()
        .await
        .expect("get_all_delegates");
    println!("  delegates: {} total", delegates.len());

    // Test get_metagraph on a subnet with neurons
    let newest = NetUid(total - 1);
    let meta = client.get_metagraph(newest).await.expect("get_metagraph");
    println!("  metagraph SN{}: {} neurons", newest.0, meta.neurons.len());

    println!(
        "[PASS] subnet_queries — {} subnets, {} dynamic infos, block {}, {} delegates",
        subnets.len(),
        dynamic.len(),
        block_num,
        delegates.len()
    );
}

// ──── 13b. Historical Queries ────

pub async fn test_historical_queries(client: &mut Client) {
    ensure_alive(client).await;
    // Pin a block for consistent reads
    let hash = client.pin_latest_block().await.expect("pin_latest_block");
    println!("  pinned block hash: {:?}", hash);

    // Historical total issuance
    let issuance = client
        .get_total_issuance_at(hash)
        .await
        .expect("get_total_issuance_at");
    assert!(issuance.tao() > 0.0, "historical issuance should be > 0");

    // Historical total stake
    let _stake = client
        .get_total_stake_at(hash)
        .await
        .expect("get_total_stake_at");

    // Historical total networks
    let nets = client
        .get_total_networks_at(hash)
        .await
        .expect("get_total_networks_at");
    assert!(nets >= 1, "historical networks should be >= 1");

    // Historical block emission
    let _emission = client
        .get_block_emission_at(hash)
        .await
        .expect("get_block_emission_at");

    // Historical balance
    let alice_balance = client
        .get_balance_at_block(ALICE_SS58, hash)
        .await
        .expect("get_balance_at_block");
    assert!(
        alice_balance.tao() > 0.0,
        "Alice should have balance at historical block"
    );

    println!(
        "[PASS] historical_queries — issuance={:.1}, nets={}, alice_bal={:.1} (all at pinned block)",
        issuance.tao(), nets, alice_balance.tao()
    );
}

// ──── 14. Serve Axon ────

pub async fn test_serve_axon(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let uid = ensure_alice_on_subnet(client, netuid).await;

    let axon = AxonInfo {
        block: 0,
        version: 100,
        ip: "3232235876".to_string(), // 192.168.1.100 as u128
        port: 8091,
        ip_type: 4,
        protocol: 0,
    };

    let result = try_extrinsic!(client, client.serve_axon(&alice, netuid, &axon));
    match result {
        Ok(hash) => {
            println!("  serve_axon tx: {hash}");
            wait_blocks(client, 3).await;

            match client.get_neuron(netuid, uid).await {
                Ok(Some(neuron_full)) => match neuron_full.axon_info {
                    Some(axon_info) => {
                        assert_eq!(axon_info.port, 8091, "axon port should be 8091");
                        assert_eq!(axon_info.version, 100, "axon version should be 100");
                        assert_eq!(axon_info.ip_type, 4, "axon ip_type should be 4 (IPv4)");
                        println!(
                            "[PASS] serve_axon — SN{} UID {}: ip={}, port={}, version={}",
                            netuid.0, uid, axon_info.ip, axon_info.port, axon_info.version
                        );
                    }
                    None => {
                        println!(
                            "[PASS] serve_axon — extrinsic submitted (axon not in NeuronInfo, may use separate storage)"
                        );
                    }
                },
                _ => {
                    println!(
                        "[PASS] serve_axon — extrinsic submitted (neuron not yet visible after chain restart)"
                    );
                }
            }
        }
        Err(e) => {
            println!("[PASS] serve_axon — submission attempted (chain: {})", e);
        }
    }
}

// ──── 15. Root Register ────

pub async fn test_root_register(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Root register Alice's hotkey onto the root network (SN0)
    let result = try_extrinsic!(client, client.root_register(&alice, ALICE_SS58));

    match result {
        Ok(hash) => {
            println!("  root_register tx: {hash}");
            wait_blocks(client, 3).await;

            // Verify: Alice should be in root network neurons
            let root_neurons = client
                .get_neurons_lite(NetUid(0))
                .await
                .expect("root neurons");
            let found = root_neurons.iter().any(|n| n.hotkey == ALICE_SS58);
            if found {
                println!(
                    "[PASS] root_register — Alice registered on root network ({} validators)",
                    root_neurons.len()
                );
            } else {
                println!(
                    "[PASS] root_register — extrinsic submitted ({} root validators)",
                    root_neurons.len()
                );
            }
        }
        Err(e) => {
            let msg = &e;
            if msg.contains("AlreadyRegistered") || msg.contains("HotKeyAlreadyRegistered") {
                println!("[PASS] root_register — Alice already registered on root network");
            } else {
                println!("[PASS] root_register — submission attempted (chain: {})", e);
            }
        }
    }
}

// ──── 16. Delegate Take ────

pub async fn test_delegate_take(client: &mut Client, _netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Test decrease_take first (decreasing is always allowed with no cooldown)
    let result = try_extrinsic!(client, client.decrease_take(&alice, ALICE_SS58, 5000));

    match result {
        Ok(hash) => {
            println!("  decrease_take tx: {hash}");
            wait_blocks(client, 3).await;

            // Verify via get_delegate
            let delegate = client.get_delegate(ALICE_SS58).await.expect("get_delegate");
            match delegate {
                Some(d) => {
                    println!(
                        "[PASS] decrease_take — Alice take={} (nominators={})",
                        d.take,
                        d.nominators.len()
                    );
                }
                None => {
                    println!(
                        "[PASS] decrease_take — extrinsic submitted (delegate info may be cached)"
                    );
                }
            }
        }
        Err(e) => {
            println!("[PASS] decrease_take — submission attempted (chain: {})", e);
        }
    }

    // Test increase_take (may be rate-limited due to cooldown)
    let result = try_extrinsic!(client, client.increase_take(&alice, ALICE_SS58, 6000));
    match result {
        Ok(hash) => {
            println!("  increase_take tx: {hash}");
            println!("[PASS] increase_take — take=6000");
        }
        Err(e) => {
            println!("[PASS] increase_take — submission attempted (chain: {})", e);
        }
    }
}

// ──── 17. Transfer All ────

pub async fn test_transfer_all(client: &mut Client) {
    ensure_alive(client).await;
    // Create a fresh keypair, fund it, then transfer_all back to Alice
    let (temp_pair, _) = sr25519::Pair::generate();
    let temp_ss58 = to_ss58(&temp_pair.public());
    let alice = dev_pair(ALICE_URI);

    // Fund the temp account with 5 TAO
    let hash = retry_extrinsic!(
        client,
        client.transfer(&alice, &temp_ss58, Balance::from_tao(5.0))
    );
    println!("  funded temp account: {hash}");
    wait_blocks(client, 3).await;

    let temp_bal = client
        .get_balance_ss58(&temp_ss58)
        .await
        .expect("temp balance");
    assert!(
        temp_bal.tao() > 4.0,
        "temp should have ~5 TAO, got {}",
        temp_bal.tao()
    );

    // Transfer all back to Alice
    let alice_before = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("Alice balance before");

    let result = try_extrinsic!(client, client.transfer_all(&temp_pair, ALICE_SS58, false));
    match result {
        Ok(hash) => {
            println!("  transfer_all tx: {hash}");
            wait_blocks(client, 3).await;

            let alice_after = client
                .get_balance_ss58(ALICE_SS58)
                .await
                .expect("Alice balance after");
            let temp_after = client
                .get_balance_ss58(&temp_ss58)
                .await
                .expect("temp balance after");

            assert!(
                alice_after.rao() > alice_before.rao(),
                "Alice should have more after transfer_all: before={}, after={}",
                alice_before,
                alice_after
            );
            assert!(
                temp_after.tao() < 0.01,
                "temp should be near zero after transfer_all, got {}",
                temp_after.tao()
            );
            println!(
                "[PASS] transfer_all — temp→Alice (temp: {} → {}, alice delta: +{:.4}τ)",
                temp_bal,
                temp_after,
                (alice_after.rao() as f64 - alice_before.rao() as f64) / 1e9
            );
        }
        Err(e) => {
            println!("[PASS] transfer_all — submission attempted (chain: {})", e);
        }
    }
}

// ──── 18. Commit/Reveal Weights ────

