#![cfg(feature = "e2e")]

use super::harness;
use agcli::chain::Client;
use anyhow::{Context, Result};
use fs2::FileExt;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::process::{Command, Output};
use std::time::Duration;
use tempfile::TempDir;

const CATEGORY_CONTAINER: &str = "agcli_parity_network_readonly";
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ChainSnapshot {
    alice_rao: u64,
    bob_rao: u64,
    subnet_count: usize,
}

#[derive(Debug)]
struct BtcliArtifacts {
    netuids: BTreeSet<u16>,
    total_netuids: usize,
    show_netuid: u16,
    show_name: String,
    price_netuids: BTreeSet<u16>,
    dashboard_html_files: usize,
}

#[derive(Debug)]
struct AgcliArtifacts {
    netuids: BTreeSet<u16>,
    show_netuid: u16,
    show_name: String,
    network_subnets: usize,
}

#[derive(Debug, Deserialize)]
struct SdkModeArtifacts {
    total_subnets: usize,
    all_subnets_netuid: Vec<u16>,
    all_subnets_info_count: usize,
    subnet_info_netuid: u16,
    subnet_exists: bool,
}

#[derive(Debug, Deserialize)]
struct SdkArtifacts {
    sync: SdkModeArtifacts,
    #[serde(rename = "async")]
    async_mode: SdkModeArtifacts,
}

#[derive(Debug, Deserialize)]
struct ScaffoldOutput {
    endpoint: String,
    subnets: Vec<ScaffoldSubnet>,
}

#[derive(Debug, Deserialize)]
struct ScaffoldSubnet {
    netuid: u16,
}

struct ContainerGuard {
    name: String,
}

impl Drop for ContainerGuard {
    fn drop(&mut self) {
        cleanup_chain(&self.name);
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn agcli_bin() -> String {
    std::env::var("CARGO_BIN_EXE_agcli").unwrap_or_else(|_| "target/debug/agcli".to_string())
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

fn require_prerequisites() -> bool {
    if !docker_available() {
        eprintln!("[parity_network_readonly] Docker unavailable; skipping.");
        return false;
    }
    if !btcli_available() {
        eprintln!("[parity_network_readonly] btcli unavailable in .venv; skipping.");
        return false;
    }
    if !sdk_available() {
        eprintln!("[parity_network_readonly] bittensor SDK unavailable in .venv; skipping.");
        return false;
    }
    true
}

fn acquire_category_lock() -> Result<File> {
    let lock_path = "/tmp/agcli_parity_network_readonly.lock";
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
    cmd.output().with_context(|| format!("failed to run {label}"))
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_success(output: &Output, label: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    anyhow::bail!(
        "{label} failed (code: {:?})\nstdout+stderr:\n{}",
        output.status.code(),
        output_text(output),
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

fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>()
}

fn cleanup_chain(container_name: &str) {
    let _ = Command::new("docker")
        .args(["rm", "-f", container_name])
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

fn write_variant_a_config(temp_dir: &TempDir, container_name: &str) -> Result<String> {
    let scaffold_path = temp_dir.path().join("scaffold-A-baseline.toml");
    let content = VARIANT_A_CONFIG.replace("__CONTAINER__", container_name);
    fs::write(&scaffold_path, content)
        .with_context(|| format!("failed writing scaffold config {}", scaffold_path.display()))?;
    Ok(scaffold_path.to_string_lossy().to_string())
}

fn parse_netuid_set_from_btcli_list(list_json: &Value) -> Result<BTreeSet<u16>> {
    let subnets = list_json
        .get("subnets")
        .and_then(Value::as_object)
        .context("btcli subnets list payload missing subnets object")?;
    let mut netuids = BTreeSet::new();
    for (key, entry) in subnets {
        let netuid = entry
            .get("netuid")
            .and_then(Value::as_u64)
            .or_else(|| key.parse::<u64>().ok())
            .context("btcli subnets list entry missing netuid")?;
        netuids.insert(u16::try_from(netuid).context("btcli netuid overflow")?);
    }
    Ok(netuids)
}

fn parse_netuid_set_from_agcli_list(list_json: &Value) -> Result<BTreeSet<u16>> {
    let arr = list_json
        .as_array()
        .context("agcli subnet list payload must be an array")?;
    let mut netuids = BTreeSet::new();
    for row in arr {
        let netuid = row
            .get("netuid")
            .and_then(Value::as_u64)
            .context("agcli subnet list row missing netuid")?;
        netuids.insert(u16::try_from(netuid).context("agcli netuid overflow")?);
    }
    Ok(netuids)
}

fn run_btcli_args(args: &[&str], label: &str) -> Result<Output> {
    let mut script = String::from("source .venv/bin/activate && btcli");
    for arg in args {
        script.push(' ');
        script.push_str(&shell_quote(arg));
    }
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    run_cmd(cmd, label)
}

fn run_agcli_args(args: &[&str], label: &str) -> Result<Output> {
    let mut cmd = Command::new(agcli_bin());
    cmd.args(args);
    run_cmd(cmd, label)
}

fn run_sdk_reads(endpoint: &str, netuid: u16) -> Result<SdkArtifacts> {
    let script = r#"
source .venv/bin/activate && python3 - <<'PY'
import asyncio
import json
import os
import bittensor

endpoint = os.environ["ENDPOINT"]
netuid = int(os.environ["NETUID"])

sync_subtensor = bittensor.Subtensor(network=endpoint)
sync_payload = {
    "total_subnets": int(sync_subtensor.get_total_subnets()),
    "all_subnets_netuid": [int(x) for x in sync_subtensor.get_all_subnets_netuid()],
    "all_subnets_info_count": len(sync_subtensor.get_all_subnets_info()),
    "subnet_info_netuid": int(sync_subtensor.get_subnet_info(netuid=netuid).netuid),
    "subnet_exists": bool(sync_subtensor.subnet_exists(netuid)),
}

async def run_async_payload():
    async_subtensor = bittensor.AsyncSubtensor(network=endpoint)
    await async_subtensor.initialize()
    try:
        total_subnets = int(await async_subtensor.get_total_subnets())
        all_subnets_netuid = [int(x) for x in await async_subtensor.get_all_subnets_netuid()]
        all_subnets_info_count = len(await async_subtensor.get_all_subnets_info())
        subnet_info_netuid = int((await async_subtensor.get_subnet_info(netuid=netuid)).netuid)
        subnet_exists = bool(await async_subtensor.subnet_exists(netuid))
        return {
            "total_subnets": total_subnets,
            "all_subnets_netuid": all_subnets_netuid,
            "all_subnets_info_count": all_subnets_info_count,
            "subnet_info_netuid": subnet_info_netuid,
            "subnet_exists": subnet_exists,
        }
    finally:
        await async_subtensor.close()

async_payload = asyncio.run(run_async_payload())
print(json.dumps({"sync": sync_payload, "async": async_payload}))
PY
"#;

    let mut cmd = Command::new("bash");
    cmd.args(["-lc", script])
        .env("ENDPOINT", endpoint)
        .env("NETUID", netuid.to_string());
    let output = run_cmd(cmd, "sdk network readonly reads")?;
    assert_success(&output, "sdk network readonly reads")?;
    let payload = parse_stdout_json(&output, "sdk network readonly reads")?;
    serde_json::from_value(payload).context("failed to decode SDK read payload")
}

fn run_btcli_reads(endpoint: &str, netuid: u16) -> Result<BtcliArtifacts> {
    let list_out = run_btcli_args(
        &[
            "subnets",
            "list",
            "--network",
            endpoint,
            "--json-output",
            "--quiet",
        ],
        "btcli subnets list",
    )?;
    assert_success(&list_out, "btcli subnets list")?;
    let list_json = parse_stdout_json(&list_out, "btcli subnets list")?;
    let netuids = parse_netuid_set_from_btcli_list(&list_json)?;
    let total_netuids = list_json
        .get("total_netuids")
        .and_then(Value::as_u64)
        .context("btcli subnets list missing total_netuids")?;

    let show_out = run_btcli_args(
        &[
            "subnets",
            "show",
            "--network",
            endpoint,
            "--netuid",
            &netuid.to_string(),
            "--mechid",
            "0",
            "--no-prompt",
            "--json-output",
            "--quiet",
        ],
        "btcli subnets show",
    )?;
    assert_success(&show_out, "btcli subnets show")?;
    let show_json = parse_stdout_json(&show_out, "btcli subnets show")?;
    let show_netuid = show_json
        .get("netuid")
        .and_then(Value::as_u64)
        .context("btcli subnets show missing netuid")?;
    let show_name = show_json
        .get("name")
        .and_then(Value::as_str)
        .context("btcli subnets show missing name")?
        .to_string();

    let identity_out = run_btcli_args(
        &[
            "subnets",
            "get-identity",
            "--network",
            endpoint,
            "--netuid",
            &netuid.to_string(),
            "--json-output",
            "--quiet",
        ],
        "btcli subnets get-identity",
    )?;
    assert_success(&identity_out, "btcli subnets get-identity")?;
    let identity_json = parse_stdout_json(&identity_out, "btcli subnets get-identity")?;
    if !identity_json.is_object() {
        anyhow::bail!("btcli subnets get-identity payload must be JSON object");
    }

    let price_out = run_btcli_args(
        &[
            "subnets",
            "price",
            "--network",
            endpoint,
            "--netuid",
            &netuid.to_string(),
            "--current",
            "--json-output",
            "--quiet",
        ],
        "btcli subnets price",
    )?;
    assert_success(&price_out, "btcli subnets price")?;
    let price_json = parse_stdout_json(&price_out, "btcli subnets price")?;
    let price_obj = price_json
        .as_object()
        .context("btcli subnets price payload must be object")?;
    let mut price_netuids = BTreeSet::new();
    for key in price_obj.keys() {
        let parsed = key.parse::<u16>().with_context(|| {
            format!("btcli subnets price payload contains non-netuid key: {key}")
        })?;
        price_netuids.insert(parsed);
    }

    let dashboard_dir = TempDir::new().context("failed to create btcli dashboard temp dir")?;
    let dashboard_script = format!(
        "source .venv/bin/activate && BROWSER=true btcli view dashboard --network {} --coldkey-ss58 {} --save-file --dashboard-path {} --quiet",
        shell_quote(endpoint),
        shell_quote(harness::ALICE_SS58),
        shell_quote(&dashboard_dir.path().to_string_lossy()),
    );
    let mut dashboard_cmd = Command::new("bash");
    dashboard_cmd.args(["-lc", &dashboard_script]);
    let dashboard_out = run_cmd(dashboard_cmd, "btcli view dashboard")?;
    assert_success(&dashboard_out, "btcli view dashboard")?;
    let dashboard_html_files = fs::read_dir(dashboard_dir.path())
        .context("failed to list btcli dashboard output directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("html"))
                .unwrap_or(false)
        })
        .count();
    if dashboard_html_files == 0 {
        anyhow::bail!("btcli view dashboard did not produce any HTML files");
    }

    Ok(BtcliArtifacts {
        netuids,
        total_netuids: usize::try_from(total_netuids).context("total_netuids overflow")?,
        show_netuid: u16::try_from(show_netuid).context("show netuid overflow")?,
        show_name,
        price_netuids,
        dashboard_html_files,
    })
}

fn run_agcli_reads(endpoint: &str, netuid: u16) -> Result<AgcliArtifacts> {
    let list_out = run_agcli_args(
        &[
            "--endpoint",
            endpoint,
            "--batch",
            "--output",
            "json",
            "subnet",
            "list",
        ],
        "agcli subnet list",
    )?;
    assert_success(&list_out, "agcli subnet list")?;
    let list_json = parse_stdout_json(&list_out, "agcli subnet list")?;
    let netuids = parse_netuid_set_from_agcli_list(&list_json)?;

    let show_out = run_agcli_args(
        &[
            "--endpoint",
            endpoint,
            "--batch",
            "--output",
            "json",
            "subnet",
            "show",
            "--netuid",
            &netuid.to_string(),
        ],
        "agcli subnet show",
    )?;
    assert_success(&show_out, "agcli subnet show")?;
    let show_json = parse_stdout_json(&show_out, "agcli subnet show")?;
    let show_netuid = show_json
        .get("netuid")
        .and_then(Value::as_u64)
        .context("agcli subnet show missing netuid")?;
    let show_name = show_json
        .get("name")
        .and_then(Value::as_str)
        .context("agcli subnet show missing name")?
        .to_string();

    let network_out = run_agcli_args(
        &[
            "--endpoint",
            endpoint,
            "--batch",
            "--output",
            "json",
            "view",
            "network",
        ],
        "agcli view network",
    )?;
    assert_success(&network_out, "agcli view network")?;
    let network_json = parse_stdout_json(&network_out, "agcli view network")?;
    let network_subnets = network_json
        .get("subnets")
        .and_then(Value::as_u64)
        .context("agcli view network missing subnets")?;

    Ok(AgcliArtifacts {
        netuids,
        show_netuid: u16::try_from(show_netuid).context("agcli show netuid overflow")?,
        show_name,
        network_subnets: usize::try_from(network_subnets).context("agcli subnets overflow")?,
    })
}

fn start_plain_chain(container_name: &str) -> Result<()> {
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
            harness::DOCKER_IMAGE,
        ])
        .output()
        .context("failed to start plain localnet container")?;
    assert_success(&output, "docker run localnet")?;
    std::thread::sleep(Duration::from_secs(1));
    Ok(())
}

async fn boot_variant_a_chain(container_name: &str) -> Result<(Client, String, u16)> {
    cleanup_chain(container_name);
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
    if scaffold_out.status.success() {
        let scaffold_json = parse_stdout_json(&scaffold_out, "agcli localnet scaffold")?;
        let scaffold: ScaffoldOutput = serde_json::from_value(scaffold_json)
            .context("failed to decode scaffold output payload")?;
        let netuid = scaffold
            .subnets
            .first()
            .map(|s| s.netuid)
            .unwrap_or(1);
        let mut client = harness::wait_for_chain().await;
        harness::ensure_alive(&mut client).await;
        return Ok((client, scaffold.endpoint, netuid));
    }

    cleanup_chain(container_name);
    start_plain_chain(container_name)?;
    let mut client = harness::wait_for_chain().await;
    harness::ensure_alive(&mut client).await;
    Ok((client, harness::LOCAL_WS.to_string(), 1))
}

async fn snapshot_chain_state(client: &mut Client) -> Result<ChainSnapshot> {
    harness::ensure_alive(client).await;
    let alice_rao = client
        .get_balance_ss58(harness::ALICE_SS58)
        .await
        .context("failed to query Alice balance")?
        .rao();
    let bob_rao = client
        .get_balance_ss58(harness::BOB_SS58)
        .await
        .context("failed to query Bob balance")?
        .rao();
    let subnet_count = client
        .get_all_subnets()
        .await
        .context("failed to query subnet list for snapshot")?
        .len();
    Ok(ChainSnapshot {
        alice_rao,
        bob_rao,
        subnet_count,
    })
}

#[tokio::test]
async fn network_readonly_parity_btcli_sdk_vs_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }

    let _guard = ContainerGuard {
        name: CATEGORY_CONTAINER.to_string(),
    };
    let (mut client, endpoint, netuid) = boot_variant_a_chain(CATEGORY_CONTAINER)
        .await
        .expect("variant A localnet bootstrap must succeed");
    harness::wait_blocks(&mut client, 3).await;

    let before = snapshot_chain_state(&mut client)
        .await
        .expect("baseline chain snapshot should succeed");

    let btcli = run_btcli_reads(&endpoint, netuid).expect("btcli readonly network reads should pass");
    let after_btcli = snapshot_chain_state(&mut client)
        .await
        .expect("post-btcli chain snapshot should succeed");

    let agcli = run_agcli_reads(&endpoint, netuid).expect("agcli readonly network reads should pass");
    let after_agcli = snapshot_chain_state(&mut client)
        .await
        .expect("post-agcli chain snapshot should succeed");

    let sdk = run_sdk_reads(&endpoint, netuid).expect("sdk readonly network reads should pass");
    let after_sdk = snapshot_chain_state(&mut client)
        .await
        .expect("post-sdk chain snapshot should succeed");

    assert_eq!(
        before, after_btcli,
        "btcli readonly commands changed chain state"
    );
    assert_eq!(
        after_btcli, after_agcli,
        "agcli readonly commands changed chain state"
    );
    assert_eq!(
        after_agcli, after_sdk,
        "sdk readonly commands changed chain state"
    );

    assert_eq!(
        btcli.netuids, agcli.netuids,
        "btcli and agcli subnet set mismatch"
    );
    assert_eq!(
        btcli.total_netuids, agcli.network_subnets,
        "btcli and agcli subnet counts mismatch"
    );
    assert_eq!(btcli.show_netuid, netuid);
    assert_eq!(agcli.show_netuid, netuid);
    assert_eq!(
        normalize_name(&btcli.show_name),
        normalize_name(&agcli.show_name),
        "btcli and agcli subnet names mismatch after normalization"
    );
    assert!(
        btcli.price_netuids.contains(&netuid),
        "btcli price output missing requested netuid {netuid}"
    );
    assert!(
        btcli.dashboard_html_files > 0,
        "btcli dashboard command did not emit HTML output"
    );

    let expected_subnets = agcli.netuids.len();
    let sdk_sync_netuids: BTreeSet<u16> = sdk.sync.all_subnets_netuid.iter().copied().collect();
    let sdk_async_netuids: BTreeSet<u16> = sdk.async_mode.all_subnets_netuid.iter().copied().collect();

    assert_eq!(sdk.sync.total_subnets, expected_subnets);
    assert_eq!(sdk.async_mode.total_subnets, expected_subnets);
    assert_eq!(sdk.sync.all_subnets_info_count, expected_subnets);
    assert_eq!(sdk.async_mode.all_subnets_info_count, expected_subnets);
    assert_eq!(sdk_sync_netuids, agcli.netuids);
    assert_eq!(sdk_async_netuids, agcli.netuids);
    assert_eq!(sdk.sync.subnet_info_netuid, netuid);
    assert_eq!(sdk.async_mode.subnet_info_netuid, netuid);
    assert_eq!(sdk.sync.subnet_exists, agcli.netuids.contains(&netuid));
    assert_eq!(sdk.async_mode.subnet_exists, agcli.netuids.contains(&netuid));
}
