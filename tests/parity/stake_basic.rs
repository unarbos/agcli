#![cfg(feature = "e2e")]

use super::harness;
use agcli::chain::Client;
use agcli::types::{Balance, NetUid};
use anyhow::{Context, Result};
use fs2::FileExt;
use serde::Deserialize;
use serde_json::Value;
use sp_core::Pair as _;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const CATEGORY_CONTAINER_PREFIX: &str = "agcli_parity_stake_basic";
const PASSWORD: &str = "parity-pass-123";

const VARIANT_A_CONFIG: &str = r#"
[chain]
image = "ghcr.io/opentensor/subtensor-localnet:devnet-ready"
container = "__CONTAINER__"
port = 9944
start = true
timeout = 180

[[subnet]]
tempo = 100
max_allowed_validators = 8
min_allowed_weights = 1
weights_rate_limit = 0
commit_reveal = false

[[subnet.neuron]]
name = "validator1"
fund_tao = 1200.0
register = true

[[subnet.neuron]]
name = "miner1"
fund_tao = 100.0
register = true

[[subnet.neuron]]
name = "miner2"
fund_tao = 100.0
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
    neurons: Vec<ScaffoldNeuron>,
}

#[derive(Debug, Clone, Deserialize)]
struct ScaffoldNeuron {
    name: String,
    ss58: String,
}

#[derive(Debug, Clone, Deserialize)]
struct DevKeyResult {
    name: String,
    coldkey: String,
    hotkey: String,
}

#[derive(Debug)]
struct ScenarioContext {
    _wallet_dir: TempDir,
    wallet_dir_path: String,
    wallet_name: String,
    seed_uri: String,
    endpoint: String,
    netuid: u16,
    coldkey_ss58: String,
    hotkey_ss58: String,
}

#[derive(Debug, Clone)]
struct ChainSnapshot {
    free_balance_rao: u64,
    stake_by_netuid: BTreeMap<u16, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChainDelta {
    free_balance_rao: i128,
    stake_by_netuid: BTreeMap<u16, i128>,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    AddStake,
    RemoveStake,
    AddStakeLimit,
    RemoveStakeLimit,
    UnstakeAll,
}

#[derive(Debug, Clone, Copy)]
enum Reference {
    Btcli,
    Sdk,
    Agcli,
}

impl Operation {
    fn slug(self) -> &'static str {
        match self {
            Self::AddStake => "add_stake",
            Self::RemoveStake => "remove_stake",
            Self::AddStakeLimit => "add_stake_limit",
            Self::RemoveStakeLimit => "remove_stake_limit",
            Self::UnstakeAll => "unstake_all",
        }
    }

    fn amount_tao(self) -> f64 {
        match self {
            Self::AddStake | Self::AddStakeLimit => 5.0,
            Self::RemoveStake | Self::RemoveStakeLimit => 2.0,
            Self::UnstakeAll => 0.0,
        }
    }

    fn needs_seed_stake(self) -> bool {
        !matches!(self, Self::AddStake | Self::AddStakeLimit)
    }

    fn seed_amount_tao(self) -> f64 {
        match self {
            Self::UnstakeAll => 7.0,
            Self::RemoveStake | Self::RemoveStakeLimit => 6.0,
            _ => 0.0,
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
    std::env::var("CARGO_BIN_EXE_agcli").unwrap_or_else(|_| "agcli".to_string())
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
        "source .venv/bin/activate >/dev/null 2>&1 && python -c 'import bittensor' >/dev/null 2>&1";
    Command::new("bash")
        .args(["-lc", script])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn require_prerequisites() -> bool {
    if !docker_available() {
        eprintln!("[parity_stake_basic] Docker unavailable; skipping.");
        return false;
    }
    if !btcli_available() {
        eprintln!("[parity_stake_basic] btcli unavailable in .venv; skipping.");
        return false;
    }
    if !sdk_available() {
        eprintln!("[parity_stake_basic] bittensor SDK unavailable in .venv; skipping.");
        return false;
    }
    true
}

fn acquire_category_lock() -> Result<File> {
    let lock_path = "/tmp/agcli_parity_stake_basic.lock";
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
    let output = cmd
        .output()
        .with_context(|| format!("failed to run {label}"))?;
    Ok(output)
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

fn parse_stdout_json(output: &Output, label: &str) -> Result<Value> {
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "{label} emitted non-JSON stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn parse_stderr_json_error(output: &Output, label: &str) -> Result<Value> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let value: Value = serde_json::from_str(&stderr)
        .with_context(|| format!("{label} stderr is not structured JSON:\n{stderr}"))?;
    if value.get("error").and_then(Value::as_bool) != Some(true) {
        anyhow::bail!("{label} stderr JSON missing error=true: {value}");
    }
    if value.get("code").and_then(Value::as_i64).is_none() {
        anyhow::bail!("{label} stderr JSON missing numeric code: {value}");
    }
    if value.get("message").and_then(Value::as_str).is_none() {
        anyhow::bail!("{label} stderr JSON missing message string: {value}");
    }
    Ok(value)
}

fn write_variant_a_config(temp_dir: &TempDir, container: &str) -> Result<String> {
    let variants_path = Path::new("examples/scaffold-variants");
    let scaffold_path = if variants_path.exists() {
        variants_path.join("scaffold-A-baseline.toml")
    } else {
        temp_dir.path().join("scaffold-A-baseline.toml")
    };
    let content = VARIANT_A_CONFIG.replace("__CONTAINER__", container);
    fs::write(&scaffold_path, content)
        .with_context(|| format!("failed writing scaffold config {}", scaffold_path.display()))?;
    Ok(scaffold_path.to_string_lossy().to_string())
}

fn cleanup_port_and_container(container: &str) {
    let _ = Command::new("docker")
        .args(["rm", "-f", container])
        .output();
    let _ = Command::new("bash")
        .args([
            "-lc",
            "docker ps -q --filter publish=9944 | xargs -r docker rm -f",
        ])
        .output();
}

fn derive_wallet_name(seed_uri: &str) -> String {
    seed_uri.trim_start_matches('/').to_lowercase()
}

fn build_btcli_script(op: Operation, ctx: &ScenarioContext) -> String {
    let base = format!(
        "source .venv/bin/activate && btcli stake {} --wallet-name {} --wallet-path {} --wallet-hotkey default --network {} --no-prompt --json-output",
        match op {
            Operation::AddStake | Operation::AddStakeLimit => "add",
            Operation::RemoveStake | Operation::RemoveStakeLimit | Operation::UnstakeAll => "remove",
        },
        shell_quote(&ctx.wallet_name),
        shell_quote(&ctx.wallet_dir_path),
        shell_quote(&ctx.endpoint),
    );

    match op {
        Operation::AddStake => format!(
            "{base} --netuid {} --amount {} --hotkey {}",
            ctx.netuid,
            op.amount_tao(),
            shell_quote(&ctx.hotkey_ss58)
        ),
        Operation::AddStakeLimit => format!(
            "{base} --netuid {} --amount {} --hotkey {} --safe-staking --allow-partial-stake --rate-tolerance 1.0",
            ctx.netuid,
            op.amount_tao(),
            shell_quote(&ctx.hotkey_ss58)
        ),
        Operation::RemoveStake => format!(
            "{base} --netuid {} --amount {} --hotkey-ss58-address {}",
            ctx.netuid,
            op.amount_tao(),
            shell_quote(&ctx.hotkey_ss58)
        ),
        Operation::RemoveStakeLimit => format!(
            "{base} --netuid {} --amount {} --hotkey-ss58-address {} --safe-staking --allow-partial-stake --rate-tolerance 1.0",
            ctx.netuid,
            op.amount_tao(),
            shell_quote(&ctx.hotkey_ss58)
        ),
        Operation::UnstakeAll => format!(
            "{base} --netuid {} --unstake-all --hotkey-ss58-address {}",
            ctx.netuid,
            shell_quote(&ctx.hotkey_ss58)
        ),
    }
}

fn build_agcli_args(op: Operation, ctx: &ScenarioContext) -> Vec<String> {
    let mut args = vec![
        "--endpoint".to_string(),
        ctx.endpoint.clone(),
        "--wallet-dir".to_string(),
        ctx.wallet_dir_path.clone(),
        "--wallet".to_string(),
        ctx.wallet_name.clone(),
        "--hotkey".to_string(),
        "default".to_string(),
        "--password".to_string(),
        PASSWORD.to_string(),
    ];

    match op {
        Operation::AddStake => {
            args.push("--yes".to_string());
            args.extend([
                "stake".to_string(),
                "add".to_string(),
                "--amount".to_string(),
                op.amount_tao().to_string(),
                "--netuid".to_string(),
                ctx.netuid.to_string(),
                "--hotkey-address".to_string(),
                ctx.hotkey_ss58.clone(),
            ]);
        }
        Operation::RemoveStake => {
            args.push("--yes".to_string());
            args.extend([
                "stake".to_string(),
                "remove".to_string(),
                "--amount".to_string(),
                op.amount_tao().to_string(),
                "--netuid".to_string(),
                ctx.netuid.to_string(),
                "--hotkey-address".to_string(),
                ctx.hotkey_ss58.clone(),
            ]);
        }
        Operation::AddStakeLimit => {
            args.push("--yes".to_string());
            args.extend([
                "stake".to_string(),
                "add-limit".to_string(),
                "--amount".to_string(),
                op.amount_tao().to_string(),
                "--netuid".to_string(),
                ctx.netuid.to_string(),
                "--price".to_string(),
                "1.0".to_string(),
                "--partial".to_string(),
                "--hotkey-address".to_string(),
                ctx.hotkey_ss58.clone(),
            ]);
        }
        Operation::RemoveStakeLimit => {
            args.push("--yes".to_string());
            args.extend([
                "stake".to_string(),
                "remove-limit".to_string(),
                "--amount".to_string(),
                op.amount_tao().to_string(),
                "--netuid".to_string(),
                ctx.netuid.to_string(),
                "--price".to_string(),
                "1.0".to_string(),
                "--partial".to_string(),
                "--hotkey-address".to_string(),
                ctx.hotkey_ss58.clone(),
            ]);
        }
        Operation::UnstakeAll => {
            args.push("--yes".to_string());
            args.extend([
                "stake".to_string(),
                "unstake-all".to_string(),
                "--hotkey-address".to_string(),
                ctx.hotkey_ss58.clone(),
            ]);
        }
    }

    args
}

fn build_agcli_dry_run_args(op: Operation, ctx: &ScenarioContext) -> Vec<String> {
    let mut args = vec![
        "--endpoint".to_string(),
        ctx.endpoint.clone(),
        "--wallet-dir".to_string(),
        ctx.wallet_dir_path.clone(),
        "--wallet".to_string(),
        ctx.wallet_name.clone(),
        "--hotkey".to_string(),
        "default".to_string(),
        "--password".to_string(),
        PASSWORD.to_string(),
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
        "--dry-run".to_string(),
    ];

    match op {
        Operation::AddStake => args.extend([
            "stake".to_string(),
            "add".to_string(),
            "--amount".to_string(),
            op.amount_tao().to_string(),
            "--netuid".to_string(),
            ctx.netuid.to_string(),
            "--hotkey-address".to_string(),
            ctx.hotkey_ss58.clone(),
        ]),
        Operation::RemoveStake => args.extend([
            "stake".to_string(),
            "remove".to_string(),
            "--amount".to_string(),
            op.amount_tao().to_string(),
            "--netuid".to_string(),
            ctx.netuid.to_string(),
            "--hotkey-address".to_string(),
            ctx.hotkey_ss58.clone(),
        ]),
        Operation::AddStakeLimit => args.extend([
            "stake".to_string(),
            "add-limit".to_string(),
            "--amount".to_string(),
            op.amount_tao().to_string(),
            "--netuid".to_string(),
            ctx.netuid.to_string(),
            "--price".to_string(),
            "1.0".to_string(),
            "--partial".to_string(),
            "--hotkey-address".to_string(),
            ctx.hotkey_ss58.clone(),
        ]),
        Operation::RemoveStakeLimit => args.extend([
            "stake".to_string(),
            "remove-limit".to_string(),
            "--amount".to_string(),
            op.amount_tao().to_string(),
            "--netuid".to_string(),
            ctx.netuid.to_string(),
            "--price".to_string(),
            "1.0".to_string(),
            "--partial".to_string(),
            "--hotkey-address".to_string(),
            ctx.hotkey_ss58.clone(),
        ]),
        Operation::UnstakeAll => args.extend([
            "stake".to_string(),
            "unstake-all".to_string(),
            "--hotkey-address".to_string(),
            ctx.hotkey_ss58.clone(),
        ]),
    }

    args
}

fn build_agcli_invalid_args(op: Operation, ctx: &ScenarioContext) -> Vec<String> {
    let mut args = vec![
        "--endpoint".to_string(),
        ctx.endpoint.clone(),
        "--wallet-dir".to_string(),
        ctx.wallet_dir_path.clone(),
        "--wallet".to_string(),
        ctx.wallet_name.clone(),
        "--hotkey".to_string(),
        "default".to_string(),
        "--password".to_string(),
        PASSWORD.to_string(),
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
    ];

    match op {
        Operation::AddStake => args.extend([
            "stake".to_string(),
            "add".to_string(),
            "--amount".to_string(),
            "-1".to_string(),
            "--netuid".to_string(),
            ctx.netuid.to_string(),
        ]),
        Operation::RemoveStake => args.extend([
            "stake".to_string(),
            "remove".to_string(),
            "--amount".to_string(),
            "-1".to_string(),
            "--netuid".to_string(),
            ctx.netuid.to_string(),
        ]),
        Operation::AddStakeLimit => args.extend([
            "stake".to_string(),
            "add-limit".to_string(),
            "--amount".to_string(),
            "1".to_string(),
            "--netuid".to_string(),
            ctx.netuid.to_string(),
            "--price".to_string(),
            "0".to_string(),
        ]),
        Operation::RemoveStakeLimit => args.extend([
            "stake".to_string(),
            "remove-limit".to_string(),
            "--amount".to_string(),
            "1".to_string(),
            "--netuid".to_string(),
            ctx.netuid.to_string(),
            "--price".to_string(),
            "0".to_string(),
        ]),
        Operation::UnstakeAll => args.extend([
            "stake".to_string(),
            "unstake-all".to_string(),
            "--hotkey-address".to_string(),
            "not-an-ss58".to_string(),
        ]),
    }

    args
}

fn build_sdk_script(op: Operation) -> &'static str {
    match op {
        Operation::AddStake => {
            r#"
import os
import bittensor
from bittensor.utils.balance import Balance

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.add_stake(
    wallet=wallet,
    netuid=int(os.environ["NETUID"]),
    hotkey_ss58=os.environ["HOTKEY_SS58"],
    amount=Balance.from_tao(float(os.environ["AMOUNT_TAO"])),
    safe_staking=False,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#
        }
        Operation::RemoveStake => {
            r#"
import os
import bittensor
from bittensor.utils.balance import Balance

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.unstake(
    wallet=wallet,
    netuid=int(os.environ["NETUID"]),
    hotkey_ss58=os.environ["HOTKEY_SS58"],
    amount=Balance.from_tao(float(os.environ["AMOUNT_TAO"])),
    safe_unstaking=False,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#
        }
        Operation::AddStakeLimit => {
            r#"
import os
import bittensor
from bittensor.utils.balance import Balance

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.add_stake(
    wallet=wallet,
    netuid=int(os.environ["NETUID"]),
    hotkey_ss58=os.environ["HOTKEY_SS58"],
    amount=Balance.from_tao(float(os.environ["AMOUNT_TAO"])),
    safe_staking=True,
    allow_partial_stake=True,
    rate_tolerance=1.0,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#
        }
        Operation::RemoveStakeLimit => {
            r#"
import os
import bittensor
from bittensor.utils.balance import Balance

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.unstake(
    wallet=wallet,
    netuid=int(os.environ["NETUID"]),
    hotkey_ss58=os.environ["HOTKEY_SS58"],
    amount=Balance.from_tao(float(os.environ["AMOUNT_TAO"])),
    safe_unstaking=True,
    allow_partial_stake=True,
    rate_tolerance=1.0,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#
        }
        Operation::UnstakeAll => {
            r#"
import os
import bittensor

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.unstake_all(
    wallet=wallet,
    netuid=int(os.environ["NETUID"]),
    hotkey_ss58=os.environ["HOTKEY_SS58"],
    rate_tolerance=1.0,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#
        }
    }
}

async fn snapshot_state(client: &mut Client, coldkey: &str, hotkey: &str) -> Result<ChainSnapshot> {
    harness::ensure_alive(client).await;
    let balance = client
        .get_balance_ss58(coldkey)
        .await
        .with_context(|| format!("failed to query balance for {coldkey}"))?;
    let stakes = client
        .get_stake_for_coldkey(coldkey)
        .await
        .with_context(|| format!("failed to query stake for {coldkey}"))?;
    let mut by_netuid = BTreeMap::<u16, u64>::new();
    for info in stakes {
        if info.hotkey == hotkey {
            *by_netuid.entry(info.netuid.0).or_insert(0) += info.stake.rao();
        }
    }
    Ok(ChainSnapshot {
        free_balance_rao: balance.rao(),
        stake_by_netuid: by_netuid,
    })
}

fn compute_delta(before: &ChainSnapshot, after: &ChainSnapshot) -> ChainDelta {
    let mut keys = BTreeSet::new();
    keys.extend(before.stake_by_netuid.keys().copied());
    keys.extend(after.stake_by_netuid.keys().copied());

    let stake_by_netuid = keys
        .into_iter()
        .map(|netuid| {
            let b = before
                .stake_by_netuid
                .get(&netuid)
                .copied()
                .unwrap_or_default();
            let a = after
                .stake_by_netuid
                .get(&netuid)
                .copied()
                .unwrap_or_default();
            (netuid, a as i128 - b as i128)
        })
        .collect();

    ChainDelta {
        free_balance_rao: after.free_balance_rao as i128 - before.free_balance_rao as i128,
        stake_by_netuid,
    }
}

async fn seed_stake_if_needed(
    op: Operation,
    client: &mut Client,
    ctx: &ScenarioContext,
) -> Result<()> {
    if !op.needs_seed_stake() {
        return Ok(());
    }

    let pair = harness::dev_pair(&ctx.seed_uri);
    let amount = Balance::from_tao(op.seed_amount_tao());
    client
        .add_stake(&pair, &ctx.hotkey_ss58, NetUid(ctx.netuid), amount)
        .await
        .with_context(|| format!("failed to seed stake for {}", op.slug()))?;
    harness::wait_blocks(client, 4).await;
    Ok(())
}

fn run_btcli_command(op: Operation, ctx: &ScenarioContext) -> Result<()> {
    let script = build_btcli_script(op, ctx);
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    let out = run_cmd(cmd, "btcli stake command")?;
    assert_success(&out, "btcli stake command")
}

fn run_sdk_command(op: Operation, ctx: &ScenarioContext) -> Result<()> {
    let py = build_sdk_script(op);
    let script = format!("source .venv/bin/activate && python - <<'PY'\n{py}\nPY");
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script])
        .env("WALLET_NAME", &ctx.wallet_name)
        .env("WALLET_DIR", &ctx.wallet_dir_path)
        .env("ENDPOINT", &ctx.endpoint)
        .env("NETUID", ctx.netuid.to_string())
        .env("HOTKEY_SS58", &ctx.hotkey_ss58)
        .env("AMOUNT_TAO", op.amount_tao().to_string());
    let out = run_cmd(cmd, "bittensor sdk command")?;
    assert_success(&out, "bittensor sdk command")
}

fn run_agcli_success_command(op: Operation, ctx: &ScenarioContext) -> Result<()> {
    let args = build_agcli_args(op, ctx);
    let mut cmd = Command::new(agcli_bin());
    cmd.args(args);
    let out = run_cmd(cmd, "agcli stake command")?;
    assert_success(&out, "agcli stake command")?;
    if out.status.code() != Some(0) {
        anyhow::bail!(
            "agcli stake command expected exit code 0, got {:?}",
            out.status.code()
        );
    }
    Ok(())
}

fn assert_agcli_ux_claims(op: Operation, ctx: &ScenarioContext) -> Result<()> {
    let dry_args = build_agcli_dry_run_args(op, ctx);
    let mut dry_cmd = Command::new(agcli_bin());
    dry_cmd.args(dry_args);
    let dry_out = run_cmd(dry_cmd, "agcli dry-run command")?;
    assert_success(&dry_out, "agcli dry-run command")?;
    let dry_json = parse_stdout_json(&dry_out, "agcli dry-run command")?;
    if !(dry_json.is_object() || dry_json.is_array()) {
        anyhow::bail!("agcli dry-run output must be JSON object/array, got: {dry_json}");
    }

    let invalid_args = build_agcli_invalid_args(op, ctx);
    let mut invalid_cmd = Command::new(agcli_bin());
    invalid_cmd.args(invalid_args);
    let invalid_out = run_cmd(invalid_cmd, "agcli invalid-input command")?;
    if invalid_out.status.success() {
        anyhow::bail!("agcli invalid-input command unexpectedly succeeded");
    }
    let err_json = parse_stderr_json_error(&invalid_out, "agcli invalid-input command")?;
    if invalid_out.status.code() != Some(12) {
        anyhow::bail!(
            "agcli invalid-input command expected exit code 12, got {:?}; payload: {}",
            invalid_out.status.code(),
            err_json
        );
    }

    Ok(())
}

async fn bootstrap_scenario(
    container_name: &str,
) -> Result<(ContainerGuard, Client, ScenarioContext)> {
    cleanup_port_and_container(container_name);
    let _guard = ContainerGuard {
        name: container_name.to_string(),
    };

    let temp = TempDir::new().context("failed creating temporary scaffold directory")?;
    let config_path = write_variant_a_config(&temp, container_name)?;

    let mut scaffold_cmd = Command::new(agcli_bin());
    scaffold_cmd.args([
        "--output",
        "json",
        "localnet",
        "scaffold",
        "--config",
        &config_path,
    ]);
    let scaffold_out = run_cmd(scaffold_cmd, "agcli localnet scaffold")?;
    assert_success(&scaffold_out, "agcli localnet scaffold")?;
    let scaffold_json = parse_stdout_json(&scaffold_out, "agcli localnet scaffold")?;
    let scaffold: ScaffoldResult =
        serde_json::from_value(scaffold_json).context("failed to decode scaffold JSON payload")?;

    let subnet = scaffold
        .subnets
        .first()
        .context("scaffold result missing subnets")?;
    let neuron = subnet
        .neurons
        .iter()
        .find(|n| n.name == "validator1")
        .or_else(|| subnet.neurons.first())
        .context("scaffold result missing neurons")?;
    let seed_uri = format!("//{}_sn{}", neuron.name, subnet.netuid);

    let wallet_temp = TempDir::new().context("failed creating wallet temp directory")?;
    let wallet_dir_path = wallet_temp.path().to_string_lossy().to_string();

    let mut dev_key_cmd = Command::new(agcli_bin());
    dev_key_cmd.args([
        "--output",
        "json",
        "--wallet-dir",
        &wallet_dir_path,
        "wallet",
        "dev-key",
        "--uri",
        &seed_uri,
        "--password",
        PASSWORD,
    ]);
    let dev_key_out = run_cmd(dev_key_cmd, "agcli wallet dev-key")?;
    assert_success(&dev_key_out, "agcli wallet dev-key")?;
    let dev_key_json = parse_stdout_json(&dev_key_out, "agcli wallet dev-key")?;
    let dev_key: DevKeyResult = serde_json::from_value(dev_key_json)
        .context("failed to decode wallet dev-key JSON payload")?;

    let wallet_name = if dev_key.name.is_empty() {
        derive_wallet_name(&seed_uri)
    } else {
        dev_key.name
    };

    let mut client = harness::wait_for_chain().await;
    harness::ensure_alive(&mut client).await;
    harness::wait_blocks(&mut client, 3).await;

    Ok((
        _guard,
        client,
        ScenarioContext {
            _wallet_dir: wallet_temp,
            wallet_dir_path,
            wallet_name,
            seed_uri,
            endpoint: scaffold.endpoint,
            netuid: subnet.netuid,
            coldkey_ss58: dev_key.coldkey,
            hotkey_ss58: if dev_key.hotkey.is_empty() {
                neuron.ss58.clone()
            } else {
                dev_key.hotkey
            },
        },
    ))
}

async fn execute_reference(op: Operation, reference: Reference) -> Result<ChainDelta> {
    let container_name = format!(
        "{CATEGORY_CONTAINER_PREFIX}_{}_{}",
        match reference {
            Reference::Btcli => "btcli",
            Reference::Sdk => "sdk",
            Reference::Agcli => "agcli",
        },
        op.slug()
    );
    let (_guard, mut client, ctx) = bootstrap_scenario(&container_name).await?;
    seed_stake_if_needed(op, &mut client, &ctx).await?;

    let before = snapshot_state(&mut client, &ctx.coldkey_ss58, &ctx.hotkey_ss58).await?;

    match reference {
        Reference::Btcli => run_btcli_command(op, &ctx)?,
        Reference::Sdk => run_sdk_command(op, &ctx)?,
        Reference::Agcli => {
            assert_agcli_ux_claims(op, &ctx)?;
            run_agcli_success_command(op, &ctx)?;
        }
    }

    harness::ensure_alive(&mut client).await;
    harness::wait_blocks(&mut client, 5).await;
    let after = snapshot_state(&mut client, &ctx.coldkey_ss58, &ctx.hotkey_ss58).await?;
    Ok(compute_delta(&before, &after))
}

async fn assert_parity_for_operation(op: Operation) -> Result<()> {
    let btcli_delta = execute_reference(op, Reference::Btcli).await?;
    let sdk_delta = execute_reference(op, Reference::Sdk).await?;
    let agcli_delta = execute_reference(op, Reference::Agcli).await?;

    if btcli_delta != agcli_delta {
        anyhow::bail!(
            "btcli delta diverged for {}:\n  btcli: {:?}\n  agcli: {:?}",
            op.slug(),
            btcli_delta,
            agcli_delta
        );
    }
    if sdk_delta != agcli_delta {
        anyhow::bail!(
            "sdk delta diverged for {}:\n  sdk: {:?}\n  agcli: {:?}",
            op.slug(),
            sdk_delta,
            agcli_delta
        );
    }
    Ok(())
}

#[tokio::test]
async fn stake_add_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }
    assert_parity_for_operation(Operation::AddStake)
        .await
        .expect("add stake parity must match");
}

#[tokio::test]
async fn stake_remove_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }
    assert_parity_for_operation(Operation::RemoveStake)
        .await
        .expect("remove stake parity must match");
}

#[tokio::test]
async fn stake_add_limit_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }
    assert_parity_for_operation(Operation::AddStakeLimit)
        .await
        .expect("add stake limit parity must match");
}

#[tokio::test]
async fn stake_remove_limit_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }
    assert_parity_for_operation(Operation::RemoveStakeLimit)
        .await
        .expect("remove stake limit parity must match");
}

#[tokio::test]
async fn stake_unstake_all_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }
    assert_parity_for_operation(Operation::UnstakeAll)
        .await
        .expect("unstake all parity must match");
}
