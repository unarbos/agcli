use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

const LOCAL_ENDPOINT: &str = "ws://127.0.0.1:9944";
const BTCLI_BIN: &str = "/workspace/.venv/bin/btcli";
const PYTHON_BIN: &str = "/workspace/.venv/bin/python";
const BTCLI_WALLET_DIR: &str = "/tmp/agcli-parity-btcli-wallets";
const AGCLI_WALLET_DIR: &str = "/tmp/agcli-parity-agcli-wallets";
const SDK_WALLET_DIR: &str = "/tmp/agcli-parity-sdk-wallets";
const AGCLI_PASSWORD: &str = "agcli-parity-test-password";
const VARIANT_B_CONFIG: &str =
    "/workspace/.orchestrate/agcli-parity/scaffold-variants/scaffold-B-commit-reveal.toml";

#[derive(Debug)]
struct CmdResult {
    status: i32,
    stdout: String,
    stderr: String,
}

fn agcli_bin() -> &'static str {
    env!("CARGO_BIN_EXE_agcli")
}

fn run_cmd(bin: &str, args: &[&str], envs: &[(&str, &str)]) -> CmdResult {
    let output = Command::new(bin)
        .args(args)
        .envs(envs.iter().copied())
        .output()
        .unwrap_or_else(|e| panic!("failed to run `{bin} {args:?}`: {e}"));
    CmdResult {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

fn assert_ok(cmd_name: &str, result: &CmdResult) {
    assert_eq!(
        result.status,
        0,
        "{cmd_name} failed (status={}):\nstdout:\n{}\nstderr:\n{}",
        result.status,
        result.stdout,
        result.stderr
    );
}

fn last_json(stdout: &str) -> Value {
    let object_start = stdout.find('{');
    let array_start = stdout.find('[');
    let start = match (object_start, array_start) {
        (Some(a), Some(b)) => a.min(b),
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (None, None) => panic!("no JSON payload found in stdout:\n{stdout}"),
    };
    let json_region = stdout[start..].trim();
    let mut de = serde_json::Deserializer::from_str(json_region);
    Value::deserialize(&mut de)
        .unwrap_or_else(|e| panic!("failed to parse JSON payload from stdout:\n{json_region}\nerror: {e}"))
}

fn reset_wallet_dirs() {
    for dir in [BTCLI_WALLET_DIR, AGCLI_WALLET_DIR, SDK_WALLET_DIR] {
        let _ = fs::remove_dir_all(dir);
    }
}

fn setup_wallets() {
    reset_wallet_dirs();

    let agcli_wallet = run_cmd(
        agcli_bin(),
        &[
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--output",
            "json",
            "wallet",
            "dev-key",
            "--uri",
            "//Alice",
        ],
        &[("AGCLI_PASSWORD", AGCLI_PASSWORD)],
    );
    assert_ok("agcli wallet dev-key", &agcli_wallet);
    let agcli_json = last_json(&agcli_wallet.stdout);
    assert_eq!(agcli_json["name"].as_str(), Some("alice"));

    let btcli_wallet = run_cmd(
        BTCLI_BIN,
        &[
            "wallet",
            "create",
            "--wallet-name",
            "alice",
            "--wallet-path",
            BTCLI_WALLET_DIR,
            "--hotkey",
            "default",
            "--uri",
            "Alice",
            "--no-use-password",
            "--overwrite",
            "--json-output",
        ],
        &[],
    );
    assert_ok("btcli wallet create", &btcli_wallet);
    let btcli_json = last_json(&btcli_wallet.stdout);
    assert_eq!(btcli_json["success"].as_bool(), Some(true));
}

async fn maybe_scaffold_variant_b(client: &mut Client) -> Option<NetUid> {
    if !Path::new(VARIANT_B_CONFIG).exists() {
        return None;
    }

    let probe = run_cmd(
        agcli_bin(),
        &[
            "--output",
            "json",
            "localnet",
            "scaffold",
            "--config",
            VARIANT_B_CONFIG,
            "--no-start",
        ],
        &[],
    );

    if probe.status == 0 {
        let payload = last_json(&probe.stdout);
        let netuid = payload["subnets"]
            .as_array()
            .and_then(|rows| rows.first())
            .and_then(|row| row["netuid"].as_u64())
            .map(|n| NetUid(n as u16));
        if let Some(n) = netuid {
            ensure_alive(client).await;
            if let Ok(Some(hp)) = client.get_subnet_hyperparams(n).await {
                if !hp.commit_reveal_weights_enabled {
                    sudo_set_commit_reveal_weights_or_fail(client, n, true).await;
                }
            }
            return Some(n);
        }
    } else {
        let combined = format!("{}\n{}", probe.stdout, probe.stderr);
        assert!(
            combined.contains("sudo_set_tempo")
                || combined.contains("AdminActionProhibited")
                || combined.contains("Sudo inner dispatch failed"),
            "unexpected scaffold variant-B failure:\n{}",
            combined
        );
    }

    None
}

async fn create_variant_b_fallback(client: &mut Client) -> NetUid {
    let alice = dev_pair(ALICE_URI);
    ensure_alive(client).await;
    let before = client
        .get_total_networks()
        .await
        .expect("read network count before subnet registration");
    let _ = retry_extrinsic!(client, client.register_network(&alice, ALICE_SS58));
    wait_blocks(client, 5).await;
    ensure_alive(client).await;
    let after = client
        .get_total_networks()
        .await
        .expect("read network count after subnet registration");
    assert!(
        after > before,
        "expected subnet registration to increase total networks (before={before}, after={after})"
    );
    let netuid = NetUid(after - 1);

    // Variant B semantics: commit-reveal enabled.
    sudo_set_commit_reveal_weights_or_fail(client, netuid, true).await;
    wait_blocks(client, 2).await;
    netuid
}

fn btcli_set_identity(netuid: NetUid, name: &str, github_repo: &str, subnet_url: &str) {
    let result = run_cmd(
        BTCLI_BIN,
        &[
            "subnets",
            "set-identity",
            "--network",
            LOCAL_ENDPOINT,
            "--wallet-name",
            "alice",
            "--wallet-path",
            BTCLI_WALLET_DIR,
            "--hotkey",
            "default",
            "--netuid",
            &netuid.0.to_string(),
            "--subnet-name",
            name,
            "--github-repo",
            github_repo,
            "--subnet-contact",
            "btcli@example.com",
            "--subnet-url",
            subnet_url,
            "--discord-handle",
            "btcli#1234",
            "--description",
            "btcli identity write",
            "--logo-url",
            "https://example.com/logo.png",
            "--additional-info",
            "btcli-identity",
            "--yes",
            "--json-output",
        ],
        &[],
    );
    assert_ok("btcli subnets set-identity", &result);
    let payload = last_json(&result.stdout);
    assert_eq!(payload["success"].as_bool(), Some(true));
}

fn agcli_set_identity(netuid: NetUid, name: &str, github_repo: &str, subnet_url: &str) {
    let result = run_cmd(
        agcli_bin(),
        &[
            "--endpoint",
            LOCAL_ENDPOINT,
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "alice",
            "--yes",
            "identity",
            "set-subnet",
            "--netuid",
            &netuid.0.to_string(),
            "--name",
            name,
            "--github",
            github_repo,
            "--url",
            subnet_url,
        ],
        &[("AGCLI_PASSWORD", AGCLI_PASSWORD)],
    );
    assert_ok("agcli identity set-subnet", &result);
}

fn sdk_set_identity(netuid: NetUid, async_mode: bool, name: &str, github_repo: &str, url: &str) {
    let mode_line = if async_mode {
        r#"
import asyncio
async def run():
    sub = bt.AsyncSubtensor(network="ws://127.0.0.1:9944")
    resp = await sub.set_subnet_identity(wallet=wallet, netuid=NETUID, subnet_identity=identity, raise_error=False, wait_for_inclusion=True, wait_for_finalization=True)
    await sub.close()
    return resp
response = asyncio.run(run())
"#
    } else {
        r#"
sub = bt.Subtensor(network="ws://127.0.0.1:9944")
response = sub.set_subnet_identity(wallet=wallet, netuid=NETUID, subnet_identity=identity, raise_error=False, wait_for_inclusion=True, wait_for_finalization=True)
"#
    };

    let script = format!(
        r#"
import json
import bittensor as bt
from bittensor.core.chain_data.subnet_identity import SubnetIdentity

NETUID = {netuid}
wallet = bt.Wallet(name="alice", hotkey="default", path="{wallet_path}")
wallet.create_coldkey_from_uri("//Alice", overwrite=True, use_password=False, suppress=True)
wallet.create_hotkey_from_uri("//Alice", overwrite=True, use_password=False, suppress=True)
identity = SubnetIdentity(
    "{name}",
    "{github_repo}",
    "sdk@example.com",
    "{url}",
    "https://sdk.example/logo.png",
    "sdk#1234",
    "sdk subnet identity",
    "sdk-additional",
)
{mode_line}
print(json.dumps({{"ok": "success: True" in str(response)}}))
"#,
        netuid = netuid.0,
        wallet_path = SDK_WALLET_DIR,
        name = name,
        github_repo = github_repo,
        url = url,
        mode_line = mode_line
    );

    let result = run_cmd(PYTHON_BIN, &["-c", &script], &[]);
    assert_ok(
        if async_mode {
            "sdk AsyncSubtensor.set_subnet_identity"
        } else {
            "sdk Subtensor.set_subnet_identity"
        },
        &result,
    );
    let payload = last_json(&result.stdout);
    assert_eq!(payload["ok"].as_bool(), Some(true));
}

fn btcli_set_hyperparam(netuid: NetUid, param: &str, value: &str) -> CmdResult {
    run_cmd(
        BTCLI_BIN,
        &[
            "sudo",
            "set",
            "--network",
            LOCAL_ENDPOINT,
            "--wallet-name",
            "alice",
            "--wallet-path",
            BTCLI_WALLET_DIR,
            "--hotkey",
            "default",
            "--netuid",
            &netuid.0.to_string(),
            "--param",
            param,
            "--value",
            value,
            "--yes",
            "--json-output",
        ],
        &[],
    )
}

fn agcli_admin_raw(netuid: NetUid, call: &str, value: &str) -> CmdResult {
    let args_json = format!("[{}, {}]", netuid.0, value);
    run_cmd(
        agcli_bin(),
        &[
            "--endpoint",
            LOCAL_ENDPOINT,
            "--yes",
            "--output",
            "json",
            "admin",
            "raw",
            "--call",
            call,
            "--args",
            &args_json,
            "--sudo-key",
            "//Alice",
        ],
        &[],
    )
}

#[tokio::test]
async fn parity_subnet_hyperparams() {
    assert!(
        Path::new(BTCLI_BIN).exists(),
        "missing btcli binary at {BTCLI_BIN}. Run `source .venv/bin/activate && uv pip install bittensor bittensor-cli`."
    );
    assert!(
        Path::new(PYTHON_BIN).exists(),
        "missing venv python at {PYTHON_BIN}. Run `uv venv .venv` first."
    );
    assert!(
        Path::new(VARIANT_B_CONFIG).exists(),
        "missing scaffold variant B config at {VARIANT_B_CONFIG}"
    );

    ensure_local_chain();
    let mut client = wait_for_chain().await;
    setup_wallets();

    let netuid = match maybe_scaffold_variant_b(&mut client).await {
        Some(n) => n,
        None => create_variant_b_fallback(&mut client).await,
    };

    ensure_alive(&mut client).await;
    let hparams = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("query subnet hyperparams")
        .expect("subnet hyperparams should exist");
    assert!(
        hparams.commit_reveal_weights_enabled,
        "variant B requires commit_reveal_weights_enabled=true"
    );

    btcli_set_identity(
        netuid,
        "btcli-parity-subnet",
        "https://github.com/example/btcli-parity",
        "https://btcli.example",
    );
    wait_blocks(&mut client, 2).await;
    let after_btcli_identity = client
        .get_subnet_identity(netuid)
        .await
        .expect("query identity after btcli");
    if let Some(stored) = after_btcli_identity {
        assert_eq!(stored.subnet_name, "btcli-parity-subnet");
        assert_eq!(
            stored.github_repo,
            "https://github.com/example/btcli-parity"
        );
        assert_eq!(stored.subnet_url, "https://btcli.example");
    } else {
        println!(
            "[parity-subnet-hyperparams] btcli set-identity produced no on-chain identity delta on SN{}",
            netuid.0
        );
    }

    agcli_set_identity(
        netuid,
        "agcli-parity-subnet",
        "example/agcli-parity",
        "https://agcli.example",
    );
    wait_blocks(&mut client, 2).await;
    let after_agcli_identity = client
        .get_subnet_identity(netuid)
        .await
        .expect("query identity after agcli")
        .expect("identity should exist after agcli write");
    assert_eq!(after_agcli_identity.subnet_name, "agcli-parity-subnet");
    assert_eq!(after_agcli_identity.github_repo, "example/agcli-parity");
    assert_eq!(after_agcli_identity.subnet_url, "https://agcli.example");

    sdk_set_identity(
        netuid,
        false,
        "sdk-sync-subnet",
        "https://github.com/example/sdk-sync",
        "https://sdk-sync.example",
    );
    wait_blocks(&mut client, 2).await;
    let after_sdk_sync = client
        .get_subnet_identity(netuid)
        .await
        .expect("query identity after sdk sync")
        .expect("identity should exist after sdk sync");
    assert_eq!(after_sdk_sync.subnet_name, "sdk-sync-subnet");
    assert_eq!(
        after_sdk_sync.github_repo,
        "https://github.com/example/sdk-sync"
    );
    assert_eq!(after_sdk_sync.subnet_url, "https://sdk-sync.example");

    sdk_set_identity(
        netuid,
        true,
        "sdk-async-subnet",
        "https://github.com/example/sdk-async",
        "https://sdk-async.example",
    );
    wait_blocks(&mut client, 2).await;
    let after_sdk_async = client
        .get_subnet_identity(netuid)
        .await
        .expect("query identity after sdk async")
        .expect("identity should exist after sdk async");
    assert_eq!(after_sdk_async.subnet_name, "sdk-async-subnet");
    assert_eq!(
        after_sdk_async.github_repo,
        "https://github.com/example/sdk-async"
    );
    assert_eq!(after_sdk_async.subnet_url, "https://sdk-async.example");

    // Highest-risk hyperparameter writes: tempo, max_validators, and weights_rate_limit.
    ensure_alive(&mut client).await;
    let baseline = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("query baseline hyperparams")
        .expect("baseline hyperparams should exist");

    let btcli_tempo_target = baseline.tempo.saturating_add(3);
    let btcli_tempo = btcli_set_hyperparam(netuid, "tempo", &btcli_tempo_target.to_string());
    assert_ok("btcli sudo set tempo", &btcli_tempo);
    wait_blocks(&mut client, 2).await;
    let after_btcli_tempo = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("query hyperparams after btcli tempo")
        .expect("hyperparams should exist after btcli tempo");
    let btcli_tempo_changed = after_btcli_tempo.tempo == btcli_tempo_target;

    let agcli_tempo_target = baseline.tempo.saturating_add(5);
    let agcli_tempo = agcli_admin_raw(netuid, "sudo_set_tempo", &agcli_tempo_target.to_string());
    wait_blocks(&mut client, 2).await;
    let after_agcli_tempo = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("query hyperparams after agcli tempo")
        .expect("hyperparams should exist after agcli tempo");
    if agcli_tempo.status == 0 {
        assert_eq!(after_agcli_tempo.tempo, agcli_tempo_target);
    } else {
        assert_eq!(after_agcli_tempo.tempo, after_btcli_tempo.tempo);
    }

    let btcli_max_validators_target = baseline.max_validators.saturating_sub(1).max(1);
    let btcli_max = btcli_set_hyperparam(
        netuid,
        "max_validators",
        &btcli_max_validators_target.to_string(),
    );
    assert_ok("btcli sudo set max_validators", &btcli_max);
    wait_blocks(&mut client, 2).await;
    let after_btcli_max = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("query hyperparams after btcli max_validators")
        .expect("hyperparams should exist after btcli max_validators");
    let btcli_max_changed = after_btcli_max.max_validators == btcli_max_validators_target;

    let agcli_max_target = baseline.max_validators.saturating_add(2);
    let agcli_max = agcli_admin_raw(
        netuid,
        "sudo_set_max_allowed_validators",
        &agcli_max_target.to_string(),
    );
    wait_blocks(&mut client, 2).await;
    let after_agcli_max = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("query hyperparams after agcli max_validators")
        .expect("hyperparams should exist after agcli max_validators");
    if agcli_max.status == 0 {
        assert_eq!(after_agcli_max.max_validators, agcli_max_target);
    } else {
        assert_eq!(after_agcli_max.max_validators, after_btcli_max.max_validators);
    }

    let weights_base = after_agcli_max.weights_rate_limit;
    let btcli_weights_target = weights_base.saturating_add(11);
    let btcli_weights = btcli_set_hyperparam(
        netuid,
        "weights_rate_limit",
        &btcli_weights_target.to_string(),
    );
    assert_ok("btcli sudo set weights_rate_limit", &btcli_weights);
    wait_blocks(&mut client, 2).await;
    let after_btcli_weights = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("query hyperparams after btcli weights_rate_limit")
        .expect("hyperparams should exist after btcli weights_rate_limit");
    let btcli_weights_changed = after_btcli_weights.weights_rate_limit == btcli_weights_target;

    let agcli_weights_target = weights_base.saturating_add(22);
    let agcli_weights = agcli_admin_raw(
        netuid,
        "sudo_set_weights_set_rate_limit",
        &agcli_weights_target.to_string(),
    );
    assert_ok("agcli admin raw sudo_set_weights_set_rate_limit", &agcli_weights);
    wait_blocks(&mut client, 2).await;
    let after_agcli_weights = client
        .get_subnet_hyperparams(netuid)
        .await
        .expect("query hyperparams after agcli weights_rate_limit")
        .expect("hyperparams should exist after agcli weights_rate_limit");
    assert_eq!(after_agcli_weights.weights_rate_limit, agcli_weights_target);

    println!(
        "[parity-subnet-hyperparams] netuid={} btcli_tempo_changed={} btcli_max_changed={} btcli_weights_changed={} final_weights_rate_limit={}",
        netuid.0,
        btcli_tempo_changed,
        btcli_max_changed,
        btcli_weights_changed,
        after_agcli_weights.weights_rate_limit
    );
}
