//! Admin command handlers — sudo AdminUtils hyperparameter management.

use crate::admin;
use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::AdminCommands;
use anyhow::Result;
use sp_core::{sr25519, Pair as _};
use subxt::dynamic::Value;

/// Resolve a sudo keypair from a URI string (e.g. "//Alice") or from the wallet.
fn resolve_sudo_key(sudo_key: &Option<String>, ctx: &Ctx<'_>) -> Result<sr25519::Pair> {
    if let Some(ref uri) = sudo_key {
        // Try as dev URI first (//Alice, //Bob, etc.)
        match sr25519::Pair::from_string(uri, None) {
            Ok(pair) => return Ok(pair),
            Err(_) => {
                anyhow::bail!(
                    "Invalid sudo key URI '{}'. Use a dev URI like //Alice or //Bob.",
                    uri
                );
            }
        }
    }
    // Fall back to wallet coldkey
    let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
    unlock_coldkey(&mut wallet, ctx.password)?;
    Ok(wallet.coldkey()?.clone())
}

pub(super) async fn handle_admin(cmd: AdminCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    match cmd {
        AdminCommands::SetTempo {
            netuid,
            tempo,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_tempo(client, &pair, netuid, tempo).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Tempo set to {} on SN{}", tempo, netuid),
            );
            Ok(())
        }

        AdminCommands::SetMaxValidators {
            netuid,
            max,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_max_allowed_validators(client, &pair, netuid, max).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Max validators set to {} on SN{}", max, netuid),
            );
            Ok(())
        }

        AdminCommands::SetMaxUids {
            netuid,
            max,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_max_allowed_uids(client, &pair, netuid, max).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Max UIDs set to {} on SN{}", max, netuid),
            );
            Ok(())
        }

        AdminCommands::SetImmunityPeriod {
            netuid,
            period,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_immunity_period(client, &pair, netuid, period).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Immunity period set to {} on SN{}", period, netuid),
            );
            Ok(())
        }

        AdminCommands::SetMinWeights {
            netuid,
            min,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_min_allowed_weights(client, &pair, netuid, min).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Min weights set to {} on SN{}", min, netuid),
            );
            Ok(())
        }

        AdminCommands::SetMaxWeightLimit {
            netuid,
            limit,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_max_weight_limit(client, &pair, netuid, limit).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Max weight limit set to {} on SN{}", limit, netuid),
            );
            Ok(())
        }

        AdminCommands::SetWeightsRateLimit {
            netuid,
            limit,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_weights_set_rate_limit(client, &pair, netuid, limit).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Weights rate limit set to {} on SN{}", limit, netuid),
            );
            Ok(())
        }

        AdminCommands::SetCommitReveal {
            netuid,
            enabled,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash =
                admin::set_commit_reveal_weights_enabled(client, &pair, netuid, enabled).await?;
            let state = if enabled { "enabled" } else { "disabled" };
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Commit-reveal {} on SN{}", state, netuid),
            );
            Ok(())
        }

        AdminCommands::SetDifficulty {
            netuid,
            difficulty,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_difficulty(client, &pair, netuid, difficulty).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Difficulty set to {} on SN{}", difficulty, netuid),
            );
            Ok(())
        }

        AdminCommands::SetActivityCutoff {
            netuid,
            cutoff,
            sudo_key,
        } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_activity_cutoff(client, &pair, netuid, cutoff).await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("Activity cutoff set to {} on SN{}", cutoff, netuid),
            );
            Ok(())
        }

        AdminCommands::SetDefaultTake { take, sudo_key } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_default_take(client, &pair, take).await?;
            print_tx_result(ctx.output, &hash, &format!("Default take set to {}", take));
            Ok(())
        }

        AdminCommands::SetTxRateLimit { limit, sudo_key } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_tx_rate_limit(client, &pair, limit).await?;
            print_tx_result(ctx.output, &hash, &format!("TX rate limit set to {}", limit));
            Ok(())
        }

        AdminCommands::SetMinDifficulty { netuid, difficulty, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_min_difficulty(client, &pair, netuid, difficulty).await?;
            print_tx_result(ctx.output, &hash, &format!("Min difficulty set to {} on SN{}", difficulty, netuid));
            Ok(())
        }

        AdminCommands::SetMaxDifficulty { netuid, difficulty, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_max_difficulty(client, &pair, netuid, difficulty).await?;
            print_tx_result(ctx.output, &hash, &format!("Max difficulty set to {} on SN{}", difficulty, netuid));
            Ok(())
        }

        AdminCommands::SetAdjustmentInterval { netuid, interval, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_adjustment_interval(client, &pair, netuid, interval).await?;
            print_tx_result(ctx.output, &hash, &format!("Adjustment interval set to {} on SN{}", interval, netuid));
            Ok(())
        }

        AdminCommands::SetKappa { netuid, kappa, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_kappa(client, &pair, netuid, kappa).await?;
            print_tx_result(ctx.output, &hash, &format!("Kappa set to {} on SN{}", kappa, netuid));
            Ok(())
        }

        AdminCommands::SetRho { netuid, rho, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_rho(client, &pair, netuid, rho).await?;
            print_tx_result(ctx.output, &hash, &format!("Rho set to {} on SN{}", rho, netuid));
            Ok(())
        }

        AdminCommands::SetMinBurn { netuid, burn, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_min_burn(client, &pair, netuid, burn).await?;
            print_tx_result(ctx.output, &hash, &format!("Min burn set to {} on SN{}", burn, netuid));
            Ok(())
        }

        AdminCommands::SetMaxBurn { netuid, burn, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_max_burn(client, &pair, netuid, burn).await?;
            print_tx_result(ctx.output, &hash, &format!("Max burn set to {} on SN{}", burn, netuid));
            Ok(())
        }

        AdminCommands::SetLiquidAlpha { netuid, enabled, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_liquid_alpha_enabled(client, &pair, netuid, enabled).await?;
            let state = if enabled { "enabled" } else { "disabled" };
            print_tx_result(ctx.output, &hash, &format!("Liquid alpha {} on SN{}", state, netuid));
            Ok(())
        }

        AdminCommands::SetAlphaValues { netuid, alpha_low, alpha_high, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_alpha_values(client, &pair, netuid, alpha_low, alpha_high).await?;
            print_tx_result(ctx.output, &hash, &format!("Alpha values set to low={}, high={} on SN{}", alpha_low, alpha_high, netuid));
            Ok(())
        }

        AdminCommands::SetYuma3 { netuid, enabled, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_yuma3_enabled(client, &pair, netuid, enabled).await?;
            let state = if enabled { "enabled" } else { "disabled" };
            print_tx_result(ctx.output, &hash, &format!("Yuma3 {} on SN{}", state, netuid));
            Ok(())
        }

        AdminCommands::SetBondsPenalty { netuid, penalty, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_bonds_penalty(client, &pair, netuid, penalty).await?;
            print_tx_result(ctx.output, &hash, &format!("Bonds penalty set to {} on SN{}", penalty, netuid));
            Ok(())
        }

        AdminCommands::SetStakeThreshold { threshold, sudo_key } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_stake_threshold(client, &pair, threshold).await?;
            print_tx_result(ctx.output, &hash, &format!("Stake threshold set to {}", threshold));
            Ok(())
        }

        AdminCommands::SetNetworkRegistration { netuid, allowed, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_network_registration_allowed(client, &pair, netuid, allowed).await?;
            let state = if allowed { "enabled" } else { "disabled" };
            print_tx_result(ctx.output, &hash, &format!("Network registration {} on SN{}", state, netuid));
            Ok(())
        }

        AdminCommands::SetPowRegistration { netuid, allowed, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_network_pow_registration_allowed(client, &pair, netuid, allowed).await?;
            let state = if allowed { "enabled" } else { "disabled" };
            print_tx_result(ctx.output, &hash, &format!("POW registration {} on SN{}", state, netuid));
            Ok(())
        }

        AdminCommands::SetAdjustmentAlpha { netuid, alpha, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_adjustment_alpha(client, &pair, netuid, alpha).await?;
            print_tx_result(ctx.output, &hash, &format!("Adjustment alpha set to {} on SN{}", alpha, netuid));
            Ok(())
        }

        AdminCommands::SetSubnetMovingAlpha { alpha, sudo_key } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_subnet_moving_alpha(client, &pair, alpha).await?;
            print_tx_result(ctx.output, &hash, &format!("Subnet moving alpha set to {}", alpha));
            Ok(())
        }

        AdminCommands::SetMechanismCount { netuid, count, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_mechanism_count(client, &pair, netuid, count).await?;
            print_tx_result(ctx.output, &hash, &format!("Mechanism count set to {} on SN{}", count, netuid));
            Ok(())
        }

        AdminCommands::SetMechanismEmissionSplit { netuid, weights, sudo_key } => {
            validate_netuid(netuid)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let emission_weights: Vec<u64> = weights
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse::<u64>()
                        .map_err(|e| anyhow::anyhow!("Invalid emission weight '{}': {}", s, e))
                })
                .collect::<Result<Vec<_>>>()?;
            let hash = admin::set_mechanism_emission_split(client, &pair, netuid, emission_weights).await?;
            print_tx_result(ctx.output, &hash, &format!("Mechanism emission split set on SN{}", netuid));
            Ok(())
        }

        AdminCommands::SetNominatorMinStake { stake, sudo_key } => {
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            let hash = admin::set_nominator_min_required_stake(client, &pair, stake).await?;
            print_tx_result(ctx.output, &hash, &format!("Nominator min required stake set to {}", stake));
            Ok(())
        }

        AdminCommands::Raw {
            call,
            args,
            sudo_key,
        } => {
            validate_admin_call_name(&call)?;
            // Validate netuid in raw args — all known admin calls take netuid as
            // first arg; reject netuid 0 to prevent accidental root network
            // modification (Issue 710).
            validate_raw_admin_netuid(&call, &args)?;
            let pair = resolve_sudo_key(&sudo_key, ctx)?;
            // Parse args as JSON array of values
            let values = parse_raw_args(&args)?;
            // Require explicit confirmation for raw admin calls (Issue 709)
            confirm_action(&format!(
                "Execute sudo AdminUtils.{} with args {}?",
                call, args
            ))?;
            let hash = admin::raw_admin_call(client, &pair, &call, values).await?;
            print_tx_result(ctx.output, &hash, &format!("AdminUtils.{} executed", call));
            Ok(())
        }

        AdminCommands::List => {
            let params = admin::known_params();
            if ctx.output.is_json() {
                let items: Vec<_> = params
                    .iter()
                    .map(|(name, desc, args)| {
                        serde_json::json!({
                            "call": name,
                            "description": desc,
                            "args": args,
                        })
                    })
                    .collect();
                print_json(&serde_json::json!(items));
            } else {
                println!("Available AdminUtils parameters:\n");
                for (name, desc, args) in &params {
                    println!("  {} — {}", name, desc);
                    println!("    args: {}", args.join(", "));
                    println!();
                }
                println!("Use `agcli admin raw --call <name> --args '[...]' --sudo-key //Alice` for any call.");
            }
            Ok(())
        }
    }
}

/// Check if the first arg of a raw admin call is netuid 0 (root network).
/// All known AdminUtils calls take `netuid: u16` as the first argument.
fn validate_raw_admin_netuid(call: &str, args: &str) -> Result<()> {
    // All known admin calls take netuid as first arg — check it
    let known = crate::admin::known_params();
    let is_known = known.iter().any(|(n, _, _)| *n == call.trim());
    if !is_known {
        return Ok(()); // unknown call — can't validate structure
    }
    // Parse the first element to check for netuid 0
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(args) {
        if let Some(arr) = parsed.as_array() {
            if let Some(first) = arr.first() {
                if let Some(n) = first.as_u64() {
                    if n == 0 {
                        anyhow::bail!(
                            "Invalid netuid: 0 in admin raw call '{}'. Root network (netuid 0) is not a user subnet.\n  Tip: user subnets start at netuid 1.",
                            call
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

/// Parse a JSON array string into dynamic Values.
/// Accepts: '[1, 2, true]' or '[]' or individual values.
fn parse_raw_args(args: &str) -> Result<Vec<Value>> {
    let parsed: serde_json::Value = serde_json::from_str(args)
        .map_err(|e| anyhow::anyhow!("Invalid JSON args '{}': {}", args, e))?;

    match parsed {
        serde_json::Value::Array(arr) => arr.iter().map(json_to_value).collect(),
        _ => anyhow::bail!("Args must be a JSON array, got: {}", args),
    }
}

fn json_to_value(v: &serde_json::Value) -> Result<Value> {
    match v {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_u64() {
                Ok(Value::u128(i as u128))
            } else if let Some(i) = n.as_i64() {
                if i < 0 {
                    anyhow::bail!(
                        "Negative numbers are not allowed in admin call args (got {}). Chain parameters use unsigned integers.",
                        i
                    );
                }
                Ok(Value::u128(i as u128))
            } else {
                anyhow::bail!("Unsupported number type: {}", n)
            }
        }
        serde_json::Value::Bool(b) => Ok(Value::bool(*b)),
        serde_json::Value::String(s) => Ok(Value::string(s.clone())),
        _ => anyhow::bail!("Unsupported arg type: {}", v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== parse_raw_args tests ==========

    #[test]
    fn parse_raw_args_empty_array() {
        let result = parse_raw_args("[]").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn parse_raw_args_single_number() {
        let result = parse_raw_args("[42]").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn parse_raw_args_multiple_types() {
        let result = parse_raw_args("[1, true, \"hello\"]").unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn parse_raw_args_rejects_non_array() {
        assert!(parse_raw_args("42").is_err());
        assert!(parse_raw_args("\"hello\"").is_err());
        assert!(parse_raw_args("{}").is_err());
    }

    #[test]
    fn parse_raw_args_rejects_invalid_json() {
        assert!(parse_raw_args("not json").is_err());
        assert!(parse_raw_args("[1, 2,]").is_err()); // trailing comma
    }

    #[test]
    fn parse_raw_args_rejects_negative_numbers() {
        let result = parse_raw_args("[-1]");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Negative"), "Error should mention negative: {}", err);
    }

    #[test]
    fn parse_raw_args_rejects_nested_objects() {
        assert!(parse_raw_args("[{\"a\": 1}]").is_err());
    }

    #[test]
    fn parse_raw_args_rejects_nested_arrays() {
        assert!(parse_raw_args("[[1, 2]]").is_err());
    }

    // ========== confirm_action tests ==========

    #[test]
    fn confirm_action_skips_in_yes_mode() {
        // Enable yes mode, confirm should pass without reading stdin
        set_yes_mode(true);
        let result = confirm_action("Test?");
        set_yes_mode(false);
        assert!(result.is_ok());
    }

    #[test]
    fn confirm_action_skips_in_batch_mode() {
        set_batch_mode(true);
        let result = confirm_action("Test?");
        set_batch_mode(false);
        assert!(result.is_ok());
    }

    // ========== Issue 710: validate_raw_admin_netuid tests ==========

    #[test]
    fn raw_admin_rejects_netuid_zero() {
        let result = validate_raw_admin_netuid("sudo_set_tempo", "[0, 100]");
        assert!(result.is_err(), "Should reject netuid 0 in raw admin call");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("netuid: 0"),
            "Error should mention netuid 0: {}",
            msg
        );
    }

    #[test]
    fn raw_admin_accepts_valid_netuid() {
        assert!(validate_raw_admin_netuid("sudo_set_tempo", "[1, 100]").is_ok());
        assert!(validate_raw_admin_netuid("sudo_set_tempo", "[42, 200]").is_ok());
    }

    #[test]
    fn raw_admin_skips_validation_for_unknown_calls() {
        // Unknown calls can't be validated — let them through
        assert!(validate_raw_admin_netuid("some_new_call", "[0, 1]").is_ok());
    }

    #[test]
    fn raw_admin_handles_empty_args() {
        // Empty array — no netuid to validate
        assert!(validate_raw_admin_netuid("sudo_set_tempo", "[]").is_ok());
    }

    #[test]
    fn raw_admin_handles_non_numeric_first_arg() {
        // String first arg — can't validate as netuid, skip
        assert!(validate_raw_admin_netuid("sudo_set_tempo", "[\"hello\", 100]").is_ok());
    }

    #[test]
    fn raw_admin_handles_invalid_json() {
        // Invalid JSON — parse fails, validation skips gracefully
        assert!(validate_raw_admin_netuid("sudo_set_tempo", "not json").is_ok());
    }
}
