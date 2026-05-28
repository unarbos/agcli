#![cfg(feature = "e2e")]

use super::harness;
use agcli::chain::Client;
use anyhow::{Context, Result};
use fs2::FileExt;
use serde::Deserialize;
use serde_json::Value;
use sp_core::Pair as _;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::process::{Command, Output};
use tempfile::TempDir;

const CATEGORY_CONTAINER_PREFIX: &str = "agcli_parity_misc";
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
fund_tao = 2000.0
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
}

#[derive(Debug, Clone, Deserialize)]
struct DevKeyResult {
    name: String,
    coldkey: String,
}

#[derive(Debug)]
struct ScenarioContext {
    _wallet_dir: TempDir,
    wallet_dir_path: String,
    wallet_name: String,
    endpoint: String,
    netuid: u16,
    coldkey_ss58: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CrowdloanDelta {
    created_count: usize,
    created_cap_rao: Option<u64>,
    created_end_block: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SwapDelta {
    scheduled: bool,
    new_coldkey: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OperationDelta {
    Crowdloan(CrowdloanDelta),
    Swap(SwapDelta),
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    CrowdloanCreate,
    SwapColdkey,
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
            Self::CrowdloanCreate => "crowdloan_create",
            Self::SwapColdkey => "swap_coldkey",
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
        "source .venv/bin/activate >/dev/null 2>&1 && python3 -c 'import bittensor' >/dev/null 2>&1";
    Command::new("bash")
        .args(["-lc", script])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn require_chain_prerequisites() -> bool {
    if !docker_available() {
        eprintln!("[parity_misc] Docker unavailable; skipping chain parity tests.");
        return false;
    }
    if !btcli_available() {
        eprintln!("[parity_misc] btcli unavailable in .venv; skipping chain parity tests.");
        return false;
    }
    if !sdk_available() {
        eprintln!("[parity_misc] bittensor SDK unavailable in .venv; skipping chain parity tests.");
        return false;
    }
    true
}

fn require_ux_prerequisites() -> bool {
    if !docker_available() {
        eprintln!("[parity_misc] Docker unavailable; skipping UX misc-group checks.");
        return false;
    }
    true
}

fn acquire_category_lock() -> Result<File> {
    let lock_path = "/tmp/agcli_parity_misc.lock";
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
        String::from_utf8_lossy(&output.stderr),
    );
}

fn parse_json_payload(bytes: &[u8], label: &str) -> Result<Value> {
    if let Ok(v) = serde_json::from_slice::<Value>(bytes) {
        return Ok(v);
    }
    let text = String::from_utf8_lossy(bytes);
    for line in text.lines().rev() {
        if let Ok(v) = serde_json::from_str::<Value>(line.trim()) {
            return Ok(v);
        }
    }
    anyhow::bail!("{label} emitted non-JSON payload:\n{text}");
}

fn parse_stdout_json(output: &Output, label: &str) -> Result<Value> {
    parse_json_payload(&output.stdout, label)
}

fn parse_stderr_json_error(output: &Output, label: &str) -> Result<Value> {
    let value = parse_json_payload(&output.stderr, label)?;
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
    let scaffold_path = temp_dir.path().join("scaffold-A-baseline.toml");
    let content = VARIANT_A_CONFIG.replace("__CONTAINER__", container);
    fs::write(&scaffold_path, content)
        .with_context(|| format!("failed writing scaffold config {}", scaffold_path.display()))?;
    Ok(scaffold_path.to_string_lossy().to_string())
}

fn cleanup_port_and_container(container: &str) {
    let _ = Command::new("docker")
        .args(["rm", "-f", container])
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

fn derive_wallet_name(seed_uri: &str) -> String {
    seed_uri.trim_start_matches('/').to_lowercase()
}

fn swap_new_coldkey() -> String {
    let bob = harness::dev_pair(harness::BOB_URI);
    harness::to_ss58(&bob.public())
}

fn build_btcli_script(op: Operation, ctx: &ScenarioContext, end_block: u32) -> String {
    match op {
        Operation::CrowdloanCreate => format!(
            "source .venv/bin/activate && btcli crowd create --wallet-name {} --wallet-path {} --wallet-hotkey default --network {} --deposit 2 --min-contribution 1 --cap 5 --end {} --no-prompt --json-output",
            shell_quote(&ctx.wallet_name),
            shell_quote(&ctx.wallet_dir_path),
            shell_quote(&ctx.endpoint),
            end_block,
        ),
        Operation::SwapColdkey => format!(
            "source .venv/bin/activate && btcli wallet swap-coldkey announce --wallet-name {} --wallet-path {} --wallet-hotkey default --network {} --new-coldkey {} --no-prompt --json-output",
            shell_quote(&ctx.wallet_name),
            shell_quote(&ctx.wallet_dir_path),
            shell_quote(&ctx.endpoint),
            shell_quote(&swap_new_coldkey()),
        ),
    }
}

fn build_sdk_script(op: Operation) -> &'static str {
    match op {
        Operation::CrowdloanCreate => {
            r#"
import os
import bittensor
from bittensor.utils.balance import Balance

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.create_crowdloan(
    wallet=wallet,
    deposit=Balance.from_tao(float(os.environ["DEPOSIT_TAO"])),
    min_contribution=Balance.from_tao(float(os.environ["MIN_TAO"])),
    cap=Balance.from_tao(float(os.environ["CAP_TAO"])),
    end=int(os.environ["END_BLOCK"]),
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#
        }
        Operation::SwapColdkey => {
            r#"
import os
import bittensor

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.announce_coldkey_swap(
    wallet=wallet,
    new_coldkey_ss58=os.environ["NEW_COLDKEY"],
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#
        }
    }
}

fn base_agcli_args(ctx: &ScenarioContext) -> Vec<String> {
    vec![
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
    ]
}

fn build_agcli_success_args(op: Operation, ctx: &ScenarioContext, end_block: u32) -> Vec<String> {
    let mut args = base_agcli_args(ctx);
    args.push("--yes".to_string());
    match op {
        Operation::CrowdloanCreate => args.extend([
            "crowdloan".to_string(),
            "create".to_string(),
            "--deposit".to_string(),
            "2".to_string(),
            "--min-contribution".to_string(),
            "1".to_string(),
            "--cap".to_string(),
            "5".to_string(),
            "--end-block".to_string(),
            end_block.to_string(),
        ]),
        Operation::SwapColdkey => args.extend([
            "swap".to_string(),
            "coldkey".to_string(),
            "--new-coldkey".to_string(),
            swap_new_coldkey(),
        ]),
    }
    args
}

fn build_agcli_dry_run_args(op: Operation, ctx: &ScenarioContext, end_block: u32) -> Vec<String> {
    let mut args = base_agcli_args(ctx);
    args.extend([
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
        "--dry-run".to_string(),
    ]);
    match op {
        Operation::CrowdloanCreate => args.extend([
            "crowdloan".to_string(),
            "create".to_string(),
            "--deposit".to_string(),
            "2".to_string(),
            "--min-contribution".to_string(),
            "1".to_string(),
            "--cap".to_string(),
            "5".to_string(),
            "--end-block".to_string(),
            end_block.to_string(),
        ]),
        Operation::SwapColdkey => args.extend([
            "swap".to_string(),
            "coldkey".to_string(),
            "--new-coldkey".to_string(),
            swap_new_coldkey(),
        ]),
    }
    args
}

fn build_agcli_invalid_args(op: Operation, ctx: &ScenarioContext, end_block: u32) -> Vec<String> {
    let mut args = base_agcli_args(ctx);
    args.extend([
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
    ]);
    match op {
        Operation::CrowdloanCreate => args.extend([
            "crowdloan".to_string(),
            "create".to_string(),
            "--deposit".to_string(),
            "-1".to_string(),
            "--min-contribution".to_string(),
            "1".to_string(),
            "--cap".to_string(),
            "5".to_string(),
            "--end-block".to_string(),
            end_block.to_string(),
        ]),
        Operation::SwapColdkey => args.extend([
            "swap".to_string(),
            "coldkey".to_string(),
            "--new-coldkey".to_string(),
            "not-an-ss58".to_string(),
        ]),
    }
    args
}

async fn snapshot_crowdloan_state(
    client: &mut Client,
) -> Result<Vec<(u32, String, u64, u64, u64, u32, bool)>> {
    harness::ensure_alive(client).await;
    client
        .list_crowdloans()
        .await
        .context("failed to list crowdloans")
}

fn compute_crowdloan_delta(
    before: &[(u32, String, u64, u64, u64, u32, bool)],
    after: &[(u32, String, u64, u64, u64, u32, bool)],
) -> CrowdloanDelta {
    let before_ids: BTreeSet<u32> = before.iter().map(|(id, ..)| *id).collect();
    let created_rows: Vec<_> = after
        .iter()
        .filter(|(id, ..)| !before_ids.contains(id))
        .collect();
    let first = created_rows.first().copied();
    CrowdloanDelta {
        created_count: created_rows.len(),
        created_cap_rao: first.map(|(_, _, _, _, cap, _, _)| *cap),
        created_end_block: first.map(|(_, _, _, _, _, end, _)| *end),
    }
}

async fn snapshot_swap_state(
    client: &mut Client,
    coldkey_ss58: &str,
) -> Result<Option<(u32, String)>> {
    harness::ensure_alive(client).await;
    client
        .get_coldkey_swap_scheduled(coldkey_ss58)
        .await
        .with_context(|| format!("failed to query coldkey swap announcement for {coldkey_ss58}"))
}

fn compute_swap_delta(before: Option<(u32, String)>, after: Option<(u32, String)>) -> SwapDelta {
    match (before, after) {
        (None, Some((_, new_coldkey))) => SwapDelta {
            scheduled: true,
            new_coldkey: Some(new_coldkey),
        },
        (_, Some((_, new_coldkey))) => SwapDelta {
            scheduled: true,
            new_coldkey: Some(new_coldkey),
        },
        _ => SwapDelta {
            scheduled: false,
            new_coldkey: None,
        },
    }
}

fn run_btcli_command(op: Operation, ctx: &ScenarioContext, end_block: u32) -> Result<()> {
    let script = build_btcli_script(op, ctx, end_block);
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    let out = run_cmd(cmd, "btcli misc command")?;
    assert_success(&out, "btcli misc command")
}

fn run_sdk_command(op: Operation, ctx: &ScenarioContext, end_block: u32) -> Result<()> {
    let py = build_sdk_script(op);
    let script = format!("source .venv/bin/activate && python3 - <<'PY'\n{py}\nPY");
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script])
        .env("WALLET_NAME", &ctx.wallet_name)
        .env("WALLET_DIR", &ctx.wallet_dir_path)
        .env("ENDPOINT", &ctx.endpoint)
        .env("DEPOSIT_TAO", "2")
        .env("MIN_TAO", "1")
        .env("CAP_TAO", "5")
        .env("END_BLOCK", end_block.to_string())
        .env("NEW_COLDKEY", swap_new_coldkey());
    let out = run_cmd(cmd, "bittensor sdk misc command")?;
    assert_success(&out, "bittensor sdk misc command")
}

fn run_agcli_success_command(op: Operation, ctx: &ScenarioContext, end_block: u32) -> Result<()> {
    let args = build_agcli_success_args(op, ctx, end_block);
    let mut cmd = Command::new(agcli_bin());
    cmd.args(args);
    let out = run_cmd(cmd, "agcli misc command")?;
    assert_success(&out, "agcli misc command")?;
    if out.status.code() != Some(0) {
        anyhow::bail!(
            "agcli misc command expected exit code 0, got {:?}",
            out.status.code()
        );
    }
    Ok(())
}

fn assert_agcli_ux_claims(op: Operation, ctx: &ScenarioContext, end_block: u32) -> Result<()> {
    let dry_args = build_agcli_dry_run_args(op, ctx, end_block);
    let mut dry_cmd = Command::new(agcli_bin());
    dry_cmd.args(dry_args);
    let dry_out = run_cmd(dry_cmd, "agcli misc dry-run command")?;
    assert_success(&dry_out, "agcli misc dry-run command")?;
    let dry_json = parse_stdout_json(&dry_out, "agcli misc dry-run command")?;
    if !(dry_json.is_object() || dry_json.is_array()) {
        anyhow::bail!("agcli misc dry-run output must be JSON object/array, got: {dry_json}");
    }

    let invalid_args = build_agcli_invalid_args(op, ctx, end_block);
    let mut invalid_cmd = Command::new(agcli_bin());
    invalid_cmd.args(invalid_args);
    let invalid_out = run_cmd(invalid_cmd, "agcli misc invalid-input command")?;
    if invalid_out.status.success() {
        anyhow::bail!("agcli misc invalid-input command unexpectedly succeeded");
    }
    let err_json = parse_stderr_json_error(&invalid_out, "agcli misc invalid-input command")?;
    if invalid_out.status.code() != Some(12) {
        anyhow::bail!(
            "agcli misc invalid-input command expected exit code 12, got {:?}; payload: {}",
            invalid_out.status.code(),
            err_json
        );
    }

    Ok(())
}

fn assert_misc_group_ux_claims(ctx: &ScenarioContext) -> Result<()> {
    let mut run_json_ok = |args: Vec<String>, label: &str| -> Result<()> {
        let mut cmd = Command::new(agcli_bin());
        cmd.args(args);
        let out = run_cmd(cmd, label)?;
        assert_success(&out, label)?;
        let v = parse_stdout_json(&out, label)?;
        if !(v.is_object() || v.is_array()) {
            anyhow::bail!("{label} expected JSON object/array, got: {v}");
        }
        Ok(())
    };
    let mut run_json_err = |args: Vec<String>, label: &str| -> Result<()> {
        let mut cmd = Command::new(agcli_bin());
        cmd.args(args);
        let out = run_cmd(cmd, label)?;
        if out.status.success() {
            anyhow::bail!("{label} unexpectedly succeeded");
        }
        let _ = parse_stderr_json_error(&out, label)?;
        Ok(())
    };

    run_json_ok(
        vec![
            "--batch".to_string(),
            "--output".to_string(),
            "json".to_string(),
            "utils".to_string(),
            "convert".to_string(),
            "--amount".to_string(),
            "1".to_string(),
            "--to-rao".to_string(),
        ],
        "agcli utils convert json",
    )?;

    let mut base = base_agcli_args(ctx);
    base.extend([
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
    ]);

    let mut evm_bad = base.clone();
    evm_bad.extend([
        "evm".to_string(),
        "call".to_string(),
        "--source".to_string(),
        "0x1234".to_string(),
        "--target".to_string(),
        "0x1111111111111111111111111111111111111111".to_string(),
    ]);
    run_json_err(evm_bad, "agcli evm invalid input json")?;

    let mut contracts_bad = base.clone();
    contracts_bad.extend([
        "contracts".to_string(),
        "remove-code".to_string(),
        "--code-hash".to_string(),
        "0x1234".to_string(),
    ]);
    run_json_err(contracts_bad, "agcli contracts invalid input json")?;

    let mut drand_bad = base.clone();
    drand_bad.extend([
        "drand".to_string(),
        "write-pulse".to_string(),
        "--payload".to_string(),
        "zz".to_string(),
        "--signature".to_string(),
        "yy".to_string(),
    ]);
    run_json_err(drand_bad, "agcli drand invalid input json")?;

    let mut liquidity_bad = base.clone();
    liquidity_bad.extend([
        "liquidity".to_string(),
        "add".to_string(),
        "--netuid".to_string(),
        ctx.netuid.to_string(),
        "--price-low".to_string(),
        "1".to_string(),
        "--price-high".to_string(),
        "2".to_string(),
        "--amount".to_string(),
        "0".to_string(),
    ]);
    run_json_err(liquidity_bad, "agcli liquidity invalid input json")?;

    let mut safe_mode_dry = base_agcli_args(ctx);
    safe_mode_dry.extend([
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
        "--dry-run".to_string(),
        "safe-mode".to_string(),
        "enter".to_string(),
    ]);
    run_json_ok(safe_mode_dry, "agcli safe-mode dry-run json")?;

    Ok(())
}

async fn bootstrap_scenario(
    container_name: &str,
) -> Result<(ContainerGuard, Client, ScenarioContext)> {
    harness::ensure_local_chain();
    cleanup_port_and_container(container_name);
    let guard = ContainerGuard {
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
        .first()
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
        guard,
        client,
        ScenarioContext {
            _wallet_dir: wallet_temp,
            wallet_dir_path,
            wallet_name,
            endpoint: scaffold.endpoint,
            netuid: subnet.netuid,
            coldkey_ss58: dev_key.coldkey,
        },
    ))
}

async fn execute_reference(op: Operation, reference: Reference) -> Result<OperationDelta> {
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
    let end_block = client
        .get_block_number()
        .await
        .context("failed to query current block for crowdloan end")?
        .saturating_add(100);

    match op {
        Operation::CrowdloanCreate => {
            let before = snapshot_crowdloan_state(&mut client).await?;
            match reference {
                Reference::Btcli => run_btcli_command(op, &ctx, end_block)?,
                Reference::Sdk => run_sdk_command(op, &ctx, end_block)?,
                Reference::Agcli => {
                    assert_agcli_ux_claims(op, &ctx, end_block)?;
                    run_agcli_success_command(op, &ctx, end_block)?;
                }
            }
            harness::ensure_alive(&mut client).await;
            harness::wait_blocks(&mut client, 6).await;
            let after = snapshot_crowdloan_state(&mut client).await?;
            Ok(OperationDelta::Crowdloan(compute_crowdloan_delta(
                &before, &after,
            )))
        }
        Operation::SwapColdkey => {
            let before = snapshot_swap_state(&mut client, &ctx.coldkey_ss58).await?;
            match reference {
                Reference::Btcli => run_btcli_command(op, &ctx, end_block)?,
                Reference::Sdk => run_sdk_command(op, &ctx, end_block)?,
                Reference::Agcli => {
                    assert_agcli_ux_claims(op, &ctx, end_block)?;
                    run_agcli_success_command(op, &ctx, end_block)?;
                }
            }
            harness::ensure_alive(&mut client).await;
            harness::wait_blocks(&mut client, 6).await;
            let after = snapshot_swap_state(&mut client, &ctx.coldkey_ss58).await?;
            Ok(OperationDelta::Swap(compute_swap_delta(before, after)))
        }
    }
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
async fn misc_crowdloan_create_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_chain_prerequisites() {
        return;
    }
    assert_parity_for_operation(Operation::CrowdloanCreate)
        .await
        .expect("crowdloan create parity must match");
}

#[tokio::test]
async fn misc_swap_coldkey_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_chain_prerequisites() {
        return;
    }
    assert_parity_for_operation(Operation::SwapColdkey)
        .await
        .expect("swap coldkey parity must match");
}

#[tokio::test]
async fn misc_group_ux_claims_json_errors() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_ux_prerequisites() {
        return;
    }

    let container_name = format!("{CATEGORY_CONTAINER_PREFIX}_ux_misc_groups");
    let (_guard, _client, ctx) = bootstrap_scenario(&container_name)
        .await
        .expect("bootstrap scenario for misc UX checks must succeed");
    assert_misc_group_ux_claims(&ctx).expect("misc group UX claims must hold");
}
