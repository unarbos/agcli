#[path = "../e2e_modules/harness.rs"]
mod harness;

use std::collections::HashSet;

use harness::{
    ALICE_URI, BOB_URI, Command, LOCAL_WS, dev_pair, ensure_alive, ensure_local_chain, to_ss58,
    wait_blocks, wait_for_chain,
};
use sp_core::Pair as _;
use subxt::dynamic::Value;

fn governance_unavailable(message: &str) -> bool {
    message.contains("Pallet 'Triumvirate' not found")
        || message.contains("Pallet with name Triumvirate not found")
        || message.contains("Call Triumvirate.")
        || message.contains("Call SubtensorModule.vote not found")
}

async fn submit_governance_proposal(
    client: &mut harness::Client,
    proposer: &sp_core::sr25519::Pair,
) -> anyhow::Result<[u8; 32]> {
    let before = client.get_triumvirate_proposals().await.unwrap_or_default();
    let before: HashSet<[u8; 32]> = before.into_iter().collect();

    let remark_call = subxt::dynamic::tx("System", "remark", vec![Value::from_bytes(vec![0u8])]);
    client
        .submit_raw_call(proposer, "Triumvirate", "propose", vec![
            remark_call.into_value(),
            Value::u128(100_000),
            Value::u128(100_000_000),
        ])
        .await?;

    wait_blocks(client, 3).await;
    let after = client.get_triumvirate_proposals().await?;
    if let Some(hash) = after.iter().copied().find(|h| !before.contains(h)) {
        return Ok(hash);
    }
    after
        .last()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("No proposal hash found after Triumvirate.propose"))
}

#[tokio::test]
async fn parity_senate_vote() {
    ensure_local_chain();
    let mut client = wait_for_chain().await;

    let alice = dev_pair(ALICE_URI);
    let bob = dev_pair(BOB_URI);
    let alice_ss58 = to_ss58(&alice.public());
    let bob_ss58 = to_ss58(&bob.public());

    ensure_alive(&mut client).await;
    if let Err(e) = client.root_register(&alice, &alice_ss58).await {
        let msg = e.to_string();
        if !msg.contains("AlreadyRegistered") && !governance_unavailable(&msg) {
            panic!("root_register Alice failed: {msg}");
        }
    }
    wait_blocks(&mut client, 2).await;

    ensure_alive(&mut client).await;
    if let Err(e) = client.root_register(&bob, &bob_ss58).await {
        let msg = e.to_string();
        if !msg.contains("AlreadyRegistered") && !governance_unavailable(&msg) {
            panic!("root_register Bob failed: {msg}");
        }
    }
    wait_blocks(&mut client, 2).await;

    let proposal_hash = match submit_governance_proposal(&mut client, &alice).await {
        Ok(hash) => hash,
        Err(e) => {
            let msg = e.to_string();
            if governance_unavailable(&msg) {
                eprintln!("Skipping senate vote parity on this runtime: {msg}");
                return;
            }
            panic!("failed to submit governance proposal: {msg}");
        }
    };

    let before = client
        .get_triumvirate_vote_data(proposal_hash)
        .await
        .expect("query vote data before vote");
    let Some(before) = before else {
        panic!(
            "missing vote data for proposal 0x{}",
            hex::encode(proposal_hash)
        );
    };
    let before_ayes = before.ayes.len();
    let before_nays = before.nays.len();

    let args_json = format!("[\"0x{}\", \"yes\"]", hex::encode(proposal_hash));
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
            "senate-vote",
            "--args",
            &args_json,
            "--sudo-key",
            ALICE_URI,
        ])
        .output()
        .expect("failed to execute agcli senate-vote alias");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if governance_unavailable(&stderr) {
            eprintln!("Skipping senate vote parity on this runtime: {stderr}");
            return;
        }
        panic!(
            "senate-vote CLI failed\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
    }

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("senate-vote output is not valid JSON");
    assert!(
        json.get("tx_hash")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false),
        "expected non-empty tx_hash from senate-vote alias: {}",
        json
    );

    wait_blocks(&mut client, 2).await;
    let after = client
        .get_triumvirate_vote_data(proposal_hash)
        .await
        .expect("query vote data after vote")
        .expect("vote data should still exist after voting");

    assert!(
        after.ayes.iter().any(|acct| acct == &alice_ss58),
        "expected Alice vote in ayes for proposal 0x{}",
        hex::encode(proposal_hash)
    );
    assert_eq!(
        after.nays.len(),
        before_nays,
        "senate-vote aye should not increase nays count"
    );
    assert!(
        after.ayes.len() >= before_ayes,
        "senate-vote should not decrease ayes count"
    );
}
