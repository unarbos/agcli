//! Stake command handlers.

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::{OutputFormat, StakeCommands};
use crate::types::balance::AlphaBalance;
use crate::types::NetUid;
use anyhow::{Context, Result};

pub async fn handle_stake(cmd: StakeCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name) = (ctx.wallet_dir, ctx.wallet_name, ctx.hotkey_name);
    let (output, password, mev) = (ctx.output, ctx.password, ctx.mev);
    match cmd {
        StakeCommands::List { address, at_block } => {
            let addr = resolve_and_validate_coldkey_address(
                address,
                wallet_dir,
                wallet_name,
                "stake list --address",
            )?;

            // Historical wayback mode
            if let Some(block_num) = at_block {
                let block_hash = client.get_block_hash(block_num).await?;
                let stakes = client
                    .get_stake_for_coldkey_at_block(&addr, block_hash)
                    .await?;
                let prices = client
                    .current_alpha_price_all_at_block(block_hash)
                    .await
                    .unwrap_or_default();
                if output.is_json() {
                    print_json(&serde_json::json!({
                        "address": addr,
                        "block": block_num,
                        "block_hash": format!("{:?}", block_hash),
                        "stakes": stakes.iter().map(|s| {
                            let price = prices.get(&s.netuid.0).copied();
                            serde_json::json!({
                                "netuid": s.netuid.0,
                                "hotkey": s.hotkey,
                                "alpha_raw": s.stake.raw(),
                                "tao_equiv_rao": price.map(|p| s.stake.to_tao(p).rao()),
                                "price": price,
                            })
                        }).collect::<Vec<_>>(),
                    }));
                } else if stakes.is_empty() {
                    println!(
                        "No stakes found for {} at block {}",
                        crate::utils::short_ss58(&addr),
                        block_num
                    );
                } else {
                    render_rows(
                        OutputFormat::Table,
                        &stakes,
                        "",
                        |_| String::new(),
                        &["Subnet", "Hotkey", "Alpha (α)", "TAO≈ (τ)", "Price (τ/α)"],
                        |s| {
                            let price = prices.get(&s.netuid.0).copied();
                            vec![
                                format!("SN{}", s.netuid),
                                crate::utils::short_ss58(&s.hotkey),
                                s.stake.display_units(),
                                price
                                    .map(|p| s.stake.to_tao(p).display_tao())
                                    .unwrap_or_else(|| "—".to_string()),
                                price
                                    .map(|p| format!("{p:.6}"))
                                    .unwrap_or_else(|| "—".to_string()),
                            ]
                        },
                        Some(&format!(
                            "Stakes for {} (at block {}):",
                            crate::utils::short_ss58(&addr),
                            block_num
                        )),
                    );
                }
                return Ok(());
            }

            let stakes = client.get_stake_for_coldkey(&addr).await?;
            let prices = client.current_alpha_price_all().await.unwrap_or_default();
            if stakes.is_empty() && !output.is_json() && !output.is_csv() {
                println!("No stakes found for {}", crate::utils::short_ss58(&addr));
            } else {
                render_rows(
                    output,
                    &stakes,
                    "netuid,hotkey,alpha_raw,tao_equiv_rao,price",
                    |s| {
                        let price = prices.get(&s.netuid.0).copied();
                        format!(
                            "{},{},{},{},{}",
                            s.netuid,
                            s.hotkey,
                            s.stake.raw(),
                            price
                                .map(|p| s.stake.to_tao(p).rao().to_string())
                                .unwrap_or_default(),
                            price.map(|p| format!("{p:.6}")).unwrap_or_default()
                        )
                    },
                    &["Subnet", "Hotkey", "Alpha (α)", "TAO≈ (τ)", "Price (τ/α)"],
                    |s| {
                        let price = prices.get(&s.netuid.0).copied();
                        vec![
                            format!("SN{}", s.netuid),
                            crate::utils::short_ss58(&s.hotkey),
                            s.stake.display_units(),
                            price
                                .map(|p| s.stake.to_tao(p).display_tao())
                                .unwrap_or_else(|| "—".to_string()),
                            price
                                .map(|p| format!("{p:.6}"))
                                .unwrap_or_else(|| "—".to_string()),
                        ]
                    },
                    Some(&format!("Stakes for {}:", crate::utils::short_ss58(&addr))),
                );
            }
            Ok(())
        }
        StakeCommands::Add {
            amount,
            netuid,
            hotkey,
            max_slippage,
        } => {
            validate_netuid(netuid)?;
            let bal = parse_cli_tao_amount(amount, "stake amount")?;
            // Spending limit check
            check_spending_limit(netuid, amount)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let pubkey = sp_core::Pair::public(&pair);
            // Pre-flight checks: balance + slippage in parallel when both needed
            if let Some(max_slip) = max_slippage {
                let (current, _) = tokio::try_join!(
                    client.get_balance(&pubkey),
                    check_slippage(client, netuid, amount, max_slip, true),
                )?;
                if current.rao() < bal.rao() {
                    anyhow::bail!("Insufficient balance: you have {} but trying to stake {}.\n  Check: agcli balance",
                        current.display_tao(), bal.display_tao());
                }
            } else {
                let current = client.get_balance(&pubkey).await?;
                if current.rao() < bal.rao() {
                    anyhow::bail!("Insufficient balance: you have {} but trying to stake {}.\n  Check: agcli balance",
                        current.display_tao(), bal.display_tao());
                }
            }
            if mev {
                eprintln!("MEV shield: encrypting stake operation");
                tracing::info!("MEV shield: encrypting stake operation");
            }
            stake_op(
                output,
                "Adding",
                "added",
                &hk,
                client
                    .add_stake_mev(&pair, &hk, NetUid(netuid), bal, mev)
                    .await,
                &format!("Staked {} on SN{}", bal.display_tao(), netuid),
            )
        }
        StakeCommands::Remove {
            amount,
            netuid,
            hotkey,
            max_slippage,
        } => {
            validate_netuid(netuid)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let alpha = preflight_alpha_amount(
                client,
                &pair,
                netuid,
                &hk,
                amount,
                "unstake amount (alpha, α)",
                "unstake",
            )
            .await?;
            // Slippage check (simulates alpha → TAO)
            if let Some(max_slip) = max_slippage {
                check_slippage(client, netuid, amount, max_slip, false).await?;
            }
            if mev {
                eprintln!("MEV shield: encrypting unstake operation");
                tracing::info!("MEV shield: encrypting unstake operation");
            }
            stake_op(
                output,
                "Removing",
                "removed",
                &hk,
                client
                    .remove_stake_mev(&pair, &hk, NetUid(netuid), alpha, mev)
                    .await,
                &format!("Unstaked {:.9} α from SN{}", alpha.units(), netuid),
            )
        }
        StakeCommands::Move {
            amount,
            from,
            to,
            hotkey,
            dest_hotkey,
        } => {
            validate_netuid(from)?;
            validate_netuid(to)?;
            if from == to {
                anyhow::bail!("Source and destination subnets are the same (SN{}). Use a different --to subnet.", from);
            }
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let alpha = preflight_alpha_amount(
                client,
                &pair,
                from,
                &hk,
                amount,
                "move amount (alpha, α)",
                "move",
            )
            .await?;
            let dest_hk = match dest_hotkey {
                Some(ref d) => {
                    validate_ss58(d, "dest-hotkey")?;
                    d.clone()
                }
                None => hk.clone(),
            };
            if mev {
                eprintln!("MEV shield: encrypting move-stake operation");
                tracing::info!("MEV shield: encrypting move-stake operation");
            }
            let detail = if dest_hk == hk {
                format!("Moved {:.9} α from SN{} to SN{}", alpha.units(), from, to)
            } else {
                format!(
                    "Moved {:.9} α from SN{} to SN{} ({} → {})",
                    alpha.units(),
                    from,
                    to,
                    crate::utils::short_ss58(&hk),
                    crate::utils::short_ss58(&dest_hk)
                )
            };
            stake_op(
                output,
                "Moving",
                "moved",
                &hk,
                client
                    .move_stake_mev(&pair, &hk, &dest_hk, NetUid(from), NetUid(to), alpha, mev)
                    .await,
                &detail,
            )
        }
        StakeCommands::Swap {
            amount,
            from,
            to,
            hotkey,
        } => {
            validate_netuid(from)?;
            validate_netuid(to)?;
            if from == to {
                anyhow::bail!("Source and destination subnets are the same (SN{}). Use a different --to subnet.", from);
            }
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let alpha = preflight_alpha_amount(
                client,
                &pair,
                from,
                &hk,
                amount,
                "swap amount (alpha, α)",
                "swap",
            )
            .await?;
            if mev {
                eprintln!("MEV shield: encrypting swap-stake operation");
                tracing::info!("MEV shield: encrypting swap-stake operation");
            }
            if !output.is_json() {
                println!(
                    "Swapping stake: {:.9} α from SN{} to SN{} for {}",
                    alpha.units(),
                    from,
                    to,
                    crate::utils::short_ss58(&hk)
                );
            }
            let hash = client
                .swap_stake_mev(&pair, &hk, NetUid(from), NetUid(to), alpha, mev)
                .await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "Stake swapped. {:.9} α moved from SN{} to SN{}\n  Tx: {}",
                    alpha.units(),
                    from,
                    to,
                    hash
                ),
                Some("swapped"),
            );
            Ok(())
        }
        StakeCommands::UnstakeAll { hotkey } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            stake_op(
                output,
                "Unstaking all from",
                "unstaked",
                &hk,
                client.unstake_all(&pair, &hk).await,
                "All stake removed from this hotkey",
            )
        }
        StakeCommands::ClaimRoot { netuid } => {
            validate_netuid(netuid)?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let hash = client.claim_root(wallet.coldkey()?, &[netuid]).await?;
            emit_stake_tx(
                output,
                &hash,
                &format!("Root dividends claimed for SN{}.\n  Tx: {}", netuid, hash),
                Some("claimed"),
            );
            Ok(())
        }
        StakeCommands::AddLimit {
            amount,
            netuid,
            price,
            partial,
            hotkey,
        } => {
            validate_netuid(netuid)?;
            let lp = parse_cli_limit_price(price, "limit price")?;
            // Spending limit check
            check_spending_limit(netuid, amount)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let bal = parse_cli_tao_amount(amount, "limit stake amount")?;
            if mev {
                eprintln!("MEV shield: encrypting add-stake-limit operation");
                tracing::info!("MEV shield: encrypting add-stake-limit operation");
            }
            if !output.is_json() {
                println!(
                    "Adding stake limit: {} at {:.4} on SN{} (partial={})",
                    bal.display_tao(),
                    price,
                    netuid,
                    partial
                );
            }
            let hash = client
                .add_stake_limit_mev(&pair, &hk, NetUid(netuid), bal, lp, partial, mev)
                .await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "Limit stake order placed. {} at price {:.4} on SN{} (partial={})\n  Tx: {}",
                    bal.display_tao(),
                    price,
                    netuid,
                    partial,
                    hash
                ),
                Some("limit_added"),
            );
            Ok(())
        }
        StakeCommands::RemoveLimit {
            amount,
            netuid,
            price,
            partial,
            hotkey,
        } => {
            validate_netuid(netuid)?;
            let lp = parse_cli_limit_price(price, "limit price")?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let alpha = preflight_alpha_amount(
                client,
                &pair,
                netuid,
                &hk,
                amount,
                "limit unstake amount (alpha, α)",
                "limit unstake",
            )
            .await?;
            if mev {
                eprintln!("MEV shield: encrypting remove-stake-limit operation");
                tracing::info!("MEV shield: encrypting remove-stake-limit operation");
            }
            if !output.is_json() {
                println!(
                    "Removing stake limit: {:.9} α at {:.4} on SN{} (partial={})",
                    alpha.units(),
                    price,
                    netuid,
                    partial
                );
            }
            let hash = client
                .remove_stake_limit_mev(&pair, &hk, NetUid(netuid), alpha, lp, partial, mev)
                .await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "Limit unstake order placed. {:.9} α at price {:.4} on SN{}\n  Tx: {}",
                    alpha.units(),
                    price,
                    netuid,
                    hash
                ),
                Some("limit_removed"),
            );
            Ok(())
        }
        StakeCommands::ChildkeyTake {
            take,
            netuid,
            hotkey,
        } => {
            validate_netuid(netuid)?;
            validate_take_pct(take)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let take_u16 = (take / 100.0 * 65535.0).round().min(65535.0) as u16;
            if !output.is_json() {
                println!(
                    "Setting childkey take to {:.2}% on SN{} for {} (on-chain u16={})",
                    take,
                    netuid,
                    crate::utils::short_ss58(&hk),
                    take_u16
                );
            } else {
                tracing::info!(
                    take_pct = take,
                    take_u16,
                    netuid,
                    "childkey take encoding u16÷65535"
                );
            }
            let hash = client
                .set_childkey_take(&pair, &hk, NetUid(netuid), take_u16)
                .await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "Childkey take set to {:.2}% on SN{}.\n  Tx: {}",
                    take, netuid, hash
                ),
                Some("childkey_take_set"),
            );
            Ok(())
        }
        StakeCommands::SetChildren {
            netuid,
            children,
            hotkey,
        } => {
            validate_netuid(netuid)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let children_parsed = parse_children(&children)?;
            let total_prop: u128 = children_parsed.iter().map(|(p, _)| *p as u128).sum();
            if !output.is_json() {
                println!(
                    "Setting {} children on SN{} for {} (total proportion raw sum={} / u64::MAX)",
                    children_parsed.len(),
                    netuid,
                    crate::utils::short_ss58(&hk),
                    total_prop
                );
                eprintln!(
                    "Note: proportions are u64 on-chain; runtime uses value÷u64::MAX. \
                     Decimal inputs like 0.5 are converted automatically."
                );
            }
            let hash = client
                .set_children(&pair, &hk, NetUid(netuid), &children_parsed)
                .await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "{} children set on SN{}.\n  Tx: {}",
                    children_parsed.len(),
                    netuid,
                    hash
                ),
                Some("children_set"),
            );
            Ok(())
        }
        StakeCommands::RecycleAlpha {
            amount,
            netuid,
            hotkey,
        } => {
            validate_netuid(netuid)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let alpha = preflight_alpha_amount(
                client,
                &pair,
                netuid,
                &hk,
                amount,
                "recycle alpha amount (alpha, α)",
                "recycle",
            )
            .await?;
            if mev {
                eprintln!("MEV shield: encrypting recycle-alpha operation");
                tracing::info!("MEV shield: encrypting recycle-alpha operation");
            }
            stake_op(
                output,
                "Recycling alpha via",
                "recycled",
                &hk,
                client
                    .recycle_alpha_mev(&pair, &hk, NetUid(netuid), alpha, mev)
                    .await,
                &format!("Recycled {:.9} α on SN{}", alpha.units(), netuid),
            )
        }
        StakeCommands::UnstakeAllAlpha { hotkey } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            stake_op(
                output,
                "Unstaking all alpha from",
                "unstaked",
                &hk,
                client.unstake_all_alpha(&pair, &hk).await,
                "All alpha unstaked from this hotkey",
            )
        }
        StakeCommands::BurnAlpha {
            amount,
            netuid,
            hotkey,
        } => {
            validate_netuid(netuid)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let alpha = preflight_alpha_amount(
                client,
                &pair,
                netuid,
                &hk,
                amount,
                "burn alpha amount (alpha, α)",
                "burn",
            )
            .await?;
            if mev {
                eprintln!("MEV shield: encrypting burn-alpha operation");
                tracing::info!("MEV shield: encrypting burn-alpha operation");
            }
            stake_op(
                output,
                "Burning alpha via",
                "burned",
                &hk,
                client
                    .burn_alpha_mev(&pair, &hk, alpha, NetUid(netuid), mev)
                    .await,
                &format!(
                    "Burned {:.9} α on SN{} (permanently destroyed)",
                    alpha.units(),
                    netuid
                ),
            )
        }
        StakeCommands::SwapLimit {
            amount,
            from,
            to,
            price,
            partial,
            hotkey,
        } => {
            validate_netuid(from)?;
            validate_netuid(to)?;
            let lp = parse_cli_limit_price(price, "swap-limit price")?;
            if from == to {
                anyhow::bail!("Source and destination subnets are the same (SN{}). Use a different --to subnet.", from);
            }
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let alpha = preflight_alpha_amount(
                client,
                &pair,
                from,
                &hk,
                amount,
                "swap-limit amount (alpha, α)",
                "swap-limit",
            )
            .await?;
            if mev {
                eprintln!("MEV shield: encrypting swap-stake-limit operation");
                tracing::info!("MEV shield: encrypting swap-stake-limit operation");
            }
            if !output.is_json() {
                println!(
                    "Swap-limit {:.9} α from SN{} to SN{} at price {:.4} (partial={})",
                    alpha.units(),
                    from,
                    to,
                    price,
                    partial
                );
            }
            let hash = client
                .swap_stake_limit_mev(
                    &pair,
                    &hk,
                    NetUid(from),
                    NetUid(to),
                    alpha,
                    lp,
                    partial,
                    mev,
                )
                .await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "Swap limit submitted. {:.9} α from SN{} to SN{} at price {:.4}\n  Tx: {}",
                    alpha.units(),
                    from,
                    to,
                    price,
                    hash
                ),
                Some("swap_limit_submitted"),
            );
            Ok(())
        }
        StakeCommands::SetAuto { netuid, hotkey } => {
            validate_netuid(netuid)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            if !output.is_json() {
                println!(
                    "Setting auto-stake on SN{} to hotkey {}...",
                    netuid,
                    crate::utils::short_ss58(&hk)
                );
            }
            let hash = client.set_auto_stake(&pair, NetUid(netuid), &hk).await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "Auto-stake configured. SN{} emissions will auto-stake to {}\n  Tx: {}",
                    netuid,
                    crate::utils::short_ss58(&hk),
                    hash
                ),
                Some("auto_stake_configured"),
            );
            Ok(())
        }
        StakeCommands::ShowAuto { address } => {
            let addr = resolve_and_validate_coldkey_address(
                address,
                wallet_dir,
                wallet_name,
                "stake show-auto --address",
            )?;
            let subnets = client.get_all_subnets().await?;
            let addr_ref = &addr;
            let futures: Vec<_> = subnets
                .iter()
                .map(|subnet| {
                    let netuid = subnet.netuid;
                    async move { (netuid, client.get_auto_stake_hotkey(addr_ref, netuid).await) }
                })
                .collect();
            let results = futures::future::join_all(futures).await;
            let mut destinations: Vec<(u16, String)> = Vec::new();
            for (netuid, result) in &results {
                if let Ok(Some(hotkey)) = result {
                    destinations.push((netuid.0, hotkey.clone()));
                }
            }
            if output.is_json() {
                print_json(&serde_json::json!({
                    "address": addr,
                    "auto_stake": destinations.iter().map(|(n, h)| serde_json::json!({
                        "netuid": n,
                        "hotkey": h,
                    })).collect::<Vec<_>>(),
                }));
            } else if destinations.is_empty() {
                println!(
                    "No auto-stake destinations set for {}",
                    crate::utils::short_ss58(&addr)
                );
            } else {
                println!(
                    "Auto-stake destinations for {}:",
                    crate::utils::short_ss58(&addr)
                );
                for (netuid, hotkey) in &destinations {
                    println!("  SN{:<4} → {}", netuid, crate::utils::short_ss58(hotkey));
                }
            }
            Ok(())
        }
        StakeCommands::SetClaim {
            claim_type,
            subnets,
        } => {
            let (pair, _) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            let subnet_ids: Option<Vec<u16>> = subnets.as_ref().map(|s| {
                let mut ids = Vec::new();
                for n in s.split(',') {
                    let trimmed = n.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match trimmed.parse::<u16>() {
                        Ok(id) => ids.push(id),
                        Err(_) => {
                            eprintln!("Warning: ignoring invalid subnet ID '{}'", trimmed);
                            tracing::warn!(input = trimmed, "Ignoring invalid subnet ID");
                        }
                    }
                }
                ids
            });
            let keep_subnets = subnet_ids.as_deref();
            if claim_type == "keep-subnets" {
                match keep_subnets {
                    None | Some([]) => {
                        anyhow::bail!(
                            "`--claim-type keep-subnets` requires at least one netuid in `--subnets`.\n  \
                             Example: agcli stake set-claim --claim-type keep-subnets --subnets \"1,2\"\n  \
                             On-chain: empty `KeepSubnets {{ subnets }}` returns InvalidSubnetNumber."
                        );
                    }
                    Some(ids) => {
                        for id in ids {
                            validate_netuid(*id)?;
                        }
                    }
                }
            }
            if !output.is_json() {
                println!(
                    "Setting root claim type to '{}'{}...",
                    claim_type,
                    keep_subnets
                        .map(|s| format!(" (subnets: {:?})", s))
                        .unwrap_or_default()
                );
            }
            let hash = client
                .set_root_claim_type(&pair, &claim_type, keep_subnets)
                .await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "Root claim type set to '{}'{}.\n  Tx: {}",
                    claim_type,
                    keep_subnets
                        .map(|s| format!(" for subnets {:?}", s))
                        .unwrap_or_default(),
                    hash
                ),
                Some("claim_type_set"),
            );
            Ok(())
        }
        StakeCommands::TransferStake {
            dest,
            amount,
            from,
            to,
            hotkey,
        } => {
            validate_netuid(from)?;
            validate_netuid(to)?;
            validate_ss58(&dest, "destination")?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let alpha = preflight_alpha_amount(
                client,
                &pair,
                from,
                &hk,
                amount,
                "transfer stake amount (alpha, α)",
                "transfer",
            )
            .await?;
            if mev {
                eprintln!("MEV shield: encrypting transfer-stake operation");
                tracing::info!("MEV shield: encrypting transfer-stake operation");
            }
            if !output.is_json() {
                println!(
                    "Transferring {:.9} α stake from SN{} to SN{} → {}",
                    alpha.units(),
                    from,
                    to,
                    crate::utils::short_ss58(&dest)
                );
            }
            let hash = client
                .transfer_stake_mev(&pair, &dest, &hk, NetUid(from), NetUid(to), alpha, mev)
                .await?;
            emit_stake_tx(
                output,
                &hash,
                &format!(
                    "Stake transferred. {:.9} α from SN{} to SN{}, destination: {}\n  Tx: {}",
                    alpha.units(),
                    from,
                    to,
                    crate::utils::short_ss58(&dest),
                    hash
                ),
                Some("transferred"),
            );
            Ok(())
        }
        StakeCommands::ProcessClaim { hotkey, netuids } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            // Parse optional netuid filter
            // (audit fix: warn on invalid IDs instead of silently dropping, matching SetClaim pattern)
            let filter_netuids: Option<Vec<u16>> = netuids.as_ref().map(|s| {
                let mut ids = Vec::new();
                for n in s.split(',') {
                    let trimmed = n.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match trimmed.parse::<u16>() {
                        Ok(id) => ids.push(id),
                        Err(_) => {
                            eprintln!("Warning: ignoring invalid subnet ID '{}'", trimmed);
                            tracing::warn!(
                                input = trimmed,
                                "Ignoring invalid subnet ID in process-claim"
                            );
                        }
                    }
                }
                ids
            });

            // Fetch stakes to find all netuids where this hotkey has root emissions
            let coldkey_ss58 = resolve_and_validate_coldkey_address(
                None,
                wallet_dir,
                wallet_name,
                "stake process-claim",
            )?;
            let stakes = client.get_stake_for_coldkey(&coldkey_ss58).await?;

            // Filter to netuids where we have stake on this hotkey
            let target_netuids: Vec<u16> = stakes
                .iter()
                .filter(|s| s.hotkey == hk)
                .filter(|s| {
                    filter_netuids
                        .as_ref()
                        .map(|f| f.contains(&s.netuid.0))
                        .unwrap_or(true)
                })
                .map(|s| s.netuid.0)
                .collect();

            if target_netuids.is_empty() {
                if output.is_json() {
                    print_json(&serde_json::json!({
                        "hotkey": hk,
                        "claimed": [],
                        "failed": [],
                        "success_count": 0,
                        "failed_count": 0,
                    }));
                } else {
                    println!(
                        "No stakes found for hotkey {} to claim from.",
                        crate::utils::short_ss58(&hk)
                    );
                }
                return Ok(());
            }

            if !output.is_json() {
                println!(
                    "Processing root claims for hotkey {} across {} subnet(s): {:?}",
                    crate::utils::short_ss58(&hk),
                    target_netuids.len(),
                    target_netuids
                );
            }

            // Submit claims in parallel for all subnets
            let hk_account = Client::ss58_to_account_id_pub(&hk)?;
            let claim_futures: Vec<_> = target_netuids
                .iter()
                .map(|&nuid| {
                    let hk_bytes = hk_account.0;
                    let pair_clone = pair.clone();
                    async move {
                        let result = client
                            .submit_raw_call(
                                &pair_clone,
                                "SubtensorModule",
                                "claim_root_dividends",
                                vec![
                                    subxt::dynamic::Value::from_bytes(hk_bytes),
                                    subxt::dynamic::Value::u128(nuid as u128),
                                ],
                            )
                            .await;
                        (nuid, result)
                    }
                })
                .collect();
            let results = futures::future::join_all(claim_futures).await;
            let mut claimed: Vec<(u16, String)> = Vec::new();
            let mut failures: Vec<(u16, String)> = Vec::new();
            for (nuid, result) in results {
                match result {
                    Ok(hash) => {
                        if !output.is_json() {
                            println!("  SN{}: claimed (tx: {})", nuid, hash);
                        }
                        claimed.push((nuid, hash));
                    }
                    Err(e) => {
                        let msg = format!("{:#}", e);
                        if !output.is_json() {
                            eprintln!("  SN{}: failed — {}", nuid, msg);
                        }
                        tracing::error!(netuid = nuid, error = %e, "claim_root_dividends failed");
                        failures.push((nuid, msg));
                    }
                }
            }
            if output.is_json() {
                print_json(&serde_json::json!({
                    "hotkey": hk,
                    "claimed": claimed.iter().map(|(n, h)| serde_json::json!({
                        "netuid": n,
                        "tx_hash": h,
                    })).collect::<Vec<_>>(),
                    "failed": failures.iter().map(|(n, e)| serde_json::json!({
                        "netuid": n,
                        "error": e,
                    })).collect::<Vec<_>>(),
                    "success_count": claimed.len(),
                    "failed_count": failures.len(),
                }));
            } else {
                println!(
                    "\nDone: {} claimed, {} failed out of {} total",
                    claimed.len(),
                    failures.len(),
                    target_netuids.len()
                );
            }
            if !failures.is_empty() {
                let failed_list: Vec<String> =
                    failures.iter().map(|(n, _)| n.to_string()).collect();
                anyhow::bail!(
                    "Root claim failed for {}/{} subnet(s): {}.\n  \
                     Per-subnet errors are printed above.\n  \
                     Tip: retry one subnet with `agcli stake claim-root --netuid <N>` or check hotkey stake on failed SNs.",
                    failures.len(),
                    target_netuids.len(),
                    failed_list.join(", ")
                );
            }
            Ok(())
        }
        StakeCommands::RemoveFullLimit {
            netuid,
            price,
            hotkey,
        } => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey, password)?;
            let limit_price = parse_cli_limit_price(price, "limit price")?;
            println!(
                "Removing all stake from SN{} hotkey {} (limit price {} TAO/α)",
                netuid,
                crate::utils::short_ss58(&hk),
                price
            );
            let hash = client
                .remove_stake_full_limit(&pair, &hk, NetUid(netuid), limit_price)
                .await?;
            print_tx_result(
                output,
                &hash,
                &format!("Full unstake from SN{} (limit {})", netuid, price),
            );
            Ok(())
        }
        StakeCommands::Wizard {
            netuid,
            amount,
            hotkey,
        } => staking_wizard(client, ctx, netuid, amount, hotkey).await,
    }
}

/// Print write-command success: JSON `{"tx_hash": "...", "action": "..."}` or human text.
fn emit_stake_tx(output: OutputFormat, hash: &str, human: &str, action: Option<&str>) {
    if output.is_json() {
        let mut payload = serde_json::json!({"tx_hash": hash});
        if let Some(a) = action {
            payload["action"] = serde_json::json!(a);
        }
        print_json(&payload);
    } else {
        println!("{}", human);
    }
}

/// Common pattern for stake operations: print action, handle result with context.
fn stake_op(
    output: OutputFormat,
    action: &str,
    past: &str,
    hotkey: &str,
    result: Result<String>,
    detail: &str,
) -> Result<()> {
    if !output.is_json() {
        println!("{} {}", action, crate::utils::short_ss58(hotkey));
    }
    let hash = result?;
    let human = if detail.is_empty() {
        format!("Stake {}. Tx: {}", past, hash)
    } else {
        format!("Stake {}. {}\n  Tx: {}", past, detail, hash)
    };
    emit_stake_tx(output, &hash, &human, Some(past));
    Ok(())
}

/// Check AMM slippage before a stake/unstake operation. Aborts if slippage exceeds max.
async fn check_slippage(
    client: &Client,
    netuid: u16,
    amount: f64,
    max_slip_pct: f64,
    is_buy: bool,
) -> Result<()> {
    let nuid = NetUid(netuid);
    let rao = safe_rao(amount);
    let slippage = if is_buy {
        // Staking: TAO → Alpha — parallel fetch price + simulation
        let (price_raw, (out, _tf, _af)) = tokio::try_join!(
            client.current_alpha_price(nuid),
            client.sim_swap_tao_for_alpha(nuid, rao),
        )?;
        let price = price_raw as f64 / 1e9;
        let out_f = out as f64 / 1e9;
        let eff_price = if out_f > 0.0 { amount / out_f } else { 0.0 };
        if price > 0.0 {
            ((eff_price - price) / price).abs() * 100.0
        } else {
            0.0
        }
    } else {
        // Unstaking: Alpha → TAO — parallel fetch price + simulation
        let (price_raw, (out, _tf, _af)) = tokio::try_join!(
            client.current_alpha_price(nuid),
            client.sim_swap_alpha_for_tao(nuid, rao),
        )?;
        let price = price_raw as f64 / 1e9;
        let out_f = out as f64 / 1e9;
        let eff_price = if amount > 0.0 { out_f / amount } else { 0.0 };
        if price > 0.0 {
            ((eff_price - price) / price).abs() * 100.0
        } else {
            0.0
        }
    };
    if slippage > max_slip_pct {
        anyhow::bail!(
            "Slippage {:.2}% exceeds maximum allowed {:.2}% on SN{}.\n  Reduce trade size or use a limit order: agcli stake add-limit / remove-limit",
            slippage, max_slip_pct, netuid
        );
    }
    if slippage > 2.0 {
        eprintln!(
            "Warning: estimated slippage is {:.2}% on SN{}",
            slippage, netuid
        );
        tracing::warn!(
            slippage_pct = slippage,
            netuid = netuid,
            "Estimated slippage exceeds 2%"
        );
    }
    Ok(())
}

async fn staking_wizard(
    client: &Client,
    ctx: &Ctx<'_>,
    netuid_arg: Option<u16>,
    amount_arg: Option<f64>,
    hotkey_arg: Option<String>,
) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name, password, output) = (
        ctx.wallet_dir,
        ctx.wallet_name,
        ctx.hotkey_name,
        ctx.password,
        ctx.output,
    );
    if !output.is_json() {
        println!("=== Staking Wizard ===\n");
    }

    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    let coldkey_ss58 = match wallet.coldkey_ss58() {
        Some(s) => s.to_string(),
        None => {
            // Public key not on disk; unlock to derive it
            unlock_coldkey(&mut wallet, password)?;
            wallet
                .coldkey_ss58()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("Could not resolve coldkey address"))?
        }
    };
    println!(
        "Wallet: {} ({})",
        wallet_name,
        crate::utils::short_ss58(&coldkey_ss58)
    );

    let (balance, dynamic) = tokio::try_join!(
        client.get_balance_ss58(&coldkey_ss58),
        client.get_all_dynamic_info(),
    )?;
    println!("Balance: {}\n", balance.display_tao());

    if balance.rao() == 0 {
        println!("You need TAO to stake. Transfer some TAO to your coldkey first.");
        return Ok(());
    }
    let mut subnets_with_pool: Vec<_> = dynamic.iter().filter(|d| d.tao_in.rao() > 0).collect();
    subnets_with_pool.sort_by_key(|item| std::cmp::Reverse(item.tao_in.rao()));

    println!("\nTop subnets by TAO pool:");
    let display_count = subnets_with_pool.len().min(15);
    for (i, d) in subnets_with_pool.iter().take(display_count).enumerate() {
        println!(
            "  {:>2}. SN{:<3} {:<20} price={:.6} τ/α  pool={:.2} τ",
            i + 1,
            d.netuid,
            &d.name,
            d.price,
            d.tao_in.tao(),
        );
    }

    // Resolve netuid: from CLI flag or interactive prompt
    let netuid: u16 = match netuid_arg {
        Some(n) => n,
        None => {
            require_tty_for_input("--netuid")?;
            let netuid_input: String = dialoguer::Input::new()
                .with_prompt("\nEnter subnet netuid to stake on")
                .interact_text()?;
            netuid_input
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid netuid"))?
        }
    };

    // Resolve amount: from CLI flag or interactive prompt
    let max_tao = balance.tao();
    let amount: f64 = match amount_arg {
        Some(a) => a,
        None => {
            require_tty_for_input("--amount")?;
            let amount_input: String = dialoguer::Input::new()
                .with_prompt(format!("Amount of TAO to stake (max {:.4})", max_tao))
                .interact_text()?;
            amount_input
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid amount"))?
        }
    };

    if amount <= 0.0 {
        anyhow::bail!("Amount must be greater than 0.");
    }
    if amount > max_tao {
        anyhow::bail!("Amount {:.4} τ exceeds your balance of {:.4} τ. Use a smaller amount or transfer more TAO first.", amount, max_tao);
    }

    // Spending limit check
    check_spending_limit(netuid, amount)?;

    // Re-fetch fresh data after interactive prompts to guard against stale prices.
    // The user may have spent >30s on the prompts, making cached data unreliable.
    client.invalidate_cache().await;
    if let Ok(fresh_dynamic) = client.get_all_dynamic_info().await {
        if let Some(fresh_d) = fresh_dynamic.iter().find(|d| d.netuid.0 == netuid) {
            // Find the original cached price we displayed to the user
            if let Some(orig_d) = subnets_with_pool.iter().find(|d| d.netuid.0 == netuid) {
                let orig_price = orig_d.price;
                let fresh_price = fresh_d.price;
                if orig_price > 0.0 {
                    let pct_change = ((fresh_price - orig_price) / orig_price).abs() * 100.0;
                    if pct_change > 5.0 {
                        eprintln!(
                            "⚠ Price for SN{} changed {:.1}% since displayed ({:.6} → {:.6} τ/α)",
                            netuid, pct_change, orig_price, fresh_price
                        );
                    }
                }
            }
        }
    }

    let hotkey_ss58 = resolve_hotkey_ss58(hotkey_arg, &mut wallet, hotkey_name)?;
    println!(
        "\nStaking {:.4} τ on SN{} with hotkey {}",
        amount,
        netuid,
        crate::utils::short_ss58(&hotkey_ss58)
    );

    // Confirm: skip if --yes or --batch, otherwise prompt
    if !is_yes_mode() {
        require_confirm_prompt_capability()?;
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Proceed?")
            .default(true)
            .interact()?;

        if !confirm {
            println!("Cancelled.");
            return Ok(());
        }
    }

    let mev = ctx.mev;
    if mev {
        eprintln!("MEV shield: encrypting stake operation");
        tracing::info!("MEV shield: encrypting stake operation");
    }
    unlock_coldkey(&mut wallet, password)?;
    let stake_balance = parse_cli_tao_amount(amount, "stake amount")?;
    let hash = client
        .add_stake_mev(
            wallet.coldkey()?,
            &hotkey_ss58,
            NetUid(netuid),
            stake_balance,
            mev,
        )
        .await?;
    emit_stake_tx(
        output,
        &hash,
        &format!("Stake added! Tx: {}", hash),
        Some("added"),
    );

    if !output.is_json() {
        println!("\nUpdated portfolio:");
        let portfolio = crate::queries::portfolio::fetch_portfolio(client, &coldkey_ss58).await?;
        println!("  Free:   {}", portfolio.free_balance.display_tao());
        println!("  Staked: {}", portfolio.total_staked.display_tao());
    }

    Ok(())
}

/// Validate amount, parse alpha, and preflight stake balance on a subnet/hotkey.
async fn preflight_alpha_amount(
    client: &Client,
    pair: &sp_core::sr25519::Pair,
    netuid: u16,
    hotkey_ss58: &str,
    amount: f64,
    amount_label: &str,
    action: &str,
) -> Result<AlphaBalance> {
    let alpha = parse_cli_alpha_amount(amount, amount_label)?;
    let coldkey_ss58 = crate::wallet::keypair::to_ss58(&sp_core::Pair::public(pair), 42);
    preflight_alpha_stake(client, &coldkey_ss58, netuid, hotkey_ss58, alpha, action).await?;
    Ok(alpha)
}

/// Client-side alpha balance check before alpha-denominated extrinsics.
async fn preflight_alpha_stake(
    client: &Client,
    coldkey_ss58: &str,
    netuid: u16,
    hotkey_ss58: &str,
    alpha: AlphaBalance,
    action: &str,
) -> Result<()> {
    if alpha.raw() == 0 {
        return Ok(());
    }
    let stakes = client
        .get_stake_for_coldkey(coldkey_ss58)
        .await
        .with_context(|| format!("Failed to load stakes for {action} preflight"))?;
    match stakes
        .iter()
        .find(|s| s.netuid.0 == netuid && s.hotkey == hotkey_ss58)
    {
        Some(pos) if alpha.raw() > pos.stake.raw() => {
            anyhow::bail!(
                "Cannot {action} {:.9} α — you have {:.9} α on SN{} for hotkey {}.\n  \
                 `--amount` is alpha (α), not TAO. Check: agcli stake list",
                alpha.units(),
                pos.stake.units(),
                netuid,
                crate::utils::short_ss58(hotkey_ss58)
            );
        }
        None => {
            anyhow::bail!(
                "No alpha stake on SN{} for hotkey {} ({action}).\n  Check: agcli stake list",
                netuid,
                crate::utils::short_ss58(hotkey_ss58)
            );
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    /// Structural: `handle_stake` is async and requires a Client, so we cannot
    /// exercise it without a live chain. These tests confirm the module compiles.
    #[test]
    fn module_compiles() {
        // If this test compiles, the stake_cmds module is structurally valid.
    }

    #[test]
    fn handle_stake_symbol_exists() {
        let _ = super::handle_stake as fn(_, _, _) -> _;
    }

    /// Structural: staking_wizard function exists and compiles with cache invalidation logic.
    /// The wizard invalidates cache after interactive prompts to prevent stale price display (Issue 647).
    #[test]
    fn staking_wizard_compiles_with_cache_invalidation() {
        // staking_wizard is private (async fn) — verify it exists in this module.
        // The actual cache invalidation is tested in query_cache::tests::invalidate_forces_fresh_dynamic_fetch.
        // This test just confirms the module still compiles after the change.
    }

    // ── Audit fix: process_claim netuid parsing must warn on invalid IDs ──

    /// Verify the fixed netuid parsing pattern warns instead of silently dropping invalid entries.
    /// Matches the SetClaim pattern (explicit match with warning).
    #[test]
    fn process_claim_netuid_parse_pattern_rejects_invalid() {
        let input = "1,2,invalid,4";
        let mut ids = Vec::new();
        let mut warnings = Vec::new();
        for n in input.split(',') {
            let trimmed = n.trim();
            if trimmed.is_empty() {
                continue;
            }
            match trimmed.parse::<u16>() {
                Ok(id) => ids.push(id),
                Err(_) => {
                    warnings.push(format!("ignoring invalid subnet ID '{}'", trimmed));
                }
            }
        }
        assert_eq!(ids, vec![1, 2, 4], "valid IDs should be collected");
        assert_eq!(warnings.len(), 1, "should produce warning for 'invalid'");
        assert!(
            warnings[0].contains("invalid"),
            "warning should mention the bad input"
        );
    }

    #[test]
    fn process_claim_netuid_parse_handles_empty_entries() {
        let input = "1,,3,";
        let mut ids = Vec::new();
        for n in input.split(',') {
            let trimmed = n.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(id) = trimmed.parse::<u16>() {
                ids.push(id);
            }
        }
        assert_eq!(
            ids,
            vec![1, 3],
            "empty entries should be skipped without error"
        );
    }

    // ── Issue 130: childkey take should round, not truncate ──

    #[test]
    fn childkey_take_rounds_correctly() {
        // 0.01% = 0.0001 * 65535 = 6.5535, should round to 7, not truncate to 6
        let take: f64 = 0.01;
        let take_u16 = (take / 100.0 * 65535.0).round().min(65535.0) as u16;
        assert_eq!(take_u16, 7, "0.01% should round to 7, not truncate to 6");
    }

    #[test]
    fn childkey_take_18_percent() {
        let take: f64 = 18.0;
        let take_u16_round = (take / 100.0 * 65535.0).round().min(65535.0) as u16;
        let take_u16_trunc = (take / 100.0 * 65535.0).min(65535.0) as u16;
        // 18/100*65535 = 11796.3 — both round and truncate give 11796
        assert_eq!(take_u16_round, 11796);
        assert_eq!(take_u16_trunc, 11796);
    }

    #[test]
    fn childkey_take_100_percent() {
        let take: f64 = 100.0;
        let take_u16 = (take / 100.0 * 65535.0).round().min(65535.0) as u16;
        assert_eq!(take_u16, 65535, "100% should map to u16::MAX");
    }
}
