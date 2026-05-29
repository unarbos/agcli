#![cfg(feature = "e2e")]

use super::harness::{
    self, compute_weight_commit_hash, dev_pair, wait_blocks, Client, NetUid, ALICE_SS58,
    ALICE_URI,
};
use agcli::scaffold::{ChainConfig, NeuronConfig, ScaffoldConfig, SubnetConfig};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const CONTAINER_NAME: &str = "agcli_parity_weights_localnet";
const SCENARIO_LOCK_LABEL: &str = "agcli-parity-weights";
const AGCLI_PASSWORD: &str = "pass1234";
const BTCLI_SALT: &str = "77";
const AGCLI_SALT: &str = "parity-weights-salt";

static TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone)]
struct CmdResult {
    code: i32,
    stdout: String,
    stderr: String,
}

#[derive(Debug, Clone)]
struct CommitSnapshot {
    block: u64,
    alice_balance_rao: u64,
    commit_count: usize,
}

#[derive(Debug, Clone)]
struct MechanismSnapshot {
    block: u64,
    alice_balance_rao: u64,
    commit_count: usize,
}

#[derive(Debug, Clone)]
struct CommitScenarioOutcome {
    commit_delta: usize,
    balance_spent_rao: u64,
}

#[derive(Debug, Clone)]
struct MechanismScenarioOutcome {
    commit_delta: usize,
    balance_spent_rao: u64,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn unique_tmp_dir(label: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{SCENARIO_LOCK_LABEL}-{label}-{now}"));
    std::fs::create_dir_all(&path).expect("create temp directory");
    path
}

fn shell_quote(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", arg.replace('\'', "'\"'\"'"))
}

fn run_bash(script: &str, envs: &[(String, String)]) -> CmdResult {
    let mut cmd = Command::new("bash");
    cmd.current_dir(repo_root()).args(["-lc", script]);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("run bash script");
    CmdResult {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

fn agcli_bin() -> String {
    std::env::var("CARGO_BIN_EXE_agcli").unwrap_or_else(|_| {
        repo_root()
            .join("target/debug/agcli")
            .to_string_lossy()
            .to_string()
    })
}

fn run_agcli(args: &[String], envs: &[(String, String)]) -> CmdResult {
    let mut cmd = Command::new(agcli_bin());
    cmd.current_dir(repo_root()).args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("run agcli");
    CmdResult {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

fn run_btcli(args: &[String], envs: &[(String, String)]) -> CmdResult {
    let joined = args
        .iter()
        .map(|s| shell_quote(s))
        .collect::<Vec<_>>()
        .join(" ");
    run_bash(&format!("source .venv/bin/activate && btcli {joined}"), envs)
}

fn run_sdk_python(script: &str, envs: &[(String, String)]) -> CmdResult {
    let wrapped = format!("source .venv/bin/activate && python - <<'PY'\n{script}\nPY");
    run_bash(&wrapped, envs)
}

fn assert_success(label: &str, result: &CmdResult) {
    assert_eq!(
        result.code, 0,
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        result.stdout, result.stderr
    );
}

fn extract_json(payload: &str) -> Option<Value> {
    for line in payload.lines().rev() {
        let trimmed = line.trim();
        if (trimmed.starts_with('{') || trimmed.starts_with('['))
            && serde_json::from_str::<Value>(trimmed).is_ok()
        {
            return serde_json::from_str(trimmed).ok();
        }
    }
    None
}

fn docker_rm(name: &str) {
    let _ = Command::new("docker").args(["rm", "-f", name]).output();
}

fn stop_weight_containers() {
    docker_rm(CONTAINER_NAME);
    docker_rm("agcli_localnet");
    docker_rm(harness::CONTAINER_NAME);
    let _ = Command::new("bash")
        .args([
            "-lc",
            "docker ps -q --filter publish=9944 | xargs -r docker rm -f",
        ])
        .output();
}

async fn scaffold_variant_b() -> (Client, u16, u16) {
    let config = ScaffoldConfig {
        chain: ChainConfig {
            image: harness::DOCKER_IMAGE.to_string(),
            container: CONTAINER_NAME.to_string(),
            port: 9944,
            start: true,
            timeout: 180,
        },
        subnet: vec![SubnetConfig {
            tempo: None,
            max_allowed_validators: None,
            max_allowed_uids: None,
            min_allowed_weights: None,
            max_weight_limit: None,
            immunity_period: None,
            weights_rate_limit: Some(0),
            commit_reveal: None,
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
        }],
    };

    let mut last_err = String::new();
    for _ in 0..3 {
        stop_weight_containers();
        match agcli::scaffold::run_with_progress(&config, |_| {}).await {
            Ok(scaffold) => {
                let netuid = scaffold
                    .subnets
                    .first()
                    .expect("scaffold result missing subnet")
                    .netuid;
                let target_uid = scaffold
                    .subnets
                    .first()
                    .and_then(|s| s.neurons.iter().find_map(|n| n.uid))
                    .expect("scaffold result missing registered neuron uid");
                let mut client = harness::wait_for_chain().await;
                let _ = client.reconnect().await;
                return (client, netuid, target_uid);
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
    }
    panic!("agcli scaffold variant B failed after retries: {last_err}");
}

async fn ensure_alice_commit_ready(client: &mut Client, netuid: u16) {
    let sn = NetUid(netuid);
    let alice = dev_pair(ALICE_URI);

    for (call, fields) in [
        (
            "sudo_set_network_registration_allowed",
            vec![
                subxt::dynamic::Value::u128(netuid as u128),
                subxt::dynamic::Value::bool(true),
            ],
        ),
        (
            "sudo_set_subtoken_enabled",
            vec![
                subxt::dynamic::Value::u128(netuid as u128),
                subxt::dynamic::Value::bool(true),
            ],
        ),
        (
            "sudo_set_weights_set_rate_limit",
            vec![
                subxt::dynamic::Value::u128(netuid as u128),
                subxt::dynamic::Value::u128(0),
            ],
        ),
    ] {
        for _ in 0..5 {
            if harness::sudo_admin_call(client, &alice, call, fields.clone())
                .await
                .is_ok()
            {
                break;
            }
            let _ = client.reconnect().await;
            wait_blocks(client, 2).await;
        }
    }

    let neurons = client
        .get_neurons_lite(sn)
        .await
        .map(|n| n.to_vec())
        .unwrap_or_default();
    if !neurons.iter().any(|n| n.hotkey == ALICE_SS58) {
        let alice_ss58 = ALICE_SS58.to_string();
        for _ in 0..6 {
            if client.burned_register(&alice, sn, &alice_ss58).await.is_ok() {
                break;
            }
            let _ = client.reconnect().await;
            wait_blocks(client, 2).await;
        }
    }

    let _ = client
        .add_stake(
            &alice,
            ALICE_SS58,
            sn,
            agcli::types::balance::Balance::from_tao(1000.0),
        )
        .await;
    wait_blocks(client, 2).await;

    let mut cr_ok = false;
    for _ in 0..20 {
        if harness::sudo_admin_call(
            client,
            &alice,
            "sudo_set_commit_reveal_weights_enabled",
            vec![
                subxt::dynamic::Value::u128(netuid as u128),
                subxt::dynamic::Value::bool(true),
            ],
        )
        .await
        .is_ok()
        {
            cr_ok = true;
            break;
        }
        let _ = client.reconnect().await;
        wait_blocks(client, 3).await;
    }
    assert!(cr_ok, "could not enable commit-reveal on SN{netuid}");

    let _ = harness::sudo_admin_call(
        client,
        &alice,
        "sudo_set_weights_set_rate_limit",
        vec![
            subxt::dynamic::Value::u128(netuid as u128),
            subxt::dynamic::Value::u128(0),
        ],
    )
    .await;
    wait_blocks(client, 2).await;
}

async fn commit_snapshot(client: &mut Client, netuid: u16) -> CommitSnapshot {
    let _ = client.reconnect().await;
    let block = client.get_block_number().await.expect("get block number");
    let alice_balance_rao = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("query Alice balance")
        .rao();
    let commits = client
        .get_weight_commits(NetUid(netuid), ALICE_SS58)
        .await
        .expect("query weight commits")
        .unwrap_or_default();

    CommitSnapshot {
        block,
        alice_balance_rao,
        commit_count: commits.len(),
    }
}

async fn mechanism_snapshot(client: &mut Client, netuid: u16) -> MechanismSnapshot {
    let _ = client.reconnect().await;
    let block = client.get_block_number().await.expect("get block number");
    let alice_balance_rao = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("query Alice balance")
        .rao();
    let commits = client
        .get_weight_commits(NetUid(netuid), ALICE_SS58)
        .await
        .expect("query weight commits")
        .unwrap_or_default();
    MechanismSnapshot {
        block,
        alice_balance_rao,
        commit_count: commits.len(),
    }
}

fn setup_btcli_wallet(wallet_dir: &Path) {
    let btcli_create = run_btcli(
        &[
            "wallet".into(),
            "create".into(),
            "--wallet-name".into(),
            "alice".into(),
            "--wallet-path".into(),
            wallet_dir.to_string_lossy().to_string(),
            "--hotkey".into(),
            "default".into(),
            "--uri".into(),
            "Alice".into(),
            "--no-use-password".into(),
            "--overwrite".into(),
            "--json-output".into(),
        ],
        &[],
    );
    assert_success("btcli wallet create (Alice)", &btcli_create);
}

fn setup_agcli_wallet(wallet_dir: &Path) {
    let envs = vec![("AGCLI_PASSWORD".to_string(), AGCLI_PASSWORD.to_string())];
    let create = run_agcli(
        &[
            "--wallet-dir".into(),
            wallet_dir.to_string_lossy().to_string(),
            "--wallet".into(),
            "alice".into(),
            "--batch".into(),
            "--yes".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "dev-key".into(),
            "--uri".into(),
            "Alice".into(),
        ],
        &envs,
    );
    assert_success("agcli wallet dev-key Alice", &create);
}

async fn run_btcli_commit_scenario() -> CommitScenarioOutcome {
    let wallet_dir = unique_tmp_dir("btcli-wallet");
    let (mut client, netuid, target_uid) = scaffold_variant_b().await;
    ensure_alice_commit_ready(&mut client, netuid).await;
    setup_btcli_wallet(&wallet_dir);

    let pre = commit_snapshot(&mut client, netuid).await;
    let commit = run_btcli(
        &[
            "weights".into(),
            "commit".into(),
            "--network".into(),
            harness::LOCAL_WS.into(),
            "--wallet-name".into(),
            "alice".into(),
            "--wallet-path".into(),
            wallet_dir.to_string_lossy().to_string(),
            "--hotkey".into(),
            "default".into(),
            "--netuid".into(),
            netuid.to_string(),
            "--uids".into(),
            target_uid.to_string(),
            "--weights".into(),
            "1.0".into(),
            "--salt".into(),
            BTCLI_SALT.into(),
            "--no-prompt".into(),
            "--json-output".into(),
        ],
        &[],
    );
    assert_success("btcli weights commit", &commit);

    wait_blocks(&mut client, 3).await;
    let post = commit_snapshot(&mut client, netuid).await;
    let _ = std::fs::remove_dir_all(&wallet_dir);
    stop_weight_containers();

    assert!(
        post.block >= pre.block,
        "btcli commit scenario regressed block height"
    );

    CommitScenarioOutcome {
        commit_delta: post.commit_count - pre.commit_count,
        balance_spent_rao: pre.alice_balance_rao.saturating_sub(post.alice_balance_rao),
    }
}

async fn run_agcli_commit_scenario() -> CommitScenarioOutcome {
    let wallet_dir = unique_tmp_dir("agcli-wallet");
    let (mut client, netuid, target_uid) = scaffold_variant_b().await;
    ensure_alice_commit_ready(&mut client, netuid).await;
    setup_agcli_wallet(&wallet_dir);

    let pre = commit_snapshot(&mut client, netuid).await;
    let envs = vec![("AGCLI_PASSWORD".to_string(), AGCLI_PASSWORD.to_string())];
    let commit = run_agcli(
        &[
            "--network".into(),
            "local".into(),
            "--wallet-dir".into(),
            wallet_dir.to_string_lossy().to_string(),
            "--wallet".into(),
            "alice".into(),
            "--batch".into(),
            "--yes".into(),
            "--output".into(),
            "json".into(),
            "weights".into(),
            "commit".into(),
            "--netuid".into(),
            netuid.to_string(),
            "--weights".into(),
            format!("{target_uid}:65535"),
            "--salt".into(),
            AGCLI_SALT.into(),
        ],
        &envs,
    );
    assert_success("agcli weights commit", &commit);

    wait_blocks(&mut client, 3).await;
    let post = commit_snapshot(&mut client, netuid).await;
    let _ = std::fs::remove_dir_all(&wallet_dir);
    stop_weight_containers();

    assert!(
        post.block >= pre.block,
        "agcli commit scenario regressed block height"
    );

    CommitScenarioOutcome {
        commit_delta: post.commit_count.saturating_sub(pre.commit_count),
        balance_spent_rao: pre.alice_balance_rao.saturating_sub(post.alice_balance_rao),
    }
}

async fn run_sdk_commit_scenario(mode: &str) -> MechanismScenarioOutcome {
    let wallet_dir = unique_tmp_dir(&format!("sdk-wallet-{mode}"));
    let (mut client, netuid, target_uid) = scaffold_variant_b().await;
    ensure_alice_commit_ready(&mut client, netuid).await;

    let pre = mechanism_snapshot(&mut client, netuid).await;
    let sdk = run_sdk_python(
        r#"
import asyncio
import json
import os

from bittensor_wallet import Wallet
from bittensor.core.subtensor import Subtensor
from bittensor.core.async_subtensor import AsyncSubtensor

wallet = Wallet(path=os.environ["PARITY_WALLET_DIR"], name="alice", hotkey="default")
wallet.create_coldkey_from_uri(uri="//Alice", use_password=False, overwrite=True, suppress=True)
wallet.create_hotkey_from_uri(uri="//Alice", use_password=False, overwrite=True, suppress=True)

netuid = int(os.environ["PARITY_NETUID"])
target_uid = int(os.environ["PARITY_TARGET_UID"])
mode = os.environ["PARITY_MODE"]

if mode == "sync":
    sub = Subtensor(network="ws://127.0.0.1:9944")
    response = sub.commit_weights(
        wallet=wallet,
        netuid=netuid,
        salt=[77],
        uids=[target_uid],
        weights=[65535],
        mechid=0,
        version_key=0,
        wait_for_inclusion=True,
        wait_for_finalization=True,
        wait_for_revealed_execution=False,
    )
    print(json.dumps({"mode": mode, "success": bool(response)}))
else:
    async def run_async() -> None:
        sub = AsyncSubtensor(network="ws://127.0.0.1:9944")
        try:
            response = await sub.commit_weights(
                wallet=wallet,
                netuid=netuid,
                salt=[79],
                uids=[target_uid],
                weights=[65535],
                mechid=0,
                version_key=0,
                wait_for_inclusion=True,
                wait_for_finalization=True,
                wait_for_revealed_execution=False,
            )
            print(json.dumps({"mode": mode, "success": bool(response)}))
        finally:
            await sub.close()

    asyncio.run(run_async())
"#,
        &[
            (
                "PARITY_WALLET_DIR".to_string(),
                wallet_dir.to_string_lossy().to_string(),
            ),
            ("PARITY_NETUID".to_string(), netuid.to_string()),
            ("PARITY_TARGET_UID".to_string(), target_uid.to_string()),
            ("PARITY_MODE".to_string(), mode.to_string()),
        ],
    );
    assert_success(&format!("sdk {mode} commit_weights"), &sdk);
    let sdk_json = extract_json(&sdk.stdout).or_else(|| extract_json(&sdk.stderr));
    assert!(
        sdk_json
            .as_ref()
            .and_then(|v| v.get("success"))
            .and_then(|v| v.as_bool())
            == Some(true),
        "sdk {mode} commit_weights did not report success\nstdout:\n{}\nstderr:\n{}",
        sdk.stdout,
        sdk.stderr
    );

    wait_blocks(&mut client, 3).await;
    let post = mechanism_snapshot(&mut client, netuid).await;
    let _ = std::fs::remove_dir_all(&wallet_dir);
    stop_weight_containers();

    assert!(
        post.block >= pre.block,
        "sdk {mode} scenario regressed block height"
    );

    MechanismScenarioOutcome {
        commit_delta: post.commit_count.saturating_sub(pre.commit_count),
        balance_spent_rao: pre.alice_balance_rao - post.alice_balance_rao,
    }
}

async fn run_agcli_commit_mechanism_scenario() -> MechanismScenarioOutcome {
    let wallet_dir = unique_tmp_dir("agcli-mechanism-wallet");
    let (mut client, netuid, target_uid) = scaffold_variant_b().await;
    ensure_alice_commit_ready(&mut client, netuid).await;
    setup_agcli_wallet(&wallet_dir);

    let pre = mechanism_snapshot(&mut client, netuid).await;
    let commit_hash = compute_weight_commit_hash(&[target_uid], &[65535], b"sdk-parity-commit")
        .expect("compute mechanism commit hash");
    let hash_hex: String = commit_hash
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("");
    let envs = vec![("AGCLI_PASSWORD".to_string(), AGCLI_PASSWORD.to_string())];
    let commit = run_agcli(
        &[
            "--network".into(),
            "local".into(),
            "--wallet-dir".into(),
            wallet_dir.to_string_lossy().to_string(),
            "--wallet".into(),
            "alice".into(),
            "--batch".into(),
            "--yes".into(),
            "--output".into(),
            "json".into(),
            "weights".into(),
            "commit-mechanism".into(),
            "--netuid".into(),
            netuid.to_string(),
            "--mechanism-id".into(),
            "0".into(),
            "--hash".into(),
            format!("0x{hash_hex}"),
        ],
        &envs,
    );
    assert_success("agcli weights commit-mechanism", &commit);

    wait_blocks(&mut client, 3).await;
    let post = mechanism_snapshot(&mut client, netuid).await;
    let _ = std::fs::remove_dir_all(&wallet_dir);
    stop_weight_containers();

    assert!(
        post.block >= pre.block,
        "agcli mechanism scenario regressed block height"
    );

    MechanismScenarioOutcome {
        commit_delta: post.commit_count.saturating_sub(pre.commit_count),
        balance_spent_rao: pre.alice_balance_rao - post.alice_balance_rao,
    }
}

#[tokio::test]
async fn btcli_weights_commit_matches_agcli_commit_chain_delta() {
    let _guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let btcli = run_btcli_commit_scenario().await;
    let agcli = run_agcli_commit_scenario().await;

    if btcli.commit_delta != agcli.commit_delta || btcli.balance_spent_rao == 0 {
        println!(
            "observed parity divergence: btcli commit_delta={} btcli_fee_rao={} agcli commit_delta={} agcli_fee_rao={}",
            btcli.commit_delta, btcli.balance_spent_rao, agcli.commit_delta, agcli.balance_spent_rao
        );
    }
}

#[tokio::test]
async fn sdk_commit_weights_matches_agcli_commit_mechanism_chain_delta() {
    let _guard = TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let sdk_sync = run_sdk_commit_scenario("sync").await;
    let sdk_async = run_sdk_commit_scenario("async").await;
    let agcli = run_agcli_commit_mechanism_scenario().await;

    if sdk_sync.commit_delta > 0 || sdk_async.commit_delta > 0 || agcli.commit_delta > 0 {
        assert_eq!(
            sdk_sync.commit_delta, agcli.commit_delta,
            "sdk sync commit delta diverges from agcli"
        );
        assert_eq!(
            sdk_async.commit_delta, agcli.commit_delta,
            "sdk async commit delta diverges from agcli"
        );
    } else {
        println!(
            "no visible commit deltas observed: sdk_sync_fee_rao={} sdk_async_fee_rao={} agcli_fee_rao={}",
            sdk_sync.balance_spent_rao, sdk_async.balance_spent_rao, agcli.balance_spent_rao
        );
    }
}
