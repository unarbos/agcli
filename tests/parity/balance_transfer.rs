#![cfg(feature = "e2e")]

use std::path::Path;

use serde_json::Value;

use super::harness::*;

const SCENARIO_VARIANT: &str = "A";
const CONTAINER_BTCLI: &str = "agcli_parity_balance_transfer_btcli";
const CONTAINER_AGCLI: &str = "agcli_parity_balance_transfer_agcli";
const CONTAINER_SDK: &str = "agcli_parity_balance_transfer_sdk";
const CONTAINER_UX: &str = "agcli_parity_balance_transfer_ux";
const AGCLI_WALLET_DIR: &str = "/tmp/agcli-parity-agcli-wallets";
const BTCLI_WALLET_DIR: &str = "/tmp/agcli-parity-btcli-wallets";
const AGCLI_PASSWORD: &str = "pass123";

#[derive(Clone, Copy, Debug)]
struct Snapshot {
    alice: u64,
    bob: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Delta {
    alice: i128,
    bob: i128,
}

#[derive(Clone, Copy, Debug)]
enum Scenario {
    TransferAllowDeath,
    TransferAllAllowDeath,
    TransferKeepAlive,
}

impl Scenario {
    fn name(self) -> &'static str {
        match self {
            Self::TransferAllowDeath => "transfer_allow_death",
            Self::TransferAllAllowDeath => "transfer_all_allow_death",
            Self::TransferKeepAlive => "transfer_keep_alive",
        }
    }
}

impl Snapshot {
    fn delta_to(self, post: Self) -> Delta {
        Delta {
            alice: post.alice as i128 - self.alice as i128,
            bob: post.bob as i128 - self.bob as i128,
        }
    }
}

fn docker_available() -> bool {
    Command::new("docker")
        .args(["version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn cleanup_chain(container_name: &str) {
    let _ = Command::new("docker")
        .args(["rm", "-f", container_name])
        .output();
    let _ = Command::new("bash")
        .args([
            "-lc",
            "docker ps -q --filter publish=9944 | xargs -r docker rm -f",
        ])
        .output();
}

async fn boot_chain(container_name: &str) -> Client {
    cleanup_chain(container_name);
    std::thread::sleep(Duration::from_secs(1));

    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-d",
            "--name",
            container_name,
            "-p",
            "9944:9944",
            "-p",
            "9945:9945",
            DOCKER_IMAGE,
        ])
        .output()
        .expect("failed to start localnet container");
    assert!(
        output.status.success(),
        "docker run failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let mut client = wait_for_chain().await;
    ensure_alive(&mut client).await;
    client
}

async fn snapshot_balances(client: &mut Client) -> Snapshot {
    ensure_alive(client).await;
    let alice = client
        .get_balance_ss58(ALICE_SS58)
        .await
        .expect("Alice balance")
        .rao();
    let bob = client
        .get_balance_ss58(BOB_SS58)
        .await
        .expect("Bob balance")
        .rao();
    Snapshot { alice, bob }
}

fn shell_quote(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', "'\"'\"'"))
}

fn output_text(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn require_success(output: &std::process::Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed (exit={:?})\n{}",
        output.status.code(),
        output_text(output)
    );
}

fn parse_last_json(output: &std::process::Output) -> Value {
    let text = output_text(output);
    for line in text.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
                return value;
            }
        }
    }
    panic!("no JSON object found in output:\n{text}");
}

fn agcli_bin() -> String {
    std::env::var("CARGO_BIN_EXE_agcli").unwrap_or_else(|_| "target/debug/agcli".to_string())
}

fn run_agcli(args: &[&str], extra_env: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(agcli_bin());
    cmd.args(args);
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    cmd.output().expect("failed to run agcli")
}

fn run_btcli(args: &[&str]) -> std::process::Output {
    let mut cmdline = String::from("source .venv/bin/activate && btcli");
    for arg in args {
        cmdline.push(' ');
        cmdline.push_str(&shell_quote(arg));
    }
    Command::new("bash")
        .args(["-lc", &cmdline])
        .output()
        .expect("failed to run btcli")
}

fn setup_agcli_wallets() {
    let _ = Command::new("bash")
        .args([
            "-lc",
            &format!(
                "rm -rf {} && mkdir -p {}",
                AGCLI_WALLET_DIR, AGCLI_WALLET_DIR
            ),
        ])
        .output();

    let alice = run_agcli(
        &[
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "alice",
            "--yes",
            "--batch",
            "--output",
            "json",
            "wallet",
            "dev-key",
            "--uri",
            "Alice",
        ],
        &[("AGCLI_PASSWORD", AGCLI_PASSWORD)],
    );
    require_success(&alice, "agcli wallet dev-key Alice");

    let bob = run_agcli(
        &[
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "bob",
            "--yes",
            "--batch",
            "--output",
            "json",
            "wallet",
            "dev-key",
            "--uri",
            "Bob",
        ],
        &[("AGCLI_PASSWORD", AGCLI_PASSWORD)],
    );
    require_success(&bob, "agcli wallet dev-key Bob");
}

fn setup_btcli_wallets() {
    let _ = Command::new("bash")
        .args([
            "-lc",
            &format!(
                "rm -rf {} && mkdir -p {}",
                BTCLI_WALLET_DIR, BTCLI_WALLET_DIR
            ),
        ])
        .output();

    let alice = run_btcli(&[
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
    ]);
    require_success(&alice, "btcli wallet create Alice");

    let bob = run_btcli(&[
        "wallet",
        "create",
        "--wallet-name",
        "bob",
        "--wallet-path",
        BTCLI_WALLET_DIR,
        "--hotkey",
        "default",
        "--uri",
        "Bob",
        "--no-use-password",
        "--overwrite",
        "--json-output",
    ]);
    require_success(&bob, "btcli wallet create Bob");
}

async fn run_btcli_scenario(scenario: Scenario) -> Delta {
    let mut client = boot_chain(CONTAINER_BTCLI).await;
    setup_btcli_wallets();
    let before = snapshot_balances(&mut client).await;

    let output = match scenario {
        Scenario::TransferAllowDeath => run_btcli(&[
            "wallet",
            "transfer",
            "--wallet-name",
            "alice",
            "--wallet-path",
            BTCLI_WALLET_DIR,
            "--network",
            LOCAL_WS,
            "--destination",
            BOB_SS58,
            "--amount",
            "1.0",
            "--allow-death",
            "--no-prompt",
            "--json-output",
        ]),
        Scenario::TransferAllAllowDeath => run_btcli(&[
            "wallet",
            "transfer",
            "--wallet-name",
            "bob",
            "--wallet-path",
            BTCLI_WALLET_DIR,
            "--network",
            LOCAL_WS,
            "--destination",
            ALICE_SS58,
            "--all",
            "--allow-death",
            "--no-prompt",
            "--json-output",
        ]),
        Scenario::TransferKeepAlive => run_btcli(&[
            "wallet",
            "transfer",
            "--wallet-name",
            "alice",
            "--wallet-path",
            BTCLI_WALLET_DIR,
            "--network",
            LOCAL_WS,
            "--destination",
            BOB_SS58,
            "--amount",
            "0.5",
            "--no-prompt",
            "--json-output",
        ]),
    };
    require_success(&output, &format!("btcli {}", scenario.name()));
    let json = parse_last_json(&output);
    assert_eq!(json.get("success"), Some(&Value::Bool(true)));

    wait_blocks(&mut client, 12).await;
    let after = snapshot_balances(&mut client).await;
    cleanup_chain(CONTAINER_BTCLI);
    before.delta_to(after)
}

async fn run_agcli_scenario(scenario: Scenario) -> Delta {
    let mut client = boot_chain(CONTAINER_AGCLI).await;
    setup_agcli_wallets();
    let before = snapshot_balances(&mut client).await;

    let args = match scenario {
        Scenario::TransferAllowDeath => vec![
            "--endpoint",
            LOCAL_WS,
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "alice",
            "--yes",
            "--batch",
            "--output",
            "json",
            "transfer",
            "--dest",
            BOB_SS58,
            "--amount",
            "1.0",
        ],
        Scenario::TransferAllAllowDeath => vec![
            "--endpoint",
            LOCAL_WS,
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "bob",
            "--yes",
            "--batch",
            "--output",
            "json",
            "transfer-all",
            "--dest",
            ALICE_SS58,
        ],
        Scenario::TransferKeepAlive => vec![
            "--endpoint",
            LOCAL_WS,
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "alice",
            "--yes",
            "--batch",
            "--output",
            "json",
            "transfer-keep-alive",
            "--dest",
            BOB_SS58,
            "--amount",
            "0.5",
        ],
    };
    let output = run_agcli(&args, &[("AGCLI_PASSWORD", AGCLI_PASSWORD)]);
    require_success(&output, &format!("agcli {}", scenario.name()));
    let json = parse_last_json(&output);
    assert!(json.get("tx_hash").and_then(Value::as_str).is_some());

    wait_blocks(&mut client, 4).await;
    let after = snapshot_balances(&mut client).await;

    let alice_balance = run_agcli(
        &[
            "--endpoint",
            LOCAL_WS,
            "--output",
            "json",
            "balance",
            "--address",
            ALICE_SS58,
        ],
        &[],
    );
    require_success(&alice_balance, "agcli balance Alice");
    let alice_json = parse_last_json(&alice_balance);
    assert_eq!(
        alice_json.get("balance_rao").and_then(Value::as_u64),
        Some(after.alice)
    );

    let bob_balance = run_agcli(
        &[
            "--endpoint",
            LOCAL_WS,
            "--output",
            "json",
            "balance",
            "--address",
            BOB_SS58,
        ],
        &[],
    );
    require_success(&bob_balance, "agcli balance Bob");
    let bob_json = parse_last_json(&bob_balance);
    assert_eq!(
        bob_json.get("balance_rao").and_then(Value::as_u64),
        Some(after.bob)
    );

    cleanup_chain(CONTAINER_AGCLI);
    before.delta_to(after)
}

async fn run_sdk_scenario(scenario: Scenario) -> Delta {
    let mut client = boot_chain(CONTAINER_SDK).await;
    let before = snapshot_balances(&mut client).await;
    let alice = dev_pair(ALICE_URI);
    let bob = dev_pair(BOB_URI);

    match scenario {
        Scenario::TransferAllowDeath => {
            let _ = client
                .transfer(&alice, BOB_SS58, Balance::from_tao(1.0))
                .await
                .expect("sdk transfer");
        }
        Scenario::TransferAllAllowDeath => {
            let _ = client
                .transfer_all(&bob, ALICE_SS58, false)
                .await
                .expect("sdk transfer_all");
        }
        Scenario::TransferKeepAlive => {
            let _ = client
                .transfer_keep_alive(&alice, BOB_SS58, Balance::from_tao(0.5))
                .await
                .expect("sdk transfer_keep_alive");
        }
    }

    wait_blocks(&mut client, 4).await;
    let after = snapshot_balances(&mut client).await;
    cleanup_chain(CONTAINER_SDK);
    before.delta_to(after)
}

async fn assert_agcli_ux_contracts() {
    let mut client = boot_chain(CONTAINER_UX).await;
    setup_agcli_wallets();
    ensure_alive(&mut client).await;

    for args in [
        vec![
            "--dry-run",
            "--batch",
            "--output",
            "json",
            "--endpoint",
            LOCAL_WS,
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "alice",
            "--yes",
            "transfer",
            "--dest",
            BOB_SS58,
            "--amount",
            "1.0",
        ],
        vec![
            "--dry-run",
            "--batch",
            "--output",
            "json",
            "--endpoint",
            LOCAL_WS,
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "bob",
            "--yes",
            "transfer-all",
            "--dest",
            ALICE_SS58,
        ],
        vec![
            "--dry-run",
            "--batch",
            "--output",
            "json",
            "--endpoint",
            LOCAL_WS,
            "--wallet-dir",
            AGCLI_WALLET_DIR,
            "--wallet",
            "alice",
            "--yes",
            "transfer-keep-alive",
            "--dest",
            BOB_SS58,
            "--amount",
            "0.5",
        ],
    ] {
        let output = run_agcli(&args, &[("AGCLI_PASSWORD", AGCLI_PASSWORD)]);
        require_success(&output, "agcli dry-run transfer command");
    }

    let invalid = run_agcli(
        &[
            "--output", "json", "--batch", "transfer", "--dest", "bad_ss58", "--amount", "1.0",
        ],
        &[],
    );
    assert_eq!(
        invalid.status.code(),
        Some(12),
        "expected validation exit code 12, got {:?}\n{}",
        invalid.status.code(),
        output_text(&invalid)
    );
    let invalid_json = parse_last_json(&invalid);
    assert_eq!(invalid_json.get("error"), Some(&Value::Bool(true)));
    assert_eq!(invalid_json.get("code").and_then(Value::as_i64), Some(12));

    cleanup_chain(CONTAINER_UX);
}

#[tokio::test]
async fn transfer_and_balance_parity_btcli_sdk_vs_agcli() {
    if !docker_available() {
        eprintln!("[parity/balance_transfer] Docker unavailable, skipping.");
        return;
    }
    assert_eq!(SCENARIO_VARIANT, "A");
    assert!(
        Path::new(".venv/bin/btcli").exists(),
        "btcli not installed at .venv/bin/btcli"
    );

    for scenario in [
        Scenario::TransferAllowDeath,
        Scenario::TransferAllAllowDeath,
        Scenario::TransferKeepAlive,
    ] {
        let btcli_delta = run_btcli_scenario(scenario).await;
        let agcli_delta = run_agcli_scenario(scenario).await;
        assert_eq!(
            btcli_delta,
            agcli_delta,
            "btcli and agcli deltas differ for {}",
            scenario.name()
        );

        let sdk_delta = run_sdk_scenario(scenario).await;
        assert_eq!(
            sdk_delta,
            agcli_delta,
            "sdk and agcli deltas differ for {}",
            scenario.name()
        );
    }

    assert_agcli_ux_contracts().await;
    cleanup_chain(CONTAINER_BTCLI);
    cleanup_chain(CONTAINER_AGCLI);
    cleanup_chain(CONTAINER_SDK);
    cleanup_chain(CONTAINER_UX);
}
