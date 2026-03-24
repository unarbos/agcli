use agcli::cli::OutputFormat;
use clap::Parser;

#[test]
fn parse_view_health_with_tcp_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "health",
        "--netuid",
        "1",
        "--tcp-check",
    ]);
    assert!(cli.is_ok(), "view health tcp: {:?}", cli.err());
}

#[test]
fn parse_view_emissions_with_limit_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "emissions",
        "--netuid",
        "1",
        "--limit",
        "25",
    ]);
    assert!(cli.is_ok(), "view emissions with limit: {:?}", cli.err());
}

#[test]
fn parse_view_no_subcommand_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view"]);
    assert!(cli.is_err(), "view without subcommand should fail");
}

// =====================================================================
// Weights commands — extra coverage (Step 14)
// =====================================================================

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

// =====================================================================
// Admin commands — extra coverage (Step 14)
// =====================================================================

#[test]
fn parse_admin_set_tempo_with_global_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "local",
        "admin",
        "set-tempo",
        "--netuid",
        "1",
        "--tempo",
        "360",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-tempo global: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_basic_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_tempo",
        "--args",
        "[1, 360]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin raw basic: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_complex_args_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_max_registrations_per_block",
        "--args",
        "[1, 5, true]",
        "--sudo-key",
        "//Bob",
    ]);
    assert!(cli.is_ok(), "admin raw complex: {:?}", cli.err());
}

#[test]
fn parse_admin_list_json_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "admin", "list"]);
    assert!(cli.is_ok(), "admin list json: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_validators_boundary_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-validators",
        "--netuid",
        "1",
        "--max",
        "65535",
    ]);
    assert!(cli.is_ok(), "admin set-max-validators max: {:?}", cli.err());
}

#[test]
fn parse_admin_set_weights_rate_limit_large_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-weights-rate-limit",
        "--netuid",
        "1",
        "--limit",
        "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "admin weights rate u64 max: {:?}", cli.err());
}

#[test]
fn parse_admin_no_subcommand_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "admin"]);
    assert!(cli.is_err(), "admin without subcommand should fail");
}

// =====================================================================
// Diff commands — extra coverage (Step 14)
// =====================================================================

#[test]
fn parse_diff_portfolio_basic_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--block1",
        "100",
        "--block2",
        "200",
    ]);
    assert!(cli.is_ok(), "diff portfolio: {:?}", cli.err());
}

#[test]
fn parse_diff_portfolio_missing_blocks_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "diff portfolio missing blocks should fail");
}

#[test]
fn parse_diff_portfolio_same_block_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--block1",
        "100",
        "--block2",
        "100",
    ]);
    assert!(cli.is_ok(), "diff portfolio same block: {:?}", cli.err());
}

#[test]
fn parse_diff_subnet_max_blocks_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "subnet",
        "--netuid",
        "1",
        "--block1",
        "0",
        "--block2",
        "4294967295",
    ]);
    assert!(cli.is_ok(), "diff subnet max: {:?}", cli.err());
}

// =====================================================================
// Weight parsing — JSON format edge cases (Step 14)
// =====================================================================

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

// ═══════════════════════════════════════════════════════════════════════
//  Step 15 — Handler validation gaps + new validators + comprehensive CLI
// ═══════════════════════════════════════════════════════════════════════

// ── Delegate show (SS58 validation added) ──

#[test]
fn parse_delegate_show_default_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "show"]);
    assert!(cli.is_ok(), "delegate show default: {:?}", cli.err());
}

#[test]
fn parse_delegate_show_with_hotkey_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "delegate",
        "show",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "delegate show with hotkey: {:?}", cli.err());
}

// ── Identity set-subnet (name/URL/GitHub validation added) ──

#[test]
fn parse_identity_set_subnet_basic_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set-subnet",
        "--netuid",
        "1",
        "--name",
        "MySubnet",
    ]);
    assert!(cli.is_ok(), "identity set-subnet basic: {:?}", cli.err());
}

#[test]
fn parse_identity_set_subnet_all_fields_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set-subnet",
        "--netuid",
        "1",
        "--name",
        "MySubnet",
        "--url",
        "https://example.com",
        "--github",
        "org/repo",
    ]);
    assert!(cli.is_ok(), "identity set-subnet all: {:?}", cli.err());
}

#[test]
fn parse_identity_set_subnet_missing_name_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "identity", "set-subnet", "--netuid", "1"]);
    assert!(cli.is_err(), "identity set-subnet missing name should fail");
}

#[test]
fn parse_identity_set_subnet_missing_netuid_s15() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "identity", "set-subnet", "--name", "Test"]);
    assert!(
        cli.is_err(),
        "identity set-subnet missing netuid should fail"
    );
}

#[test]
fn parse_identity_show_with_address_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "show",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "identity show: {:?}", cli.err());
}

// ── Serve reset (netuid validation added) ──

#[test]
fn parse_serve_reset_basic_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "serve", "reset", "--netuid", "1"]);
    assert!(cli.is_ok(), "serve reset: {:?}", cli.err());
}

#[test]
fn parse_serve_reset_missing_netuid_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "serve", "reset"]);
    assert!(cli.is_err(), "serve reset missing netuid should fail");
}

// ── Stake swap (SS58 validation added) ──

#[test]
fn parse_stake_swap_with_both_hotkeys_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap",
        "--from",
        "1",
        "--to",
        "2",
        "--amount",
        "1.0",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "stake swap: {:?}", cli.err());
}

#[test]
fn parse_stake_swap_missing_amount_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap",
        "--from",
        "1",
        "--to",
        "2",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "stake swap missing amount should fail");
}

// ── Subnet pow (thread validation added) ──

#[test]
fn parse_subnet_pow_default_threads_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "pow", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet pow default: {:?}", cli.err());
}

#[test]
fn parse_subnet_pow_custom_threads_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "pow",
        "--netuid",
        "1",
        "--threads",
        "16",
    ]);
    assert!(cli.is_ok(), "subnet pow 16 threads: {:?}", cli.err());
}

#[test]
fn parse_subnet_pow_max_threads_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "pow",
        "--netuid",
        "1",
        "--threads",
        "256",
    ]);
    assert!(cli.is_ok(), "subnet pow max threads: {:?}", cli.err());
}

#[test]
fn parse_subnet_pow_single_thread_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "pow",
        "--netuid",
        "1",
        "--threads",
        "1",
    ]);
    assert!(cli.is_ok(), "subnet pow 1 thread: {:?}", cli.err());
}

// ── Localnet port validation ──

#[test]
fn parse_localnet_start_default_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start"]);
    assert!(cli.is_ok(), "localnet start default: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_custom_port_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--port", "9945"]);
    assert!(cli.is_ok(), "localnet start port 9945: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_all_args_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "start",
        "--image",
        "ghcr.io/opentensor/subtensor-localnet:devnet-ready",
        "--container",
        "my-chain",
        "--port",
        "9944",
        "--wait",
        "true",
        "--timeout",
        "180",
    ]);
    assert!(cli.is_ok(), "localnet start all args: {:?}", cli.err());
}

#[test]
fn parse_localnet_status_custom_port_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "status", "--port", "9945"]);
    assert!(cli.is_ok(), "localnet status port: {:?}", cli.err());
}

#[test]
fn parse_localnet_reset_all_args_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "reset",
        "--image",
        "img:latest",
        "--port",
        "9946",
        "--timeout",
        "60",
    ]);
    assert!(cli.is_ok(), "localnet reset all: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_with_port_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "scaffold", "--port", "9947"]);
    assert!(cli.is_ok(), "localnet scaffold port: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_no_start_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "scaffold", "--no-start"]);
    assert!(cli.is_ok(), "localnet scaffold no-start: {:?}", cli.err());
}

#[test]
fn parse_localnet_logs_default_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "logs"]);
    assert!(cli.is_ok(), "localnet logs: {:?}", cli.err());
}

#[test]
fn parse_localnet_logs_with_tail_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "logs", "--tail", "50"]);
    assert!(cli.is_ok(), "localnet logs tail: {:?}", cli.err());
}

#[test]
fn parse_localnet_stop_default_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "stop"]);
    assert!(cli.is_ok(), "localnet stop: {:?}", cli.err());
}

#[test]
fn parse_localnet_stop_with_container_s15() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "localnet", "stop", "--container", "my-chain"]);
    assert!(cli.is_ok(), "localnet stop container: {:?}", cli.err());
}

// ── View metagraph/emissions limit validation ──

#[test]
fn parse_view_metagraph_with_limit_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "metagraph",
        "--netuid",
        "1",
        "--limit",
        "50",
    ]);
    assert!(cli.is_ok(), "view metagraph limit: {:?}", cli.err());
}

#[test]
fn parse_view_metagraph_limit_and_since_block_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "metagraph",
        "--netuid",
        "1",
        "--limit",
        "100",
        "--since-block",
        "50",
    ]);
    assert!(cli.is_ok(), "view metagraph limit+since: {:?}", cli.err());
}

#[test]
fn parse_view_emissions_with_limit_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "emissions",
        "--netuid",
        "1",
        "--limit",
        "25",
    ]);
    assert!(cli.is_ok(), "view emissions limit: {:?}", cli.err());
}

#[test]
fn parse_view_emissions_no_limit_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "emissions", "--netuid", "1"]);
    assert!(cli.is_ok(), "view emissions default: {:?}", cli.err());
}

// ── Subnet dissolve/check-start/set-param/snipe/emission-split/mechanism-count ──

#[test]
fn parse_subnet_dissolve_basic_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "dissolve", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet dissolve: {:?}", cli.err());
}

#[test]
fn parse_subnet_terminate_lease_basic_s15() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "terminate-lease", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet terminate-lease: {:?}", cli.err());
}

#[test]
fn parse_subnet_check_start_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "check-start", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet check-start: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_tempo_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "tempo",
        "--value",
        "100",
    ]);
    assert!(cli.is_ok(), "subnet set-param tempo: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_list_mode_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "list",
    ]);
    assert!(cli.is_ok(), "subnet set-param list: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_defaults_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "snipe", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet snipe: {:?}", cli.err());
}

#[test]
fn parse_subnet_snipe_all_opts_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "snipe",
        "--netuid",
        "1",
        "--max-cost",
        "1.5",
        "--max-attempts",
        "10",
        "--all-hotkeys",
        "--fast",
    ]);
    assert!(cli.is_ok(), "subnet snipe all: {:?}", cli.err());
}

#[test]
fn parse_subnet_emission_split_view_s15() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "emission-split", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet emission-split: {:?}", cli.err());
}

#[test]
fn parse_subnet_mechanism_count_view_s15() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subnet", "mechanism-count", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet mechanism-count: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_emission_split_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-emission-split",
        "--netuid",
        "1",
        "--weights",
        "100,200",
    ]);
    assert!(cli.is_ok(), "subnet set-emission-split: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_mechanism_count_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-mechanism-count",
        "--netuid",
        "1",
        "--count",
        "3",
    ]);
    assert!(cli.is_ok(), "subnet set-mechanism-count: {:?}", cli.err());
}

#[test]
fn parse_subnet_trim_basic_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "trim",
        "--netuid",
        "1",
        "--max-uids",
        "256",
    ]);
    assert!(cli.is_ok(), "subnet trim: {:?}", cli.err());
}

// ── Proxy/Swap/Stake misc ──

#[test]
fn parse_proxy_add_staking_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Staking",
        "--delay",
        "100",
    ]);
    assert!(cli.is_ok(), "proxy add staking: {:?}", cli.err());
}

#[test]
fn parse_swap_hotkey_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "swap",
        "hotkey",
        "--new-hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "swap hotkey: {:?}", cli.err());
}

#[test]
fn parse_stake_unstake_all_s15() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "unstake-all"]);
    assert!(cli.is_ok(), "stake unstake-all: {:?}", cli.err());
}

#[test]
fn parse_stake_transfer_stake_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "10.0",
        "--from",
        "1",
        "--to",
        "2",
    ]);
    assert!(cli.is_ok(), "stake transfer-stake: {:?}", cli.err());
}

#[test]
fn parse_stake_swap_limit_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "swap-limit",
        "--amount",
        "5.0",
        "--price",
        "1.5",
        "--from",
        "1",
        "--to",
        "2",
    ]);
    assert!(cli.is_ok(), "stake swap-limit: {:?}", cli.err());
}

#[test]
fn parse_view_health_probe_timeout_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "health",
        "--netuid",
        "1",
        "--tcp-check",
        "--probe-timeout-ms",
        "3000",
    ]);
    assert!(cli.is_ok(), "view health probe-timeout: {:?}", cli.err());
}

// ── Serve axon edge cases ──

#[test]
fn parse_serve_axon_all_fields_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "axon",
        "--netuid",
        "1",
        "--ip",
        "192.168.1.1",
        "--port",
        "8080",
    ]);
    assert!(cli.is_ok(), "serve axon all: {:?}", cli.err());
}

#[test]
fn parse_serve_axon_max_port_s15() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "10.0.0.1", "--port", "65535",
    ]);
    assert!(cli.is_ok(), "serve axon max port: {:?}", cli.err());
}

// ── Step 16: Proxy type validation + proxy command coverage ──

#[test]
fn parse_proxy_add_staking_type_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Staking",
    ]);
    assert!(cli.is_ok(), "proxy add staking: {:?}", cli.err());
}

#[test]
fn parse_proxy_add_governance_type_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Governance",
    ]);
    assert!(cli.is_ok(), "proxy add governance: {:?}", cli.err());
}

#[test]
fn parse_proxy_add_nontransfer_type_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "NonTransfer",
    ]);
    assert!(cli.is_ok(), "proxy add nontransfer: {:?}", cli.err());
}

#[test]
fn parse_proxy_add_with_delay_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Staking",
        "--delay",
        "100",
    ]);
    assert!(cli.is_ok(), "proxy add with delay: {:?}", cli.err());
}

#[test]
fn parse_proxy_remove_transfer_type_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "remove",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Transfer",
    ]);
    assert!(cli.is_ok(), "proxy remove transfer: {:?}", cli.err());
}

#[test]
fn parse_proxy_create_pure_any_s16() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "proxy", "create-pure", "--proxy-type", "Any"]);
    assert!(cli.is_ok(), "proxy create-pure any: {:?}", cli.err());
}

#[test]
fn parse_proxy_create_pure_with_delay_index_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "create-pure",
        "--proxy-type",
        "Registration",
        "--delay",
        "50",
        "--index",
        "3",
    ]);
    assert!(
        cli.is_ok(),
        "proxy create-pure delay+index: {:?}",
        cli.err()
    );
}

#[test]
fn parse_proxy_kill_pure_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "kill-pure",
        "--spawner",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Any",
        "--index",
        "0",
        "--height",
        "100",
        "--ext-index",
        "1",
    ]);
    assert!(cli.is_ok(), "proxy kill-pure: {:?}", cli.err());
}

#[test]
fn parse_proxy_announce_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "announce",
        "--real",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "proxy announce: {:?}", cli.err());
}

#[test]
fn parse_proxy_proxy_announced_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "proxy-announced",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--real",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--pallet",
        "SubtensorModule",
        "--call",
        "add_stake",
    ]);
    assert!(cli.is_ok(), "proxy proxy-announced: {:?}", cli.err());
}

#[test]
fn parse_proxy_proxy_announced_with_type_and_args_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "proxy-announced",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--real",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--proxy-type",
        "Staking",
        "--pallet",
        "SubtensorModule",
        "--call",
        "add_stake",
        "--args",
        "[1, 100]",
    ]);
    assert!(cli.is_ok(), "proxy proxy-announced args: {:?}", cli.err());
}

#[test]
fn parse_proxy_reject_announcement_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "reject-announcement",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "proxy reject-announcement: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_announcements_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "list-announcements",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy list-announcements: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_announcements_no_address_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list-announcements"]);
    assert!(
        cli.is_ok(),
        "proxy list-announcements no addr: {:?}",
        cli.err()
    );
}

// ── Proxy type variants for all known types ──

#[test]
fn parse_proxy_add_all_known_types_s16() {
    let types = [
        "Any",
        "Owner",
        "NonTransfer",
        "Staking",
        "NonCritical",
        "Triumvirate",
        "Governance",
        "Senate",
        "NonFungible",
        "Registration",
        "Transfer",
        "SmallTransfer",
        "RootWeights",
        "ChildKeys",
    ];
    for pt in types {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli",
            "proxy",
            "add",
            "--delegate",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            "--proxy-type",
            pt,
        ]);
        assert!(cli.is_ok(), "proxy add type {}: {:?}", pt, cli.err());
    }
}

// ── Config spending limit edge cases ──

#[test]
fn parse_config_set_spending_limit_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "spending_limit.1",
        "--value",
        "100.5",
    ]);
    assert!(cli.is_ok(), "config set spending_limit: {:?}", cli.err());
}

#[test]
fn parse_config_set_network_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "config", "set", "--key", "network", "--value", "finney",
    ]);
    assert!(cli.is_ok(), "config set network: {:?}", cli.err());
}

#[test]
fn parse_config_set_output_s16() {
    for fmt in ["table", "json", "csv"] {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli", "config", "set", "--key", "output", "--value", fmt,
        ]);
        assert!(cli.is_ok(), "config set output {}: {:?}", fmt, cli.err());
    }
}

#[test]
fn parse_config_set_batch_s16() {
    for v in ["true", "false"] {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli", "config", "set", "--key", "batch", "--value", v,
        ]);
        assert!(cli.is_ok(), "config set batch {}: {:?}", v, cli.err());
    }
}

#[test]
fn parse_config_set_live_interval_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "live_interval",
        "--value",
        "5",
    ]);
    assert!(cli.is_ok(), "config set live_interval: {:?}", cli.err());
}

#[test]
fn parse_config_set_wallet_dir_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "wallet_dir",
        "--value",
        "/home/user/.bittensor/wallets",
    ]);
    assert!(cli.is_ok(), "config set wallet_dir: {:?}", cli.err());
}

#[test]
fn parse_config_set_proxy_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "proxy",
        "--value",
        "socks5://127.0.0.1:9050",
    ]);
    assert!(cli.is_ok(), "config set proxy: {:?}", cli.err());
}

#[test]
fn parse_config_unset_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "unset", "--key", "network"]);
    assert!(cli.is_ok(), "config unset: {:?}", cli.err());
}

#[test]
fn parse_config_show_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "show"]);
    assert!(cli.is_ok(), "config show: {:?}", cli.err());
}

// ── Admin raw with args edge cases ──

#[test]
fn parse_admin_raw_with_args_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_tempo",
        "--args",
        "[1, 100]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin raw with args: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_empty_args_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_tempo",
        "--args",
        "[]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin raw empty args: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_bool_args_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_commit_reveal_weights_enabled",
        "--args",
        "[1, true]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin raw bool args: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_string_args_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_weights_version_key",
        "--args",
        "[1, \"test\"]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin raw string args: {:?}", cli.err());
}

#[test]
fn parse_admin_list_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "admin", "list"]);
    assert!(cli.is_ok(), "admin list: {:?}", cli.err());
}

#[test]
fn parse_admin_list_json_output_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "admin", "list"]);
    assert!(cli.is_ok(), "admin list json: {:?}", cli.err());
}

// ── Subnet set-param CLI parsing ──

#[test]
fn parse_subnet_set_param_list_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "list",
    ]);
    assert!(cli.is_ok(), "subnet set-param list: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_tempo_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "tempo",
        "--value",
        "100",
    ]);
    assert!(cli.is_ok(), "subnet set-param tempo: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_max_allowed_uids_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "max_allowed_uids",
        "--value",
        "256",
    ]);
    assert!(
        cli.is_ok(),
        "subnet set-param max_allowed_uids: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_set_param_commit_reveal_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "commit_reveal_weights_enabled",
        "--value",
        "true",
    ]);
    assert!(
        cli.is_ok(),
        "subnet set-param commit_reveal: {:?}",
        cli.err()
    );
}

#[test]
fn parse_subnet_set_param_no_value_s16() {
    // Should parse OK (value is optional at CLI level — handler requires it for non-list params)
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "tempo",
    ]);
    assert!(cli.is_ok(), "subnet set-param no value: {:?}", cli.err());
}

#[test]
fn parse_subnet_set_param_json_output_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "subnet",
        "set-param",
        "--netuid",
        "1",
        "--param",
        "list",
    ]);
    assert!(cli.is_ok(), "subnet set-param json list: {:?}", cli.err());
}

// ── Subnet commits CLI parsing ──

#[test]
fn parse_subnet_commits_default_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "commits", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet commits default: {:?}", cli.err());
}

#[test]
fn parse_subnet_commits_with_hotkey_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "commits",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "subnet commits hotkey: {:?}", cli.err());
}

// ── Subnet cost CLI parsing ──

#[test]
fn parse_subnet_cost_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "cost", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet cost: {:?}", cli.err());
}

// ── Subnet probe CLI parsing ──

#[test]
fn parse_subnet_probe_default_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subnet", "probe", "--netuid", "1"]);
    assert!(cli.is_ok(), "subnet probe default: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_with_uids_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "subnet", "probe", "--netuid", "1", "--uids", "0,1,2,3",
    ]);
    assert!(cli.is_ok(), "subnet probe uids: {:?}", cli.err());
}

#[test]
fn parse_subnet_probe_with_timeout_concurrency_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subnet",
        "probe",
        "--netuid",
        "1",
        "--timeout-ms",
        "5000",
        "--concurrency",
        "32",
    ]);
    assert!(
        cli.is_ok(),
        "subnet probe timeout+concurrency: {:?}",
        cli.err()
    );
}

// ── Wallet derive edge cases ──

#[test]
fn parse_wallet_derive_pubkey_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "derive",
        "--input",
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d",
    ]);
    assert!(cli.is_ok(), "wallet derive pubkey: {:?}", cli.err());
}

#[test]
fn parse_wallet_derive_mnemonic_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "derive",
        "--input", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(cli.is_ok(), "wallet derive mnemonic: {:?}", cli.err());
}

// ── Transfer ──

#[test]
fn parse_transfer_basic_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "1.5",
    ]);
    assert!(cli.is_ok(), "transfer basic: {:?}", cli.err());
}

// ── Completions ──

#[test]
fn parse_completions_bash_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "bash"]);
    assert!(cli.is_ok(), "completions bash: {:?}", cli.err());
}

#[test]
fn parse_completions_zsh_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "zsh"]);
    assert!(cli.is_ok(), "completions zsh: {:?}", cli.err());
}

#[test]
fn parse_completions_fish_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "fish"]);
    assert!(cli.is_ok(), "completions fish: {:?}", cli.err());
}

// ── Explain command ──

#[test]
fn parse_explain_default_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain"]);
    assert!(cli.is_ok(), "explain default: {:?}", cli.err());
}

#[test]
fn parse_explain_with_topic_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "tempo"]);
    assert!(cli.is_ok(), "explain with topic: {:?}", cli.err());
}

#[test]
fn parse_explain_full_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "amm", "--full"]);
    assert!(cli.is_ok(), "explain full: {:?}", cli.err());
}

// ── Swap commands ──

#[test]
fn parse_swap_hotkey_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "swap",
        "hotkey",
        "--new-hotkey",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "swap hotkey: {:?}", cli.err());
}

#[test]
fn parse_swap_coldkey_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "swap",
        "coldkey",
        "--new-coldkey",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "swap coldkey: {:?}", cli.err());
}

// ── Diff commands ──

#[test]
fn parse_diff_portfolio_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--block1",
        "100",
        "--block2",
        "200",
    ]);
    assert!(cli.is_ok(), "diff portfolio: {:?}", cli.err());
}

#[test]
fn parse_diff_portfolio_with_address_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--block1",
        "100",
        "--block2",
        "200",
    ]);
    assert!(cli.is_ok(), "diff portfolio with address: {:?}", cli.err());
}

#[test]
fn parse_diff_subnet_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "subnet", "--netuid", "1", "--block1", "100", "--block2", "200",
    ]);
    assert!(cli.is_ok(), "diff subnet: {:?}", cli.err());
}

#[test]
fn parse_diff_network_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "network", "--block1", "100", "--block2", "200",
    ]);
    assert!(cli.is_ok(), "diff network: {:?}", cli.err());
}

// ── Block commands ──

#[test]
fn parse_block_info_number_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "info", "--number", "100"]);
    assert!(cli.is_ok(), "block info number: {:?}", cli.err());
}

#[test]
fn parse_block_latest_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "latest"]);
    assert!(cli.is_ok(), "block latest: {:?}", cli.err());
}

#[test]
fn parse_block_range_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "block", "range", "--from", "100", "--to", "200",
    ]);
    assert!(cli.is_ok(), "block range: {:?}", cli.err());
}

// ── Subscribe ──

#[test]
fn parse_subscribe_blocks_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "blocks"]);
    assert!(cli.is_ok(), "subscribe blocks: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_s16() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "events"]);
    assert!(cli.is_ok(), "subscribe events: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_with_filter_s16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subscribe",
        "events",
        "--filter",
        "SubtensorModule::*",
    ]);
    assert!(cli.is_ok(), "subscribe events filter: {:?}", cli.err());
}

// ══════════════════════════════════════════════════════════════════════
// Step 17 — Call hash validation, config validation, multisig tests
// ══════════════════════════════════════════════════════════════════════

// ── Proxy announce with call_hash ─────────────────────────────────────

#[test]
fn parse_proxy_announce_with_valid_hash_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "announce",
        "--real",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(cli.is_ok(), "proxy announce: {:?}", cli.err());
}

#[test]
fn parse_proxy_announce_bare_hash_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "announce",
        "--real",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash",
        "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(cli.is_ok(), "proxy announce bare hash: {:?}", cli.err());
}

#[test]
fn parse_proxy_reject_with_hash_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "reject-announcement",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "proxy reject: {:?}", cli.err());
}

// ── Multisig with call_hash ──────────────────────────────────────────

#[test]
fn parse_multisig_approve_with_hash_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "approve",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call-hash",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(cli.is_ok(), "multisig approve: {:?}", cli.err());
}

#[test]
fn parse_multisig_cancel_with_hash_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "cancel",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call-hash",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "--timepoint-height",
        "100",
        "--timepoint-index",
        "1",
    ]);
    assert!(cli.is_ok(), "multisig cancel: {:?}", cli.err());
}

#[test]
fn parse_multisig_approve_missing_hash_fails_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "approve",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
    ]);
    assert!(
        cli.is_err(),
        "multisig approve without call-hash should fail"
    );
}

#[test]
fn parse_multisig_cancel_missing_timepoint_fails_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "cancel",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call-hash",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(
        cli.is_err(),
        "multisig cancel without timepoint should fail"
    );
}

#[test]
fn parse_multisig_execute_with_timepoint_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "execute",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
        "--args",
        "[\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 1000000000]",
        "--timepoint-height",
        "100",
        "--timepoint-index",
        "1",
    ]);
    assert!(
        cli.is_ok(),
        "multisig execute with timepoint: {:?}",
        cli.err()
    );
}

#[test]
fn parse_multisig_execute_without_timepoint_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "execute",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
    ]);
    assert!(
        cli.is_ok(),
        "multisig execute without timepoint (optional): {:?}",
        cli.err()
    );
}

#[test]
fn parse_multisig_list_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "list",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "multisig list: {:?}", cli.err());
}

#[test]
fn parse_multisig_list_missing_address_fails_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "multisig", "list"]);
    assert!(cli.is_err(), "multisig list without address should fail");
}

// ── Config set with validation ──────────────────────────────────────

#[test]
fn parse_config_set_network_valid_values_s17() {
    for network in &["finney", "test", "local", "archive"] {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli", "config", "set", "--key", "network", "--value", network,
        ]);
        assert!(
            cli.is_ok(),
            "config set network {}: {:?}",
            network,
            cli.err()
        );
    }
}

#[test]
fn parse_config_set_endpoint_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "endpoint",
        "--value",
        "wss://example.com:443",
    ]);
    assert!(cli.is_ok(), "config set endpoint: {:?}", cli.err());
}

#[test]
fn parse_config_set_proxy_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "proxy",
        "--value",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "config set proxy: {:?}", cli.err());
}

#[test]
fn parse_config_cache_clear_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "cache-clear"]);
    assert!(cli.is_ok(), "config cache-clear: {:?}", cli.err());
}

#[test]
fn parse_config_cache_info_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "cache-info"]);
    assert!(cli.is_ok(), "config cache-info: {:?}", cli.err());
}

#[test]
fn parse_config_path_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "path"]);
    assert!(cli.is_ok(), "config path: {:?}", cli.err());
}

// ── Proxy list-announcements ─────────────────────────────────────────

#[test]
fn parse_proxy_list_announcements_with_address_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "list-announcements",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "proxy list-announcements with addr: {:?}",
        cli.err()
    );
}

#[test]
fn parse_proxy_list_announcements_default_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list-announcements"]);
    assert!(
        cli.is_ok(),
        "proxy list-announcements default: {:?}",
        cli.err()
    );
}

// ── Admin commands — boundary tests ─────────────────────────────────

#[test]
fn parse_admin_set_tempo_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-tempo",
        "--netuid",
        "1",
        "--tempo",
        "100",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-tempo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_validators_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-validators",
        "--netuid",
        "1",
        "--max",
        "64",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-max-validators: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_uids_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-uids",
        "--netuid",
        "1",
        "--max",
        "4096",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-max-uids: {:?}", cli.err());
}

#[test]
fn parse_admin_set_immunity_period_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-immunity-period",
        "--netuid",
        "1",
        "--period",
        "100",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-immunity-period: {:?}", cli.err());
}

#[test]
fn parse_admin_set_min_weights_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-min-weights",
        "--netuid",
        "1",
        "--min",
        "1",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-min-weights: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_weight_limit_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-weight-limit",
        "--netuid",
        "1",
        "--limit",
        "65535",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-max-weight-limit: {:?}", cli.err());
}

#[test]
fn parse_admin_set_weights_rate_limit_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-weights-rate-limit",
        "--netuid",
        "1",
        "--limit",
        "0",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-weights-rate-limit: {:?}", cli.err());
}

#[test]
fn parse_admin_set_commit_reveal_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-commit-reveal",
        "--netuid",
        "1",
        "--enabled",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-commit-reveal: {:?}", cli.err());
}

#[test]
fn parse_admin_set_difficulty_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-difficulty",
        "--netuid",
        "1",
        "--difficulty",
        "1000000",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-difficulty: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_bonds_moving_avg_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_bonds_moving_average",
        "--args",
        "[1, 900000]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(
        cli.is_ok(),
        "admin raw bonds_moving_average: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_raw_target_regs_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_target_registrations_per_interval",
        "--args",
        "[1, 3]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(
        cli.is_ok(),
        "admin raw target_registrations: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_activity_cutoff_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-activity-cutoff",
        "--netuid",
        "1",
        "--cutoff",
        "5000",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-activity-cutoff: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_serving_rate_limit_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_serving_rate_limit",
        "--args",
        "[1, 10]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin raw serving_rate_limit: {:?}", cli.err());
}

// ── Admin raw with all known call names ──────────────────────────────

#[test]
fn parse_admin_raw_all_known_calls_s17() {
    let known_calls = [
        "sudo_set_tempo",
        "sudo_set_max_allowed_validators",
        "sudo_set_max_allowed_uids",
        "sudo_set_immunity_period",
        "sudo_set_min_allowed_weights",
        "sudo_set_max_weight_limit",
        "sudo_set_weights_set_rate_limit",
        "sudo_set_commit_reveal_weights_enabled",
        "sudo_set_difficulty",
        "sudo_set_bonds_moving_average",
        "sudo_set_target_registrations_per_interval",
        "sudo_set_activity_cutoff",
        "sudo_set_serving_rate_limit",
    ];
    for call in &known_calls {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli",
            "admin",
            "raw",
            "--call",
            call,
            "--args",
            "[1, 100]",
            "--sudo-key",
            "//Alice",
        ]);
        assert!(cli.is_ok(), "admin raw --call {}: {:?}", call, cli.err());
    }
}

// ── Multisig execute with various arg formats ───────────────────────

#[test]
fn parse_multisig_execute_empty_args_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "execute",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--args",
        "[\"hello\"]",
    ]);
    assert!(cli.is_ok(), "multisig execute remark: {:?}", cli.err());
}

#[test]
fn parse_multisig_submit_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "submit",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
        "--args",
        "[\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 1000000000]",
    ]);
    assert!(cli.is_ok(), "multisig submit: {:?}", cli.err());
}

// ── Proxy advanced combinations ──────────────────────────────────────

#[test]
fn parse_proxy_all_types_announce_hash_s17() {
    // Test that proxy announce parsing works with various hash formats
    let hashes = [
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    ];
    for hash in &hashes {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli",
            "proxy",
            "announce",
            "--real",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            "--call-hash",
            hash,
        ]);
        assert!(cli.is_ok(), "proxy announce hash {}: {:?}", hash, cli.err());
    }
}

// ── Config edge cases ───────────────────────────────────────────────

#[test]
fn parse_config_set_live_interval_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "live_interval",
        "--value",
        "5",
    ]);
    assert!(cli.is_ok(), "config set live_interval: {:?}", cli.err());
}

#[test]
fn parse_config_set_batch_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "config", "set", "--key", "batch", "--value", "true",
    ]);
    assert!(cli.is_ok(), "config set batch: {:?}", cli.err());
}

#[test]
fn parse_config_set_wallet_dir_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "wallet_dir",
        "--value",
        "/home/user/.bittensor/wallets",
    ]);
    assert!(cli.is_ok(), "config set wallet_dir: {:?}", cli.err());
}

#[test]
fn parse_config_set_wallet_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "config", "set", "--key", "wallet", "--value", "default",
    ]);
    assert!(cli.is_ok(), "config set wallet: {:?}", cli.err());
}

#[test]
fn parse_config_set_hotkey_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "config", "set", "--key", "hotkey", "--value", "default",
    ]);
    assert!(cli.is_ok(), "config set hotkey: {:?}", cli.err());
}

#[test]
fn parse_config_set_output_all_formats_s17() {
    for fmt in &["table", "json", "csv"] {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli", "config", "set", "--key", "output", "--value", fmt,
        ]);
        assert!(cli.is_ok(), "config set output {}: {:?}", fmt, cli.err());
    }
}

#[test]
fn parse_config_unset_all_keys_s17() {
    let keys = [
        "network",
        "endpoint",
        "wallet_dir",
        "wallet",
        "hotkey",
        "output",
        "proxy",
        "live_interval",
        "batch",
    ];
    for key in &keys {
        let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "unset", "--key", key]);
        assert!(cli.is_ok(), "config unset {}: {:?}", key, cli.err());
    }
}

// ── Multisig submit with multiple signatories ───────────────────────

#[test]
fn parse_multisig_submit_multiple_signatories_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "submit",
        "--others", "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty,5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--threshold", "3",
        "--pallet", "System",
        "--call", "remark",
    ]);
    assert!(
        cli.is_ok(),
        "multisig submit multi signers: {:?}",
        cli.err()
    );
}

// ── Proxy proxy-announced with full args ────────────────────────────

#[test]
fn parse_proxy_announced_full_args_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "proxy-announced",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--real",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--proxy-type",
        "Staking",
        "--pallet",
        "SubtensorModule",
        "--call",
        "add_stake",
        "--args",
        "[\"0xabcd\", 1000000000]",
    ]);
    assert!(cli.is_ok(), "proxy announced full: {:?}", cli.err());
}

// ── Subscribe with all variations ───────────────────────────────────

#[test]
fn parse_subscribe_blocks_with_output_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "blocks", "--output", "json"]);
    assert!(cli.is_ok(), "subscribe blocks json output: {:?}", cli.err());
}

// ── Global flags with various commands ──────────────────────────────

#[test]
fn parse_global_json_output_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "balance"]);
    assert!(cli.is_ok(), "global json output: {:?}", cli.err());
}

#[test]
fn parse_global_csv_output_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "csv", "balance"]);
    assert!(cli.is_ok(), "global csv output: {:?}", cli.err());
}

#[test]
fn parse_global_network_flag_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--network", "test", "balance"]);
    assert!(cli.is_ok(), "global network flag: {:?}", cli.err());
}

#[test]
fn parse_global_endpoint_flag_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--endpoint",
        "wss://custom.node.com:443",
        "balance",
    ]);
    assert!(cli.is_ok(), "global endpoint flag: {:?}", cli.err());
}

#[test]
fn parse_global_wallet_dir_flag_s17() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--wallet-dir", "/custom/wallets", "balance"]);
    assert!(cli.is_ok(), "global wallet dir flag: {:?}", cli.err());
}

#[test]
fn parse_transfer_dry_run_flag_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dry-run",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "1.0",
    ]);
    assert!(cli.is_ok(), "transfer dry-run: {:?}", cli.err());
}

// ── Root commands ────────────────────────────────────────────────────

#[test]
fn parse_root_register_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "register"]);
    assert!(cli.is_ok(), "root register: {:?}", cli.err());
}

#[test]
fn parse_root_weights_s17() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "root", "weights", "--weights", "1:100,2:200"]);
    assert!(cli.is_ok(), "root weights: {:?}", cli.err());
}

#[test]
fn parse_root_weights_missing_arg_fails_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "weights"]);
    assert!(cli.is_err(), "root weights without --weights should fail");
}

// ── Doctor subcommand ───────────────────────────────────────────────

#[test]
fn parse_doctor_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "doctor"]);
    assert!(cli.is_ok(), "doctor: {:?}", cli.err());
}

// ── Update subcommand ───────────────────────────────────────────────

#[test]
fn parse_update_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "update"]);
    assert!(cli.is_ok(), "update: {:?}", cli.err());
}

// ── View with all parameters ───────────────────────────────────────

#[test]
fn parse_view_metagraph_with_limit_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "metagraph",
        "--netuid",
        "1",
        "--limit",
        "10",
    ]);
    assert!(cli.is_ok(), "view metagraph limit: {:?}", cli.err());
}

#[test]
fn parse_view_subnet_analytics_s17() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "subnet-analytics", "--netuid", "1"]);
    assert!(cli.is_ok(), "view subnet-analytics: {:?}", cli.err());
}

#[test]
fn parse_view_portfolio_s17() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "portfolio"]);
    assert!(cli.is_ok(), "view portfolio: {:?}", cli.err());
}

#[test]
fn parse_view_portfolio_with_address_s17() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "view portfolio addr: {:?}", cli.err());
}

// ═══════════════════════════════════════════════════════════════════════
// Step 18 — Expanded CLI tests for weak command groups + netuid validation
// ═══════════════════════════════════════════════════════════════════════

// ── Drand commands ──────────────────────────────────────────────────

#[test]
fn parse_drand_write_pulse_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "drand",
        "write-pulse",
        "--payload",
        "0xdeadbeef",
        "--signature",
        "0xcafebabe",
    ]);
    assert!(cli.is_ok(), "drand write-pulse: {:?}", cli.err());
}

#[test]
fn parse_drand_write_pulse_missing_payload_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "drand",
        "write-pulse",
        "--signature",
        "0xcafebabe",
    ]);
    assert!(cli.is_err(), "drand write-pulse should require --payload");
}

#[test]
fn parse_drand_write_pulse_missing_signature_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "drand",
        "write-pulse",
        "--payload",
        "0xdeadbeef",
    ]);
    assert!(cli.is_err(), "drand write-pulse should require --signature");
}

#[test]
fn parse_drand_no_args_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "drand", "write-pulse"]);
    assert!(cli.is_err(), "drand write-pulse without args should fail");
}

#[test]
fn parse_drand_with_endpoint_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--endpoint",
        "ws://localhost:9944",
        "drand",
        "write-pulse",
        "--payload",
        "0x01",
        "--signature",
        "0x02",
    ]);
    assert!(cli.is_ok(), "drand with endpoint: {:?}", cli.err());
}

#[test]
fn parse_drand_with_wallet_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--wallet",
        "test_wallet",
        "drand",
        "write-pulse",
        "--payload",
        "0xaabb",
        "--signature",
        "0xccdd",
    ]);
    assert!(cli.is_ok(), "drand with wallet: {:?}", cli.err());
}

// ── Balance commands ────────────────────────────────────────────────

#[test]
fn parse_balance_default_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance"]);
    assert!(cli.is_ok(), "balance default: {:?}", cli.err());
}

#[test]
fn parse_balance_with_address_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "balance",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "balance with address: {:?}", cli.err());
}

#[test]
fn parse_balance_with_watch_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch"]);
    assert!(cli.is_ok(), "balance --watch: {:?}", cli.err());
}

#[test]
fn parse_balance_with_watch_interval_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch", "30"]);
    assert!(cli.is_ok(), "balance --watch 30: {:?}", cli.err());
}

#[test]
fn parse_balance_with_threshold_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--threshold", "100.0"]);
    assert!(cli.is_ok(), "balance --threshold: {:?}", cli.err());
}

#[test]
fn parse_balance_with_at_block_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--at-block", "1000000"]);
    assert!(cli.is_ok(), "balance --at-block: {:?}", cli.err());
}

#[test]
fn parse_balance_all_flags_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "balance",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--threshold",
        "50.0",
    ]);
    assert!(cli.is_ok(), "balance all flags: {:?}", cli.err());
}

#[test]
fn parse_balance_json_output_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "balance"]);
    assert!(cli.is_ok(), "balance json: {:?}", cli.err());
}

#[test]
fn parse_balance_csv_output_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "csv", "balance"]);
    assert!(cli.is_ok(), "balance csv: {:?}", cli.err());
}

// ── Completions commands ────────────────────────────────────────────

#[test]
fn parse_completions_bash_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "bash"]);
    assert!(cli.is_ok(), "completions bash: {:?}", cli.err());
}

#[test]
fn parse_completions_zsh_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "zsh"]);
    assert!(cli.is_ok(), "completions zsh: {:?}", cli.err());
}

#[test]
fn parse_completions_fish_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "fish"]);
    assert!(cli.is_ok(), "completions fish: {:?}", cli.err());
}

#[test]
fn parse_completions_powershell_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "powershell"]);
    assert!(cli.is_ok(), "completions powershell: {:?}", cli.err());
}

#[test]
fn parse_completions_missing_shell_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions"]);
    assert!(cli.is_err(), "completions without --shell should fail");
}

#[test]
fn parse_completions_invalid_shell_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "cmd"]);
    assert!(cli.is_err(), "completions --shell cmd should fail");
}

// ── Root commands ───────────────────────────────────────────────────

#[test]
fn parse_root_register_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "register"]);
    assert!(cli.is_ok(), "root register: {:?}", cli.err());
}

#[test]
fn parse_root_weights_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "root", "weights", "--weights", "1:100,2:200"]);
    assert!(cli.is_ok(), "root weights: {:?}", cli.err());
}

#[test]
fn parse_root_weights_missing_arg_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "weights"]);
    assert!(cli.is_err(), "root weights without --weights should fail");
}

#[test]
fn parse_root_register_with_wallet_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--wallet", "mywallet", "root", "register"]);
    assert!(cli.is_ok(), "root register wallet: {:?}", cli.err());
}

#[test]
fn parse_root_weights_with_password_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--password",
        "secret123",
        "root",
        "weights",
        "--weights",
        "1:500",
    ]);
    assert!(cli.is_ok(), "root weights password: {:?}", cli.err());
}

#[test]
fn parse_root_register_with_endpoint_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--endpoint",
        "wss://entrypoint-finney.opentensor.ai:443",
        "root",
        "register",
    ]);
    assert!(cli.is_ok(), "root register endpoint: {:?}", cli.err());
}

#[test]
fn parse_root_weights_json_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "root",
        "weights",
        "--weights",
        r#"{"1": 500, "2": 300}"#,
    ]);
    assert!(cli.is_ok(), "root weights JSON: {:?}", cli.err());
}

#[test]
fn parse_root_register_yes_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "-y", "root", "register"]);
    assert!(cli.is_ok(), "root register -y: {:?}", cli.err());
}

// ── Wallet regen commands ───────────────────────────────────────────

#[test]
fn parse_wallet_regen_coldkey_bare_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "regen-coldkey"]);
    assert!(cli.is_ok(), "regen-coldkey bare: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_coldkey_with_mnemonic_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "regen-coldkey", "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"]);
    assert!(cli.is_ok(), "regen-coldkey mnemonic: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_coldkey_with_password_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "regen-coldkey",
        "--password",
        "my_secure_pass",
    ]);
    assert!(cli.is_ok(), "regen-coldkey password: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_coldkey_all_flags_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "regen-coldkey", "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about", "--password", "secure"]);
    assert!(cli.is_ok(), "regen-coldkey all: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_hotkey_bare_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "regen-hotkey"]);
    assert!(cli.is_ok(), "regen-hotkey bare: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_hotkey_with_name_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "wallet", "regen-hotkey", "--name", "myhot"]);
    assert!(cli.is_ok(), "regen-hotkey name: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_hotkey_with_mnemonic_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "regen-hotkey", "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"]);
    assert!(cli.is_ok(), "regen-hotkey mnemonic: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_hotkey_all_flags_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "regen-hotkey", "--name", "hot2", "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"]);
    assert!(cli.is_ok(), "regen-hotkey all: {:?}", cli.err());
}

// ── Identity commands ───────────────────────────────────────────────

#[test]
fn parse_identity_show_addr_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "show",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "identity show addr: {:?}", cli.err());
}

#[test]
fn parse_identity_show_requires_address_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "identity", "show"]);
    assert!(cli.is_err(), "identity show without --address should fail");
}

#[test]
fn parse_identity_set_name_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "identity", "set", "--name", "MyIdentity"]);
    assert!(cli.is_ok(), "identity set: {:?}", cli.err());
}

#[test]
fn parse_identity_set_subnet_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set-subnet",
        "--netuid",
        "1",
        "--name",
        "MySubnet",
    ]);
    assert!(cli.is_ok(), "identity set-subnet: {:?}", cli.err());
}

#[test]
fn parse_identity_set_all_fields_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set",
        "--name",
        "Test",
        "--url",
        "https://example.com",
        "--github",
        "user/repo",
    ]);
    assert!(cli.is_ok(), "identity set all: {:?}", cli.err());
}

#[test]
fn parse_identity_set_subnet_all_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set-subnet",
        "--netuid",
        "5",
        "--name",
        "SubnetName",
        "--url",
        "https://subnet.io",
        "--github",
        "org/repo",
    ]);
    assert!(cli.is_ok(), "identity set-subnet all: {:?}", cli.err());
}

// ── Delegate commands ───────────────────────────────────────────────

#[test]
fn parse_delegate_show_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "delegate",
        "show",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "delegate show: {:?}", cli.err());
}

#[test]
fn parse_delegate_show_no_hotkey_s18() {
    // hotkey is optional for delegate show
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "show"]);
    assert!(cli.is_ok(), "delegate show no hotkey: {:?}", cli.err());
}

