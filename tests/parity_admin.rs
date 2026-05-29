#![cfg(feature = "e2e")]

#[path = "e2e_modules/harness.rs"]
mod e2e_harness;

use e2e_harness::*;
use std::process::Command;

#[tokio::test]
async fn parity_stake_burn() {
    ensure_local_chain();
    let mut client = wait_for_chain().await;
    let alice = dev_pair(ALICE_URI);
    let netuid = NetUid(1);

    ensure_alive(&mut client).await;
    setup_subnet(&mut client, &alice, netuid).await;
    let _ = ensure_alice_on_subnet(&mut client, netuid).await;
    wait_blocks(&mut client, 3).await;

    let subnet_before = client
        .get_subnet_info(netuid)
        .await
        .expect("query subnet info before stake-burn")
        .expect("subnet should exist before stake-burn");
    let burn_before = subnet_before.burn.rao();
    let balance_before = client
        .get_balance(&alice.public())
        .await
        .expect("query alice balance before stake-burn")
        .rao();

    let amount_tao = 1.0f64;
    let amount_rao = Balance::from_tao(amount_tao).rao();
    let args_json = format!("[{},{}]", netuid.0, amount_tao);
    let output = Command::new(env!("CARGO_BIN_EXE_agcli"))
        .args([
            "--network",
            "local",
            "--endpoint",
            LOCAL_WS,
            "--output",
            "json",
            "--yes",
            "admin",
            "raw",
            "--call",
            "stake-burn",
            "--args",
            &args_json,
            "--sudo-key",
            "//Alice",
        ])
        .output()
        .expect("failed to invoke agcli admin raw stake-burn");

    assert!(
        output.status.success(),
        "stake-burn command failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout_json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stake-burn stdout should be valid JSON");
    let tx_hash = stdout_json
        .get("tx_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        !tx_hash.is_empty(),
        "expected non-empty tx_hash output: {}",
        stdout_json
    );

    wait_blocks(&mut client, 4).await;
    ensure_alive(&mut client).await;

    let subnet_after = client
        .get_subnet_info(netuid)
        .await
        .expect("query subnet info after stake-burn")
        .expect("subnet should exist after stake-burn");
    let burn_after = subnet_after.burn.rao();
    let balance_after = client
        .get_balance(&alice.public())
        .await
        .expect("query alice balance after stake-burn")
        .rao();
    let balance_delta = balance_before.saturating_sub(balance_after);

    assert!(
        balance_delta > 0,
        "expected alice balance to decrease, before={} after={}",
        balance_before,
        balance_after
    );
    assert!(
        burn_after > burn_before || balance_delta >= amount_rao,
        "expected stake-burn chain effect, burn {} -> {}, balance delta {} (amount_rao={})",
        burn_before,
        burn_after,
        balance_delta,
        amount_rao
    );
}
