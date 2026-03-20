//! Weights CLI binary stress tests: run agcli with bad/edge args and assert clean failure.
//! Run with: cargo test --test weights_binary_stress
//!
//! These tests invoke the agcli binary (no wallet or chain required for validation paths).

use std::process::Command;

fn agcli_bin() -> Option<String> {
    std::env::var("CARGO_BIN_EXE_agcli").ok()
}

/// Run agcli with args; return (success, stdout, stderr).
fn run_agcli(args: &[&str]) -> (bool, String, String) {
    let bin = agcli_bin().unwrap_or_else(|| "agcli".to_string());
    let out = Command::new(&bin)
        .args(args)
        .output()
        .expect("failed to run agcli");
    let ok = out.status.success();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    (ok, stdout, stderr)
}

#[test]
fn weights_set_empty_weights_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "",
        "--dry-run",
    ]);
    assert!(!ok, "weights set with empty --weights should fail");
    assert!(
        stderr.to_lowercase().contains("empty") || stderr.to_lowercase().contains("weight"),
        "stderr should mention empty or weight: {}",
        stderr
    );
}

#[test]
fn weights_set_trailing_comma_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100,",
        "--dry-run",
    ]);
    assert!(!ok, "weights set with trailing comma should fail");
    assert!(
        stderr.contains("pair") || stderr.contains("comma") || stderr.contains("empty"),
        "stderr should mention pair/comma/empty: {}",
        stderr
    );
}

#[test]
fn weights_set_no_colon_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0100",
        "--dry-run",
    ]);
    assert!(!ok, "weights set without colon should fail");
    assert!(
        stderr.contains(":") || stderr.contains("separator") || stderr.contains("pair"),
        "stderr should mention separator or pair: {}",
        stderr
    );
}

#[test]
fn weights_commit_empty_weights_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "commit",
        "--netuid",
        "1",
        "--weights",
        "",
    ]);
    assert!(!ok, "weights commit with empty --weights should fail");
    assert!(
        stderr.to_lowercase().contains("empty") || stderr.to_lowercase().contains("weight"),
        "stderr should mention empty or weight: {}",
        stderr
    );
}

#[test]
fn weights_set_netuid_zero_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "0",
        "--weights",
        "0:100",
        "--dry-run",
    ]);
    assert!(!ok, "weights set with netuid 0 should fail (root network not a user subnet)");
    assert!(
        stderr.to_lowercase().contains("netuid") || stderr.to_lowercase().contains("root"),
        "stderr should mention netuid or root: {}",
        stderr
    );
}

#[test]
fn weights_set_negative_weight_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:-1",
        "--dry-run",
    ]);
    assert!(!ok, "weights set with negative weight should fail");
    assert!(
        stderr.to_lowercase().contains("weight")
            || stderr.to_lowercase().contains("invalid")
            || stderr.contains("65535"),
        "stderr should mention weight/invalid/range: {}",
        stderr
    );
}

#[test]
fn weights_set_valid_pair_dry_run_exits_clean() {
    // With --dry-run and valid weights, we may fail at wallet/chain (no wallet), but must not panic.
    let (_ok, stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
        "--dry-run",
    ]);
    // Either success (if default wallet exists and chain reachable) or clean error (no wallet / connection).
    let combined = format!("{} {}", stdout, stderr);
    assert!(
        !combined.contains("panic") && !combined.contains("thread 'main' panicked"),
        "must not panic; combined: {}",
        combined
    );
}

#[test]
fn weights_set_from_file_at_path_exits_clean() {
    // Exercise resolve_weights(@path): agcli reads weights from a file.
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let path = tmp.path();
    let content = r#"[{"uid":0,"weight":100},{"uid":1,"weight":200}]"#;
    std::fs::write(path, content).expect("write temp file");
    let weights_arg = format!("@{}", path.display());
    let (_ok, stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        &weights_arg,
        "--dry-run",
    ]);
    let combined = format!("{} {}", stdout, stderr);
    assert!(
        !combined.contains("panic") && !combined.contains("thread 'main' panicked"),
        "weights set @file must not panic; combined: {}",
        combined
    );
}

#[test]
fn weights_reveal_with_empty_weights_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "reveal",
        "--netuid",
        "1",
        "--weights",
        "",
        "--salt",
        "somesalt32charsxxxxxxxxxxxxxxxx",
    ]);
    assert!(!ok, "weights reveal with empty --weights should fail");
    assert!(
        stderr.to_lowercase().contains("empty") || stderr.to_lowercase().contains("weight"),
        "stderr should mention empty or weight: {}",
        stderr
    );
}

#[test]
fn weights_commit_valid_args_exits_clean() {
    // Commit with valid weights and salt; will fail at wallet/chain but must not panic.
    let (_ok, stdout, stderr) = run_agcli(&[
        "weights",
        "commit",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
        "--salt",
        "somesalt32charsxxxxxxxxxxxxxxxx",
    ]);
    let combined = format!("{} {}", stdout, stderr);
    assert!(
        !combined.contains("panic") && !combined.contains("thread 'main' panicked"),
        "weights commit with valid args must not panic; combined: {}",
        combined
    );
}

#[test]
fn weights_commit_reveal_empty_weights_fails() {
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "",
    ]);
    assert!(!ok, "weights commit-reveal with empty --weights should fail");
    assert!(
        stderr.to_lowercase().contains("empty") || stderr.to_lowercase().contains("weight"),
        "stderr should mention empty or weight: {}",
        stderr
    );
}

#[test]
fn weights_set_from_file_invalid_json_fails() {
    // @path with invalid JSON must fail cleanly (no panic).
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    std::fs::write(tmp.path(), b"not valid json {{{").expect("write");
    let weights_arg = format!("@{}", tmp.path().display());
    let (ok, stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        &weights_arg,
        "--dry-run",
    ]);
    assert!(!ok, "weights set @file with invalid JSON should fail");
    let combined = format!("{} {}", stdout, stderr);
    assert!(
        !combined.contains("panic") && !combined.contains("thread 'main' panicked"),
        "must not panic: {}",
        combined
    );
    assert!(
        combined.to_lowercase().contains("json") || combined.to_lowercase().contains("weight") || combined.to_lowercase().contains("invalid"),
        "stderr should mention json/weight/invalid: {}",
        combined
    );
}

#[test]
fn weights_set_json_object_non_numeric_weight_fails() {
    // Object format with non-numeric value must fail.
    let (ok, _stdout, stderr) = run_agcli(&[
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        r#"{"0":"hundred","1":200}"#,
        "--dry-run",
    ]);
    assert!(!ok, "weights with non-numeric weight value should fail");
    assert!(
        stderr.to_lowercase().contains("weight") || stderr.to_lowercase().contains("invalid") || stderr.to_lowercase().contains("number"),
        "stderr should mention weight/invalid/number: {}",
        stderr
    );
}
