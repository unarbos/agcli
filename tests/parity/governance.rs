#![cfg(feature = "e2e")]

use super::harness;
use agcli::chain::Client;
use agcli::types::NetUid;
use anyhow::{Context, Result};
use fs2::FileExt;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const CATEGORY_CONTAINER_PREFIX: &str = "agcli_parity_governance";
const PASSWORD: &str = "parity-pass-123";
const DELEGATE_SS58: &str = harness::BOB_SS58;

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
    alice_wallet_name: String,
    seed_uri: String,
    endpoint: String,
    netuid: u16,
    coldkey_ss58: String,
    delegate_ss58: String,
    target_tempo: u16,
}

#[derive(Debug, Clone)]
struct ChainSnapshot {
    free_balance_rao: u64,
    proxies: BTreeSet<(String, String, u32)>,
    tempo: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChainDelta {
    free_balance_rao: i128,
    proxies_added: BTreeSet<(String, String, u32)>,
    proxies_removed: BTreeSet<(String, String, u32)>,
    tempo_before: Option<u16>,
    tempo_after: Option<u16>,
}

#[derive(Debug, Clone, Copy)]
enum GovernanceOperation {
    ProxyAdd,
    ProxyRemove,
    SudoSetTempo,
}

#[derive(Debug, Clone, Copy)]
enum Reference {
    Btcli,
    Sdk,
    Agcli,
}

impl GovernanceOperation {
    fn slug(self) -> &'static str {
        match self {
            Self::ProxyAdd => "proxy_add",
            Self::ProxyRemove => "proxy_remove",
            Self::SudoSetTempo => "sudo_set_tempo",
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
        eprintln!("[parity_governance] Docker unavailable; skipping.");
        return false;
    }
    if !btcli_available() {
        eprintln!("[parity_governance] btcli unavailable in .venv; skipping.");
        return false;
    }
    true
}

fn acquire_category_lock() -> Result<File> {
    let lock_path = "/tmp/agcli_parity_governance.lock";
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
        String::from_utf8_lossy(&output.stderr)
    );
}

fn parse_stdout_json(output: &Output, label: &str) -> Result<Value> {
    if let Ok(value) = serde_json::from_slice::<Value>(&output.stdout) {
        return Ok(value);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            return Ok(value);
        }
    }
    anyhow::bail!("{label} emitted non-JSON stdout:\n{stdout}");
}

fn parse_stderr_json_error(output: &Output, label: &str) -> Result<Value> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if let Ok(value) = serde_json::from_str::<Value>(&stderr) {
        return validate_error_json(value, label);
    }
    for line in stderr.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            return validate_error_json(value, label);
        }
    }
    anyhow::bail!("{label} stderr is not structured JSON:\n{stderr}");
}

fn validate_error_json(value: Value, label: &str) -> Result<Value> {
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

fn build_btcli_script(op: GovernanceOperation, ctx: &ScenarioContext) -> String {
    let wallet_path = shell_quote(&ctx.wallet_dir_path);
    let endpoint = shell_quote(&ctx.endpoint);
    match op {
        GovernanceOperation::ProxyAdd => format!(
            "source .venv/bin/activate && btcli proxy add --wallet-name {} --wallet-path {} --wallet-hotkey default --delegate {} --proxy-type Any --delay 0 --network {} --no-prompt --json-output",
            shell_quote(&ctx.wallet_name),
            wallet_path,
            shell_quote(&ctx.delegate_ss58),
            endpoint
        ),
        GovernanceOperation::ProxyRemove => format!(
            "source .venv/bin/activate && btcli proxy remove --wallet-name {} --wallet-path {} --wallet-hotkey default --delegate {} --proxy-type Any --delay 0 --network {} --no-prompt --json-output",
            shell_quote(&ctx.wallet_name),
            wallet_path,
            shell_quote(&ctx.delegate_ss58),
            endpoint
        ),
        GovernanceOperation::SudoSetTempo => format!(
            "source .venv/bin/activate && btcli sudo set --wallet-name {} --wallet-path {} --wallet-hotkey default --network {} --netuid {} --param tempo --value {} --no-prompt --json-output",
            shell_quote(&ctx.alice_wallet_name),
            wallet_path,
            endpoint,
            ctx.netuid,
            ctx.target_tempo
        ),
    }
}

fn build_sdk_script(op: GovernanceOperation) -> Option<&'static str> {
    match op {
        GovernanceOperation::ProxyAdd => Some(
            r#"
import os
import bittensor

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.add_proxy(
    wallet=wallet,
    delegate_ss58=os.environ["DELEGATE_SS58"],
    proxy_type="Any",
    delay=0,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#,
        ),
        GovernanceOperation::ProxyRemove => Some(
            r#"
import os
import bittensor

wallet = bittensor.wallet(name=os.environ["WALLET_NAME"], hotkey="default", path=os.environ["WALLET_DIR"])
subtensor = bittensor.subtensor(network=os.environ["ENDPOINT"])
subtensor.remove_proxy(
    wallet=wallet,
    delegate_ss58=os.environ["DELEGATE_SS58"],
    proxy_type="Any",
    delay=0,
    wait_for_inclusion=True,
    wait_for_finalization=True,
)
print("{\"ok\": true}")
"#,
        ),
        GovernanceOperation::SudoSetTempo => None,
    }
}

fn build_agcli_success_args(op: GovernanceOperation, ctx: &ScenarioContext) -> Vec<String> {
    let mut args = vec!["--endpoint".to_string(), ctx.endpoint.clone()];
    match op {
        GovernanceOperation::ProxyAdd | GovernanceOperation::ProxyRemove => {
            args.extend([
                "--wallet-dir".to_string(),
                ctx.wallet_dir_path.clone(),
                "--wallet".to_string(),
                ctx.wallet_name.clone(),
                "--hotkey".to_string(),
                "default".to_string(),
                "--password".to_string(),
                PASSWORD.to_string(),
                "--yes".to_string(),
                "proxy".to_string(),
                if matches!(op, GovernanceOperation::ProxyAdd) {
                    "add".to_string()
                } else {
                    "remove".to_string()
                },
                "--delegate".to_string(),
                ctx.delegate_ss58.clone(),
                "--proxy-type".to_string(),
                "any".to_string(),
                "--delay".to_string(),
                "0".to_string(),
            ]);
        }
        GovernanceOperation::SudoSetTempo => {
            args.extend([
                "--yes".to_string(),
                "admin".to_string(),
                "raw".to_string(),
                "--call".to_string(),
                "sudo_set_tempo".to_string(),
                "--args".to_string(),
                format!("[{},{}]", ctx.netuid, ctx.target_tempo),
                "--sudo-key".to_string(),
                "//Alice".to_string(),
            ]);
        }
    }
    args
}

fn build_agcli_dry_run_args(op: GovernanceOperation, ctx: &ScenarioContext) -> Vec<String> {
    let mut args = vec![
        "--endpoint".to_string(),
        ctx.endpoint.clone(),
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
        "--dry-run".to_string(),
    ];
    match op {
        GovernanceOperation::ProxyAdd | GovernanceOperation::ProxyRemove => {
            args.extend([
                "--wallet-dir".to_string(),
                ctx.wallet_dir_path.clone(),
                "--wallet".to_string(),
                ctx.wallet_name.clone(),
                "--hotkey".to_string(),
                "default".to_string(),
                "--password".to_string(),
                PASSWORD.to_string(),
                "proxy".to_string(),
                if matches!(op, GovernanceOperation::ProxyAdd) {
                    "add".to_string()
                } else {
                    "remove".to_string()
                },
                "--delegate".to_string(),
                ctx.delegate_ss58.clone(),
                "--proxy-type".to_string(),
                "any".to_string(),
                "--delay".to_string(),
                "0".to_string(),
            ]);
        }
        GovernanceOperation::SudoSetTempo => {
            args.extend([
                "admin".to_string(),
                "raw".to_string(),
                "--call".to_string(),
                "sudo_set_tempo".to_string(),
                "--args".to_string(),
                format!("[{},{}]", ctx.netuid, ctx.target_tempo),
                "--sudo-key".to_string(),
                "//Alice".to_string(),
            ]);
        }
    }
    args
}

fn build_agcli_invalid_args(op: GovernanceOperation, ctx: &ScenarioContext) -> Vec<String> {
    let mut args = vec![
        "--endpoint".to_string(),
        ctx.endpoint.clone(),
        "--batch".to_string(),
        "--output".to_string(),
        "json".to_string(),
    ];
    match op {
        GovernanceOperation::ProxyAdd | GovernanceOperation::ProxyRemove => {
            args.extend([
                "proxy".to_string(),
                if matches!(op, GovernanceOperation::ProxyAdd) {
                    "add".to_string()
                } else {
                    "remove".to_string()
                },
                "--delegate".to_string(),
                "not-an-ss58".to_string(),
            ]);
        }
        GovernanceOperation::SudoSetTempo => {
            args.extend([
                "admin".to_string(),
                "raw".to_string(),
                "--call".to_string(),
                "sudo_set_tempo".to_string(),
                "--args".to_string(),
                "[0,100]".to_string(),
                "--sudo-key".to_string(),
                "//Alice".to_string(),
            ]);
        }
    }
    args
}

fn assert_agcli_ux_claims(op: GovernanceOperation, ctx: &ScenarioContext) -> Result<()> {
    let mut dry_cmd = Command::new(agcli_bin());
    dry_cmd.args(build_agcli_dry_run_args(op, ctx));
    let dry_out = run_cmd(dry_cmd, "agcli dry-run command")?;
    assert_success(&dry_out, "agcli dry-run command")?;
    let dry_json = parse_stdout_json(&dry_out, "agcli dry-run command")?;
    if !(dry_json.is_object() || dry_json.is_array()) {
        anyhow::bail!("agcli dry-run output must be JSON object/array, got: {dry_json}");
    }

    let mut invalid_cmd = Command::new(agcli_bin());
    invalid_cmd.args(build_agcli_invalid_args(op, ctx));
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

fn assert_agcli_group_ux_claims(ctx: &ScenarioContext) -> Result<()> {
    let ux_cases: &[(&str, &[&str], &[&str])] = &[
        (
            "multisig",
            &[
                "--batch",
                "--output",
                "json",
                "--dry-run",
                "--endpoint",
                "__ENDPOINT__",
                "--wallet-dir",
                "__WALLET_DIR__",
                "--wallet",
                "__WALLET__",
                "--hotkey",
                "default",
                "--password",
                PASSWORD,
                "multisig",
                "submit",
                "--others",
                harness::BOB_SS58,
                "--threshold",
                "1",
                "--pallet",
                "System",
                "--call",
                "remark",
                "--args",
                "[\"0x676f762d706172697479\"]",
            ],
            &[
                "--batch",
                "--output",
                "json",
                "--endpoint",
                "__ENDPOINT__",
                "multisig",
                "submit",
                "--others",
                "bad-ss58",
                "--threshold",
                "1",
                "--pallet",
                "System",
                "--call",
                "remark",
            ],
        ),
        (
            "scheduler",
            &[
                "--batch",
                "--output",
                "json",
                "--dry-run",
                "--endpoint",
                "__ENDPOINT__",
                "--wallet-dir",
                "__WALLET_DIR__",
                "--wallet",
                "__WALLET__",
                "--hotkey",
                "default",
                "--password",
                PASSWORD,
                "scheduler",
                "schedule",
                "--when",
                "500000",
                "--pallet",
                "System",
                "--call",
                "remark",
                "--args",
                "[\"0x676f762d7363686564\"]",
            ],
            &[
                "--batch",
                "--output",
                "json",
                "--endpoint",
                "__ENDPOINT__",
                "scheduler",
                "schedule",
                "--when",
                "500000",
                "--pallet",
                "System",
                "--call",
                "remark",
                "--repeat-every",
                "5",
            ],
        ),
        (
            "preimage",
            &[
                "--batch",
                "--output",
                "json",
                "--dry-run",
                "--endpoint",
                "__ENDPOINT__",
                "--wallet-dir",
                "__WALLET_DIR__",
                "--wallet",
                "__WALLET__",
                "--hotkey",
                "default",
                "--password",
                PASSWORD,
                "preimage",
                "note",
                "--pallet",
                "System",
                "--call",
                "remark",
                "--args",
                "[\"0x676f762d707265696d616765\"]",
            ],
            &[
                "--batch",
                "--output",
                "json",
                "--endpoint",
                "__ENDPOINT__",
                "preimage",
                "unnote",
                "--hash",
                "0x1234",
            ],
        ),
    ];

    for (label, dry_case, invalid_case) in ux_cases {
        let build = |parts: &[&str]| -> Vec<String> {
            parts
                .iter()
                .map(|p| match *p {
                    "__ENDPOINT__" => ctx.endpoint.clone(),
                    "__WALLET_DIR__" => ctx.wallet_dir_path.clone(),
                    "__WALLET__" => ctx.wallet_name.clone(),
                    _ => p.to_string(),
                })
                .collect()
        };

        let mut dry_cmd = Command::new(agcli_bin());
        dry_cmd.args(build(dry_case));
        let dry_out = run_cmd(dry_cmd, &format!("agcli {label} dry-run"))?;
        assert_success(&dry_out, &format!("agcli {label} dry-run"))?;
        let dry_json = parse_stdout_json(&dry_out, &format!("agcli {label} dry-run"))?;
        if !(dry_json.is_object() || dry_json.is_array()) {
            anyhow::bail!("agcli {label} dry-run output must be JSON object/array");
        }

        let mut invalid_cmd = Command::new(agcli_bin());
        invalid_cmd.args(build(invalid_case));
        let invalid_out = run_cmd(invalid_cmd, &format!("agcli {label} invalid"))?;
        if invalid_out.status.success() {
            anyhow::bail!("agcli {label} invalid-input command unexpectedly succeeded");
        }
        let _err_json = parse_stderr_json_error(&invalid_out, &format!("agcli {label} invalid"))?;
        if invalid_out.status.code() != Some(12) {
            anyhow::bail!(
                "agcli {label} invalid command expected exit code 12, got {:?}",
                invalid_out.status.code()
            );
        }
    }

    Ok(())
}

fn run_btcli_command(op: GovernanceOperation, ctx: &ScenarioContext) -> Result<()> {
    let script = build_btcli_script(op, ctx);
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    let out = run_cmd(cmd, "btcli governance command")?;
    assert_success(&out, "btcli governance command")
}

fn run_sdk_command(op: GovernanceOperation, ctx: &ScenarioContext) -> Result<()> {
    let Some(py) = build_sdk_script(op) else {
        anyhow::bail!("sdk command is not implemented for {}", op.slug());
    };
    let script = format!("source .venv/bin/activate && python - <<'PY'\n{py}\nPY");
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script])
        .env("WALLET_NAME", &ctx.wallet_name)
        .env("WALLET_DIR", &ctx.wallet_dir_path)
        .env("ENDPOINT", &ctx.endpoint)
        .env("DELEGATE_SS58", &ctx.delegate_ss58);
    let out = run_cmd(cmd, "bittensor sdk governance command")?;
    assert_success(&out, "bittensor sdk governance command")
}

fn run_agcli_success_command(op: GovernanceOperation, ctx: &ScenarioContext) -> Result<()> {
    let mut cmd = Command::new(agcli_bin());
    cmd.args(build_agcli_success_args(op, ctx));
    let out = run_cmd(cmd, "agcli governance command")?;
    assert_success(&out, "agcli governance command")?;
    if out.status.code() != Some(0) {
        anyhow::bail!(
            "agcli governance command expected exit code 0, got {:?}",
            out.status.code()
        );
    }
    Ok(())
}

async fn snapshot_state(client: &mut Client, ctx: &ScenarioContext) -> Result<ChainSnapshot> {
    harness::ensure_alive(client).await;
    let balance = client
        .get_balance_ss58(&ctx.coldkey_ss58)
        .await
        .with_context(|| format!("failed to query balance for {}", ctx.coldkey_ss58))?;
    let proxies = client
        .list_proxies(&ctx.coldkey_ss58)
        .await
        .with_context(|| format!("failed to list proxies for {}", ctx.coldkey_ss58))?;
    let tempo = client
        .get_subnet_hyperparams(NetUid(ctx.netuid))
        .await
        .with_context(|| {
            format!(
                "failed to query subnet hyperparams for netuid {}",
                ctx.netuid
            )
        })?
        .map(|h| h.tempo);
    Ok(ChainSnapshot {
        free_balance_rao: balance.rao(),
        proxies: proxies
            .into_iter()
            .map(|(delegate, proxy_type, delay)| (delegate, proxy_type, delay))
            .collect(),
        tempo,
    })
}

fn compute_delta(before: &ChainSnapshot, after: &ChainSnapshot) -> ChainDelta {
    let proxies_added = after
        .proxies
        .difference(&before.proxies)
        .cloned()
        .collect::<BTreeSet<_>>();
    let proxies_removed = before
        .proxies
        .difference(&after.proxies)
        .cloned()
        .collect::<BTreeSet<_>>();
    ChainDelta {
        free_balance_rao: after.free_balance_rao as i128 - before.free_balance_rao as i128,
        proxies_added,
        proxies_removed,
        tempo_before: before.tempo,
        tempo_after: after.tempo,
    }
}

async fn seed_state_for_operation(
    op: GovernanceOperation,
    client: &mut Client,
    ctx: &ScenarioContext,
) -> Result<()> {
    if matches!(op, GovernanceOperation::ProxyRemove) {
        let pair = harness::dev_pair(&ctx.seed_uri);
        client
            .add_proxy(&pair, &ctx.delegate_ss58, "any", 0)
            .await
            .context("failed to seed proxy for remove operation")?;
        harness::wait_blocks(client, 3).await;
    }
    Ok(())
}

async fn bootstrap_scenario(
    container_name: &str,
) -> Result<(ContainerGuard, Client, ScenarioContext)> {
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

    let mut alice_key_cmd = Command::new(agcli_bin());
    alice_key_cmd.args([
        "--output",
        "json",
        "--wallet-dir",
        &wallet_dir_path,
        "wallet",
        "dev-key",
        "--uri",
        harness::ALICE_URI,
        "--password",
        PASSWORD,
    ]);
    let alice_key_out = run_cmd(alice_key_cmd, "agcli wallet dev-key alice")?;
    assert_success(&alice_key_out, "agcli wallet dev-key alice")?;
    let alice_key_json = parse_stdout_json(&alice_key_out, "agcli wallet dev-key alice")?;
    let alice_dev_key: DevKeyResult = serde_json::from_value(alice_key_json)
        .context("failed to decode alice wallet dev-key JSON payload")?;

    let mut client = harness::wait_for_chain().await;
    harness::ensure_alive(&mut client).await;
    harness::wait_blocks(&mut client, 3).await;
    let tempo_now = client
        .get_subnet_hyperparams(NetUid(subnet.netuid))
        .await
        .context("failed querying subnet hyperparams during bootstrap")?
        .map(|h| h.tempo)
        .unwrap_or(100);

    Ok((
        guard,
        client,
        ScenarioContext {
            _wallet_dir: wallet_temp,
            wallet_dir_path,
            wallet_name: if dev_key.name.is_empty() {
                derive_wallet_name(&seed_uri)
            } else {
                dev_key.name
            },
            alice_wallet_name: if alice_dev_key.name.is_empty() {
                "alice".to_string()
            } else {
                alice_dev_key.name
            },
            seed_uri,
            endpoint: scaffold.endpoint,
            netuid: subnet.netuid,
            coldkey_ss58: dev_key.coldkey,
            delegate_ss58: DELEGATE_SS58.to_string(),
            target_tempo: tempo_now.saturating_add(1),
        },
    ))
}

fn supports_reference(op: GovernanceOperation, reference: Reference) -> bool {
    match (op, reference) {
        (GovernanceOperation::SudoSetTempo, Reference::Sdk) => false,
        _ => true,
    }
}

async fn execute_reference(
    op: GovernanceOperation,
    reference: Reference,
) -> Result<Option<ChainDelta>> {
    if !supports_reference(op, reference) {
        return Ok(None);
    }
    if matches!(reference, Reference::Sdk) && !sdk_available() {
        return Ok(None);
    }

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
    seed_state_for_operation(op, &mut client, &ctx).await?;

    let before = snapshot_state(&mut client, &ctx).await?;
    match reference {
        Reference::Btcli => run_btcli_command(op, &ctx)?,
        Reference::Sdk => run_sdk_command(op, &ctx)?,
        Reference::Agcli => {
            assert_agcli_ux_claims(op, &ctx)?;
            if matches!(op, GovernanceOperation::ProxyAdd) {
                assert_agcli_group_ux_claims(&ctx)?;
            }
            run_agcli_success_command(op, &ctx)?;
        }
    }

    harness::ensure_alive(&mut client).await;
    harness::wait_blocks(&mut client, 5).await;
    let after = snapshot_state(&mut client, &ctx).await?;
    Ok(Some(compute_delta(&before, &after)))
}

async fn assert_parity_for_operation(op: GovernanceOperation) -> Result<()> {
    let btcli_delta = execute_reference(op, Reference::Btcli).await?;
    let sdk_delta = execute_reference(op, Reference::Sdk).await?;
    let agcli_delta = execute_reference(op, Reference::Agcli).await?;

    let agcli_delta =
        agcli_delta.with_context(|| format!("agcli reference unavailable for {}", op.slug()))?;
    let btcli_delta =
        btcli_delta.with_context(|| format!("btcli reference unavailable for {}", op.slug()))?;

    if btcli_delta != agcli_delta {
        anyhow::bail!(
            "btcli delta diverged for {}:\n  btcli: {:?}\n  agcli: {:?}",
            op.slug(),
            btcli_delta,
            agcli_delta
        );
    }

    if let Some(sdk_delta) = sdk_delta {
        if sdk_delta != agcli_delta {
            anyhow::bail!(
                "sdk delta diverged for {}:\n  sdk: {:?}\n  agcli: {:?}",
                op.slug(),
                sdk_delta,
                agcli_delta
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn governance_proxy_add_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }
    assert_parity_for_operation(GovernanceOperation::ProxyAdd)
        .await
        .expect("proxy add parity must match");
}

#[tokio::test]
async fn governance_proxy_remove_parity_btcli_sdk_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }
    assert_parity_for_operation(GovernanceOperation::ProxyRemove)
        .await
        .expect("proxy remove parity must match");
}

#[tokio::test]
async fn governance_sudo_set_tempo_parity_btcli_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }
    assert_parity_for_operation(GovernanceOperation::SudoSetTempo)
        .await
        .expect("sudo set tempo parity must match");
}
