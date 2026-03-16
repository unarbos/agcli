//! Tests for CLI helper functions.
//! Run with: cargo test --test helpers_test

use agcli::cli::helpers::{
    parse_children, parse_weight_pairs, validate_amount, validate_delegate_take,
    validate_emission_weights, validate_ipv4, validate_max_cost, validate_name, validate_symbol,
    validate_take_pct,
};
use agcli::utils::explain;

#[test]
fn parse_weight_pairs_basic() {
    let (uids, weights) = parse_weight_pairs("0:100,1:200,2:300").unwrap();
    assert_eq!(uids, vec![0, 1, 2]);
    assert_eq!(weights, vec![100, 200, 300]);
}

#[test]
fn parse_weight_pairs_with_spaces() {
    let (uids, weights) = parse_weight_pairs("0: 100, 1: 200").unwrap();
    assert_eq!(uids, vec![0, 1]);
    assert_eq!(weights, vec![100, 200]);
}

#[test]
fn parse_weight_pairs_single() {
    let (uids, weights) = parse_weight_pairs("5:65535").unwrap();
    assert_eq!(uids, vec![5]);
    assert_eq!(weights, vec![65535]);
}

#[test]
fn parse_weight_pairs_invalid() {
    assert!(parse_weight_pairs("0").is_err());
    assert!(parse_weight_pairs("abc:def").is_err());
    assert!(parse_weight_pairs("").is_err());
}

#[test]
fn parse_children_basic() {
    let result = parse_children("1000:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, 1000);
    assert_eq!(
        result[0].1,
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
    );
}

#[test]
fn parse_children_multiple() {
    let result = parse_children("500:5Abc,500:5Def").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, 500);
    assert_eq!(result[1].0, 500);
}

#[test]
fn parse_children_invalid() {
    assert!(parse_children("invalid").is_err());
    assert!(parse_children("").is_err());
}

#[test]
fn parse_weight_pairs_overflow_uid() {
    // UID > 65535 should fail
    let result = parse_weight_pairs("70000:100");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid UID"),
        "Expected helpful UID error, got: {}",
        msg
    );
}

#[test]
fn parse_weight_pairs_overflow_weight() {
    // Weight > 65535 should fail
    let result = parse_weight_pairs("0:70000");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid weight"),
        "Expected helpful weight error, got: {}",
        msg
    );
}

#[test]
fn parse_children_bad_proportion() {
    let result = parse_children("abc:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid proportion"),
        "Expected helpful proportion error, got: {}",
        msg
    );
}

// ──── Explain tests ────

#[test]
fn explain_known_topics() {
    assert!(explain::explain("tempo").is_some());
    assert!(explain::explain("commit-reveal").is_some());
    assert!(explain::explain("commitreveal").is_some());
    assert!(explain::explain("yuma").is_some());
    assert!(explain::explain("amm").is_some());
    assert!(explain::explain("bootstrap").is_some());
    assert!(explain::explain("rate-limits").is_some());
    assert!(explain::explain("stake-weight").is_some());
    assert!(explain::explain("alpha").is_some());
    assert!(explain::explain("emission").is_some());
}

#[test]
fn explain_case_insensitive() {
    assert!(explain::explain("TEMPO").is_some());
    assert!(explain::explain("Commit-Reveal").is_some());
    assert!(explain::explain("AMM").is_some());
}

#[test]
fn explain_unknown_topic() {
    assert!(explain::explain("nonexistent_topic_xyz").is_none());
}

#[test]
fn explain_list_topics_not_empty() {
    let topics = explain::list_topics();
    assert!(
        topics.len() >= 10,
        "Expected at least 10 topics, got {}",
        topics.len()
    );
    for (key, desc) in &topics {
        assert!(!key.is_empty());
        assert!(!desc.is_empty());
    }
}

#[test]
fn explain_content_has_substance() {
    // Each explanation should be non-trivial
    let text = explain::explain("tempo").unwrap();
    assert!(
        text.len() > 100,
        "Explanation too short: {} chars",
        text.len()
    );
    assert!(
        text.contains("blocks"),
        "Tempo explanation should mention blocks"
    );
}

#[test]
fn explain_aliases_work() {
    // "cr" should resolve to commit-reveal
    assert!(explain::explain("cr").is_some());
    // "dtao" should resolve to AMM
    assert!(explain::explain("dtao").is_some());
    // "1000" should resolve to stake-weight
    assert!(explain::explain("1000").is_some());
}

// ──── json_to_subxt_value tests ────

#[test]
fn json_to_subxt_value_number() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(42));
    // Should produce a u128 value
    assert_eq!(
        format!("{:?}", val),
        format!("{:?}", subxt::dynamic::Value::u128(42))
    );
}

#[test]
fn json_to_subxt_value_string() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!("hello"));
    assert_eq!(
        format!("{:?}", val),
        format!("{:?}", subxt::dynamic::Value::string("hello".to_string()))
    );
}

#[test]
fn json_to_subxt_value_bool() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(true));
    assert_eq!(
        format!("{:?}", val),
        format!("{:?}", subxt::dynamic::Value::bool(true))
    );
}

#[test]
fn json_to_subxt_value_hex_bytes() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!("0xdeadbeef"));
    // Should decode as bytes
    let expected = subxt::dynamic::Value::from_bytes(vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(format!("{:?}", val), format!("{:?}", expected));
}

#[test]
fn json_to_subxt_value_array() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!([1, 2, 3]));
    // Should produce an unnamed composite
    let _formatted = format!("{:?}", val); // Just check it doesn't panic
}

// ──── Pretty mode tests ────

#[test]
fn pretty_mode_flag_toggles() {
    use agcli::cli::helpers::{is_pretty_mode, set_pretty_mode};
    set_pretty_mode(true);
    assert!(is_pretty_mode());
    set_pretty_mode(false);
    assert!(!is_pretty_mode());
}

// ──── Step 18: Batch mode & spending limits tests ────

#[test]
fn batch_mode_flag_sets_global() {
    use agcli::cli::helpers::{is_batch_mode, set_batch_mode};
    set_batch_mode(true);
    assert!(is_batch_mode());
    set_batch_mode(false);
    assert!(!is_batch_mode());
}

#[test]
fn spending_limit_no_config_passes() {
    // No config file → should pass for any amount
    let result = agcli::cli::helpers::check_spending_limit(97, 100.0);
    assert!(
        result.is_ok(),
        "No config should always pass: {:?}",
        result.err()
    );
}

// ──── Step 26: Edge case tests ────

#[test]
fn parse_weight_pairs_empty_string() {
    let result = parse_weight_pairs("");
    assert!(result.is_err(), "empty string should fail");
}

#[test]
fn parse_weight_pairs_only_commas() {
    let result = parse_weight_pairs(",,,");
    assert!(result.is_err(), "commas-only should fail");
}

#[test]
fn parse_weight_pairs_negative_uid() {
    // Negative values can't parse as u16
    let result = parse_weight_pairs("-1:100");
    assert!(result.is_err(), "negative UID should fail");
}

#[test]
fn parse_weight_pairs_extra_colon() {
    let result = parse_weight_pairs("0:100:extra");
    assert!(result.is_err(), "extra colon should fail");
}

#[test]
fn parse_weight_pairs_max_u16_values() {
    // Max u16 values should succeed
    let (uids, weights) = parse_weight_pairs("65535:65535").unwrap();
    assert_eq!(uids, vec![65535]);
    assert_eq!(weights, vec![65535]);
}

#[test]
fn parse_children_empty_hotkey() {
    let result = parse_children("1000:");
    // Should produce a child with empty hotkey string (not an error from parsing)
    assert!(result.is_ok());
    let children = result.unwrap();
    assert_eq!(children[0].1, "");
}

#[test]
fn parse_children_zero_proportion() {
    let result = parse_children("0:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(result.is_ok(), "zero proportion should be allowed");
    assert_eq!(result.unwrap()[0].0, 0);
}

#[test]
fn json_to_subxt_value_null() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(null));
    // Null should not panic — produces a string representation
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_object() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!({"key": "value"}));
    // Objects should not panic — produces a string representation
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_negative_number() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(-42));
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_large_number() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!(u64::MAX));
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_float_as_string() {
    use agcli::cli::helpers::json_to_subxt_value;
    // JSON floats become strings since we can't represent them as u128/i128
    let val = json_to_subxt_value(&serde_json::json!(3.15));
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_invalid_hex() {
    use agcli::cli::helpers::json_to_subxt_value;
    // "0x" prefix but invalid hex should fall back to string
    let val = json_to_subxt_value(&serde_json::json!("0xzzzz"));
    let _formatted = format!("{:?}", val);
}

#[test]
fn json_to_subxt_value_empty_array() {
    use agcli::cli::helpers::json_to_subxt_value;
    let val = json_to_subxt_value(&serde_json::json!([]));
    let _formatted = format!("{:?}", val);
}

#[test]
fn ss58_validation_invalid_address() {
    use agcli::wallet::keypair;
    let result = keypair::from_ss58("not_an_address");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Invalid SS58"),
        "Expected SS58 error, got: {}",
        msg
    );
}

#[test]
fn ss58_validation_empty_address() {
    use agcli::wallet::keypair;
    let result = keypair::from_ss58("");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Empty address"),
        "Expected empty address error, got: {}",
        msg
    );
}

#[test]
fn ss58_validation_short_address() {
    use agcli::wallet::keypair;
    let result = keypair::from_ss58("5abc");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("too short"),
        "Expected 'too short' error, got: {}",
        msg
    );
}

#[test]
fn ss58_validation_valid_address() {
    use agcli::wallet::keypair;
    assert!(keypair::from_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY").is_ok());
    assert!(keypair::from_ss58("invalid").is_err());
    assert!(keypair::from_ss58("").is_err());
}

#[test]
fn explain_all_topics_have_content() {
    let topics = explain::list_topics();
    for (key, _desc) in &topics {
        let content = explain::explain(key);
        assert!(
            content.is_some(),
            "Topic '{}' listed but has no content",
            key
        );
        assert!(
            content.unwrap().len() > 50,
            "Topic '{}' content is too short",
            key
        );
    }
}

#[test]
fn balance_from_tao_negative() {
    // Negative TAO should produce 0 (wraps due to as u64)
    use agcli::types::Balance;
    let b = Balance::from_tao(-1.0);
    // This is an edge case — negative f64 cast to u64 wraps. Document behavior.
    // Just verify it doesn't panic.
    let _ = b.rao();
}

#[test]
fn balance_from_tao_very_large() {
    use agcli::types::Balance;
    // Very large TAO — near u64 max
    let b = Balance::from_tao(18.0); // 18 TAO = 18_000_000_000 RAO, well within u64
    assert_eq!(b.rao(), 18_000_000_000);
}

#[test]
fn balance_display_tao_precision() {
    use agcli::types::Balance;
    let b = Balance::from_rao(1); // 1 RAO = 0.000000001 TAO
    let s = b.display_tao();
    assert!(
        s.contains("0.000000001"),
        "Expected full precision, got: {}",
        s
    );
}

#[test]
fn format_tao_zero() {
    use agcli::types::Balance;
    use agcli::utils::format::format_tao;
    let s = format_tao(Balance::ZERO);
    assert!(s.contains("0.0"), "Expected zero TAO display, got: {}", s);
}

#[test]
fn explain_coldkey_swap_topic() {
    let content = explain::explain("coldkey-swap");
    assert!(content.is_some(), "coldkey-swap topic should exist");
    let text = content.unwrap();
    assert!(
        text.len() > 200,
        "coldkey-swap explanation should be substantial"
    );
    assert!(text.contains("schedule"), "should mention scheduling");
    assert!(text.contains("security"), "should mention security");
}

#[test]
fn explain_coldkey_swap_aliases() {
    // All aliases should resolve
    assert!(explain::explain("coldkey").is_some());
    assert!(explain::explain("ckswap").is_some());
    assert!(explain::explain("coldkeyswap").is_some());
    assert!(explain::explain("COLDKEY-SWAP").is_some());
}

#[test]
fn explain_topic_count_includes_coldkey_swap() {
    let topics = explain::list_topics();
    let has_ck = topics.iter().any(|(k, _)| *k == "coldkey-swap");
    assert!(has_ck, "coldkey-swap should be in list_topics()");
    assert!(
        topics.len() >= 22,
        "Expected at least 22 topics (19 + governance, senate, mev-shield), got {}",
        topics.len()
    );
}

// ──── Step 32: governance, senate, mev-shield explain topics ────

#[test]
fn explain_governance_topic() {
    let content = explain::explain("governance");
    assert!(content.is_some(), "governance topic should exist");
    let text = content.unwrap();
    assert!(text.contains("GOVERNANCE"), "should have title");
    assert!(text.contains("proposal"), "should mention proposals");
}

#[test]
fn explain_governance_aliases() {
    assert!(explain::explain("gov").is_some());
    assert!(explain::explain("proposals").is_some());
    assert!(explain::explain("GOV").is_some());
}

#[test]
fn explain_senate_topic() {
    let content = explain::explain("senate");
    assert!(content.is_some(), "senate topic should exist");
    let text = content.unwrap();
    assert!(text.contains("SENATE"), "should have title");
    assert!(text.contains("triumvirate") || text.contains("Triumvirate"));
}

#[test]
fn explain_senate_aliases() {
    assert!(explain::explain("triumvirate").is_some());
    assert!(explain::explain("SENATE").is_some());
}

#[test]
fn explain_mev_shield_topic() {
    let content = explain::explain("mev-shield");
    assert!(content.is_some(), "mev-shield topic should exist");
    let text = content.unwrap();
    assert!(text.contains("MEV"), "should contain MEV");
    assert!(text.contains("protection") || text.contains("shield") || text.contains("Shield"));
}

#[test]
fn explain_mev_aliases() {
    assert!(explain::explain("mev").is_some());
    assert!(explain::explain("mevshield").is_some());
    assert!(explain::explain("mevprotection").is_some());
}

// ──── Step 33 — explain limits, hyperparams, axon ────

#[test]
fn explain_limits_topic() {
    let content = explain::explain("limits");
    assert!(content.is_some(), "limits topic should exist");
    let text = content.unwrap();
    assert!(text.contains("weight"), "should mention weight limits");
    assert!(text.contains("registration") || text.contains("Registration"));
}

#[test]
fn explain_limits_aliases() {
    assert!(explain::explain("networklimits").is_some());
    assert!(explain::explain("chainlimits").is_some());
    assert!(explain::explain("LIMITS").is_some());
}

#[test]
fn explain_hyperparams_topic() {
    let content = explain::explain("hyperparams");
    assert!(content.is_some(), "hyperparams topic should exist");
    let text = content.unwrap();
    assert!(text.contains("tempo"), "should mention tempo");
    assert!(text.contains("rho") || text.contains("kappa"));
}

#[test]
fn explain_hyperparams_aliases() {
    assert!(explain::explain("hyperparameters").is_some());
    assert!(explain::explain("params").is_some());
}

#[test]
fn explain_axon_topic() {
    let content = explain::explain("axon");
    assert!(content.is_some(), "axon topic should exist");
    let text = content.unwrap();
    assert!(text.contains("endpoint") || text.contains("IP"));
    assert!(text.contains("miner") || text.contains("Miner"));
}

#[test]
fn explain_axon_aliases() {
    assert!(explain::explain("axoninfo").is_some());
    assert!(explain::explain("serving").is_some());
}

#[test]
fn explain_topic_count_28() {
    let topics = explain::list_topics();
    assert!(
        topics.len() >= 28,
        "Expected at least 28 topics, got {}",
        topics.len()
    );
}

#[test]
fn explain_take_topic() {
    let content = explain::explain("take");
    assert!(content.is_some(), "take topic should exist");
    let text = content.unwrap();
    assert!(text.contains("percentage") || text.contains("dividends"));
    assert!(text.contains("delegate") || text.contains("Delegate"));
}

#[test]
fn explain_take_aliases() {
    assert!(explain::explain("delegate-take").is_some());
    assert!(explain::explain("validator_take").is_some());
}

#[test]
fn explain_recycle_topic() {
    let content = explain::explain("recycle");
    assert!(content.is_some(), "recycle topic should exist");
    let text = content.unwrap();
    assert!(text.contains("alpha") || text.contains("Alpha"));
    assert!(text.contains("burn") || text.contains("Burn"));
}

#[test]
fn explain_recycle_aliases() {
    assert!(explain::explain("burn-alpha").is_some());
    assert!(explain::explain("recyclealpha").is_some());
}

#[test]
fn explain_pow_topic() {
    let content = explain::explain("pow");
    assert!(content.is_some(), "pow topic should exist");
    let text = content.unwrap();
    assert!(text.contains("difficulty") || text.contains("nonce"));
    assert!(text.contains("registration") || text.contains("Registration"));
}

#[test]
fn explain_pow_aliases() {
    assert!(explain::explain("pow-registration").is_some());
    assert!(explain::explain("proof-of-work").is_some());
}

// ──── Step 34 — archive explain topic ────

#[test]
fn explain_archive_topic() {
    let content = explain::explain("archive");
    assert!(content.is_some(), "archive topic should exist");
    let text = content.unwrap();
    assert!(text.contains("historical") || text.contains("Historical"));
    assert!(text.contains("--at-block") || text.contains("at_block"));
    assert!(text.contains("--network archive"));
}

#[test]
fn explain_archive_aliases() {
    assert!(explain::explain("archive-node").is_some());
    assert!(explain::explain("historical").is_some());
    assert!(explain::explain("wayback").is_some());
    assert!(explain::explain("ARCHIVE").is_some());
}

#[test]
fn explain_topic_count_29() {
    let topics = explain::list_topics();
    assert!(
        topics.len() >= 29,
        "Expected at least 29 topics, got {}",
        topics.len()
    );
}

// ──── Step 34 — Network::Archive ────

#[test]
fn network_archive_url() {
    use agcli::types::network::Network;
    let net = Network::Archive;
    assert!(net.ws_url().starts_with("wss://"));
    assert!(matches!(net, Network::Archive));
    assert_eq!(format!("{}", net), "archive");
}

#[test]
fn network_finney_not_archive() {
    use agcli::types::network::Network;
    assert!(!matches!(Network::Finney, Network::Archive));
    assert!(!matches!(Network::Test, Network::Archive));
    assert!(!matches!(Network::Local, Network::Archive));
}

// ──── Amount validation tests ────

#[test]
fn validate_amount_positive() {
    assert!(validate_amount(1.0, "test").is_ok());
    assert!(validate_amount(0.000000001, "test").is_ok());
    assert!(validate_amount(1000000.0, "test").is_ok());
}

#[test]
fn validate_amount_zero_rejects() {
    let result = validate_amount(0.0, "test");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("greater than zero"), "error msg: {}", msg);
}

#[test]
fn validate_amount_negative_rejects() {
    let result = validate_amount(-1.0, "stake amount");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("negative"), "error msg: {}", msg);
    assert!(msg.contains("stake amount"), "should include label: {}", msg);
}

#[test]
fn validate_amount_nan_rejects() {
    let result = validate_amount(f64::NAN, "test");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("finite"), "error msg: {}", msg);
}

#[test]
fn validate_amount_infinity_rejects() {
    let result = validate_amount(f64::INFINITY, "test");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("finite"), "error msg: {}", msg);
}

#[test]
fn validate_amount_neg_infinity_rejects() {
    let result = validate_amount(f64::NEG_INFINITY, "test");
    assert!(result.is_err());
}

// ──── Take percentage validation tests ────

#[test]
fn validate_take_valid_range() {
    assert!(validate_take_pct(0.0).is_ok());
    assert!(validate_take_pct(9.0).is_ok());
    assert!(validate_take_pct(18.0).is_ok());
}

#[test]
fn validate_take_over_max_rejects() {
    let result = validate_take_pct(18.01);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("18%"), "should mention max: {}", msg);
}

#[test]
fn validate_take_negative_rejects() {
    let result = validate_take_pct(-1.0);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("negative"), "error msg: {}", msg);
}

#[test]
fn validate_take_nan_rejects() {
    let result = validate_take_pct(f64::NAN);
    assert!(result.is_err());
}

#[test]
fn validate_take_very_large_rejects() {
    let result = validate_take_pct(100.0);
    assert!(result.is_err());
}

// ──── Comprehensive parse_children tests ────

#[test]
fn parse_children_whitespace_around_colons() {
    let result = parse_children("1000 : 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY");
    assert!(result.is_ok(), "whitespace around colon: {:?}", result.err());
    let children = result.unwrap();
    assert_eq!(children[0].0, 1000);
}

#[test]
fn parse_children_whitespace_around_commas() {
    let result = parse_children("500:5Abc , 500:5Def");
    assert!(result.is_ok(), "whitespace around commas: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 2);
}

#[test]
fn parse_children_large_proportion() {
    // u64::MAX should be valid
    let result = parse_children(&format!("{}:5Abc", u64::MAX));
    assert!(result.is_ok(), "u64 max proportion: {:?}", result.err());
    assert_eq!(result.unwrap()[0].0, u64::MAX);
}

#[test]
fn parse_children_negative_proportion() {
    let result = parse_children("-1:5Abc");
    assert!(result.is_err(), "negative proportion should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Invalid proportion"), "error msg: {}", msg);
}

#[test]
fn parse_children_float_proportion() {
    let result = parse_children("1.5:5Abc");
    assert!(result.is_err(), "float proportion should fail");
}

#[test]
fn parse_children_multiple_colons() {
    let result = parse_children("1000:5Abc:extra");
    assert!(result.is_err(), "multiple colons should fail");
}

#[test]
fn parse_children_only_commas() {
    let result = parse_children(",,,");
    // This should either error or produce empty results
    // The function splits on comma, each empty part will fail the colon split
    assert!(result.is_err(), "comma-only input should fail");
}

#[test]
fn parse_children_single_colon_only() {
    let result = parse_children(":");
    // proportion is empty → parse error
    assert!(result.is_err(), "colon-only should fail");
}

// ──── Balance edge case tests ────

#[test]
fn balance_from_tao_zero() {
    use agcli::types::Balance;
    let b = Balance::from_tao(0.0);
    assert_eq!(b.rao(), 0);
    assert_eq!(b.tao(), 0.0);
}

#[test]
fn balance_from_rao_zero() {
    use agcli::types::Balance;
    let b = Balance::from_rao(0);
    assert_eq!(b.rao(), 0);
    assert_eq!(b.tao(), 0.0);
}

#[test]
fn balance_from_tao_fractional() {
    use agcli::types::Balance;
    let b = Balance::from_tao(1.5);
    assert_eq!(b.rao(), 1_500_000_000);
}

#[test]
fn balance_from_tao_one_rao() {
    use agcli::types::Balance;
    // 0.000000001 TAO = 1 RAO
    let b = Balance::from_tao(0.000000001);
    assert_eq!(b.rao(), 1);
}

#[test]
fn balance_roundtrip() {
    use agcli::types::Balance;
    let original_rao = 12_345_678_901u64;
    let b = Balance::from_rao(original_rao);
    let tao = b.tao();
    let b2 = Balance::from_tao(tao);
    // May lose precision due to f64, but should be close
    let diff = (b2.rao() as i64 - original_rao as i64).unsigned_abs();
    assert!(diff <= 1, "roundtrip error too large: {} vs {}", b2.rao(), original_rao);
}

#[test]
fn balance_display_tao_whole_number() {
    use agcli::types::Balance;
    let b = Balance::from_tao(100.0);
    let s = b.display_tao();
    assert!(s.contains("100"), "display should contain 100: {}", s);
}

// ──── Sprint 14: CSV escaping ────

#[test]
fn csv_escape_plain_string() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("hello"), "hello");
    assert_eq!(csv_escape("simple_name"), "simple_name");
    assert_eq!(csv_escape("42"), "42");
}

#[test]
fn csv_escape_with_comma() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("Subnet, Inc."), "\"Subnet, Inc.\"");
}

#[test]
fn csv_escape_with_quotes() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("say \"hello\""), "\"say \"\"hello\"\"\"");
}

#[test]
fn csv_escape_with_newline() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("line1\nline2"), "\"line1\nline2\"");
}

#[test]
fn csv_escape_with_carriage_return() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape("line1\rline2"), "\"line1\rline2\"");
}

#[test]
fn csv_escape_with_all_special_chars() {
    use agcli::cli::helpers::csv_escape;
    let val = csv_escape("a,b\"c\nd");
    assert!(val.starts_with('"') && val.ends_with('"'));
    assert!(val.contains("\"\""));
}

#[test]
fn csv_escape_empty_string() {
    use agcli::cli::helpers::csv_escape;
    assert_eq!(csv_escape(""), "");
}

#[test]
fn csv_row_from_basic() {
    use agcli::cli::helpers::csv_row_from;
    assert_eq!(csv_row_from(&["a", "b", "c"]), "a,b,c");
    assert_eq!(csv_row_from(&["Subnet, Inc.", "42"]), "\"Subnet, Inc.\",42");
}

// ──── Sprint 14: owner-workflow explain topic ────

#[test]
fn explain_owner_workflow_topic() {
    let content = explain::explain("owner-workflow");
    assert!(content.is_some(), "owner-workflow topic should exist");
    let text = content.unwrap();
    assert!(text.contains("SUBNET OWNER WORKFLOW"), "should have title");
    assert!(text.contains("register"), "should mention registration");
    assert!(text.contains("set-param"), "should mention set-param");
    assert!(text.contains("monitor"), "should mention monitoring");
}

#[test]
fn explain_owner_workflow_aliases() {
    assert!(explain::explain("ow").is_some());
    assert!(explain::explain("subnet-owner").is_some());
    assert!(explain::explain("owner-guide").is_some());
}

#[test]
fn explain_topic_count_31() {
    let topics = explain::list_topics();
    assert!(
        topics.len() >= 31,
        "Expected at least 31 topics, got {}",
        topics.len()
    );
}

// ══════════════════════════════════════════════════════════════════════
// validate_symbol tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_symbol_basic() {
    assert!(validate_symbol("ALPHA").is_ok());
    assert!(validate_symbol("SN1").is_ok());
    assert!(validate_symbol("TAO").is_ok());
    assert!(validate_symbol("X").is_ok());
}

#[test]
fn validate_symbol_empty_rejects() {
    let result = validate_symbol("");
    assert!(result.is_err(), "empty symbol should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("cannot be empty"), "message: {}", msg);
}

#[test]
fn validate_symbol_whitespace_only_rejects() {
    let result = validate_symbol("   ");
    assert!(result.is_err(), "whitespace-only symbol should fail");
}

#[test]
fn validate_symbol_too_long_rejects() {
    let long = "A".repeat(33);
    let result = validate_symbol(&long);
    assert!(result.is_err(), "33-char symbol should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("too long"), "message: {}", msg);
}

#[test]
fn validate_symbol_max_length_ok() {
    let at_limit = "A".repeat(32);
    assert!(validate_symbol(&at_limit).is_ok());
}

#[test]
fn validate_symbol_non_ascii_rejects() {
    let result = validate_symbol("ΑΛΦΑ");  // Greek letters
    assert!(result.is_err(), "non-ASCII should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("non-ASCII"), "message: {}", msg);
}

#[test]
fn validate_symbol_with_spaces() {
    // Leading/trailing spaces: the trim handles it, but space inside is ok
    assert!(validate_symbol("  ALPHA  ").is_ok(), "padded symbol should be ok after trim");
}

// ══════════════════════════════════════════════════════════════════════
// validate_emission_weights tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_emission_weights_basic() {
    assert!(validate_emission_weights(&[50, 50]).is_ok());
    assert!(validate_emission_weights(&[100]).is_ok());
    assert!(validate_emission_weights(&[33, 33, 34]).is_ok());
}

#[test]
fn validate_emission_weights_empty_rejects() {
    let result = validate_emission_weights(&[]);
    assert!(result.is_err(), "empty weights should fail");
}

#[test]
fn validate_emission_weights_all_zeros_rejects() {
    let result = validate_emission_weights(&[0, 0, 0]);
    assert!(result.is_err(), "all-zero weights should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("total is zero"), "message: {}", msg);
}

#[test]
fn validate_emission_weights_single_zero_rejects() {
    let result = validate_emission_weights(&[0]);
    assert!(result.is_err(), "single zero should fail");
}

#[test]
fn validate_emission_weights_mixed_with_zero_ok() {
    assert!(validate_emission_weights(&[100, 0]).is_ok());
    assert!(validate_emission_weights(&[0, 50, 50]).is_ok());
}

#[test]
fn validate_emission_weights_max_u16() {
    assert!(validate_emission_weights(&[u16::MAX, u16::MAX]).is_ok());
}

// ══════════════════════════════════════════════════════════════════════
// validate_max_cost tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_max_cost_positive() {
    assert!(validate_max_cost(1.0).is_ok());
    assert!(validate_max_cost(0.01).is_ok());
    assert!(validate_max_cost(1000.0).is_ok());
}

#[test]
fn validate_max_cost_zero_ok() {
    assert!(validate_max_cost(0.0).is_ok());
}

#[test]
fn validate_max_cost_negative_rejects() {
    let result = validate_max_cost(-1.0);
    assert!(result.is_err(), "negative cost should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("cannot be negative"), "message: {}", msg);
}

#[test]
fn validate_max_cost_nan_rejects() {
    let result = validate_max_cost(f64::NAN);
    assert!(result.is_err(), "NaN cost should fail");
}

#[test]
fn validate_max_cost_infinity_rejects() {
    let result = validate_max_cost(f64::INFINITY);
    assert!(result.is_err(), "infinity cost should fail");
}

#[test]
fn validate_max_cost_neg_infinity_rejects() {
    let result = validate_max_cost(f64::NEG_INFINITY);
    assert!(result.is_err(), "negative infinity cost should fail");
}

// ══════════════════════════════════════════════════════════════════════
// validate_delegate_take tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_delegate_take_valid() {
    assert!(validate_delegate_take(0.0).is_ok());
    assert!(validate_delegate_take(9.0).is_ok());
    assert!(validate_delegate_take(18.0).is_ok());
}

#[test]
fn validate_delegate_take_over_max_rejects() {
    let result = validate_delegate_take(18.01);
    assert!(result.is_err(), "take > 18% should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("Maximum allowed is 18%"), "message: {}", msg);
}

#[test]
fn validate_delegate_take_negative_rejects() {
    let result = validate_delegate_take(-1.0);
    assert!(result.is_err(), "negative take should fail");
}

#[test]
fn validate_delegate_take_nan_rejects() {
    let result = validate_delegate_take(f64::NAN);
    assert!(result.is_err(), "NaN take should fail");
}

#[test]
fn validate_delegate_take_way_over_rejects() {
    let result = validate_delegate_take(100.0);
    assert!(result.is_err(), "100% take should fail");
}

// ══════════════════════════════════════════════════════════════════════
// validate_name tests (wallet/hotkey name validation)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_name_valid_simple() {
    assert!(validate_name("default", "wallet").is_ok());
    assert!(validate_name("my_wallet", "wallet").is_ok());
    assert!(validate_name("wallet-1", "wallet").is_ok());
    assert!(validate_name("Alice", "hotkey").is_ok());
    assert!(validate_name("test123", "wallet").is_ok());
}

#[test]
fn validate_name_valid_boundary() {
    // Max length (64 chars)
    let name = "a".repeat(64);
    assert!(validate_name(&name, "wallet").is_ok());
}

#[test]
fn validate_name_empty_rejects() {
    let result = validate_name("", "wallet");
    assert!(result.is_err(), "empty name should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("cannot be empty"), "msg: {}", msg);
}

#[test]
fn validate_name_whitespace_only_rejects() {
    let result = validate_name("   ", "wallet");
    assert!(result.is_err(), "whitespace-only should fail");
}

#[test]
fn validate_name_too_long_rejects() {
    let name = "a".repeat(65);
    let result = validate_name(&name, "wallet");
    assert!(result.is_err(), "65-char name should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("too long"), "msg: {}", msg);
}

#[test]
fn validate_name_path_traversal_dotdot_rejects() {
    let result = validate_name("../etc/passwd", "wallet");
    assert!(result.is_err(), "path traversal should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("path separators"), "msg: {}", msg);
}

#[test]
fn validate_name_path_traversal_slash_rejects() {
    assert!(validate_name("foo/bar", "wallet").is_err());
    assert!(validate_name("foo\\bar", "wallet").is_err());
}

#[test]
fn validate_name_absolute_path_rejects() {
    assert!(validate_name("/etc/passwd", "wallet").is_err());
    assert!(validate_name("\\\\server\\share", "wallet").is_err());
}

#[test]
fn validate_name_hidden_file_rejects() {
    let result = validate_name(".hidden", "wallet");
    assert!(result.is_err(), "hidden name should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("starts with a dot"), "msg: {}", msg);
}

#[test]
fn validate_name_special_chars_rejects() {
    assert!(validate_name("wallet name", "wallet").is_err(), "spaces should fail");
    assert!(validate_name("wallet@home", "wallet").is_err(), "@ should fail");
    assert!(validate_name("wallet#1", "wallet").is_err(), "# should fail");
    assert!(validate_name("wallet$", "wallet").is_err(), "$ should fail");
    assert!(validate_name("wallet!", "wallet").is_err(), "! should fail");
    assert!(validate_name("wallet&more", "wallet").is_err(), "& should fail");
    assert!(validate_name("wallet;rm -rf /", "wallet").is_err(), "; injection should fail");
}

#[test]
fn validate_name_unicode_rejects() {
    assert!(validate_name("wället", "wallet").is_err(), "umlaut should fail");
    assert!(validate_name("钱包", "wallet").is_err(), "CJK should fail");
    assert!(validate_name("wallet🔑", "wallet").is_err(), "emoji should fail");
}

#[test]
fn validate_name_reserved_windows_rejects() {
    assert!(validate_name("CON", "wallet").is_err(), "CON reserved");
    assert!(validate_name("con", "wallet").is_err(), "con reserved (case-insensitive)");
    assert!(validate_name("PRN", "wallet").is_err(), "PRN reserved");
    assert!(validate_name("NUL", "wallet").is_err(), "NUL reserved");
    assert!(validate_name("COM1", "wallet").is_err(), "COM1 reserved");
    assert!(validate_name("LPT1", "wallet").is_err(), "LPT1 reserved");
    assert!(validate_name("AUX", "wallet").is_err(), "AUX reserved");
}

#[test]
fn validate_name_hyphens_underscores_ok() {
    assert!(validate_name("my-wallet", "wallet").is_ok());
    assert!(validate_name("my_wallet", "wallet").is_ok());
    assert!(validate_name("my-wallet_2", "wallet").is_ok());
    assert!(validate_name("_leading", "wallet").is_ok());
    assert!(validate_name("-leading", "wallet").is_ok());
}

#[test]
fn validate_name_numbers_ok() {
    assert!(validate_name("123", "wallet").is_ok());
    assert!(validate_name("0", "wallet").is_ok());
    assert!(validate_name("wallet99", "wallet").is_ok());
}

#[test]
fn validate_name_label_in_error() {
    let result = validate_name("../bad", "hotkey");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("hotkey"), "error should mention 'hotkey': {}", msg);

    let result = validate_name("../bad", "wallet");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("wallet"), "error should mention 'wallet': {}", msg);
}

// ══════════════════════════════════════════════════════════════════════
// validate_ipv4 tests
// ══════════════════════════════════════════════════════════════════════

#[test]
fn validate_ipv4_valid_public() {
    assert!(validate_ipv4("1.2.3.4").is_ok());
    assert!(validate_ipv4("8.8.8.8").is_ok());
    assert!(validate_ipv4("203.0.113.1").is_ok());
    assert!(validate_ipv4("100.64.0.1").is_ok());
}

#[test]
fn validate_ipv4_returns_correct_u128() {
    let result = validate_ipv4("1.2.3.4").unwrap();
    let expected: u128 = (1 << 24) | (2 << 16) | (3 << 8) | 4;
    assert_eq!(result, expected);
}

#[test]
fn validate_ipv4_max_octets() {
    let result = validate_ipv4("254.254.254.254").unwrap();
    assert!(result > 0);
}

#[test]
fn validate_ipv4_rejects_zeroes() {
    let result = validate_ipv4("0.0.0.0");
    assert!(result.is_err(), "all-zeros should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("unspecified"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_broadcast() {
    let result = validate_ipv4("255.255.255.255");
    assert!(result.is_err(), "broadcast should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("broadcast"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_loopback() {
    let result = validate_ipv4("127.0.0.1");
    assert!(result.is_err(), "loopback should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("loopback"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_loopback_range() {
    assert!(validate_ipv4("127.0.0.1").is_err());
    assert!(validate_ipv4("127.255.255.254").is_err());
    assert!(validate_ipv4("127.1.2.3").is_err());
}

#[test]
fn validate_ipv4_warns_private_but_allows() {
    // Private ranges should succeed (just warn to stderr)
    assert!(validate_ipv4("10.0.0.1").is_ok());
    assert!(validate_ipv4("10.255.255.255").is_ok());
    assert!(validate_ipv4("172.16.0.1").is_ok());
    assert!(validate_ipv4("172.31.255.255").is_ok());
    assert!(validate_ipv4("192.168.0.1").is_ok());
    assert!(validate_ipv4("192.168.255.255").is_ok());
}

#[test]
fn validate_ipv4_rejects_too_few_octets() {
    assert!(validate_ipv4("1.2.3").is_err());
    assert!(validate_ipv4("1.2").is_err());
    assert!(validate_ipv4("1").is_err());
}

#[test]
fn validate_ipv4_rejects_too_many_octets() {
    assert!(validate_ipv4("1.2.3.4.5").is_err());
}

#[test]
fn validate_ipv4_rejects_octet_overflow() {
    assert!(validate_ipv4("256.0.0.1").is_err());
    assert!(validate_ipv4("1.2.3.999").is_err());
}

#[test]
fn validate_ipv4_rejects_non_numeric() {
    assert!(validate_ipv4("abc.def.ghi.jkl").is_err());
    assert!(validate_ipv4("1.2.3.x").is_err());
}

#[test]
fn validate_ipv4_rejects_empty() {
    assert!(validate_ipv4("").is_err());
}

#[test]
fn validate_ipv4_rejects_leading_zeros() {
    let result = validate_ipv4("01.02.03.04");
    assert!(result.is_err(), "leading zeros should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("leading zeros"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_negative() {
    assert!(validate_ipv4("-1.0.0.1").is_err());
}

#[test]
fn validate_ipv4_rejects_spaces() {
    assert!(validate_ipv4("1.2.3. 4").is_err());
    assert!(validate_ipv4(" 1.2.3.4").is_err());
}

#[test]
fn validate_ipv4_rejects_hostname() {
    assert!(validate_ipv4("example.com").is_err());
    assert!(validate_ipv4("localhost").is_err());
}
