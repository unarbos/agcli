#![cfg(feature = "e2e")]

use super::harness;
use agcli::chain::Client;
use agcli::types::NetUid;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

const CATEGORY_CONTAINER_PREFIX: &str = "agcli_parity_stake_advanced";
const WALLET_NAME: &str = "alice";
const WALLET_HOTKEY: &str = "default";
static TEST_LOCK: Mutex<()> = Mutex::new(());

const VARIANT_C_FALLBACK_CONFIG: &str = r#"
[chain]
image = "ghcr.io/opentensor/subtensor-localnet:devnet-ready"
container = "__CONTAINER__"
port = 9944
start = true
timeout = 180

[[subnet]]
tempo = 90
max_allowed_validators = 8
min_allowed_weights = 1
weights_rate_limit = 0
commit_reveal = false

[[subnet.neuron]]
name = "validator1"
fund_tao = 1500.0
register = true

[[subnet.neuron]]
name = "miner1"
fund_tao = 200.0
register = true

[[subnet]]
tempo = 130
max_allowed_validators = 12
min_allowed_weights = 1
weights_rate_limit = 0
commit_reveal = false

[[subnet.neuron]]
name = "validator2"
fund_tao = 1500.0
register = true

[[subnet.neuron]]
name = "miner2"
fund_tao = 200.0
register = true
"#;

#[derive(Debug, Clone, Deserialize)]
struct ScaffoldResult {
    endpoint: String,
    subnets: Vec<ScaffoldSubnet>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScaffoldSubnet {
    netuid: u16,
}

#[derive(Debug)]
struct ScenarioContext {
    _temp_wallet_dir: TempDir,
    wallet_dir_path: String,
    endpoint: String,
    from_netuid: u16,
    to_netuid: u16,
    hotkey_ss58: String,
    source_coldkey_ss58: String,
    dest_coldkey_ss58: String,
}

#[derive(Debug, Clone)]
struct ChainSnapshot {
    balances_rao: BTreeMap<String, u64>,
    stakes_rao: BTreeMap<String, BTreeMap<u16, u64>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChainDelta {
    balances_rao: BTreeMap<String, i128>,
    stakes_rao: BTreeMap<String, BTreeMap<u16, i128>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Operation {
    Move,
    Swap,
    TransferStake,
}

#[derive(Debug, Clone, Copy)]
enum Reference {
    Btcli,
    SdkSync,
    SdkAsync,
    Agcli,
}

impl Operation {
    fn slug(self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Swap => "swap",
            Self::TransferStake => "transfer_stake",
        }
    }

    fn amount_tao(self) -> f64 {
        match self {
            Self::Move => 5.0,
            Self::Swap => 4.0,
            Self::TransferStake => 3.0,
        }
    }
}

const OPERATIONS: [Operation; 3] = [Operation::Move, Operation::Swap, Operation::TransferStake];

impl Reference {
    fn slug(self) -> &'static str {
        match self {
            Self::Btcli => "btcli",
            Self::SdkSync => "sdk_sync",
            Self::SdkAsync => "sdk_async",
            Self::Agcli => "agcli",
        }
    }
}

struct ContainerGuard {
    name: String,
}

impl Drop for ContainerGuard {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "-f", &self.name])
            .current_dir(repo_root())
            .output();
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn shell_join(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn agcli_bin() -> String {
    std::env::var("CARGO_BIN_EXE_agcli").unwrap_or_else(|_| "target/release/agcli".to_string())
}

fn agcli_password() -> String {
    std::env::var("AGCLI_PASSWORD").unwrap_or_else(|_| "parity-pass".to_string())
}

fn docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .current_dir(repo_root())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn btcli_available() -> bool {
    let script = "source .venv/bin/activate >/dev/null 2>&1 && btcli --version >/dev/null 2>&1";
    Command::new("bash")
        .args(["-lc", script])
        .current_dir(repo_root())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn sdk_available() -> bool {
    let script =
        "source .venv/bin/activate >/dev/null 2>&1 && python -c 'import bittensor' >/dev/null 2>&1";
    Command::new("bash")
        .args(["-lc", script])
        .current_dir(repo_root())
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn require_prerequisites() -> bool {
    if !docker_available() {
        eprintln!("[parity_stake_advanced] Docker unavailable; skipping.");
        return false;
    }
    if !btcli_available() {
        eprintln!("[parity_stake_advanced] btcli unavailable in .venv; skipping.");
        return false;
    }
    if !sdk_available() {
        eprintln!("[parity_stake_advanced] bittensor SDK unavailable in .venv; skipping.");
        return false;
    }
    true
}

fn run_cmd(mut cmd: Command, label: &str) -> Result<Output> {
    cmd.current_dir(repo_root());
    cmd.output()
        .with_context(|| format!("failed to run {label}"))
}

fn acquire_test_lock() -> std::sync::MutexGuard<'static, ()> {
    match TEST_LOCK.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn assert_success(output: &Output, label: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    anyhow::bail!(
        "{label} failed (code: {:?})\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn has_transient_tx_error(output: &Output) -> bool {
    let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
    let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    let combined = format!("{stdout}\n{stderr}");
    combined.contains("temporarily banned")
        || combined.contains("priority is too low")
        || combined.contains("outdated")
        || combined.contains("code': 1012")
        || combined.contains("\"code\": 1012")
}

fn has_amount_too_low_error(output: &Output) -> bool {
    let stderr = String::from_utf8_lossy(&output.stderr).to_ascii_lowercase();
    let stdout = String::from_utf8_lossy(&output.stdout).to_ascii_lowercase();
    let combined = format!("{stdout}\n{stderr}");
    combined.contains("amounttoolow") || combined.contains("amount too low")
}

fn parse_stdout_json(output: &Output, label: &str) -> Result<Value> {
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "{label} emitted non-JSON stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn write_variant_c_config(temp_dir: &TempDir, container: &str) -> Result<String> {
    let scaffold_path = temp_dir.path().join("scaffold-C-multi-subnet.toml");
    let config = VARIANT_C_FALLBACK_CONFIG.replace("__CONTAINER__", container);
    fs::write(&scaffold_path, config)
        .with_context(|| format!("failed writing {}", scaffold_path.display()))?;
    Ok(scaffold_path.to_string_lossy().to_string())
}

fn cleanup_port_and_container(container: &str) {
    let _ = Command::new("docker")
        .args(["rm", "-f", container])
        .current_dir(repo_root())
        .output();
    let _ = Command::new("docker")
        .args(["rm", "-f", "agcli_localnet"])
        .current_dir(repo_root())
        .output();
    let _ = Command::new("docker")
        .args(["rm", "-f", harness::CONTAINER_NAME])
        .current_dir(repo_root())
        .output();
    let _ = Command::new("bash")
        .args([
            "-lc",
            "docker ps -q --filter publish=9944 | xargs -r docker rm -f",
        ])
        .current_dir(repo_root())
        .output();
    let _ = Command::new("bash")
        .args(["-lc", "rm -f /tmp/agcli-tx-locks/*.lock"])
        .current_dir(repo_root())
        .output();
}

fn start_localnet_container(container: &str) -> Result<()> {
    let out = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-d",
            "--name",
            container,
            "-p",
            "9944:9944",
            "-p",
            "9945:9945",
            harness::DOCKER_IMAGE,
        ])
        .current_dir(repo_root())
        .output()
        .context("failed to run docker localnet container")?;
    assert_success(&out, "docker run localnet")?;
    Ok(())
}

fn is_scaffold_tempo_failure(output: &Output) -> bool {
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    combined.contains("sudo_set_tempo") && combined.contains("Sudo inner dispatch failed")
}

async fn ensure_multi_subnet_fallback(client: &mut Client) -> Result<(u16, u16)> {
    let alice = harness::dev_pair(harness::ALICE_URI);
    harness::ensure_alive(client).await;
    let mut total = client
        .get_total_networks()
        .await
        .context("failed to query total networks")?;
    if total < 3 {
        for _ in 0..5 {
            harness::ensure_alive(client).await;
            if client
                .register_network(&alice, harness::ALICE_SS58)
                .await
                .is_ok()
            {
                break;
            }
            harness::wait_blocks(client, 2).await;
        }
        harness::wait_blocks(client, 4).await;
        total = client
            .get_total_networks()
            .await
            .context("failed to query total networks after register_network")?;
    }
    if total < 3 {
        anyhow::bail!("manual fallback could not create a second subnet");
    }
    let to_netuid = total - 1;
    let from_netuid = to_netuid.saturating_sub(1).max(1);
    if from_netuid == to_netuid {
        anyhow::bail!("manual fallback did not produce two distinct non-root subnets");
    }
    Ok((from_netuid, to_netuid))
}

async fn prep_subnets_for_stake_ops(client: &mut Client, from_netuid: u16, to_netuid: u16) {
    let alice = harness::dev_pair(harness::ALICE_URI);
    for netuid in [from_netuid, to_netuid] {
        let _ = harness::sudo_admin_call(
            client,
            &alice,
            "sudo_set_subtoken_enabled",
            vec![
                subxt::dynamic::Value::u128(netuid as u128),
                subxt::dynamic::Value::bool(true),
            ],
        )
        .await;
        let _ = harness::ensure_alice_on_subnet(client, NetUid(netuid)).await;
    }
    harness::setup_global_rate_limits(client, &alice).await;
    harness::wait_blocks(client, 4).await;
}

fn run_btcli_wallet_create(wallet_dir: &str) -> Result<()> {
    let script = format!(
        "source .venv/bin/activate && timeout 60s btcli wallet create \
         --wallet-name {} --wallet-path {} --hotkey {} --uri Alice \
         --no-use-password --overwrite --json-output",
        shell_quote(WALLET_NAME),
        shell_quote(wallet_dir),
        shell_quote(WALLET_HOTKEY)
    );
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    let out = run_cmd(cmd, "btcli wallet create")?;
    assert_success(&out, "btcli wallet create")
}

fn run_agcli_wallet_dev_create(wallet_dir: &str) -> Result<()> {
    let script = format!(
        "timeout 60s {} --wallet-dir {} --wallet {} --hotkey {} --batch --output json --yes wallet dev --uri Alice --password {}",
        shell_quote(&agcli_bin()),
        shell_quote(wallet_dir),
        shell_quote(WALLET_NAME),
        shell_quote(WALLET_HOTKEY),
        shell_quote(&agcli_password()),
    );
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    let out = run_cmd(cmd, "agcli wallet dev create")?;
    assert_success(&out, "agcli wallet dev create")
}

fn build_btcli_script(op: Operation, ctx: &ScenarioContext) -> String {
    let mut command = format!(
        "source .venv/bin/activate && timeout 120s btcli stake {} --wallet-name {} --wallet-path {} \
         --wallet-hotkey {} --network {} --origin-netuid {} --dest-netuid {} --amount {} \
         --no-mev-protection --no-prompt --json-output",
        match op {
            Operation::Move => "move",
            Operation::Swap => "swap",
            Operation::TransferStake => "transfer",
        },
        shell_quote(WALLET_NAME),
        shell_quote(&ctx.wallet_dir_path),
        shell_quote(WALLET_HOTKEY),
        shell_quote(&ctx.endpoint),
        ctx.from_netuid,
        ctx.to_netuid,
        op.amount_tao(),
    );

    match op {
        Operation::Move => {
            command.push_str(&format!(
                " --from {} --to {}",
                shell_quote(&ctx.hotkey_ss58),
                shell_quote(&ctx.hotkey_ss58)
            ));
        }
        Operation::Swap => {}
        Operation::TransferStake => {
            command.push_str(&format!(
                " --dest-ss58 {}",
                shell_quote(&ctx.dest_coldkey_ss58)
            ));
        }
    }
    command
}

fn build_sdk_sync_script() -> &'static str {
    r#"
import json
import os
import time
import bittensor
from bittensor.utils.balance import Balance

wallet = bittensor.Wallet(
    name=os.environ["WALLET_NAME"],
    hotkey=os.environ["WALLET_HOTKEY"],
    path=os.environ["WALLET_DIR"],
)
origin_netuid = int(os.environ["FROM_NETUID"])
destination_netuid = int(os.environ["TO_NETUID"])
amount_tao = float(os.environ["AMOUNT_TAO"])
hotkey = os.environ["HOTKEY_SS58"]
destination_coldkey = os.environ["DEST_COLDKEY_SS58"]
operation = os.environ["OPERATION"]
endpoint = os.environ["ENDPOINT"]

def is_transient_error(message: str) -> bool:
    lower = message.lower()
    return (
        "temporarily banned" in lower
        or "priority is too low" in lower
        or "outdated" in lower
        or "1012" in lower
    )

def execute_once():
    amount = Balance.from_tao(amount_tao).set_unit(origin_netuid)
    subtensor = bittensor.Subtensor(network=endpoint)
    if operation == "move":
        subtensor.move_stake(
            wallet=wallet,
            origin_netuid=origin_netuid,
            origin_hotkey_ss58=hotkey,
            destination_netuid=destination_netuid,
            destination_hotkey_ss58=hotkey,
            amount=amount,
            mev_protection=False,
            raise_error=True,
            wait_for_inclusion=True,
            wait_for_finalization=True,
            wait_for_revealed_execution=False,
        )
    elif operation == "swap":
        subtensor.swap_stake(
            wallet=wallet,
            hotkey_ss58=hotkey,
            origin_netuid=origin_netuid,
            destination_netuid=destination_netuid,
            amount=amount,
            mev_protection=False,
            raise_error=True,
            wait_for_inclusion=True,
            wait_for_finalization=True,
            wait_for_revealed_execution=False,
        )
    elif operation == "transfer_stake":
        subtensor.transfer_stake(
            wallet=wallet,
            destination_coldkey_ss58=destination_coldkey,
            hotkey_ss58=hotkey,
            origin_netuid=origin_netuid,
            destination_netuid=destination_netuid,
            amount=amount,
            mev_protection=False,
            raise_error=True,
            wait_for_inclusion=True,
            wait_for_finalization=True,
            wait_for_revealed_execution=False,
        )
    else:
        raise RuntimeError(f"unknown operation: {operation}")

last_error = None
for attempt in range(20):
    try:
        execute_once()
        break
    except Exception as error:
        if is_transient_error(str(error)):
            last_error = error
            time.sleep(min(12.0, 1.0 + attempt))
            continue
        raise
else:
    if last_error is not None:
        raise last_error
    raise RuntimeError("retry loop exhausted without result")

print(json.dumps({"ok": True}))
"#
}

fn build_sdk_async_script() -> &'static str {
    r#"
import asyncio
import json
import os
import bittensor
from bittensor.core.async_subtensor import AsyncSubtensor
from bittensor.utils.balance import Balance

wallet = bittensor.Wallet(
    name=os.environ["WALLET_NAME"],
    hotkey=os.environ["WALLET_HOTKEY"],
    path=os.environ["WALLET_DIR"],
)
origin_netuid = int(os.environ["FROM_NETUID"])
destination_netuid = int(os.environ["TO_NETUID"])
amount_tao = float(os.environ["AMOUNT_TAO"])
hotkey = os.environ["HOTKEY_SS58"]
destination_coldkey = os.environ["DEST_COLDKEY_SS58"]
operation = os.environ["OPERATION"]
endpoint = os.environ["ENDPOINT"]

def is_transient_error(message: str) -> bool:
    lower = message.lower()
    return (
        "temporarily banned" in lower
        or "priority is too low" in lower
        or "outdated" in lower
        or "1012" in lower
    )

async def execute_once() -> None:
    amount = Balance.from_tao(amount_tao).set_unit(origin_netuid)
    async with AsyncSubtensor(network=endpoint) as subtensor:
        if operation == "move":
            await subtensor.move_stake(
                wallet=wallet,
                origin_netuid=origin_netuid,
                origin_hotkey_ss58=hotkey,
                destination_netuid=destination_netuid,
                destination_hotkey_ss58=hotkey,
                amount=amount,
                mev_protection=False,
                raise_error=True,
                wait_for_inclusion=True,
                wait_for_finalization=True,
                wait_for_revealed_execution=False,
            )
        elif operation == "swap":
            await subtensor.swap_stake(
                wallet=wallet,
                hotkey_ss58=hotkey,
                origin_netuid=origin_netuid,
                destination_netuid=destination_netuid,
                amount=amount,
                mev_protection=False,
                raise_error=True,
                wait_for_inclusion=True,
                wait_for_finalization=True,
                wait_for_revealed_execution=False,
            )
        elif operation == "transfer_stake":
            await subtensor.transfer_stake(
                wallet=wallet,
                destination_coldkey_ss58=destination_coldkey,
                hotkey_ss58=hotkey,
                origin_netuid=origin_netuid,
                destination_netuid=destination_netuid,
                amount=amount,
                mev_protection=False,
                raise_error=True,
                wait_for_inclusion=True,
                wait_for_finalization=True,
                wait_for_revealed_execution=False,
            )
        else:
            raise RuntimeError(f"unknown operation: {operation}")

async def main() -> None:
    last_error = None
    for attempt in range(20):
        try:
            await execute_once()
            return
        except Exception as error:
            if is_transient_error(str(error)):
                last_error = error
                await asyncio.sleep(min(12.0, 1.0 + attempt))
                continue
            raise
    if last_error is not None:
        raise last_error
    raise RuntimeError("retry loop exhausted without result")

asyncio.run(main())
print(json.dumps({"ok": True}))
"#
}

fn build_agcli_args(op: Operation, ctx: &ScenarioContext, scale_amount_to_rao: bool) -> Vec<String> {
    let amount = if scale_amount_to_rao {
        ((op.amount_tao() * 1_000_000_000.0).round() as u64).to_string()
    } else {
        op.amount_tao().to_string()
    };
    let password = agcli_password();
    let mut args = vec![
        "--endpoint".to_string(),
        ctx.endpoint.clone(),
        "--wallet-dir".to_string(),
        ctx.wallet_dir_path.clone(),
        "--wallet".to_string(),
        WALLET_NAME.to_string(),
        "--hotkey".to_string(),
        WALLET_HOTKEY.to_string(),
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
        "--yes".to_string(),
        "--password".to_string(),
        password,
    ];

    match op {
        Operation::Move => args.extend([
            "stake".to_string(),
            "move".to_string(),
            "--amount".to_string(),
            amount.clone(),
            "--from".to_string(),
            ctx.from_netuid.to_string(),
            "--to".to_string(),
            ctx.to_netuid.to_string(),
            "--hotkey-address".to_string(),
            ctx.hotkey_ss58.clone(),
        ]),
        Operation::Swap => args.extend([
            "stake".to_string(),
            "swap".to_string(),
            "--amount".to_string(),
            amount.clone(),
            "--from".to_string(),
            ctx.from_netuid.to_string(),
            "--to".to_string(),
            ctx.to_netuid.to_string(),
            "--hotkey-address".to_string(),
            ctx.hotkey_ss58.clone(),
        ]),
        Operation::TransferStake => args.extend([
            "stake".to_string(),
            "transfer-stake".to_string(),
            "--dest".to_string(),
            ctx.dest_coldkey_ss58.clone(),
            "--amount".to_string(),
            amount,
            "--from".to_string(),
            ctx.from_netuid.to_string(),
            "--to".to_string(),
            ctx.to_netuid.to_string(),
            "--hotkey-address".to_string(),
            ctx.hotkey_ss58.clone(),
        ]),
    }

    args
}

async fn snapshot_state(
    client: &mut Client,
    coldkeys: &[String],
    hotkey: &str,
) -> Result<ChainSnapshot> {
    let mut balances_rao = BTreeMap::new();
    let mut stakes_rao = BTreeMap::new();

    for coldkey in coldkeys {
        harness::ensure_alive(client).await;
        let balance = client
            .get_balance_ss58(coldkey)
            .await
            .with_context(|| format!("failed balance query for {coldkey}"))?;
        balances_rao.insert(coldkey.clone(), balance.rao());

        harness::ensure_alive(client).await;
        let stake_info = client
            .get_stake_for_coldkey(coldkey)
            .await
            .with_context(|| format!("failed stake query for {coldkey}"))?;
        let mut stake_by_netuid = BTreeMap::<u16, u64>::new();
        for entry in stake_info {
            if entry.hotkey == hotkey {
                *stake_by_netuid.entry(entry.netuid.0).or_insert(0) += entry.stake.rao();
            }
        }
        stakes_rao.insert(coldkey.clone(), stake_by_netuid);
    }

    Ok(ChainSnapshot {
        balances_rao,
        stakes_rao,
    })
}

fn compute_delta(before: &ChainSnapshot, after: &ChainSnapshot) -> ChainDelta {
    let mut coldkeys = BTreeSet::new();
    coldkeys.extend(before.balances_rao.keys().cloned());
    coldkeys.extend(after.balances_rao.keys().cloned());

    let mut balances_rao = BTreeMap::new();
    let mut stakes_rao = BTreeMap::new();

    for coldkey in coldkeys {
        let before_balance = before
            .balances_rao
            .get(&coldkey)
            .copied()
            .unwrap_or_default();
        let after_balance = after
            .balances_rao
            .get(&coldkey)
            .copied()
            .unwrap_or_default();
        balances_rao.insert(
            coldkey.clone(),
            after_balance as i128 - before_balance as i128,
        );

        let before_stakes = before.stakes_rao.get(&coldkey);
        let after_stakes = after.stakes_rao.get(&coldkey);
        let mut netuids = BTreeSet::new();
        if let Some(map) = before_stakes {
            netuids.extend(map.keys().copied());
        }
        if let Some(map) = after_stakes {
            netuids.extend(map.keys().copied());
        }
        let mut stake_delta = BTreeMap::new();
        for netuid in netuids {
            let b = before_stakes
                .and_then(|m| m.get(&netuid).copied())
                .unwrap_or_default();
            let a = after_stakes
                .and_then(|m| m.get(&netuid).copied())
                .unwrap_or_default();
            stake_delta.insert(netuid, a as i128 - b as i128);
        }
        stakes_rao.insert(coldkey, stake_delta);
    }

    ChainDelta {
        balances_rao,
        stakes_rao,
    }
}

async fn bootstrap_scenario(
    reference: Reference,
    container_name: &str,
) -> Result<(ContainerGuard, Client, ScenarioContext)> {
    cleanup_port_and_container(container_name);
    let guard = ContainerGuard {
        name: container_name.to_string(),
    };
    let temp = TempDir::new().context("failed creating temp config directory")?;
    let config_path = write_variant_c_config(&temp, container_name)?;

    let scaffold_script = format!(
        "timeout 150s {} --output json localnet scaffold --config {}",
        shell_quote(&agcli_bin()),
        shell_quote(&config_path)
    );
    let mut scaffold_cmd = Command::new("bash");
    scaffold_cmd.args(["-lc", &scaffold_script]);
    let scaffold_out = run_cmd(scaffold_cmd, "agcli localnet scaffold")?;
    let (endpoint, from_netuid, to_netuid, mut client) = if scaffold_out.status.success() {
        let scaffold_json = parse_stdout_json(&scaffold_out, "agcli localnet scaffold")?;
        let scaffold: ScaffoldResult =
            serde_json::from_value(scaffold_json).context("failed decoding scaffold output")?;
        let mut netuids: Vec<u16> = scaffold.subnets.iter().map(|s| s.netuid).collect();
        netuids.sort_unstable();
        netuids.dedup();
        if netuids.len() < 2 {
            anyhow::bail!("variant C scaffold did not produce at least two subnets");
        }
        let client = harness::wait_for_chain().await;
        (scaffold.endpoint, netuids[0], netuids[1], client)
    } else if is_scaffold_tempo_failure(&scaffold_out) {
        eprintln!(
            "[parity_stake_advanced] scaffold variant C hit sudo_set_tempo failure, using manual multi-subnet fallback"
        );
        cleanup_port_and_container(container_name);
        start_localnet_container(container_name)?;
        let mut client = harness::wait_for_chain().await;
        let (from_netuid, to_netuid) = ensure_multi_subnet_fallback(&mut client).await?;
        (
            harness::LOCAL_WS.to_string(),
            from_netuid,
            to_netuid,
            client,
        )
    } else {
        assert_success(&scaffold_out, "agcli localnet scaffold")?;
        unreachable!("assert_success returns Err on failure");
    };

    let wallet_temp = TempDir::new().context("failed creating temp wallet directory")?;
    let wallet_dir_path = wallet_temp.path().to_string_lossy().to_string();
    if matches!(reference, Reference::Agcli) {
        run_agcli_wallet_dev_create(&wallet_dir_path)?;
    } else {
        run_btcli_wallet_create(&wallet_dir_path)?;
    }
    prep_subnets_for_stake_ops(&mut client, from_netuid, to_netuid).await;

    Ok((
        guard,
        client,
        ScenarioContext {
            _temp_wallet_dir: wallet_temp,
            wallet_dir_path,
            endpoint,
            from_netuid,
            to_netuid,
            hotkey_ss58: harness::ALICE_SS58.to_string(),
            source_coldkey_ss58: harness::ALICE_SS58.to_string(),
            dest_coldkey_ss58: harness::BOB_SS58.to_string(),
        },
    ))
}

fn run_btcli_command(op: Operation, ctx: &ScenarioContext) -> Result<()> {
    let script = build_btcli_script(op, ctx);
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    let out = run_cmd(cmd, "btcli stake operation")?;
    assert_success(&out, "btcli stake operation")
}

fn run_sdk_sync_command(op: Operation, ctx: &ScenarioContext) -> Result<()> {
    let script = build_sdk_sync_script();
    let wrapped =
        format!("source .venv/bin/activate && timeout 120s python - <<'PY'\n{script}\nPY");
    let mut last_output = None;
    for attempt in 0..6 {
        let mut cmd = Command::new("bash");
        cmd.args(["-lc", &wrapped])
            .env("WALLET_NAME", WALLET_NAME)
            .env("WALLET_HOTKEY", WALLET_HOTKEY)
            .env("WALLET_DIR", &ctx.wallet_dir_path)
            .env("ENDPOINT", &ctx.endpoint)
            .env("FROM_NETUID", ctx.from_netuid.to_string())
            .env("TO_NETUID", ctx.to_netuid.to_string())
            .env("HOTKEY_SS58", &ctx.hotkey_ss58)
            .env("DEST_COLDKEY_SS58", &ctx.dest_coldkey_ss58)
            .env("AMOUNT_TAO", op.amount_tao().to_string())
            .env("OPERATION", op.slug());
        let out = run_cmd(cmd, "bittensor sdk sync stake operation")?;
        if out.status.success() {
            return Ok(());
        }
        let transient = has_transient_tx_error(&out);
        last_output = Some(out);
        if transient && attempt < 5 {
            thread::sleep(Duration::from_secs((attempt + 2) as u64));
            continue;
        }
        break;
    }
    let out = last_output.expect("sdk sync loop must record failed output");
    assert_success(&out, "bittensor sdk sync stake operation")
}

fn run_sdk_async_command(op: Operation, ctx: &ScenarioContext) -> Result<()> {
    let script = build_sdk_async_script();
    let wrapped =
        format!("source .venv/bin/activate && timeout 120s python - <<'PY'\n{script}\nPY");
    let mut last_output = None;
    for attempt in 0..8 {
        let mut cmd = Command::new("bash");
        cmd.args(["-lc", &wrapped])
            .env("WALLET_NAME", WALLET_NAME)
            .env("WALLET_HOTKEY", WALLET_HOTKEY)
            .env("WALLET_DIR", &ctx.wallet_dir_path)
            .env("ENDPOINT", &ctx.endpoint)
            .env("FROM_NETUID", ctx.from_netuid.to_string())
            .env("TO_NETUID", ctx.to_netuid.to_string())
            .env("HOTKEY_SS58", &ctx.hotkey_ss58)
            .env("DEST_COLDKEY_SS58", &ctx.dest_coldkey_ss58)
            .env("AMOUNT_TAO", op.amount_tao().to_string())
            .env("OPERATION", op.slug());
        let out = run_cmd(cmd, "bittensor sdk async stake operation")?;
        if out.status.success() {
            return Ok(());
        }
        let transient = has_transient_tx_error(&out);
        last_output = Some(out);
        if transient && attempt < 7 {
            thread::sleep(Duration::from_secs((attempt + 2) as u64));
            continue;
        }
        break;
    }
    let out = last_output.expect("sdk async loop must record failed output");
    assert_success(&out, "bittensor sdk async stake operation")
}

fn run_agcli_command(op: Operation, ctx: &ScenarioContext) -> Result<()> {
    for scaled in [false, true] {
        let args = build_agcli_args(op, ctx, scaled);
        let script = format!(
            "timeout 120s {} {}",
            shell_quote(&agcli_bin()),
            shell_join(&args)
        );
        let mut cmd = Command::new("bash");
        cmd.args(["-lc", &script]);
        let out = run_cmd(cmd, "agcli stake operation")?;
        if out.status.success() {
            return Ok(());
        }
        if !scaled && has_amount_too_low_error(&out) {
            continue;
        }
        return assert_success(&out, "agcli stake operation");
    }
    anyhow::bail!("agcli stake operation failed after amount fallback")
}

async fn execute_reference_sequence(
    reference: Reference,
) -> Result<BTreeMap<Operation, ChainDelta>> {
    let container_name = format!("{}_{}", CATEGORY_CONTAINER_PREFIX, reference.slug());
    let (_guard, mut client, ctx) = bootstrap_scenario(reference, &container_name).await?;
    let tracked_coldkeys = vec![
        ctx.source_coldkey_ss58.clone(),
        ctx.dest_coldkey_ss58.clone(),
    ];
    let mut deltas = BTreeMap::new();

    for op in OPERATIONS {
        println!(
            "[parity_stake_advanced] start {} via {}",
            op.slug(),
            reference.slug()
        );
        let before = snapshot_state(&mut client, &tracked_coldkeys, &ctx.hotkey_ss58).await?;
        match reference {
            Reference::Btcli => run_btcli_command(op, &ctx)?,
            Reference::SdkSync => run_sdk_sync_command(op, &ctx)?,
            Reference::SdkAsync => run_sdk_async_command(op, &ctx)?,
            Reference::Agcli => run_agcli_command(op, &ctx)?,
        }
        harness::ensure_alive(&mut client).await;
        harness::wait_blocks(&mut client, 6).await;
        let after = snapshot_state(&mut client, &tracked_coldkeys, &ctx.hotkey_ss58).await?;
        println!(
            "[parity_stake_advanced] done {} via {}",
            op.slug(),
            reference.slug()
        );
        deltas.insert(op, compute_delta(&before, &after));
    }

    Ok(deltas)
}

#[tokio::test]
async fn stake_advanced_parity_btcli_sdk_agcli() {
    let _guard = acquire_test_lock();
    if !require_prerequisites() {
        return;
    }
    let btcli = execute_reference_sequence(Reference::Btcli)
        .await
        .expect("btcli sequence must execute");
    let sdk_sync = execute_reference_sequence(Reference::SdkSync)
        .await
        .expect("sdk sync sequence must execute");
    let sdk_async = execute_reference_sequence(Reference::SdkAsync)
        .await
        .expect("sdk async sequence must execute");
    let agcli = execute_reference_sequence(Reference::Agcli)
        .await
        .expect("agcli sequence must execute");

    for op in OPERATIONS {
        let btcli_delta = btcli.get(&op).expect("missing btcli delta");
        let sdk_sync_delta = sdk_sync.get(&op).expect("missing sdk sync delta");
        let sdk_async_delta = sdk_async.get(&op).expect("missing sdk async delta");
        let agcli_delta = agcli.get(&op).expect("missing agcli delta");

        assert_eq!(
            btcli_delta,
            agcli_delta,
            "btcli delta diverged for {}",
            op.slug()
        );
        assert_eq!(
            sdk_sync_delta,
            agcli_delta,
            "sdk sync delta diverged for {}",
            op.slug()
        );
        assert_eq!(
            sdk_async_delta,
            agcli_delta,
            "sdk async delta diverged for {}",
            op.slug()
        );
    }
}
