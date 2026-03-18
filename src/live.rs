//! Live mode — poll chain data at intervals and display changes.
//!
//! Provides `--live` functionality for metagraph, dynamic, portfolio, and stake views.
//! Tracks deltas between polls to highlight what changed.

use crate::chain::Client;
use crate::types::chain_data::DynamicInfo;
use crate::types::NetUid;
use crate::utils::truncate;
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::time::Duration;
use tokio::signal;

/// Default poll interval (12 seconds = 1 Bittensor block).
pub const DEFAULT_POLL_SECS: u64 = 12;

/// What kind of change a DynamicDelta represents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaKind {
    /// Subnet existed in both snapshots and a tracked field changed.
    Changed,
    /// Subnet appeared (in curr but not prev).
    New,
    /// Subnet disappeared (in prev but not curr).
    Removed,
}

/// Delta tracking for DynamicInfo changes.
#[derive(Debug)]
pub struct DynamicDelta {
    pub netuid: u16,
    pub name: String,
    pub kind: DeltaKind,
    pub price_prev: f64,
    pub price_now: f64,
    pub price_pct: f64,
    pub tao_in_prev: u64,
    pub tao_in_now: u64,
    pub volume_prev: u128,
    pub volume_now: u128,
}

/// Compute deltas between two snapshots of DynamicInfo.
///
/// Reports three kinds of changes:
/// - `Changed`: a subnet exists in both snapshots and a tracked field differs.
/// - `New`: a subnet appears in `curr` but not `prev`.
/// - `Removed`: a subnet was in `prev` but absent from `curr`.
pub fn compute_dynamic_deltas(prev: &[DynamicInfo], curr: &[DynamicInfo]) -> Vec<DynamicDelta> {
    let prev_map: HashMap<u16, &DynamicInfo> = prev.iter().map(|d| (d.netuid.0, d)).collect();
    let curr_map: HashMap<u16, &DynamicInfo> = curr.iter().map(|d| (d.netuid.0, d)).collect();

    let mut deltas = Vec::new();

    // Changed + New subnets
    for c in curr {
        if let Some(p) = prev_map.get(&c.netuid.0) {
            let price_pct = if p.price > 0.0 {
                (c.price - p.price) / p.price * 100.0
            } else {
                0.0
            };
            // Only report if something actually changed
            if (c.price - p.price).abs() < 1e-15
                && c.tao_in.rao() == p.tao_in.rao()
                && c.subnet_volume == p.subnet_volume
            {
                continue;
            }
            deltas.push(DynamicDelta {
                netuid: c.netuid.0,
                name: c.name.clone(),
                kind: DeltaKind::Changed,
                price_prev: p.price,
                price_now: c.price,
                price_pct,
                tao_in_prev: p.tao_in.rao(),
                tao_in_now: c.tao_in.rao(),
                volume_prev: p.subnet_volume,
                volume_now: c.subnet_volume,
            });
        } else {
            // New subnet — not in previous snapshot
            deltas.push(DynamicDelta {
                netuid: c.netuid.0,
                name: c.name.clone(),
                kind: DeltaKind::New,
                price_prev: 0.0,
                price_now: c.price,
                price_pct: 0.0,
                tao_in_prev: 0,
                tao_in_now: c.tao_in.rao(),
                volume_prev: 0,
                volume_now: c.subnet_volume,
            });
        }
    }

    // Removed subnets (in prev but not curr)
    for p in prev {
        if !curr_map.contains_key(&p.netuid.0) {
            deltas.push(DynamicDelta {
                netuid: p.netuid.0,
                name: p.name.clone(),
                kind: DeltaKind::Removed,
                price_prev: p.price,
                price_now: 0.0,
                price_pct: -100.0,
                tao_in_prev: p.tao_in.rao(),
                tao_in_now: 0,
                volume_prev: p.subnet_volume,
                volume_now: 0,
            });
        }
    }

    deltas
}

/// Run live dynamic info polling loop.
pub async fn live_dynamic(client: &Client, interval_secs: u64) -> Result<()> {
    let interval = Duration::from_secs(if interval_secs == 0 {
        DEFAULT_POLL_SECS
    } else {
        interval_secs
    });
    let mut prev = client.get_all_dynamic_info().await?;
    let mut poll_count = 0u64;
    let mut consecutive_failures: u32 = 0;

    print_dynamic_header();
    print_dynamic_snapshot(&prev);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {},
            _ = signal::ctrl_c() => {
                println!("\nStopping live dynamic polling (received Ctrl+C)");
                return Ok(());
            }
        }
        poll_count += 1;
        let curr = match client.get_all_dynamic_info().await {
            Ok(v) => {
                consecutive_failures = 0;
                v
            }
            Err(e) => {
                consecutive_failures += 1;
                let backoff = Duration::from_secs(1 << consecutive_failures.min(5));
                eprintln!(
                    "Warning: poll #{} failed: {} (retrying in {}s)",
                    poll_count, e, backoff.as_secs()
                );
                tokio::time::sleep(backoff).await;
                continue;
            }
        };
        let deltas = compute_dynamic_deltas(&prev, &curr);

        if !deltas.is_empty() {
            println!("\n--- Poll #{} ({} changes) ---", poll_count, deltas.len());
            for d in &deltas {
                match d.kind {
                    DeltaKind::New => {
                        println!(
                            "  SN{:<3} {:<16} [NEW]  price: {:.6} τ/α  pool: {:.2} τ",
                            d.netuid,
                            truncate(&d.name, 16),
                            d.price_now,
                            d.tao_in_now as f64 / 1e9,
                        );
                    }
                    DeltaKind::Removed => {
                        println!(
                            "  SN{:<3} {:<16} [REMOVED]  was: {:.6} τ/α  pool: {:.2} τ",
                            d.netuid,
                            truncate(&d.name, 16),
                            d.price_prev,
                            d.tao_in_prev as f64 / 1e9,
                        );
                    }
                    DeltaKind::Changed => {
                        let arrow = if d.price_pct > 0.0 {
                            "↑"
                        } else if d.price_pct < 0.0 {
                            "↓"
                        } else {
                            "→"
                        };
                        println!(
                            "  SN{:<3} {:<16} {:>10.6} → {:>10.6} τ/α  ({}{:>+.2}%)  pool: {:.2} → {:.2} τ",
                            d.netuid,
                            truncate(&d.name, 16),
                            d.price_prev,
                            d.price_now,
                            arrow,
                            d.price_pct,
                            d.tao_in_prev as f64 / 1e9,
                            d.tao_in_now as f64 / 1e9,
                        );
                    }
                }
            }
            let _ = std::io::stdout().flush();
        }
        prev = curr;
    }
}

/// Run live metagraph polling loop.
pub async fn live_metagraph(client: &Client, netuid: NetUid, interval_secs: u64) -> Result<()> {
    let interval = Duration::from_secs(if interval_secs == 0 {
        DEFAULT_POLL_SECS
    } else {
        interval_secs
    });
    let mut prev_neurons = client.get_neurons_lite(netuid).await?;
    let mut poll_count = 0u64;
    let mut consecutive_failures: u32 = 0;

    println!(
        "Live metagraph for SN{} (polling every {}s, Ctrl+C to stop)\n",
        netuid.0,
        interval.as_secs()
    );
    println!("Tracking {} neurons...", prev_neurons.len());

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {},
            _ = signal::ctrl_c() => {
                println!("\nStopping live metagraph polling (received Ctrl+C)");
                return Ok(());
            }
        }
        poll_count += 1;
        let curr_neurons = match client.get_neurons_lite(netuid).await {
            Ok(v) => {
                consecutive_failures = 0;
                v
            }
            Err(e) => {
                consecutive_failures += 1;
                let backoff = Duration::from_secs(1 << consecutive_failures.min(5));
                eprintln!(
                    "Warning: poll #{} failed: {} (retrying in {}s)",
                    poll_count, e, backoff.as_secs()
                );
                tokio::time::sleep(backoff).await;
                continue;
            }
        };
        let mut changes = Vec::new();

        let prev_map: HashMap<u16, &crate::types::chain_data::NeuronInfoLite> =
            prev_neurons.iter().map(|n| (n.uid, n)).collect();

        for c in curr_neurons.iter() {
            if let Some(p) = prev_map.get(&c.uid) {
                let stake_diff = c.stake.rao() as i128 - p.stake.rao() as i128;
                let incentive_diff = c.incentive - p.incentive;
                let emission_diff = c.emission - p.emission;

                if stake_diff.unsigned_abs() > 100_000_000 // > 0.1 TAO
                    || incentive_diff.abs() > 0.001
                    || emission_diff.abs() > 0.001
                {
                    changes.push(format!(
                        "  UID {:<4} stake:{:>+.4}τ  incentive:{:>+.4}  emission:{:>+.1}",
                        c.uid,
                        stake_diff as f64 / 1e9,
                        incentive_diff,
                        emission_diff,
                    ));
                }
            } else {
                let hk_prefix = if c.hotkey.len() >= 8 {
                    &c.hotkey[..8]
                } else {
                    &c.hotkey
                };
                changes.push(format!(
                    "  UID {:<4} NEW neuron (hotkey: {})",
                    c.uid, hk_prefix
                ));
            }
        }

        if !changes.is_empty() {
            println!("\n--- Poll #{} ({} changes) ---", poll_count, changes.len());
            for ch in &changes {
                println!("{}", ch);
            }
            let _ = std::io::stdout().flush();
        }
        prev_neurons = curr_neurons;
    }
}

/// Run live portfolio polling.
pub async fn live_portfolio(client: &Client, coldkey_ss58: &str, interval_secs: u64) -> Result<()> {
    let interval = Duration::from_secs(if interval_secs == 0 {
        DEFAULT_POLL_SECS
    } else {
        interval_secs
    });
    let mut prev = crate::queries::portfolio::fetch_portfolio(client, coldkey_ss58).await?;
    let mut poll_count = 0u64;
    let mut consecutive_failures: u32 = 0;

    println!(
        "Live portfolio for {} (polling every {}s, Ctrl+C to stop)\n",
        coldkey_ss58,
        interval.as_secs()
    );
    println!(
        "Free: {}  Staked: {}  Positions: {}",
        prev.free_balance,
        prev.total_staked,
        prev.positions.len()
    );

    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {},
            _ = signal::ctrl_c() => {
                println!("\nStopping live portfolio polling (received Ctrl+C)");
                return Ok(());
            }
        }
        poll_count += 1;
        let curr = match crate::queries::portfolio::fetch_portfolio(client, coldkey_ss58).await {
            Ok(v) => {
                consecutive_failures = 0;
                v
            }
            Err(e) => {
                consecutive_failures += 1;
                let backoff = Duration::from_secs(1 << consecutive_failures.min(5));
                eprintln!(
                    "Warning: poll #{} failed: {} (retrying in {}s)",
                    poll_count, e, backoff.as_secs()
                );
                tokio::time::sleep(backoff).await;
                continue;
            }
        };

        let free_diff = curr.free_balance.rao() as i128 - prev.free_balance.rao() as i128;
        let staked_diff = curr.total_staked.rao() as i128 - prev.total_staked.rao() as i128;

        if free_diff.unsigned_abs() > 100_000 || staked_diff.unsigned_abs() > 100_000 {
            println!(
                "\n--- Poll #{} ---  Free:{:>+.6}τ  Staked:{:>+.6}τ",
                poll_count,
                free_diff as f64 / 1e9,
                staked_diff as f64 / 1e9,
            );
        }
        prev = curr;
    }
}

/// Print dynamic info table header.
fn print_dynamic_header() {
    println!("Live Dynamic TAO (Ctrl+C to stop)\n");
    println!(
        "{:<5} {:<16} {:>12} {:>12} {:>12} {:>12}",
        "SN", "Name", "Price (τ/α)", "TAO Pool", "Alpha In", "Volume"
    );
    println!("{}", "-".repeat(75));
}

/// Print full dynamic snapshot.
fn print_dynamic_snapshot(subnets: &[DynamicInfo]) {
    for d in subnets {
        if d.tao_in.rao() == 0 {
            continue; // skip empty subnets
        }
        println!(
            "{:<5} {:<16} {:>12.6} {:>12.2} {:>12.2} {:>12}",
            d.netuid.0,
            truncate(&d.name, 16),
            d.price,
            d.tao_in.tao(),
            d.alpha_in.raw() as f64 / 1e9,
            d.subnet_volume,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::balance::{AlphaBalance, Balance};
    use crate::types::network::NetUid;

    fn make_di(netuid: u16, price: f64, tao_rao: u64, volume: u128) -> DynamicInfo {
        DynamicInfo {
            netuid: NetUid(netuid),
            name: format!("SN{}", netuid),
            symbol: format!("α{}", netuid),
            tempo: 360,
            emission: 0,
            tao_in: Balance::from_rao(tao_rao),
            alpha_in: AlphaBalance::from_raw(1_000_000_000),
            alpha_out: AlphaBalance::from_raw(1_000_000_000),
            price,
            owner_hotkey: String::new(),
            owner_coldkey: String::new(),
            last_step: 0,
            blocks_since_last_step: 0,
            alpha_out_emission: 0,
            alpha_in_emission: 0,
            tao_in_emission: 0,
            pending_alpha_emission: 0,
            pending_root_emission: 0,
            subnet_volume: volume,
            network_registered_at: 0,
        }
    }

    #[test]
    fn default_poll_secs_value() {
        assert_eq!(DEFAULT_POLL_SECS, 12);
    }

    #[test]
    fn identical_snapshots_produce_no_deltas() {
        let snap = vec![
            make_di(1, 1.0, 1_000_000_000, 500),
            make_di(2, 2.5, 2_000_000_000, 1000),
        ];
        let deltas = compute_dynamic_deltas(&snap, &snap);
        assert!(deltas.is_empty(), "identical snapshots should yield zero deltas");
    }

    #[test]
    fn price_change_produces_delta_with_correct_pct() {
        let prev = vec![make_di(1, 2.0, 1_000_000_000, 500)];
        let curr = vec![make_di(1, 3.0, 1_000_000_000, 500)];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        let d = &deltas[0];
        assert_eq!(d.netuid, 1);
        assert!((d.price_prev - 2.0).abs() < 1e-10);
        assert!((d.price_now - 3.0).abs() < 1e-10);
        // (3.0 - 2.0) / 2.0 * 100 = 50%
        assert!((d.price_pct - 50.0).abs() < 1e-10, "expected 50%, got {}", d.price_pct);
    }

    #[test]
    fn tao_pool_change_produces_delta() {
        let prev = vec![make_di(5, 1.0, 1_000_000_000, 100)];
        let curr = vec![make_di(5, 1.0, 2_000_000_000, 100)];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        let d = &deltas[0];
        assert_eq!(d.tao_in_prev, 1_000_000_000);
        assert_eq!(d.tao_in_now, 2_000_000_000);
    }

    #[test]
    fn volume_change_produces_delta() {
        let prev = vec![make_di(3, 1.5, 500_000_000, 100)];
        let curr = vec![make_di(3, 1.5, 500_000_000, 999)];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        let d = &deltas[0];
        assert_eq!(d.volume_prev, 100);
        assert_eq!(d.volume_now, 999);
    }

    #[test]
    fn new_subnet_produces_new_delta() {
        let prev = vec![make_di(1, 1.0, 1_000_000_000, 100)];
        let curr = vec![
            make_di(1, 1.0, 1_000_000_000, 100), // unchanged
            make_di(99, 5.0, 3_000_000_000, 200), // new, not in prev
        ];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        // SN1 is unchanged (no delta), SN99 is new (produces New delta)
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].netuid, 99);
        assert_eq!(deltas[0].kind, DeltaKind::New);
        assert!((deltas[0].price_now - 5.0).abs() < 1e-10);
        assert_eq!(deltas[0].tao_in_now, 3_000_000_000);
    }

    #[test]
    fn multiple_changes_produce_multiple_deltas() {
        let prev = vec![
            make_di(1, 1.0, 1_000_000_000, 100),
            make_di(2, 2.0, 2_000_000_000, 200),
            make_di(3, 3.0, 3_000_000_000, 300),
        ];
        let curr = vec![
            make_di(1, 1.1, 1_000_000_000, 100), // price changed
            make_di(2, 2.0, 2_000_000_000, 200), // unchanged
            make_di(3, 3.0, 4_000_000_000, 300), // tao_in changed
        ];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 2, "expected 2 deltas, got {}", deltas.len());
        let netuids: Vec<u16> = deltas.iter().map(|d| d.netuid).collect();
        assert!(netuids.contains(&1));
        assert!(netuids.contains(&3));
    }

    #[test]
    fn delta_field_values_are_correct() {
        let prev = vec![make_di(7, 4.0, 10_000_000_000, 5000)];
        let curr = vec![make_di(7, 5.0, 12_000_000_000, 6000)];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        let d = &deltas[0];
        assert_eq!(d.netuid, 7);
        assert_eq!(d.name, "SN7");
        assert!((d.price_prev - 4.0).abs() < 1e-10);
        assert!((d.price_now - 5.0).abs() < 1e-10);
        // (5 - 4) / 4 * 100 = 25%
        assert!((d.price_pct - 25.0).abs() < 1e-10);
        assert_eq!(d.tao_in_prev, 10_000_000_000);
        assert_eq!(d.tao_in_now, 12_000_000_000);
        assert_eq!(d.volume_prev, 5000);
        assert_eq!(d.volume_now, 6000);
    }

    // ========== Issue 722: New/removed subnet detection ==========

    #[test]
    fn removed_subnet_produces_removed_delta() {
        let prev = vec![
            make_di(1, 1.0, 1_000_000_000, 100),
            make_di(5, 3.0, 2_000_000_000, 500),
        ];
        let curr = vec![
            make_di(1, 1.0, 1_000_000_000, 100), // unchanged
            // SN5 removed
        ];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].netuid, 5);
        assert_eq!(deltas[0].kind, DeltaKind::Removed);
        assert!((deltas[0].price_prev - 3.0).abs() < 1e-10);
        assert!((deltas[0].price_now - 0.0).abs() < 1e-10);
        assert_eq!(deltas[0].tao_in_prev, 2_000_000_000);
        assert_eq!(deltas[0].tao_in_now, 0);
    }

    #[test]
    fn new_subnet_has_zero_prev_values() {
        let prev = vec![];
        let curr = vec![make_di(42, 7.5, 5_000_000_000, 999)];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        let d = &deltas[0];
        assert_eq!(d.kind, DeltaKind::New);
        assert_eq!(d.netuid, 42);
        assert!((d.price_prev - 0.0).abs() < 1e-10);
        assert!((d.price_now - 7.5).abs() < 1e-10);
        assert_eq!(d.tao_in_prev, 0);
        assert_eq!(d.tao_in_now, 5_000_000_000);
        assert_eq!(d.volume_prev, 0);
        assert_eq!(d.volume_now, 999);
    }

    #[test]
    fn simultaneous_new_changed_removed() {
        let prev = vec![
            make_di(1, 1.0, 1_000_000_000, 100),
            make_di(2, 2.0, 2_000_000_000, 200), // will be removed
        ];
        let curr = vec![
            make_di(1, 1.5, 1_000_000_000, 100), // changed
            make_di(3, 3.0, 3_000_000_000, 300),  // new
        ];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 3, "expected changed + new + removed");
        let changed: Vec<_> = deltas.iter().filter(|d| d.kind == DeltaKind::Changed).collect();
        let new: Vec<_> = deltas.iter().filter(|d| d.kind == DeltaKind::New).collect();
        let removed: Vec<_> = deltas.iter().filter(|d| d.kind == DeltaKind::Removed).collect();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].netuid, 1);
        assert_eq!(new.len(), 1);
        assert_eq!(new[0].netuid, 3);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].netuid, 2);
    }

    #[test]
    fn changed_delta_has_changed_kind() {
        let prev = vec![make_di(1, 1.0, 1_000_000_000, 100)];
        let curr = vec![make_di(1, 2.0, 1_000_000_000, 100)];
        let deltas = compute_dynamic_deltas(&prev, &curr);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].kind, DeltaKind::Changed);
    }

    #[test]
    fn both_empty_produces_no_deltas() {
        let deltas = compute_dynamic_deltas(&[], &[]);
        assert!(deltas.is_empty());
    }
}
