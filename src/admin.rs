//! AdminUtils sudo calls — set subnet hyperparameters via the chain's sudo mechanism.
//!
//! These functions wrap `submit_raw_call` for the `AdminUtils` pallet, making
//! it easy to configure subnets programmatically.
//!
//! ```rust,no_run
//! use agcli::admin;
//! use agcli::Client;
//! use sp_core::Pair as _;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = Client::connect("ws://127.0.0.1:9944").await?;
//! let alice = sp_core::sr25519::Pair::from_string("//Alice", None)?;
//! admin::set_tempo(&client, &alice, 1, 100).await?;
//! # Ok(())
//! # }
//! ```

use crate::chain::Client;
use anyhow::Result;
use sp_core::sr25519;
use subxt::dynamic::Value;

/// Set the tempo (blocks per epoch) for a subnet.
pub async fn set_tempo(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    tempo: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_tempo",
            vec![Value::u128(netuid as u128), Value::u128(tempo as u128)],
        )
        .await
}

/// Set max allowed validators for a subnet.
pub async fn set_max_allowed_validators(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    max: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_max_allowed_validators",
            vec![Value::u128(netuid as u128), Value::u128(max as u128)],
        )
        .await
}

/// Set max allowed UIDs for a subnet.
pub async fn set_max_allowed_uids(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    max: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_max_allowed_uids",
            vec![Value::u128(netuid as u128), Value::u128(max as u128)],
        )
        .await
}

/// Set immunity period for a subnet.
pub async fn set_immunity_period(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    period: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_immunity_period",
            vec![Value::u128(netuid as u128), Value::u128(period as u128)],
        )
        .await
}

/// Set min allowed weights for a subnet.
pub async fn set_min_allowed_weights(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    min: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_min_allowed_weights",
            vec![Value::u128(netuid as u128), Value::u128(min as u128)],
        )
        .await
}

/// Set max weights limit for a subnet.
pub async fn set_max_weight_limit(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    limit: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_max_weight_limit",
            vec![Value::u128(netuid as u128), Value::u128(limit as u128)],
        )
        .await
}

/// Set weights rate limit (0 = no rate limit).
pub async fn set_weights_set_rate_limit(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    limit: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_weights_set_rate_limit",
            vec![Value::u128(netuid as u128), Value::u128(limit as u128)],
        )
        .await
}

/// Set commit-reveal weights enabled/disabled.
pub async fn set_commit_reveal_weights_enabled(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    enabled: bool,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_commit_reveal_weights_enabled",
            vec![Value::u128(netuid as u128), Value::bool(enabled)],
        )
        .await
}

/// Set difficulty for a subnet.
pub async fn set_difficulty(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    difficulty: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_difficulty",
            vec![Value::u128(netuid as u128), Value::u128(difficulty as u128)],
        )
        .await
}

/// Set bonds moving average for a subnet.
pub async fn set_bonds_moving_average(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    avg: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_bonds_moving_average",
            vec![Value::u128(netuid as u128), Value::u128(avg as u128)],
        )
        .await
}

/// Set target registrations per interval for a subnet.
pub async fn set_target_registrations_per_interval(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    target: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_target_registrations_per_interval",
            vec![Value::u128(netuid as u128), Value::u128(target as u128)],
        )
        .await
}

/// Set activity cutoff for a subnet.
pub async fn set_activity_cutoff(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    cutoff: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_activity_cutoff",
            vec![Value::u128(netuid as u128), Value::u128(cutoff as u128)],
        )
        .await
}

/// Set serving rate limit for a subnet.
pub async fn set_serving_rate_limit(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    limit: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_serving_rate_limit",
            vec![Value::u128(netuid as u128), Value::u128(limit as u128)],
        )
        .await
}

// ──────── New admin wrappers from subtensor.com audit ────────

/// Set default delegate take rate.
pub async fn set_default_take(
    client: &Client,
    sudo_key: &sr25519::Pair,
    take: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_default_take",
            vec![Value::u128(take as u128)],
        )
        .await
}

/// Set global transaction rate limit.
pub async fn set_tx_rate_limit(
    client: &Client,
    sudo_key: &sr25519::Pair,
    limit: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_tx_rate_limit",
            vec![Value::u128(limit as u128)],
        )
        .await
}

/// Set minimum POW difficulty for a subnet.
pub async fn set_min_difficulty(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    min: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_min_difficulty",
            vec![Value::u128(netuid as u128), Value::u128(min as u128)],
        )
        .await
}

/// Set maximum POW difficulty for a subnet.
pub async fn set_max_difficulty(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    max: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_max_difficulty",
            vec![Value::u128(netuid as u128), Value::u128(max as u128)],
        )
        .await
}

/// Set adjustment interval for a subnet.
pub async fn set_adjustment_interval(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    interval: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_adjustment_interval",
            vec![Value::u128(netuid as u128), Value::u128(interval as u128)],
        )
        .await
}

/// Set adjustment alpha for a subnet's difficulty adjustment.
pub async fn set_adjustment_alpha(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    alpha: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_adjustment_alpha",
            vec![Value::u128(netuid as u128), Value::u128(alpha as u128)],
        )
        .await
}

/// Set kappa for a subnet.
pub async fn set_kappa(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    kappa: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_kappa",
            vec![Value::u128(netuid as u128), Value::u128(kappa as u128)],
        )
        .await
}

/// Set rho for a subnet.
pub async fn set_rho(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    rho: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_rho",
            vec![Value::u128(netuid as u128), Value::u128(rho as u128)],
        )
        .await
}

/// Set min burn for a subnet.
pub async fn set_min_burn(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    min_burn: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_min_burn",
            vec![Value::u128(netuid as u128), Value::u128(min_burn as u128)],
        )
        .await
}

/// Set max burn for a subnet.
pub async fn set_max_burn(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    max_burn: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_max_burn",
            vec![Value::u128(netuid as u128), Value::u128(max_burn as u128)],
        )
        .await
}

/// Enable or disable liquid alpha for a subnet.
pub async fn set_liquid_alpha_enabled(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    enabled: bool,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_liquid_alpha_enabled",
            vec![Value::u128(netuid as u128), Value::bool(enabled)],
        )
        .await
}

/// Set alpha values (high, low) for a subnet.
pub async fn set_alpha_values(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    alpha_low: u16,
    alpha_high: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_alpha_values",
            vec![
                Value::u128(netuid as u128),
                Value::u128(alpha_low as u128),
                Value::u128(alpha_high as u128),
            ],
        )
        .await
}

/// Enable or disable Yuma3 consensus for a subnet.
pub async fn set_yuma3_enabled(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    enabled: bool,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_yuma3_enabled",
            vec![Value::u128(netuid as u128), Value::bool(enabled)],
        )
        .await
}

/// Set bonds penalty for a subnet.
pub async fn set_bonds_penalty(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    penalty: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_bonds_penalty",
            vec![Value::u128(netuid as u128), Value::u128(penalty as u128)],
        )
        .await
}

/// Set subnet moving alpha.
pub async fn set_subnet_moving_alpha(
    client: &Client,
    sudo_key: &sr25519::Pair,
    alpha: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_subnet_moving_alpha",
            vec![Value::u128(alpha as u128)],
        )
        .await
}

/// Set mechanism count for a subnet.
pub async fn set_mechanism_count(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    count: u16,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_mechanism_count",
            vec![Value::u128(netuid as u128), Value::u128(count as u128)],
        )
        .await
}

/// Set mechanism emission split for a subnet.
pub async fn set_mechanism_emission_split(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    split: Vec<u64>,
) -> Result<String> {
    let split_vals: Vec<Value> = split.iter().map(|s| Value::u128(*s as u128)).collect();
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_mechanism_emission_split",
            vec![
                Value::u128(netuid as u128),
                Value::unnamed_composite(split_vals),
            ],
        )
        .await
}

/// Set stake threshold for validators.
pub async fn set_stake_threshold(
    client: &Client,
    sudo_key: &sr25519::Pair,
    threshold: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_stake_threshold",
            vec![Value::u128(threshold as u128)],
        )
        .await
}

/// Set nominator minimum required stake.
pub async fn set_nominator_min_required_stake(
    client: &Client,
    sudo_key: &sr25519::Pair,
    min_stake: u64,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_nominator_min_required_stake",
            vec![Value::u128(min_stake as u128)],
        )
        .await
}

/// Enable or disable network registration for a subnet.
pub async fn set_network_registration_allowed(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    allowed: bool,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_network_registration_allowed",
            vec![Value::u128(netuid as u128), Value::bool(allowed)],
        )
        .await
}

/// Enable or disable POW network registration for a subnet.
pub async fn set_network_pow_registration_allowed(
    client: &Client,
    sudo_key: &sr25519::Pair,
    netuid: u16,
    allowed: bool,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(
            sudo_key,
            "AdminUtils",
            "sudo_set_network_pow_registration_allowed",
            vec![Value::u128(netuid as u128), Value::bool(allowed)],
        )
        .await
}

/// Generic AdminUtils call for parameters not covered by specific helpers.
///
/// `call_name` is the AdminUtils extrinsic name (e.g. "sudo_set_tempo").
/// `args` are the SCALE-encoded arguments as dynamic values.
pub async fn raw_admin_call(
    client: &Client,
    sudo_key: &sr25519::Pair,
    call_name: &str,
    args: Vec<Value>,
) -> Result<String> {
    client
        .submit_sudo_raw_call_checked(sudo_key, "AdminUtils", call_name, args)
        .await
}

/// All known AdminUtils parameters and their expected argument counts.
/// Returns (call_name, description, arg_types).
pub fn known_params() -> Vec<(&'static str, &'static str, &'static [&'static str])> {
    vec![
        (
            "sudo_set_tempo",
            "Blocks per epoch",
            &["netuid: u16", "tempo: u16"],
        ),
        (
            "sudo_set_max_allowed_validators",
            "Max validator slots",
            &["netuid: u16", "max: u16"],
        ),
        (
            "sudo_set_max_allowed_uids",
            "Max total UID slots",
            &["netuid: u16", "max: u16"],
        ),
        (
            "sudo_set_immunity_period",
            "Blocks of immunity after registration",
            &["netuid: u16", "period: u16"],
        ),
        (
            "sudo_set_min_allowed_weights",
            "Minimum weights a validator must set",
            &["netuid: u16", "min: u16"],
        ),
        (
            "sudo_set_max_weight_limit",
            "Maximum weight value",
            &["netuid: u16", "limit: u16"],
        ),
        (
            "sudo_set_weights_set_rate_limit",
            "Blocks between weight submissions (0=unlimited)",
            &["netuid: u16", "limit: u64"],
        ),
        (
            "sudo_set_commit_reveal_weights_enabled",
            "Enable/disable commit-reveal weights",
            &["netuid: u16", "enabled: bool"],
        ),
        (
            "sudo_set_difficulty",
            "POW registration difficulty",
            &["netuid: u16", "difficulty: u64"],
        ),
        (
            "sudo_set_bonds_moving_average",
            "Bonds moving average",
            &["netuid: u16", "avg: u64"],
        ),
        (
            "sudo_set_target_registrations_per_interval",
            "Target registrations per interval",
            &["netuid: u16", "target: u16"],
        ),
        (
            "sudo_set_activity_cutoff",
            "Blocks before a neuron is considered inactive",
            &["netuid: u16", "cutoff: u16"],
        ),
        (
            "sudo_set_serving_rate_limit",
            "Axon serving rate limit",
            &["netuid: u16", "limit: u64"],
        ),
        (
            "sudo_set_default_take",
            "Default delegate take rate",
            &["take: u16"],
        ),
        (
            "sudo_set_tx_rate_limit",
            "Global transaction rate limit",
            &["limit: u64"],
        ),
        (
            "sudo_set_min_difficulty",
            "Minimum POW difficulty",
            &["netuid: u16", "min: u64"],
        ),
        (
            "sudo_set_max_difficulty",
            "Maximum POW difficulty",
            &["netuid: u16", "max: u64"],
        ),
        (
            "sudo_set_adjustment_interval",
            "Difficulty adjustment interval",
            &["netuid: u16", "interval: u16"],
        ),
        (
            "sudo_set_adjustment_alpha",
            "Difficulty adjustment alpha",
            &["netuid: u16", "alpha: u64"],
        ),
        (
            "sudo_set_kappa",
            "Kappa parameter",
            &["netuid: u16", "kappa: u16"],
        ),
        (
            "sudo_set_rho",
            "Rho parameter",
            &["netuid: u16", "rho: u16"],
        ),
        (
            "sudo_set_min_burn",
            "Minimum burn for registration",
            &["netuid: u16", "min_burn: u64"],
        ),
        (
            "sudo_set_max_burn",
            "Maximum burn for registration",
            &["netuid: u16", "max_burn: u64"],
        ),
        (
            "sudo_set_liquid_alpha_enabled",
            "Enable/disable liquid alpha",
            &["netuid: u16", "enabled: bool"],
        ),
        (
            "sudo_set_alpha_values",
            "Alpha values (low, high)",
            &["netuid: u16", "alpha_low: u16", "alpha_high: u16"],
        ),
        (
            "sudo_set_yuma3_enabled",
            "Enable/disable Yuma3 consensus",
            &["netuid: u16", "enabled: bool"],
        ),
        (
            "sudo_set_bonds_penalty",
            "Bonds penalty",
            &["netuid: u16", "penalty: u16"],
        ),
        (
            "sudo_set_subnet_moving_alpha",
            "Subnet moving alpha (global)",
            &["alpha: u64"],
        ),
        (
            "sudo_set_mechanism_count",
            "Number of mechanisms in a subnet",
            &["netuid: u16", "count: u16"],
        ),
        (
            "sudo_set_mechanism_emission_split",
            "Emission split across mechanisms",
            &["netuid: u16", "split: Vec<u64>"],
        ),
        (
            "sudo_set_stake_threshold",
            "Minimum stake threshold for validators",
            &["threshold: u64"],
        ),
        (
            "sudo_set_nominator_min_required_stake",
            "Minimum required stake for nominators",
            &["min_stake: u64"],
        ),
        (
            "sudo_set_network_registration_allowed",
            "Enable/disable network registration",
            &["netuid: u16", "allowed: bool"],
        ),
        (
            "sudo_set_network_pow_registration_allowed",
            "Enable/disable POW network registration",
            &["netuid: u16", "allowed: bool"],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn known_params_is_non_empty() {
        let params = known_params();
        assert!(!params.is_empty(), "known_params should return a non-empty list");
    }

    #[test]
    fn all_entries_have_non_empty_fields() {
        for (call_name, description, args) in known_params() {
            assert!(
                !call_name.is_empty(),
                "call_name must not be empty"
            );
            assert!(
                !description.is_empty(),
                "description must not be empty for {}",
                call_name
            );
            assert!(
                !args.is_empty(),
                "args must not be empty for {}",
                call_name
            );
        }
    }

    #[test]
    fn all_call_names_start_with_sudo_set() {
        for (call_name, _, _) in known_params() {
            assert!(
                call_name.starts_with("sudo_set_"),
                "call_name '{}' does not start with 'sudo_set_'",
                call_name
            );
        }
    }

    #[test]
    fn subnet_params_first_arg_is_netuid_u16() {
        // Global params (no netuid) are allowed — only check per-subnet params
        let global_params = [
            "sudo_set_default_take",
            "sudo_set_tx_rate_limit",
            "sudo_set_subnet_moving_alpha",
            "sudo_set_stake_threshold",
            "sudo_set_nominator_min_required_stake",
        ];
        for (call_name, _, args) in known_params() {
            assert!(
                !args.is_empty(),
                "args must not be empty for {}",
                call_name
            );
            if !global_params.contains(&call_name) {
                assert_eq!(
                    args[0], "netuid: u16",
                    "first arg of {} should be 'netuid: u16', got '{}'",
                    call_name, args[0]
                );
            }
        }
    }

    #[test]
    fn no_duplicate_call_names() {
        let params = known_params();
        let mut seen = HashSet::new();
        for (call_name, _, _) in &params {
            assert!(
                seen.insert(call_name),
                "duplicate call_name found: {}",
                call_name
            );
        }
        assert_eq!(seen.len(), params.len());
    }
}
