#![cfg(feature = "e2e")]

use super::harness::{self, ensure_alive, wait_blocks, Client, DOCKER_IMAGE};
use anyhow::{Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::process::{Command, Output};
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

const CATEGORY_CONTAINER_PREFIX: &str = "agcli_parity_root";
const PASSWORD: &str = "parity-pass-123";

#[derive(Debug, Clone, Deserialize)]
struct DevKeyResult {
    name: String,
    hotkey: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct IdentityFocus {
    subnet_name: String,
    github_repo: String,
    subnet_url: String,
}

#[derive(Debug, Clone)]
struct Snapshot {
    root_neuron_count: usize,
    root_hotkeys: BTreeSet<String>,
    root_identity: Option<IdentityFocus>,
    root_weights: BTreeMap<u16, BTreeMap<u16, u16>>,
    bob_balance_rao: u64,
}

#[derive(Debug)]
struct ScenarioContext {
    _wallet_dir: TempDir,
    wallet_dir_path: String,
    endpoint: String,
    root_wallet_name: String,
    root_hotkey_ss58: String,
    alice_wallet_name: String,
}

#[derive(Debug, Clone, Copy)]
enum RootOperation {
    Register,
    Identity,
    Weights,
}

#[derive(Debug, Clone, Copy)]
enum Reference {
    Btcli,
    Agcli,
}

#[derive(Debug)]
struct RunResult {
    before: Snapshot,
    after: Snapshot,
    output: Output,
    ctx: ScenarioContext,
}

impl RootOperation {
    fn slug(self) -> &'static str {
        match self {
            Self::Register => "register",
            Self::Identity => "identity",
            Self::Weights => "weights",
        }
    }
}

impl Reference {
    fn slug(self) -> &'static str {
        match self {
            Self::Btcli => "btcli",
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
            .output();
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn agcli_bin() -> String {
    std::env::var("CARGO_BIN_EXE_agcli")
        .unwrap_or_else(|_| format!("{}/target/debug/agcli", env!("CARGO_MANIFEST_DIR")))
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

fn require_prerequisites() -> bool {
    if !docker_available() {
        eprintln!("[parity_root] Docker unavailable; skipping.");
        return false;
    }
    if !btcli_available() {
        eprintln!("[parity_root] btcli unavailable in .venv; skipping.");
        return false;
    }
    true
}

fn acquire_category_lock() -> Result<File> {
    let lock_path = "/tmp/agcli_parity_root.lock";
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

fn parse_stdout_json<T: for<'de> serde::Deserialize<'de>>(output: &Output, label: &str) -> Result<T> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<T>(trimmed) {
            return Ok(value);
        }
    }
    serde_json::from_slice::<T>(&output.stdout).with_context(|| {
        format!(
            "{label} emitted non-JSON stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
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

fn start_container(container: &str) -> Result<()> {
    cleanup_port_and_container(container);
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
            DOCKER_IMAGE,
        ])
        .output()
        .context("failed to run docker container for parity-root")?;
    if !out.status.success() {
        anyhow::bail!(
            "failed to start docker container {container}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

async fn snapshot_state(client: &mut Client, ctx: &ScenarioContext) -> Result<Snapshot> {
    ensure_alive(client).await;
    let root_neurons = client
        .get_neurons_lite(agcli::types::NetUid::ROOT)
        .await
        .context("failed to query root neurons")?;
    let root_hotkeys = root_neurons
        .iter()
        .map(|n| n.hotkey.clone())
        .collect::<BTreeSet<_>>();
    let root_identity = client
        .get_subnet_identity(agcli::types::NetUid::ROOT)
        .await
        .context("failed to query root subnet identity")?
        .map(|id| IdentityFocus {
            subnet_name: id.subnet_name,
            github_repo: id.github_repo,
            subnet_url: id.subnet_url,
        });
    let mut root_weights = BTreeMap::<u16, BTreeMap<u16, u16>>::new();
    for (uid, entries) in client
        .get_all_weights(agcli::types::NetUid::ROOT)
        .await
        .context("failed to query root weights")?
    {
        root_weights.insert(uid, entries.into_iter().collect());
    }
    let bob_balance_rao = client
        .get_balance_ss58(&ctx.root_hotkey_ss58)
        .await
        .context("failed to query root hotkey balance")?
        .rao();
    Ok(Snapshot {
        root_neuron_count: root_hotkeys.len(),
        root_hotkeys,
        root_identity,
        root_weights,
        bob_balance_rao,
    })
}

fn run_btcli_root_register(ctx: &ScenarioContext) -> Result<Output> {
    let script = format!(
        "source .venv/bin/activate && timeout 90s btcli subnets register --wallet-name {} --wallet-path {} --hotkey default --network {} --netuid 0 --no-prompt --json-output",
        shell_quote(&ctx.root_wallet_name),
        shell_quote(&ctx.wallet_dir_path),
        shell_quote(&ctx.endpoint)
    );
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    run_cmd(cmd, "btcli root-register equivalent")
}

fn run_agcli_root_register(ctx: &ScenarioContext) -> Result<Output> {
    let mut cmd = Command::new(agcli_bin());
    cmd.args([
        "--endpoint",
        &ctx.endpoint,
        "--wallet-dir",
        &ctx.wallet_dir_path,
        "--wallet",
        &ctx.root_wallet_name,
        "--hotkey",
        "default",
        "--password",
        PASSWORD,
        "--yes",
        "root",
        "register",
    ]);
    run_cmd(cmd, "agcli root register")
}

fn run_btcli_root_identity(ctx: &ScenarioContext) -> Result<Output> {
    let script = format!(
        "source .venv/bin/activate && timeout 90s btcli subnets set-identity --wallet-name {} --wallet-path {} --hotkey default --network {} --netuid 0 --subnet-name parity-root-sn --github-repo https://github.com/opentensor/parity-root --subnet-contact root@example.com --subnet-url https://parity-root.example --discord-handle parity-root --description parity-root --logo-url https://parity-root.example/logo.png --additional-info parity-root --no-prompt --json-output",
        shell_quote(&ctx.alice_wallet_name),
        shell_quote(&ctx.wallet_dir_path),
        shell_quote(&ctx.endpoint),
    );
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    run_cmd(cmd, "btcli root-identity equivalent")
}

fn run_agcli_root_identity(ctx: &ScenarioContext) -> Result<Output> {
    let mut cmd = Command::new(agcli_bin());
    cmd.args([
        "--endpoint",
        &ctx.endpoint,
        "--wallet-dir",
        &ctx.wallet_dir_path,
        "--wallet",
        &ctx.alice_wallet_name,
        "--hotkey",
        "default",
        "--password",
        PASSWORD,
        "--yes",
        "identity",
        "set-subnet",
        "--netuid",
        "0",
        "--name",
        "parity-root-sn",
        "--github",
        "opentensor/parity-root",
        "--url",
        "https://parity-root.example",
    ]);
    run_cmd(cmd, "agcli root identity")
}

fn run_btcli_root_weights(ctx: &ScenarioContext) -> Result<Output> {
    let script = format!(
        "source .venv/bin/activate && timeout 90s btcli weights commit --wallet-name {} --wallet-path {} --hotkey default --network {} --netuid 0 --uids 1 --weights 1.0 --salt 1,2,3 --no-prompt --json-output",
        shell_quote(&ctx.root_wallet_name),
        shell_quote(&ctx.wallet_dir_path),
        shell_quote(&ctx.endpoint)
    );
    let mut cmd = Command::new("bash");
    cmd.args(["-lc", &script]);
    run_cmd(cmd, "btcli root-weights equivalent")
}

fn run_agcli_root_weights(ctx: &ScenarioContext) -> Result<Output> {
    let mut cmd = Command::new(agcli_bin());
    cmd.args([
        "--endpoint",
        &ctx.endpoint,
        "--wallet-dir",
        &ctx.wallet_dir_path,
        "--wallet",
        &ctx.root_wallet_name,
        "--hotkey",
        "default",
        "--password",
        PASSWORD,
        "--yes",
        "root",
        "weights",
        "--weights",
        "1:65535",
    ]);
    run_cmd(cmd, "agcli root weights")
}

async fn bootstrap_scenario(container_name: &str) -> Result<(ContainerGuard, Client, ScenarioContext)> {
    start_container(container_name)?;
    let guard = ContainerGuard {
        name: container_name.to_string(),
    };

    let mut client = timeout(Duration::from_secs(120), harness::wait_for_chain())
        .await
        .context("timed out waiting for local chain connection")?;
    ensure_alive(&mut client).await;
    timeout(Duration::from_secs(90), wait_blocks(&mut client, 4))
        .await
        .context("timed out waiting for startup blocks")?;

    let wallet_temp = TempDir::new().context("failed creating temporary wallet directory")?;
    let wallet_dir_path = wallet_temp.path().to_string_lossy().to_string();

    let mut root_cmd = Command::new(agcli_bin());
    root_cmd.args([
        "--output",
        "json",
        "--wallet-dir",
        &wallet_dir_path,
        "wallet",
        "dev-key",
        "--uri",
        "//Bob",
        "--password",
        PASSWORD,
    ]);
    let root_out = run_cmd(root_cmd, "agcli wallet dev-key root key")?;
    assert_success(&root_out, "agcli wallet dev-key root key")?;
    let root_key: DevKeyResult = parse_stdout_json(&root_out, "agcli wallet dev-key root key")?;

    let mut alice_cmd = Command::new(agcli_bin());
    alice_cmd.args([
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
    let alice_out = run_cmd(alice_cmd, "agcli wallet dev-key Alice")?;
    assert_success(&alice_out, "agcli wallet dev-key Alice")?;
    let alice_key: DevKeyResult = parse_stdout_json(&alice_out, "agcli wallet dev-key Alice")?;

    Ok((
        guard,
        client,
        ScenarioContext {
            _wallet_dir: wallet_temp,
            wallet_dir_path,
            endpoint: "ws://127.0.0.1:9944".to_string(),
            root_wallet_name: if root_key.name.is_empty() {
                "bob".to_string()
            } else {
                root_key.name
            },
            root_hotkey_ss58: root_key.hotkey,
            alice_wallet_name: if alice_key.name.is_empty() {
                "alice".to_string()
            } else {
                alice_key.name
            },
        },
    ))
}

fn added_hotkeys(before: &Snapshot, after: &Snapshot) -> BTreeSet<String> {
    after
        .root_hotkeys
        .difference(&before.root_hotkeys)
        .cloned()
        .collect()
}

fn neuron_delta(before: &Snapshot, after: &Snapshot) -> i64 {
    after.root_neuron_count as i64 - before.root_neuron_count as i64
}

fn normalize_identity(identity: Option<&IdentityFocus>) -> Option<IdentityFocus> {
    identity.map(|id| IdentityFocus {
        subnet_name: id.subnet_name.clone(),
        github_repo: id
            .github_repo
            .strip_prefix("https://github.com/")
            .unwrap_or(&id.github_repo)
            .trim_end_matches('/')
            .to_string(),
        subnet_url: id.subnet_url.clone(),
    })
}

async fn run_reference(op: RootOperation, reference: Reference) -> Result<RunResult> {
    let container_name = format!("{CATEGORY_CONTAINER_PREFIX}_{}_{}", reference.slug(), op.slug());
    let (_guard, mut client, ctx) = bootstrap_scenario(&container_name).await?;

    if matches!(op, RootOperation::Weights) {
        let register_out = run_agcli_root_register(&ctx)?;
        assert_success(&register_out, "agcli root register (weights seed)")?;
        timeout(Duration::from_secs(90), wait_blocks(&mut client, 3))
            .await
            .context("timed out waiting after weights seed register")?;
    }

    let before = timeout(Duration::from_secs(120), snapshot_state(&mut client, &ctx))
        .await
        .context("timed out taking pre-operation snapshot")??;
    let output = match (reference, op) {
        (Reference::Btcli, RootOperation::Register) => run_btcli_root_register(&ctx)?,
        (Reference::Btcli, RootOperation::Identity) => run_btcli_root_identity(&ctx)?,
        (Reference::Btcli, RootOperation::Weights) => run_btcli_root_weights(&ctx)?,
        (Reference::Agcli, RootOperation::Register) => run_agcli_root_register(&ctx)?,
        (Reference::Agcli, RootOperation::Identity) => run_agcli_root_identity(&ctx)?,
        (Reference::Agcli, RootOperation::Weights) => run_agcli_root_weights(&ctx)?,
    };
    harness::ensure_alive(&mut client).await;
    timeout(Duration::from_secs(90), wait_blocks(&mut client, 4))
        .await
        .context("timed out waiting for post-operation finalization blocks")?;
    let after = timeout(Duration::from_secs(120), snapshot_state(&mut client, &ctx))
        .await
        .context("timed out taking post-operation snapshot")??;
    Ok(RunResult {
        before,
        after,
        output,
        ctx,
    })
}

#[tokio::test]
async fn root_register_parity_btcli_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }

    let btcli = run_reference(RootOperation::Register, Reference::Btcli)
        .await
        .expect("btcli register scenario must run");
    assert_success(&btcli.output, "btcli root-register equivalent")
        .expect("btcli root-register should succeed");

    let agcli = run_reference(RootOperation::Register, Reference::Agcli)
        .await
        .expect("agcli register scenario must run");
    assert_success(&agcli.output, "agcli root register").expect("agcli root register should succeed");

    assert_eq!(
        neuron_delta(&btcli.before, &btcli.after),
        neuron_delta(&agcli.before, &agcli.after),
        "root register neuron-count delta diverged between btcli and agcli"
    );
    assert_eq!(
        added_hotkeys(&btcli.before, &btcli.after),
        added_hotkeys(&agcli.before, &agcli.after),
        "root register hotkey-set delta diverged between btcli and agcli"
    );
    let btcli_registered = btcli.after.root_hotkeys.contains(&btcli.ctx.root_hotkey_ss58);
    let agcli_registered = agcli.after.root_hotkeys.contains(&agcli.ctx.root_hotkey_ss58);
    assert_eq!(
        btcli_registered, agcli_registered,
        "root register final registration state diverged between btcli and agcli"
    );
}

#[tokio::test]
async fn root_identity_parity_btcli_agcli() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }

    let btcli = run_reference(RootOperation::Identity, Reference::Btcli)
        .await
        .expect("btcli identity scenario must run");
    assert_success(&btcli.output, "btcli identity equivalent")
        .expect("btcli identity should exit 0");

    let agcli = run_reference(RootOperation::Identity, Reference::Agcli)
        .await
        .expect("agcli identity scenario must run");
    assert_success(&agcli.output, "agcli identity command").expect("agcli identity should succeed");

    let btcli_changed = btcli.before.root_identity != btcli.after.root_identity;
    let agcli_changed = agcli.before.root_identity != agcli.after.root_identity;
    assert!(
        !btcli_changed,
        "btcli root identity path unexpectedly mutated root identity"
    );
    assert!(
        agcli_changed,
        "agcli root identity path should mutate root identity on baseline localnet"
    );

    let agcli_identity = normalize_identity(agcli.after.root_identity.as_ref())
        .expect("agcli should leave a root identity record");
    assert_eq!(agcli_identity.subnet_name, "parity-root-sn");
    assert_eq!(agcli_identity.github_repo, "opentensor/parity-root");
    assert_eq!(agcli_identity.subnet_url, "https://parity-root.example");
}

#[tokio::test]
async fn root_weights_path_btcli_vs_agcli_commit_reveal() {
    let _lock = acquire_category_lock().expect("category lock acquisition must succeed");
    if !require_prerequisites() {
        return;
    }

    let btcli = run_reference(RootOperation::Weights, Reference::Btcli)
        .await
        .expect("btcli weights scenario must run");
    assert_success(&btcli.output, "btcli root-weights equivalent")
        .expect("btcli weights commit should exit 0");
    assert_eq!(
        btcli.before.root_weights, btcli.after.root_weights,
        "btcli root-weights path should not mutate root weights in commit step"
    );

    let agcli = run_reference(RootOperation::Weights, Reference::Agcli)
        .await
        .expect("agcli weights scenario must run");
    assert!(
        !agcli.output.status.success(),
        "agcli root weights should fail on localnet with commit-reveal gating"
    );
    assert_eq!(
        agcli.before.root_weights, agcli.after.root_weights,
        "agcli root-weights rejection should keep root weights unchanged"
    );
    assert!(
        String::from_utf8_lossy(&agcli.output.stderr).contains("CommitRevealEnabled"),
        "agcli root weights failure should explain commit-reveal gating"
    );
    assert!(
        agcli.before.bob_balance_rao >= agcli.after.bob_balance_rao,
        "root hotkey balance should not increase during rejected root-weights flow"
    );
}
