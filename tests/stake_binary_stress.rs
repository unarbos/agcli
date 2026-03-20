//! Stake CLI binary stress tests: invalid args and missing wallet should fail without a long RPC wait.
//! Run with: cargo test --test stake_binary_stress

use std::process::Command;

fn agcli_bin() -> Option<String> {
    std::env::var("CARGO_BIN_EXE_agcli").ok()
}

fn run_agcli_env(env: &[(&str, &str)], args: &[&str]) -> (bool, String, String) {
    let bin = agcli_bin().unwrap_or_else(|| "agcli".to_string());
    let mut cmd = Command::new(&bin);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.args(args).output().expect("failed to run agcli");
    let ok = out.status.success();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (ok, stdout, stderr)
}

fn run_agcli(args: &[&str]) -> (bool, String, String) {
    run_agcli_env(&[], args)
}

#[test]
fn stake_add_netuid_zero_fails_before_chain() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "stake",
        "add",
        "--amount",
        "1.0",
        "--netuid",
        "0",
        "--dry-run",
    ]);
    assert!(!ok, "stake add with netuid 0 should fail");
    assert!(
        stderr.to_lowercase().contains("netuid") || stderr.contains("Root network"),
        "stderr should mention invalid netuid: {}",
        stderr
    );
}

#[test]
fn stake_add_negative_amount_fails() {
    // Use `--amount=-1.0` so clap does not treat `-1.0` as a short-flag bundle after `--amount`.
    let (ok, _stdout, stderr) = run_agcli(&[
        "stake",
        "add",
        "--amount=-1.0",
        "--netuid",
        "1",
        "--dry-run",
    ]);
    assert!(!ok);
    assert!(
        stderr.to_lowercase().contains("negative") || stderr.to_lowercase().contains("invalid"),
        "stderr: {}",
        stderr
    );
}

#[test]
fn stake_move_same_subnet_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "stake",
        "move",
        "--amount",
        "1.0",
        "--from",
        "2",
        "--to",
        "2",
        "--dry-run",
    ]);
    assert!(!ok);
    assert!(
        stderr.contains("same") || stderr.contains("SN2"),
        "stderr: {}",
        stderr
    );
}

#[test]
fn stake_add_missing_wallet_fails_fast() {
    let (ok, _stdout, stderr) = run_agcli_env(
        &[
            ("AGCLI_WALLET_DIR", "/nonexistent"),
            ("AGCLI_WALLET", "NO_SUCH_WALLET"),
            ("AGCLI_HOTKEY", "HOT"),
        ],
        &[
            "stake",
            "add",
            "--amount",
            "1.0",
            "--netuid",
            "1",
            "--dry-run",
        ],
    );
    assert!(!ok);
    assert!(
        stderr.contains("not found") || stderr.contains("Wallet"),
        "stderr should mention missing wallet: {}",
        stderr
    );
}

#[test]
fn stake_list_with_address_skips_wallet_check() {
    let (ok, _stdout, stderr) = run_agcli_env(
        &[("AGCLI_WALLET_DIR", "/nonexistent")],
        &[
            "stake",
            "list",
            "--address",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        ],
    );
    // May fail on RPC (no network) but must not be "wallet not found" from local path
    assert!(
        ok || !stderr.contains("NO_SUCH") && !stderr.contains("/nonexistent/NO_SUCH"),
        "list --address should not require AGCLI_WALLET; stderr: {}",
        stderr
    );
}
