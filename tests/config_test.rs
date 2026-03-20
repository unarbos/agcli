//! Config and network resolution tests.
//! Run with: cargo test --test config_test

use agcli::cli::Cli;
use agcli::types::network::Network;
use clap::Parser;

#[test]
fn resolve_finney() {
    let cli = Cli::try_parse_from(["agcli", "balance"]).unwrap();
    let net = cli.resolve_network();
    assert!(matches!(net, Network::Finney));
    assert_eq!(net.ws_url(), "wss://entrypoint-finney.opentensor.ai:443");
}

#[test]
fn resolve_test_network() {
    let cli = Cli::try_parse_from(["agcli", "--network", "test", "balance"]).unwrap();
    let net = cli.resolve_network();
    assert!(matches!(net, Network::Test));
}

#[test]
fn resolve_local_network() {
    let cli = Cli::try_parse_from(["agcli", "--network", "local", "balance"]).unwrap();
    let net = cli.resolve_network();
    assert!(matches!(net, Network::Local));
}

#[test]
fn endpoint_overrides_network() {
    let cli = Cli::try_parse_from([
        "agcli",
        "--endpoint",
        "ws://custom:9944",
        "--network",
        "test",
        "balance",
    ])
    .unwrap();
    let net = cli.resolve_network();
    assert!(matches!(net, Network::Custom(_)));
    assert_eq!(net.ws_url(), "ws://custom:9944");
}

#[test]
fn config_apply_defaults() {
    let mut cli = Cli::try_parse_from(["agcli", "balance"]).unwrap();
    let cfg = agcli::Config {
        network: Some("test".to_string()),
        wallet: Some("mywallet".to_string()),
        ..Default::default()
    };
    // No explicit flags on command line — config should apply
    let args: Vec<String> = vec!["agcli".to_string(), "balance".to_string()];
    cli.apply_config_with_args(&cfg, &args);
    assert_eq!(cli.network, "test");
    assert_eq!(cli.wallet, "mywallet");
}

#[test]
fn cli_flags_override_config() {
    let mut cli = Cli::try_parse_from([
        "agcli",
        "--network",
        "local",
        "--wallet",
        "explicit",
        "balance",
    ])
    .unwrap();
    let cfg = agcli::Config {
        network: Some("test".to_string()),
        wallet: Some("config_wallet".to_string()),
        ..Default::default()
    };
    let args: Vec<String> = vec![
        "agcli".to_string(),
        "--network".to_string(),
        "local".to_string(),
        "--wallet".to_string(),
        "explicit".to_string(),
        "balance".to_string(),
    ];
    cli.apply_config_with_args(&cfg, &args);
    // CLI flags should take precedence
    assert_eq!(cli.network, "local");
    assert_eq!(cli.wallet, "explicit");
}

#[test]
fn live_interval_parsing() {
    // --live with explicit value
    let cli = Cli::try_parse_from([
        "agcli",
        "--live",
        "5",
        "subnet",
        "metagraph",
        "--netuid",
        "1",
    ])
    .unwrap();
    assert_eq!(cli.live_interval(), Some(5));
}

#[test]
fn config_batch_default_applies() {
    let mut cli = Cli::try_parse_from(["agcli", "balance"]).unwrap();
    let cfg = agcli::Config {
        batch: Some(true),
        ..Default::default()
    };
    let args: Vec<String> = vec!["agcli".to_string(), "balance".to_string()];
    cli.apply_config_with_args(&cfg, &args);
    assert!(cli.batch);
}

#[test]
fn config_batch_cli_overrides() {
    // --batch on CLI should stay true even if config says false
    let mut cli = Cli::try_parse_from(["agcli", "--batch", "balance"]).unwrap();
    let cfg = agcli::Config {
        batch: Some(false),
        ..Default::default()
    };
    let args: Vec<String> = vec![
        "agcli".to_string(),
        "--batch".to_string(),
        "balance".to_string(),
    ];
    cli.apply_config_with_args(&cfg, &args);
    assert!(cli.batch);
}

#[test]
/// Issue 639: Explicit --network finney should NOT be overridden by config.
/// Previously, `apply_config` compared against the clap default ("finney")
/// and couldn't distinguish "user passed finney" from "clap filled default".
fn config_explicit_finney_not_overridden() {
    let mut cli = Cli::try_parse_from(["agcli", "--network", "finney", "balance"]).unwrap();
    let cfg = agcli::Config {
        network: Some("test".to_string()),
        ..Default::default()
    };
    // Use apply_config_with_args to simulate the explicit flag being in argv
    let args: Vec<String> = vec![
        "agcli".to_string(),
        "--network".to_string(),
        "finney".to_string(),
        "balance".to_string(),
    ];
    cli.apply_config_with_args(&cfg, &args);
    // CLI explicitly passed finney — must NOT be overridden by config
    assert_eq!(cli.network, "finney");
}

#[test]
/// Issue 639: Explicit --wallet default should NOT be overridden by config.
fn config_explicit_default_wallet_not_overridden() {
    let mut cli = Cli::try_parse_from(["agcli", "--wallet", "default", "balance"]).unwrap();
    let cfg = agcli::Config {
        wallet: Some("config_wallet".to_string()),
        ..Default::default()
    };
    let args: Vec<String> = vec![
        "agcli".to_string(),
        "--wallet".to_string(),
        "default".to_string(),
        "balance".to_string(),
    ];
    cli.apply_config_with_args(&cfg, &args);
    assert_eq!(cli.wallet, "default");
}

#[test]
/// Issue 639: Explicit --hotkey-name default should NOT be overridden by config.
fn config_explicit_default_hotkey_not_overridden() {
    let mut cli = Cli::try_parse_from(["agcli", "--hotkey-name", "default", "balance"]).unwrap();
    let cfg = agcli::Config {
        hotkey: Some("config_hotkey".to_string()),
        ..Default::default()
    };
    let args: Vec<String> = vec![
        "agcli".to_string(),
        "--hotkey-name".to_string(),
        "default".to_string(),
        "balance".to_string(),
    ];
    cli.apply_config_with_args(&cfg, &args);
    assert_eq!(cli.hotkey_name, "default");
}

#[test]
/// Issue 639: Explicit --wallet-dir with default value should NOT be overridden.
fn config_explicit_default_wallet_dir_not_overridden() {
    let mut cli =
        Cli::try_parse_from(["agcli", "--wallet-dir", "~/.bittensor/wallets", "balance"]).unwrap();
    let cfg = agcli::Config {
        wallet_dir: Some("/custom/path".to_string()),
        ..Default::default()
    };
    let args: Vec<String> = vec![
        "agcli".to_string(),
        "--wallet-dir".to_string(),
        "~/.bittensor/wallets".to_string(),
        "balance".to_string(),
    ];
    cli.apply_config_with_args(&cfg, &args);
    assert_eq!(cli.wallet_dir, "~/.bittensor/wallets");
}

#[test]
/// Issue 639: When no flag is passed, config values should apply.
fn config_applies_when_no_flags() {
    let mut cli = Cli::try_parse_from(["agcli", "balance"]).unwrap();
    let cfg = agcli::Config {
        network: Some("test".to_string()),
        wallet: Some("mywallet".to_string()),
        hotkey: Some("myhotkey".to_string()),
        wallet_dir: Some("/custom/dir".to_string()),
        ..Default::default()
    };
    let args: Vec<String> = vec!["agcli".to_string(), "balance".to_string()];
    cli.apply_config_with_args(&cfg, &args);
    assert_eq!(cli.network, "test");
    assert_eq!(cli.wallet, "mywallet");
    assert_eq!(cli.hotkey_name, "myhotkey");
    assert_eq!(cli.wallet_dir, "/custom/dir");
}

#[test]
/// Issue 639: --wallet short form (-w) should be preserved.
fn config_short_wallet_flag_not_overridden() {
    let mut cli = Cli::try_parse_from(["agcli", "-w", "explicit", "balance"]).unwrap();
    let cfg = agcli::Config {
        wallet: Some("config_wallet".to_string()),
        ..Default::default()
    };
    let args: Vec<String> = vec![
        "agcli".to_string(),
        "-w".to_string(),
        "explicit".to_string(),
        "balance".to_string(),
    ];
    cli.apply_config_with_args(&cfg, &args);
    assert_eq!(cli.wallet, "explicit");
}

#[test]
/// Issue 642: --batch and --yes should be separate modes.
fn batch_and_yes_modes_separate() {
    use agcli::cli::helpers::{is_batch_mode, is_yes_mode, set_batch_mode, set_yes_mode};

    // Only --batch: batch mode on, yes mode off (but is_yes_mode includes batch)
    set_batch_mode(true);
    set_yes_mode(false);
    assert!(is_batch_mode());
    assert!(is_yes_mode()); // batch implies yes

    // Only --yes: yes mode on, batch mode off
    set_batch_mode(false);
    set_yes_mode(true);
    assert!(!is_batch_mode()); // --yes alone should NOT trigger batch mode
    assert!(is_yes_mode());

    // Neither: both off
    set_batch_mode(false);
    set_yes_mode(false);
    assert!(!is_batch_mode());
    assert!(!is_yes_mode());

    // Both: both on
    set_batch_mode(true);
    set_yes_mode(true);
    assert!(is_batch_mode());
    assert!(is_yes_mode());

    // Reset
    set_batch_mode(false);
    set_yes_mode(false);
}

#[test]
fn config_spending_limits_serialization() {
    use std::collections::HashMap;
    let mut limits = HashMap::new();
    limits.insert("97".to_string(), 100.0);
    limits.insert("*".to_string(), 500.0);
    let cfg = agcli::Config {
        spending_limits: Some(limits),
        ..Default::default()
    };
    let s = toml::to_string_pretty(&cfg).unwrap();
    assert!(s.contains("97"));
    assert!(s.contains("100"));
    let parsed: agcli::Config = toml::from_str(&s).unwrap();
    let sl = parsed.spending_limits.unwrap();
    assert_eq!(*sl.get("97").unwrap(), 100.0);
    assert_eq!(*sl.get("*").unwrap(), 500.0);
}
