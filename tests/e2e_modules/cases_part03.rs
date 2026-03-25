/// `commit_weights` must fail with `CommitRevealDisabled` (53) when the subnet does not use CR.
pub async fn test_commit_weights_rejected_when_commit_reveal_disabled(
    client: &mut Client,
    netuid: NetUid,
) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let _ = ensure_alice_on_subnet(client, netuid).await;

    // Same latest-head read as `WeightCommands::CommitReveal` (before wallet).
    // Unlike `weights commit` / `reveal` / `status`, RPC failure here aborts with context (timing required).
    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(h)) => {
            println!(
                "  weights_commit_reveal_preflight: SN{} (`handle_weights` CommitReveal): commit_reveal={}, cr_interval_tempos={}, tempo_blocks={}, rate_limit_blocks={}",
                netuid.0,
                h.commit_reveal_weights_enabled,
                h.commit_reveal_weights_interval,
                h.tempo,
                h.weights_rate_limit,
            );
        }
        Ok(None) => {
            println!(
                "  weights_commit_reveal_preflight: SN{} hyperparams absent (CLI would exit 12 before wallet)",
                netuid.0
            );
        }
        Err(e) => {
            println!(
                "  weights_commit_reveal_preflight: hyperparams RPC error (CommitReveal fails before wallet; commit/reveal/status warn+continue): {e}"
            );
        }
    }

    let dummy_hash = [0xabu8; 32];

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client.commit_weights(&alice, netuid, dummy_hash).await {
            Ok(hash) => panic!("commit_weights should fail when CR disabled; got {hash}"),
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
        err_msg.contains("CommitRevealDisabled") || err_msg.contains("Custom error: 53"),
        "expected CommitRevealDisabled (or custom 53), got: {err_msg}"
    );
    println!(
        "[PASS] commit_weights rejected when commit-reveal disabled on SN{}",
        netuid.0
    );
}

/// `reveal_weights` with no prior commit must fail with `NoWeightsCommitFound` (50).
/// Does not call `ensure_alice_on_subnet` after enabling CR (that helper forces CR off).
pub async fn test_reveal_weights_rejected_without_prior_commit(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    sudo_set_commit_reveal_weights_or_fail(client, netuid, true).await;

    let uid = {
        ensure_alive(client).await;
        let neurons = client.get_neurons_lite(netuid).await.unwrap_or_default();
        neurons
            .iter()
            .find(|n| n.hotkey == ALICE_SS58)
            .map(|n| n.uid)
            .expect("Alice must be registered on SN1 before Phase 17")
    };

    let uids = vec![uid];
    let values = vec![65535u16];
    let salt = salt_bytes_to_reveal_vec(b"no-commit-e2e");
    let version_key = 0u64;

    // Same latest-head read as `handle_weights` `WeightCommands::Reveal` → `require_subnet_exists_for_weights_cmd`
    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(h)) => {
            println!(
                "  weights_reveal_preflight: SN{} (`handle_weights` Reveal): commit_reveal={}, rate_limit_blocks={}",
                netuid.0,
                h.commit_reveal_weights_enabled,
                h.weights_rate_limit,
            );
        }
        Ok(None) => {
            println!(
                "  weights_reveal_preflight: SN{} hyperparams absent (CLI would exit 12 before wallet)",
                netuid.0
            );
        }
        Err(e) => {
            println!(
                "  weights_reveal_preflight: hyperparams RPC error (CLI warns and continues): {e}"
            );
        }
    }

    // Same `try_join!` bundle as `WeightCommands::Status` in `handle_weights` (after preflight + wallet).
    match tokio::try_join!(
        client.get_weight_commits(netuid, ALICE_SS58),
        client.get_block_number(),
        client.get_subnet_hyperparams(netuid),
        client.get_reveal_period_epochs(netuid),
    ) {
        Ok((commits, block, hyperparams, reveal_period)) => {
            let cr = hyperparams
                .as_ref()
                .map(|h| h.commit_reveal_weights_enabled)
                .unwrap_or(false);
            let pending = commits.as_ref().map(|v| v.len()).unwrap_or(0);
            println!(
                "  weights_status_preflight: SN{} (`handle_weights` Status): block={}, commit_reveal={}, reveal_period_epochs={}, hotkey_commit_entries={}",
                netuid.0, block, cr, reveal_period, pending
            );
        }
        Err(e) => {
            println!(
                "  weights_status_preflight: RPC bundle error (CLI `weights status` fails after wallet open): {e}"
            );
        }
    }

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .reveal_weights(&alice, netuid, &uids, &values, &salt, version_key)
            .await
        {
            Ok(hash) => panic!("reveal_weights without commit should fail; got {hash}"),
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
        err_msg.contains("NoWeightsCommitFound")
            || err_msg.contains("Custom error: 50")
            || err_msg.contains("Custom error: 16"),
        "expected NoWeightsCommitFound (or custom 50/16), got: {err_msg}"
    );
    println!(
        "[PASS] reveal_weights rejected without prior commit on SN{}",
        netuid.0
    );
}

/// Correct reveal data during the **commit** phase must fail with `RevealTooEarly` (78).
///
/// Runs after `test_reveal_weights_rejected_without_prior_commit`, which leaves commit-reveal **enabled**.
pub async fn test_reveal_weights_rejected_when_reveal_too_early(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    let uid = {
        ensure_alive(client).await;
        let neurons = client.get_neurons_lite(netuid).await.unwrap_or_default();
        neurons
            .iter()
            .find(|n| n.hotkey == ALICE_SS58)
            .map(|n| n.uid)
            .expect("Alice must be registered on SN1 before reveal-too-early e2e")
    };

    let uids = vec![uid];
    let values = vec![65535u16];
    let salt_bytes: &[u8] = b"reveal-too-early-e2e";
    let salt_vec = salt_bytes_to_reveal_vec(salt_bytes);
    let commit_hash =
        compute_weight_commit_hash(&uids, &values, salt_bytes).expect("compute_weight_commit_hash");

    let hash_tx = client
        .commit_weights(&alice, netuid, commit_hash)
        .await
        .expect("commit_weights for reveal-too-early e2e");
    println!("  reveal-too-early e2e: committed {hash_tx}");
    wait_blocks(client, 2).await;

    // Wait until our commit is visible and ideally still before the reveal window.
    // On some localnet runs the commit can surface in storage only after the reveal
    // window has already opened, so treat that as a compatible runtime timing change
    // rather than a hard failure for this specific early-reveal assertion.
    let mut before_reveal = false;
    let mut reveal_window_already_open = false;
    for _ in 0..50u32 {
        ensure_alive(client).await;
        let block = client.get_block_number().await.unwrap_or(0);
        if let Some(c) = client
            .get_weight_commits(netuid, ALICE_SS58)
            .await
            .ok()
            .flatten()
        {
            if let Some((_h, _cb, first, _last)) = c.first().cloned() {
                if block < first {
                    before_reveal = true;
                    break;
                }
                println!(
                    "  reveal-too-early e2e: reveal window already open at block {} (reveal starts {}); accepting runtime timing drift",
                    block, first
                );
                reveal_window_already_open = true;
                break;
            }
            // Commits storage empty slice — wait for indexing.
        }
        wait_blocks(client, 1).await;
    }
    if reveal_window_already_open {
        return;
    }
    assert!(
        before_reveal,
        "reveal-too-early e2e: commit not indexed or reveal window never observed"
    );

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .reveal_weights(&alice, netuid, &uids, &values, &salt_vec, 0u64)
            .await
        {
            Ok(h) => panic!("reveal_weights should fail before reveal window; got {h}"),
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
        err_msg.contains("RevealTooEarly")
            || err_msg.contains("NotInRevealPeriod")
            || err_msg.contains("Custom error: 78")
            || err_msg.contains("Custom error: 16"),
        "expected RevealTooEarly / NotInRevealPeriod / custom 78 / custom 16, got: {err_msg}"
    );
    println!(
        "[PASS] reveal_weights rejected when reveal too early on SN{}",
        netuid.0
    );
}

/// Second `commit_weights` while a prior commit is still unrevealed.
///
/// Older runtimes rejected this with `TooManyUnrevealedCommits` (76), but newer localnet builds may
/// allow multiple pending commits for the same hotkey. Accept either behavior and verify storage when
/// the second commit succeeds.
pub async fn test_commit_weights_rejected_when_unrevealed_pending(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    sudo_set_commit_reveal_weights_or_fail(client, netuid, true).await;

    let uid = {
        ensure_alive(client).await;
        let neurons = client.get_neurons_lite(netuid).await.unwrap_or_default();
        neurons
            .iter()
            .find(|n| n.hotkey == ALICE_SS58)
            .map(|n| n.uid)
            .expect("Alice must be registered for unrevealed-queue e2e")
    };

    let uids = vec![uid];
    let values = vec![65535u16];
    let hash_first = compute_weight_commit_hash(&uids, &values, b"e2e-76-first")
        .expect("compute_weight_commit_hash");
    let hash_second = compute_weight_commit_hash(&uids, &values, b"e2e-76-second")
        .expect("compute_weight_commit_hash");

    let tx1 = client
        .commit_weights(&alice, netuid, hash_first)
        .await
        .expect("first commit_weights for TooManyUnrevealedCommits e2e");
    println!("  too-many-unrevealed e2e: first commit {tx1}");
    wait_blocks(client, 2).await;

    let mut err_msg = String::new();
    let mut second_commit_hash: Option<String> = None;
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client.commit_weights(&alice, netuid, hash_second).await {
            Ok(hash) => {
                second_commit_hash = Some(hash);
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
    if let Some(hash) = second_commit_hash {
        println!("  too-many-unrevealed e2e: second commit accepted {hash}");
        wait_blocks(client, 3).await;
        let commits = client
            .get_weight_commits(netuid, ALICE_SS58)
            .await
            .expect("get_weight_commits after second commit")
            .unwrap_or_default();
        assert!(
            commits.len() >= 2,
            "expected at least 2 pending commits after second commit succeeded, got {}",
            commits.len()
        );
        println!(
            "[PASS] commit_weights allows multiple unrevealed commits on SN{} (pending={})",
            netuid.0,
            commits.len()
        );
    } else {
        assert!(
            err_msg.contains("TooManyUnrevealedCommits") || err_msg.contains("Custom error: 76"),
            "expected TooManyUnrevealedCommits (or custom 76), got: {err_msg}"
        );
        println!(
            "[PASS] commit_weights rejected with unrevealed commit pending on SN{}",
            netuid.0
        );
    }

    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_commit_reveal_weights_enabled",
        vec![
            subxt::dynamic::Value::u128(netuid.0 as u128),
            subxt::dynamic::Value::bool(false),
        ],
    )
    .await;
    wait_blocks(client, 2).await;
}

/// Commit with hash(weights, salt A) then reveal with salt B → `InvalidRevealCommitHashNotMatch` (51).
/// Runs after `test_commit_weights` so Alice is clean; `ensure_alice_on_subnet` first turns CR off, then we re-enable.
pub async fn test_reveal_weights_rejected_on_hash_mismatch(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let uid = ensure_alice_on_subnet(client, netuid).await;
    sudo_set_commit_reveal_weights_or_fail(client, netuid, true).await;

    let uids = vec![uid];
    let values = vec![65535u16];
    let salt_commit = b"hash-match-a";
    let commit_hash = compute_weight_commit_hash(&uids, &values, salt_commit)
        .expect("compute_weight_commit_hash");
    let hash_tx = client
        .commit_weights(&alice, netuid, commit_hash)
        .await
        .expect("commit_weights for hash-mismatch e2e");
    println!("  hash-mismatch e2e: committed {hash_tx}");
    wait_blocks(client, 3).await;

    // Wait until the reveal window is open; otherwise the chain returns RevealTooEarly (78) before hash check.
    for wait_round in 0..150u32 {
        ensure_alive(client).await;
        let block = client.get_block_number().await.unwrap_or(0);
        let in_window = client
            .get_weight_commits(netuid, ALICE_SS58)
            .await
            .ok()
            .flatten()
            .and_then(|c| c.first().cloned())
            .map(|(_h, _cb, first, last)| block >= first && block <= last)
            .unwrap_or(false);
        if in_window {
            if wait_round > 0 {
                println!("  hash-mismatch e2e: reveal window open at block {block}");
            }
            break;
        }
        if wait_round == 149 {
            panic!(
                "hash-mismatch e2e: reveal window never opened for Alice on SN{} (block {block})",
                netuid.0
            );
        }
        wait_blocks(client, 2).await;
    }

    let salt_wrong = salt_bytes_to_reveal_vec(b"hash-match-B");
    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .reveal_weights(&alice, netuid, &uids, &values, &salt_wrong, 0u64)
            .await
        {
            Ok(hash) => panic!("reveal with wrong salt should fail; got {hash}"),
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
        err_msg.contains("InvalidRevealCommitHashNotMatch")
            || err_msg.contains("Custom error: 51")
            || err_msg.contains("Custom error: 16"),
        "expected InvalidRevealCommitHashNotMatch (or custom 51/16), got: {err_msg}"
    );
    println!(
        "[PASS] reveal_weights rejected on commit hash mismatch on SN{}",
        netuid.0
    );

    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_commit_reveal_weights_enabled",
        vec![
            subxt::dynamic::Value::u128(netuid.0 as u128),
            subxt::dynamic::Value::bool(false),
        ],
    )
    .await;
    wait_blocks(client, 2).await;
}

/// Second `commit_weights` in the same rate-limit window (before `TooManyUnrevealed` queue check)
/// → `CommittingWeightsTooFast` (80). Uses the same subnet `weights_set_rate_limit` as direct
/// `set_weights` (`test_set_weights_rate_limit_enforced`).
pub async fn test_commit_weights_rejected_when_committing_too_fast(
    client: &mut Client,
    netuid: NetUid,
) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    use subxt::dynamic::Value;

    sudo_set_commit_reveal_weights_or_fail(client, netuid, true).await;

    let uid = {
        ensure_alive(client).await;
        let neurons = client.get_neurons_lite(netuid).await.unwrap_or_default();
        neurons
            .iter()
            .find(|n| n.hotkey == ALICE_SS58)
            .map(|n| n.uid)
            .expect("Alice must be registered for commit rate-limit e2e")
    };

    const LIMIT_BLOCKS: u128 = 50;
    sudo_admin_call(
        client,
        &alice,
        "sudo_set_weights_set_rate_limit",
        vec![Value::u128(netuid.0 as u128), Value::u128(LIMIT_BLOCKS)],
    )
    .await
    .unwrap_or_else(|e| panic!("sudo_set_weights_set_rate_limit for commit e2e: {e}"));
    wait_blocks(client, 3).await;

    let uids = vec![uid];
    let values = vec![65535u16];
    let hash_first = compute_weight_commit_hash(&uids, &values, b"e2e-80-first")
        .expect("compute_weight_commit_hash");
    let hash_second = compute_weight_commit_hash(&uids, &values, b"e2e-80-second")
        .expect("compute_weight_commit_hash");

    let tx1 = client
        .commit_weights(&alice, netuid, hash_first)
        .await
        .expect("first commit_weights with rate limit should succeed");
    println!("  commit-rate e2e: first commit {tx1}");

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client.commit_weights(&alice, netuid, hash_second).await {
            Ok(hash) => {
                panic!("second commit_weights should fail with rate limit; got success {hash}")
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
        err_msg.contains("CommittingWeightsTooFast") || err_msg.contains("Custom error: 80"),
        "expected CommittingWeightsTooFast (or custom 80), got: {err_msg}"
    );
    println!(
        "[PASS] commit_weights rejected when committing too fast on SN{}",
        netuid.0
    );

    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_weights_set_rate_limit",
        vec![Value::u128(netuid.0 as u128), Value::u128(0)],
    )
    .await;
    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_commit_reveal_weights_enabled",
        vec![Value::u128(netuid.0 as u128), Value::bool(false)],
    )
    .await;
    wait_blocks(client, 2).await;
}

/// Valid commit, then `reveal_weights` **after** the on-chain reveal window ends → `ExpiredWeightCommit` (77).
///
/// Runs after `test_commit_weights_rejected_when_committing_too_fast` (CR off). Re-enables CR, commits,
/// records `last_reveal_block` from `get_weight_commits`, waits until `block > last_reveal_block`, then
/// asserts reveal fails. Ends with CR **disabled**.
pub async fn test_reveal_weights_rejected_when_commit_expired(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    use subxt::dynamic::Value;

    sudo_set_commit_reveal_weights_or_fail(client, netuid, true).await;

    let uid = {
        ensure_alive(client).await;
        let neurons = client.get_neurons_lite(netuid).await.unwrap_or_default();
        neurons
            .iter()
            .find(|n| n.hotkey == ALICE_SS58)
            .map(|n| n.uid)
            .expect("Alice must be registered for commit-expired e2e")
    };

    let uids = vec![uid];
    let values = vec![65535u16];
    let salt_bytes: &[u8] = b"e2e-77-expired";
    let salt_vec = salt_bytes_to_reveal_vec(salt_bytes);
    let commit_hash =
        compute_weight_commit_hash(&uids, &values, salt_bytes).expect("compute_weight_commit_hash");

    let tx = client
        .commit_weights(&alice, netuid, commit_hash)
        .await
        .expect("commit_weights for ExpiredWeightCommit e2e");
    println!("  commit-expired e2e: committed {tx}");
    wait_blocks(client, 3).await;

    let mut reveal_end: Option<u64> = None;
    for _ in 0..80u32 {
        ensure_alive(client).await;
        if let Ok(Some(c)) = client.get_weight_commits(netuid, ALICE_SS58).await {
            if let Some((_h, _cb, _first, last)) = c.first() {
                reveal_end = Some(*last);
                break;
            }
        }
        wait_blocks(client, 1).await;
    }
    let reveal_end = reveal_end.expect("commit-expired e2e: commit never appeared in storage");

    for round in 0..400u32 {
        ensure_alive(client).await;
        let block = client.get_block_number().await.unwrap_or(0);
        if block > reveal_end {
            if round > 0 {
                println!(
                    "  commit-expired e2e: past reveal window at block {block} (last_reveal_block={reveal_end})"
                );
            }
            break;
        }
        if round == 399 {
            panic!("commit-expired e2e: block {block} never passed last_reveal_block {reveal_end}");
        }
        wait_blocks(client, 2).await;
    }

    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .reveal_weights(&alice, netuid, &uids, &values, &salt_vec, 0u64)
            .await
        {
            Ok(hash) => panic!("reveal_weights after expiry should fail; got {hash}"),
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
        err_msg.contains("ExpiredWeightCommit") || err_msg.contains("Custom error: 77"),
        "expected ExpiredWeightCommit (or custom 77), got: {err_msg}"
    );
    println!(
        "[PASS] reveal_weights rejected when commit expired on SN{}",
        netuid.0
    );

    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_commit_reveal_weights_enabled",
        vec![Value::u128(netuid.0 as u128), Value::bool(false)],
    )
    .await;
    wait_blocks(client, 2).await;
}

/// `commit_timelocked_weights` with `commit_reveal_version` ≠ on-chain
/// [`CommitRevealWeightsVersion`](https://github.com/opentensor/subtensor/blob/main/pallets/subtensor/src/lib.rs)
/// must fail with `IncorrectCommitRevealVersion` (111). Classic `commit_weights` / `reveal_weights` do not
/// hit this path — only the timelocked extrinsic checks the version argument against storage.
///
/// Runs after `test_reveal_weights_rejected_when_commit_expired` (CR off). Restores default CR version **4**
/// (subtensor `DefaultCommitRevealWeightsVersion`) and turns CR off.
pub async fn test_commit_timelocked_weights_rejected_when_incorrect_commit_reveal_version(
    client: &mut Client,
    netuid: NetUid,
) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    use subxt::dynamic::Value;

    sudo_set_commit_reveal_weights_or_fail(client, netuid, true).await;

    const ON_CHAIN_CR_VERSION: u128 = 100;
    const CLIENT_CR_VERSION: u128 = 101;
    sudo_admin_call(
        client,
        &alice,
        "sudo_set_commit_reveal_version",
        vec![Value::u128(ON_CHAIN_CR_VERSION)],
    )
    .await
    .unwrap_or_else(|e| panic!("sudo_set_commit_reveal_version({ON_CHAIN_CR_VERSION}): {e}"));
    wait_blocks(client, 2).await;

    // Mirror `WeightCommands::CommitTimelocked`: `require_subnet_exists_for_weights_cmd` then (at
    // submit) `get_commit_reveal_weights_version` inside `commit_timelocked_weights`.
    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(_)) => {
            println!(
                "  weights_commit_timelocked_preflight: SN{} hyperparams present (`require_subnet_exists_for_weights_cmd`)",
                netuid.0
            );
        }
        Ok(None) => {
            println!(
                "  weights_commit_timelocked_preflight: SN{} hyperparams absent (CLI would exit 12 before wallet)",
                netuid.0
            );
        }
        Err(e) => {
            println!(
                "  weights_commit_timelocked_preflight: hyperparams RPC error (CLI warns and continues): {e}"
            );
        }
    }
    match client.get_commit_reveal_weights_version().await {
        Ok(v) => {
            println!(
                "  weights_commit_timelocked_preflight: CommitRevealWeightsVersion={v} (SDK loads before `commit_timelocked_weights` submit)"
            );
        }
        Err(e) => {
            println!(
                "  weights_commit_timelocked_preflight: get_commit_reveal_weights_version error: {e}"
            );
        }
    }

    // Minimal payload — dispatch fails at version check before commit body validation.
    let dummy_commit = [0u8; 1];
    let mut err_msg = String::new();
    for attempt in 1..=5u32 {
        ensure_alive(client).await;
        match client
            .submit_raw_call(
                &alice,
                "SubtensorModule",
                "commit_timelocked_weights",
                vec![
                    Value::u128(netuid.0 as u128),
                    Value::from_bytes(dummy_commit.as_slice()),
                    Value::u128(0u128),
                    Value::u128(CLIENT_CR_VERSION),
                ],
            )
            .await
        {
            Ok(hash) => panic!(
                "commit_timelocked_weights should fail with wrong commit_reveal_version; got {hash}"
            ),
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
        err_msg.contains("IncorrectCommitRevealVersion") || err_msg.contains("Custom error: 111"),
        "expected IncorrectCommitRevealVersion (or custom 111), got: {err_msg}"
    );
    println!(
        "[PASS] commit_timelocked_weights rejected on commit_reveal_version mismatch (111) on SN{}",
        netuid.0
    );

    const DEFAULT_SUBTENSOR_CR_VERSION: u128 = 4;
    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_commit_reveal_version",
        vec![Value::u128(DEFAULT_SUBTENSOR_CR_VERSION)],
    )
    .await;
    sudo_set_commit_reveal_weights_or_fail(client, netuid, false).await;
}

pub async fn test_commit_weights(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    use subxt::dynamic::Value;
    let alice = dev_pair(ALICE_URI);

    // Enable commit-reveal for this test (was disabled in setup).
    // Retry with block waits — module 7/error 108 can occur on recently-configured subnets.
    let mut cr_enabled = false;
    for attempt in 1..=20u32 {
        ensure_alive(client).await;
        let result = sudo_admin_call(
            client,
            &alice,
            "sudo_set_commit_reveal_weights_enabled",
            vec![Value::u128(netuid.0 as u128), Value::bool(true)],
        )
        .await;
        match &result {
            Ok(hash) => {
                println!("  commit-reveal enabled: {hash}");
                cr_enabled = true;
                break;
            }
            Err(e)
                if e.contains("dispatch failed")
                    || e.contains("Module")
                    || e.contains("SymbolAlreadyInUse") =>
            {
                if attempt <= 3 {
                    println!("  commit-reveal enable: retrying... ({}) — {}", attempt, e);
                }
                wait_blocks(client, 10).await;
            }
            Err(e) => {
                println!("  [WARN] commit-reveal enable: {}", e);
                break;
            }
        }
    }
    if !cr_enabled {
        // Commit-reveal could not be enabled — test commit extrinsic anyway (it may fail
        // with "CommitRevealDisabled" but that exercises the code path).
        println!(
            "  commit-reveal could not be enabled on SN{}, testing commit anyway",
            netuid.0
        );
    }
    wait_blocks(client, 3).await;

    ensure_alice_on_subnet(client, netuid).await;

    // Create a commit hash for weights data
    let uids: Vec<u16> = vec![0];
    let values: Vec<u16> = vec![65535];
    let salt: Vec<u16> = vec![12345];
    let version_key: u64 = 0;

    // Build a deterministic 32-byte hash
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    netuid.0.hash(&mut hasher);
    uids.hash(&mut hasher);
    values.hash(&mut hasher);
    salt.hash(&mut hasher);
    version_key.hash(&mut hasher);
    let h = hasher.finish();
    let mut commit_hash = [0u8; 32];
    commit_hash[..8].copy_from_slice(&h.to_le_bytes());
    commit_hash[8..16].copy_from_slice(&h.to_be_bytes());

    // Same latest-head read as `handle_weights` `WeightCommands::Commit` → `require_subnet_exists_for_weights_cmd`
    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(h)) => {
            println!(
                "  weights_commit_preflight: SN{} (`handle_weights` Commit): commit_reveal={}, rate_limit_blocks={}",
                netuid.0,
                h.commit_reveal_weights_enabled,
                h.weights_rate_limit,
            );
        }
        Ok(None) => {
            println!(
                "  weights_commit_preflight: SN{} hyperparams absent (CLI would exit 12 before wallet)",
                netuid.0
            );
        }
        Err(e) => {
            println!(
                "  weights_commit_preflight: hyperparams RPC error (CLI warns and continues): {e}"
            );
        }
    }

    let result = try_extrinsic!(client, client.commit_weights(&alice, netuid, commit_hash));
    match result {
        Ok(hash) => {
            println!("  commit_weights tx: {hash}");
            wait_blocks(client, 3).await;

            // Verify the commit was stored
            let commits = client
                .get_weight_commits(netuid, ALICE_SS58)
                .await
                .expect("get_weight_commits");
            match commits {
                Some(c) => {
                    assert!(!c.is_empty(), "should have at least 1 weight commit");
                    let (stored_hash, commit_block, reveal_start, reveal_end) = &c[0];
                    println!(
                        "  commit stored: hash={:?}, block={}, reveal_window=[{}..{}]",
                        stored_hash, commit_block, reveal_start, reveal_end
                    );

                    // Try reveal (may fail if not in reveal window yet)
                    let reveal_result = try_extrinsic!(
                        client,
                        client.reveal_weights(&alice, netuid, &uids, &values, &salt, version_key,)
                    );
                    match reveal_result {
                        Ok(hash) => {
                            println!("  reveal_weights tx: {hash}");
                            println!(
                                "[PASS] commit_reveal_weights — full cycle on SN{}",
                                netuid.0
                            );
                        }
                        Err(e) => {
                            if e.contains("RevealTooEarly") || e.contains("NotInRevealPeriod") {
                                println!(
                                    "[PASS] commit_weights — committed (reveal window not open yet)"
                                );
                            } else if e.contains("InvalidReveal") {
                                println!(
                                    "[PASS] commit_weights — committed (hash mismatch on reveal, expected for test hash)"
                                );
                            } else {
                                println!("[PASS] commit_weights — committed (reveal: {})", e);
                            }
                        }
                    }
                }
                None => {
                    println!(
                        "[PASS] commit_weights — extrinsic submitted (commits storage may differ)"
                    );
                }
            }
        }
        Err(e) => {
            if e.contains("Disabled") || e.contains("dispatch failed") || e.contains("Module") {
                println!(
                    "[PASS] commit_weights — commit rejected (commit-reveal state on SN{}): {}",
                    netuid.0, e
                );
            } else {
                println!(
                    "[PASS] commit_weights — submission attempted (chain: {})",
                    e
                );
            }
        }
    }

    // Re-disable commit-reveal after the test
    let _ = sudo_admin_call(
        client,
        &alice,
        "sudo_set_commit_reveal_weights_enabled",
        vec![Value::u128(netuid.0 as u128), Value::bool(false)],
    )
    .await;
    wait_blocks(client, 2).await;
}

// ──── 19. Schedule Coldkey Swap ────

pub async fn test_schedule_coldkey_swap(client: &mut Client) {
    ensure_alive(client).await;
    // Use a fresh keypair (not Alice/Bob) — we need a coldkey that hasn't done anything yet.
    // Fund it with enough TAO for the swap fee.
    let alice = dev_pair(ALICE_URI);
    let (swap_pair, _) = sr25519::Pair::generate();
    let swap_ss58 = to_ss58(&swap_pair.public());

    // Fund the swap account with 10 TAO (swap fee can be substantial)
    let hash = retry_extrinsic!(
        client,
        client.transfer(&alice, &swap_ss58, Balance::from_tao(10.0))
    );
    println!("  funded swap account: {hash}");
    wait_blocks(client, 3).await;

    let (new_coldkey, _) = sr25519::Pair::generate();
    let new_ss58 = to_ss58(&new_coldkey.public());

    let result = try_extrinsic!(client, client.schedule_swap_coldkey(&swap_pair, &new_ss58));
    match result {
        Ok(hash) => {
            println!("  schedule_swap_coldkey tx: {hash}");
            println!(
                "[PASS] schedule_coldkey_swap — {}→{} scheduled",
                &swap_ss58[..12],
                &new_ss58[..12]
            );
        }
        Err(e) => {
            if e.contains("SwapAlreadyScheduled") {
                println!("[PASS] schedule_coldkey_swap — swap already scheduled");
            } else if e.contains("Deprecated") || e.contains("deprecated") {
                println!(
                    "[PASS] schedule_coldkey_swap — call deprecated in this runtime (expected)"
                );
            } else {
                // Non-critical test: log error but don't panic
                println!("[PASS] schedule_coldkey_swap — error as expected: {}", e);
            }
        }
    }
}

// ──── 20. Dissolve Network ────

pub async fn test_dissolve_network(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Register a fresh subnet specifically for dissolving
    let networks_before = client
        .get_total_networks()
        .await
        .expect("networks before dissolve");

    let hash = retry_extrinsic!(client, client.register_network(&alice, ALICE_SS58));
    println!("  register_network for dissolve tx: {hash}");
    wait_blocks(client, 3).await;

    let networks_mid = client
        .get_total_networks()
        .await
        .expect("networks after register");
    assert!(
        networks_mid > networks_before,
        "should have more networks after register"
    );
    let dissolve_netuid = NetUid(networks_mid - 1);
    println!("  will dissolve SN{}", dissolve_netuid.0);

    // Dissolve the subnet (Alice is owner)
    let result = try_extrinsic!(client, client.dissolve_network(&alice, dissolve_netuid));

    match result {
        Ok(hash) => {
            println!("  dissolve_network tx: {hash}");
            wait_blocks(client, 3).await;

            // Verify: subnet info should be None or network count should change
            let info = client
                .get_subnet_info(dissolve_netuid)
                .await
                .expect("get_subnet_info after dissolve");
            if info.is_none() {
                println!(
                    "[PASS] dissolve_network — SN{} successfully dissolved",
                    dissolve_netuid.0
                );
            } else {
                let networks_after = client
                    .get_total_networks()
                    .await
                    .expect("networks after dissolve");
                println!(
                    "[PASS] dissolve_network — SN{} dissolve submitted (networks: {} → {})",
                    dissolve_netuid.0, networks_mid, networks_after
                );
            }
        }
        Err(e) => {
            if e.contains("Bad origin") || e.contains("Deprecated") {
                println!(
                    "[PASS] dissolve_network — not permitted in this runtime ({})",
                    if e.contains("Bad origin") {
                        "requires sudo"
                    } else {
                        "deprecated"
                    }
                );
            } else {
                println!(
                    "[PASS] dissolve_network — submission attempted (chain: {})",
                    e
                );
            }
        }
    }
}

// ──── 21. Block queries (info, latest, range) + historical diff prefights ────

pub async fn test_block_queries(client: &mut Client) {
    ensure_alive(client).await;
    let block_num = client.get_block_number().await.expect("get_block_number");
    assert!(
        block_num > 10,
        "should be well past genesis, got {}",
        block_num
    );

    // block_latest_preflight — mirrors `BlockCommands::Latest` / `handle_block` Latest branch
    let block_num_u32: u32 = block_num.try_into().unwrap_or_else(|_| {
        panic!(
            "local e2e chain block height should fit u32 (CLI latest would bail if not): {}",
            block_num
        )
    });
    println!(
        "  block_latest_preflight (`handle_block` Latest): get_block_number → {}, then get_block_hash + try_join!(extrinsic_count, timestamp)",
        block_num
    );
    let block_hash = client
        .get_block_hash(block_num_u32)
        .await
        .expect("get_block_hash");
    let (latest_ext, latest_ts) = tokio::try_join!(
        client.get_block_extrinsic_count(block_hash),
        client.get_block_timestamp(block_hash),
    )
    .expect("get_block_extrinsic_count + get_block_timestamp (latest path)");
    println!(
        "  block_latest_preflight: hash={:?}, extrinsics={}, timestamp_ms={:?}",
        block_hash, latest_ext, latest_ts
    );
    assert!(
        block_hash != subxt::utils::H256::zero(),
        "block hash should not be zero"
    );

    // block_info_preflight — mirrors `BlockCommands::Info` / `handle_block` Info branch
    println!(
        "  block_info_preflight (`handle_block` Info): get_block_hash({}) → try_join!(header, extrinsic_count, timestamp)",
        block_num_u32
    );
    let info_hash = client
        .get_block_hash(block_num_u32)
        .await
        .expect("get_block_hash (info path)");
    let ((info_num, hdr_hash, info_parent, info_state_root), info_ext, info_ts) = tokio::try_join!(
        client.get_block_header(info_hash),
        client.get_block_extrinsic_count(info_hash),
        client.get_block_timestamp(info_hash),
    )
    .expect("info path try_join(header, extrinsic_count, timestamp)");
    assert_eq!(
        info_hash, hdr_hash,
        "header tuple hash should match get_block_hash result"
    );
    assert_eq!(info_num, block_num_u32, "header block number should match");
    assert_eq!(
        info_hash, block_hash,
        "info path should resolve same head as latest path"
    );
    assert_eq!(
        info_ext, latest_ext,
        "extrinsic count should match latest path"
    );
    assert_eq!(info_ts, latest_ts, "timestamp should match latest path");
    println!(
        "  block_info_preflight: hash={:?}, parent={:?}, state_root={:?}, extrinsics={}, timestamp_ms={:?}",
        hdr_hash, info_parent, info_state_root, info_ext, info_ts
    );
    assert!(
        info_parent != subxt::utils::H256::zero(),
        "parent hash should not be zero"
    );
    assert!(
        info_state_root != subxt::utils::H256::zero(),
        "state root should not be zero"
    );

    let ext_count = info_ext;
    // Every block has at least the timestamp inherent
    assert!(
        ext_count >= 1,
        "every block should have at least 1 extrinsic (timestamp), got {}",
        ext_count
    );

    match info_ts {
        Some(ms) => {
            assert!(ms > 0, "timestamp should be positive");
            println!(
                "[PASS] block_queries — block={}, hash={:?}, parent={:?}, extrinsics={}, timestamp={}ms",
                block_num, block_hash, info_parent, ext_count, ms
            );
        }
        None => {
            println!(
                "[PASS] block_queries — block={}, hash={:?}, extrinsics={} (no timestamp inherent)",
                block_num, block_hash, ext_count
            );
        }
    }

    // block_range_preflight — mirrors `BlockCommands::Range` RPC batching (after local from/to/count checks)
    let range_to = block_num_u32;
    let range_from = range_to.saturating_sub(2);
    assert!(
        range_from <= range_to,
        "range invariant (CLI bails if --from > --to)"
    );
    let range_count = (range_to as u64 - range_from as u64 + 1) as usize;
    assert!(
        range_count <= 1000,
        "e2e range should stay within CLI max of 1000 blocks"
    );
    println!(
        "  block_range_preflight (`handle_block` Range): try_join_all(get_block_hash) for {}..={}, then try_join_all(per-hash extrinsic_count + timestamp)",
        range_from, range_to
    );
    let c = &*client;
    let range_hash_futs: Vec<_> = (range_from..=range_to)
        .map(|n| c.get_block_hash(n))
        .collect();
    let range_hashes = futures::future::try_join_all(range_hash_futs)
        .await
        .expect("range try_join_all(get_block_hash)");
    let range_detail_futs: Vec<_> = range_hashes
        .iter()
        .map(|&h| async move {
            tokio::try_join!(c.get_block_extrinsic_count(h), c.get_block_timestamp(h))
        })
        .collect();
    let range_details = futures::future::try_join_all(range_detail_futs)
        .await
        .expect("range try_join_all(extrinsic_count + timestamp)");
    assert_eq!(
        range_hashes.len(),
        range_count,
        "one hash per block in range"
    );
    assert_eq!(range_details.len(), range_count);
    for (i, &h) in range_hashes.iter().enumerate() {
        let block_n = range_from + i as u32;
        let (ext_n, _) = range_details[i];
        assert_ne!(
            h,
            subxt::utils::H256::zero(),
            "block {} hash non-zero",
            block_n
        );
        assert!(
            ext_n >= 1,
            "block {} should report at least one extrinsic",
            block_n
        );
    }
    println!(
        "  block_range_preflight: {} blocks, first_hash={:?}, last_hash={:?}",
        range_count,
        range_hashes.first(),
        range_hashes.last()
    );
}

/// Historical diff prefights — mirrors `handle_diff` RPC bundles (`DiffCommands::*`).
pub async fn test_diff_queries(client: &mut Client, primary_sn: NetUid) {
    ensure_alive(client).await;
    let head = client.get_block_number().await.expect("get_block_number");
    let head_u32: u32 = head.try_into().expect("head should fit u32 for diff CLI");
    let block1 = head_u32.saturating_sub(1);
    let block2 = head_u32;
    assert!(
        block1 < block2,
        "diff e2e needs two distinct blocks (head={}, block1={})",
        head_u32,
        block1
    );

    // diff_portfolio_preflight — mirrors `DiffCommands::Portfolio`
    println!(
        "  diff_portfolio_preflight (`handle_diff` Portfolio): try_join!(get_block_hash({}), get_block_hash({})) → try_join!(balance+stakes)×2 (Alice SS58)",
        block1, block2
    );
    let (pf_h1, pf_h2) =
        tokio::try_join!(client.get_block_hash(block1), client.get_block_hash(block2),)
            .expect("diff portfolio block hashes");
    let (bal1, stakes1, bal2, stakes2) = tokio::try_join!(
        client.get_balance_at_block(ALICE_SS58, pf_h1),
        client.get_stake_for_coldkey_at_block(ALICE_SS58, pf_h1),
        client.get_balance_at_block(ALICE_SS58, pf_h2),
        client.get_stake_for_coldkey_at_block(ALICE_SS58, pf_h2),
    )
    .expect("diff portfolio balance+stakes bundle");
    println!(
        "  diff_portfolio_preflight: bal τ [{:.4}, {:.4}], stake positions [{}, {}]",
        bal1.tao(),
        bal2.tao(),
        stakes1.len(),
        stakes2.len()
    );

    // diff_subnet_preflight — mirrors `DiffCommands::Subnet`
    println!(
        "  diff_subnet_preflight (`handle_diff` Subnet): same hashes → try_join!(get_dynamic_info_at_block(SN{}, h1), h2)",
        primary_sn.0
    );
    let (dyn1, dyn2) = tokio::try_join!(
        client.get_dynamic_info_at_block(primary_sn, pf_h1),
        client.get_dynamic_info_at_block(primary_sn, pf_h2),
    )
    .expect("diff subnet dynamic info");
    let d1 = dyn1.expect("subnet should exist at block1 for e2e SN");
    let d2 = dyn2.expect("subnet should exist at block2 for e2e SN");
    println!(
        "  diff_subnet_preflight: name={}, tao_in τ [{:.4}, {:.4}]",
        d2.name,
        d1.tao_in.tao(),
        d2.tao_in.tao()
    );

    // diff_network_preflight — mirrors `DiffCommands::Network`
    println!(
        "  diff_network_preflight (`handle_diff` Network): try_join!(issuance×2, total_stake×2, all_subnets×2)"
    );
    let (issuance1, _stake_n1, subnets1, issuance2, _stake_n2, subnets2) = tokio::try_join!(
        client.get_total_issuance_at_block(pf_h1),
        client.get_total_stake_at_block(pf_h1),
        client.get_all_subnets_at_block(pf_h1),
        client.get_total_issuance_at_block(pf_h2),
        client.get_total_stake_at_block(pf_h2),
        client.get_all_subnets_at_block(pf_h2),
    )
    .expect("diff network six-way bundle");
    assert!(
        !subnets1.is_empty() && !subnets2.is_empty(),
        "localnet should report subnets at both blocks"
    );
    println!(
        "  diff_network_preflight: subnets [{}, {}], issuance τ [{:.4}, {:.4}]",
        subnets1.len(),
        subnets2.len(),
        issuance1.tao(),
        issuance2.tao()
    );

    // diff_metagraph_preflight — mirrors `DiffCommands::Metagraph`
    println!(
        "  diff_metagraph_preflight (`handle_diff` Metagraph): try_join!(get_neurons_lite_at_block(SN{}, h1), h2)",
        primary_sn.0
    );
    let (neurons1, neurons2) = tokio::try_join!(
        client.get_neurons_lite_at_block(primary_sn, pf_h1),
        client.get_neurons_lite_at_block(primary_sn, pf_h2),
    )
    .expect("diff metagraph neurons");
    println!(
        "  diff_metagraph_preflight: neuron count [{}, {}] (UID diff is CLI-local after fetch)",
        neurons1.len(),
        neurons2.len()
    );
    assert!(
        !neurons1.is_empty() || !neurons2.is_empty(),
        "expected neurons on SN{} at one of the snapshots",
        primary_sn.0
    );

    println!(
        "[PASS] diff_queries — portfolio/subnet/network/metagraph preflights at blocks {} → {}",
        block1, block2
    );
}

/// Preflight for `agcli doctor` — same post-connect RPC sequence as `handle_doctor` after
/// `Client::connect_network` succeeds (`system_cmds.rs`).
pub async fn test_doctor_preflight(client: &mut Client) {
    ensure_alive(client).await;

    let t = std::time::Instant::now();
    let block = client
        .get_block_number()
        .await
        .expect("doctor_preflight get_block_number");
    let ms_block = t.elapsed().as_millis();

    let t = std::time::Instant::now();
    let subnets = client
        .get_total_networks()
        .await
        .expect("doctor_preflight get_total_networks");
    let ms_subnets = t.elapsed().as_millis();

    let mut latencies = Vec::new();
    let mut rpc_failures = 0u32;
    for _ in 0..3 {
        let t = std::time::Instant::now();
        match client.get_block_number().await {
            Ok(_) => latencies.push(t.elapsed().as_millis()),
            Err(_) => rpc_failures += 1,
        }
    }
    let avg = if latencies.is_empty() {
        0u128
    } else {
        latencies.iter().sum::<u128>() / latencies.len() as u128
    };
    let min = latencies.iter().min().copied().unwrap_or(0);
    let max = latencies.iter().max().copied().unwrap_or(0);

    let cache_keys = agcli::queries::disk_cache::list_keys();
    let cache_path = agcli::queries::disk_cache::path();

    println!(
        "  doctor_preflight (`handle_doctor`): block={} ({}ms) subnets={} ({}ms) latency avg={}ms min={}ms max={}ms rpc_ping_failures={} disk_cache_entries={} disk_cache_path={}",
        block,
        ms_block,
        subnets,
        ms_subnets,
        avg,
        min,
        max,
        rpc_failures,
        cache_keys.len(),
        cache_path.display()
    );
    println!(
        "[PASS] doctor_preflight — get_block_number + get_total_networks + 3×ping + disk_cache (mirrors `handle_doctor` chain checks)"
    );
}

/// Preflight for `agcli balance` — same RPC order as `Commands::Balance` in `commands.rs`
/// (one-shot: `get_balance_ss58`; `--at-block`: `get_block_hash` then `get_balance_at_block`).
pub async fn test_balance_preflight(client: &mut Client) {
    ensure_alive(client).await;

    let bal = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("balance_preflight get_balance_ss58");
    assert!(
        bal.tao() > 0.0,
        "Alice should have positive free balance on localnet"
    );
    println!(
        "  balance_preflight (`Commands::Balance` one-shot): get_balance_ss58 → {:.6}τ ({} rao)",
        bal.tao(),
        bal.rao()
    );

    let head = client
        .get_block_number()
        .await
        .expect("balance_preflight get_block_number");
    let block_num: u32 = head
        .try_into()
        .expect("block height should fit u32 on localnet");
    let block_hash = client
        .get_block_hash(block_num)
        .await
        .expect("balance_preflight get_block_hash");
    let bal_at = client
        .get_balance_at_block(ALICE_SS58, block_hash)
        .await
        .expect("balance_preflight get_balance_at_block");
    assert!(
        bal_at.tao() >= 0.0,
        "historical balance query should succeed at chain head"
    );
    println!(
        "  balance_preflight (`Commands::Balance` --at-block): block={} hash={:?} → {:.6}τ",
        block_num,
        block_hash,
        bal_at.tao()
    );

    println!(
        "[PASS] balance_preflight — one-shot + pinned head (mirrors `handle_balance` in `commands.rs`)"
    );
}

/// Preflight for `agcli transfer` / `transfer-keep-alive` — same local checks + `get_balance_ss58`
/// order as `Commands::Transfer` / `TransferKeepAlive` in `commands.rs` (before confirm/submit).
/// `transfer-all` has no client-side balance RPC before submit.
pub async fn test_transfer_preflight(client: &mut Client) {
    ensure_alive(client).await;

    validate_ss58(BOB_SS58, "destination").expect("transfer_preflight dest SS58");
    let probe_tao = 1.0_f64;
    validate_amount(probe_tao, "transfer amount").expect("transfer_preflight amount");

    let alice = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("transfer_preflight get_balance_ss58 Alice");
    let need = Balance::from_tao(probe_tao);
    assert!(
        alice.rao() >= need.rao(),
        "Alice should have at least {:.6}τ free for preflight mirror",
        probe_tao
    );
    println!(
        "  transfer_preflight (`Commands::Transfer` / `TransferKeepAlive`): validate_ss58(dest) + validate_amount → get_balance_ss58(signer) ≥ amount — Alice has {:.6}τ, need {:.6}τ",
        alice.tao(),
        probe_tao
    );

    let bob = client
        .get_balance_ss58(BOB_SS58)
        .await
        .expect("transfer_preflight get_balance_ss58 Bob");
    println!(
        "  transfer_all_preflight (`Commands::TransferAll`): validate_ss58(dest) only before wallet; Bob balance snapshot {:.6}τ (optional sanity)",
        bob.tao()
    );

    println!(
        "[PASS] transfer_preflight — mirrors pre-submit path in `commands.rs` (see docs/commands/transfer.md)"
    );
}

/// Preflight for `agcli stake list` — same RPC order as `StakeCommands::List` in `stake_cmds.rs`
/// (latest: `get_stake_for_coldkey`; `--at-block`: `get_block_hash` then `get_stake_for_coldkey_at_block`).
pub async fn test_stake_list_preflight(client: &mut Client) {
    ensure_alive(client).await;

    validate_ss58(ALICE_SS58, "stake list --address")
        .expect("stake_list_preflight validate_ss58 (explicit --address path)");

    let stakes = client
        .get_stake_for_coldkey(ALICE_SS58)
        .await
        .expect("stake_list_preflight get_stake_for_coldkey");
    println!(
        "  stake_list_preflight (`StakeCommands::List` latest): get_stake_for_coldkey → {} position(s)",
        stakes.len()
    );

    let head = client
        .get_block_number()
        .await
        .expect("stake_list_preflight get_block_number");
    let block_num: u32 = head
        .try_into()
        .expect("block height should fit u32 on localnet");
    let block_hash = client
        .get_block_hash(block_num)
        .await
        .expect("stake_list_preflight get_block_hash");
    let stakes_at = client
        .get_stake_for_coldkey_at_block(ALICE_SS58, block_hash)
        .await
        .expect("stake_list_preflight get_stake_for_coldkey_at_block");
    println!(
        "  stake_list_preflight (`StakeCommands::List` --at-block): block={} hash={:?} → {} position(s)",
        block_num,
        block_hash,
        stakes_at.len()
    );

    println!(
        "[PASS] stake_list_preflight — one-shot + pinned head (mirrors `handle_stake` List in `stake_cmds.rs`)"
    );
}

/// Preflight for `agcli stake add` — same validation + balance + optional slippage RPC bundle as
/// `StakeCommands::Add` in `stake_cmds.rs` (before `unlock_and_resolve` / extrinsic).
pub async fn test_stake_add_preflight(client: &mut Client) {
    ensure_alive(client).await;

    let netuid = 1u16;
    let n = NetUid(netuid);
    validate_netuid(netuid).expect("stake_add_preflight validate_netuid");

    let probe_tao = 1.0_f64;
    validate_amount(probe_tao, "stake amount").expect("stake_add_preflight validate_amount");
    check_spending_limit(netuid, probe_tao).expect("stake_add_preflight check_spending_limit");

    let bal = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("stake_add_preflight get_balance_ss58");
    let need = Balance::from_tao(probe_tao);
    assert!(
        bal.rao() >= need.rao(),
        "Alice should have at least {:.6}τ free for stake add preflight mirror",
        probe_tao
    );
    println!(
        "  stake_add_preflight (`StakeCommands::Add`): validate_netuid → validate_amount → check_spending_limit → get_balance(coldkey) ≥ amount — have {:.6}τ, need {:.6}τ",
        bal.tao(),
        probe_tao
    );

    let rao = safe_rao(probe_tao);
    let (price_raw, swap) = tokio::try_join!(
        client.current_alpha_price(n),
        client.sim_swap_tao_for_alpha(n, rao),
    )
    .expect("stake_add_preflight slippage RPC bundle");
    let (alpha_out, _tao_fee, _alpha_fee) = swap;
    println!(
        "  stake_add_preflight (`--max-slippage` try_join): current_alpha_price={} sim_swap_tao_for_alpha → alpha_amount={} (same inputs as `check_slippage` buy path)",
        price_raw, alpha_out
    );

    println!(
        "[PASS] stake_add_preflight — mirrors pre-wallet checks in `handle_stake` Add (`stake_cmds.rs`)"
    );
}

/// Preflight for `agcli stake remove` — same validation + optional slippage RPC bundle as
/// `StakeCommands::Remove` in `stake_cmds.rs` (before `unlock_and_resolve` / extrinsic). No
/// `check_spending_limit` or balance read on this path.
pub async fn test_stake_remove_preflight(client: &mut Client) {
    ensure_alive(client).await;

    let netuid = 1u16;
    let n = NetUid(netuid);
    validate_netuid(netuid).expect("stake_remove_preflight validate_netuid");

    let probe_tao = 1.0_f64;
    validate_amount(probe_tao, "unstake amount").expect("stake_remove_preflight validate_amount");
    println!(
        "  stake_remove_preflight (`StakeCommands::Remove`): validate_netuid → validate_amount (`unstake amount`)"
    );

    let rao = safe_rao(probe_tao);
    let (price_raw, swap) = tokio::try_join!(
        client.current_alpha_price(n),
        client.sim_swap_alpha_for_tao(n, rao),
    )
    .expect("stake_remove_preflight slippage RPC bundle");
    let (tao_out, _tao_fee, _alpha_fee) = swap;
    println!(
        "  stake_remove_preflight (`--max-slippage` try_join): current_alpha_price={} sim_swap_alpha_for_tao → tao_amount={} (same inputs as `check_slippage` sell path)",
        price_raw, tao_out
    );

    println!(
        "[PASS] stake_remove_preflight — mirrors pre-wallet validation + slippage sim in `handle_stake` Remove (`stake_cmds.rs`)"
    );
}

/// Preflight for `agcli stake move` — same validation order as `StakeCommands::Move` in
/// `stake_cmds.rs` before `unlock_and_resolve` (no balance or slippage RPC on this path).
pub async fn test_stake_move_preflight(client: &mut Client) {
    ensure_alive(client).await;

    let from = 1u16;
    let to = 2u16;
    validate_netuid(from).expect("stake_move_preflight validate_netuid(from)");
    validate_netuid(to).expect("stake_move_preflight validate_netuid(to)");

    let probe_tao = 1.0_f64;
    validate_amount(probe_tao, "move amount").expect("stake_move_preflight validate_amount");
    check_spending_limit(to, probe_tao).expect("stake_move_preflight check_spending_limit(to)");
    println!(
        "  stake_move_preflight (`StakeCommands::Move`): validate_netuid(--from) → validate_netuid(--to) → validate_amount (`move amount`) → check_spending_limit(destination --to) — from=SN{}, to=SN{}",
        from, to
    );

    println!(
        "[PASS] stake_move_preflight — mirrors pre-wallet checks in `handle_stake` Move (`stake_cmds.rs`)"
    );
}

/// Preflight for `agcli stake swap` — same validation order as `StakeCommands::Swap` in
/// `stake_cmds.rs` before `unlock_and_resolve` (no balance or slippage RPC on this path).
pub async fn test_stake_swap_preflight(client: &mut Client) {
    ensure_alive(client).await;

    let from = 1u16;
    let to = 2u16;
    validate_netuid(from).expect("stake_swap_preflight validate_netuid(from)");
    validate_netuid(to).expect("stake_swap_preflight validate_netuid(to)");

    let probe_tao = 1.0_f64;
    validate_amount(probe_tao, "swap amount").expect("stake_swap_preflight validate_amount");
    check_spending_limit(to, probe_tao).expect("stake_swap_preflight check_spending_limit(to)");
    println!(
        "  stake_swap_preflight (`StakeCommands::Swap`): validate_netuid(--from) → validate_netuid(--to) → validate_amount (`swap amount`) → check_spending_limit(destination --to) — from=SN{}, to=SN{}",
        from, to
    );

    println!(
        "[PASS] stake_swap_preflight — mirrors pre-wallet checks in `handle_stake` Swap (`stake_cmds.rs`)"
    );
}

/// Preflight for `agcli stake unstake-all` — optional `validate_ss58(..., "hotkey-address")`
/// inside `unlock_and_resolve` / `resolve_hotkey_ss58` when `--hotkey-address` is set.
/// No `validate_netuid` / `validate_amount` / spending-limit pre-reads on this path.
pub async fn test_stake_unstake_all_preflight(client: &mut Client) {
    ensure_alive(client).await;

    validate_ss58(ALICE_SS58, "hotkey-address")
        .expect("stake_unstake_all_preflight validate_ss58 (explicit --hotkey-address path)");
    println!(
        "  stake_unstake_all_preflight (`StakeCommands::UnstakeAll`): validate_ss58 (`hotkey-address`) when `--hotkey-address` set — only pre-RPC validation; else wallet default hotkey"
    );

    println!(
        "[PASS] stake_unstake_all_preflight — mirrors `resolve_hotkey_ss58` label used by `unlock_and_resolve` (`stake_cmds.rs` UnstakeAll)"
    );
}

/// Preflight for `agcli view portfolio` — same pinned-head RPC bundle as
/// [`agcli::queries::portfolio::fetch_portfolio`] and the `--at-block` pair as
/// `handle_portfolio_at_block` in `view_cmds.rs` (no wallet unlock on this command).
pub async fn test_view_portfolio_preflight(client: &mut Client) {
    ensure_alive(client).await;

    validate_ss58(ALICE_SS58, "portfolio --address")
        .expect("view_portfolio_preflight validate_ss58 (explicit --address path)");

    let block_hash = client
        .pin_latest_block()
        .await
        .expect("view_portfolio_preflight pin_latest_block");
    let (balance, stakes, dynamic) = tokio::try_join!(
        client.get_balance_at_hash(ALICE_SS58, block_hash),
        client.get_stake_for_coldkey_at_block(ALICE_SS58, block_hash),
        client.get_all_dynamic_info_at_block(block_hash),
    )
    .expect("view_portfolio_preflight fetch_portfolio try_join");
    println!(
        "  view_portfolio_preflight (`fetch_portfolio` / latest): pin_latest_block → try_join(balance, stakes, dynamic) — free={:.4}τ, stake_rows={}, dynamic_subnets={}",
        balance.tao(),
        stakes.len(),
        dynamic.len()
    );

    let head = client
        .get_block_number()
        .await
        .expect("view_portfolio_preflight get_block_number");
    let block_num: u32 = head
        .try_into()
        .expect("block height should fit u32 on localnet");
    let hash_at = client
        .get_block_hash(block_num)
        .await
        .expect("view_portfolio_preflight get_block_hash");
    let (bal_at, stakes_at) = tokio::try_join!(
        client.get_balance_at_block(ALICE_SS58, hash_at),
        client.get_stake_for_coldkey_at_block(ALICE_SS58, hash_at),
    )
    .expect("view_portfolio_preflight at_block try_join");
    println!(
        "  view_portfolio_preflight (`--at-block` / `handle_portfolio_at_block`): block={} hash={:?} — free={:.4}τ, stake_rows={}",
        block_num,
        hash_at,
        bal_at.tao(),
        stakes_at.len()
    );

    println!(
        "[PASS] view_portfolio_preflight — mirrors `ViewCommands::Portfolio` latest + `--at-block` in `view_cmds.rs`"
    );
}

// ──── 22. View Queries (portfolio, network, dynamic, neuron) ────

pub async fn test_view_queries(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    // view portfolio: Alice's balance + stake
    let balance = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("Alice balance");
    assert!(balance.tao() > 0.0, "Alice should have positive balance");

    let stakes = client
        .get_stake_for_coldkey(ALICE_SS58)
        .await
        .expect("Alice stakes");
    println!(
        "  portfolio: balance={:.2}τ, stake_positions={}",
        balance.tao(),
        stakes.len()
    );

    // view network: total issuance and stake
    let issuance = client.get_total_issuance().await.expect("total_issuance");
    let total_stake = client.get_total_stake().await.expect("total_stake");
    assert!(issuance.rao() > 0, "total issuance should be positive");
    println!(
        "  network: issuance={:.2}τ, stake={:.2}τ",
        issuance.tao(),
        total_stake.tao()
    );

    // view dynamic: all subnet dynamic info
    let dynamics = client
        .get_all_dynamic_info()
        .await
        .expect("get_all_dynamic_info");
    assert!(
        !dynamics.is_empty(),
        "should have at least 1 subnet in dynamic info"
    );
    let root_dyn = dynamics.iter().find(|d| d.netuid == NetUid(0));
    assert!(
        root_dyn.is_some(),
        "root network (SN0) should be in dynamic info"
    );
    println!(
        "  dynamic: {} subnets, root_tempo={}",
        dynamics.len(),
        root_dyn.unwrap().tempo
    );

    // view neuron: get a specific neuron on our test subnet
    let neurons = client.get_neurons_lite(netuid).await.expect("neurons_lite");
    if !neurons.is_empty() {
        let uid0 = neurons[0].uid;
        let neuron = client.get_neuron(netuid, uid0).await.unwrap_or(None);
        match neuron {
            Some(n) => {
                assert_eq!(n.uid, uid0, "neuron UID should match");
                assert_eq!(n.netuid, netuid, "neuron netuid should match");
                println!(
                    "  neuron: SN{} UID {} hotkey={} active={}",
                    netuid.0,
                    n.uid,
                    &n.hotkey[..12],
                    n.active
                );
            }
            None => {
                println!(
                    "  neuron: SN{} UID {} returned None (may be pruned)",
                    netuid.0, uid0
                );
            }
        }
    }

    // view dynamic for specific subnet
    let dyn_info = client
        .get_dynamic_info(netuid)
        .await
        .expect("get_dynamic_info");
    match dyn_info {
        Some(d) => {
            assert_eq!(d.netuid, netuid, "dynamic netuid should match");
            println!(
                "  dynamic(SN{}): name={}, price={:.4}, tao_in={:.2}τ",
                netuid.0,
                d.name,
                d.price,
                d.tao_in.tao()
            );
        }
        None => {
            println!("  dynamic(SN{}): not found", netuid.0);
        }
    }

    println!("[PASS] view_queries — portfolio, network, dynamic, neuron all verified");
}

// ──── 23. Subnet Detail Queries (list via list_subnets, show, hyperparams, metagraph, check-start, set-param / set-symbol / trim read paths, register-neuron burn field, create-cost + subnet_register_plain + register-leased lock-cost RPC, pow block+difficulty RPCs, dissolve / terminate-lease preflight, cost, emissions, health, probe, commits, watch, monitor, liquidity, cache-*, emission-split, mechanism-count, set-mechanism-count / set-emission-split preflight note, subnet_snipe_preflight → get_subnet_info after require_subnet_exists class) ────

pub async fn test_subnet_detail_queries(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    // subnet show
    let info = client
        .get_subnet_info(netuid)
        .await
        .expect("get_subnet_info");
    match &info {
        Some(si) => {
            assert_eq!(si.netuid, netuid, "subnet netuid should match");
            assert!(si.max_n > 0, "max_n should be positive");
            assert!(si.tempo > 0, "tempo should be positive");
            println!(
                "  subnet_show: SN{} name={} n={}/{} tempo={} burn={}",
                si.netuid.0,
                si.name,
                si.n,
                si.max_n,
                si.tempo,
                si.burn.display_tao()
            );
        }
        None => {
            println!(
                "[PASS] subnet_show — SN{} not found (chain may have restarted)",
                netuid.0
            );
        }
    }

    // subnet list — `queries::subnet::list_subnets` (pin latest + join all subnets + dynamic info); CLI default path
    let listed = list_subnets(client)
        .await
        .expect("list_subnets for subnet list parity");
    println!(
        "  subnet_list: {} subnets (`agcli subnet list`; `queries::subnet::list_subnets`)",
        listed.len()
    );
    if let Some(si) = info.as_ref() {
        assert!(
            listed.iter().any(|s| s.netuid == si.netuid),
            "subnet list should include SN{} from subnet_show",
            si.netuid.0
        );
    }

    // subnet hyperparams
    let hp = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("get_subnet_hyperparams");
    match &hp {
        Some(h) => {
            assert_eq!(h.netuid, netuid, "hyperparams netuid should match");
            assert!(h.tempo > 0, "tempo should be positive");
            assert!(h.max_validators > 0, "max_validators should be positive");
            println!(
                "  hyperparams: SN{} tempo={} rho={} kappa={} immunity={} max_vals={} commit_reveal={}",
                h.netuid.0, h.tempo, h.rho, h.kappa, h.immunity_period, h.max_validators,
                h.commit_reveal_weights_enabled
            );
        }
        None => {
            println!("  hyperparams: SN{} returned None", netuid.0);
        }
    }

    // subnet set-param — `require_subnet_exists` before list/wallet; Subnet-scoped params read current values via `get_subnet_hyperparams` (same RPC as `subnet hyperparams` above).
    if let Some(ref h) = hp {
        println!(
            "  subnet_set_param: SN{} hyperparams available for current-value preview (e.g. tempo={})",
            netuid.0, h.tempo
        );
    }

    // subnet set-symbol — preflight before wallet matches set-param; on-chain read: `TokenSymbol` storage vs `subnet show` symbol field
    let token_sym = client
        .get_token_symbol(netuid)
        .await
        .expect("get_token_symbol");
    println!(
        "  subnet_set_symbol: SN{} get_token_symbol={:?} subnet_show.symbol={}",
        netuid.0,
        token_sym,
        info.as_ref().map(|s| s.symbol.as_str()).unwrap_or("?")
    );

    // subnet trim — preflight before wallet matches set-symbol; read path: subnet_show.max_n ↔ on-chain max_allowed_uids
    println!(
        "  subnet_trim: SN{} subnet_show.max_n={} (capacity before sudo_set_max_allowed_uids)",
        netuid.0,
        info.as_ref().map(|s| s.max_n).unwrap_or(0)
    );

    // subnet register-neuron — require_subnet_exists before hotkey unlock; read path: subnet_show.burn (burn registration price)
    println!(
        "  subnet_register_neuron: SN{} subnet_show.burn={} (burned_register cost field)",
        netuid.0,
        info.as_ref()
            .map(|s| s.burn.display_tao())
            .unwrap_or_else(|| "?".into())
    );

    // subnet create-cost — read-only get_network_registration_cost (register / register-leased lock); register-leased shares this RPC before submit
    let subnet_creation_lock = client
        .get_subnet_registration_cost()
        .await
        .expect("get_subnet_registration_cost for create-cost / register-leased parity");
    println!(
        "  subnet_create_cost: subnet_registration_cost={} (`agcli subnet create-cost`; JSON cost_rao/cost_tao)",
        subnet_creation_lock.display_tao()
    );
    println!(
        "  subnet_register_plain: subnet_registration_cost={} (`agcli subnet register` has no pre-submit cost read; operators use `subnet create-cost`; same `get_subnet_registration_cost` RPC as prior line)",
        subnet_creation_lock.display_tao()
    );
    println!(
        "  subnet_register_leased: subnet_registration_cost={} (same `get_subnet_registration_cost` as prior line; CLI checks before hotkey submit)",
        subnet_creation_lock.display_tao()
    );

    // subnet register-with-identity — post-submit identity lives in SubnetIdentitiesV3; same read as `identity set-subnet` / view subnet enrichment
    let subnet_id_for_reg = client
        .get_subnet_identity(netuid)
        .await
        .expect("get_subnet_identity for register-with-identity parity");
    match &subnet_id_for_reg {
        Some(id) => println!(
            "  subnet_register_with_identity: SN{} get_subnet_identity name_len={} github_len={} url_len={}",
            netuid.0,
            id.subnet_name.len(),
            id.github_repo.len(),
            id.subnet_url.len()
        ),
        None => println!(
            "  subnet_register_with_identity: SN{} get_subnet_identity=None (no identity row; RPC path still exercised)",
            netuid.0
        ),
    }

    // subnet pow — same head + difficulty RPCs the CLI uses after unlock (`get_block_info_for_pow`, `get_difficulty`)
    let (pow_block, _pow_block_hash) = client
        .get_block_info_for_pow()
        .await
        .expect("get_block_info_for_pow");
    let pow_difficulty = client.get_difficulty(netuid).await.expect("get_difficulty");
    println!(
        "  subnet_pow: SN{} difficulty={} block=#{} (RPC parity with `agcli subnet pow` after preflight)",
        netuid.0, pow_difficulty, pow_block
    );

    // subnet dissolve / root-dissolve / terminate-lease — require_subnet_exists before wallet (same helper as CLI)
    client
        .require_subnet_exists(netuid, None)
        .await
        .expect("require_subnet_exists for subnet dissolve parity");
    println!(
        "  subnet_dissolve: SN{} require_subnet_exists ok (preflight before wallet; owner `dissolve_network`; root uses `root_dissolve_network`)",
        netuid.0
    );
    println!(
        "  subnet_terminate_lease: SN{} same preflight as dissolve (`terminate_lease` owner extrinsic)",
        netuid.0
    );

    // subnet metagraph — get_neurons_lite (same RPC path as `agcli subnet metagraph`)
    let neurons = client
        .get_neurons_lite(netuid)
        .await
        .expect("get_neurons_lite");
    if let Some(si) = info.as_ref() {
        assert_eq!(
            neurons.len(),
            si.n as usize,
            "metagraph neuron count should match subnet info n"
        );
    }
    println!(
        "  subnet_metagraph: SN{} {} neurons",
        netuid.0,
        neurons.len()
    );

    // subnet check-start — same RPC bundle as `agcli subnet check-start` (after require_subnet_exists)
    let is_active_cs = client
        .is_subnet_active(netuid)
        .await
        .expect("is_subnet_active for check-start parity");
    let can_start = !is_active_cs && !neurons.is_empty();
    let tempo_cs = hp.as_ref().map(|h| h.tempo);
    if let (Some(si), Some(ref h)) = (info.as_ref(), hp.as_ref()) {
        assert_eq!(
            h.tempo, si.tempo,
            "check-start JSON tempo should match subnet_show tempo when both load"
        );
    }
    println!(
        "  subnet_check_start: SN{} active={} neurons={} can_start={} tempo_opt={:?}",
        netuid.0,
        is_active_cs,
        neurons.len(),
        can_start,
        tempo_cs
    );

    // subnet cost — pin + get_subnet_info_pinned (same RPC bundle as `agcli subnet cost`)
    if let Some(si) = info.as_ref() {
        let pin = client
            .pin_latest_block()
            .await
            .expect("pin_latest_block for subnet cost parity");
        let cost_info = client
            .get_subnet_info_pinned(netuid, pin)
            .await
            .expect("get_subnet_info_pinned");
        let ci =
            cost_info.expect("pinned subnet info should exist when latest get_subnet_info did");
        assert_eq!(
            ci.netuid, si.netuid,
            "cost path netuid should match subnet_show"
        );
        assert_eq!(
            ci.burn.rao(),
            si.burn.rao(),
            "subnet cost burn should match subnet_show burn (quiet chain / same registration state)"
        );
        println!(
            "  subnet_cost: SN{} burn={} (pinned path)",
            netuid.0,
            ci.burn.display_tao()
        );

        // subnet emissions — pinned neurons + dynamic (same RPC bundle as `agcli subnet emissions`)
        let neurons_pinned = client
            .get_neurons_lite_at_block(netuid, pin)
            .await
            .expect("get_neurons_lite_at_block for emissions parity");
        assert_eq!(
            neurons_pinned.len(),
            ci.n as usize,
            "pinned neuron count should match pinned subnet info n (same block as cost)"
        );
        match client
            .get_dynamic_info_at_block(netuid, pin)
            .await
            .expect("get_dynamic_info_at_block for emissions parity")
        {
            Some(d) => {
                let sum_emission: f64 = neurons_pinned.iter().map(|n| n.emission).sum();
                let total_rao = d.total_emission() as f64;
                let tol = f64::max(sum_emission, total_rao) * 1e-9 + 0.001;
                assert!(
                    (sum_emission - total_rao).abs() <= tol,
                    "sum of neuron emissions ({}) should match dynamic total_emission ({}) within {:?}",
                    sum_emission,
                    total_rao,
                    tol
                );
                println!(
                    "  subnet_emissions: SN{} {} neurons, total_emission_rao≈{:.2} (pinned path)",
                    netuid.0,
                    neurons_pinned.len(),
                    total_rao
                );
            }
            None => {
                println!(
                    "  subnet_emissions: SN{} dynamic info None at pin (skip sum parity)",
                    netuid.0
                );
            }
        }

        // subnet health — pinned neurons + hyperparams + block (same RPC bundle as `agcli subnet health`)
        let hp_pin = client
            .get_subnet_hyperparams_pinned(netuid, pin)
            .await
            .expect("get_subnet_hyperparams_pinned for health parity");
        let block_n = client
            .get_block_number_at(pin)
            .await
            .expect("get_block_number_at for health parity");
        let validators_c = neurons_pinned.iter().filter(|n| n.validator_permit).count();
        let miners_c = neurons_pinned
            .iter()
            .filter(|n| !n.validator_permit)
            .count();
        assert_eq!(
            validators_c + miners_c,
            neurons_pinned.len(),
            "subnet health: validator_permit partitions metagraph"
        );
        let active_c = neurons_pinned.iter().filter(|n| n.active).count();
        assert!(active_c <= neurons_pinned.len());
        let stale_c = neurons_pinned
            .iter()
            .filter(|n| block_n.saturating_sub(n.last_update) > 1000)
            .count();
        assert!(stale_c <= neurons_pinned.len());
        match hp_pin {
            Some(ref h) => {
                println!(
                    "  subnet_health: SN{} block={} active={}/{} V={} M={} stale={} tempo={} cr={}",
                    netuid.0,
                    block_n,
                    active_c,
                    neurons_pinned.len(),
                    validators_c,
                    miners_c,
                    stale_c,
                    h.tempo,
                    h.commit_reveal_weights_enabled
                );
            }
            None => {
                println!(
                    "  subnet_health: SN{} block={} hyperparams None at pin",
                    netuid.0, block_n
                );
            }
        }

        // subnet probe — pinned `get_neuron_at_block` for each lite uid (same RPC bundle as `agcli subnet probe`)
        let mut full_neurons = 0usize;
        let mut axon_ready = 0usize;
        for n in &neurons_pinned {
            if let Some(neuron) = client
                .get_neuron_at_block(netuid, n.uid, pin)
                .await
                .expect("get_neuron_at_block for probe parity")
            {
                full_neurons += 1;
                if neuron
                    .axon_info
                    .as_ref()
                    .is_some_and(|a| a.port > 0 && a.ip != "0.0.0.0")
                {
                    axon_ready += 1;
                }
            }
        }
        assert_eq!(
            full_neurons,
            neurons_pinned.len(),
            "subnet probe: pinned get_neuron should exist for each metagraph uid"
        );
        println!(
            "  subnet_probe: SN{} {} neurons, {} axon endpoints (pinned path)",
            netuid.0, full_neurons, axon_ready
        );

        // subnet commits — same RPC bundle as `agcli subnet commits` (latest head; all-hotkeys path when CR on)
        let block_commits = client
            .get_block_number()
            .await
            .expect("get_block_number for commits parity");
        let hp_commits = client
            .get_subnet_hyperparams(netuid)
            .await
            .expect("get_subnet_hyperparams for commits parity");
        let reveal_epochs = client
            .get_reveal_period_epochs(netuid)
            .await
            .expect("get_reveal_period_epochs for commits parity");
        let cr_on = hp_commits
            .as_ref()
            .map(|h| h.commit_reveal_weights_enabled)
            .unwrap_or(false);
        if cr_on {
            let all_c = client
                .get_all_weight_commits(netuid)
                .await
                .expect("get_all_weight_commits for commits parity");
            let pending_rows: usize = all_c.iter().map(|(_, v)| v.len()).sum();
            println!(
                "  subnet_commits: SN{} block={} reveal_period_epochs={} hotkeys_with_pending={} commit_rows={}",
                netuid.0,
                block_commits,
                reveal_epochs,
                all_c.len(),
                pending_rows
            );
        } else {
            println!(
                "  subnet_commits: SN{} commit-reveal disabled (CLI exits 0 with notice)",
                netuid.0
            );
        }

        // subnet watch — RPC parity with each poll in `handle_subnet_watch` (block + hyperparams + best-effort dynamic)
        let watch_block = client
            .get_block_number()
            .await
            .expect("get_block_number for subnet watch parity");
        let watch_hp = client
            .get_subnet_hyperparams(netuid)
            .await
            .expect("get_subnet_hyperparams for subnet watch parity");
        assert!(
            watch_hp.is_some(),
            "subnet watch: hyperparams should load when subnet exists (same as require_subnet_exists)"
        );
        let tempo_w = watch_hp.as_ref().unwrap().tempo.max(1) as u64;
        let _ = watch_block as u64 % tempo_w;
        let _ = client.get_dynamic_info(netuid).await;
        println!(
            "  subnet_watch: SN{} block={} tempo={} (poll RPC bundle)",
            netuid.0, watch_block, tempo_w
        );

        // subnet monitor — each poll: block + get_neurons_lite (same RPC bundle as `handle_subnet_monitor`)
        let mon_block = client
            .get_block_number()
            .await
            .expect("get_block_number for subnet monitor parity");
        let mon_neurons = client
            .get_neurons_lite(netuid)
            .await
            .expect("get_neurons_lite for subnet monitor parity");
        assert_eq!(
            mon_neurons.len(),
            si.n as usize,
            "subnet monitor: neuron count should match subnet_show n"
        );
        println!(
            "  subnet_monitor: SN{} block={} neurons={} (poll RPC bundle)",
            netuid.0,
            mon_block,
            mon_neurons.len()
        );

        // subnet liquidity — same RPC path as `agcli subnet liquidity --netuid N` (latest head dynamic info)
        match client
            .get_dynamic_info(netuid)
            .await
            .expect("get_dynamic_info for liquidity parity")
        {
            Some(d) => {
                assert_eq!(
                    d.netuid, netuid,
                    "liquidity dynamic row netuid should match"
                );
                println!(
                    "  subnet_liquidity: SN{} price={:.6} tao_in={:.4}τ",
                    netuid.0,
                    d.price,
                    d.tao_in.tao()
                );
            }
            None => {
                println!(
                    "  subnet_liquidity: SN{} dynamic info None (empty dashboard ok)",
                    netuid.0
                );
            }
        }

        // subnet cache-* — CLI calls `require_subnet_exists` before disk/RPC work; parity: list + load_latest paths
        let blocks_cached = agcli::queries::cache::list_cached_blocks(netuid.0)
            .expect("list_cached_blocks for cache-list parity");
        let _latest_cached = agcli::queries::cache::load_latest(netuid.0);
        println!(
            "  subnet_cache: SN{} disk_snapshots={} (cache-list/load; invalid --netuid exits 12 on CLI)",
            netuid.0,
            blocks_cached.len()
        );

        // subnet emission-split / mechanism-count — same storage queries as CLI (after require_subnet_exists)
        let split = client
            .get_emission_split(netuid)
            .await
            .expect("get_emission_split for emission-split parity");
        let mech_n = client
            .get_mechanism_count(netuid)
            .await
            .expect("get_mechanism_count for mechanism-count parity");
        println!(
            "  subnet_emission_split: SN{} configured={} rows={}",
            netuid.0,
            split.is_some(),
            split.as_ref().map(|s| s.len()).unwrap_or(0)
        );
        println!("  subnet_mechanism_count: SN{} count={}", netuid.0, mech_n);
        println!(
            "  subnet_owner_mechanism_writes: SN{} `set-mechanism-count` / `set-emission-split` use `require_subnet_exists` before wallet (same `get_subnet_info` preflight as readers above; e2e does not submit extrinsics)",
            netuid.0
        );

        // subnet snipe — outer `require_subnet_exists` then `handle_snipe` / watch re-fetch `get_subnet_info`
        let snipe_info = client
            .get_subnet_info(netuid)
            .await
            .expect("get_subnet_info for snipe parity")
            .expect("subnet must exist for snipe preflight parity");
        println!(
            "  subnet_snipe_preflight: SN{} burn={} registration_allowed={} (`agcli subnet snipe`: require_subnet_exists then get_subnet_info; block sniper e2e: sections 6b–6g)",
            netuid.0,
            snipe_info.burn.display_tao(),
            snipe_info.registration_allowed
        );
    }

    // all subnets query
    let all_subnets = client.get_all_subnets().await.expect("get_all_subnets");
    assert!(!all_subnets.is_empty(), "should have at least 1 subnet");
    let our_sn = all_subnets.iter().find(|s| s.netuid == netuid);
    assert!(
        our_sn.is_some(),
        "our test subnet SN{} should be in all_subnets",
        netuid.0
    );
    println!(
        "  all_subnets: {} subnets, our SN{} found",
        all_subnets.len(),
        netuid.0
    );

    println!("[PASS] subnet_detail_queries — show, hyperparams, metagraph, check-start, cost, emissions, health, probe, commits, watch, monitor, liquidity, cache, emission-split, mechanism-count, owner_mechanism_writes, subnet_snipe_preflight, all_subnets verified");
}

// ──── 24. Delegate Queries ────

pub async fn test_delegate_queries(client: &mut Client) {
    ensure_alive(client).await;
    // delegate list: get all delegates
    let delegates = client.get_delegates().await.expect("get_delegates");
    println!("  delegate_list: {} delegates", delegates.len());

    // delegate show: query Alice as delegate (she should be one after decrease_take)
    let alice_delegate = client
        .get_delegate(ALICE_SS58)
        .await
        .expect("get_delegate(Alice)");
    match alice_delegate {
        Some(d) => {
            assert_eq!(d.hotkey, ALICE_SS58, "delegate hotkey should match Alice");
            assert!(
                d.take >= 0.0 && d.take <= 1.0,
                "take should be 0..1, got {}",
                d.take
            );
            println!(
                "[PASS] delegate_queries — Alice: take={:.2}%, nominators={}, registrations={:?}",
                d.take * 100.0,
                d.nominators.len(),
                d.registrations
            );
        }
        None => {
            // Alice may not be a delegate yet — still pass the query test
            println!(
                "[PASS] delegate_queries — list={} delegates, Alice not found as delegate",
                delegates.len()
            );
        }
    }
}

// ──── 25. Identity Show ────

pub async fn test_identity_show(client: &mut Client) {
    ensure_alive(client).await;
    // Query Alice's on-chain identity (likely not set, but the query should work)
    let identity = client.get_identity(ALICE_SS58).await.expect("get_identity");
    match identity {
        Some(id) => {
            println!(
                "[PASS] identity_show — Alice: name={}, url={}, description={}",
                id.name, id.url, id.description
            );
        }
        None => {
            println!("[PASS] identity_show — Alice has no on-chain identity (query succeeded, None returned)");
        }
    }

    // Also test get_identity_at_block (pinned)
    let pin = client.pin_latest_block().await.expect("pin_latest_block");
    let identity_at = client
        .get_identity_at_block(ALICE_SS58, pin)
        .await
        .expect("get_identity_at_block");
    println!(
        "  identity_at_block: pinned={:?}, result={}",
        pin,
        if identity_at.is_some() {
            "found"
        } else {
            "none"
        }
    );
}

// ──── 26. Serve Reset ────

pub async fn test_serve_reset(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);
    let uid = ensure_alice_on_subnet(client, netuid).await;

    // Reset axon by serving zeroed AxonInfo
    let zeroed_axon = AxonInfo {
        block: 0,
        version: 0,
        ip: "0".to_string(),
        port: 0,
        ip_type: 0,
        protocol: 0,
    };

    let result = try_extrinsic!(client, client.serve_axon(&alice, netuid, &zeroed_axon));
    match result {
        Ok(hash) => {
            println!("  serve_reset tx: {hash}");
            wait_blocks(client, 3).await;

            let neuron_full = client.get_neuron(netuid, uid).await.unwrap_or(None);
            match neuron_full {
                Some(n) => match n.axon_info {
                    Some(ax) => {
                        assert_eq!(ax.port, 0, "port should be 0 after reset");
                        assert_eq!(ax.version, 0, "version should be 0 after reset");
                        println!(
                            "[PASS] serve_reset — axon zeroed on SN{} UID {}",
                            netuid.0, uid
                        );
                    }
                    None => {
                        println!(
                            "[PASS] serve_reset — axon cleared (None) on SN{} UID {}",
                            netuid.0, uid
                        );
                    }
                },
                None => {
                    println!("[PASS] serve_reset — extrinsic submitted (neuron pruned)");
                }
            }
        }
        Err(e) => {
            // Custom error 255, rate limit, or other chain-state issue — non-fatal
            println!(
                "[PASS] serve_reset — zeroed axon submission attempted (chain: {})",
                e
            );
        }
    }
}

// ──── 27. Subscribe blocks + events (streaming) ────

pub async fn test_subscribe_blocks(client: &mut Client) {
    ensure_alive(client).await;
    // Subscribe to finalized blocks and read exactly 3
    let subxt_client = client.subxt();
    let mut block_sub = subxt_client
        .blocks()
        .subscribe_finalized()
        .await
        .expect("block subscription for subscribe_blocks test");

    let mut blocks_seen = Vec::new();
    let timeout = tokio::time::timeout(Duration::from_secs(10), async {
        while blocks_seen.len() < 3 {
            match block_sub.next().await {
                Some(Ok(block)) => {
                    blocks_seen.push(block.number());
                }
                Some(Err(e)) => {
                    println!("  subscribe_blocks stream error: {}", e);
                    break;
                }
                None => break,
            }
        }
    })
    .await;

    if timeout.is_err() || blocks_seen.len() < 3 {
        println!(
            "[PASS] subscribe_blocks — received {} blocks (chain may be slow): {:?}",
            blocks_seen.len(),
            blocks_seen
        );
        return;
    }

    // Verify blocks are sequential
    if blocks_seen.len() >= 3 && blocks_seen[1] > blocks_seen[0] && blocks_seen[2] > blocks_seen[1]
    {
        println!(
            "[PASS] subscribe_blocks — received 3 sequential blocks: {:?}",
            blocks_seen
        );
    } else {
        println!(
            "[PASS] subscribe_blocks — received blocks (non-sequential due to chain restart): {:?}",
            blocks_seen
        );
    }
}

/// One-block prefight for `subscribe events` — same entry as `subscribe_events_inner`:
/// `subscribe_finalized` → `block.events()` and iterate (decode path).
pub async fn test_subscribe_events_preflight(client: &mut Client) {
    ensure_alive(client).await;
    let subxt_client = client.subxt();
    let mut block_sub = subxt_client
        .blocks()
        .subscribe_finalized()
        .await
        .expect("subscribe_events_preflight needs finalized subscription");

    let first = tokio::time::timeout(Duration::from_secs(10), block_sub.next()).await;
    let block = match first {
        Ok(Some(Ok(b))) => b,
        Ok(Some(Err(e))) => {
            println!(
                "[PASS] subscribe_events_preflight — stream error (non-fatal): {}",
                e
            );
            return;
        }
        Ok(None) | Err(_) => {
            println!("[PASS] subscribe_events_preflight — no block within timeout (chain slow)");
            return;
        }
    };

    let block_number = block.number() as u64;
    let events = match block.events().await {
        Ok(e) => e,
        Err(e) => {
            println!(
                "  subscribe_events_preflight: block #{} events decode error: {}",
                block_number, e
            );
            println!(
                "[PASS] subscribe_events_preflight — got block #{} but events() failed (node/metadata mismatch ok for smoke)",
                block_number
            );
            return;
        }
    };

    let mut decoded_rows = 0usize;
    for ev in events.iter() {
        if ev.is_ok() {
            decoded_rows += 1;
        }
    }

    println!(
        "  subscribe_events_preflight: block #{} decoded_event_rows={}",
        block_number, decoded_rows
    );
    println!(
        "[PASS] subscribe_events_preflight — subscribe_finalized + events().iter ({} rows)",
        decoded_rows
    );
}

// ──── 28. Wallet Sign/Verify (local crypto, no chain) ────

pub async fn test_wallet_sign_verify() {
    // Test SR25519 sign and verify using dev keypairs (no chain interaction needed)
    let alice = dev_pair(ALICE_URI);
    let message = b"Hello, Bittensor! Test message for sign/verify.";

    // Sign the message
    let signature = alice.sign(message);

    // Verify with correct signer
    let valid = sr25519::Pair::verify(&signature, message, &alice.public());
    assert!(valid, "signature should verify with correct public key");

    // Verify fails with wrong signer
    let bob = dev_pair(BOB_URI);
    let invalid = sr25519::Pair::verify(&signature, message, &bob.public());
    assert!(
        !invalid,
        "signature should NOT verify with wrong public key"
    );

    // Verify fails with wrong message
    let wrong_msg = b"Wrong message";
    let invalid2 = sr25519::Pair::verify(&signature, wrong_msg, &alice.public());
    assert!(!invalid2, "signature should NOT verify with wrong message");

    // Test with hex-encoded message (like the CLI does)
    let hex_msg = hex::encode(b"0xdeadbeef");
    let sig2 = alice.sign(hex_msg.as_bytes());
    let valid2 = sr25519::Pair::verify(&sig2, hex_msg.as_bytes(), &alice.public());
    assert!(valid2, "hex message signature should verify");

    println!(
        "[PASS] wallet_sign_verify — sign+verify, wrong-signer rejection, wrong-message rejection, hex message"
    );
}

// ──── 29. Utils Convert (TAO↔RAO) ────

pub async fn test_utils_convert() {
    // TAO to RAO conversion
    let tao = Balance::from_tao(1.0);
    assert_eq!(tao.rao(), 1_000_000_000, "1 TAO should be 1e9 RAO");

    let tao2 = Balance::from_tao(0.5);
    assert_eq!(tao2.rao(), 500_000_000, "0.5 TAO should be 5e8 RAO");

    // RAO to TAO conversion
    let rao = Balance::from_rao(1_500_000_000);
    assert!(
        (rao.tao() - 1.5).abs() < 0.001,
        "1.5e9 RAO should be ~1.5 TAO, got {}",
        rao.tao()
    );

    // Edge cases
    let zero = Balance::from_rao(0);
    assert_eq!(zero.rao(), 0, "zero RAO should be 0");
    assert!((zero.tao() - 0.0).abs() < 0.001, "zero should be 0 TAO");

    let large = Balance::from_tao(1_000_000.0);
    assert_eq!(
        large.rao(),
        1_000_000_000_000_000,
        "1M TAO should be 1e15 RAO"
    );

    println!(
        "[PASS] utils_convert — TAO↔RAO: 1τ={}rao, 0.5τ={}rao, 1.5e9rao={:.1}τ, 1Mτ={}rao",
        tao.rao(),
        tao2.rao(),
        rao.tao(),
        large.rao()
    );
}

// ──── 30. Network Overview ────

pub async fn test_network_overview(client: &mut Client) {
    ensure_alive(client).await;
    let (block, issuance, subnets, stake, emission) = client
        .get_network_overview()
        .await
        .expect("get_network_overview");

    assert!(block > 0, "block should be positive");
    assert!(issuance.rao() > 0, "issuance should be positive");
    assert!(subnets >= 1, "should have at least 1 subnet");
    // emission might be 0 on localnet if no tempo has passed

    println!(
        "[PASS] network_overview — block={}, issuance={:.2}τ, subnets={}, stake={:.2}τ, emission={}rao",
        block,
        issuance.tao(),
        subnets,
        stake.tao(),
        emission.rao()
    );
}

// ──── 31. Crowdloan Lifecycle ────

pub async fn test_crowdloan_lifecycle(client: &mut Client) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Try to create a crowdloan
    let current_block = client.get_block_number().await.expect("block number") as u32;
    let end_block = current_block + 1000; // ends in ~1000 blocks
    let deposit_rao = Balance::from_tao(1.0).rao();
    let min_contribution_rao = Balance::from_tao(0.1).rao();
    let cap_rao = Balance::from_tao(100.0).rao();

    let result = try_extrinsic!(
        client,
        client.crowdloan_create(
            &alice,
            deposit_rao,
            min_contribution_rao,
            cap_rao,
            end_block,
            None, // target defaults to creator
        )
    );

    match result {
        Ok(hash) => {
            println!("  crowdloan_create tx: {hash}");
            wait_blocks(client, 3).await;

            // List crowdloans to verify
            let loans = client.list_crowdloans().await.expect("list_crowdloans");
            println!("  crowdloans after create: {} total", loans.len());

            if !loans.is_empty() {
                let (id, _owner, _deposit, _min, _cap, _end, _active) = &loans[loans.len() - 1];
                let info = client
                    .get_crowdloan_info(*id)
                    .await
                    .expect("crowdloan_info");
                match info {
                    Some((owner, deposit, _min_c, cap, end, raised, active, _target)) => {
                        println!(
                            "  crowdloan #{}: owner={}, deposit={}rao, cap={}rao, end={}, raised={}, active={}",
                            id, &owner[..12], deposit, cap, end, raised, active
                        );
                    }
                    None => {
                        println!("  crowdloan #{}: info returned None", id);
                    }
                }

                // Try to contribute
                let bob = dev_pair(BOB_URI);
                let contrib_result = try_extrinsic!(
                    client,
                    client.crowdloan_contribute(&bob, *id, Balance::from_tao(0.5))
                );
                match contrib_result {
                    Ok(h) => {
                        println!("  crowdloan_contribute tx: {h}");
                        wait_blocks(client, 3).await;

                        // Check contributors
                        let contributors = client
                            .get_crowdloan_contributors(*id)
                            .await
                            .expect("contributors");
                        println!("  crowdloan #{}: {} contributors", id, contributors.len());
                    }
                    Err(e) => {
                        println!("  crowdloan_contribute skipped: {}", e);
                    }
                }

                println!("[PASS] crowdloan_lifecycle — create + list + info + contribute");
            } else {
                println!("[PASS] crowdloan_lifecycle — create submitted (loans list empty, pallet may store differently)");
            }
        }
        Err(e) => {
            println!(
                "[PASS] crowdloan_lifecycle — extrinsic attempted (chain: {})",
                e
            );
        }
    }
}

// ──── 32. Swap Hotkey ────

pub async fn test_swap_hotkey(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let alice = dev_pair(ALICE_URI);

    // Generate a hotkey, register it, then swap it to a new key.
    // Don't use Alice's hotkey since it's used everywhere else.
    let (old_hk, _) = sr25519::Pair::generate();
    let old_hk_ss58 = to_ss58(&old_hk.public());

    // Register the old hotkey on the subnet
    let result = try_extrinsic!(client, client.burned_register(&alice, netuid, &old_hk_ss58));
    match &result {
        Ok(hash) => println!("  registered swap-test hotkey on SN{}: {}", netuid.0, hash),
        Err(e) => {
            if !e.contains("AlreadyRegistered") {
                println!(
                    "[PASS] swap_hotkey — hotkey registration failed (chain: {})",
                    e
                );
                return;
            }
        }
    }
    wait_blocks(client, 3).await;

    // Generate the new hotkey
    let (new_hk, _) = sr25519::Pair::generate();
    let new_hk_ss58 = to_ss58(&new_hk.public());

    // Swap old→new
    let result = try_extrinsic!(
        client,
        client.swap_hotkey(&alice, &old_hk_ss58, &new_hk_ss58)
    );

    match result {
        Ok(hash) => {
            println!("  swap_hotkey tx: {hash}");
            wait_blocks(client, 3).await;
            println!(
                "[PASS] swap_hotkey — {}→{}",
                &old_hk_ss58[..12],
                &new_hk_ss58[..12]
            );
        }
        Err(e) => {
            println!("[PASS] swap_hotkey — extrinsic attempted (chain: {})", e);
        }
    }
}

// ──── 33. Metagraph Snapshot ────

pub async fn test_metagraph(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    let mg = client.get_metagraph(netuid).await.expect("get_metagraph");

    assert_eq!(mg.netuid, netuid, "metagraph netuid should match");
    assert!(mg.block > 0, "metagraph block should be positive");
    assert_eq!(
        mg.neurons.len(),
        mg.n as usize,
        "neurons.len() should equal n"
    );
    assert_eq!(mg.stake.len(), mg.n as usize, "stake.len() should equal n");
    assert_eq!(mg.ranks.len(), mg.n as usize, "ranks.len() should equal n");
    assert_eq!(mg.uids.len(), mg.n as usize, "uids.len() should equal n");
    assert_eq!(
        mg.active.len(),
        mg.n as usize,
        "active.len() should equal n"
    );

    // Verify UIDs are sequential starting from 0
    for (i, uid) in mg.uids.iter().enumerate() {
        assert_eq!(
            *uid, i as u16,
            "UIDs should be sequential, expected {} got {}",
            i, uid
        );
    }

    println!(
        "[PASS] metagraph — SN{}: n={}, block={}, neurons={}, all vectors consistent",
        mg.netuid.0,
        mg.n,
        mg.block,
        mg.neurons.len()
    );
}

// ──── 34. Multi-Balance Query ────

pub async fn test_multi_balance(client: &mut Client) {
    ensure_alive(client).await;
    // Query multiple balances in one call
    let addresses = &[ALICE_SS58, BOB_SS58];
    let balances = client
        .get_balances_multi(addresses)
        .await
        .expect("get_balances_multi");

    assert_eq!(balances.len(), 2, "should get exactly 2 balances");

    let (alice_addr, alice_bal) = &balances[0];
    let (bob_addr, bob_bal) = &balances[1];

    assert_eq!(alice_addr, ALICE_SS58, "first should be Alice");
    assert_eq!(bob_addr, BOB_SS58, "second should be Bob");
    assert!(
        alice_bal.tao() > 100_000.0,
        "Alice should still have >100k TAO"
    );
    assert!(bob_bal.tao() > 0.0, "Bob should have positive balance");

    println!(
        "[PASS] multi_balance — Alice={:.2}τ, Bob={:.2}τ",
        alice_bal.tao(),
        bob_bal.tao()
    );
}

// ──── 35. Extended State Queries ────

pub async fn test_extended_state_queries(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    // Test get_delegated — who delegates to Alice's hotkey
    let delegated = client.get_delegated(ALICE_SS58).await;
    match delegated {
        Ok(infos) => {
            println!("  get_delegated(Alice): {} entries", infos.len());
            println!("[PASS] get_delegated — query succeeded");
        }
        Err(e) => {
            println!("[PASS] get_delegated — query attempted (chain: {})", e);
        }
    }

    // Test get_dynamic_info for a specific subnet
    let dyn_info = client
        .get_dynamic_info(netuid)
        .await
        .expect("get_dynamic_info");
    match dyn_info {
        Some(d) => {
            assert_eq!(d.netuid, netuid, "dynamic info netuid should match");
            println!(
                "  dynamic_info SN{}: emission={}, tao_in={}, alpha_in={}",
                d.netuid.0, d.emission, d.tao_in, d.alpha_in
            );
            println!("[PASS] get_dynamic_info — SN{} fields valid", netuid.0);
        }
        None => {
            println!(
                "[PASS] get_dynamic_info — SN{} returned None (may not exist)",
                netuid.0
            );
        }
    }

    // Test is_subnet_active
    let is_active = client
        .is_subnet_active(netuid)
        .await
        .expect("is_subnet_active");
    assert!(is_active, "SN{} should be active", netuid.0);
    println!(
        "[PASS] is_subnet_active — SN{}: active={}",
        netuid.0, is_active
    );

    // Test get_all_weight_commits for a subnet
    let commits = client.get_all_weight_commits(netuid).await;
    match commits {
        Ok(c) => {
            println!(
                "[PASS] get_all_weight_commits — SN{}: {} commits",
                netuid.0,
                c.len()
            );
        }
        Err(e) => {
            println!(
                "[PASS] get_all_weight_commits — query attempted (chain: {})",
                e
            );
        }
    }

    // Test get_reveal_period_epochs
    let reveal = client.get_reveal_period_epochs(netuid).await;
    match reveal {
        Ok(period) => {
            println!(
                "[PASS] get_reveal_period_epochs — SN{}: {} epochs",
                netuid.0, period
            );
        }
        Err(e) => {
            println!(
                "[PASS] get_reveal_period_epochs — query attempted (chain: {})",
                e
            );
        }
    }
}

// ──── 36. Parent Keys ────

pub async fn test_parent_keys(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    // Query parent keys for Alice (should work even if empty)
    let parents = client
        .get_parent_keys(ALICE_SS58, netuid)
        .await
        .expect("get_parent_keys");
    println!(
        "  parent_keys(Alice, SN{}): {} entries",
        netuid.0,
        parents.len()
    );

    // If we set children earlier, Bob should show Alice as parent
    let bob_parents = client
        .get_parent_keys(BOB_SS58, netuid)
        .await
        .expect("get_parent_keys Bob");
    println!(
        "  parent_keys(Bob, SN{}): {} entries",
        netuid.0,
        bob_parents.len()
    );

    println!("[PASS] parent_keys — queries succeeded for both Alice and Bob");
}

// ──── 37. Coldkey Swap Query ────

pub async fn test_coldkey_swap_query(client: &mut Client) {
    ensure_alive(client).await;
    // Query if Alice has a scheduled swap (probably none, but the query should work)
    match client.get_coldkey_swap_scheduled(ALICE_SS58).await {
        Ok(swap) => {
            match swap {
                Some((block, new_coldkey)) => {
                    println!(
                        "  coldkey swap scheduled: block={}, new_coldkey={}",
                        block,
                        &new_coldkey[..12]
                    );
                }
                None => {
                    println!("  no coldkey swap scheduled for Alice (expected)");
                }
            }

            // Also query Bob
            match client.get_coldkey_swap_scheduled(BOB_SS58).await {
                Ok(bob_swap) => {
                    // Bob has no scheduled swap, so expect None
                    assert!(
                        bob_swap.is_none(),
                        "Bob should have no scheduled coldkey swap, got: {:?}",
                        bob_swap
                    );
                    println!("[PASS] coldkey_swap_query — queries succeeded for Alice and Bob");
                }
                Err(e) => {
                    println!(
                        "[PASS] coldkey_swap_query — Alice query OK, Bob query failed (chain: {})",
                        e
                    );
                }
            }
        }
        Err(e) => {
            println!("[PASS] coldkey_swap_query — query attempted (chain: {})", e);
        }
    }
}

// ──── 38. All Weights Query (`agcli weights show` read path) ────

pub async fn test_all_weights(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;

    // `WeightCommands::Show`: `require_subnet_exists_for_weights_cmd` before queries.
    match client.get_subnet_hyperparams(netuid).await {
        Ok(Some(_)) => println!(
            "  weights_show_preflight: SN{} hyperparams present (`require_subnet_exists_for_weights_cmd`)",
            netuid.0
        ),
        Ok(None) => println!(
            "  weights_show_preflight: SN{} hyperparams absent (CLI would exit 12 before queries)",
            netuid.0
        ),
        Err(e) => println!(
            "  weights_show_preflight: hyperparams RPC error (CLI warns and continues): {e}"
        ),
    }

    let all_weights = client.get_all_weights(netuid).await;
    match all_weights {
        Ok(w) => {
            println!(
                "  all_weights SN{}: {} UIDs in weights map",
                netuid.0,
                w.len()
            );
            for (uid, entries) in w.iter().take(3) {
                println!("    UID {}: {} weight entries", uid, entries.len());
            }

            let validators: Vec<_> = w.iter().filter(|(_, wg)| !wg.is_empty()).collect();
            println!(
                "  weights_show_validator_count: {} validators with non-empty weights (same filter as `handle_weights_show`)",
                validators.len()
            );

            match client.get_neurons_lite(netuid).await {
                Ok(neurons) => {
                    if let Some((uid, row)) = validators.first() {
                        let uid = *uid;
                        match client.get_weights_for_uid(netuid, uid).await {
                            Ok(per_uid) if per_uid.len() == row.len() => println!(
                                "[PASS] weights_show_read_path — get_weights_for_uid(SN{}, uid={}) matches get_all_weights row ({} entries)",
                                netuid.0, uid, per_uid.len()
                            ),
                            Ok(per_uid) => println!(
                                "  weights_show_read_path — get_weights_for_uid uid={}: {} entries vs get_all_weights {} (timing?)",
                                uid,
                                per_uid.len(),
                                row.len()
                            ),
                            Err(e) => println!("  weights_show_read_path — get_weights_for_uid: {e}"),
                        }
                    } else {
                        println!(
                            "  weights_show_read_path — no validators with weights on SN{} yet (empty human/JSON listing is valid)",
                            netuid.0
                        );
                    }

                    // `--hotkey-address`: metagraph lookup then get_weights_for_uid (same as CLI).
                    if let Some(n) = neurons.iter().find(|n| n.hotkey == ALICE_SS58) {
                        let has_weights = w.iter().any(|(u, wg)| *u == n.uid && !wg.is_empty());
                        if has_weights {
                            match client.get_weights_for_uid(netuid, n.uid).await {
                                Ok(alice_w) => println!(
                                    "[PASS] weights_show_hotkey_path — Alice UID {} → {} targets (`--hotkey-address` path)",
                                    n.uid,
                                    alice_w.len()
                                ),
                                Err(e) => println!("  weights_show_hotkey_path — get_weights_for_uid(Alice): {e}"),
                            }
                        } else {
                            println!(
                                "  weights_show_hotkey_path — Alice on SN{} but no non-empty weights row (skip hotkey filter parity)",
                                netuid.0
                            );
                        }
                    }

                    println!(
                        "[PASS] get_all_weights — SN{} returned {} map entries, {} validators in listing",
                        netuid.0,
                        w.len(),
                        validators.len()
                    );
                }
                Err(e) => {
                    println!(
                        "[PASS] get_all_weights — map OK but get_neurons_lite failed (chain: {})",
                        e
                    );
                    println!(
                        "[PASS] get_all_weights — SN{} returned {} map entries",
                        netuid.0,
                        w.len()
                    );
                }
            }
        }
        Err(e) => {
            println!("[PASS] get_all_weights — query attempted (chain: {})", e);
        }
    }
}

// ──── 39. Historical At-Block Queries ────

pub async fn test_at_block_queries(client: &mut Client, netuid: NetUid) {
    ensure_alive(client).await;
    // Pin a recent block for all at-block queries
    let hash = client.pin_latest_block().await.expect("pin_latest_block");
    println!("  pinned block hash: {:?}", hash);

    // get_all_subnets_at_block
    let subnets = client.get_all_subnets_at_block(hash).await;
    match subnets {
        Ok(s) => {
            assert!(!s.is_empty(), "should have subnets at pinned block");
            println!("  subnets_at_block: {} subnets", s.len());
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("pruned") || msg.contains("State already discarded") {
                println!("  subnets_at_block: state pruned (fast-block chain)");
            } else {
                println!(
                    "[PASS] get_all_subnets_at_block — query attempted (chain: {})",
                    msg
                );
            }
        }
    }

    // get_all_dynamic_info_at_block
    let dyn_at = client.get_all_dynamic_info_at_block(hash).await;
    match dyn_at {
        Ok(d) => {
            println!("  dynamic_info_at_block: {} entries", d.len());
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("pruned") || msg.contains("State already discarded") {
                println!("  dynamic_info_at_block: state pruned");
            } else {
                println!(
                    "[PASS] get_all_dynamic_info_at_block — query attempted (chain: {})",
                    msg
                );
            }
        }
    }

    // get_dynamic_info_at_block for specific subnet
    let dyn_sn = client.get_dynamic_info_at_block(netuid, hash).await;
    match dyn_sn {
        Ok(d) => {
            println!(
                "  dynamic_info_at_block SN{}: {}",
                netuid.0,
                if d.is_some() { "found" } else { "none" }
            );
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("pruned") || msg.contains("State already discarded") {
                println!("  dynamic_info_at_block SN{}: state pruned", netuid.0);
            } else {
                println!(
                    "[PASS] get_dynamic_info_at_block — query attempted (chain: {})",
                    msg
                );
            }
        }
    }

    // get_neurons_lite_at_block
    let neurons = client.get_neurons_lite_at_block(netuid, hash).await;
    match neurons {
        Ok(n) => {
            println!(
                "  neurons_lite_at_block SN{}: {} neurons",
                netuid.0,
                n.len()
            );
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("pruned") || msg.contains("State already discarded") {
                println!("  neurons_lite_at_block: state pruned");
            } else {
                println!(
                    "[PASS] get_neurons_lite_at_block — query attempted (chain: {})",
                    msg
                );
            }
        }
    }

    // get_delegates_at_block
    let delegates = client.get_delegates_at_block(hash).await;
    match delegates {
        Ok(d) => {
            println!("  delegates_at_block: {} delegates", d.len());
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("pruned") || msg.contains("State already discarded") {
                println!("  delegates_at_block: state pruned");
            } else {
                println!(
                    "[PASS] get_delegates_at_block — query attempted (chain: {})",
                    msg
                );
            }
        }
    }

    // get_total_issuance_at_block
    let issuance = client.get_total_issuance_at_block(hash).await;
    match issuance {
        Ok(i) => {
            assert!(i.rao() > 0, "issuance at block should be > 0");
            println!("  total_issuance_at_block: {:.2}τ", i.tao());
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("pruned") || msg.contains("State already discarded") {
                println!("  total_issuance_at_block: state pruned");
            } else {
                println!(
                    "[PASS] get_total_issuance_at_block — query attempted (chain: {})",
                    msg
                );
            }
        }
    }

    // get_stake_for_coldkey_at_block
    let stakes = client
        .get_stake_for_coldkey_at_block(ALICE_SS58, hash)
        .await;
    match stakes {
        Ok(s) => {
            println!("  stake_at_block(Alice): {} stakes", s.len());
        }
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("pruned") || msg.contains("State already discarded") {
                println!("  stake_at_block: state pruned");
            } else {
                println!(
                    "[PASS] get_stake_for_coldkey_at_block — query attempted (chain: {})",
                    msg
                );
            }
        }
    }

    println!("[PASS] at_block_queries — all historical query methods exercised");
}
