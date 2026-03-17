//! Tests for scaffold configuration parsing and type behavior.
//! Run with: cargo test --test scaffold_config_test

use agcli::scaffold::ScaffoldConfig;
use agcli::types::balance::{Balance, RAO_PER_TAO};
use agcli::types::network::{NetUid, Network};

// ───────────────── ScaffoldConfig TOML parsing ─────────────────

#[test]
fn scaffold_default_has_one_subnet() {
    let cfg = ScaffoldConfig::default();
    assert_eq!(cfg.subnet.len(), 1);
    assert_eq!(cfg.subnet[0].neuron.len(), 3);
}

#[test]
fn scaffold_default_subnet_tempo() {
    let cfg = ScaffoldConfig::default();
    assert_eq!(cfg.subnet[0].tempo, Some(100));
}

#[test]
fn scaffold_from_toml_minimal() {
    let toml_str = r#"
        [[subnet]]
        tempo = 50

        [[subnet.neuron]]
        name = "val"
        fund_tao = 500.0
    "#;
    let cfg: ScaffoldConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.subnet.len(), 1);
    assert_eq!(cfg.subnet[0].tempo, Some(50));
    assert_eq!(cfg.subnet[0].neuron.len(), 1);
    assert_eq!(cfg.subnet[0].neuron[0].name, "val");
    assert_eq!(cfg.subnet[0].neuron[0].fund_tao, Some(500.0));
    assert!(cfg.subnet[0].neuron[0].register); // default true
}

#[test]
fn scaffold_from_toml_multiple_subnets() {
    let toml_str = r#"
        [[subnet]]
        tempo = 100
        max_allowed_validators = 16

        [[subnet.neuron]]
        name = "miner1"
        fund_tao = 10.0

        [[subnet]]
        tempo = 200
        commit_reveal = true

        [[subnet.neuron]]
        name = "miner2"
        fund_tao = 20.0
        register = false
    "#;
    let cfg: ScaffoldConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.subnet.len(), 2);
    assert_eq!(cfg.subnet[0].max_allowed_validators, Some(16));
    assert_eq!(cfg.subnet[1].tempo, Some(200));
    assert_eq!(cfg.subnet[1].commit_reveal, Some(true));
    assert!(!cfg.subnet[1].neuron[0].register);
}

#[test]
fn scaffold_from_toml_all_hyperparams() {
    let toml_str = r#"
        [[subnet]]
        tempo = 360
        max_allowed_validators = 32
        max_allowed_uids = 256
        min_allowed_weights = 4
        max_weight_limit = 65535
        immunity_period = 1000
        weights_rate_limit = 10
        commit_reveal = false
        activity_cutoff = 500

        [[subnet.neuron]]
        name = "node1"
    "#;
    let cfg: ScaffoldConfig = toml::from_str(toml_str).unwrap();
    let s = &cfg.subnet[0];
    assert_eq!(s.tempo, Some(360));
    assert_eq!(s.max_allowed_validators, Some(32));
    assert_eq!(s.max_allowed_uids, Some(256));
    assert_eq!(s.min_allowed_weights, Some(4));
    assert_eq!(s.max_weight_limit, Some(65535));
    assert_eq!(s.immunity_period, Some(1000));
    assert_eq!(s.weights_rate_limit, Some(10));
    assert_eq!(s.commit_reveal, Some(false));
    assert_eq!(s.activity_cutoff, Some(500));
}

#[test]
fn scaffold_chain_config_defaults() {
    let cfg = ScaffoldConfig::default();
    assert_eq!(cfg.chain.port, 9944);
    assert!(cfg.chain.start);
    assert_eq!(cfg.chain.timeout, 120);
}

#[test]
fn scaffold_chain_config_override() {
    let toml_str = r#"
        [chain]
        port = 9955
        start = false
        timeout = 60
        container = "my-localnet"

        [[subnet]]
        [[subnet.neuron]]
        name = "n1"
    "#;
    let cfg: ScaffoldConfig = toml::from_str(toml_str).unwrap();
    assert_eq!(cfg.chain.port, 9955);
    assert!(!cfg.chain.start);
    assert_eq!(cfg.chain.timeout, 60);
    assert_eq!(cfg.chain.container, "my-localnet");
}

// ───────────────── Network type ─────────────────

#[test]
fn network_default_is_finney() {
    let net = Network::default();
    assert!(matches!(net, Network::Finney));
}

#[test]
fn network_display_variants() {
    assert_eq!(format!("{}", Network::Finney), "finney");
    assert_eq!(format!("{}", Network::Test), "test");
    assert_eq!(format!("{}", Network::Local), "local");
    assert_eq!(format!("{}", Network::Archive), "archive");
    assert_eq!(
        format!("{}", Network::Custom("ws://mynode:9944".into())),
        "custom(ws://mynode:9944)"
    );
}

#[test]
fn network_ws_urls_finney_has_fallback() {
    let urls = Network::Finney.ws_urls();
    assert!(urls.len() >= 2, "Finney should have primary + fallback");
}

#[test]
fn network_ws_urls_custom_has_one() {
    let net = Network::Custom("ws://x:9944".into());
    let urls = net.ws_urls();
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "ws://x:9944");
}

#[test]
fn network_archive_ws_url() {
    let net = Network::Archive;
    assert!(net.ws_url().contains("onfinality"));
}

// ───────────────── NetUid type ─────────────────

#[test]
fn netuid_root_is_zero() {
    assert_eq!(NetUid::ROOT.as_u16(), 0);
}

#[test]
fn netuid_from_u16() {
    let nid: NetUid = 42u16.into();
    assert_eq!(nid.as_u16(), 42);
    assert_eq!(format!("{}", nid), "42");
}

// ───────────────── Balance edge cases ─────────────────

#[test]
fn balance_display_four_decimals() {
    let b = Balance::from_tao(1.123456789);
    let s = format!("{}", b);
    // Display uses 4 decimal places
    assert!(s.starts_with("1.1235"), "got: {}", s);
    assert!(s.contains("τ"));
}

#[test]
fn balance_display_tao_nine_decimals() {
    let b = Balance::from_rao(1);
    let s = b.display_tao();
    assert!(s.contains("0.000000001"), "got: {}", s);
}

#[test]
fn balance_max_rao_to_tao() {
    let b = Balance::from_rao(u64::MAX);
    // Should not panic; tao() returns a large float
    let tao = b.tao();
    assert!(tao > 0.0);
    assert!(tao > 1e10);
}

#[test]
fn balance_sub_is_saturating() {
    let small = Balance::from_tao(1.0);
    let large = Balance::from_tao(100.0);
    let result = small - large;
    assert_eq!(result.rao(), 0);
}

#[test]
fn balance_from_tao_zero() {
    let b = Balance::from_tao(0.0);
    assert_eq!(b.rao(), 0);
    assert_eq!(b.tao(), 0.0);
}

#[test]
fn balance_one_tao_equals_rao_per_tao() {
    let b = Balance::from_tao(1.0);
    assert_eq!(b.rao(), RAO_PER_TAO);
}
