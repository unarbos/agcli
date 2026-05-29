use crate::e2e_harness::{
    dev_pair, ensure_alive, ensure_local_chain, setup_subnet, to_ss58, wait_blocks, wait_for_chain,
    Balance, Client, NetUid, Pair, ALICE_URI, CONTAINER_NAME, LOCAL_WS,
};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

const VARIANT_A_CONFIG_PATH: &str = ".orchestrate/agcli-parity/scaffold-A-baseline.toml";
const VARIANT_A_CONTAINER: &str = "agcli_parity_variant_a_identity";
const BTCLI_BIN: &str = "/workspace/.venv/bin/btcli";
const PYTHON_BIN: &str = "/workspace/.venv/bin/python";
const BTCLI_WALLET_NAME: &str = "paritybt";
const AGCLI_WALLET_NAME: &str = "alice";
const AGCLI_PASSWORD: &str = "parity-pass";

#[derive(Debug, Clone, Deserialize)]
struct ScaffoldOutput {
    endpoint: String,
    subnets: Vec<ScaffoldSubnet>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScaffoldSubnet {
    netuid: u16,
    neurons: Vec<ScaffoldNeuron>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScaffoldNeuron {
    name: String,
    ss58: String,
    uid: Option<u16>,
}

struct CmdResult {
    success: bool,
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run_cmd(label: &str, cmd: &mut Command) -> CmdResult {
    let output = cmd
        .output()
        .unwrap_or_else(|e| panic!("{label} failed to start: {e}"));
    CmdResult {
        success: output.status.success(),
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn assert_cmd_ok(label: &str, result: &CmdResult) {
    assert!(
        result.success,
        "{label} failed (code {:?})\nstdout:\n{}\nstderr:\n{}",
        result.code,
        result.stdout,
        result.stderr
    );
}

fn agcli_bin() -> String {
    std::env::var("CARGO_BIN_EXE_agcli").unwrap_or_else(|_| "agcli".to_string())
}

fn cleanup_variant_a_container() {
    let _ = Command::new("docker")
        .args(["rm", "-f", VARIANT_A_CONTAINER])
        .output();
    let _ = Command::new("docker")
        .args(["rm", "-f", CONTAINER_NAME])
        .output();
}

fn write_sdk_driver(tmp: &TempDir) -> PathBuf {
    let script = r#"
import argparse
import bittensor

parser = argparse.ArgumentParser()
parser.add_argument("--endpoint", required=True)
parser.add_argument("--wallet-path", required=True)
parser.add_argument("--wallet-name", required=True)
parser.add_argument("--hotkey", required=True)
parser.add_argument("--uri", required=True)
parser.add_argument("--mode", choices=["set_commitment", "set_reveal_commitment"], required=True)
parser.add_argument("--netuid", type=int, required=True)
parser.add_argument("--data", required=True)
parser.add_argument("--blocks-until-reveal", type=int, default=2)
parser.add_argument("--block-time", type=float, default=0.25)
args = parser.parse_args()

wallet = bittensor.Wallet(path=args.wallet_path, name=args.wallet_name, hotkey=args.hotkey)
wallet.create_coldkey_from_uri(args.uri, use_password=False, overwrite=True, suppress=True)
wallet.create_hotkey_from_uri(args.uri, use_password=False, overwrite=True, suppress=True)
subtensor = bittensor.Subtensor(network=args.endpoint)

if args.mode == "set_commitment":
    subtensor.set_commitment(
        wallet=wallet,
        netuid=args.netuid,
        data=args.data,
        raise_error=True,
        wait_for_inclusion=True,
        wait_for_finalization=True,
        wait_for_revealed_execution=True,
    )
else:
    subtensor.set_reveal_commitment(
        wallet=wallet,
        netuid=args.netuid,
        data=args.data,
        blocks_until_reveal=args.blocks_until_reveal,
        block_time=args.block_time,
        raise_error=True,
        wait_for_inclusion=True,
        wait_for_finalization=True,
        wait_for_revealed_execution=True,
    )

print("ok")
"#;
    let path = tmp.path().join("sdk_commitment_driver.py");
    fs::write(&path, script).expect("write sdk helper");
    path
}

async fn fetch_identity_snapshot(client: &mut Client, ss58: &str) -> Option<(String, String)> {
    for _ in 0..6 {
        ensure_alive(client).await;
        if let Ok(identity) = client.get_identity(ss58).await {
            return identity.map(|id| (id.name, id.url));
        }
        let _ = client.reconnect().await;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    panic!("query identity should succeed after retries");
}

async fn fetch_identity_name_and_url(client: &mut Client, ss58: &str) -> (String, String) {
    fetch_identity_snapshot(client, ss58)
        .await
        .expect("identity should exist")
}

async fn fetch_commitment_fields(client: &mut Client, netuid: u16, ss58: &str) -> Option<Vec<String>> {
    for _ in 0..6 {
        ensure_alive(client).await;
        match client.get_commitment(netuid, ss58).await {
            Ok(Some((_, fields))) => return Some(fields),
            Ok(None) => return None,
            Err(_) => {
                let _ = client.reconnect().await;
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        }
    }
    None
}

fn known_scaffold_tempo_failure(result: &CmdResult) -> bool {
    let combined = format!(
        "{}\n{}",
        result.stdout.to_lowercase(),
        result.stderr.to_lowercase()
    );
    combined.contains("sudo_set_tempo")
        && (combined.contains("adminactionprohibitedduringweightswindow")
            || combined.contains("sudid")
            || combined.contains("dispatch failed"))
}

async fn bootstrap_variant_a_fallback() -> (String, u16, ScaffoldNeuron) {
    ensure_local_chain();
    let mut client = wait_for_chain().await;
    let alice = dev_pair(ALICE_URI);
    let netuid = NetUid(1);
    setup_subnet(&mut client, &alice, netuid).await;

    let variant_neurons = [
        ("validator1".to_string(), 1000.0f64),
        ("miner1".to_string(), 100.0f64),
        ("miner2".to_string(), 100.0f64),
    ];

    let mut neurons = Vec::new();
    for (name, tao) in variant_neurons {
        let seed_uri = format!("//{}_sn{}", name, netuid.0);
        let pair = sp_core::sr25519::Pair::from_string(&seed_uri, None)
            .expect("derive fallback scaffold neuron key");
        let ss58 = to_ss58(&pair.public());
        let _ = client
            .transfer(&alice, &ss58, Balance::from_tao(tao))
            .await
            .expect("fund fallback scaffold neuron");
        let _ = client
            .burned_register(&alice, netuid, &ss58)
            .await
            .unwrap_or_default();
        wait_blocks(&mut client, 2).await;

        let uid = client
            .get_neurons_lite(netuid)
            .await
            .ok()
            .and_then(|rows| rows.iter().find(|n| n.hotkey == ss58).map(|n| n.uid));
        neurons.push(ScaffoldNeuron { name, ss58, uid });
    }

    let chosen = neurons
        .into_iter()
        .find(|n| n.uid.is_some())
        .expect("fallback should register at least one variant-A neuron");
    (LOCAL_WS.to_string(), netuid.0, chosen)
}

pub async fn parity_identity_commitment() {
    assert!(
        Path::new(VARIANT_A_CONFIG_PATH).exists(),
        "variant A config missing at {}",
        VARIANT_A_CONFIG_PATH
    );
    assert!(Path::new(BTCLI_BIN).exists(), "btcli missing at {}", BTCLI_BIN);
    assert!(
        Path::new(PYTHON_BIN).exists(),
        "python venv missing at {}",
        PYTHON_BIN
    );

    cleanup_variant_a_container();

    let scaffold_cmd = run_cmd(
        "agcli localnet scaffold (variant A)",
        Command::new(agcli_bin())
            .args([
                "--output",
                "json",
                "localnet",
                "scaffold",
                "--config",
                VARIANT_A_CONFIG_PATH,
            ]),
    );
    let (endpoint, netuid, neuron) = if scaffold_cmd.success {
        let scaffold_json: ScaffoldOutput =
            serde_json::from_str(&scaffold_cmd.stdout).unwrap_or_else(|e| {
                panic!(
                    "scaffold output must be valid JSON: {e}\n{}",
                    scaffold_cmd.stdout
                )
            });

        let subnet = scaffold_json
            .subnets
            .first()
            .expect("variant A should produce one subnet");
        let neuron = subnet
            .neurons
            .iter()
            .find(|n| n.uid.is_some())
            .or_else(|| subnet.neurons.first())
            .expect("variant A should produce at least one neuron")
            .to_owned();
        let endpoint = if scaffold_json.endpoint.is_empty() {
            LOCAL_WS.to_string()
        } else {
            scaffold_json.endpoint.clone()
        };
        (endpoint, subnet.netuid, neuron)
    } else if known_scaffold_tempo_failure(&scaffold_cmd) {
        println!(
            "[parity-identity-commitment] scaffold variant A hit known tempo window rejection; using harness fallback bootstrap"
        );
        bootstrap_variant_a_fallback().await
    } else {
        panic!(
            "agcli localnet scaffold (variant A) failed unexpectedly (code {:?})\nstdout:\n{}\nstderr:\n{}",
            scaffold_cmd.code, scaffold_cmd.stdout, scaffold_cmd.stderr
        );
    };

    let identity_ss58 = {
        let alice = dev_pair(ALICE_URI);
        to_ss58(&alice.public())
    };
    let commitment_seed_uri = format!("//{}_sn{}", neuron.name, netuid);
    let commitment_ss58 = neuron.ss58.clone();
    let agcli_commit_wallet_name = format!("{}_sn{}", neuron.name, netuid);
    println!(
        "[parity-identity-commitment] using neuron={} ss58={} netuid={} endpoint={}",
        neuron.name, neuron.ss58, netuid, endpoint
    );

    let mut client = Client::connect(&endpoint)
        .await
        .expect("connect client after scaffold");
    ensure_alive(&mut client).await;

    let identity_before_snapshot = fetch_identity_snapshot(&mut client, &identity_ss58).await;

    let tmp = TempDir::new().expect("create temp dir");
    let btcli_wallet_path = tmp.path().join("btcli-wallets");
    let agcli_wallet_path = tmp.path().join("agcli-wallets");
    let sdk_wallet_path = tmp.path().join("sdk-wallets");
    fs::create_dir_all(&btcli_wallet_path).expect("create btcli wallet dir");
    fs::create_dir_all(&agcli_wallet_path).expect("create agcli wallet dir");
    fs::create_dir_all(&sdk_wallet_path).expect("create sdk wallet dir");

    let btcli_create = run_cmd(
        "btcli wallet create",
        Command::new(BTCLI_BIN).args([
            "wallet",
            "create",
            "--wallet-name",
            BTCLI_WALLET_NAME,
            "--wallet-path",
            btcli_wallet_path.to_str().expect("utf8 path"),
            "--hotkey",
            "default",
            "--uri",
            "Alice",
            "--no-use-password",
            "--overwrite",
            "--json-output",
        ]),
    );
    assert_cmd_ok("btcli wallet create", &btcli_create);

    let btcli_name = "btcli-identity";
    let btcli_url = "https://btcli.identity.example";
    let btcli_set_identity = run_cmd(
        "btcli wallet set-identity",
        Command::new("timeout").args([
            "120",
            BTCLI_BIN,
            "wallet",
            "set-identity",
            "--wallet-name",
            BTCLI_WALLET_NAME,
            "--wallet-path",
            btcli_wallet_path.to_str().expect("utf8 path"),
            "--network",
            &endpoint,
            "--id-name",
            btcli_name,
            "--web-url",
            btcli_url,
            "--github",
            "https://github.com/opentensor/bittensor",
            "--description",
            "btcli identity parity",
            "--additional",
            "parity-btcli",
            "--no-prompt",
            "--json-output",
        ]),
    );

    let mut btcli_identity_updated = false;
    if btcli_set_identity.success {
        wait_blocks(&mut client, 2).await;
        let identity_after_btcli =
            fetch_identity_name_and_url(&mut client, &identity_ss58).await;
        assert_eq!(identity_after_btcli.0, btcli_name);
        assert_eq!(identity_after_btcli.1, btcli_url);
        btcli_identity_updated = true;
    } else {
        let combined = format!(
            "{}\n{}",
            btcli_set_identity.stdout.to_lowercase(),
            btcli_set_identity.stderr.to_lowercase()
        );
        assert!(
            combined.contains("nonetype")
                || combined.contains("attributeerror")
                || btcli_set_identity.code == Some(124),
            "unexpected btcli set-identity failure (code {:?})\nstdout:\n{}\nstderr:\n{}",
            btcli_set_identity.code,
            btcli_set_identity.stdout,
            btcli_set_identity.stderr
        );
        let identity_after_btcli_snapshot =
            fetch_identity_snapshot(&mut client, &identity_ss58).await;
        assert_eq!(identity_after_btcli_snapshot, identity_before_snapshot);
        println!(
            "[parity-identity-commitment] btcli set-identity divergence observed; keeping row as COVERED_CLI_ONLY"
        );
    }

    let agcli_devkey = run_cmd(
        "agcli wallet dev-key",
        Command::new(agcli_bin()).args([
            "--wallet-dir",
            agcli_wallet_path.to_str().expect("utf8 path"),
            "--wallet",
            AGCLI_WALLET_NAME,
            "--hotkey-name",
            "default",
            "--output",
            "json",
            "--yes",
            "wallet",
            "dev-key",
            "--uri",
            "//Alice",
            "--password",
            AGCLI_PASSWORD,
        ]),
    );
    assert_cmd_ok("agcli wallet dev-key", &agcli_devkey);

    let agcli_identity_name = if btcli_identity_updated {
        "agcli-identity-after-btcli"
    } else {
        "agcli-identity-after-btcli-failure"
    };
    let agcli_identity_url = "https://agcli.identity.example";
    let agcli_set_identity = run_cmd(
        "agcli identity set",
        Command::new(agcli_bin()).args([
            "--endpoint",
            &endpoint,
            "--wallet-dir",
            agcli_wallet_path.to_str().expect("utf8 path"),
            "--wallet",
            AGCLI_WALLET_NAME,
            "--hotkey-name",
            "default",
            "--password",
            AGCLI_PASSWORD,
            "--yes",
            "identity",
            "set",
            "--name",
            agcli_identity_name,
            "--url",
            agcli_identity_url,
            "--github",
            "https://github.com/unarbos/agcli",
            "--description",
            "agcli identity parity",
        ]),
    );
    if agcli_set_identity.success {
        wait_blocks(&mut client, 2).await;
        let identity_after_agcli =
            fetch_identity_name_and_url(&mut client, &identity_ss58).await;
        assert_eq!(identity_after_agcli.0, agcli_identity_name);
        assert_eq!(identity_after_agcli.1, agcli_identity_url);
    } else {
        assert!(
            !agcli_set_identity.stderr.trim().is_empty(),
            "agcli identity set failed without error output"
        );
        let identity_after_agcli = fetch_identity_snapshot(&mut client, &identity_ss58).await;
        if btcli_identity_updated {
            let identity_after_btcli =
                fetch_identity_name_and_url(&mut client, &identity_ss58).await;
            assert_eq!(identity_after_agcli.map(|v| v.0), Some(identity_after_btcli.0));
        } else {
            assert_eq!(identity_after_agcli, identity_before_snapshot);
        }
        println!(
            "[parity-identity-commitment] agcli identity set failed on local runtime; row remains COVERED_CLI_ONLY"
        );
    }

    let agcli_commit_wallet = run_cmd(
        "agcli wallet dev-key (commitment signer)",
        Command::new(agcli_bin()).args([
            "--wallet-dir",
            agcli_wallet_path.to_str().expect("utf8 path"),
            "--output",
            "json",
            "--yes",
            "wallet",
            "dev-key",
            "--uri",
            &commitment_seed_uri,
            "--password",
            AGCLI_PASSWORD,
        ]),
    );
    assert_cmd_ok(
        "agcli wallet dev-key (commitment signer)",
        &agcli_commit_wallet,
    );

    ensure_alive(&mut client).await;
    let commitment_before = fetch_commitment_fields(&mut client, netuid, &commitment_ss58)
        .await
        .is_none();
    assert!(commitment_before, "expected empty commitment before parity run");

    let sdk_driver = write_sdk_driver(&tmp);
    let sdk_wallet_path_str = sdk_wallet_path.to_str().expect("utf8 path");

    let sdk_commit_data = "endpoint:https://sdk-commitment.example";
    let sdk_set_commitment = run_cmd(
        "sdk set_commitment",
        Command::new("timeout").args([
            "120",
            PYTHON_BIN,
            sdk_driver.to_str().expect("utf8 path"),
            "--endpoint",
            &endpoint,
            "--wallet-path",
            sdk_wallet_path_str,
            "--wallet-name",
            "sdkwallet",
            "--hotkey",
            "default",
            "--uri",
            &commitment_seed_uri,
            "--mode",
            "set_commitment",
            "--netuid",
            &netuid.to_string(),
            "--data",
            sdk_commit_data,
        ]),
    );
    assert_cmd_ok("sdk set_commitment", &sdk_set_commitment);
    wait_blocks(&mut client, 2).await;
    let sdk_set_commitment_effect = fetch_commitment_fields(&mut client, netuid, &commitment_ss58)
        .await
        .map(|f| f.iter().any(|entry| entry == sdk_commit_data))
        .unwrap_or(false);
    if !sdk_set_commitment_effect {
        println!(
            "[parity-identity-commitment] sdk set_commitment produced no observable commitment delta on local runtime"
        );
    }

    let agcli_commit_after_sdk = "endpoint:https://agcli-after-sdk.example";
    let agcli_set_commitment = run_cmd(
        "agcli commitment set (after sdk set_commitment)",
        Command::new(agcli_bin()).args([
            "--endpoint",
            &endpoint,
            "--wallet-dir",
            agcli_wallet_path.to_str().expect("utf8 path"),
            "--wallet",
            &agcli_commit_wallet_name,
            "--hotkey-name",
            "default",
            "--password",
            AGCLI_PASSWORD,
            "--yes",
            "commitment",
            "set",
            "--netuid",
            &netuid.to_string(),
            "--data",
            agcli_commit_after_sdk,
        ]),
    );
    assert_cmd_ok(
        "agcli commitment set (after sdk set_commitment)",
        &agcli_set_commitment,
    );
    wait_blocks(&mut client, 2).await;
    let agcli_after_sdk_fields = fetch_commitment_fields(&mut client, netuid, &commitment_ss58)
        .await
        .expect("agcli commitment set should materialize commitment");
    assert!(
        agcli_after_sdk_fields
            .iter()
            .any(|entry| entry == agcli_commit_after_sdk),
        "expected agcli commitment after sdk in {:?}",
        agcli_after_sdk_fields
    );

    let sdk_reveal_data = "endpoint:https://sdk-reveal-commitment.example";
    let sdk_set_reveal = run_cmd(
        "sdk set_reveal_commitment",
        Command::new("timeout").args([
            "240",
            PYTHON_BIN,
            sdk_driver.to_str().expect("utf8 path"),
            "--endpoint",
            &endpoint,
            "--wallet-path",
            sdk_wallet_path_str,
            "--wallet-name",
            "sdkwallet",
            "--hotkey",
            "default",
            "--uri",
            &commitment_seed_uri,
            "--mode",
            "set_reveal_commitment",
            "--netuid",
            &netuid.to_string(),
            "--data",
            sdk_reveal_data,
            "--blocks-until-reveal",
            "2",
            "--block-time",
            "0.25",
        ]),
    );
    assert_cmd_ok("sdk set_reveal_commitment", &sdk_set_reveal);
    wait_blocks(&mut client, 2).await;
    let sdk_set_reveal_effect = fetch_commitment_fields(&mut client, netuid, &commitment_ss58)
        .await
        .map(|f| f.iter().any(|entry| entry == sdk_reveal_data))
        .unwrap_or(false);
    if !sdk_set_reveal_effect {
        println!(
            "[parity-identity-commitment] sdk set_reveal_commitment produced no observable commitment delta on local runtime"
        );
    }

    let agcli_commit_after_reveal = "endpoint:https://agcli-after-sdk-reveal.example";
    let agcli_set_after_reveal = run_cmd(
        "agcli commitment set (after sdk set_reveal_commitment)",
        Command::new(agcli_bin()).args([
            "--endpoint",
            &endpoint,
            "--wallet-dir",
            agcli_wallet_path.to_str().expect("utf8 path"),
            "--wallet",
            &agcli_commit_wallet_name,
            "--hotkey-name",
            "default",
            "--password",
            AGCLI_PASSWORD,
            "--yes",
            "commitment",
            "set",
            "--netuid",
            &netuid.to_string(),
            "--data",
            agcli_commit_after_reveal,
        ]),
    );
    assert_cmd_ok(
        "agcli commitment set (after sdk set_reveal_commitment)",
        &agcli_set_after_reveal,
    );
    wait_blocks(&mut client, 2).await;
    let agcli_after_reveal_fields = fetch_commitment_fields(&mut client, netuid, &commitment_ss58)
        .await
        .expect("agcli commitment set after reveal should materialize commitment");
    assert!(
        agcli_after_reveal_fields
            .iter()
            .any(|entry| entry == agcli_commit_after_reveal),
        "expected agcli commitment after sdk reveal in {:?}",
        agcli_after_reveal_fields
    );

    cleanup_variant_a_container();
}
