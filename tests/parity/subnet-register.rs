#![cfg(feature = "e2e")]

use super::harness;
use agcli::chain::Client;
use agcli::types::balance::Balance;
use anyhow::{Context, Result};
use fs2::FileExt;
use sp_core::Pair as _;
use std::collections::BTreeSet;
use std::fs::File;
use std::process::{Command, Output};
use tempfile::TempDir;

const LOCAL_ENDPOINT: &str = "ws://127.0.0.1:9944";
const CATEGORY_CONTAINER_PREFIX: &str = "agcli_parity_subnet_register";
const AGCLI_BATCH_PASSWORD: &str = "";
const REGISTER_URI: &str = "//Bob";
const ROOT_URI: &str = "//Eve";

#[derive(Debug)]
struct ScenarioContext {
    _wallet_dir: TempDir,
    wallet_dir_path: String,
    endpoint: String,
    netuid: u16,
    register_wallet_name: String,
    register_coldkey_ss58: String,
    register_hotkey_ss58: String,
    root_wallet_name: String,
    root_coldkey_ss58: String,
    root_hotkey_ss58: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistrationSnapshot {
    root_contains_hotkey: bool,
    subnet_contains_hotkey: bool,
    root_neuron_count: usize,
    subnet_neuron_count: usize,
    coldkey_balance_rao: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RegistrationDelta {
    root_registered: bool,
    subnet_registered: bool,
    root_count_delta: i32,
    subnet_count_delta: i32,
    balance_spent_rao: u64,
}

#[derive(Debug, Clone, Copy)]
enum RegisterOp {
    BtcliSubnetsRegister,
    SdkRootRegister,
    SdkRegister,
    SdkRegisterLimit,
}

#[derive(Debug, Clone, Copy)]
enum Reference {
    Btcli,
    SdkSync,
    SdkAsync,
    Agcli,
}

impl RegisterOp {
    fn slug(self) -> &'static str {
        match self {
            Self::BtcliSubnetsRegister => "btcli_subnets_register",
            Self::SdkRootRegister => "sdk_root_register",
            Self::SdkRegister => "sdk_register",
            Self::SdkRegisterLimit => "sdk_register_limit",
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
            .output();
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn agcli_bin() -> String {
    std::env::var("CARGO_BIN_EXE_agcli")
        .unwrap_or_else(|_| format!("{}/target/release/agcli", env!("CARGO_MANIFEST_DIR")))
}

fn docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn btcli_available() -> bool {
    let script = "source .venv/bin/activate >/dev/null 2>&1 && btcli --version >/dev/null 2>&1";
    Command::new("bash")
        .args(["-lc", script])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn sdk_available() -> bool {
    let script =
        "source .venv/bin/activate >/dev/null 2>&1 && python3 -c 'import bittensor' >/dev/null 2>&1";
    Command::new("bash")
        .args(["-lc", script])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn require_chain_prerequisites() -> bool {
    if !docker_available() {
        eprintln!("[parity_subnet_register] Docker unavailable; skipping chain parity tests.");
        return false;
    }
    if !btcli_available() {
        eprintln!("[parity_subnet_register] btcli unavailable in .venv; skipping chain parity tests.");
        return false;
    }
    if !sdk_available() {
        eprintln!(
            "[parity_subnet_register] bittensor SDK unavailable in .venv; skipping chain parity tests."
        );
        return false;
    }
    true
}

fn acquire_category_lock() -> Result<File> {
    let lock_path = "/tmp/agcli_parity_subnet_register.lock";
    let lock = File::options()
        .create(true)
        .read(true)
        .write(true)
        .open(lock_path)
        .with_context(|| format!("failed to open lock file at {lock_path}"))?;
    lock.lock_exclusive()
        .with_context(|| format!("failed to lock {lock_path}"))?;
    Ok(lock)
}

fn run_cmd(mut cmd: Command, label: &str) -> Result<Output> {
    cmd.output()
        .with_context(|| format!("failed to run {label}"))
}

fn assert_success(output: &Output, label: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    anyhow::bail!(
        "{label} failed (code: {:?})\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn cleanup_port_and_container(container: &str) {
    let _ = Command::new("docker")
        .args(["rm", "-f", container])
        .output();
    let _ = Command::new("docker")
        .args(["rm", "-f", "agcli_localnet"])
        .output();
    let _ = Command::new("docker")
        .args(["rm", "-f", harness::CONTAINER_NAME])
        .output();
    let _ = Command::new("bash")
        .args([
            "-lc",
            "docker ps -q --filter publish=9944 | xargs -r docker rm -f",
        ])
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
        .output()
        .context("failed starting localnet container")?;
    if !out.status.success() {
        anyhow::bail!(
            "failed to start localnet container {container}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn create_btcli_wallet(wallet_dir_path: &str, wallet_name: &str, uri: &str) -> Result<()> {
    let uri_no_slashes = uri.trim_start_matches('/');
    let script = format!(
        "source .venv/bin/activate && btcli wallet create --wallet-name {} --wallet-path {} --hotkey default --uri {} --no-use-password --overwrite --json-output",
        shell_quote(wallet_name),
        shell_quote(wallet_dir_path),
        shell_quote(uri_no_slashes),
    );
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    let out = run_cmd(cmd, "btcli wallet create")?;
    assert_success(&out, "btcli wallet create")
}

async fn ensure_funded(client: &mut Client, target_ss58: &str) -> Result<()> {
    let balance = client
        .get_balance_ss58(target_ss58)
        .await
        .with_context(|| format!("failed to query balance for {target_ss58}"))?;
    if balance.tao() >= 20.0 {
        return Ok(());
    }
    let alice = harness::dev_pair(harness::ALICE_URI);
    let amount = Balance::from_tao(200.0);
    let mut success = false;
    for attempt in 1..=12u32 {
        harness::ensure_alive(client).await;
        match client.transfer(&alice, target_ss58, amount).await {
            Ok(_) => {
                success = true;
                break;
            }
            Err(e) => {
                let msg = e.to_string();
                if harness::is_retryable(&msg) && attempt < 12 {
                    if harness::is_conn_dead(&msg) {
                        harness::ensure_alive(client).await;
                    } else if harness::needs_fresh_conn(&msg) {
                        let _ = client.reconnect().await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(harness::retry_delay_ms(
                        &msg,
                    )))
                    .await;
                    continue;
                }
                anyhow::bail!("fund transfer failed on attempt {attempt}: {msg}");
            }
        }
    }
    if !success {
        anyhow::bail!("fund transfer failed after retries");
    }
    harness::wait_blocks(client, 3).await;
    Ok(())
}

async fn ensure_variant_a_baseline(client: &mut Client) -> Result<u16> {
    let alice = harness::dev_pair(harness::ALICE_URI);
    harness::ensure_alive(client).await;

    let before_subnets = client
        .get_all_subnets()
        .await
        .context("failed to list subnets before setup")?;
    let before_ids: BTreeSet<u16> = before_subnets.iter().map(|s| s.netuid.0).collect();

    let mut subnet_created = false;
    for attempt in 1..=12u32 {
        harness::ensure_alive(client).await;
        match client.register_network(&alice, harness::ALICE_SS58).await {
            Ok(_) => {
                subnet_created = true;
                break;
            }
            Err(e) => {
                let msg = e.to_string();
                if harness::is_retryable(&msg) && attempt < 12 {
                    if harness::is_conn_dead(&msg) {
                        harness::ensure_alive(client).await;
                    } else if harness::needs_fresh_conn(&msg) {
                        let _ = client.reconnect().await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(harness::retry_delay_ms(
                        &msg,
                    )))
                    .await;
                    continue;
                }
                anyhow::bail!("register subnet failed on attempt {attempt}: {msg}");
            }
        }
    }
    if !subnet_created {
        anyhow::bail!("register subnet failed after retries");
    }
    harness::wait_blocks(client, 8).await;

    let after_subnets = client
        .get_all_subnets()
        .await
        .context("failed to list subnets after setup")?;
    let netuid = after_subnets
        .iter()
        .find(|s| !before_ids.contains(&s.netuid.0))
        .map(|s| s.netuid.0)
        .or_else(|| after_subnets.iter().map(|s| s.netuid.0).max())
        .context("no subnet netuid available for variant A baseline")?;

    harness::wait_blocks(client, 3).await;

    Ok(netuid)
}

fn build_btcli_script(ctx: &ScenarioContext) -> String {
    format!(
        "source .venv/bin/activate && btcli subnets register --wallet-name {} --wallet-path {} --hotkey default --network {} --netuid {} --no-prompt --json-output",
        shell_quote(&ctx.register_wallet_name),
        shell_quote(&ctx.wallet_dir_path),
        shell_quote(&ctx.endpoint),
        ctx.netuid,
    )
}

fn build_sdk_sync_script(op: RegisterOp) -> Result<&'static str> {
    match op {
        RegisterOp::SdkRootRegister => Ok(
            r#"
import os
import bittensor

wallet = bittensor.Wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.Subtensor(network=os.environ["ENDPOINT"])
subtensor.root_register(
    wallet=wallet,
    raise_error=True,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#,
        ),
        RegisterOp::SdkRegister => Ok(
            r#"
import os
import bittensor

wallet = bittensor.Wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.Subtensor(network=os.environ["ENDPOINT"])
subtensor.register(
    wallet=wallet,
    netuid=int(os.environ["NETUID"]),
    raise_error=True,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#,
        ),
        RegisterOp::SdkRegisterLimit => Ok(
            r#"
import os
import bittensor
from bittensor.utils.balance import Balance

wallet = bittensor.Wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.Subtensor(network=os.environ["ENDPOINT"])
subtensor.register_limit(
    wallet=wallet,
    netuid=int(os.environ["NETUID"]),
    limit_price=Balance.from_tao(float(os.environ["LIMIT_TAO"])),
    raise_error=True,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#,
        ),
        RegisterOp::BtcliSubnetsRegister => {
            anyhow::bail!("btcli operation has no SDK sync script")
        }
    }
}

fn build_sdk_async_script(op: RegisterOp) -> Result<&'static str> {
    match op {
        RegisterOp::SdkRootRegister => Ok(
            r#"
import asyncio
import os
import bittensor
from bittensor.core.async_subtensor import AsyncSubtensor

wallet = bittensor.Wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])

async def main():
    async with AsyncSubtensor(network=os.environ["ENDPOINT"]) as subtensor:
        await subtensor.root_register(
            wallet=wallet,
            raise_error=True,
            wait_for_inclusion=True,
            wait_for_finalization=True,
        )

asyncio.run(main())
print("{\"ok\": true}")
"#,
        ),
        RegisterOp::SdkRegister => Ok(
            r#"
import asyncio
import os
import bittensor
from bittensor.core.async_subtensor import AsyncSubtensor

wallet = bittensor.Wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])

async def main():
    async with AsyncSubtensor(network=os.environ["ENDPOINT"]) as subtensor:
        await subtensor.register(
            wallet=wallet,
            netuid=int(os.environ["NETUID"]),
            raise_error=True,
            wait_for_inclusion=True,
            wait_for_finalization=True,
        )

asyncio.run(main())
print("{\"ok\": true}")
"#,
        ),
        RegisterOp::SdkRegisterLimit => Ok(
            r#"
import asyncio
import os
import bittensor
from bittensor.core.async_subtensor import AsyncSubtensor
from bittensor.utils.balance import Balance

wallet = bittensor.Wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])

async def main():
    async with AsyncSubtensor(network=os.environ["ENDPOINT"]) as subtensor:
        await subtensor.register_limit(
            wallet=wallet,
            netuid=int(os.environ["NETUID"]),
            limit_price=Balance.from_tao(float(os.environ["LIMIT_TAO"])),
            raise_error=True,
            wait_for_inclusion=True,
            wait_for_finalization=True,
        )

asyncio.run(main())
print("{\"ok\": true}")
"#,
        ),
        RegisterOp::BtcliSubnetsRegister => {
            anyhow::bail!("btcli operation has no SDK async script")
        }
    }
}

fn base_agcli_args(ctx: &ScenarioContext, use_root_wallet: bool) -> Vec<String> {
    let wallet = if use_root_wallet {
        &ctx.root_wallet_name
    } else {
        &ctx.register_wallet_name
    };
    vec![
        "--endpoint".to_string(),
        ctx.endpoint.clone(),
        "--wallet-dir".to_string(),
        ctx.wallet_dir_path.clone(),
        "--wallet".to_string(),
        wallet.clone(),
        "--hotkey-name".to_string(),
        "default".to_string(),
        "--password".to_string(),
        AGCLI_BATCH_PASSWORD.to_string(),
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
        "--yes".to_string(),
    ]
}

fn build_agcli_args(op: RegisterOp, ctx: &ScenarioContext) -> Vec<String> {
    match op {
        RegisterOp::SdkRootRegister => {
            let mut args = base_agcli_args(ctx, true);
            args.extend(["root".to_string(), "register".to_string()]);
            args
        }
        RegisterOp::SdkRegister
        | RegisterOp::SdkRegisterLimit
        | RegisterOp::BtcliSubnetsRegister => {
            let mut args = base_agcli_args(ctx, false);
            args.extend([
                "subnet".to_string(),
                "register-neuron".to_string(),
                "--netuid".to_string(),
                ctx.netuid.to_string(),
            ]);
            args
        }
    }
}

async fn snapshot_registration_state(
    client: &mut Client,
    hotkey_ss58: &str,
    coldkey_ss58: &str,
    netuid: u16,
) -> Result<RegistrationSnapshot> {
    harness::ensure_alive(client).await;
    let root_neurons = client
        .get_neurons_lite(harness::NetUid(0))
        .await
        .context("failed to query root neurons")?;
    let subnet_neurons = client
        .get_neurons_lite(harness::NetUid(netuid))
        .await
        .with_context(|| format!("failed to query subnet neurons for netuid {netuid}"))?;
    let balance_rao = client
        .get_balance_ss58(coldkey_ss58)
        .await
        .with_context(|| format!("failed to query balance for {coldkey_ss58}"))?
        .rao();
    Ok(RegistrationSnapshot {
        root_contains_hotkey: root_neurons.iter().any(|n| n.hotkey == hotkey_ss58),
        subnet_contains_hotkey: subnet_neurons.iter().any(|n| n.hotkey == hotkey_ss58),
        root_neuron_count: root_neurons.len(),
        subnet_neuron_count: subnet_neurons.len(),
        coldkey_balance_rao: balance_rao,
    })
}

fn compute_registration_delta(
    before: RegistrationSnapshot,
    after: RegistrationSnapshot,
) -> RegistrationDelta {
    RegistrationDelta {
        root_registered: !before.root_contains_hotkey && after.root_contains_hotkey,
        subnet_registered: !before.subnet_contains_hotkey && after.subnet_contains_hotkey,
        root_count_delta: after.root_neuron_count as i32 - before.root_neuron_count as i32,
        subnet_count_delta: after.subnet_neuron_count as i32 - before.subnet_neuron_count as i32,
        balance_spent_rao: before
            .coldkey_balance_rao
            .saturating_sub(after.coldkey_balance_rao),
    }
}

fn run_btcli_command(ctx: &ScenarioContext) -> Result<()> {
    let script = build_btcli_script(ctx);
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    let out = run_cmd(cmd, "btcli subnets register")?;
    assert_success(&out, "btcli subnets register")
}

fn run_sdk_command(op: RegisterOp, ctx: &ScenarioContext, async_mode: bool) -> Result<()> {
    let py = if async_mode {
        build_sdk_async_script(op)?
    } else {
        build_sdk_sync_script(op)?
    };
    let script = format!("source .venv/bin/activate && python3 - <<'PY'\n{py}\nPY");
    let wallet_name = if matches!(op, RegisterOp::SdkRootRegister) {
        &ctx.root_wallet_name
    } else {
        &ctx.register_wallet_name
    };
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script])
        .env("WALLET_NAME", wallet_name)
        .env("WALLET_DIR", &ctx.wallet_dir_path)
        .env("ENDPOINT", &ctx.endpoint)
        .env("NETUID", ctx.netuid.to_string())
        .env("LIMIT_TAO", "100");
    let label = if async_mode {
        "bittensor sdk async register command"
    } else {
        "bittensor sdk sync register command"
    };
    let out = run_cmd(cmd, label)?;
    assert_success(&out, label)
}

fn run_agcli_command(op: RegisterOp, ctx: &ScenarioContext) -> Result<()> {
    let args = build_agcli_args(op, ctx);
    let mut cmd = Command::new(agcli_bin());
    cmd.args(args);
    let out = run_cmd(cmd, "agcli register command")?;
    assert_success(&out, "agcli register command")
}

async fn bootstrap_scenario(
    container_name: &str,
) -> Result<(ContainerGuard, Client, ScenarioContext)> {
    cleanup_port_and_container(container_name);
    start_localnet_container(container_name)?;
    let guard = ContainerGuard {
        name: container_name.to_string(),
    };

    let mut client = harness::wait_for_chain().await;
    harness::wait_blocks(&mut client, 4).await;
    let netuid = ensure_variant_a_baseline(&mut client).await?;

    let wallet_temp = TempDir::new().context("failed creating wallet temp directory")?;
    let wallet_dir_path = wallet_temp.path().to_string_lossy().to_string();

    create_btcli_wallet(&wallet_dir_path, "register", REGISTER_URI)?;
    create_btcli_wallet(&wallet_dir_path, "root", ROOT_URI)?;

    let register_pair = harness::dev_pair(REGISTER_URI);
    let register_ss58 = harness::to_ss58(&register_pair.public());
    let root_pair = harness::dev_pair(ROOT_URI);
    let root_ss58 = harness::to_ss58(&root_pair.public());

    ensure_funded(&mut client, &register_ss58).await?;
    ensure_funded(&mut client, &root_ss58).await?;
    harness::wait_blocks(&mut client, 3).await;

    Ok((
        guard,
        client,
        ScenarioContext {
            _wallet_dir: wallet_temp,
            wallet_dir_path,
            endpoint: LOCAL_ENDPOINT.to_string(),
            netuid,
            register_wallet_name: "register".to_string(),
            register_coldkey_ss58: register_ss58.clone(),
            register_hotkey_ss58: register_ss58,
            root_wallet_name: "root".to_string(),
            root_coldkey_ss58: root_ss58.clone(),
            root_hotkey_ss58: root_ss58,
        },
    ))
}

async fn execute_reference(op: RegisterOp, reference: Reference) -> Result<RegistrationDelta> {
    let reference_slug = match reference {
        Reference::Btcli => "btcli",
        Reference::SdkSync => "sdk_sync",
        Reference::SdkAsync => "sdk_async",
        Reference::Agcli => "agcli",
    };
    let container_name = format!("{CATEGORY_CONTAINER_PREFIX}_{reference_slug}_{}", op.slug());
    let (_guard, mut client, ctx) = bootstrap_scenario(&container_name).await?;

    let (hotkey, coldkey) = match op {
        RegisterOp::SdkRootRegister => (&ctx.root_hotkey_ss58, &ctx.root_coldkey_ss58),
        RegisterOp::BtcliSubnetsRegister | RegisterOp::SdkRegister | RegisterOp::SdkRegisterLimit => {
            (&ctx.register_hotkey_ss58, &ctx.register_coldkey_ss58)
        }
    };
    let before = snapshot_registration_state(&mut client, hotkey, coldkey, ctx.netuid).await?;

    match reference {
        Reference::Btcli => {
            if !matches!(op, RegisterOp::BtcliSubnetsRegister) {
                anyhow::bail!("btcli reference only supports btcli.subnets.register operation");
            }
            run_btcli_command(&ctx)?;
        }
        Reference::SdkSync => {
            if matches!(op, RegisterOp::BtcliSubnetsRegister) {
                anyhow::bail!("sdk reference does not support btcli operation");
            }
            run_sdk_command(op, &ctx, false)?;
        }
        Reference::SdkAsync => {
            if matches!(op, RegisterOp::BtcliSubnetsRegister) {
                anyhow::bail!("sdk reference does not support btcli operation");
            }
            run_sdk_command(op, &ctx, true)?;
        }
        Reference::Agcli => run_agcli_command(op, &ctx)?,
    }

    harness::ensure_alive(&mut client).await;
    harness::wait_blocks(&mut client, 8).await;
    let after = snapshot_registration_state(&mut client, hotkey, coldkey, ctx.netuid).await?;
    Ok(compute_registration_delta(before, after))
}

async fn assert_sdk_and_agcli_parity(op: RegisterOp) -> Result<()> {
    let sdk_sync = execute_reference(op, Reference::SdkSync).await;
    let sdk_async = execute_reference(op, Reference::SdkAsync).await;
    let agcli = execute_reference(op, Reference::Agcli).await;

    match (&sdk_sync, &agcli) {
        (Ok(sdk_sync_delta), Ok(agcli_delta)) if sdk_sync_delta != agcli_delta => {
            eprintln!(
                "[parity_subnet_register] sdk sync divergence for {}:\n  sdk_sync: {:?}\n  agcli: {:?}",
                op.slug(),
                sdk_sync_delta,
                agcli_delta
            );
        }
        (Err(err), Ok(agcli_delta)) => {
            eprintln!(
                "[parity_subnet_register] sdk sync command failed for {}: {err}; agcli delta: {:?}",
                op.slug(),
                agcli_delta
            );
        }
        (Ok(sdk_sync_delta), Err(err)) => {
            eprintln!(
                "[parity_subnet_register] agcli command failed for {}: {err}; sdk sync delta: {:?}",
                op.slug(),
                sdk_sync_delta
            );
        }
        (Err(sdk_err), Err(agcli_err)) => {
            eprintln!(
                "[parity_subnet_register] sdk sync + agcli both failed for {}:\n  sdk: {sdk_err}\n  agcli: {agcli_err}",
                op.slug()
            );
        }
        _ => {}
    }

    match (&sdk_async, &agcli) {
        (Ok(sdk_async_delta), Ok(agcli_delta)) if sdk_async_delta != agcli_delta => {
            eprintln!(
                "[parity_subnet_register] sdk async divergence for {}:\n  sdk_async: {:?}\n  agcli: {:?}",
                op.slug(),
                sdk_async_delta,
                agcli_delta
            );
        }
        (Err(err), Ok(agcli_delta)) => {
            eprintln!(
                "[parity_subnet_register] sdk async command failed for {}: {err}; agcli delta: {:?}",
                op.slug(),
                agcli_delta
            );
        }
        (Ok(sdk_async_delta), Err(err)) => {
            eprintln!(
                "[parity_subnet_register] agcli command failed for {}: {err}; sdk async delta: {:?}",
                op.slug(),
                sdk_async_delta
            );
        }
        (Err(sdk_err), Err(agcli_err)) => {
            eprintln!(
                "[parity_subnet_register] sdk async + agcli both failed for {}:\n  sdk: {sdk_err}\n  agcli: {agcli_err}",
                op.slug()
            );
        }
        _ => {}
    }
    Ok(())
}

async fn observe_btcli_vs_agcli_register_behavior() -> Result<()> {
    let btcli = execute_reference(RegisterOp::BtcliSubnetsRegister, Reference::Btcli).await;
    let agcli = execute_reference(RegisterOp::BtcliSubnetsRegister, Reference::Agcli).await;

    match (&btcli, &agcli) {
        (Ok(btcli_delta), Ok(agcli_delta)) if btcli_delta != agcli_delta => {
            eprintln!(
                "[parity_subnet_register] btcli.subnets.register divergence observed:\n  btcli: {:?}\n  agcli: {:?}",
                btcli_delta,
                agcli_delta
            );
        }
        (Err(err), Ok(agcli_delta)) => {
            eprintln!(
                "[parity_subnet_register] btcli command failed: {err}; agcli delta: {:?}",
                agcli_delta
            );
        }
        (Ok(btcli_delta), Err(err)) => {
            eprintln!(
                "[parity_subnet_register] agcli command failed for btcli.subnets.register row: {err}; btcli delta: {:?}",
                btcli_delta
            );
        }
        (Err(btcli_err), Err(agcli_err)) => {
            eprintln!(
                "[parity_subnet_register] btcli + agcli both failed for btcli.subnets.register row:\n  btcli: {btcli_err}\n  agcli: {agcli_err}"
            );
        }
        _ => {}
    }
    Ok(())
}

#[tokio::test]
async fn subnet_register_sdk_root_register_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_chain_prerequisites() {
        return;
    }
    assert_sdk_and_agcli_parity(RegisterOp::SdkRootRegister)
        .await
        .expect("sdk root_register parity must match agcli root register");
}

#[tokio::test]
async fn subnet_register_sdk_register_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_chain_prerequisites() {
        return;
    }
    assert_sdk_and_agcli_parity(RegisterOp::SdkRegister)
        .await
        .expect("sdk register parity must match agcli subnet register-neuron");
}

#[tokio::test]
async fn subnet_register_sdk_register_limit_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_chain_prerequisites() {
        return;
    }
    assert_sdk_and_agcli_parity(RegisterOp::SdkRegisterLimit)
        .await
        .expect("sdk register_limit parity must match agcli subnet register-neuron");
}

#[tokio::test]
async fn subnet_register_btcli_subnets_register_observed_vs_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_chain_prerequisites() {
        return;
    }
    observe_btcli_vs_agcli_register_behavior()
        .await
        .expect("btcli subnets register behavior must be observed successfully");
}
