use agcli::cli::OutputFormat;
use clap::Parser;

#[test]
fn parse_utils_convert_to_rao() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--amount", "1.5", "--to-rao",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_convert_to_tao() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "utils", "convert", "--amount", "1500000000"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_latency() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "latency"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_utils_latency_with_extra() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "utils",
        "latency",
        "--extra",
        "wss://custom.node:9944",
        "--pings",
        "3",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ──── Sprint 26 — explain --full, block pinning, multi-process safety ────

#[test]
fn parse_explain_full_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "stake", "--full"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    if let agcli::cli::Commands::Explain { topic, full } = &cli.unwrap().command {
        assert_eq!(topic.as_deref(), Some("stake"));
        assert!(full);
    } else {
        panic!("wrong command variant");
    }
}

#[test]
fn parse_explain_full_flag_defaults_false() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "tempo"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    if let agcli::cli::Commands::Explain { full, .. } = &cli.unwrap().command {
        assert!(!full);
    } else {
        panic!("wrong command variant");
    }
}

#[test]
fn parse_explain_full_no_topic() {
    // --full without --topic should parse fine (lists all doc files)
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--full"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    if let agcli::cli::Commands::Explain { topic, full } = &cli.unwrap().command {
        assert!(topic.is_none());
        assert!(full);
    } else {
        panic!("wrong command variant");
    }
}

#[test]
fn explain_full_loads_doc_file() {
    // Run from the repo root so docs/commands/ is found
    let result = agcli::utils::explain::explain("stake");
    assert!(result.is_some(), "built-in explain should find 'stake'");
}

#[test]
fn explain_all_topics_have_content() {
    // Every topic in list_topics() should resolve to Some
    for (key, _desc) in agcli::utils::explain::list_topics() {
        let content = agcli::utils::explain::explain(key);
        assert!(content.is_some(), "explain('{}') returned None", key);
        assert!(
            !content.unwrap().is_empty(),
            "explain('{}') returned empty string",
            key
        );
    }
}

#[test]
fn explain_topic_descriptions_unique() {
    // No two topics should share the same description
    let topics = agcli::utils::explain::list_topics();
    let mut descs: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (_key, desc) in &topics {
        assert!(descs.insert(desc), "duplicate description: '{}'", desc);
    }
}

#[test]
fn explain_fuzzy_matching_works() {
    // Substring matching: "cold" should match "coldkey-swap"
    let result = agcli::utils::explain::explain("cold");
    assert!(
        result.is_some(),
        "fuzzy match for 'cold' should find coldkey-swap"
    );
}

#[test]
fn explain_normalization_strips_hyphens_underscores() {
    // "commit-reveal" and "commit_reveal" should both resolve
    let r1 = agcli::utils::explain::explain("commit-reveal");
    let r2 = agcli::utils::explain::explain("commit_reveal");
    assert!(r1.is_some());
    assert!(r2.is_some());
    assert_eq!(r1.unwrap(), r2.unwrap());
}

// ──────── New Feature CLI Parsing Tests ────────

#[test]
fn parse_weights_show() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "weights", "show", "--netuid", "97"]);
    assert!(cli.is_ok(), "weights show: {:?}", cli.err());
}

#[test]
fn parse_view_metagraph() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "metagraph", "--netuid", "97"]);
    assert!(cli.is_ok(), "view metagraph: {:?}", cli.err());
}

#[test]
fn parse_view_metagraph_with_since_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "metagraph",
        "--netuid",
        "97",
        "--since-block",
        "1000000",
    ]);
    assert!(cli.is_ok(), "view metagraph --since-block: {:?}", cli.err());
}

#[test]
fn parse_view_axon_by_uid() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "axon", "--netuid", "97", "--uid", "42"]);
    assert!(cli.is_ok(), "view axon --uid: {:?}", cli.err());
}

#[test]
fn parse_view_health() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "health", "--netuid", "97"]);
    assert!(cli.is_ok(), "view health: {:?}", cli.err());
}

#[test]
fn parse_view_health_with_tcp_check() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "health",
        "--netuid",
        "97",
        "--tcp-check",
        "--probe-timeout-ms",
        "5000",
    ]);
    assert!(cli.is_ok(), "view health --tcp-check: {:?}", cli.err());
}

#[test]
fn parse_view_emissions() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "emissions",
        "--netuid",
        "97",
        "--limit",
        "20",
    ]);
    assert!(cli.is_ok(), "view emissions: {:?}", cli.err());
}

#[test]
fn parse_serve_batch_axon() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "batch-axon",
        "--file",
        "/tmp/axons.json",
    ]);
    assert!(cli.is_ok(), "serve batch-axon: {:?}", cli.err());
}

#[test]
fn parse_diff_metagraph() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "metagraph",
        "--netuid",
        "97",
        "--block1",
        "1000000",
        "--block2",
        "1000100",
    ]);
    assert!(cli.is_ok(), "diff metagraph: {:?}", cli.err());
}

// ──── Commitment Commands ────

#[test]
fn parse_commitment_set() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "commitment",
        "set",
        "--netuid",
        "97",
        "--data",
        "endpoint:http://1.2.3.4:8091,version:1.0",
    ]);
    assert!(cli.is_ok(), "commitment set: {:?}", cli.err());
}

#[test]
fn parse_commitment_get() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "commitment",
        "get",
        "--netuid",
        "97",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "commitment get: {:?}", cli.err());
}

#[test]
fn parse_commitment_list() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "commitment",
        "list",
        "--netuid",
        "97",
    ]);
    assert!(cli.is_ok(), "commitment list: {:?}", cli.err());
}

// ──── Utils Convert Alpha/TAO ────

#[test]
fn parse_utils_convert_tao_to_alpha() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--tao", "10.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "utils convert --tao: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_alpha_to_tao() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--alpha", "500.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "utils convert --alpha: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_tao_to_rao() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--amount", "10.0", "--to-rao",
    ]);
    assert!(cli.is_ok(), "utils convert --to-rao: {:?}", cli.err());
}

// ──── Snipe ────

#[test]
fn parse_subnet_snipe_basic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe", "--netuid", "97"]);
    assert!(cli.is_ok(), "subnet snipe: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_with_max_cost() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "97",
        "--max-cost",
        "1.5",
    ]);
    assert!(cli.is_ok(), "subnet snipe --max-cost: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_with_max_attempts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "1",
        "--max-attempts",
        "100",
    ]);
    assert!(cli.is_ok(), "subnet snipe --max-attempts: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--password",
        "test",
        "--batch",
        "subnet",
        "snipe",
        "--netuid",
        "97",
        "--max-cost",
        "0.5",
        "--max-attempts",
        "50",
    ]);
    assert!(cli.is_ok(), "subnet snipe all opts: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_fast_mode() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe", "--netuid", "97", "--fast"]);
    assert!(cli.is_ok(), "subnet snipe --fast: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_watch_mode() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe", "--netuid", "1", "--watch"]);
    assert!(cli.is_ok(), "subnet snipe --watch: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_watch_with_max_cost() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "1",
        "--watch",
        "--max-cost",
        "2.0",
    ]);
    assert!(
        cli.is_ok(),
        "subnet snipe --watch --max-cost: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_snipe_all_hotkeys() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--password",
        "test",
        "subnet",
        "snipe",
        "--netuid",
        "97",
        "--all-hotkeys",
    ]);
    assert!(cli.is_ok(), "subnet snipe --all-hotkeys: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_fast_with_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--password",
        "test",
        "--batch",
        "subnet",
        "snipe",
        "--netuid",
        "97",
        "--fast",
        "--max-cost",
        "1.0",
        "--max-attempts",
        "25",
        "--all-hotkeys",
    ]);
    assert!(cli.is_ok(), "subnet snipe full combo: {:?}", cli.err());
}

// ──── Comprehensive stake CLI arg edge case tests ────

// ── stake add edge cases ──

#[test]
fn parse_stake_add_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add",
        "--amount",
        "1.5",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "stake add with hotkey: {:?}", cli.err());
}

#[test]
fn parse_stake_add_with_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--password",
        "pw",
        "--mev",
        "stake",
        "add",
        "--amount",
        "10.0",
        "--netuid",
        "42",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--max-slippage",
        "5.0",
    ]);
    assert!(cli.is_ok(), "stake add full combo: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.mev);
}

#[test]
fn parse_stake_add_zero_amount() {
    // 0 amount should parse (chain may reject, but CLI accepts)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "0.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "zero amount should parse: {:?}", cli.err());
}

#[test]
fn parse_stake_add_tiny_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add",
        "--amount",
        "0.000000001",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "tiny amount (1 RAO): {:?}", cli.err());
}

#[test]
fn parse_stake_add_large_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add",
        "--amount",
        "1000000.0",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "large amount: {:?}", cli.err());
}

#[test]
fn parse_stake_add_netuid_zero() {
    // Root network is netuid 0
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "0",
    ]);
    assert!(cli.is_ok(), "netuid 0 (root): {:?}", cli.err());
}

#[test]
fn parse_stake_add_netuid_max() {
    // Max u16 netuid
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "65535",
    ]);
    assert!(cli.is_ok(), "netuid 65535: {:?}", cli.err());
}

#[test]
fn parse_stake_add_netuid_overflow() {
    // Beyond u16 should fail
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "65536",
    ]);
    assert!(cli.is_err(), "netuid 65536 should overflow u16");
}

#[test]
fn parse_stake_add_negative_amount() {
    // Clap treats -1.0 as an unknown argument (the dash is ambiguous),
    // so negative amounts are rejected at the CLI parsing level — good UX.
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "-1.0", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "negative amount should be rejected by clap");
}

#[test]
fn parse_stake_add_non_numeric_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "abc", "--netuid", "1",
    ]);
    assert!(cli.is_err(), "non-numeric amount should fail");
}

#[test]
fn parse_stake_add_non_numeric_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "add", "--amount", "1.0", "--netuid", "abc",
    ]);
    assert!(cli.is_err(), "non-numeric netuid should fail");
}

#[test]
fn parse_stake_add_negative_slippage() {
    // Clap treats negative numbers as unknown args (dash prefix), so this is rejected.
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add",
        "--amount",
        "1.0",
        "--netuid",
        "1",
        "--max-slippage",
        "-1.0",
    ]);
    assert!(cli.is_err(), "negative slippage should be rejected by clap");
}

// ── stake remove edge cases ──

#[test]
fn parse_stake_remove_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "remove",
        "--amount",
        "1.0",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_remove_zero_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "remove", "--amount", "0.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_remove_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "remove", "--amount", "1.0"]);
    assert!(cli.is_err(), "remove without netuid should fail");
    let err = cli.unwrap_err().to_string();
    assert!(
        err.contains("netuid"),
        "error should mention netuid: {}",
        err
    );
}

#[test]
fn parse_stake_remove_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "remove", "--netuid", "1"]);
    assert!(cli.is_err(), "remove without amount should fail");
    let err = cli.unwrap_err().to_string();
    assert!(
        err.contains("amount"),
        "error should mention amount: {}",
        err
    );
}

// ── stake move edge cases ──

#[test]
fn parse_stake_move_same_subnet() {
    // Moving from and to same subnet — semantically odd but should parse
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "move", "--amount", "1.0", "--from", "1", "--to", "1",
    ]);
    assert!(cli.is_ok(), "move same subnet: {:?}", cli.err());
}

#[test]
fn parse_stake_move_missing_from() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "move", "--amount", "1.0", "--to", "2"]);
    assert!(cli.is_err(), "move without --from should fail");
}

#[test]
fn parse_stake_move_missing_to() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "move", "--amount", "1.0", "--from", "1",
    ]);
    assert!(cli.is_err(), "move without --to should fail");
}

#[test]
fn parse_stake_move_missing_amount() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "move", "--from", "1", "--to", "2"]);
    assert!(cli.is_err(), "move without --amount should fail");
}

#[test]
fn parse_stake_move_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "move",
        "--amount",
        "5.0",
        "--from",
        "1",
        "--to",
        "2",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake swap edge cases (swap uses --from/--to subnet UIDs + optional --hotkey-address) ──

#[test]
fn parse_stake_swap_missing_from_subnet() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "swap", "--amount", "1.0", "--to", "2"]);
    assert!(cli.is_err(), "swap without --from should fail");
}

#[test]
fn parse_stake_swap_missing_to_subnet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "swap", "--amount", "1.0", "--from", "1",
    ]);
    assert!(cli.is_err(), "swap without --to should fail");
}

#[test]
fn parse_stake_swap_same_hotkey_ss58_optional() {
    // One optional SS58 for the wallet hotkey; swap is between subnets
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap",
        "--amount",
        "1.0",
        "--from",
        "1",
        "--to",
        "2",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "swap with explicit hotkey-address: {:?}",
        cli.err()
    );
}

#[test]
fn parse_stake_swap_missing_amount_only() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "swap", "--from", "1", "--to", "2"]);
    assert!(cli.is_err(), "swap without --amount should fail");
}

// ── stake list edge cases ──

#[test]
fn parse_stake_list_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "list"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_list_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "list",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_list_with_address_and_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "list",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--at-block",
        "1000000",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_list_json_output() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "stake", "list"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

// ── stake add-limit edge cases ──

#[test]
fn parse_stake_add_limit_missing_price() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add-limit",
        "--amount",
        "1.0",
        "--netuid",
        "1",
        "--partial",
    ]);
    assert!(cli.is_err(), "add-limit without --price should fail");
}

#[test]
fn parse_stake_add_limit_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add-limit",
        "--netuid",
        "1",
        "--price",
        "0.5",
        "--partial",
    ]);
    assert!(cli.is_err(), "add-limit without --amount should fail");
}

#[test]
fn parse_stake_add_limit_zero_price() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add-limit",
        "--amount",
        "1.0",
        "--netuid",
        "1",
        "--price",
        "0.0",
        "--partial",
    ]);
    assert!(cli.is_ok(), "zero price: {:?}", cli.err());
}

#[test]
fn parse_stake_add_limit_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add-limit",
        "--amount",
        "10.0",
        "--netuid",
        "1",
        "--price",
        "0.001",
        "--partial",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake remove-limit edge cases ──

#[test]
fn parse_stake_remove_limit_missing_price() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "remove-limit",
        "--amount",
        "1.0",
        "--netuid",
        "1",
        "--partial",
    ]);
    assert!(cli.is_err(), "remove-limit without --price should fail");
}

#[test]
fn parse_stake_remove_limit_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "remove-limit",
        "--amount",
        "1.0",
        "--price",
        "0.5",
        "--partial",
    ]);
    assert!(cli.is_err(), "remove-limit without --netuid should fail");
}

// ── stake swap-limit edge cases ──

#[test]
fn parse_stake_swap_limit_missing_from() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap-limit",
        "--amount",
        "1.0",
        "--to",
        "2",
        "--price",
        "0.5",
        "--partial",
    ]);
    assert!(cli.is_err(), "swap-limit without --from should fail");
}

#[test]
fn parse_stake_swap_limit_missing_to() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap-limit",
        "--amount",
        "1.0",
        "--from",
        "1",
        "--price",
        "0.5",
        "--partial",
    ]);
    assert!(cli.is_err(), "swap-limit without --to should fail");
}

#[test]
fn parse_stake_swap_limit_missing_price() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap-limit",
        "--amount",
        "1.0",
        "--from",
        "1",
        "--to",
        "2",
        "--partial",
    ]);
    assert!(cli.is_err(), "swap-limit without --price should fail");
}

#[test]
fn parse_stake_swap_limit_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap-limit",
        "--amount",
        "50.0",
        "--from",
        "1",
        "--to",
        "5",
        "--price",
        "1.5",
        "--partial",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake childkey-take edge cases ──

#[test]
fn parse_stake_childkey_take_missing_take() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "childkey-take", "--netuid", "1"]);
    assert!(cli.is_err(), "childkey-take without --take should fail");
}

#[test]
fn parse_stake_childkey_take_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "childkey-take", "--take", "5.0"]);
    assert!(cli.is_err(), "childkey-take without --netuid should fail");
}

#[test]
fn parse_stake_childkey_take_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "childkey-take",
        "--take",
        "0.0",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "zero take: {:?}", cli.err());
}

#[test]
fn parse_stake_childkey_take_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "childkey-take",
        "--take",
        "18.0",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "max take (18%): {:?}", cli.err());
}

#[test]
fn parse_stake_childkey_take_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "childkey-take",
        "--take",
        "10.0",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake set-children edge cases ──

#[test]
fn parse_stake_set_children_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "set-children",
        "--children",
        "1000:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "set-children without --netuid should fail");
}

#[test]
fn parse_stake_set_children_missing_children() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-children", "--netuid", "1"]);
    assert!(cli.is_err(), "set-children without --children should fail");
}

#[test]
fn parse_stake_set_children_multiple() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "stake", "set-children", "--netuid", "1",
        "--children", "500:5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,500:5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake recycle-alpha edge cases ──

#[test]
fn parse_stake_recycle_alpha_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "recycle-alpha", "--netuid", "1"]);
    assert!(cli.is_err(), "recycle-alpha without --amount should fail");
}

#[test]
fn parse_stake_recycle_alpha_missing_netuid() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "recycle-alpha", "--amount", "1.0"]);
    assert!(cli.is_err(), "recycle-alpha without --netuid should fail");
}

// ── stake burn-alpha edge cases ──

#[test]
fn parse_stake_burn_alpha_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "burn-alpha", "--netuid", "1"]);
    assert!(cli.is_err(), "burn-alpha without --amount should fail");
}

#[test]
fn parse_stake_burn_alpha_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "burn-alpha", "--amount", "1.0"]);
    assert!(cli.is_err(), "burn-alpha without --netuid should fail");
}

// ── stake unstake-all edge cases ──

#[test]
fn parse_stake_unstake_all_no_args() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "unstake-all"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_unstake_all_alpha_no_args() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "unstake-all-alpha"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_unstake_all_alpha_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "unstake-all-alpha",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake claim-root edge cases ──

#[test]
fn parse_stake_claim_root_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "claim-root"]);
    assert!(cli.is_err(), "claim-root without --netuid should fail");
}

// ── stake set-auto edge cases ──

#[test]
fn parse_stake_set_auto_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-auto"]);
    assert!(cli.is_err(), "set-auto without --netuid should fail");
}

#[test]
fn parse_stake_set_auto_netuid_zero() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-auto", "--netuid", "0"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake show-auto edge cases ──

#[test]
fn parse_stake_show_auto_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "show-auto"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_show_auto_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "show-auto",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake set-claim edge cases ──

#[test]
fn parse_stake_set_claim_invalid_type() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-claim", "--claim-type", "invalid"]);
    assert!(cli.is_err(), "invalid claim type should fail");
}

#[test]
fn parse_stake_set_claim_keep_subnets_without_subnets() {
    // keep-subnets without --subnets should parse (subnets is optional)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "set-claim",
        "--claim-type",
        "keep-subnets",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_keep_subnets_with_subnets() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "set-claim",
        "--claim-type",
        "keep-subnets",
        "--subnets",
        "1,2,5,10",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_missing_type() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-claim"]);
    assert!(cli.is_err(), "set-claim without --claim-type should fail");
}

// ── stake transfer-stake edge cases ──

#[test]
fn parse_stake_transfer_stake_missing_from() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount",
        "10.0",
        "--to",
        "2",
    ]);
    assert!(cli.is_err(), "transfer-stake without --from should fail");
}

#[test]
fn parse_stake_transfer_stake_missing_to() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount",
        "10.0",
        "--from",
        "1",
    ]);
    assert!(cli.is_err(), "transfer-stake without --to should fail");
}

#[test]
fn parse_stake_transfer_stake_same_subnet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount",
        "10.0",
        "--from",
        "1",
        "--to",
        "1",
    ]);
    assert!(cli.is_ok(), "same subnet transfer: {:?}", cli.err());
}

// ── stake process-claim edge cases ──

#[test]
fn parse_stake_process_claim_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "process-claim",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

#[test]
fn parse_stake_process_claim_with_hotkey_and_netuids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "process-claim",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--netuids",
        "1,5,10",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
}

// ── stake wizard edge cases ──

#[test]
fn parse_stake_wizard_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--password",
        "pass",
        "stake",
        "wizard",
        "--netuid",
        "1",
        "--amount",
        "5.0",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "wizard full non-interactive: {:?}", cli.err());
}

#[test]
fn parse_stake_wizard_partial_flags() {
    // Only netuid — amount and hotkey will be prompted
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--yes", "stake", "wizard", "--netuid", "5"]);
    assert!(cli.is_ok(), "wizard with only netuid: {:?}", cli.err());
}

#[test]
fn parse_stake_wizard_no_flags() {
    // Fully interactive mode
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "wizard"]);
    assert!(cli.is_ok(), "wizard no flags: {:?}", cli.err());
}

// ── global flag combinations with stake commands ──

#[test]
fn parse_stake_add_with_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--dry-run",
        "stake",
        "add",
        "--amount",
        "1.0",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn parse_stake_add_with_batch_mode() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--batch", "stake", "add", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert!(cli.unwrap().batch);
}

#[test]
fn parse_stake_list_with_verbose_and_time() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--verbose", "--time", "stake", "list"]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.verbose);
    assert!(cli.time);
}

#[test]
fn parse_stake_add_with_proxy() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--proxy",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "stake",
        "add",
        "--amount",
        "1.0",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert!(cli.unwrap().proxy.is_some());
}

// ══════════════════════════════════════════════════════════════════════
// Batch 3: Comprehensive subnet command edge cases
// ══════════════════════════════════════════════════════════════════════

// ── subnet list edge cases ──

#[test]
fn parse_subnet_list_with_at_block() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "list", "--at-block", "1000000"]);
    assert!(cli.is_ok(), "list --at-block: {:?}", cli.err());
}

#[test]
fn parse_subnet_list_with_json_output() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "subnet", "list"]);
    assert!(cli.is_ok(), "list --output json: {:?}", cli.err());
}

#[test]
fn parse_subnet_list_at_block_zero() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "list", "--at-block", "0"]);
    assert!(cli.is_ok(), "list --at-block 0: {:?}", cli.err());
}

#[test]
fn parse_subnet_list_at_block_overflow() {
    // u32::MAX+1 should fail
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "list", "--at-block", "4294967296"]);
    assert!(cli.is_err(), "at-block overflow should fail");
}

// ── subnet show edge cases ──

#[test]
fn parse_subnet_show_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "show"]);
    assert!(cli.is_err(), "show without --netuid should fail");
}

#[test]
fn parse_subnet_show_netuid_zero() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "show", "--netuid", "0"]);
    assert!(cli.is_ok(), "show netuid 0 (root): {:?}", cli.err());
}

#[test]
fn parse_subnet_show_netuid_max() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "show", "--netuid", "65535"]);
    assert!(cli.is_ok(), "show netuid max: {:?}", cli.err());
}

#[test]
fn parse_subnet_show_netuid_overflow() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "show", "--netuid", "65536"]);
    assert!(cli.is_err(), "netuid overflow u16 should fail");
}

#[test]
fn parse_subnet_show_netuid_negative() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "show", "--netuid", "-1"]);
    assert!(cli.is_err(), "negative netuid should fail");
}

#[test]
fn parse_subnet_show_netuid_non_numeric() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "show", "--netuid", "abc"]);
    assert!(cli.is_err(), "non-numeric netuid should fail");
}

#[test]
fn parse_subnet_show_with_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "show",
        "--netuid",
        "1",
        "--at-block",
        "500000",
    ]);
    assert!(cli.is_ok(), "show with --at-block: {:?}", cli.err());
}

// ── subnet hyperparams edge cases ──

#[test]
fn parse_subnet_hyperparams_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "hyperparams"]);
    assert!(cli.is_err(), "hyperparams without --netuid should fail");
}

#[test]
fn parse_subnet_hyperparams_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--verbose",
        "--output",
        "json",
        "subnet",
        "hyperparams",
        "--netuid",
        "1",
    ]);
    assert!(
        cli.is_ok(),
        "hyperparams with global flags: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_hyperparams_with_at_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "hyperparams",
        "--netuid",
        "1",
        "--at-block",
        "500000",
    ]);
    assert!(cli.is_ok(), "hyperparams with --at-block: {:?}", cli.err());
}

// ── subnet metagraph edge cases ──

#[test]
fn parse_subnet_metagraph_with_uid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "metagraph",
        "--netuid",
        "1",
        "--uid",
        "0",
    ]);
    assert!(cli.is_ok(), "metagraph with --uid: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_full_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "metagraph",
        "--netuid",
        "1",
        "--full",
    ]);
    assert!(cli.is_ok(), "metagraph --full: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_save_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "metagraph",
        "--netuid",
        "1",
        "--save",
    ]);
    assert!(cli.is_ok(), "metagraph --save: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "metagraph",
        "--netuid",
        "97",
        "--uid",
        "10",
        "--at-block",
        "1000",
        "--full",
        "--save",
    ]);
    assert!(cli.is_ok(), "metagraph all opts: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_uid_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "metagraph",
        "--netuid",
        "1",
        "--uid",
        "65535",
    ]);
    assert!(cli.is_ok(), "metagraph uid max: {:?}", cli.err());
}

#[test]
fn parse_subnet_metagraph_uid_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "metagraph",
        "--netuid",
        "1",
        "--uid",
        "65536",
    ]);
    assert!(cli.is_err(), "metagraph uid overflow should fail");
}

// ── subnet cache commands edge cases ──

#[test]
fn parse_subnet_cache_load_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-load"]);
    assert!(cli.is_err(), "cache-load without --netuid should fail");
}

#[test]
fn parse_subnet_cache_load_with_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "cache-load",
        "--netuid",
        "1",
        "--block",
        "5000000",
    ]);
    assert!(cli.is_ok(), "cache-load block: {:?}", cli.err());
}

#[test]
fn parse_subnet_cache_list_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-list"]);
    assert!(cli.is_err(), "cache-list without --netuid should fail");
}

#[test]
fn parse_subnet_cache_list_with_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-list", "--netuid", "1"]);
    assert!(cli.is_ok(), "cache-list with netuid: {:?}", cli.err());
}

#[test]
fn parse_subnet_cache_load_latest_only() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-load", "--netuid", "1"]);
    assert!(
        cli.is_ok(),
        "cache-load latest (no --block): {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_cache_diff_all_blocks() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "cache-diff",
        "--netuid",
        "1",
        "--from-block",
        "100000",
        "--to-block",
        "200000",
    ]);
    assert!(cli.is_ok(), "cache-diff with blocks: {:?}", cli.err());
}

#[test]
fn parse_subnet_cache_diff_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-diff"]);
    assert!(cli.is_err(), "cache-diff without --netuid should fail");
}

#[test]
fn parse_subnet_cache_prune_custom_keep() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "cache-prune",
        "--netuid",
        "1",
        "--keep",
        "5",
    ]);
    assert!(cli.is_ok(), "cache-prune --keep 5: {:?}", cli.err());
}

#[test]
fn parse_subnet_cache_prune_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cache-prune"]);
    assert!(cli.is_err(), "cache-prune without --netuid should fail");
}

#[test]
fn parse_subnet_cache_prune_keep_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "cache-prune",
        "--netuid",
        "1",
        "--keep",
        "0",
    ]);
    assert!(cli.is_ok(), "cache-prune --keep 0: {:?}", cli.err());
}

// ── subnet probe edge cases ──

#[test]
fn parse_subnet_probe_basic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "probe", "--netuid", "1"]);
    assert!(cli.is_ok(), "probe basic: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_with_uids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "probe", "--netuid", "1", "--uids", "0,1,5,10",
    ]);
    assert!(cli.is_ok(), "probe --uids: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_custom_timeout() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "probe",
        "--netuid",
        "1",
        "--timeout-ms",
        "10000",
    ]);
    assert!(cli.is_ok(), "probe --timeout-ms: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_custom_concurrency() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "probe",
        "--netuid",
        "1",
        "--concurrency",
        "64",
    ]);
    assert!(cli.is_ok(), "probe --concurrency: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "probe",
        "--netuid",
        "97",
        "--uids",
        "0,1,2",
        "--timeout-ms",
        "5000",
        "--concurrency",
        "16",
    ]);
    assert!(cli.is_ok(), "probe all opts: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "probe"]);
    assert!(cli.is_err(), "probe without --netuid should fail");
}

// ── subnet register edge cases ──

#[test]
fn parse_subnet_register_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--password",
        "pw",
        "--batch",
        "--network",
        "finney",
        "subnet",
        "register",
    ]);
    assert!(cli.is_ok(), "register all flags: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
}

#[test]
fn parse_subnet_register_with_identity_name_only() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "register-with-identity",
        "--name",
        "Test SN",
    ]);
    assert!(
        cli.is_ok(),
        "register-with-identity --name only: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_register_with_identity_all_optional_fields() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "register-with-identity",
        "--name",
        "My Subnet",
        "--github",
        "opentensor/subtensor",
        "--url",
        "https://subnet.example",
        "--contact",
        "ops@example.com",
        "--discord",
        "https://discord.gg/example",
        "--description",
        "Short desc",
        "--additional",
        "More info",
    ]);
    assert!(
        cli.is_ok(),
        "register-with-identity all fields: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_register_with_identity_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--password",
        "pw",
        "--batch",
        "subnet",
        "register-with-identity",
        "--name",
        "Flagged",
    ]);
    assert!(
        cli.is_ok(),
        "register-with-identity global flags: {:?}",
        cli.err()
    );
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
}

#[test]
fn parse_subnet_register_leased_basic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "register-leased"]);
    assert!(
        cli.is_ok(),
        "should parse subnet register-leased: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_register_leased_end_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "register-leased",
        "--end-block",
        "12345678",
    ]);
    assert!(
        cli.is_ok(),
        "register-leased with --end-block: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_register_leased_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--password",
        "pw",
        "--batch",
        "subnet",
        "register-leased",
        "--end-block",
        "99",
    ]);
    assert!(cli.is_ok(), "register-leased all flags: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
}

#[test]
fn parse_subnet_create_cost() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "create-cost"]);
    assert!(
        cli.is_ok(),
        "should parse subnet create-cost: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_create_cost_json_output() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "subnet", "create-cost"]);
    assert!(
        cli.is_ok(),
        "create-cost with global json output: {:?}",
        cli.err()
    );
}

// ── subnet register-neuron edge cases ──

#[test]
fn parse_subnet_register_neuron_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "register-neuron"]);
    assert!(cli.is_err(), "register-neuron without --netuid should fail");
}

#[test]
fn parse_subnet_register_neuron_netuid_zero() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "register-neuron", "--netuid", "0"]);
    assert!(cli.is_ok(), "register-neuron netuid 0: {:?}", cli.err());
}

#[test]
fn parse_subnet_register_neuron_with_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--dry-run",
        "subnet",
        "register-neuron",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "{:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

// ── subnet pow edge cases ──

#[test]
fn parse_subnet_pow_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "pow"]);
    assert!(cli.is_err(), "pow without --netuid should fail");
}

#[test]
fn parse_subnet_pow_custom_threads() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "pow",
        "--netuid",
        "1",
        "--threads",
        "16",
    ]);
    assert!(cli.is_ok(), "pow --threads 16: {:?}", cli.err());
}

#[test]
fn parse_subnet_pow_one_thread() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "pow",
        "--netuid",
        "1",
        "--threads",
        "1",
    ]);
    assert!(cli.is_ok(), "pow --threads 1: {:?}", cli.err());
}

#[test]
fn parse_subnet_pow_zero_threads() {
    // 0 thread should parse (runtime may reject)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "pow",
        "--netuid",
        "1",
        "--threads",
        "0",
    ]);
    assert!(cli.is_ok(), "pow --threads 0: {:?}", cli.err());
}

// ── subnet dissolve edge cases ──

#[test]
fn parse_subnet_dissolve_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "dissolve"]);
    assert!(cli.is_err(), "dissolve without --netuid should fail");
}

#[test]
fn parse_subnet_dissolve_with_batch() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--yes", "--batch", "subnet", "dissolve", "--netuid", "5",
    ]);
    assert!(cli.is_ok(), "dissolve with batch: {:?}", cli.err());
}

// ── subnet terminate-lease edge cases ──

#[test]
fn parse_subnet_terminate_lease_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "terminate-lease"]);
    assert!(cli.is_err(), "terminate-lease without --netuid should fail");
}

#[test]
fn parse_subnet_terminate_lease_with_batch() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--batch",
        "subnet",
        "terminate-lease",
        "--netuid",
        "5",
    ]);
    assert!(cli.is_ok(), "terminate-lease with batch: {:?}", cli.err());
}

// ── subnet watch edge cases ──

#[test]
fn parse_subnet_watch_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "watch"]);
    assert!(cli.is_err(), "watch without --netuid should fail");
}

#[test]
fn parse_subnet_watch_default_interval() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "watch", "--netuid", "1"]);
    assert!(cli.is_ok(), "watch default interval: {:?}", cli.err());
}

#[test]
fn parse_subnet_watch_one_sec_interval() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "watch",
        "--netuid",
        "1",
        "--interval",
        "1",
    ]);
    assert!(cli.is_ok(), "watch --interval 1: {:?}", cli.err());
}

// ── subnet liquidity edge cases ──

#[test]
fn parse_subnet_liquidity_missing_netuid_ok() {
    // netuid is optional for all-subnet view
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "liquidity"]);
    assert!(
        cli.is_ok(),
        "liquidity without netuid (all): {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_liquidity_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "subnet",
        "liquidity",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "liquidity json: {:?}", cli.err());
}

// ── subnet monitor edge cases ──

#[test]
fn parse_subnet_monitor_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "monitor"]);
    assert!(cli.is_err(), "monitor without --netuid should fail");
}

#[test]
fn parse_subnet_monitor_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "monitor",
        "--netuid",
        "97",
        "--interval",
        "60",
        "--json",
    ]);
    assert!(cli.is_ok(), "monitor all opts: {:?}", cli.err());
}

// ── subnet health edge cases ──

#[test]
fn parse_subnet_health_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "health"]);
    assert!(cli.is_err(), "health without --netuid should fail");
}

#[test]
fn parse_subnet_health_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "subnet", "health", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "health json: {:?}", cli.err());
}

// ── subnet emissions edge cases ──

#[test]
fn parse_subnet_emissions_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "emissions"]);
    assert!(cli.is_err(), "emissions without --netuid should fail");
}

// ── subnet cost edge cases ──

#[test]
fn parse_subnet_cost_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cost"]);
    assert!(cli.is_err(), "cost without --netuid should fail");
}

#[test]
fn parse_subnet_cost_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "json", "subnet", "cost", "--netuid", "42",
    ]);
    assert!(cli.is_ok(), "cost json: {:?}", cli.err());
}

// ── subnet commits edge cases ──

#[test]
fn parse_subnet_commits_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "commits"]);
    assert!(cli.is_err(), "commits without --netuid should fail");
}

#[test]
fn parse_subnet_commits_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "subnet",
        "commits",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "commits json + hotkey: {:?}", cli.err());
}

// ── subnet set-param comprehensive edge cases ──

#[test]
fn parse_subnet_set_param_u64_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "min_difficulty",
        "--value",
        "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "set-param u64 max: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_all_bool_variants() {
    for val in &["true", "false", "1", "0", "yes", "no", "on", "off"] {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli",
            "subnet",
            "set-param",
            "--netuid",
            "1",
            "--param",
            "registration_allowed",
            "--value",
            val,
        ]);
        assert!(
            cli.is_ok(),
            "set-param bool {} should parse: {:?}",
            val,
            cli.err()
        );
    }
}

#[test]
fn parse_subnet_set_param_with_batch_and_yes() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--batch",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "tempo",
        "--value",
        "100",
    ]);
    assert!(cli.is_ok(), "set-param batch+yes: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
}

// ── subnet set-symbol edge cases ──

#[test]
fn parse_subnet_set_symbol() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-symbol",
        "--netuid",
        "1",
        "--symbol",
        "ALPHA",
    ]);
    assert!(cli.is_ok(), "set-symbol: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_symbol_missing_netuid() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "set-symbol", "--symbol", "ALPHA"]);
    assert!(cli.is_err(), "set-symbol without --netuid should fail");
}

#[test]
fn parse_subnet_set_symbol_missing_symbol() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "set-symbol", "--netuid", "1"]);
    assert!(cli.is_err(), "set-symbol without --symbol should fail");
}

#[test]
fn parse_subnet_set_symbol_empty_string() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-symbol",
        "--netuid",
        "1",
        "--symbol",
        "",
    ]);
    // Empty string should parse (validation at runtime)
    assert!(cli.is_ok(), "set-symbol empty: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_symbol_long_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-symbol",
        "--netuid",
        "1",
        "--symbol",
        "VERYLONGSYMBOLNAME",
    ]);
    assert!(cli.is_ok(), "set-symbol long: {:?}", cli.err());
}

// ── subnet emission-split edge cases ──

#[test]
fn parse_subnet_emission_split_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "emission-split"]);
    assert!(cli.is_err(), "emission-split without --netuid should fail");
}

#[test]
fn parse_subnet_emission_split_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "subnet",
        "emission-split",
        "--netuid",
        "2",
    ]);
    assert!(cli.is_ok(), "emission-split json: {:?}", cli.err());
}

// ── subnet trim edge cases ──

#[test]
fn parse_subnet_trim_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "trim", "--max-uids", "256"]);
    assert!(cli.is_err(), "trim without --netuid should fail");
}

#[test]
fn parse_subnet_trim_missing_max_uids() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "trim", "--netuid", "1"]);
    assert!(cli.is_err(), "trim without --max-uids should fail");
}

#[test]
fn parse_subnet_trim_zero_max_uids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "trim",
        "--netuid",
        "1",
        "--max-uids",
        "0",
    ]);
    assert!(cli.is_ok(), "trim --max-uids 0: {:?}", cli.err());
}

#[test]
fn parse_subnet_trim_max_uids_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "trim",
        "--netuid",
        "1",
        "--max-uids",
        "65536",
    ]);
    assert!(cli.is_err(), "trim --max-uids overflow should fail");
}

// ── subnet check-start edge cases ──

#[test]
fn parse_subnet_check_start_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "check-start"]);
    assert!(cli.is_err(), "check-start without --netuid should fail");
}

#[test]
fn parse_subnet_check_start_with_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "subnet",
        "check-start",
        "--netuid",
        "5",
    ]);
    assert!(cli.is_ok(), "check-start json: {:?}", cli.err());
}

// ── subnet start edge cases ──

#[test]
fn parse_subnet_start_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "start"]);
    assert!(cli.is_err(), "start without --netuid should fail");
}

// ── subnet mechanism-count edge cases ──

#[test]
fn parse_subnet_mechanism_count_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "mechanism-count"]);
    assert!(cli.is_err(), "mechanism-count without --netuid should fail");
}

#[test]
fn parse_subnet_mechanism_count_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "subnet",
        "mechanism-count",
        "--netuid",
        "3",
    ]);
    assert!(cli.is_ok(), "mechanism-count json: {:?}", cli.err());
}

// ── subnet set-mechanism-count edge cases ──

#[test]
fn parse_subnet_set_mechanism_count_missing_netuid() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "set-mechanism-count", "--count", "2"]);
    assert!(
        cli.is_err(),
        "set-mechanism-count without --netuid should fail"
    );
}

#[test]
fn parse_subnet_set_mechanism_count_missing_count() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-mechanism-count",
        "--netuid",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "set-mechanism-count without --count should fail"
    );
}

#[test]
fn parse_subnet_set_mechanism_count_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-mechanism-count",
        "--netuid",
        "1",
        "--count",
        "0",
    ]);
    assert!(cli.is_ok(), "set-mechanism-count 0: {:?}", cli.err());
}

// ── subnet set-emission-split edge cases ──

#[test]
fn parse_subnet_set_emission_split_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-emission-split",
        "--weights",
        "50,50",
    ]);
    assert!(
        cli.is_err(),
        "set-emission-split without --netuid should fail"
    );
}

#[test]
fn parse_subnet_set_emission_split_missing_weights() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "set-emission-split", "--netuid", "1"]);
    assert!(
        cli.is_err(),
        "set-emission-split without --weights should fail"
    );
}

#[test]
fn parse_subnet_set_emission_split_three_way() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-emission-split",
        "--netuid",
        "1",
        "--weights",
        "33,33,34",
    ]);
    assert!(cli.is_ok(), "emission-split 3-way: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_emission_split_single() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-emission-split",
        "--netuid",
        "1",
        "--weights",
        "100",
    ]);
    assert!(cli.is_ok(), "emission-split single: {:?}", cli.err());
}

// ── subnet snipe missing netuid ──

#[test]
fn parse_subnet_snipe_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe"]);
    assert!(cli.is_err(), "snipe without --netuid should fail");
}

#[test]
fn parse_subnet_snipe_max_cost_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "1",
        "--max-cost",
        "0.0",
    ]);
    assert!(cli.is_ok(), "snipe --max-cost 0: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_max_cost_negative() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "1",
        "--max-cost",
        "-1.0",
    ]);
    // Clap rejects negative floats (treats -1.0 as unknown arg) — catches user error early
    assert!(
        cli.is_err(),
        "snipe --max-cost -1 should be rejected by clap"
    );
}

#[test]
fn parse_subnet_snipe_max_attempts_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "1",
        "--max-attempts",
        "0",
    ]);
    assert!(cli.is_ok(), "snipe --max-attempts 0: {:?}", cli.err());
}

// ══════════════════════════════════════════════════════════════════════
// Batch 4: Weight commands comprehensive tests
// ══════════════════════════════════════════════════════════════════════

// ── weights set edge cases ──

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

// ── weights commit edge cases ──

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

// ── weights reveal edge cases ──

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

// ── weights show edge cases ──

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
    assert!(
        cli.is_ok(),
        "weights show --hotkey-address: {:?}",
        cli.err()
    );
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

// ── weights status edge cases ──

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

// ── weights set-mechanism ──

#[test]
fn parse_weights_set_mechanism_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set-mechanism",
        "--netuid",
        "1",
        "--mechanism-id",
        "0",
        "--weights",
        "0:100",
    ]);
    assert!(cli.is_ok(), "set-mechanism basic: {:?}", cli.err());
}

#[test]
fn parse_weights_set_mechanism_with_version_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set-mechanism",
        "--netuid",
        "2",
        "--mechanism-id",
        "1",
        "--weights",
        "0:50,1:50",
        "--version-key",
        "7",
    ]);
    assert!(cli.is_ok(), "set-mechanism --version-key: {:?}", cli.err());
}

#[test]
fn parse_weights_set_mechanism_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set-mechanism",
        "--mechanism-id",
        "0",
        "--weights",
        "0:100",
    ]);
    assert!(cli.is_err(), "set-mechanism without --netuid should fail");
}

#[test]
fn parse_weights_set_mechanism_missing_mechanism_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set-mechanism",
        "--netuid",
        "1",
        "--weights",
        "0:100",
    ]);
    assert!(
        cli.is_err(),
        "set-mechanism without --mechanism-id should fail"
    );
}

#[test]
fn parse_weights_set_mechanism_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "set-mechanism",
        "--netuid",
        "1",
        "--mechanism-id",
        "0",
    ]);
    assert!(cli.is_err(), "set-mechanism without --weights should fail");
}

// ── weights commit-mechanism ──

const COMMIT_MECH_HASH_32: &str =
    "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

#[test]
fn parse_weights_commit_mechanism_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-mechanism",
        "--netuid",
        "1",
        "--mechanism-id",
        "0",
        "--hash",
        COMMIT_MECH_HASH_32,
    ]);
    assert!(cli.is_ok(), "commit-mechanism basic: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_mechanism_hash_with_0x_prefix() {
    let h = format!("0x{COMMIT_MECH_HASH_32}");
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-mechanism",
        "--netuid",
        "2",
        "--mechanism-id",
        "1",
        "--hash",
        &h,
    ]);
    assert!(cli.is_ok(), "commit-mechanism 0x hash: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_mechanism_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-mechanism",
        "--mechanism-id",
        "0",
        "--hash",
        COMMIT_MECH_HASH_32,
    ]);
    assert!(
        cli.is_err(),
        "commit-mechanism without --netuid should fail"
    );
}

#[test]
fn parse_weights_commit_mechanism_missing_mechanism_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-mechanism",
        "--netuid",
        "1",
        "--hash",
        COMMIT_MECH_HASH_32,
    ]);
    assert!(
        cli.is_err(),
        "commit-mechanism without --mechanism-id should fail"
    );
}

#[test]
fn parse_weights_commit_mechanism_missing_hash() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-mechanism",
        "--netuid",
        "1",
        "--mechanism-id",
        "0",
    ]);
    assert!(cli.is_err(), "commit-mechanism without --hash should fail");
}

// ── weights reveal-mechanism ──

#[test]
fn parse_weights_reveal_mechanism_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal-mechanism",
        "--netuid",
        "1",
        "--mechanism-id",
        "0",
        "--weights",
        "0:65535",
        "--salt",
        "e2e-mech-commit",
    ]);
    assert!(cli.is_ok(), "reveal-mechanism basic: {:?}", cli.err());
}

#[test]
fn parse_weights_reveal_mechanism_with_version_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal-mechanism",
        "--netuid",
        "2",
        "--mechanism-id",
        "1",
        "--weights",
        "0:100,1:200",
        "--salt",
        "ab",
        "--version-key",
        "7",
    ]);
    assert!(
        cli.is_ok(),
        "reveal-mechanism with version-key: {:?}",
        cli.err()
    );
}

#[test]
fn parse_weights_reveal_mechanism_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal-mechanism",
        "--mechanism-id",
        "0",
        "--weights",
        "0:1",
        "--salt",
        "x",
    ]);
    assert!(
        cli.is_err(),
        "reveal-mechanism without --netuid should fail"
    );
}
