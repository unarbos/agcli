use agcli::cli::OutputFormat;
use clap::Parser;

#[test]
fn parse_delegate_decrease_take_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "delegate", "decrease-take", "--take", "10.0"]);
    assert!(cli.is_ok(), "delegate decrease-take: {:?}", cli.err());
}

#[test]
fn parse_delegate_increase_take_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "delegate", "increase-take", "--take", "15.0"]);
    assert!(cli.is_ok(), "delegate increase-take: {:?}", cli.err());
}

#[test]
fn parse_delegate_show_json_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "delegate",
        "show",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "delegate show json: {:?}", cli.err());
}

#[test]
fn parse_delegate_decrease_take_endpoint_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--endpoint",
        "ws://localhost:9944",
        "delegate",
        "decrease-take",
        "--take",
        "5.5",
    ]);
    assert!(
        cli.is_ok(),
        "delegate decrease-take endpoint: {:?}",
        cli.err()
    );
}

#[test]
fn parse_delegate_list_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "delegate", "list"]);
    assert!(cli.is_ok(), "delegate list: {:?}", cli.err());
}

// ── Subscribe commands ──────────────────────────────────────────────

#[test]
fn parse_subscribe_events_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "events"]);
    assert!(cli.is_ok(), "subscribe events: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_filter_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "events", "--filter", "staking"]);
    assert!(cli.is_ok(), "subscribe events filter: {:?}", cli.err());
}

#[test]
fn parse_subscribe_blocks_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "blocks"]);
    assert!(cli.is_ok(), "subscribe blocks: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_json_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "subscribe", "events"]);
    assert!(cli.is_ok(), "subscribe events json: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_all_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "events", "--filter", "all"]);
    assert!(cli.is_ok(), "subscribe events all: {:?}", cli.err());
}

// ── View extra subcommands ──────────────────────────────────────────

#[test]
fn parse_view_neuron_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "neuron", "--netuid", "1", "--uid", "0"]);
    assert!(cli.is_ok(), "view neuron: {:?}", cli.err());
}

#[test]
fn parse_view_neuron_at_block_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "neuron",
        "--netuid",
        "1",
        "--uid",
        "5",
        "--at-block",
        "100000",
    ]);
    assert!(cli.is_ok(), "view neuron at-block: {:?}", cli.err());
}

#[test]
fn parse_view_history_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "history"]);
    assert!(cli.is_ok(), "view history: {:?}", cli.err());
}

#[test]
fn parse_view_history_limit_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "history", "--limit", "50"]);
    assert!(cli.is_ok(), "view history limit: {:?}", cli.err());
}

#[test]
fn parse_view_account_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "account"]);
    assert!(cli.is_ok(), "view account: {:?}", cli.err());
}

#[test]
fn parse_view_account_addr_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "account",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "view account addr: {:?}", cli.err());
}

#[test]
fn parse_view_staking_analytics_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "staking-analytics"]);
    assert!(cli.is_ok(), "view staking-analytics: {:?}", cli.err());
}

#[test]
fn parse_view_swap_sim_tao_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "swap-sim", "--netuid", "1", "--tao", "10.0",
    ]);
    assert!(cli.is_ok(), "view swap-sim tao: {:?}", cli.err());
}

#[test]
fn parse_view_swap_sim_alpha_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "swap-sim", "--netuid", "1", "--alpha", "100.0",
    ]);
    assert!(cli.is_ok(), "view swap-sim alpha: {:?}", cli.err());
}

#[test]
fn parse_view_nominations_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "nominations",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "view nominations: {:?}", cli.err());
}

#[test]
fn parse_view_axon_uid_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "axon", "--netuid", "1", "--uid", "0"]);
    assert!(cli.is_ok(), "view axon uid: {:?}", cli.err());
}

#[test]
fn parse_view_axon_hotkey_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "axon",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "view axon hotkey: {:?}", cli.err());
}

#[test]
fn parse_view_health_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "health", "--netuid", "1"]);
    assert!(cli.is_ok(), "view health: {:?}", cli.err());
}

#[test]
fn parse_view_health_tcp_s18() {
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
fn parse_view_emissions_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "emissions", "--netuid", "1"]);
    assert!(cli.is_ok(), "view emissions: {:?}", cli.err());
}

#[test]
fn parse_view_emissions_limit_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "emissions",
        "--netuid",
        "1",
        "--limit",
        "20",
    ]);
    assert!(cli.is_ok(), "view emissions limit: {:?}", cli.err());
}

#[test]
fn parse_view_network_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "network"]);
    assert!(cli.is_ok(), "view network: {:?}", cli.err());
}

#[test]
fn parse_view_network_at_block_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "network", "--at-block", "500000"]);
    assert!(cli.is_ok(), "view network at-block: {:?}", cli.err());
}

#[test]
fn parse_view_dynamic_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "dynamic"]);
    assert!(cli.is_ok(), "view dynamic: {:?}", cli.err());
}

#[test]
fn parse_view_dynamic_at_block_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "dynamic", "--at-block", "200000"]);
    assert!(cli.is_ok(), "view dynamic at-block: {:?}", cli.err());
}

// ── Stake edge case commands ────────────────────────────────────────

#[test]
fn parse_stake_set_auto_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-auto", "--netuid", "1"]);
    assert!(cli.is_ok(), "stake set-auto: {:?}", cli.err());
}

#[test]
fn parse_stake_show_auto_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "show-auto"]);
    assert!(cli.is_ok(), "stake show-auto: {:?}", cli.err());
}

#[test]
fn parse_stake_claim_root_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "claim-root", "--netuid", "1"]);
    assert!(cli.is_ok(), "stake claim-root: {:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_swap_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-claim", "--claim-type", "swap"]);
    assert!(cli.is_ok(), "stake set-claim swap: {:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_keep_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-claim", "--claim-type", "keep"]);
    assert!(cli.is_ok(), "stake set-claim keep: {:?}", cli.err());
}

#[test]
fn parse_stake_set_claim_invalid_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "set-claim", "--claim-type", "all"]);
    assert!(
        cli.is_err(),
        "stake set-claim 'all' should fail (valid: swap, keep, keep-subnets)"
    );
}

#[test]
fn parse_stake_set_claim_subnets_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "set-claim",
        "--claim-type",
        "keep",
        "--subnets",
        "1,2,3",
    ]);
    assert!(cli.is_ok(), "stake set-claim subnets: {:?}", cli.err());
}

#[test]
fn parse_stake_process_claim_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "process-claim"]);
    assert!(cli.is_ok(), "stake process-claim: {:?}", cli.err());
}

#[test]
fn parse_stake_process_claim_netuids_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "stake", "process-claim", "--netuids", "1,2"]);
    assert!(cli.is_ok(), "stake process-claim netuids: {:?}", cli.err());
}

#[test]
fn parse_stake_transfer_stake_s18() {
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
fn parse_stake_recycle_alpha_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "recycle-alpha",
        "--amount",
        "5.0",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "stake recycle-alpha: {:?}", cli.err());
}

#[test]
fn parse_stake_burn_alpha_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "burn-alpha",
        "--amount",
        "1.0",
        "--netuid",
        "1",
    ]);
    assert!(cli.is_ok(), "stake burn-alpha: {:?}", cli.err());
}

#[test]
fn parse_stake_unstake_all_alpha_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "unstake-all-alpha"]);
    assert!(cli.is_ok(), "stake unstake-all-alpha: {:?}", cli.err());
}

#[test]
fn parse_stake_mev_add_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--mev", "stake", "add", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "stake add --mev: {:?}", cli.err());
}

#[test]
fn parse_stake_mev_remove_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--mev", "stake", "remove", "--amount", "1.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "stake remove --mev: {:?}", cli.err());
}

// ── Doctor & Audit ──────────────────────────────────────────────────

#[test]
fn parse_doctor_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "doctor"]);
    assert!(cli.is_ok(), "doctor: {:?}", cli.err());
}

#[test]
fn parse_doctor_json_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "doctor"]);
    assert!(cli.is_ok(), "doctor json: {:?}", cli.err());
}

#[test]
fn parse_audit_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "audit",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "audit: {:?}", cli.err());
}

#[test]
fn parse_audit_json_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "audit",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "audit json: {:?}", cli.err());
}

// ── Weight commit/reveal/status ─────────────────────────────────────

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

// ── Preimage & Safe mode ────────────────────────────────────────────

#[test]
fn parse_preimage_note_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "preimage",
        "note",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--args",
        "[\"hello\"]",
    ]);
    assert!(cli.is_ok(), "preimage note: {:?}", cli.err());
}

#[test]
fn parse_preimage_unnote_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "preimage",
        "unnote",
        "--hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "preimage unnote: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_enter_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "enter"]);
    assert!(cli.is_ok(), "safe-mode enter: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_extend_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "extend"]);
    assert!(cli.is_ok(), "safe-mode extend: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_force_exit_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "force-exit"]);
    assert!(cli.is_ok(), "safe-mode force-exit: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_force_enter_s18() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "force-enter", "--duration", "100"]);
    assert!(cli.is_ok(), "safe-mode force-enter: {:?}", cli.err());
}

// ── Admin netuid 0 / high netuid parsing ────────────────────────────

#[test]
fn parse_admin_set_tempo_netuid_0_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-tempo",
        "--netuid",
        "0",
        "--tempo",
        "100",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-tempo netuid 0 parses: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_tempo_netuid_max_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-tempo",
        "--netuid",
        "65535",
        "--tempo",
        "100",
    ]);
    assert!(cli.is_ok(), "admin set-tempo netuid max: {:?}", cli.err());
}

#[test]
fn parse_admin_raw_known_call_s18() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "raw",
        "--call",
        "sudo_set_tempo",
        "--args",
        "[1, 100]",
    ]);
    assert!(cli.is_ok(), "admin raw known: {:?}", cli.err());
}

#[test]
fn parse_admin_list_json_s18() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "admin", "list"]);
    assert!(cli.is_ok(), "admin list json: {:?}", cli.err());
}

// ═══════════════════════════════════════════════════════════════════
// Step 20 — Address validation & threshold tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn parse_balance_with_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "balance",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "balance with valid address: {:?}", cli.err());
}

#[test]
fn parse_balance_with_threshold_s20() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch", "--threshold", "1.5"]);
    assert!(cli.is_ok(), "balance watch with threshold: {:?}", cli.err());
}

#[test]
fn parse_balance_watch_interval_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch", "30"]);
    assert!(cli.is_ok(), "balance watch with interval: {:?}", cli.err());
}

#[test]
fn parse_balance_at_block_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--at-block", "1000000"]);
    assert!(cli.is_ok(), "balance at-block: {:?}", cli.err());
}

#[test]
fn parse_balance_json_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "balance"]);
    assert!(cli.is_ok(), "balance json: {:?}", cli.err());
}

#[test]
fn parse_balance_threshold_zero_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch", "--threshold", "0"]);
    assert!(
        cli.is_ok(),
        "balance threshold zero parses: {:?}",
        cli.err()
    );
}

#[test]
fn parse_check_swap_with_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "wallet",
        "check-swap",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "check-swap with address: {:?}", cli.err());
}

#[test]
fn parse_check_swap_no_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "wallet", "check-swap"]);
    assert!(cli.is_ok(), "check-swap no address: {:?}", cli.err());
}

#[test]
fn parse_audit_with_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "audit",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "audit with address: {:?}", cli.err());
}

#[test]
fn parse_audit_no_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "audit"]);
    assert!(cli.is_ok(), "audit no address: {:?}", cli.err());
}

#[test]
fn parse_audit_json_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "audit"]);
    assert!(cli.is_ok(), "audit json: {:?}", cli.err());
}

#[test]
fn parse_view_portfolio_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "portfolio with address: {:?}", cli.err());
}

#[test]
fn parse_view_portfolio_at_block_s20() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "view", "portfolio", "--at-block", "5000000"]);
    assert!(cli.is_ok(), "portfolio at-block: {:?}", cli.err());
}

#[test]
fn parse_view_history_address_limit_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "history",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--limit",
        "50",
    ]);
    assert!(
        cli.is_ok(),
        "history with address and limit: {:?}",
        cli.err()
    );
}

#[test]
fn parse_view_account_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "account",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "account with address: {:?}", cli.err());
}

#[test]
fn parse_view_account_at_block_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "account", "--at-block", "1000"]);
    assert!(cli.is_ok(), "account at-block: {:?}", cli.err());
}

#[test]
fn parse_view_staking_analytics_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "staking-analytics",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "staking-analytics with addr: {:?}", cli.err());
}

#[test]
fn parse_view_staking_analytics_no_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "staking-analytics"]);
    assert!(cli.is_ok(), "staking-analytics no addr: {:?}", cli.err());
}

#[test]
fn parse_stake_list_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "list",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "stake list address: {:?}", cli.err());
}

#[test]
fn parse_stake_list_at_block_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "list", "--at-block", "500000"]);
    assert!(cli.is_ok(), "stake list at-block: {:?}", cli.err());
}

#[test]
fn parse_stake_show_auto_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "show-auto",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "stake show-auto address: {:?}", cli.err());
}

#[test]
fn parse_stake_show_auto_no_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "stake", "show-auto"]);
    assert!(cli.is_ok(), "stake show-auto no addr: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "list",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "proxy list address: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_no_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list"]);
    assert!(cli.is_ok(), "proxy list no addr: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_announcements_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "list-announcements",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(
        cli.is_ok(),
        "proxy list-announcements addr: {:?}",
        cli.err()
    );
}

#[test]
fn parse_proxy_list_announcements_no_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "proxy", "list-announcements"]);
    assert!(
        cli.is_ok(),
        "proxy list-announcements no addr: {:?}",
        cli.err()
    );
}

#[test]
fn parse_diff_portfolio_address_blocks_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--block1",
        "100000",
        "--block2",
        "200000",
    ]);
    assert!(
        cli.is_ok(),
        "diff portfolio address+blocks: {:?}",
        cli.err()
    );
}

#[test]
fn parse_diff_portfolio_no_address_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--block1",
        "100000",
        "--block2",
        "200000",
    ]);
    assert!(cli.is_ok(), "diff portfolio no addr: {:?}", cli.err());
}

// ── Batch file CLI tests ──

#[test]
fn parse_batch_file_json_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "batch", "--file", "/tmp/batch.json"]);
    assert!(cli.is_ok(), "batch file: {:?}", cli.err());
}

#[test]
fn parse_batch_no_atomic_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "batch",
        "--file",
        "/tmp/batch.json",
        "--no-atomic",
    ]);
    assert!(cli.is_ok(), "batch no-atomic: {:?}", cli.err());
}

#[test]
fn parse_batch_force_s20() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "batch", "--file", "/tmp/batch.json", "--force"]);
    assert!(cli.is_ok(), "batch force: {:?}", cli.err());
}

// ── View commands edge cases ──

#[test]
fn parse_view_portfolio_json_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "view", "portfolio"]);
    assert!(cli.is_ok(), "portfolio json: {:?}", cli.err());
}

#[test]
fn parse_view_history_json_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "view", "history"]);
    assert!(cli.is_ok(), "history json: {:?}", cli.err());
}

#[test]
fn parse_view_account_json_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "view", "account"]);
    assert!(cli.is_ok(), "account json: {:?}", cli.err());
}

#[test]
fn parse_view_staking_analytics_json_s20() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "view", "staking-analytics"]);
    assert!(cli.is_ok(), "staking-analytics json: {:?}", cli.err());
}

// ── Weights input edge cases ──

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

// ── Proxy commands edge cases ──

#[test]
fn parse_proxy_list_json_s20() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "proxy", "list"]);
    assert!(cli.is_ok(), "proxy list json: {:?}", cli.err());
}

#[test]
fn parse_proxy_list_announcements_json_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "proxy",
        "list-announcements",
    ]);
    assert!(
        cli.is_ok(),
        "proxy list-announcements json: {:?}",
        cli.err()
    );
}

// ── URL validator CLI edge cases (for config set/identity) ──

#[test]
fn parse_config_set_endpoint_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "config",
        "set",
        "--key",
        "endpoint",
        "--value",
        "wss://custom.node:9944",
    ]);
    assert!(cli.is_ok(), "config set endpoint: {:?}", cli.err());
}

#[test]
fn parse_config_set_network_s20() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "config", "set", "--key", "network", "--value", "finney",
    ]);
    assert!(cli.is_ok(), "config set network: {:?}", cli.err());
}

#[test]
fn parse_config_set_proxy_s20() {
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

// ====== Step 21: Stake limit price, Scheduler, Liquidity, BatchAxon CLI tests ======

// --- Stake AddLimit / RemoveLimit price tests ---

#[test]
fn parse_stake_add_limit_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add-limit",
        "--amount",
        "1.0",
        "--netuid",
        "1",
        "--price",
        "0.5",
    ]);
    assert!(cli.is_ok(), "stake add-limit basic: {:?}", cli.err());
}

#[test]
fn parse_stake_add_limit_partial_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add-limit",
        "--amount",
        "10.0",
        "--netuid",
        "2",
        "--price",
        "1.5",
        "--partial",
    ]);
    assert!(cli.is_ok(), "stake add-limit partial: {:?}", cli.err());
}

#[test]
fn parse_stake_add_limit_with_hotkey_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "add-limit",
        "--amount",
        "5.0",
        "--netuid",
        "3",
        "--price",
        "0.001",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "stake add-limit with hotkey: {:?}", cli.err());
}

#[test]
fn parse_stake_remove_limit_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "remove-limit",
        "--amount",
        "2.0",
        "--netuid",
        "1",
        "--price",
        "0.8",
    ]);
    assert!(cli.is_ok(), "stake remove-limit: {:?}", cli.err());
}

#[test]
fn parse_stake_remove_limit_partial_hotkey_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "stake",
        "remove-limit",
        "--amount",
        "3.0",
        "--netuid",
        "2",
        "--price",
        "1.2",
        "--partial",
        "--hotkey-address",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(
        cli.is_ok(),
        "stake remove-limit partial+hotkey: {:?}",
        cli.err()
    );
}

// --- Scheduler Schedule tests ---

#[test]
fn parse_scheduler_schedule_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "1000",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--args",
        "{}",
    ]);
    assert!(cli.is_ok(), "scheduler schedule basic: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_with_priority_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "2000",
        "--pallet",
        "Balances",
        "--call",
        "transfer_keep_alive",
        "--args",
        "{}",
        "--priority",
        "200",
    ]);
    assert!(cli.is_ok(), "scheduler schedule priority: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_with_repeat_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "500",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--args",
        "{}",
        "--repeat-every",
        "100",
        "--repeat-count",
        "10",
    ]);
    assert!(cli.is_ok(), "scheduler schedule repeat: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_named_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule-named",
        "--id",
        "my_task",
        "--when",
        "3000",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--args",
        "{}",
    ]);
    assert!(cli.is_ok(), "scheduler schedule-named: {:?}", cli.err());
}

#[test]
fn parse_scheduler_cancel_s21() {
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
fn parse_scheduler_cancel_named_s21() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "scheduler", "cancel-named", "--id", "my_task"]);
    assert!(cli.is_ok(), "scheduler cancel-named: {:?}", cli.err());
}

// --- Liquidity tests ---

#[test]
fn parse_liquidity_add_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "1",
        "--price-low",
        "0.5",
        "--price-high",
        "1.5",
        "--amount",
        "1000000",
    ]);
    assert!(cli.is_ok(), "liquidity add: {:?}", cli.err());
}

#[test]
fn parse_liquidity_add_with_hotkey_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "2",
        "--price-low",
        "0.1",
        "--price-high",
        "0.9",
        "--amount",
        "5000000",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "liquidity add with hotkey: {:?}", cli.err());
}

#[test]
fn parse_liquidity_remove_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "remove",
        "--netuid",
        "1",
        "--position-id",
        "42",
    ]);
    assert!(cli.is_ok(), "liquidity remove: {:?}", cli.err());
}

#[test]
fn parse_liquidity_modify_positive_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--position-id",
        "7",
        "--delta",
        "500000",
    ]);
    assert!(cli.is_ok(), "liquidity modify positive: {:?}", cli.err());
}

#[test]
fn parse_liquidity_modify_negative_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "3",
        "--position-id",
        "12",
        "--delta",
        "-100000",
    ]);
    assert!(cli.is_ok(), "liquidity modify negative: {:?}", cli.err());
}

#[test]
fn parse_liquidity_toggle_enable_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "toggle",
        "--netuid",
        "1",
        "--enable",
    ]);
    assert!(cli.is_ok(), "liquidity toggle enable: {:?}", cli.err());
}

#[test]
fn parse_liquidity_toggle_disable_s21() {
    // Without --enable flag, defaults to false
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "liquidity", "toggle", "--netuid", "5"]);
    assert!(cli.is_ok(), "liquidity toggle disable: {:?}", cli.err());
}

// --- Serve BatchAxon tests ---

#[test]
fn parse_serve_batch_axon_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "serve",
        "batch-axon",
        "--file",
        "/tmp/batch.json",
    ]);
    assert!(cli.is_ok(), "serve batch-axon: {:?}", cli.err());
}

// --- Contracts tests ---

#[test]
fn parse_contracts_upload_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "upload",
        "--code",
        "/tmp/contract.wasm",
    ]);
    assert!(cli.is_ok(), "contracts upload: {:?}", cli.err());
}

#[test]
fn parse_contracts_instantiate_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--data",
        "0x01",
        "--value",
        "0",
    ]);
    assert!(cli.is_ok(), "contracts instantiate: {:?}", cli.err());
}

#[test]
fn parse_contracts_call_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "call",
        "--contract",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--data",
        "0x02",
        "--value",
        "100",
    ]);
    assert!(cli.is_ok(), "contracts call: {:?}", cli.err());
}

// --- EVM tests ---

#[test]
fn parse_evm_call_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--input",
        "0x01",
    ]);
    assert!(cli.is_ok(), "evm call: {:?}", cli.err());
}

#[test]
fn parse_evm_withdraw_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "withdraw",
        "--address",
        "0x0000000000000000000000000000000000000001",
        "--amount",
        "1000000000",
    ]);
    assert!(cli.is_ok(), "evm withdraw: {:?}", cli.err());
}

// --- Crowdloan tests ---

#[test]
fn parse_crowdloan_create_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--deposit",
        "10.0",
        "--cap",
        "1000.0",
        "--end-block",
        "50000",
        "--min-contribution",
        "1.0",
    ]);
    assert!(cli.is_ok(), "crowdloan create: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_contribute_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "contribute",
        "--crowdloan-id",
        "0",
        "--amount",
        "5.0",
    ]);
    assert!(cli.is_ok(), "crowdloan contribute: {:?}", cli.err());
}

// --- SafeMode tests ---

#[test]
fn parse_safe_mode_enter_s21() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "enter"]);
    assert!(cli.is_ok(), "safe-mode enter: {:?}", cli.err());
}

#[test]
fn parse_safe_mode_extend_s21() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "safe-mode", "extend"]);
    assert!(cli.is_ok(), "safe-mode extend: {:?}", cli.err());
}

// --- Multisig tests ---

#[test]
fn parse_multisig_address_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "address",
        "--signatories",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
    ]);
    assert!(cli.is_ok(), "multisig address: {:?}", cli.err());
}

#[test]
fn parse_multisig_submit_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "submit",
        "--others",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--args",
        "{}",
    ]);
    assert!(cli.is_ok(), "multisig submit: {:?}", cli.err());
}

// --- Proxy tests ---

#[test]
fn parse_proxy_add_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--proxy-type",
        "Any",
    ]);
    assert!(cli.is_ok(), "proxy add: {:?}", cli.err());
}

#[test]
fn parse_proxy_add_with_delay_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "add",
        "--delegate",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--proxy-type",
        "Staking",
        "--delay",
        "100",
    ]);
    assert!(cli.is_ok(), "proxy add with delay: {:?}", cli.err());
}

#[test]
fn parse_proxy_remove_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "remove",
        "--delegate",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--proxy-type",
        "Any",
    ]);
    assert!(cli.is_ok(), "proxy remove: {:?}", cli.err());
}

#[test]
fn parse_proxy_create_pure_s21() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "proxy", "create-pure", "--proxy-type", "Any"]);
    assert!(cli.is_ok(), "proxy create-pure: {:?}", cli.err());
}

// --- Drand tests ---

#[test]
fn parse_drand_write_pulse_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "drand",
        "write-pulse",
        "--payload",
        "0x01020304",
        "--signature",
        "0x0000000000000000000000000000000000000000000000000000000000000000",
    ]);
    assert!(cli.is_ok(), "drand write-pulse: {:?}", cli.err());
}

// --- Preimage tests ---

#[test]
fn parse_preimage_note_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "preimage", "note", "--pallet", "System", "--call", "remark", "--args", "{}",
    ]);
    assert!(cli.is_ok(), "preimage note: {:?}", cli.err());
}

#[test]
fn parse_preimage_unnote_s21() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "preimage",
        "unnote",
        "--hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "preimage unnote: {:?}", cli.err());
}

// ═══════════════════════════════════════════════════════════════════════════
// Step-2 hardening (deduped): unique parse tests not present earlier in this file
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_admin_set_max_uids_max_boundary() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-uids",
        "--netuid",
        "65535",
        "--max",
        "65535",
    ]);
    assert!(cli.is_ok(), "admin set-max-uids max: {:?}", cli.err());
}

#[test]
fn parse_admin_set_immunity_period_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-immunity-period",
        "--netuid",
        "18",
        "--period",
        "7200",
        "--sudo-key",
        "//Bob",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-immunity-period sudo: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_min_weights_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-min-weights",
        "--netuid",
        "1",
        "--min",
        "0",
    ]);
    assert!(cli.is_ok(), "admin set-min-weights 0: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_weight_limit_with_sudo() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-weight-limit",
        "--netuid",
        "8",
        "--limit",
        "1000",
        "--sudo-key",
        "//Alice",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-max-weight-limit sudo: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_weights_rate_limit_zero() {
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
        "admin set-weights-rate-limit 0: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_weights_rate_limit_large() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-weights-rate-limit",
        "--netuid",
        "1",
        "--limit",
        "18446744073709551615",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-weights-rate-limit max u64: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_difficulty_max_u64() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-difficulty",
        "--netuid",
        "1",
        "--difficulty",
        "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "admin set-difficulty max: {:?}", cli.err());
}

#[test]
fn parse_admin_set_max_uids_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-max-uids", "--max", "256"]);
    assert!(
        cli.is_err(),
        "admin set-max-uids missing netuid should fail"
    );
}

#[test]
fn parse_admin_set_max_uids_missing_max() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-max-uids", "--netuid", "1"]);
    assert!(cli.is_err(), "admin set-max-uids missing max should fail");
}

#[test]
fn parse_admin_set_immunity_period_missing_period() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-immunity-period", "--netuid", "1"]);
    assert!(
        cli.is_err(),
        "admin set-immunity-period missing period should fail"
    );
}

#[test]
fn parse_admin_set_min_weights_missing_min() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-min-weights", "--netuid", "1"]);
    assert!(
        cli.is_err(),
        "admin set-min-weights missing min should fail"
    );
}

#[test]
fn parse_admin_set_difficulty_missing_difficulty() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "admin", "set-difficulty", "--netuid", "1"]);
    assert!(
        cli.is_err(),
        "admin set-difficulty missing difficulty should fail"
    );
}

#[test]
fn parse_admin_set_weights_rate_limit_missing_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-weights-rate-limit",
        "--netuid",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "admin set-weights-rate-limit missing limit should fail"
    );
}

#[test]
fn parse_crowdloan_update_cap_missing_cap() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-cap",
        "--crowdloan-id",
        "1",
    ]);
    assert!(cli.is_err(), "crowdloan update-cap missing cap should fail");
}

#[test]
fn parse_crowdloan_update_end_missing_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-end",
        "--crowdloan-id",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "crowdloan update-end missing block should fail"
    );
}

#[test]
fn parse_crowdloan_update_min_contribution_missing() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-min-contribution",
        "--crowdloan-id",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "crowdloan update-min-contribution missing should fail"
    );
}

#[test]
fn parse_crowdloan_contributors() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "contributors",
        "--crowdloan-id",
        "42",
    ]);
    assert!(cli.is_ok(), "crowdloan contributors: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_contributors_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "contributors"]);
    assert!(
        cli.is_err(),
        "crowdloan contributors missing id should fail"
    );
}

#[test]
fn parse_diff_metagraph_missing_block1() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "metagraph",
        "--netuid",
        "1",
        "--block2",
        "1001000",
    ]);
    assert!(cli.is_err(), "diff metagraph missing block1 should fail");
}

#[test]
fn parse_diff_metagraph_missing_block2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "metagraph",
        "--netuid",
        "1",
        "--block1",
        "1000000",
    ]);
    assert!(cli.is_err(), "diff metagraph missing block2 should fail");
}

#[test]
fn parse_evm_call_defaults() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--target",
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
    ]);
    assert!(cli.is_ok(), "evm call defaults: {:?}", cli.err());
}

#[test]
fn parse_evm_call_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--target",
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
        "--input",
        "0xa9059cbb",
        "--value",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--gas-limit",
        "100000",
        "--max-fee-per-gas",
        "0x0000000000000000000000000000000000000000000000000000000000000064",
    ]);
    assert!(cli.is_ok(), "evm call all flags: {:?}", cli.err());
}

#[test]
fn parse_liquidity_modify_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "18",
        "--position-id",
        "999",
        "--delta",
        "-100000",
        "--hotkey-address",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "liquidity modify with hotkey: {:?}", cli.err());
}

#[test]
fn parse_liquidity_modify_missing_position_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--delta",
        "100",
    ]);
    assert!(
        cli.is_err(),
        "liquidity modify missing position-id should fail"
    );
}

#[test]
fn parse_multisig_approve_missing_call_hash() {
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
        "multisig approve missing call-hash should fail"
    );
}

#[test]
fn parse_multisig_approve_missing_threshold() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "approve",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--call-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(
        cli.is_err(),
        "multisig approve missing threshold should fail"
    );
}

#[test]
fn parse_multisig_execute() {
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
    assert!(cli.is_ok(), "multisig execute: {:?}", cli.err());
}

#[test]
fn parse_multisig_execute_with_args_and_timepoint() {
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
        "12345",
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
fn parse_multisig_cancel_missing_timepoint_height() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "cancel",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--timepoint-index",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "multisig cancel missing timepoint-height should fail"
    );
}

#[test]
fn parse_multisig_cancel_missing_timepoint_index() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "cancel",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--timepoint-height",
        "12345",
    ]);
    assert!(
        cli.is_err(),
        "multisig cancel missing timepoint-index should fail"
    );
}

#[test]
fn parse_proxy_kill_pure_with_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "kill-pure",
        "--spawner",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--proxy-type",
        "Staking",
        "--index",
        "3",
        "--height",
        "999999",
        "--ext-index",
        "0",
    ]);
    assert!(cli.is_ok(), "proxy kill-pure all flags: {:?}", cli.err());
}

#[test]
fn parse_proxy_proxy_announced() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "proxy-announced",
        "--delegate",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--real",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
    ]);
    assert!(cli.is_ok(), "proxy proxy-announced: {:?}", cli.err());
}

#[test]
fn parse_proxy_proxy_announced_with_type_and_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "proxy-announced",
        "--delegate",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--real",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--proxy-type",
        "Any",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
        "--args",
        "[\"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY\", 1000000000]",
    ]);
    assert!(
        cli.is_ok(),
        "proxy proxy-announced with args: {:?}",
        cli.err()
    );
}

#[test]
fn parse_proxy_proxy_announced_missing_delegate() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "proxy-announced",
        "--real",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
    ]);
    assert!(
        cli.is_err(),
        "proxy proxy-announced missing delegate should fail"
    );
}

#[test]
fn parse_proxy_reject_announcement_missing_delegate() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "reject-announcement",
        "--call-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(
        cli.is_err(),
        "proxy reject-announcement missing delegate should fail"
    );
}

#[test]
fn parse_proxy_reject_announcement_missing_call_hash() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "proxy",
        "reject-announcement",
        "--delegate",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(
        cli.is_err(),
        "proxy reject-announcement missing call-hash should fail"
    );
}

#[test]
fn parse_scheduler_schedule() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "5000000",
        "--pallet",
        "System",
        "--call",
        "remark",
    ]);
    assert!(cli.is_ok(), "scheduler schedule: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_named() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule-named",
        "--id",
        "my-scheduled-task",
        "--when",
        "5000000",
        "--pallet",
        "System",
        "--call",
        "remark",
    ]);
    assert!(cli.is_ok(), "scheduler schedule-named: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_named_with_repeat() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule-named",
        "--id",
        "recurring-job",
        "--when",
        "5000000",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--priority",
        "255",
        "--repeat-every",
        "7200",
        "--repeat-count",
        "100",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler schedule-named with repeat: {:?}",
        cli.err()
    );
}

#[test]
fn parse_transfer_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount",
        "1.5",
    ]);
    assert!(cli.is_ok(), "transfer: {:?}", cli.err());
}

#[test]
fn parse_transfer_very_small_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--amount",
        "0.000000001",
    ]);
    assert!(cli.is_ok(), "transfer tiny amount: {:?}", cli.err());
}

#[test]
fn parse_transfer_all_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer-all",
        "--dest",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "transfer-all: {:?}", cli.err());
}

#[test]
fn parse_batch_basic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "batch", "--file", "/tmp/batch.json"]);
    assert!(cli.is_ok(), "batch: {:?}", cli.err());
}

#[test]
fn parse_batch_no_atomic_and_force() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "batch",
        "--file",
        "/tmp/batch.json",
        "--no-atomic",
        "--force",
    ]);
    assert!(cli.is_ok(), "batch no-atomic + force: {:?}", cli.err());
}

#[test]
fn parse_doctor_standalone() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "doctor"]);
    assert!(cli.is_ok(), "doctor: {:?}", cli.err());
}

#[test]
fn parse_explain_with_full() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "amm", "--full"]);
    assert!(cli.is_ok(), "explain full: {:?}", cli.err());
}

#[test]
fn parse_contracts_upload() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "upload",
        "--code",
        "/tmp/contract.wasm",
    ]);
    assert!(cli.is_ok(), "contracts upload: {:?}", cli.err());
}

#[test]
fn parse_contracts_instantiate() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "contracts instantiate: {:?}", cli.err());
}

#[test]
fn parse_contracts_instantiate_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--value",
        "1000",
        "--data",
        "0xdeadbeef",
        "--salt",
        "0x01020304",
        "--gas-ref-time",
        "50000000000",
        "--gas-proof-size",
        "2097152",
        "--storage-deposit-limit",
        "5000000000",
    ]);
    assert!(
        cli.is_ok(),
        "contracts instantiate all flags: {:?}",
        cli.err()
    );
}

#[test]
fn parse_contracts_call() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "call",
        "--contract",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--data",
        "0xa9059cbb",
    ]);
    assert!(cli.is_ok(), "contracts call: {:?}", cli.err());
}

#[test]
fn parse_contracts_call_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "call",
        "--contract",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--value",
        "500",
        "--data",
        "0xa9059cbb0000000000000000000000000000000000000000000000000000000000000001",
        "--gas-ref-time",
        "25000000000",
        "--gas-proof-size",
        "512000",
        "--storage-deposit-limit",
        "10000000",
    ]);
    assert!(cli.is_ok(), "contracts call all flags: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_with_netuid_and_account() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subscribe",
        "events",
        "--filter",
        "staking",
        "--netuid",
        "18",
        "--account",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
    ]);
    assert!(cli.is_ok(), "subscribe events all filters: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_filter_types() {
    // Test all valid filter types parse
    for filter in &[
        "all",
        "staking",
        "registration",
        "transfer",
        "weights",
        "subnet",
    ] {
        let cli =
            agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "events", "--filter", filter]);
        assert!(
            cli.is_ok(),
            "subscribe events filter {}: {:?}",
            filter,
            cli.err()
        );
    }
}

#[test]
fn parse_balance_watch_bare_flag() {
    // --watch without a value should parse as Some(None)
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch"]);
    assert!(cli.is_ok(), "balance --watch bare: {:?}", cli.err());
}

#[test]
fn parse_balance_watch_with_value() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch", "30"]);
    assert!(cli.is_ok(), "balance watch with value: {:?}", cli.err());
}

#[test]
fn parse_global_all_output_formats() {
    for fmt in &["table", "json", "csv"] {
        let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", fmt, "balance"]);
        assert!(cli.is_ok(), "output format {}: {:?}", fmt, cli.err());
    }
}

#[test]
fn parse_global_invalid_output_format() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "yaml", "balance"]);
    assert!(cli.is_err(), "invalid output format should fail");
}

#[test]
fn parse_global_proxy_flag() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--proxy",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "balance",
    ]);
    assert!(cli.is_ok(), "global proxy: {:?}", cli.err());
}

#[test]
fn parse_global_mev_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--mev", "balance"]);
    assert!(cli.is_ok(), "global mev: {:?}", cli.err());
}

#[test]
fn parse_global_dry_run_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--dry-run", "balance"]);
    assert!(cli.is_ok(), "global dry-run: {:?}", cli.err());
}

#[test]
fn parse_global_best_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--best", "balance"]);
    assert!(cli.is_ok(), "global best: {:?}", cli.err());
}

#[test]
fn parse_global_hotkey_name_long_form() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--hotkey-name", "miner1", "balance"]);
    assert!(cli.is_ok(), "global --hotkey-name: {:?}", cli.err());
    let cli = cli.unwrap();
    assert_eq!(cli.hotkey_name, "miner1");
}

#[test]
fn parse_global_hotkey_alias() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--hotkey", "validator1", "balance"]);
    assert!(cli.is_ok(), "global --hotkey alias: {:?}", cli.err());
    let cli = cli.unwrap();
    assert_eq!(cli.hotkey_name, "validator1");
}

#[test]
fn parse_global_live_bare() {
    // Put subcommand before --live: bare `--live` accepts an optional u64, so `--live balance`
    // would try to parse `balance` as the interval.
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--live"]);
    assert!(cli.is_ok(), "global --live bare: {:?}", cli.err());
}

#[test]
fn parse_global_live_with_interval() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--live", "30"]);
    assert!(cli.is_ok(), "global --live 30: {:?}", cli.err());
}

#[test]
fn parse_global_endpoint_custom() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--endpoint", "ws://127.0.0.1:9944", "balance"]);
    assert!(cli.is_ok(), "global --endpoint: {:?}", cli.err());
}

#[test]
fn parse_global_wallet_dir_custom() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--wallet-dir", "/custom/wallets", "balance"]);
    assert!(cli.is_ok(), "global --wallet-dir: {:?}", cli.err());
}

#[test]
fn parse_global_short_flags() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "-n", "test", "-w", "mywallet", "-y", "balance"]);
    assert!(cli.is_ok(), "short flags -n -w -y: {:?}", cli.err());
}

#[test]
fn parse_global_all_flags_combined_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--endpoint",
        "ws://127.0.0.1:9944",
        "--wallet-dir",
        "/tmp/wallets",
        "--wallet",
        "mywallet",
        "--hotkey-name",
        "myhk",
        "--output",
        "json",
        "--proxy",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--yes",
        "--batch",
        "--mev",
        "--dry-run",
        "--best",
        "balance",
    ]);
    assert!(cli.is_ok(), "all global flags v2: {:?}", cli.err());
    let cli = cli.unwrap();
    assert_eq!(cli.network, "test");
    assert_eq!(cli.wallet, "mywallet");
    assert_eq!(cli.hotkey_name, "myhk");
    assert!(cli.yes);
    assert!(cli.batch);
    assert!(cli.mev);
    assert!(cli.dry_run);
    assert!(cli.best);
}

#[test]
fn parse_proxy_add_all_proxy_types() {
    let proxy_types = [
        "any",
        "owner",
        "staking",
        "non_transfer",
        "non_critical",
        "governance",
        "senate",
        "registration",
        "transfer",
        "small_transfer",
        "root_weights",
        "child_keys",
        "swap_hotkey",
        "subnet_lease_beneficiary",
        "root_claim",
    ];
    for pt in &proxy_types {
        let cli = agcli::cli::Cli::try_parse_from([
            "agcli",
            "proxy",
            "add",
            "--delegate",
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
            "--proxy-type",
            pt,
        ]);
        assert!(cli.is_ok(), "proxy add type {}: {:?}", pt, cli.err());
    }
}

#[test]
fn parse_localnet_start_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "start",
        "--image",
        "devnet-ready:latest",
        "--container",
        "my-localnet",
        "--port",
        "9945",
        "--wait",
        "true",
        "--timeout",
        "300",
    ]);
    assert!(cli.is_ok(), "localnet start all flags: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_no_wait() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--wait", "false"]);
    assert!(cli.is_ok(), "localnet start no wait: {:?}", cli.err());
}

#[test]
fn parse_localnet_stop_with_container() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "stop",
        "--container",
        "my-localnet",
    ]);
    assert!(cli.is_ok(), "localnet stop container: {:?}", cli.err());
}

#[test]
fn parse_localnet_reset_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "reset",
        "--image",
        "devnet-ready:v2",
        "--container",
        "my-localnet",
        "--port",
        "9945",
        "--timeout",
        "60",
    ]);
    assert!(cli.is_ok(), "localnet reset all flags: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_all_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "scaffold",
        "--config",
        "/tmp/scaffold.toml",
        "--image",
        "devnet-ready:v2",
        "--port",
        "9945",
        "--no-start",
    ]);
    assert!(cli.is_ok(), "localnet scaffold all flags: {:?}", cli.err());
}

#[test]
fn parse_diff_portfolio_default_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--block1",
        "1000000",
        "--block2",
        "1001000",
    ]);
    assert!(cli.is_ok(), "diff portfolio default: {:?}", cli.err());
}

#[test]
fn parse_diff_network_missing_block1() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "diff", "network", "--block2", "1001000"]);
    assert!(cli.is_err(), "diff network missing block1 should fail");
}

#[test]
fn parse_admin_raw_with_sudo() {
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
    assert!(cli.is_ok(), "admin raw with sudo: {:?}", cli.err());
}

#[test]
fn parse_utils_latency_with_extra_and_pings() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "utils",
        "latency",
        "--extra",
        "ws://custom1.example.com,ws://custom2.example.com",
        "--pings",
        "10",
    ]);
    assert!(cli.is_ok(), "utils latency extra+pings: {:?}", cli.err());
}

#[test]
fn parse_config_cache_clear() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "cache-clear"]);
    assert!(cli.is_ok(), "config cache-clear: {:?}", cli.err());
}

#[test]
fn parse_config_cache_info() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "cache-info"]);
    assert!(cli.is_ok(), "config cache-info: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_withdraw_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "withdraw"]);
    assert!(cli.is_err(), "crowdloan withdraw missing id should fail");
}

#[test]
fn parse_crowdloan_finalize_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "finalize"]);
    assert!(cli.is_err(), "crowdloan finalize missing id should fail");
}

#[test]
fn parse_crowdloan_refund_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "refund"]);
    assert!(cli.is_err(), "crowdloan refund missing id should fail");
}

#[test]
fn parse_crowdloan_dissolve_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "dissolve"]);
    assert!(cli.is_err(), "crowdloan dissolve missing id should fail");
}

#[test]
fn parse_crowdloan_info_missing_id() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "info"]);
    assert!(cli.is_err(), "crowdloan info missing id should fail");
}

#[test]
fn parse_liquidity_toggle_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "liquidity", "toggle"]);
    assert!(cli.is_err(), "liquidity toggle missing netuid should fail");
}

#[test]
fn parse_liquidity_remove_missing_position_id() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "liquidity", "remove", "--netuid", "1"]);
    assert!(
        cli.is_err(),
        "liquidity remove missing position-id should fail"
    );
}

#[test]
fn parse_multisig_submit_missing_pallet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "submit",
        "--others",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold",
        "2",
        "--call",
        "transfer",
    ]);
    assert!(cli.is_err(), "multisig submit missing pallet should fail");
}

#[test]
fn parse_multisig_list_missing_address() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "multisig", "list"]);
    assert!(cli.is_err(), "multisig list missing address should fail");
}

#[test]
fn parse_swap_hotkey_missing_new_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "hotkey"]);
    assert!(cli.is_err(), "swap hotkey missing new-hotkey should fail");
}

#[test]
fn parse_swap_coldkey_missing_new_coldkey() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "coldkey"]);
    assert!(cli.is_err(), "swap coldkey missing new-coldkey should fail");
}

#[test]
fn parse_admin_set_tempo_u16_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-tempo",
        "--netuid",
        "1",
        "--tempo",
        "70000",
    ]);
    assert!(cli.is_err(), "u16 overflow (70000) should fail");
}

#[test]
fn parse_admin_set_max_validators_u16_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-validators",
        "--netuid",
        "1",
        "--max",
        "100000",
    ]);
    assert!(cli.is_err(), "u16 overflow (100000) should fail");
}

#[test]
fn parse_crowdloan_update_end_u32_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-end",
        "--crowdloan-id",
        "1",
        "--end-block",
        "5000000000",
    ]);
    assert!(cli.is_err(), "u32 overflow should fail");
}

#[test]
fn parse_evm_call_gas_limit_u64_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--target",
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
        "--gas-limit",
        "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "evm call max u64 gas: {:?}", cli.err());
}

#[test]
fn parse_evm_call_gas_limit_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--target",
        "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
        "--gas-limit",
        "18446744073709551616",
    ]);
    assert!(cli.is_err(), "u64 overflow for gas-limit should fail");
}

#[test]
fn parse_liquidity_modify_i64_boundaries() {
    // Max positive i64
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--position-id",
        "1",
        "--delta",
        "9223372036854775807",
    ]);
    assert!(cli.is_ok(), "i64 max: {:?}", cli.err());

    // Min negative i64
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--position-id",
        "1",
        "--delta",
        "-9223372036854775808",
    ]);
    assert!(cli.is_ok(), "i64 min: {:?}", cli.err());
}

#[test]
fn parse_contracts_instantiate_u128_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--value",
        "340282366920938463463374607431768211455",
    ]);
    assert!(cli.is_ok(), "u128 max value: {:?}", cli.err());
}

#[test]
fn parse_unknown_command_fails() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "nonexistent-command"]);
    assert!(cli.is_err(), "unknown command should fail");
}

#[test]
fn parse_no_command_fails() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli"]);
    assert!(cli.is_err(), "no command should fail");
}

#[test]
fn parse_global_flags_after_subcommand() {
    // Global flags should work after the subcommand too
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--output", "json", "--yes"]);
    assert!(
        cli.is_ok(),
        "global flags after subcommand: {:?}",
        cli.err()
    );
    let cli = cli.unwrap();
    assert_eq!(cli.output, OutputFormat::Json);
    assert!(cli.yes);
}

#[test]
fn parse_global_network_aliases() {
    // These are string values so they all parse; resolve_network handles mapping
    for net in &[
        "finney",
        "main",
        "test",
        "testnet",
        "local",
        "localhost",
        "archive",
    ] {
        let cli = agcli::cli::Cli::try_parse_from(["agcli", "--network", net, "balance"]);
        assert!(cli.is_ok(), "network {}: {:?}", net, cli.err());
    }
}
