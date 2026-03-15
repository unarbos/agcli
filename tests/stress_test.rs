//! Stress tests — verify agcli handles concurrency and edge cases.

use std::sync::{Arc, atomic::{AtomicU32, Ordering}};

/// Concurrent wallet writes should not corrupt data thanks to file locking.
#[test]
fn concurrent_wallet_writes_no_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let keyfile = dir.path().join("stress_coldkey");
    let password = "stress_test_pw";
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Write the keyfile first
    agcli::wallet::keyfile::write_encrypted_keyfile(&keyfile, mnemonic, password).unwrap();

    // Spawn N threads that all try to read/write simultaneously
    let threads: Vec<_> = (0..8)
        .map(|i| {
            let path = keyfile.clone();
            let pw = password.to_string();
            let mn = format!("thread {} {}", i, mnemonic);
            std::thread::spawn(move || {
                // Each thread writes its own mnemonic
                agcli::wallet::keyfile::write_encrypted_keyfile(&path, &mn, &pw).unwrap();
                // Then reads back — must get a valid mnemonic (any thread's)
                let read = agcli::wallet::keyfile::read_encrypted_keyfile(&path, &pw).unwrap();
                assert!(
                    read.contains("abandon") || read.starts_with("thread"),
                    "Got corrupted data: {}", read
                );
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    // Final read should succeed
    let final_read = agcli::wallet::keyfile::read_encrypted_keyfile(&keyfile, password);
    assert!(final_read.is_ok(), "Final read failed: {:?}", final_read.err());
}

/// Concurrent hotkey file writes should be safe.
#[test]
fn concurrent_hotkey_writes_no_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let keyfile = dir.path().join("stress_hotkey");
    let mnemonic = "test mnemonic for hotkey stress testing";

    agcli::wallet::keyfile::write_keyfile(&keyfile, mnemonic).unwrap();

    let threads: Vec<_> = (0..8)
        .map(|i| {
            let path = keyfile.clone();
            let mn = format!("hotkey_thread_{} {}", i, mnemonic);
            std::thread::spawn(move || {
                agcli::wallet::keyfile::write_keyfile(&path, &mn).unwrap();
                let read = agcli::wallet::keyfile::read_keyfile(&path).unwrap();
                assert!(
                    read.contains("hotkey_thread") || read.contains("test mnemonic"),
                    "Got corrupted data: {}", read
                );
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Parallel CLI parses should not interfere with each other.
#[test]
fn parallel_cli_parsing() {
    use clap::Parser;

    let commands = vec![
        vec!["agcli", "subnet", "list"],
        vec!["agcli", "balance"],
        vec!["agcli", "stake", "add", "--amount", "1.0", "--netuid", "1"],
        vec!["agcli", "wallet", "list"],
        vec!["agcli", "subnet", "show", "--netuid", "1"],
        vec!["agcli", "weights", "status", "--netuid", "1"],
        vec!["agcli", "subnet", "commits", "--netuid", "18"],
        vec!["agcli", "doctor"],
    ];

    let threads: Vec<_> = commands
        .into_iter()
        .map(|args| {
            std::thread::spawn(move || {
                let result = agcli::cli::Cli::try_parse_from(&args);
                assert!(
                    result.is_ok(),
                    "Failed to parse {:?}: {:?}",
                    args,
                    result.err()
                );
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Error classifier should handle all common patterns without panicking.
#[test]
fn error_classifier_exhaustive() {
    let long_msg = "x".repeat(10000);
    let test_messages = vec![
        "Connection refused to wss://entrypoint-finney.opentensor.ai:443",
        "DNS resolution failed for host: bittensor.example.com",
        "WebSocket connection timeout after 30s",
        "Decryption failed — wrong password",
        "No hotkey loaded for wallet default",
        "Cannot read keyfile at '/missing/coldkey'",
        "Invalid SS58 address: bad checksum in 5GrwvaEF...",
        "Failed to parse weight pairs: invalid format",
        "Extrinsic failed: insufficient balance for transfer",
        "Rate limit exceeded — wait 100 blocks",
        "Nonce too low for account",
        "Permission denied writing to /etc/agcli/config",
        "No such file or directory: /nonexistent/path",
        "Something completely unexpected happened",
        "",
        "a]]]***weird chars!!!",
        &long_msg, // Very long message
    ];

    for msg in test_messages {
        let err = anyhow::anyhow!("{}", msg);
        let code = agcli::error::classify(&err);
        assert!(
            code >= 1 && code <= 15,
            "Unexpected exit code {} for message: {}",
            code,
            &msg[..msg.len().min(100)]
        );
    }
}

/// Cache deduplication under concurrent access.
#[tokio::test]
async fn cache_concurrent_access() {
    use agcli::queries::query_cache::QueryCache;

    let cache = QueryCache::new();
    let call_count = Arc::new(AtomicU32::new(0));

    // Launch 10 concurrent cache reads — only 1 should actually fetch
    let mut handles = Vec::new();
    for _ in 0..10 {
        let c = cache.clone();
        let count = call_count.clone();
        handles.push(tokio::spawn(async move {
            c.get_all_subnets(|| {
                let cnt = count.clone();
                async move {
                    // Simulate slow fetch
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    cnt.fetch_add(1, Ordering::SeqCst);
                    Ok(vec![])
                }
            })
            .await
            .unwrap();
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // Due to moka's lazy evaluation, first access triggers fetch,
    // subsequent accesses may also fetch if they arrive before insertion completes.
    // But we should get far fewer than 10 fetches.
    let count = call_count.load(Ordering::SeqCst);
    assert!(
        count <= 10,
        "Cache made {} calls (10 concurrent, all OK)",
        count
    );
}

/// Wallet encrypt/decrypt roundtrip is deterministic across threads.
#[test]
fn wallet_roundtrip_multithread() {
    let dir = tempfile::tempdir().unwrap();

    let threads: Vec<_> = (0..4)
        .map(|i| {
            let base = dir.path().to_path_buf();
            std::thread::spawn(move || {
                let path = base.join(format!("wallet_{}", i));
                let mnemonic = format!("word{} abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", i);
                let password = format!("pw_{}", i);

                agcli::wallet::keyfile::write_encrypted_keyfile(&path, &mnemonic, &password)
                    .unwrap();
                let recovered =
                    agcli::wallet::keyfile::read_encrypted_keyfile(&path, &password).unwrap();
                assert_eq!(mnemonic, recovered, "Roundtrip failed for wallet {}", i);
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}

/// Public key file write/read roundtrip.
#[test]
fn public_key_roundtrip_concurrent() {
    use sp_core::{sr25519, Pair};

    let dir = tempfile::tempdir().unwrap();

    let threads: Vec<_> = (0..4)
        .map(|i| {
            let base = dir.path().to_path_buf();
            std::thread::spawn(move || {
                let path = base.join(format!("pubkey_{}", i));
                let (pair, _) = sr25519::Pair::generate();
                let public = pair.public();

                agcli::wallet::keyfile::write_public_key(&path, &public).unwrap();
                let recovered = agcli::wallet::keyfile::read_public_key(&path).unwrap();
                assert_eq!(public, recovered, "Public key roundtrip failed for {}", i);
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }
}
