#![cfg(feature = "e2e")]

use super::harness::{self, ensure_alive, wait_blocks, Client, ALICE_SS58};
use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const BTCLI_CONTAINER: &str = "agcli_parity_wallet_btcli";
const AGCLI_CONTAINER: &str = "agcli_parity_wallet_agcli";
const TEST_MNEMONIC: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

static TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone)]
struct ChainSnapshot {
    block: u64,
    alice_balance_rao: u64,
    alice_swap: Option<(u32, String)>,
}

#[derive(Debug, Clone)]
struct CmdResult {
    code: i32,
    stdout: String,
    stderr: String,
}

impl CmdResult {
    fn merged(&self) -> String {
        if self.stderr.trim().is_empty() {
            self.stdout.clone()
        } else if self.stdout.trim().is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn unique_tmp_dir(label: &str) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("agcli-parity-wallet-{label}-{now}"));
    std::fs::create_dir_all(&path).expect("create temp parity wallet dir");
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
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let out = cmd.output().expect("run bash command");
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

fn agcli_bin_path() -> String {
    std::env::var("CARGO_BIN_EXE_agcli").unwrap_or_else(|_| {
        repo_root()
            .join("target/release/agcli")
            .to_string_lossy()
            .to_string()
    })
}

fn run_agcli(args: &[String], envs: &[(String, String)]) -> CmdResult {
    let mut cmd = Command::new(agcli_bin_path());
    cmd.current_dir(repo_root()).args(args);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let out = cmd.output().expect("run agcli command");
    CmdResult {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    }
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

    let compact = payload.lines().map(|s| s.trim()).collect::<String>();
    if let (Some(start), Some(end)) = (compact.find('{'), compact.rfind('}')) {
        return serde_json::from_str(&compact[start..=end]).ok();
    }
    if let (Some(start), Some(end)) = (compact.find('['), compact.rfind(']')) {
        return serde_json::from_str(&compact[start..=end]).ok();
    }
    None
}

fn assert_success(label: &str, result: &CmdResult) {
    assert_eq!(
        result.code,
        0,
        "{label} failed\nstdout:\n{}\nstderr:\n{}",
        result.stdout,
        result.stderr
    );
}

fn assert_structured_json_error(label: &str, result: &CmdResult) {
    assert!(
        result.code != 0,
        "{label} unexpectedly succeeded:\n{}",
        result.merged()
    );
    let payload = result.merged();
    let json = extract_json(&payload).unwrap_or_else(|| {
        panic!("{label} did not emit parsable JSON error:\n{payload}");
    });
    assert!(
        json.get("error").and_then(|v| v.as_bool()) == Some(true),
        "{label} JSON payload missing error=true:\n{json}"
    );
    assert!(
        json.get("code").and_then(|v| v.as_i64()).is_some(),
        "{label} JSON payload missing numeric error code:\n{json}"
    );
}

fn docker_rm(name: &str) {
    let _ = Command::new("docker").args(["rm", "-f", name]).output();
}

fn stop_wallet_containers() {
    docker_rm(BTCLI_CONTAINER);
    docker_rm(AGCLI_CONTAINER);
    docker_rm("agcli_localnet");
    docker_rm(harness::CONTAINER_NAME);
}

fn start_wallet_container(name: &str) {
    stop_wallet_containers();
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-d",
            "--name",
            name,
            "-p",
            "9944:9944",
            "-p",
            "9945:9945",
            harness::DOCKER_IMAGE,
        ])
        .output()
        .expect("start wallet parity localnet container");
    assert!(
        output.status.success(),
        "could not start {name}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

async fn snapshot(client: &mut Client) -> ChainSnapshot {
    ensure_alive(client).await;
    let block = client.get_block_number().await.expect("query block number");
    let alice_balance = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("query Alice balance")
        .rao();
    let alice_swap = client
        .get_coldkey_swap_scheduled(ALICE_SS58)
        .await
        .expect("query coldkey swap status");
    ChainSnapshot {
        block,
        alice_balance_rao: alice_balance,
        alice_swap,
    }
}

#[tokio::test]
async fn wallet_filesystem_and_sdk_workflows_parity() {
    let _guard = TEST_LOCK.lock().expect("lock wallet parity tests");

    start_wallet_container(BTCLI_CONTAINER);
    let mut btcli_client = harness::wait_for_chain().await;
    let btcli_pre = snapshot(&mut btcli_client).await;

    let btcli_wallet_dir = unique_tmp_dir("btcli");
    let btcli_wallet_dir_str = btcli_wallet_dir.to_string_lossy().to_string();

    let btcli_create = run_btcli(
        &[
            "wallet".into(),
            "create".into(),
            "--wallet-name".into(),
            "walletbt".into(),
            "--wallet-path".into(),
            btcli_wallet_dir_str.clone(),
            "--hotkey".into(),
            "hkb".into(),
            "--uri".into(),
            "Alice".into(),
            "--no-use-password".into(),
            "--overwrite".into(),
            "--json-output".into(),
        ],
        &[],
    );
    assert_success("btcli wallet create", &btcli_create);

    let btcli_list = run_btcli(
        &[
            "wallet".into(),
            "list".into(),
            "--wallet-path".into(),
            btcli_wallet_dir_str.clone(),
            "--json-output".into(),
        ],
        &[],
    );
    assert_success("btcli wallet list", &btcli_list);

    let btcli_regen_hotkey = run_btcli(
        &[
            "wallet".into(),
            "regen-hotkey".into(),
            "--wallet-name".into(),
            "walletbt".into(),
            "--wallet-path".into(),
            btcli_wallet_dir_str.clone(),
            "--hotkey".into(),
            "regenhkb".into(),
            "--mnemonic".into(),
            TEST_MNEMONIC.into(),
            "--no-use-password".into(),
            "--overwrite".into(),
            "--json-output".into(),
        ],
        &[],
    );
    assert_success("btcli wallet regen-hotkey", &btcli_regen_hotkey);

    let sdk_checks = run_sdk_python(
        r#"
import json
import os
from bittensor_wallet import Keypair, Wallet

wallet_path = os.environ["PARITY_WALLET_DIR"]

sdk_wallet = Wallet(path=wallet_path, name="sdkwallet", hotkey="sdkhk")
sdk_wallet.create_new_coldkey(use_password=False, overwrite=True, suppress=True)
sdk_wallet.create_new_hotkey(use_password=False, overwrite=True, suppress=True)
sdk_wallet.regenerate_hotkey(mnemonic="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", use_password=False, suppress=True, overwrite=True)
derived_from_uri = Keypair.create_from_uri("//Bob").ss58_address
derived_from_mnemonic = Keypair.create_from_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").ss58_address
print(json.dumps({
    "create": sdk_wallet.coldkeypub.ss58_address,
    "derive_uri": derived_from_uri,
    "derive_mnemonic": derived_from_mnemonic
}))
"#,
        &[("PARITY_WALLET_DIR".into(), btcli_wallet_dir_str.clone())],
    );
    assert_success("sdk wallet create/regen/derive", &sdk_checks);
    assert!(
        extract_json(&sdk_checks.stdout).is_some(),
        "sdk check output missing JSON: {}",
        sdk_checks.stdout
    );

    wait_blocks(&mut btcli_client, 2).await;
    let btcli_post = snapshot(&mut btcli_client).await;
    stop_wallet_containers();

    start_wallet_container(AGCLI_CONTAINER);
    let mut agcli_client = harness::wait_for_chain().await;
    let agcli_pre = snapshot(&mut agcli_client).await;

    let agcli_wallet_dir = unique_tmp_dir("agcli");
    let agcli_wallet_dir_str = agcli_wallet_dir.to_string_lossy().to_string();
    let agcli_env = vec![
        ("AGCLI_PASSWORD".into(), "pass1234".into()),
        ("AGCLI_MNEMONIC".into(), TEST_MNEMONIC.into()),
    ];

    let agcli_create = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "--dry-run".into(),
            "wallet".into(),
            "create".into(),
            "--name".into(),
            "walletag".into(),
            "--hotkey-name".into(),
            "hka".into(),
            "--no-mnemonic".into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet create --batch --output json --dry-run", &agcli_create);
    assert!(
        extract_json(&agcli_create.stdout).is_some(),
        "agcli create missing JSON payload: {}",
        agcli_create.stdout
    );

    let agcli_list = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "list".into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet list", &agcli_list);

    let agcli_show = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "show".into(),
            "--all".into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet show --all", &agcli_show);

    let agcli_regen_hotkey = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "regen-hotkey".into(),
            "--name".into(),
            "regenhka".into(),
            "--mnemonic".into(),
            TEST_MNEMONIC.into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet regen-hotkey", &agcli_regen_hotkey);

    let agcli_derive = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "derive".into(),
            "--input".into(),
            TEST_MNEMONIC.into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet derive", &agcli_derive);

    let agcli_dev_key = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "dev-key".into(),
            "--uri".into(),
            "Bob".into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet dev-key", &agcli_dev_key);

    let agcli_show_mnemonic = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "show-mnemonic".into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet show-mnemonic", &agcli_show_mnemonic);

    let agcli_bad_derive = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "derive".into(),
            "--input".into(),
            "0xdeadbeef".into(),
        ],
        &agcli_env,
    );
    assert_structured_json_error("agcli wallet derive invalid input", &agcli_bad_derive);

    let agcli_bad_mnemonic = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "walletag".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "show-mnemonic".into(),
        ],
        &[("AGCLI_PASSWORD".into(), "wrong-password".into())],
    );
    assert_structured_json_error(
        "agcli wallet show-mnemonic wrong password",
        &agcli_bad_mnemonic,
    );

    wait_blocks(&mut agcli_client, 2).await;
    let agcli_post = snapshot(&mut agcli_client).await;

    let btcli_balance_delta = btcli_pre
        .alice_balance_rao
        .saturating_sub(btcli_post.alice_balance_rao);
    let agcli_balance_delta = agcli_pre
        .alice_balance_rao
        .saturating_sub(agcli_post.alice_balance_rao);

    assert_eq!(
        btcli_balance_delta, agcli_balance_delta,
        "wallet filesystem workflows should not change chain balance"
    );
    assert_eq!(
        btcli_post.alice_swap, agcli_post.alice_swap,
        "wallet filesystem workflows should not alter swap announcements"
    );

    assert!(btcli_post.block >= btcli_pre.block, "btcli scenario block regressed");
    assert!(agcli_post.block >= agcli_pre.block, "agcli scenario block regressed");

    let _ = std::fs::remove_dir_all(btcli_wallet_dir);
    let _ = std::fs::remove_dir_all(agcli_wallet_dir);
    stop_wallet_containers();
}

#[tokio::test]
async fn wallet_associate_hotkey_and_check_swap_equivalence() {
    let _guard = TEST_LOCK.lock().expect("lock wallet parity tests");

    start_wallet_container(BTCLI_CONTAINER);
    let mut btcli_client = harness::wait_for_chain().await;
    let btcli_pre = snapshot(&mut btcli_client).await;

    let btcli_wallet_dir = unique_tmp_dir("assoc-btcli");
    let btcli_wallet_dir_str = btcli_wallet_dir.to_string_lossy().to_string();

    let btcli_create = run_btcli(
        &[
            "wallet".into(),
            "create".into(),
            "--wallet-name".into(),
            "alice".into(),
            "--wallet-path".into(),
            btcli_wallet_dir_str.clone(),
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
    assert_success("btcli wallet create (associate test)", &btcli_create);

    let btcli_associate = run_btcli(
        &[
            "wallet".into(),
            "associate-hotkey".into(),
            "--wallet-name".into(),
            "alice".into(),
            "--wallet-path".into(),
            btcli_wallet_dir_str.clone(),
            "--hotkey".into(),
            "default".into(),
            "--network".into(),
            "local".into(),
            "--no-prompt".into(),
        ],
        &[],
    );
    assert_success("btcli wallet associate-hotkey", &btcli_associate);

    let btcli_check_swap = run_btcli(
        &[
            "wallet".into(),
            "swap-check".into(),
            "--wallet-name".into(),
            "alice".into(),
            "--wallet-path".into(),
            btcli_wallet_dir_str.clone(),
            "--network".into(),
            "local".into(),
            "--json-output".into(),
        ],
        &[],
    );
    assert_success("btcli wallet swap-check", &btcli_check_swap);

    let sdk_swap = run_sdk_python(
        r#"
import json
from bittensor.core.subtensor import Subtensor

sub = Subtensor(network="local")
print(json.dumps({
    "announcement": sub.get_coldkey_swap_announcement("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
    "owned_hotkeys": sub.get_owned_hotkeys("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY")
}))
"#,
        &[],
    );
    assert_success("sdk get_coldkey_swap_announcement/get_owned_hotkeys", &sdk_swap);
    assert!(
        extract_json(&sdk_swap.stdout).is_some(),
        "sdk swap/owned_hotkeys output missing JSON: {}",
        sdk_swap.stdout
    );

    wait_blocks(&mut btcli_client, 2).await;
    let btcli_post = snapshot(&mut btcli_client).await;
    stop_wallet_containers();

    start_wallet_container(AGCLI_CONTAINER);
    let mut agcli_client = harness::wait_for_chain().await;
    let agcli_pre = snapshot(&mut agcli_client).await;

    let agcli_wallet_dir = unique_tmp_dir("assoc-agcli");
    let agcli_wallet_dir_str = agcli_wallet_dir.to_string_lossy().to_string();
    let agcli_env = vec![("AGCLI_PASSWORD".into(), "pass1234".into())];

    let agcli_dev_wallet = run_agcli(
        &[
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "alice".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "dev-key".into(),
            "--uri".into(),
            "Alice".into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet dev-key (associate test)", &agcli_dev_wallet);

    let agcli_associate = run_agcli(
        &[
            "--network".into(),
            "local".into(),
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "alice".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "associate-hotkey".into(),
            "--hotkey-address".into(),
            ALICE_SS58.into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet associate-hotkey", &agcli_associate);

    let agcli_check_swap = run_agcli(
        &[
            "--network".into(),
            "local".into(),
            "--wallet-dir".into(),
            agcli_wallet_dir_str.clone(),
            "--wallet".into(),
            "alice".into(),
            "--batch".into(),
            "--output".into(),
            "json".into(),
            "wallet".into(),
            "check-swap".into(),
        ],
        &agcli_env,
    );
    assert_success("agcli wallet check-swap", &agcli_check_swap);
    assert!(
        extract_json(&agcli_check_swap.stdout).is_some(),
        "agcli check-swap output missing JSON: {}",
        agcli_check_swap.stdout
    );

    wait_blocks(&mut agcli_client, 2).await;
    let agcli_post = snapshot(&mut agcli_client).await;

    let btcli_balance_delta = btcli_pre
        .alice_balance_rao
        .saturating_sub(btcli_post.alice_balance_rao);
    let agcli_balance_delta = agcli_pre
        .alice_balance_rao
        .saturating_sub(agcli_post.alice_balance_rao);

    assert_eq!(
        btcli_post.alice_swap, agcli_post.alice_swap,
        "check-swap parity mismatch between btcli and agcli scenarios"
    );
    assert!(
        btcli_balance_delta < agcli_balance_delta,
        "associate-hotkey divergence expected: btcli delta={} agcli delta={}",
        btcli_balance_delta,
        agcli_balance_delta
    );
    assert!(btcli_post.block >= btcli_pre.block, "btcli scenario block regressed");
    assert!(agcli_post.block >= agcli_pre.block, "agcli scenario block regressed");

    let _ = std::fs::remove_dir_all(btcli_wallet_dir);
    let _ = std::fs::remove_dir_all(agcli_wallet_dir);
    stop_wallet_containers();
}
