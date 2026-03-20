//! CLI parse tests: weights subcommands
//! Run with: cargo test --test cli_weights

use clap::Parser;

#[test]
fn parse_weights_commit_reveal() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "97",
        "--weights",
        "0:100,1:200",
        "--wait",
    ]);
    assert!(
        cli.is_ok(),
        "should parse weights commit-reveal: {:?}",
        cli.err()
    );
}

#[test]
fn parse_weights_set_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
        "--dry-run",
    ]);
    assert!(
        cli.is_ok(),
        "should parse weights set --dry-run: {:?}",
        cli.err()
    );
}

#[test]
fn parse_weights_set() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:0.5,1:0.3,2:0.2",
    ]);
    assert!(cli.is_ok(), "should parse weights set: {:?}", cli.err());
}

#[test]
fn parse_weights_commit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit",
        "--netuid",
        "97",
        "--weights",
        "0:0.5,1:0.5",
    ]);
    assert!(cli.is_ok(), "should parse weights commit: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_with_salt() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit",
        "--netuid",
        "97",
        "--weights",
        "0:1.0",
        "--salt",
        "deadbeef",
    ]);
    assert!(
        cli.is_ok(),
        "should parse weights commit with salt: {:?}",
        cli.err()
    );
}

#[test]
fn parse_weights_reveal() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal",
        "--netuid",
        "97",
        "--weights",
        "0:0.5,1:0.5",
        "--salt",
        "abc123",
        "--version-key",
        "42",
    ]);
    assert!(cli.is_ok(), "should parse weights reveal: {:?}", cli.err());
}

#[test]
fn parse_weights_status() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "status", "--netuid", "1"]);
    assert!(cli.is_ok(), "should parse weights status: {:?}", cli.err());
}

#[test]
fn parse_weights_show() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "show", "--netuid", "97"]);
    assert!(cli.is_ok(), "weights show: {:?}", cli.err());
}

#[test]
fn parse_weights_set_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
    ]);
    assert!(cli.is_ok(), "weights set: {:?}", cli.err());
}

#[test]
fn parse_weights_set_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "set", "--weights", "0:100"]);
    assert!(cli.is_err(), "weights set without --netuid should fail");
}

#[test]
fn parse_weights_set_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "set", "--netuid", "1"]);
    assert!(cli.is_err(), "weights set without --weights should fail");
}

#[test]
fn parse_weights_set_with_version_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--version-key",
        "42",
    ]);
    assert!(cli.is_ok(), "weights set --version-key: {:?}", cli.err());
}

#[test]
fn parse_weights_set_stdin() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "-",
    ]);
    assert!(cli.is_ok(), "weights set stdin: {:?}", cli.err());
}

#[test]
fn parse_weights_set_file_path() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "@weights.json",
    ]);
    assert!(cli.is_ok(), "weights set file: {:?}", cli.err());
}

#[test]
fn parse_weights_set_single_weight() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:65535",
    ]);
    assert!(cli.is_ok(), "weights set single: {:?}", cli.err());
}

#[test]
fn parse_weights_set_with_all_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--password",
        "pw",
        "--batch",
        "--dry-run",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100",
    ]);
    assert!(cli.is_ok(), "weights set all flags: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
    assert!(cli.dry_run);
}

#[test]
fn parse_weights_commit_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
    ]);
    assert!(cli.is_ok(), "weights commit: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "commit", "--weights", "0:100"]);
    assert!(cli.is_err(), "commit without --netuid should fail");
}

#[test]
fn parse_weights_commit_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "commit", "--netuid", "1"]);
    assert!(cli.is_err(), "commit without --weights should fail");
}

#[test]
fn parse_weights_reveal_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
        "--salt",
        "mysalt",
    ]);
    assert!(cli.is_ok(), "weights reveal: {:?}", cli.err());
}

#[test]
fn parse_weights_reveal_missing_salt() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100",
    ]);
    assert!(cli.is_err(), "reveal without --salt should fail");
}

#[test]
fn parse_weights_reveal_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "reveal", "--netuid", "1", "--salt", "abc",
    ]);
    assert!(cli.is_err(), "reveal without --weights should fail");
}

#[test]
fn parse_weights_reveal_with_version_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--salt",
        "abc",
        "--version-key",
        "99",
    ]);
    assert!(cli.is_ok(), "reveal --version-key: {:?}", cli.err());
}

#[test]
fn parse_weights_show_basic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "show", "--netuid", "1"]);
    assert!(cli.is_ok(), "weights show: {:?}", cli.err());
}

#[test]
fn parse_weights_show_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "show",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "weights show --hotkey-address: {:?}", cli.err());
}

#[test]
fn parse_weights_show_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "weights",
        "show",
        "--netuid",
        "97",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--limit",
        "5",
    ]);
    assert!(cli.is_ok(), "weights show all: {:?}", cli.err());
}

#[test]
fn parse_weights_show_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "show"]);
    assert!(cli.is_err(), "weights show without --netuid should fail");
}

#[test]
fn parse_weights_status_basic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "status", "--netuid", "1"]);
    assert!(cli.is_ok(), "weights status: {:?}", cli.err());
}

#[test]
fn parse_weights_status_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "status"]);
    assert!(cli.is_err(), "weights status without --netuid should fail");
}

#[test]
fn parse_weights_commit_reveal_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100",
    ]);
    assert!(cli.is_ok(), "commit-reveal basic: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_with_wait() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
        "--wait",
    ]);
    assert!(cli.is_ok(), "commit-reveal --wait: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_with_version_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--version-key",
        "42",
    ]);
    assert!(cli.is_ok(), "commit-reveal --version-key: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--weights",
        "0:100",
    ]);
    assert!(cli.is_err(), "commit-reveal without --netuid should fail");
}

#[test]
fn parse_weights_commit_reveal_missing_weights() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "weights", "commit-reveal", "--netuid", "1"]);
    assert!(cli.is_err(), "commit-reveal without --weights should fail");
}

#[test]
fn parse_weights_commit_reveal_stdin() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "-",
    ]);
    assert!(cli.is_ok(), "commit-reveal stdin: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_file() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "@weights.json",
    ]);
    assert!(cli.is_ok(), "commit-reveal file: {:?}", cli.err());
}

#[test]
fn parse_weights_set_version_key_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--version-key",
        "42",
    ]);
    assert!(cli.is_ok(), "weights set version-key: {:?}", cli.err());
}

#[test]
fn parse_weights_set_json_input_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        r#"[{"uid":0,"weight":100}]"#,
    ]);
    assert!(cli.is_ok(), "weights set JSON: {:?}", cli.err());
}

#[test]
fn parse_weights_set_file_input_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "@weights.json",
    ]);
    assert!(cli.is_ok(), "weights set file: {:?}", cli.err());
}

#[test]
fn parse_weights_set_stdin_input_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "-",
    ]);
    assert!(cli.is_ok(), "weights set stdin: {:?}", cli.err());
}

#[test]
fn parse_weights_show_with_limit_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "weights", "show", "--netuid", "1", "--limit", "10",
    ]);
    assert!(cli.is_ok(), "weights show limit: {:?}", cli.err());
}

#[test]
fn parse_weights_show_all_args_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "show",
        "--netuid",
        "5",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--limit",
        "100",
    ]);
    assert!(cli.is_ok(), "weights show all: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_wait_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--wait",
    ]);
    assert!(cli.is_ok(), "weights commit-reveal wait: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_all_args_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "42",
        "--weights",
        r#"{"0":100,"1":200}"#,
        "--version-key",
        "7",
        "--wait",
    ]);
    assert!(cli.is_ok(), "weights commit-reveal all: {:?}", cli.err());
}

#[test]
fn parse_weights_no_subcommand_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights"]);
    assert!(cli.is_err(), "weights without subcommand should fail");
}

#[test]
fn parse_weights_set_json_object_format_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        r#"{"0":100,"1":200,"2":300}"#,
    ]);
    assert!(cli.is_ok(), "weights set JSON object: {:?}", cli.err());
}

#[test]
fn parse_weights_set_large_version_key_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--version-key",
        "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "weights set max version-key: {:?}", cli.err());
}

#[test]
fn parse_weights_set_with_global_dry_run_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--output",
        "json",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--dry-run",
    ]);
    assert!(cli.is_ok(), "weights set global+dry-run: {:?}", cli.err());
}

#[test]
fn parse_weights_status_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "status", "--netuid", "1"]);
    assert!(cli.is_ok(), "weights status: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
    ]);
    assert!(cli.is_ok(), "weights commit: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_salt_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--salt",
        "mysecret",
    ]);
    assert!(cli.is_ok(), "weights commit salt: {:?}", cli.err());
}

#[test]
fn parse_weights_reveal_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--salt",
        "mysecret",
    ]);
    assert!(cli.is_ok(), "weights reveal: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
    ]);
    assert!(cli.is_ok(), "weights commit-reveal: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_reveal_wait_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-reveal",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--wait",
    ]);
    assert!(cli.is_ok(), "weights commit-reveal wait: {:?}", cli.err());
}

#[test]
fn parse_weights_set_pid_uid_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200,2:50",
    ]);
    assert!(cli.is_ok(), "weights set pairs: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit",
        "--netuid",
        "1",
        "--weights",
        "[100,200,300]",
    ]);
    assert!(cli.is_ok(), "weights commit: {:?}", cli.err());
}

// ──── Stress / edge: many pairs and invalid payloads (parse accepts, runtime would fail) ────

/// Stress: 500 uid:weight pairs — parser should not hang or panic.
#[test]
fn parse_weights_set_stress_many_pairs() {
    let pairs: Vec<String> = (0..500).map(|i| format!("{}:1", i)).collect();
    let weights = pairs.join(",");
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        &weights,
    ]);
    assert!(
        cli.is_ok(),
        "weights set with 500 pairs should parse: {:?}",
        cli.err()
    );
}

/// Empty weights string parses (validation happens at run time).
#[test]
fn parse_weights_set_empty_string_parses() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "",
    ]);
    assert!(cli.is_ok(), "empty --weights parses; validation at run: {:?}", cli.err());
}

/// JSON array with missing "weight" key parses; resolve_weights would fail at run time.
#[test]
fn parse_weights_set_json_missing_weight_parses() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        r#"[{"uid":0}]"#,
    ]);
    assert!(
        cli.is_ok(),
        "malformed JSON parses; resolve_weights fails at run: {:?}",
        cli.err()
    );
}

/// Max u16 weight value in pair form.
#[test]
fn parse_weights_set_max_u16_pair() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:65535,65535:65535",
    ]);
    assert!(cli.is_ok(), "max u16 pairs: {:?}", cli.err());
}
