#![allow(
    clippy::needless_borrow,
    clippy::if_same_then_else,
    clippy::single_match
)]
//! End-to-end tests against a real local subtensor chain (Docker).
//!
//! Requires: `docker pull ghcr.io/opentensor/subtensor-localnet:devnet-ready`
//!
//! Run with:
//!   cargo test --features e2e --test e2e_test -- --nocapture
//!
//! The test harness:
//!   1. Starts a local subtensor chain via Docker (fast-block mode, 250ms blocks).
//!   2. Waits for the chain to produce blocks.
//!   3. Runs tests that submit real extrinsics and verify storage map effects.
//!   4. Tears down the container on completion.
//!
//! Dev accounts (pre-funded in genesis):
//!   Alice: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY (sudo, 1M TAO)
//!   Bob:   5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty

#[macro_use]
#[path = "e2e_modules/harness.rs"]
mod e2e_harness;

/// Case implementations in `e2e_modules/cases_part*.rs` (included — not a second test binary).
mod e2e_cases {
    use super::e2e_harness::*;

    include!("e2e_modules/cases_part01.rs");
    include!("e2e_modules/cases_part02.rs");
    include!("e2e_modules/cases_part03.rs");
}

use e2e_cases::*;
use e2e_harness::*;

// ──────── Tests ────────

/// All e2e tests run in a single tokio runtime sharing one chain instance.
/// Tests are sequential within this function to avoid race conditions on chain state.
#[tokio::test]
async fn e2e_local_chain() {
    ensure_local_chain();
    let mut client = wait_for_chain().await;
    let alice = dev_pair(ALICE_URI);

    // Auto-reconnect before each phase if the connection dropped.
    macro_rules! reconnect {
        () => {
            ensure_alive(&mut client).await;
        };
    }

    println!("\n═══ E2E Test Suite — Local Subtensor Chain ═══\n");

    // ── Phase 1: Basic connectivity and queries ──
    test_connectivity(&mut client).await;
    test_alice_balance(&mut client).await;
    test_total_networks(&mut client).await;

    // ── Phase 2: Transfers ──
    test_transfer(&mut client).await;

    // ── Phase 3: Subnet registration ──
    test_register_network(&mut client).await;

    // ── Phase 3b: Early sudo config — global rate limits + both subnets ──
    reconnect!();
    setup_global_rate_limits(&mut client, &alice).await;
    reconnect!();
    setup_subnet(&mut client, &alice, NetUid(1)).await;
    reconnect!();
    wait_blocks(&mut client, 5).await;
    let mut total = client.get_total_networks().await.unwrap();
    if total < 3 {
        // Chain may have restarted — re-register the network
        println!("  only {} networks, re-registering...", total);
        let _ = try_extrinsic!(&mut client, client.register_network(&alice, ALICE_SS58));
        wait_blocks(&mut client, 5).await;
        total = client.get_total_networks().await.unwrap();
    }
    assert!(total >= 2, "Expected at least 2 networks, got {}", total);
    let newest_sn = NetUid(total - 1);
    println!("  newest subnet: SN{} (total={})", newest_sn.0, total);
    setup_subnet(&mut client, &alice, newest_sn).await;
    reconnect!();
    let primary_sn = NetUid(1);

    // ── Phase 4: Neuron registration (uses newly created SN) ──
    test_burned_register(&mut client).await;
    reconnect!();
    test_snipe_register(&mut client).await;
    reconnect!();
    test_snipe_fast_mode(&mut client).await;
    reconnect!();
    test_snipe_already_registered(&mut client).await;
    reconnect!();
    test_snipe_max_cost_guard(&mut client).await;
    reconnect!();
    test_snipe_max_attempts_guard(&mut client).await;
    reconnect!();
    test_snipe_watch(&mut client).await;
    reconnect!();

    // ── Phase 5: Weights (uses SN1 which has commit-reveal disabled) ──
    test_set_weights(&mut client, primary_sn).await;
    reconnect!();
    test_set_mechanism_weights(&mut client, primary_sn).await;
    reconnect!();
    test_commit_mechanism_weights(&mut client, primary_sn).await;
    reconnect!();
    test_reveal_mechanism_weights(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rate_limit_enforced(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_on_duplicate_uids(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_on_invalid_uid(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_on_max_weight_exceeded(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_on_weight_vec_not_equal_size(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_on_uids_length_exceeds_subnet(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_on_root_network(&mut client).await;
    reconnect!();
    test_set_weights_rejected_when_commit_reveal_enabled(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_on_wrong_version_key(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_without_validator_permit(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_when_weight_vec_below_min(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_rejected_when_stake_below_threshold(&mut client, primary_sn).await;
    reconnect!();
    test_set_weights_finalization_timeout_when_chain_paused(&mut client, primary_sn).await;
    reconnect!();

    // ── Phase 6: Staking (comprehensive) ──
    test_add_remove_stake(&mut client).await;
    reconnect!();
    test_stake_move(&mut client).await;
    reconnect!();
    test_stake_unstake_all(&mut client).await;
    reconnect!();
    test_stake_queries(&mut client).await;
    reconnect!();
    test_stake_childkey_take(&mut client).await;
    reconnect!();
    test_stake_set_auto(&mut client).await;
    reconnect!();
    test_stake_set_claim(&mut client).await;
    reconnect!();
    test_stake_edge_cases(&mut client).await;
    reconnect!();

    // ── Phase 7: Identity ──
    test_subnet_identity(&mut client, primary_sn).await;
    reconnect!();

    // ── Phase 8: Proxy ──
    test_proxy(&mut client).await;
    reconnect!();

    // ── Phase 9: Child Keys ──
    test_child_keys(&mut client, primary_sn).await;
    reconnect!();

    // ── Phase 10: Commitments ──
    test_commitments(&mut client, primary_sn).await;
    reconnect!();

    // ── Phase 11: Subnet queries (comprehensive) ──
    test_subnet_queries(&mut client).await;
    test_historical_queries(&mut client).await;
    reconnect!();

    // ── Phase 12: Serve axon ──
    test_serve_axon(&mut client, primary_sn).await;
    reconnect!();

    // ── Phase 13: Root register ──
    test_root_register(&mut client).await;

    // ── Phase 15: Delegate take ──
    reconnect!();
    test_delegate_take(&mut client, primary_sn).await;

    // ── Phase 16: Transfer all ──
    reconnect!();
    test_transfer_all(&mut client).await;

    // ── Phase 17: Commit/reveal weights ──
    reconnect!();
    test_commit_weights_rejected_when_commit_reveal_disabled(&mut client, primary_sn).await;
    reconnect!();
    test_reveal_weights_rejected_without_prior_commit(&mut client, primary_sn).await;
    reconnect!();
    test_reveal_weights_rejected_when_reveal_too_early(&mut client, primary_sn).await;
    reconnect!();
    test_commit_weights(&mut client, primary_sn).await;
    reconnect!();
    test_commit_weights_rejected_when_unrevealed_pending(&mut client, primary_sn).await;
    reconnect!();
    test_reveal_weights_rejected_on_hash_mismatch(&mut client, primary_sn).await;
    reconnect!();
    test_commit_weights_rejected_when_committing_too_fast(&mut client, primary_sn).await;
    reconnect!();
    test_reveal_weights_rejected_when_commit_expired(&mut client, primary_sn).await;
    reconnect!();
    test_commit_timelocked_weights_rejected_when_incorrect_commit_reveal_version(
        &mut client,
        primary_sn,
    )
    .await;

    // ── Phase 18: Schedule coldkey swap ──
    reconnect!();
    test_schedule_coldkey_swap(&mut client).await;

    // ── Phase 19: Dissolve network (uses the newly-registered SN, not SN1) ──
    reconnect!();
    test_dissolve_network(&mut client).await;

    // ── Phase 20: Block queries + historical diff + doctor (RPC parity) ──
    reconnect!();
    test_block_queries(&mut client).await;
    test_diff_queries(&mut client, primary_sn).await;
    test_doctor_preflight(&mut client).await;
    test_balance_preflight(&mut client).await;
    test_transfer_preflight(&mut client).await;
    test_stake_list_preflight(&mut client).await;
    test_stake_add_preflight(&mut client).await;
    test_stake_remove_preflight(&mut client).await;
    test_stake_move_preflight(&mut client).await;
    test_stake_swap_preflight(&mut client).await;
    test_stake_unstake_all_preflight(&mut client).await;
    test_view_portfolio_preflight(&mut client).await;

    // ── Phase 21: View queries ──
    reconnect!();
    test_view_queries(&mut client, primary_sn).await;

    // ── Phase 22: Subnet detail queries ──
    reconnect!();
    test_subnet_detail_queries(&mut client, primary_sn).await;

    // ── Phase 23: Delegate queries ──
    reconnect!();
    test_delegate_queries(&mut client).await;

    // ── Phase 24: Identity show ──
    reconnect!();
    test_identity_show(&mut client).await;

    // ── Phase 25: Serve reset ──
    reconnect!();
    test_serve_reset(&mut client, primary_sn).await;

    // ── Phase 26: Subscribe blocks + events (streaming prefights) ──
    reconnect!();
    test_subscribe_blocks(&mut client).await;
    test_subscribe_events_preflight(&mut client).await;

    // ── Phase 27: Wallet sign/verify (local crypto) ──
    test_wallet_sign_verify().await;

    // ── Phase 28: Utils convert (TAO↔RAO) ──
    test_utils_convert().await;

    // ── Phase 29: Network overview ──
    reconnect!();
    test_network_overview(&mut client).await;

    // ── Phase 30: Crowdloan lifecycle ──
    reconnect!();
    test_crowdloan_lifecycle(&mut client).await;

    // ── Phase 31: Swap hotkey ──
    reconnect!();
    test_swap_hotkey(&mut client, primary_sn).await;

    // ── Phase 32: Metagraph snapshot ──
    reconnect!();
    test_metagraph(&mut client, primary_sn).await;

    // ── Phase 33: Multi-balance query ──
    reconnect!();
    test_multi_balance(&mut client).await;

    // ── Phase 34: Extended state queries (untested methods) ──
    reconnect!();
    test_extended_state_queries(&mut client, primary_sn).await;

    // ── Phase 35: Parent keys (reverse of child keys) ──
    reconnect!();
    test_parent_keys(&mut client, primary_sn).await;

    // ── Phase 36: Coldkey swap scheduled query ──
    reconnect!();
    test_coldkey_swap_query(&mut client).await;

    // ── Phase 37: All weights query ──
    reconnect!();
    test_all_weights(&mut client, primary_sn).await;

    // ── Phase 38: Historical at-block queries (comprehensive) ──
    reconnect!();
    test_at_block_queries(&mut client, primary_sn).await;

    // Cleanup
    println!("\n═══ All E2E Tests Passed ═══\n");
    let _ = Command::new("docker")
        .args(["rm", "-f", CONTAINER_NAME])
        .output();
}
