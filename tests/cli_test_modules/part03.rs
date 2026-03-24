use agcli::cli::OutputFormat;
use clap::Parser;

#[test]
fn parse_weights_reveal_mechanism_missing_mechanism_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal-mechanism",
        "--netuid",
        "1",
        "--weights",
        "0:1",
        "--salt",
        "x",
    ]);
    assert!(
        cli.is_err(),
        "reveal-mechanism without --mechanism-id should fail"
    );
}

#[test]
fn parse_weights_reveal_mechanism_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal-mechanism",
        "--netuid",
        "1",
        "--mechanism-id",
        "0",
        "--salt",
        "x",
    ]);
    assert!(
        cli.is_err(),
        "reveal-mechanism without --weights should fail"
    );
}

#[test]
fn parse_weights_reveal_mechanism_missing_salt() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "reveal-mechanism",
        "--netuid",
        "1",
        "--mechanism-id",
        "0",
        "--weights",
        "0:1",
    ]);
    assert!(cli.is_err(), "reveal-mechanism without --salt should fail");
}

// ── weights commit-timelocked ──

#[test]
fn parse_weights_commit_timelocked_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-timelocked",
        "--netuid",
        "1",
        "--weights",
        "0:100",
        "--round",
        "99",
    ]);
    assert!(cli.is_ok(), "commit-timelocked basic: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_timelocked_with_salt() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-timelocked",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:50",
        "--round",
        "12345",
        "--salt",
        "abc",
    ]);
    assert!(cli.is_ok(), "commit-timelocked --salt: {:?}", cli.err());
}

#[test]
fn parse_weights_commit_timelocked_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-timelocked",
        "--weights",
        "0:100",
        "--round",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "commit-timelocked without --netuid should fail"
    );
}

#[test]
fn parse_weights_commit_timelocked_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-timelocked",
        "--netuid",
        "1",
        "--round",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "commit-timelocked without --weights should fail"
    );
}

#[test]
fn parse_weights_commit_timelocked_missing_round() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "weights",
        "commit-timelocked",
        "--netuid",
        "1",
        "--weights",
        "0:100",
    ]);
    assert!(
        cli.is_err(),
        "commit-timelocked without --round should fail"
    );
}

// ── weights commit-reveal (atomic) edge cases ──

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

// ══════════════════════════════════════════════════════════════════════
// Batch 5: Delegate, proxy, root, identity, serve commands
// ══════════════════════════════════════════════════════════════════════

// ── delegate show ──

#[test]
fn parse_delegate_show_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "delegate",
        "show",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "delegate show --hotkey-address: {:?}",
        cli.err()
    );
}

// ── delegate list ──

#[test]
fn parse_delegate_list_with_json() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "delegate", "list"]);
    assert!(cli.is_ok(), "delegate list json: {:?}", cli.err());
}

// ── delegate decrease-take ──

#[test]
fn parse_delegate_decrease_take_basic() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "delegate", "decrease-take", "--take", "5.0"]);
    assert!(cli.is_ok(), "decrease-take: {:?}", cli.err());
}

#[test]
fn parse_delegate_decrease_take_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "delegate",
        "decrease-take",
        "--take",
        "10.0",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "decrease-take --hotkey-address: {:?}",
        cli.err()
    );
}

#[test]
fn parse_delegate_decrease_take_missing_take() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "decrease-take"]);
    assert!(cli.is_err(), "decrease-take without --take should fail");
}

#[test]
fn parse_delegate_decrease_take_zero() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "delegate", "decrease-take", "--take", "0.0"]);
    assert!(cli.is_ok(), "decrease-take 0: {:?}", cli.err());
}

// ── delegate increase-take ──

#[test]
fn parse_delegate_increase_take_basic() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "delegate", "increase-take", "--take", "15.0"]);
    assert!(cli.is_ok(), "increase-take: {:?}", cli.err());
}

#[test]
fn parse_delegate_increase_take_missing_take() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "increase-take"]);
    assert!(cli.is_err(), "increase-take without --take should fail");
}

// ── root commands ──

#[test]
fn parse_root_weights() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "root", "weights", "--weights", "1:100,2:200"]);
    assert!(cli.is_ok(), "root weights: {:?}", cli.err());
}

#[test]
fn parse_root_weights_missing_weights() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "weights"]);
    assert!(cli.is_err(), "root weights without --weights should fail");
}

#[test]
fn parse_root_register_with_global_flags() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--yes", "--password", "pw", "root", "register"]);
    assert!(cli.is_ok(), "root register flags: {:?}", cli.err());
}

// ── identity commands ──

#[test]
fn parse_identity_set_basic() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "identity", "set", "--name", "MyValidator"]);
    assert!(cli.is_ok(), "identity set: {:?}", cli.err());
}

#[test]
fn parse_identity_set_all_fields() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set",
        "--name",
        "MyValidator",
        "--url",
        "https://example.com",
        "--github",
        "myuser",
        "--description",
        "My awesome validator",
    ]);
    assert!(cli.is_ok(), "identity set all: {:?}", cli.err());
}

#[test]
fn parse_identity_set_missing_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set",
        "--url",
        "https://example.com",
    ]);
    assert!(cli.is_err(), "identity set without --name should fail");
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "identity", "set"]);
    assert!(
        cli.is_err(),
        "identity set with no args should fail: {:?}",
        cli.err()
    );
}

#[test]
fn parse_identity_show_missing_address() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "identity", "show"]);
    assert!(cli.is_err(), "identity show without --address should fail");
}

#[test]
fn parse_identity_set_subnet_all_fields() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "identity",
        "set-subnet",
        "--netuid",
        "1",
        "--name",
        "MySN",
        "--github",
        "myrepo",
        "--url",
        "https://sn1.example.com",
    ]);
    assert!(cli.is_ok(), "identity set-subnet all: {:?}", cli.err());
}

#[test]
fn parse_identity_set_subnet_missing_netuid() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "identity", "set-subnet", "--name", "MySN"]);
    assert!(cli.is_err(), "set-subnet without --netuid should fail");
}

#[test]
fn parse_identity_set_subnet_missing_name() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "identity", "set-subnet", "--netuid", "1"]);
    assert!(cli.is_err(), "set-subnet without --name should fail");
}

// ── serve commands ──

#[test]
fn parse_serve_axon_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "axon",
        "--netuid",
        "1",
        "--ip",
        "192.168.1.1",
        "--port",
        "8091",
    ]);
    assert!(cli.is_ok(), "serve axon: {:?}", cli.err());
}

#[test]
fn parse_serve_axon_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "axon",
        "--netuid",
        "1",
        "--ip",
        "10.0.0.1",
        "--port",
        "8091",
        "--protocol",
        "4",
        "--version",
        "1",
    ]);
    assert!(cli.is_ok(), "serve axon all: {:?}", cli.err());
}

#[test]
fn parse_serve_axon_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--ip", "1.2.3.4", "--port", "8091",
    ]);
    assert!(cli.is_err(), "serve axon without --netuid should fail");
}

#[test]
fn parse_serve_axon_missing_ip() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--port", "8091",
    ]);
    assert!(cli.is_err(), "serve axon without --ip should fail");
}

#[test]
fn parse_serve_axon_missing_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "1.2.3.4",
    ]);
    assert!(cli.is_err(), "serve axon without --port should fail");
}

#[test]
fn parse_serve_axon_port_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "1.2.3.4", "--port", "65536",
    ]);
    assert!(cli.is_err(), "serve axon port overflow should fail");
}

#[test]
fn parse_serve_axon_port_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "0.0.0.0", "--port", "0",
    ]);
    assert!(cli.is_ok(), "serve axon port 0: {:?}", cli.err());
}

#[test]
fn parse_serve_reset_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "serve", "reset"]);
    assert!(cli.is_err(), "serve reset without --netuid should fail");
}

#[test]
fn parse_serve_batch_axon_missing_file() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "serve", "batch-axon"]);
    assert!(cli.is_err(), "batch-axon without --file should fail");
}

// ── proxy commands ──

#[test]
fn parse_proxy_add_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy add: {:?}", cli.err());
}

#[test]
fn parse_proxy_add_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "staking",
        "--delay",
        "100",
    ]);
    assert!(cli.is_ok(), "proxy add all: {:?}", cli.err());
}

#[test]
fn parse_proxy_add_missing_delegate() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "add", "--proxy-type", "any"]);
    assert!(cli.is_err(), "proxy add without --delegate should fail");
}

#[test]
fn parse_proxy_remove_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "remove",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy remove: {:?}", cli.err());
}

#[test]
fn parse_proxy_remove_missing_delegate() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "remove"]);
    assert!(cli.is_err(), "proxy remove without --delegate should fail");
}

#[test]
fn parse_proxy_create_pure_defaults() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "create-pure"]);
    assert!(cli.is_ok(), "proxy create-pure defaults: {:?}", cli.err());
}

#[test]
fn parse_proxy_create_pure_all_opts() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "create-pure",
        "--proxy-type",
        "staking",
        "--delay",
        "50",
        "--index",
        "3",
    ]);
    assert!(cli.is_ok(), "proxy create-pure all: {:?}", cli.err());
}

#[test]
fn parse_proxy_kill_pure() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "kill-pure",
        "--spawner",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--height",
        "1000",
        "--ext-index",
        "2",
    ]);
    assert!(cli.is_ok(), "proxy kill-pure: {:?}", cli.err());
}

#[test]
fn parse_proxy_kill_pure_missing_spawner() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "kill-pure",
        "--height",
        "1000",
        "--ext-index",
        "2",
    ]);
    assert!(cli.is_err(), "kill-pure without --spawner should fail");
}

#[test]
fn parse_proxy_kill_pure_missing_height() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "kill-pure",
        "--spawner",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--ext-index",
        "2",
    ]);
    assert!(cli.is_err(), "kill-pure without --height should fail");
}

#[test]
fn parse_proxy_kill_pure_missing_ext_index() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "kill-pure",
        "--spawner",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--height",
        "1000",
    ]);
    assert!(cli.is_err(), "kill-pure without --ext-index should fail");
}

#[test]
fn parse_proxy_list_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list"]);
    assert!(cli.is_ok(), "proxy list: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "list",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy list --address: {:?}", cli.err());
}

#[test]
fn parse_proxy_announce() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "announce",
        "--real",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash",
        "0xabcdef1234567890",
    ]);
    assert!(cli.is_ok(), "proxy announce: {:?}", cli.err());
}

#[test]
fn parse_proxy_announce_missing_real() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "proxy", "announce", "--call-hash", "0xabc"]);
    assert!(cli.is_err(), "announce without --real should fail");
}

#[test]
fn parse_proxy_announce_missing_call_hash() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "announce",
        "--real",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "announce without --call-hash should fail");
}

#[test]
fn parse_proxy_reject_announcement() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "reject-announcement",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--call-hash",
        "0xabc",
    ]);
    assert!(cli.is_ok(), "proxy reject: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_announcements() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list-announcements"]);
    assert!(cli.is_ok(), "proxy list-announcements: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_announcements_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "list-announcements",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "proxy list-announcements --address: {:?}",
        cli.err()
    );
}

// ── swap commands ──

#[test]
fn parse_swap_hotkey_missing_new() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "hotkey"]);
    assert!(cli.is_err(), "swap hotkey without --new-hotkey should fail");
}

#[test]
fn parse_swap_coldkey_missing_new() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "coldkey"]);
    assert!(
        cli.is_err(),
        "swap coldkey without --new-coldkey should fail"
    );
}

// ── view commands comprehensive ──

#[test]
fn parse_view_portfolio_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "view portfolio --address: {:?}", cli.err());
}

#[test]
fn parse_view_neuron_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "neuron", "--uid", "0"]);
    assert!(cli.is_err(), "view neuron without --netuid should fail");
}

#[test]
fn parse_view_neuron_missing_uid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "neuron", "--netuid", "1"]);
    assert!(cli.is_err(), "view neuron without --uid should fail");
}

#[test]
fn parse_view_validators_with_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "validators",
        "--netuid",
        "1",
        "--limit",
        "20",
    ]);
    assert!(cli.is_ok(), "view validators --netuid: {:?}", cli.err());
}

#[test]
fn parse_view_subnet_analytics_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "subnet-analytics"]);
    assert!(
        cli.is_err(),
        "view subnet-analytics without --netuid should fail"
    );
}

#[test]
fn parse_view_swap_sim_alpha_direction() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "swap-sim", "--netuid", "1", "--alpha", "100.0",
    ]);
    assert!(cli.is_ok(), "view swap-sim --alpha: {:?}", cli.err());
}

#[test]
fn parse_view_swap_sim_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "swap-sim", "--tao", "10.0"]);
    assert!(cli.is_err(), "view swap-sim without --netuid should fail");
}

#[test]
fn parse_view_nominations_missing_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "nominations"]);
    assert!(
        cli.is_err(),
        "nominations without --hotkey-address should fail"
    );
}

#[test]
fn parse_view_metagraph_with_diff() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "metagraph",
        "--netuid",
        "1",
        "--since-block",
        "1000000",
        "--limit",
        "10",
    ]);
    assert!(cli.is_ok(), "view metagraph diff: {:?}", cli.err());
}

#[test]
fn parse_view_axon() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "axon", "--netuid", "1", "--uid", "0"]);
    assert!(cli.is_ok(), "view axon: {:?}", cli.err());
}

#[test]
fn parse_view_axon_by_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "axon",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "view axon --hotkey-address: {:?}", cli.err());
}

// ── multisig commands ──

#[test]
fn parse_multisig_address_missing_signatories() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "multisig", "address", "--threshold", "2"]);
    assert!(cli.is_err(), "multisig without --signatories should fail");
}

#[test]
fn parse_multisig_address_missing_threshold() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "address",
        "--signatories",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "multisig without --threshold should fail");
}

#[test]
fn parse_multisig_cancel() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "cancel",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call-hash",
        "0xabc",
        "--timepoint-height",
        "1000",
        "--timepoint-index",
        "1",
    ]);
    assert!(cli.is_ok(), "multisig cancel: {:?}", cli.err());
}

// ── transfer edge cases ──

#[test]
fn parse_transfer_with_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--yes",
        "--password",
        "pw",
        "--batch",
        "--dry-run",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "1.0",
    ]);
    assert!(cli.is_ok(), "transfer all flags: {:?}", cli.err());
    let cli = cli.unwrap();
    assert!(cli.yes);
    assert!(cli.batch);
    assert!(cli.dry_run);
}

#[test]
fn parse_transfer_zero_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "0.0",
    ]);
    assert!(cli.is_ok(), "transfer zero: {:?}", cli.err());
}

#[test]
fn parse_transfer_tiny_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "0.000000001",
    ]);
    assert!(cli.is_ok(), "transfer 1 RAO: {:?}", cli.err());
}

#[test]
fn parse_transfer_large_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "21000000.0",
    ]);
    assert!(cli.is_ok(), "transfer 21M TAO: {:?}", cli.err());
}

// ══════════════════════════════════════════════════════════════════════
// Wallet name CLI parsing tests (edge cases for --name, --wallet, --hotkey)
// ══════════════════════════════════════════════════════════════════════

#[test]
fn parse_wallet_create_valid_names() {
    for name in &["default", "my-wallet", "wallet_1", "Alice", "test123"] {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli",
            "wallet",
            "create",
            "--name",
            name,
            "--password",
            "test",
        ]);
        assert!(
            cli.is_ok(),
            "valid name '{}' should parse: {:?}",
            name,
            cli.err()
        );
    }
}

#[test]
fn parse_wallet_create_empty_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "create",
        "--name",
        "",
        "--password",
        "test",
    ]);
    // clap accepts empty string, but validate_name will reject it at runtime
    assert!(
        cli.is_ok(),
        "empty name should parse (validation is runtime)"
    );
}

#[test]
fn parse_wallet_create_long_name() {
    let long_name = "a".repeat(100);
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "create",
        "--name",
        &long_name,
        "--password",
        "test",
    ]);
    // clap accepts any string, runtime validation catches length
    assert!(cli.is_ok(), "long name should parse: {:?}", cli.err());
}

#[test]
fn parse_wallet_global_wallet_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--wallet", "my-wallet", "balance"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.wallet, "my-wallet");
}

#[test]
fn parse_wallet_global_wallet_short() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "-w", "custom", "balance"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.wallet, "custom");
}

#[test]
fn parse_wallet_global_hotkey_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--hotkey", "miner1", "balance"]);
    assert!(cli.is_ok());
    let cli = cli.unwrap();
    assert_eq!(cli.hotkey_name, "miner1");
}

#[test]
fn parse_wallet_new_hotkey_valid_name() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "wallet", "new-hotkey", "--name", "validator-1"]);
    assert!(cli.is_ok(), "valid hotkey name: {:?}", cli.err());
}

#[test]
fn parse_wallet_new_hotkey_empty_name_fails() {
    // --name is required for new-hotkey, clap should enforce this
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "new-hotkey"]);
    assert!(cli.is_err(), "new-hotkey without --name should fail");
}

#[test]
fn parse_wallet_import_with_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "import",
        "--name", "imported-wallet",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "test",
    ]);
    assert!(cli.is_ok(), "import with name: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_hotkey_with_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-hotkey",
        "--name", "hot-1",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(cli.is_ok(), "regen-hotkey with name: {:?}", cli.err());
}

// Additional serve IP validation tests (runtime validation tests)

#[test]
fn parse_serve_axon_max_port_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "8.8.8.8", "--port", "65535",
    ]);
    assert!(cli.is_ok(), "max port 65535: {:?}", cli.err());
}

#[test]
fn parse_serve_axon_negative_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "1.2.3.4", "--port", "-1",
    ]);
    assert!(cli.is_err(), "negative port should fail u16 parse");
}

#[test]
fn parse_serve_axon_non_numeric_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "1.2.3.4", "--port", "abc",
    ]);
    assert!(cli.is_err(), "non-numeric port should fail");
}

#[test]
fn parse_serve_axon_protocol_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "axon",
        "--netuid",
        "1",
        "--ip",
        "1.2.3.4",
        "--port",
        "8080",
        "--protocol",
        "256",
    ]);
    assert!(cli.is_err(), "protocol 256 should overflow u8");
}

// ──── Transfer SS58 validation (CLI parsing tests) ────

#[test]
fn parse_transfer_valid_ss58_dest() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "1.0",
    ]);
    assert!(cli.is_ok(), "valid SS58 dest should parse");
}

#[test]
fn parse_transfer_no_dest_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "transfer", "--amount", "1.0"]);
    assert!(cli.is_err(), "missing dest should fail");
}

#[test]
fn parse_transfer_no_amount_arg() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "missing amount should fail");
}

#[test]
fn parse_transfer_all_bob_dest() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer-all",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "transfer-all with valid dest should parse");
}

#[test]
fn parse_transfer_all_with_keep_alive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer-all",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--keep-alive",
    ]);
    assert!(cli.is_ok(), "transfer-all with --keep-alive should parse");
}

#[test]
fn parse_transfer_all_no_dest_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "transfer-all"]);
    assert!(cli.is_err(), "transfer-all without dest should fail");
}

// ──── Proxy command CLI parsing ────

#[test]
fn parse_proxy_add_ss58_delegate() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Any",
    ]);
    assert!(cli.is_ok(), "proxy add with valid delegate should parse");
}

#[test]
fn parse_proxy_add_no_delegate_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "add", "--proxy-type", "Any"]);
    assert!(cli.is_err(), "proxy add without delegate should fail");
}

#[test]
fn parse_proxy_remove_ss58_delegate() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "remove",
        "--delegate",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--proxy-type",
        "Staking",
    ]);
    assert!(cli.is_ok(), "proxy remove with valid delegate should parse");
}

#[test]
fn parse_proxy_add_delay_100() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Any",
        "--delay",
        "100",
    ]);
    assert!(cli.is_ok(), "proxy add with delay should parse");
}

// ──── Swap command CLI parsing ────

#[test]
fn parse_swap_hotkey_ss58() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "swap",
        "hotkey",
        "--new-hotkey",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "swap hotkey with valid address should parse");
}

#[test]
fn parse_swap_hotkey_no_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "hotkey"]);
    assert!(cli.is_err(), "swap hotkey without new-hotkey should fail");
}

#[test]
fn parse_swap_coldkey_ss58() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "swap",
        "coldkey",
        "--new-coldkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "swap coldkey with valid address should parse");
}

#[test]
fn parse_swap_coldkey_no_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "coldkey"]);
    assert!(cli.is_err(), "swap coldkey without new-coldkey should fail");
}

// ──── Stake transfer-stake CLI parsing ────

#[test]
fn parse_stake_transfer_stake_all_required() {
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
        "2",
    ]);
    assert!(
        cli.is_ok(),
        "transfer-stake with all required args should parse"
    );
}

#[test]
fn parse_stake_transfer_stake_no_dest() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--amount",
        "10.0",
        "--from",
        "1",
        "--to",
        "2",
    ]);
    assert!(cli.is_err(), "transfer-stake without dest should fail");
}

#[test]
fn parse_stake_transfer_stake_no_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "transfer-stake",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--from",
        "1",
        "--to",
        "2",
    ]);
    assert!(cli.is_err(), "transfer-stake without amount should fail");
}

#[test]
fn parse_stake_transfer_stake_optional_hotkey() {
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
        "2",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "transfer-stake with --hotkey-address should parse"
    );
}

// ──── Serve batch-axon CLI parsing ────

#[test]
fn parse_serve_batch_axon_with_file() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "batch-axon",
        "--file",
        "/tmp/axons.json",
    ]);
    assert!(cli.is_ok(), "batch-axon with --file should parse");
}

#[test]
fn parse_serve_batch_axon_no_file_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "serve", "batch-axon"]);
    assert!(cli.is_err(), "batch-axon without --file should fail");
}

// ──── Serve axon port boundary tests ────

#[test]
fn parse_serve_axon_port_zero_parses() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "1.2.3.4", "--port", "0",
    ]);
    assert!(
        cli.is_ok(),
        "port 0 should parse (rejected at runtime by validate_port)"
    );
}

#[test]
fn parse_serve_axon_port_max_65535() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "1.2.3.4", "--port", "65535",
    ]);
    assert!(cli.is_ok(), "max port 65535 should parse");
}

#[test]
fn parse_serve_axon_port_65536_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "serve", "axon", "--netuid", "1", "--ip", "1.2.3.4", "--port", "65536",
    ]);
    assert!(cli.is_err(), "port 65536 should overflow u16");
}

// ──── Additional global flags edge cases ────

#[test]
fn parse_balance_ss58_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "balance",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "balance with valid address should parse");
}

#[test]
fn parse_balance_watch_30s() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch", "30"]);
    assert!(cli.is_ok(), "balance with --watch interval should parse");
}

#[test]
fn parse_balance_at_specific_block() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--at-block", "1000000"]);
    assert!(cli.is_ok(), "balance with --at-block should parse");
}

#[test]
fn parse_global_timeout_30() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--timeout", "30", "balance"]);
    assert!(cli.is_ok(), "global --timeout flag should parse");
}

#[test]
fn parse_global_log_file() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--log-file", "/tmp/agcli.log", "balance"]);
    assert!(cli.is_ok(), "global --log-file flag should parse");
}

#[test]
fn parse_global_best_endpoint() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--best", "balance"]);
    assert!(cli.is_ok(), "global --best flag should parse");
}

#[test]
fn parse_global_debug_mode() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--debug", "balance"]);
    assert!(cli.is_ok(), "global --debug flag should parse");
}

// ──── wallet sign/verify/derive CLI edge cases ────

#[test]
fn parse_wallet_sign_hex_message() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "wallet", "sign", "--message", "0xdeadbeef"]);
    assert!(cli.is_ok(), "wallet sign hex message: {:?}", cli.err());
}

#[test]
fn parse_wallet_sign_empty_message() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "sign", "--message", ""]);
    assert!(
        cli.is_ok(),
        "wallet sign empty message should parse: {:?}",
        cli.err()
    );
}

#[test]
fn parse_wallet_sign_unicode_message() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "wallet", "sign", "--message", "Hello 🌐🔑"]);
    assert!(cli.is_ok(), "wallet sign unicode: {:?}", cli.err());
}

#[test]
fn parse_wallet_sign_long_message() {
    let long_msg = "a".repeat(10_000);
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "sign", "--message", &long_msg]);
    assert!(cli.is_ok(), "wallet sign long message: {:?}", cli.err());
}

#[test]
fn parse_wallet_sign_missing_message() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "sign"]);
    assert!(cli.is_err(), "wallet sign without --message should fail");
}

#[test]
fn parse_wallet_sign_with_wallet_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--wallet",
        "mywallet",
        "wallet",
        "sign",
        "--message",
        "test",
    ]);
    assert!(cli.is_ok(), "wallet sign with --wallet: {:?}", cli.err());
}

#[test]
fn parse_wallet_verify_without_signer() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "verify",
        "--message",
        "hello",
        "--signature",
        "0xabcd1234",
    ]);
    assert!(
        cli.is_ok(),
        "wallet verify without --signer: {:?}",
        cli.err()
    );
}

#[test]
fn parse_wallet_verify_with_signer() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "verify",
        "--message",
        "hello",
        "--signature",
        "0xabcd1234",
        "--signer",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "wallet verify with --signer: {:?}", cli.err());
}

#[test]
fn parse_wallet_verify_missing_signature() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "verify", "--message", "hello"]);
    assert!(
        cli.is_err(),
        "wallet verify without --signature should fail"
    );
}

#[test]
fn parse_wallet_verify_missing_message() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "wallet", "verify", "--signature", "0xabcd1234"]);
    assert!(cli.is_err(), "wallet verify without --message should fail");
}

#[test]
fn parse_wallet_verify_hex_message() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "verify",
        "--message",
        "0xdeadbeef",
        "--signature",
        "0x",
    ]);
    assert!(cli.is_ok(), "wallet verify hex message: {:?}", cli.err());
}

#[test]
fn parse_wallet_derive_from_hex() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "derive",
        "--input",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(
        cli.is_ok(),
        "wallet derive from hex pubkey: {:?}",
        cli.err()
    );
}

#[test]
fn parse_wallet_derive_from_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "derive",
        "--input", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(cli.is_ok(), "wallet derive from mnemonic: {:?}", cli.err());
}

#[test]
fn parse_wallet_derive_missing_input() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "derive"]);
    assert!(cli.is_err(), "wallet derive without --input should fail");
}

#[test]
fn parse_wallet_derive_with_output_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "wallet",
        "derive",
        "--input",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(
        cli.is_ok(),
        "wallet derive with --output json: {:?}",
        cli.err()
    );
}

// ──── multisig JSON args CLI edge cases ────

#[test]
fn parse_multisig_submit_with_complex_args() {
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
        "transfer_keep_alive",
        "--args",
        r#"[{"Id":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"},1000000000]"#,
    ]);
    assert!(cli.is_ok(), "multisig submit complex args: {:?}", cli.err());
}

#[test]
fn parse_multisig_submit_without_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "submit",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--pallet",
        "System",
        "--call",
        "remark",
    ]);
    assert!(
        cli.is_ok(),
        "multisig submit without --args: {:?}",
        cli.err()
    );
}

#[test]
fn parse_multisig_execute_with_timepoint() {
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
        "transfer_keep_alive",
        "--args",
        "[1000]",
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
fn parse_multisig_execute_missing_pallet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "execute",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call",
        "transfer_keep_alive",
    ]);
    assert!(
        cli.is_err(),
        "multisig execute without --pallet should fail"
    );
}

// ──── wallet new-hotkey / regen-hotkey CLI edge cases ────

#[test]
fn parse_wallet_new_hotkey_custom_name() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "wallet", "new-hotkey", "--name", "miner1"]);
    assert!(
        cli.is_ok(),
        "wallet new-hotkey with custom name: {:?}",
        cli.err()
    );
}

#[test]
fn parse_wallet_new_hotkey_no_name_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "new-hotkey"]);
    assert!(cli.is_err(), "wallet new-hotkey without --name should fail");
}

#[test]
fn parse_wallet_regen_hotkey_with_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-hotkey",
        "--name", "recovered",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(cli.is_ok(), "wallet regen-hotkey: {:?}", cli.err());
}

#[test]
fn parse_wallet_regen_hotkey_default_name() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-hotkey",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    ]);
    assert!(
        cli.is_ok(),
        "wallet regen-hotkey with default name: {:?}",
        cli.err()
    );
}

#[test]
fn parse_wallet_regen_coldkey_with_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "regen-coldkey",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "testpass",
    ]);
    assert!(cli.is_ok(), "wallet regen-coldkey: {:?}", cli.err());
}

#[test]
fn parse_wallet_show_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "show-mnemonic",
        "--password",
        "mypass",
    ]);
    assert!(cli.is_ok(), "wallet show-mnemonic: {:?}", cli.err());
}

#[test]
fn parse_wallet_import_with_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "wallet", "import",
        "--name", "imported",
        "--mnemonic", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        "--password", "mypass",
    ]);
    assert!(cli.is_ok(), "wallet import with all args: {:?}", cli.err());
}

#[test]
fn parse_wallet_create_with_no_mnemonic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "create",
        "--name",
        "quiet",
        "--password",
        "pw",
        "--no-mnemonic",
    ]);
    assert!(
        cli.is_ok(),
        "wallet create with --no-mnemonic: {:?}",
        cli.err()
    );
}

#[test]
fn parse_wallet_dev_key() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "dev-key", "--uri", "Alice"]);
    assert!(cli.is_ok(), "wallet dev-key: {:?}", cli.err());
}

#[test]
fn parse_wallet_dev_key_with_double_slash() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "dev-key", "--uri", "//Alice"]);
    assert!(cli.is_ok(), "wallet dev-key with //: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_ss58_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "list",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy list with --address should parse");
}

#[test]
fn parse_proxy_create_pure_any() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "proxy", "create-pure", "--proxy-type", "Any"]);
    assert!(cli.is_ok(), "proxy create-pure should parse");
}

#[test]
fn parse_proxy_kill_pure_full_args() {
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
    assert!(
        cli.is_ok(),
        "proxy kill-pure with all required args should parse"
    );
}

// =====================================================================
// Admin commands
// =====================================================================

#[test]
fn parse_admin_set_tempo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-tempo",
        "--netuid",
        "1",
        "--tempo",
        "360",
    ]);
    assert!(cli.is_ok(), "admin set-tempo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_tempo_with_sudo_key() {
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
    assert!(
        cli.is_ok(),
        "admin set-tempo with sudo-key: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_tempo_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-tempo", "--tempo", "360"]);
    assert!(cli.is_err(), "admin set-tempo without --netuid should fail");
}

#[test]
fn parse_admin_set_tempo_missing_tempo() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-tempo", "--netuid", "1"]);
    assert!(cli.is_err(), "admin set-tempo without --tempo should fail");
}

#[test]
fn parse_admin_set_max_validators() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-validators",
        "--netuid",
        "1",
        "--max",
        "256",
    ]);
    assert!(cli.is_ok(), "admin set-max-validators: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_validators_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-validators",
        "--netuid",
        "3",
        "--max",
        "64",
        "--sudo-key",
        "//Bob",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-max-validators with sudo: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_max_validators_missing_max() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-max-validators", "--netuid", "1"]);
    assert!(
        cli.is_err(),
        "admin set-max-validators without --max should fail"
    );
}

#[test]
fn parse_admin_set_max_uids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-uids",
        "--netuid",
        "1",
        "--max",
        "4096",
    ]);
    assert!(cli.is_ok(), "admin set-max-uids: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_uids_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-uids",
        "--netuid",
        "2",
        "--max",
        "1024",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin set-max-uids with sudo: {:?}", cli.err());
}

#[test]
fn parse_admin_set_immunity_period() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-immunity-period",
        "--netuid",
        "1",
        "--period",
        "7200",
    ]);
    assert!(cli.is_ok(), "admin set-immunity-period: {:?}", cli.err());
}

#[test]
fn parse_admin_set_immunity_period_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-immunity-period",
        "--netuid",
        "1",
        "--period",
        "0",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-immunity-period zero: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_min_weights() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-min-weights",
        "--netuid",
        "1",
        "--min",
        "10",
    ]);
    assert!(cli.is_ok(), "admin set-min-weights: {:?}", cli.err());
}

#[test]
fn parse_admin_set_min_weights_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-min-weights",
        "--netuid",
        "1",
        "--min",
        "0",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-min-weights with sudo: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_max_weight_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-weight-limit",
        "--netuid",
        "1",
        "--limit",
        "65535",
    ]);
    assert!(cli.is_ok(), "admin set-max-weight-limit: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_weight_limit_missing_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-weight-limit",
        "--netuid",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "admin set-max-weight-limit without --limit should fail"
    );
}

#[test]
fn parse_admin_set_weights_rate_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-weights-rate-limit",
        "--netuid",
        "1",
        "--limit",
        "100",
    ]);
    assert!(cli.is_ok(), "admin set-weights-rate-limit: {:?}", cli.err());
}

#[test]
fn parse_admin_set_weights_rate_limit_unlimited() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-weights-rate-limit",
        "--netuid",
        "1",
        "--limit",
        "0",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-weights-rate-limit unlimited (0): {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_commit_reveal_enable() {
    // --enabled is a bool flag: present = true, absent = false
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-commit-reveal",
        "--netuid",
        "1",
        "--enabled",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-commit-reveal enable: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_commit_reveal_disable() {
    // Omitting --enabled means enabled = false
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-commit-reveal", "--netuid", "1"]);
    assert!(
        cli.is_ok(),
        "admin set-commit-reveal disable: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_commit_reveal_with_sudo() {
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
    assert!(
        cli.is_ok(),
        "admin set-commit-reveal with sudo: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_difficulty() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-difficulty",
        "--netuid",
        "1",
        "--difficulty",
        "1000000",
    ]);
    assert!(cli.is_ok(), "admin set-difficulty: {:?}", cli.err());
}

#[test]
fn parse_admin_set_difficulty_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-difficulty",
        "--netuid",
        "1",
        "--difficulty",
        "999999999",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-difficulty with sudo: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_activity_cutoff() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-activity-cutoff",
        "--netuid",
        "1",
        "--cutoff",
        "5000",
    ]);
    assert!(cli.is_ok(), "admin set-activity-cutoff: {:?}", cli.err());
}

#[test]
fn parse_admin_set_activity_cutoff_missing_cutoff() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-activity-cutoff", "--netuid", "1"]);
    assert!(
        cli.is_err(),
        "admin set-activity-cutoff without --cutoff should fail"
    );
}

#[test]
fn parse_admin_raw() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_tempo",
        "--args",
        "[1, 100]",
    ]);
    assert!(cli.is_ok(), "admin raw: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_with_sudo_key() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_max_allowed_validators",
        "--args",
        "[1, 256]",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(cli.is_ok(), "admin raw with sudo-key: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_missing_call() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "admin", "raw", "--args", "[1, 100]"]);
    assert!(cli.is_err(), "admin raw without --call should fail");
}

#[test]
fn parse_admin_raw_missing_args() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "admin", "raw", "--call", "sudo_set_tempo"]);
    assert!(cli.is_err(), "admin raw without --args should fail");
}

#[test]
fn parse_admin_list() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "admin", "list"]);
    assert!(cli.is_ok(), "admin list: {:?}", cli.err());
}

// =====================================================================
// Scheduler commands
// =====================================================================

#[test]
fn parse_scheduler_schedule_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "1000",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
    ]);
    assert!(cli.is_ok(), "scheduler schedule basic: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_with_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "500",
        "--pallet",
        "SubtensorModule",
        "--call",
        "add_stake",
        "--args",
        "[1, \"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 1000000000]",
    ]);
    assert!(cli.is_ok(), "scheduler schedule with args: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_with_priority() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "2000",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
        "--priority",
        "0",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler schedule with priority: {:?}",
        cli.err()
    );
}

#[test]
fn parse_scheduler_schedule_with_repeat() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "100",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--repeat-every",
        "50",
        "--repeat-count",
        "10",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler schedule with repeat: {:?}",
        cli.err()
    );
}

#[test]
fn parse_scheduler_schedule_full_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "3000",
        "--pallet",
        "SubtensorModule",
        "--call",
        "set_weights",
        "--args",
        "[1, [0,1], [100,200], 0]",
        "--priority",
        "255",
        "--repeat-every",
        "100",
        "--repeat-count",
        "5",
    ]);
    assert!(cli.is_ok(), "scheduler schedule full args: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_missing_when() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
    ]);
    assert!(
        cli.is_err(),
        "scheduler schedule without --when should fail"
    );
}

#[test]
fn parse_scheduler_schedule_missing_pallet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "100",
        "--call",
        "transfer_allow_death",
    ]);
    assert!(
        cli.is_err(),
        "scheduler schedule without --pallet should fail"
    );
}

#[test]
fn parse_scheduler_schedule_missing_call() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "100",
        "--pallet",
        "Balances",
    ]);
    assert!(
        cli.is_err(),
        "scheduler schedule without --call should fail"
    );
}

#[test]
fn parse_scheduler_schedule_named_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule-named",
        "--id",
        "my_task_1",
        "--when",
        "5000",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
    ]);
    assert!(cli.is_ok(), "scheduler schedule-named: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_named_full() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule-named",
        "--id",
        "recurring_stake",
        "--when",
        "1000",
        "--pallet",
        "SubtensorModule",
        "--call",
        "add_stake",
        "--args",
        "[1, \"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 500000000]",
        "--priority",
        "64",
        "--repeat-every",
        "200",
        "--repeat-count",
        "100",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler schedule-named full: {:?}",
        cli.err()
    );
}

#[test]
fn parse_scheduler_schedule_named_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule-named",
        "--when",
        "5000",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
    ]);
    assert!(
        cli.is_err(),
        "scheduler schedule-named without --id should fail"
    );
}

#[test]
fn parse_scheduler_cancel() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "cancel",
        "--when",
        "1000",
        "--index",
        "0",
    ]);
    assert!(cli.is_ok(), "scheduler cancel: {:?}", cli.err());
}

#[test]
fn parse_scheduler_cancel_missing_when() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "scheduler", "cancel", "--index", "0"]);
    assert!(cli.is_err(), "scheduler cancel without --when should fail");
}

#[test]
fn parse_scheduler_cancel_missing_index() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "scheduler", "cancel", "--when", "1000"]);
    assert!(cli.is_err(), "scheduler cancel without --index should fail");
}

#[test]
fn parse_scheduler_cancel_named() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "cancel-named",
        "--id",
        "my_task_1",
    ]);
    assert!(cli.is_ok(), "scheduler cancel-named: {:?}", cli.err());
}

#[test]
fn parse_scheduler_cancel_named_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "scheduler", "cancel-named"]);
    assert!(
        cli.is_err(),
        "scheduler cancel-named without --id should fail"
    );
}

// =====================================================================
// Preimage commands
// =====================================================================

#[test]
fn parse_preimage_note_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "preimage",
        "note",
        "--pallet",
        "SubtensorModule",
        "--call",
        "set_weights",
    ]);
    assert!(cli.is_ok(), "preimage note basic: {:?}", cli.err());
}

#[test]
fn parse_preimage_note_with_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "preimage",
        "note",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
        "--args",
        "[\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 1000000000]",
    ]);
    assert!(cli.is_ok(), "preimage note with args: {:?}", cli.err());
}

#[test]
fn parse_preimage_note_missing_pallet() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "preimage", "note", "--call", "set_weights"]);
    assert!(cli.is_err(), "preimage note without --pallet should fail");
}

#[test]
fn parse_preimage_note_missing_call() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "preimage",
        "note",
        "--pallet",
        "SubtensorModule",
    ]);
    assert!(cli.is_err(), "preimage note without --call should fail");
}

#[test]
fn parse_preimage_unnote() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "preimage",
        "unnote",
        "--hash",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(cli.is_ok(), "preimage unnote: {:?}", cli.err());
}

#[test]
fn parse_preimage_unnote_missing_hash() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "preimage", "unnote"]);
    assert!(cli.is_err(), "preimage unnote without --hash should fail");
}

// =====================================================================
// Contracts commands
// =====================================================================

#[test]
fn parse_contracts_upload_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "upload",
        "--code",
        "/path/to/contract.wasm",
    ]);
    assert!(cli.is_ok(), "contracts upload basic: {:?}", cli.err());
}

#[test]
fn parse_contracts_upload_with_deposit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "upload",
        "--code",
        "/path/to/contract.wasm",
        "--storage-deposit-limit",
        "1000000000",
    ]);
    assert!(
        cli.is_ok(),
        "contracts upload with deposit: {:?}",
        cli.err()
    );
}

#[test]
fn parse_contracts_upload_missing_code() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "contracts", "upload"]);
    assert!(cli.is_err(), "contracts upload without --code should fail");
}

#[test]
fn parse_contracts_instantiate_minimal() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(
        cli.is_ok(),
        "contracts instantiate minimal: {:?}",
        cli.err()
    );
}

#[test]
fn parse_contracts_instantiate_full() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        "--value",
        "1000000",
        "--data",
        "0xdeadbeef",
        "--salt",
        "0x01020304",
        "--gas-ref-time",
        "50000000000",
        "--gas-proof-size",
        "2097152",
        "--storage-deposit-limit",
        "500000000",
    ]);
    assert!(cli.is_ok(), "contracts instantiate full: {:?}", cli.err());
}

#[test]
fn parse_contracts_instantiate_missing_code_hash() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "contracts", "instantiate"]);
    assert!(
        cli.is_err(),
        "contracts instantiate without --code-hash should fail"
    );
}

#[test]
fn parse_contracts_call_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "call",
        "--contract",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--data",
        "0xdeadbeef",
    ]);
    assert!(cli.is_ok(), "contracts call basic: {:?}", cli.err());
}

#[test]
fn parse_contracts_call_full() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "call",
        "--contract",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--data",
        "0x12345678",
        "--value",
        "500000",
        "--gas-ref-time",
        "20000000000",
        "--gas-proof-size",
        "524288",
        "--storage-deposit-limit",
        "100000000",
    ]);
    assert!(cli.is_ok(), "contracts call full: {:?}", cli.err());
}

#[test]
fn parse_contracts_call_missing_contract() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "contracts", "call", "--data", "0xdeadbeef"]);
    assert!(
        cli.is_err(),
        "contracts call without --contract should fail"
    );
}

#[test]
fn parse_contracts_call_missing_data() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "call",
        "--contract",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "contracts call without --data should fail");
}

#[test]
fn parse_contracts_remove_code() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "remove-code",
        "--code-hash",
        "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
    ]);
    assert!(cli.is_ok(), "contracts remove-code: {:?}", cli.err());
}

#[test]
fn parse_contracts_remove_code_missing_hash() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "contracts", "remove-code"]);
    assert!(
        cli.is_err(),
        "contracts remove-code without --code-hash should fail"
    );
}

// =====================================================================
// EVM commands
// =====================================================================

#[test]
fn parse_evm_call_minimal() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--target",
        "0xabcdef1234567890abcdef1234567890abcdef12",
    ]);
    assert!(cli.is_ok(), "evm call minimal: {:?}", cli.err());
}

#[test]
fn parse_evm_call_full() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--target",
        "0xabcdef1234567890abcdef1234567890abcdef12",
        "--input",
        "0xa9059cbb000000000000000000000000",
        "--value",
        "0x0000000000000000000000000000000000000000000000000000000000000064",
        "--gas-limit",
        "100000",
        "--max-fee-per-gas",
        "0x0000000000000000000000000000000000000000000000000000000000000010",
    ]);
    assert!(cli.is_ok(), "evm call full: {:?}", cli.err());
}

#[test]
fn parse_evm_call_missing_source() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--target",
        "0xabcdef1234567890abcdef1234567890abcdef12",
    ]);
    assert!(cli.is_err(), "evm call without --source should fail");
}

#[test]
fn parse_evm_call_missing_target() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x1234567890abcdef1234567890abcdef12345678",
    ]);
    assert!(cli.is_err(), "evm call without --target should fail");
}

#[test]
fn parse_evm_call_custom_gas_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--gas-limit",
        "500000",
    ]);
    assert!(cli.is_ok(), "evm call custom gas limit: {:?}", cli.err());
}

#[test]
fn parse_evm_withdraw() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "withdraw",
        "--address",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--amount",
        "1000000000",
    ]);
    assert!(cli.is_ok(), "evm withdraw: {:?}", cli.err());
}

#[test]
fn parse_evm_withdraw_missing_address() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "evm", "withdraw", "--amount", "1000000000"]);
    assert!(cli.is_err(), "evm withdraw without --address should fail");
}

#[test]
fn parse_evm_withdraw_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "withdraw",
        "--address",
        "0x1234567890abcdef1234567890abcdef12345678",
    ]);
    assert!(cli.is_err(), "evm withdraw without --amount should fail");
}

#[test]
fn parse_evm_withdraw_large_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "withdraw",
        "--address",
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--amount",
        "340282366920938463463374607431768211455",
    ]);
    assert!(cli.is_ok(), "evm withdraw u128::MAX: {:?}", cli.err());
}

// =====================================================================
// SafeMode commands
// =====================================================================

#[test]
fn parse_safe_mode_enter() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "enter"]);
    assert!(cli.is_ok(), "safe-mode enter: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_extend() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "extend"]);
    assert!(cli.is_ok(), "safe-mode extend: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_force_enter() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "force-enter", "--duration", "100"]);
    assert!(cli.is_ok(), "safe-mode force-enter: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_force_enter_missing_duration() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "force-enter"]);
    assert!(
        cli.is_err(),
        "safe-mode force-enter without --duration should fail"
    );
}

#[test]
fn parse_safe_mode_force_enter_large_duration() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "safe-mode",
        "force-enter",
        "--duration",
        "4294967295",
    ]);
    assert!(
        cli.is_ok(),
        "safe-mode force-enter max u32: {:?}",
        cli.err()
    );
}

#[test]
fn parse_safe_mode_force_exit() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "force-exit"]);
    assert!(cli.is_ok(), "safe-mode force-exit: {:?}", cli.err());
}

// =====================================================================
// Drand commands
// =====================================================================

#[test]
fn parse_drand_write_pulse() {
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
fn parse_drand_write_pulse_missing_payload() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "drand",
        "write-pulse",
        "--signature",
        "0xcafebabe",
    ]);
    assert!(
        cli.is_err(),
        "drand write-pulse without --payload should fail"
    );
}

