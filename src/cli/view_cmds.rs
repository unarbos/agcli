//! View command handlers (portfolio, network, dynamic, neuron, validators, history, account, analytics).

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::{OutputFormat, ViewCommands};
use crate::types::chain_data::DelegateInfo;
use crate::types::{AlphaBalance, Balance, NetUid};
use anyhow::Result;

pub async fn handle_view(cmd: ViewCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name) = (ctx.wallet_dir, ctx.wallet_name);
    let (output, live_interval) = (ctx.output, ctx.live_interval);
    match cmd {
        ViewCommands::Portfolio { address, at_block } => {
            let addr = resolve_and_validate_coldkey_address(
                address,
                wallet_dir,
                wallet_name,
                "portfolio --address",
            )?;
            if let Some(bn) = at_block {
                return handle_portfolio_at_block(client, &addr, output, bn).await;
            }
            if let Some(interval) = live_interval {
                return crate::live::live_portfolio(client, &addr, interval).await;
            }
            handle_portfolio(client, &addr, output).await
        }
        ViewCommands::Network { at_block } => handle_network(client, output, at_block).await,
        ViewCommands::Dynamic { at_block } => {
            if let Some(bn) = at_block {
                return handle_dynamic_at_block(client, output, bn).await;
            }
            if let Some(interval) = live_interval {
                return crate::live::live_dynamic(client, interval).await;
            }
            handle_dynamic(client, output).await
        }
        ViewCommands::Neuron {
            netuid,
            uid,
            at_block,
        } => {
            validate_netuid(netuid)?;
            handle_neuron(client, output, netuid, uid, at_block).await
        }
        ViewCommands::Validators {
            netuid,
            limit,
            at_block,
        } => {
            if let Some(n) = netuid {
                validate_netuid(n)?;
            }
            validate_view_limit(limit, "validators --limit")?;
            handle_validators(client, output, netuid, limit, at_block).await
        }
        ViewCommands::History { address, limit } => {
            validate_view_limit(limit, "history --limit")?;
            let addr = resolve_and_validate_coldkey_address(
                address,
                wallet_dir,
                wallet_name,
                "history --address",
            )?;
            handle_history(&addr, output, limit).await
        }
        ViewCommands::Account { address, at_block } => {
            let addr = resolve_and_validate_coldkey_address(
                address,
                wallet_dir,
                wallet_name,
                "account --address",
            )?;
            handle_account_explorer(client, &addr, output, at_block).await
        }
        ViewCommands::SubnetAnalytics { netuid } => {
            validate_netuid(netuid)?;
            handle_subnet_analytics(client, netuid, output).await
        }
        ViewCommands::StakingAnalytics { address } => {
            let addr = resolve_and_validate_coldkey_address(
                address,
                wallet_dir,
                wallet_name,
                "staking-analytics --address",
            )?;
            handle_staking_analytics(client, &addr, output).await
        }
        ViewCommands::SwapSim { netuid, tao, alpha } => {
            validate_netuid(netuid)?;
            if let Some(t) = tao {
                validate_amount(t, "swap --tao")?;
            }
            if let Some(a) = alpha {
                validate_amount(a, "swap --alpha")?;
            }
            if tao.is_none() && alpha.is_none() {
                anyhow::bail!("Specify either --tao or --alpha for swap simulation.\n  Tip: use --tao 1.0 to simulate swapping 1 TAO to alpha.");
            }
            handle_swap_sim(client, netuid, tao, alpha, output).await
        }
        ViewCommands::Nominations { hotkey } => {
            validate_ss58(&hotkey, "nominations --hotkey-address")?;
            handle_nominations(client, &hotkey, output).await
        }
        ViewCommands::Metagraph {
            netuid,
            since_block,
            limit,
        } => {
            validate_netuid(netuid)?;
            if let Some(lim) = limit {
                validate_view_limit(lim, "metagraph --limit")?;
            }
            if let Some(interval) = live_interval {
                return crate::live::live_metagraph(client, NetUid(netuid), interval).await;
            }
            handle_metagraph_view(client, NetUid(netuid), since_block, limit, output).await
        }
        ViewCommands::Axon {
            netuid,
            uid,
            hotkey,
        } => {
            validate_netuid(netuid)?;
            if let Some(ref hk) = hotkey {
                validate_ss58(hk, "axon --hotkey-address")?;
            }
            handle_axon_lookup(client, NetUid(netuid), uid, hotkey.as_deref(), output).await
        }
        ViewCommands::Health {
            netuid,
            tcp_check,
            probe_timeout_ms,
        } => {
            validate_netuid(netuid)?;
            handle_subnet_health(client, NetUid(netuid), tcp_check, probe_timeout_ms, output).await
        }
        ViewCommands::Emissions { netuid, limit } => {
            validate_netuid(netuid)?;
            if let Some(lim) = limit {
                validate_view_limit(lim, "emissions --limit")?;
            }
            handle_emissions(client, NetUid(netuid), limit, output).await
        }
    }
}

async fn handle_portfolio(client: &Client, addr: &str, output: OutputFormat) -> Result<()> {
    let portfolio = crate::queries::portfolio::fetch_portfolio(client, addr).await?;
    if output.is_json() {
        print_json_ser(&portfolio);
    } else {
        if !output.is_csv() {
            println!("Portfolio for {}", crate::utils::short_ss58(addr));
            println!("  Free:   {}", portfolio.free_balance.display_tao());
            println!(
                "  Staked: {} (τ-equiv)",
                portfolio.total_staked.display_tao()
            );
            println!(
                "  Total:  {}",
                (portfolio.free_balance + portfolio.total_staked).display_tao()
            );
        }
        if !portfolio.positions.is_empty() {
            render_rows(
                output,
                &portfolio.positions,
                "netuid,subnet_name,hotkey,alpha_stake,tao_equiv_rao,price",
                |p| {
                    format!(
                        "{},{},{},{},{},{:.6}",
                        p.netuid,
                        csv_escape(&p.subnet_name),
                        p.hotkey_ss58,
                        p.alpha_stake,
                        p.tao_equivalent.rao(),
                        p.price
                    )
                },
                &[
                    "Subnet",
                    "Name",
                    "Hotkey",
                    "Alpha (α)",
                    "TAO≈ (τ)",
                    "Price (τ/α)",
                ],
                |p| {
                    vec![
                        format!("SN{}", p.netuid),
                        p.subnet_name.clone(),
                        crate::utils::short_ss58(&p.hotkey_ss58),
                        format!("{:.9} α", p.alpha_stake as f64 / 1e9),
                        p.tao_equivalent.display_tao(),
                        format!("{:.6}", p.price),
                    ]
                },
                None,
            );
        }
    }
    Ok(())
}

async fn handle_network(
    client: &Client,
    output: OutputFormat,
    at_block: Option<u32>,
) -> Result<()> {
    // Historical wayback mode
    if let Some(block_num) = at_block {
        let block_hash = client.get_block_hash(block_num).await?;
        let (total_stake, total_issuance) =
            tokio::try_join!(client.get_total_stake_at_block(block_hash), async {
                // Total issuance at block
                let addr = crate::api::storage().balances().total_issuance();
                let val = client.subxt().storage().at(block_hash).fetch(&addr).await?;
                Ok::<_, anyhow::Error>(Balance::from_rao(val.unwrap_or(0) as u64))
            },)?;
        let staking_ratio = if total_issuance.rao() > 0 {
            total_stake.tao() / total_issuance.tao() * 100.0
        } else {
            0.0
        };
        if output.is_json() {
            print_json(&serde_json::json!({
                "block": block_num,
                "block_hash": format!("{:?}", block_hash),
                "total_issuance_rao": total_issuance.rao(),
                "total_issuance_tao": total_issuance.tao(),
                "total_stake_rao": total_stake.rao(),
                "total_stake_tao": total_stake.tao(),
                "staking_ratio_pct": staking_ratio,
            }));
        } else {
            println!("Network Overview (at block {})", block_num);
            println!("  Block hash:   {:?}", block_hash);
            println!("  Total issued: {}", total_issuance.display_tao());
            println!("  Total staked: {}", total_stake.display_tao());
            println!("  Staking ratio: {:.1}%", staking_ratio);
        }
        return Ok(());
    }

    // Single pinned block: saves 4 redundant at_latest() RPC round-trips
    let (block, total_stake, total_networks, total_issuance, emission) =
        client.get_network_overview().await?;
    let staking_ratio = if total_issuance.rao() > 0 {
        total_stake.tao() / total_issuance.tao() * 100.0
    } else {
        0.0
    };
    if output.is_json() {
        print_json(&serde_json::json!({
            "block": block,
            "subnets": total_networks,
            "total_issuance_rao": total_issuance.rao(),
            "total_issuance_tao": total_issuance.tao(),
            "total_stake_rao": total_stake.rao(),
            "total_stake_tao": total_stake.tao(),
            "emission_per_block_rao": emission.rao(),
            "staking_ratio_pct": staking_ratio,
        }));
    } else {
        println!("Network Overview");
        println!("  Block:        {}", block);
        println!("  Subnets:      {}", total_networks);
        println!("  Total issued: {}", total_issuance.display_tao());
        println!("  Total staked: {}", total_stake.display_tao());
        println!("  Emission/blk: {}", emission.display_tao());
        println!("  Staking ratio: {:.1}%", staking_ratio);
    }
    Ok(())
}

async fn handle_dynamic(client: &Client, output: OutputFormat) -> Result<()> {
    let dynamic = client.get_all_dynamic_info().await?;
    render_rows(
        output,
        &dynamic,
        "netuid,name,symbol,tempo,price,tao_in_rao,alpha_in,alpha_out,emission,volume",
        |d| {
            format!(
                "{},{},{},{},{:.6},{},{},{},{},{}",
                d.netuid,
                csv_escape(&d.name),
                csv_escape(&d.symbol),
                d.tempo,
                d.price,
                d.tao_in.rao(),
                d.alpha_in.raw(),
                d.alpha_out.raw(),
                d.total_emission(),
                d.subnet_volume
            )
        },
        &[
            "NetUID",
            "Name",
            "Symbol",
            "Price (τ/α)",
            "TAO In",
            "Alpha In",
            "Alpha Out",
            "Emission",
            "Tempo",
        ],
        |d| {
            vec![
                format!("{}", d.netuid),
                d.name.clone(),
                d.symbol.clone(),
                format!("{:.6}", d.price),
                d.tao_in.display_tao(),
                format!("{}", d.alpha_in),
                format!("{}", d.alpha_out),
                format!("{:.4} τ", d.total_emission() as f64 / 1e9),
                format!("{}", d.tempo),
            ]
        },
        Some(&format!("Dynamic TAO — {} subnets", dynamic.len())),
    );
    Ok(())
}

async fn handle_portfolio_at_block(
    client: &Client,
    addr: &str,
    output: OutputFormat,
    block_num: u32,
) -> Result<()> {
    let block_hash = client.get_block_hash(block_num).await?;
    let (balance, stakes, prices) = tokio::try_join!(
        client.get_balance_at_block(addr, block_hash),
        client.get_stake_for_coldkey_at_block(addr, block_hash),
        async {
            Ok::<_, anyhow::Error>(
                client
                    .current_alpha_price_all_at_block(block_hash)
                    .await
                    .ok(),
            )
        },
    )?;
    let priced = prices.is_some();
    let prices = prices.unwrap_or_default();
    // Cross-subnet total: sum the τ-equivalent of each position we have a price for.
    let total_staked = stakes.iter().fold(Balance::ZERO, |acc, s| {
        acc + prices
            .get(&s.netuid.0)
            .copied()
            .map(|p| s.stake.to_tao(p))
            .unwrap_or(Balance::ZERO)
    });
    if output.is_json() {
        print_json(&serde_json::json!({
            "address": addr,
            "block": block_num,
            "free_balance_rao": balance.rao(),
            "free_balance_tao": balance.tao(),
            "total_staked_tao_rao": priced.then(|| total_staked.rao()),
            "total_staked_tao": priced.then(|| total_staked.tao()),
            "stakes": stakes.iter().map(|s| {
                let price = prices.get(&s.netuid.0).copied();
                serde_json::json!({
                    "hotkey": s.hotkey,
                    "netuid": s.netuid.0,
                    "alpha_raw": s.stake.raw(),
                    "tao_equiv_rao": price.map(|p| s.stake.to_tao(p).rao()),
                    "price": price,
                })
            }).collect::<Vec<_>>(),
        }));
    } else {
        println!(
            "Portfolio for {} (block {})",
            crate::utils::short_ss58(addr),
            block_num
        );
        println!("  Free:   {}", balance.display_tao());
        println!(
            "  Staked: {} (τ-equiv)",
            if priced {
                total_staked.display_tao()
            } else {
                "—".to_string()
            }
        );
        println!(
            "  Total:  {}",
            if priced {
                (balance + total_staked).display_tao()
            } else {
                "—".to_string()
            }
        );
        if !stakes.is_empty() {
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
                &["NetUID", "Hotkey", "Alpha (α)", "TAO≈ (τ)", "Price (τ/α)"],
                |s| {
                    let price = prices.get(&s.netuid.0).copied();
                    vec![
                        format!("{}", s.netuid),
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
                None,
            );
        }
    }
    Ok(())
}

async fn handle_dynamic_at_block(
    client: &Client,
    output: OutputFormat,
    block_num: u32,
) -> Result<()> {
    let block_hash = client.get_block_hash(block_num).await?;
    let dynamic = client.get_all_dynamic_info_at_block(block_hash).await?;
    render_rows(
        output,
        &dynamic,
        "netuid,name,symbol,tempo,price,tao_in_rao,alpha_in,alpha_out,emission,volume",
        |d| {
            format!(
                "{},{},{},{},{:.6},{},{},{},{},{}",
                d.netuid,
                csv_escape(&d.name),
                csv_escape(&d.symbol),
                d.tempo,
                d.price,
                d.tao_in.rao(),
                d.alpha_in.raw(),
                d.alpha_out.raw(),
                d.total_emission(),
                d.subnet_volume
            )
        },
        &[
            "NetUID",
            "Name",
            "Symbol",
            "Price (τ/α)",
            "TAO In",
            "Alpha In",
            "Alpha Out",
            "Emission",
            "Tempo",
        ],
        |d| {
            vec![
                format!("{}", d.netuid),
                d.name.clone(),
                d.symbol.clone(),
                format!("{:.6}", d.price),
                d.tao_in.display_tao(),
                format!("{}", d.alpha_in),
                format!("{}", d.alpha_out),
                format!("{:.4} τ", d.total_emission() as f64 / 1e9),
                format!("{}", d.tempo),
            ]
        },
        Some(&format!(
            "Dynamic TAO at block {} — {} subnets",
            block_num,
            dynamic.len()
        )),
    );
    Ok(())
}

async fn handle_neuron(
    client: &Client,
    output: OutputFormat,
    netuid: u16,
    uid: u16,
    at_block: Option<u32>,
) -> Result<()> {
    let (neuron, bh) = if let Some(bn) = at_block {
        let bh = client.get_block_hash(bn).await?;
        (
            client.get_neuron_at_block(NetUid(netuid), uid, bh).await?,
            Some(bh),
        )
    } else {
        (client.get_neuron(NetUid(netuid), uid).await?, None)
    };
    let price = match client.alpha_price_f64(NetUid(netuid), bh).await {
        Ok(p) => Some(p),
        Err(e) => {
            tracing::warn!("alpha price fetch failed (non-fatal): {e:#}");
            None
        }
    };
    match neuron {
        Some(n) => {
            if output.is_json() {
                print_json_ser(&n);
            } else if output.is_csv() {
                let axon = n
                    .axon_info
                    .as_ref()
                    .map(|a| format!("{}:{}", a.ip, a.port))
                    .unwrap_or_default();
                let prom = n
                    .prometheus_info
                    .as_ref()
                    .map(|p| format!("{}:{}", p.ip, p.port))
                    .unwrap_or_default();
                println!(
                    "uid,netuid,hotkey,coldkey,active,stake_alpha_raw,stake_tao_equiv_rao,price,rank,trust,consensus,incentive,dividends,emission_alpha,validator_trust,validator_permit,pruning_score,last_update,axon,prometheus"
                );
                println!(
                    "{},{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.0},{:.6},{},{:.6},{},{},{}",
                    n.uid,
                    netuid,
                    csv_escape(&n.hotkey),
                    csv_escape(&n.coldkey),
                    n.active,
                    n.stake.raw(),
                    price.map(|p| n.stake.to_tao(p).rao().to_string()).unwrap_or_default(),
                    price.map(|p| format!("{p:.6}")).unwrap_or_default(),
                    n.rank,
                    n.trust,
                    n.consensus,
                    n.incentive,
                    n.dividends,
                    n.emission,
                    n.validator_trust,
                    n.validator_permit,
                    n.pruning_score,
                    n.last_update,
                    csv_escape(&axon),
                    csv_escape(&prom),
                );
            } else {
                println!("Neuron UID {} on SN{}", uid, netuid);
                println!("  Hotkey:          {}", n.hotkey);
                println!("  Coldkey:         {}", n.coldkey);
                println!("  Active:          {}", n.active);
                println!(
                    "  Stake:           {} ≈ {}",
                    n.stake.display_units(),
                    price
                        .map(|p| n.stake.to_tao(p).display_tao())
                        .unwrap_or_else(|| "—".to_string())
                );
                println!("  Rank:            {:.6}", n.rank);
                println!("  Trust:           {:.6}", n.trust);
                println!("  Consensus:       {:.6}", n.consensus);
                println!("  Incentive:       {:.6}", n.incentive);
                println!("  Dividends:       {:.6}", n.dividends);
                println!(
                    "  Emission:        {:.4} α/tempo ≈ {}",
                    n.emission / 1e9,
                    price
                        .map(|p| format!("{:.4} τ", n.emission / 1e9 * p))
                        .unwrap_or_else(|| "—".to_string())
                );
                println!("  Val. Trust:      {:.6}", n.validator_trust);
                println!("  Val. Permit:     {}", n.validator_permit);
                println!("  Pruning Score:   {:.6}", n.pruning_score);
                println!("  Last Update:     {}", n.last_update);
                if let Some(axon) = &n.axon_info {
                    println!(
                        "  Axon:            {}:{} (v{}, proto {})",
                        axon.ip, axon.port, axon.version, axon.protocol
                    );
                }
                if let Some(prom) = &n.prometheus_info {
                    println!(
                        "  Prometheus:      {}:{} (v{})",
                        prom.ip, prom.port, prom.version
                    );
                }
            }
        }
        None => {
            anyhow::bail!(
                "Neuron UID {} not found on SN{}.\n  Tip: agcli subnet metagraph --netuid {}",
                uid,
                netuid,
                netuid
            );
        }
    }
    Ok(())
}

async fn handle_validators(
    client: &Client,
    output: OutputFormat,
    netuid: Option<u16>,
    limit: usize,
    at_block: Option<u32>,
) -> Result<()> {
    if let Some(nuid) = netuid {
        let (neurons, bh) = if let Some(bn) = at_block {
            let bh = client.get_block_hash(bn).await?;
            let ns = client.get_neurons_lite_at_block(NetUid(nuid), bh).await?;
            (ns, Some(bh))
        } else {
            let arc = client.get_neurons_lite(NetUid(nuid)).await?;
            let ns = std::sync::Arc::try_unwrap(arc).unwrap_or_else(|a| (*a).clone());
            (ns, None)
        };
        let price = match client.alpha_price_f64(NetUid(nuid), bh).await {
            Ok(p) => Some(p),
            Err(e) => {
                tracing::warn!("alpha price fetch failed (non-fatal): {e:#}");
                None
            }
        };
        let mut validators: Vec<_> = neurons.into_iter().filter(|n| n.validator_permit).collect();
        // Single subnet → one price; sorting by alpha is identical to sorting by τ.
        validators.sort_by_key(|item| std::cmp::Reverse(item.stake.raw()));
        validators.truncate(limit);

        render_rows(
            output,
            &validators,
            "uid,hotkey,coldkey,alpha_raw,stake_tao_equiv_rao,price,trust,vtrust,dividends,emission_alpha",
            |v| {
                format!(
                    "{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.0}",
                    v.uid,
                    v.hotkey,
                    v.coldkey,
                    v.stake.raw(),
                    price.map(|p| v.stake.to_tao(p).rao().to_string()).unwrap_or_default(),
                    price.map(|p| format!("{p:.6}")).unwrap_or_default(),
                    v.trust,
                    v.validator_trust,
                    v.dividends,
                    v.emission
                )
            },
            &["UID", "Hotkey", "Stake (α)", "≈ τ", "VTrust", "Dividends", "Emission (α)"],
            |v| {
                vec![
                    format!("{}", v.uid),
                    crate::utils::short_ss58(&v.hotkey),
                    format!("{:.4} α", v.stake.units()),
                    price
                        .map(|p| v.stake.to_tao(p).display_tao())
                        .unwrap_or_else(|| "—".to_string()),
                    format!("{:.4}", v.validator_trust),
                    format!("{:.4}", v.dividends),
                    format!("{:.4} α", v.emission / 1e9),
                ]
            },
            Some(&format!(
                "Validators on SN{} ({} with permits)",
                nuid,
                validators.len()
            )),
        );
    } else {
        // The ranking IS the priced totals, so a missing price RPC is fatal (like the delegate fetch).
        let (delegates, prices) = if let Some(bn) = at_block {
            let bh = client.get_block_hash(bn).await?;
            let d = client.get_delegates_at_block(bh).await?;
            let p = client.current_alpha_price_all_at_block(bh).await?;
            (d, p)
        } else {
            let d = client.get_delegates().await?;
            let p = client.current_alpha_price_all().await?;
            (d, p)
        };
        // Cross-subnet totals must convert per-subnet alpha to τ before summing.
        let price_fn = |n: NetUid| prices.get(&n.0).copied().unwrap_or(0.0);
        let mut sorted = delegates;
        sorted.sort_by_key(|item| std::cmp::Reverse(item.total_tao(price_fn).rao()));
        sorted.truncate(limit);

        // Add rank index for table display
        let ranked: Vec<(usize, _)> = sorted.into_iter().enumerate().collect();
        let title = if let Some(bn) = at_block {
            format!(
                "Top {} validators by total stake τ-equiv (block {})",
                ranked.len(),
                bn
            )
        } else {
            format!("Top {} validators by total stake (τ-equiv)", ranked.len())
        };
        render_rows(
            output,
            &ranked,
            "rank,hotkey,owner,take_pct,total_stake_tao_rao,nominators,registrations",
            |(i, d)| {
                format!(
                    "{},{},{},{:.2},{},{},{}",
                    i + 1,
                    d.hotkey,
                    d.owner,
                    d.take * 100.0,
                    d.total_tao(price_fn).rao(),
                    d.nominators.len(),
                    csv_escape(&format!("{:?}", d.registrations))
                )
            },
            &[
                "#",
                "Hotkey",
                "Owner",
                "Take",
                "Total Stake τ",
                "Nominators",
                "Subnets",
            ],
            |(i, d)| {
                vec![
                    format!("{}", i + 1),
                    crate::utils::short_ss58(&d.hotkey),
                    crate::utils::short_ss58(&d.owner),
                    format!("{:.2}%", d.take * 100.0),
                    d.total_tao(price_fn).display_tao(),
                    format!("{}", d.nominators.len()),
                    format!("{}", d.registrations.len()),
                ]
            },
            Some(&title),
        );
    }
    Ok(())
}

async fn handle_history(address: &str, output: OutputFormat, limit: usize) -> Result<()> {
    println!(
        "Fetching transaction history for {}...",
        crate::utils::short_ss58(address)
    );
    let url = "https://bittensor.api.subscan.io/api/v2/scan/extrinsics";
    let body = serde_json::json!({
        "address": address,
        "row": limit.min(100),
        "page": 0,
    });
    let http = reqwest::Client::new();
    let resp = http
        .post(url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let json: serde_json::Value = resp.json().await?;
    let extrinsics = json
        .get("data")
        .and_then(|d| d.get("extrinsics"))
        .and_then(|e| e.as_array());

    match extrinsics {
        Some(txs) if !txs.is_empty() => {
            render_rows(
                output,
                txs,
                "block,hash,module,call,success,timestamp",
                |tx| {
                    format!(
                        "{},{},{},{},{},{}",
                        tx.get("block_num").and_then(|v| v.as_u64()).unwrap_or(0),
                        tx.get("extrinsic_hash")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                        csv_escape(tx.get("call_module").and_then(|v| v.as_str()).unwrap_or("")),
                        csv_escape(
                            tx.get("call_module_function")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                        ),
                        tx.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
                        tx.get("block_timestamp")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0),
                    )
                },
                &["Block", "Module", "Call", "Success", "Hash"],
                |tx| {
                    let block = tx.get("block_num").and_then(|v| v.as_u64()).unwrap_or(0);
                    let module = tx
                        .get("call_module")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let call = tx
                        .get("call_module_function")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let success = tx.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let hash = tx
                        .get("extrinsic_hash")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let hash_short: String = hash.chars().take(18).collect();
                    vec![
                        format!("{}", block),
                        module.to_string(),
                        call.to_string(),
                        if success { "OK" } else { "FAIL" }.to_string(),
                        format!("{}...", hash_short),
                    ]
                },
                Some(&format!("Recent transactions ({}):", txs.len())),
            );
        }
        _ => {
            println!(
                "No transactions found for {}",
                crate::utils::short_ss58(address)
            );
            println!("Note: Subscan API may have rate limits or the address may have no activity.");
        }
    }
    Ok(())
}

async fn handle_account_explorer(
    client: &Client,
    address: &str,
    output: OutputFormat,
    at_block: Option<u32>,
) -> Result<()> {
    // Historical wayback mode
    if let Some(block_num) = at_block {
        let block_hash = client.get_block_hash(block_num).await?;
        let (balance, stakes, identity, prices) = tokio::try_join!(
            client.get_balance_at_block(address, block_hash),
            client.get_stake_for_coldkey_at_block(address, block_hash),
            client.get_identity_at_block(address, block_hash),
            async {
                Ok::<_, anyhow::Error>(
                    client
                        .current_alpha_price_all_at_block(block_hash)
                        .await
                        .ok(),
                )
            },
        )?;
        // `prices` is None only when the price RPC errored — distinct from a valid empty map.
        let priced = prices.is_some();
        let prices = prices.unwrap_or_default();
        let total_staked = stakes.iter().fold(Balance::ZERO, |acc, s| {
            acc + prices
                .get(&s.netuid.0)
                .copied()
                .map(|p| s.stake.to_tao(p))
                .unwrap_or(Balance::ZERO)
        });
        let total_value = balance + total_staked;

        if output.is_json() {
            let positions: Vec<serde_json::Value> = stakes
                .iter()
                .map(|s| {
                    let price = prices.get(&s.netuid.0).copied();
                    serde_json::json!({
                        "netuid": s.netuid.0,
                        "hotkey": s.hotkey,
                        "alpha_raw": s.stake.raw(),
                        "tao_equiv_rao": price.map(|p| s.stake.to_tao(p).rao()),
                        "price": price,
                    })
                })
                .collect();
            print_json(&serde_json::json!({
                "address": address,
                "block": block_num,
                "block_hash": format!("{:?}", block_hash),
                "balance_rao": balance.rao(),
                "balance_tao": balance.tao(),
                "total_staked_tao": priced.then(|| total_staked.tao()),
                "total_value_tao": priced.then(|| total_value.tao()),
                "stakes": positions,
                "identity": identity.as_ref().map(|id| serde_json::json!({
                    "name": id.name, "url": id.url, "discord": id.discord,
                })),
            }));
            return Ok(());
        }

        println!("Account: {} (at block {})\n", address, block_num);
        println!("  Block hash:    {:?}", block_hash);
        println!("  Free balance:  {}", balance.display_tao());
        println!(
            "  Total staked:  {} (τ-equiv)",
            if priced {
                total_staked.display_tao()
            } else {
                "—".to_string()
            }
        );
        println!(
            "  Total value:   {}",
            if priced {
                total_value.display_tao()
            } else {
                "—".to_string()
            }
        );

        if let Some(id) = &identity {
            if !id.name.is_empty() {
                println!("\n  Identity:");
                println!("    Name:    {}", id.name);
            }
        }

        if !stakes.is_empty() {
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
                Some(&format!("\n  Stake Positions ({}):", stakes.len())),
            );
        }
        return Ok(());
    }

    // Pin a single block for consistency and to save 4 redundant at_latest() RPC round-trips.
    let pin = client.pin_latest_block().await?;
    let (balance, stakes, identity, dynamic, prices, delegate) = tokio::try_join!(
        client.get_balance_at_hash(address, pin),
        client.get_stake_for_coldkey_at_block(address, pin),
        client.get_identity_at_block(address, pin),
        async {
            match client.get_all_dynamic_info().await {
                Ok(d) => Ok::<_, anyhow::Error>(d),
                Err(e) => {
                    tracing::warn!("get_all_dynamic_info failed (non-fatal): {e:#}");
                    Ok(Default::default())
                }
            }
        },
        async { Ok::<_, anyhow::Error>(client.current_alpha_price_all_at_block(pin).await.ok()) },
        async {
            Ok::<_, anyhow::Error>(match client.get_delegate_at_block(address, pin).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(error = %e, "get_delegate failed (non-fatal)");
                    None
                }
            })
        },
    )?;
    // `prices` is None only when the price RPC errored — distinct from a valid empty map.
    let priced = prices.is_some();
    let prices = prices.unwrap_or_default();
    let dynamic_map = build_dynamic_map(&dynamic);

    // τ-equivalent of one position at its subnet's spot price (None when the price is unknown).
    let pos_tao = |s: &crate::types::chain_data::StakeInfo| {
        prices
            .get(&s.netuid.0)
            .copied()
            .map(|p| s.stake.to_tao(p))
    };
    if output.is_json() {
        let total_staked = stakes.iter().fold(Balance::ZERO, |acc, s| {
            acc + pos_tao(s).unwrap_or(Balance::ZERO)
        });
        let positions: Vec<serde_json::Value> = stakes
            .iter()
            .map(|s| {
                let di = dynamic_map.get(&s.netuid.0);
                serde_json::json!({
                    "netuid": s.netuid.0,
                    "hotkey": s.hotkey,
                    "alpha_raw": s.stake.raw(),
                    "tao_equiv_rao": pos_tao(s).map(|b| b.rao()),
                    "subnet_name": di.map(|d| d.name.clone()).unwrap_or_default(),
                    "price": prices.get(&s.netuid.0).copied(),
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "address": address,
            "balance_rao": balance.rao(),
            "balance_tao": balance.tao(),
            "total_staked_tao_rao": priced.then(|| total_staked.rao()),
            "total_staked_tao": priced.then(|| total_staked.tao()),
            "stakes": positions,
            "identity": identity.as_ref().map(|id| serde_json::json!({
                "name": id.name, "url": id.url, "discord": id.discord,
            })),
            "is_delegate": delegate.is_some(),
        }));
        return Ok(());
    }

    println!("Account: {}\n", address);
    let total_staked = stakes.iter().fold(Balance::ZERO, |acc, s| {
        acc + pos_tao(s).unwrap_or(Balance::ZERO)
    });
    let total_value = balance + total_staked;
    println!("  Free balance:  {}", balance.display_tao());
    println!(
        "  Total staked:  {}",
        if priced {
            total_staked.display_tao()
        } else {
            "—".to_string()
        }
    );
    println!(
        "  Total value:   {}",
        if priced {
            total_value.display_tao()
        } else {
            "—".to_string()
        }
    );

    if let Some(id) = &identity {
        println!("\n  Identity:");
        if !id.name.is_empty() {
            println!("    Name:    {}", id.name);
        }
        if !id.url.is_empty() {
            println!("    URL:     {}", id.url);
        }
        if !id.discord.is_empty() {
            println!("    Discord: {}", id.discord);
        }
        if !id.github.is_empty() {
            println!("    GitHub:  {}", id.github);
        }
    }

    if let Some(d) = &delegate {
        println!("\n  Delegate:");
        println!("    Take:        {:.2}%", d.take * 100.0);
        println!("    Nominators:  {}", d.nominators.len());
        println!("    Subnets:     {:?}", d.registrations);
        println!("    VP subnets:  {:?}", d.validator_permits);
    }

    if !stakes.is_empty() {
        // Pair each stake with its dynamic info for render_rows
        let rows: Vec<_> = stakes
            .iter()
            .map(|s| {
                let di = dynamic_map.get(&s.netuid.0);
                (
                    s,
                    di.map(|d| d.name.clone())
                        .unwrap_or_else(|| "?".to_string()),
                    prices
                        .get(&s.netuid.0)
                        .copied()
                        .map(|p| format!("{p:.6}"))
                        .unwrap_or_else(|| "—".to_string()),
                )
            })
            .collect();
        render_rows(
            OutputFormat::Table,
            &rows,
            "",
            |_| String::new(),
            &[
                "Subnet",
                "Name",
                "Hotkey",
                "Alpha (α)",
                "TAO≈ (τ)",
                "Price (τ/α)",
            ],
            |(s, name, price)| {
                vec![
                    format!("SN{}", s.netuid.0),
                    name.clone(),
                    crate::utils::short_ss58(&s.hotkey),
                    s.stake.display_units(),
                    pos_tao(s)
                        .map(|b| b.display_tao())
                        .unwrap_or_else(|| "—".to_string()),
                    price.clone(),
                ]
            },
            Some(&format!("\n  Stake Positions ({}):", stakes.len())),
        );
    } else {
        println!("\n  No active stakes.");
    }

    Ok(())
}

async fn handle_subnet_analytics(client: &Client, netuid: u16, output: OutputFormat) -> Result<()> {
    let nuid = NetUid(netuid);
    // Pin a single block for consistency and to save 4 redundant at_latest() RPC round-trips.
    let pin = client.pin_latest_block().await?;
    let (info, dynamic, neurons, hyperparams, subnet_identity) = tokio::try_join!(
        client.get_subnet_info_at_block(nuid, pin),
        async {
            Ok::<_, anyhow::Error>(match client.get_dynamic_info_at_block(nuid, pin).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(netuid = nuid.0, error = %e, "get_dynamic_info failed (non-fatal)");
                    None
                }
            })
        },
        client.get_neurons_lite(nuid),
        async {
            Ok::<_, anyhow::Error>(
                match client.get_subnet_hyperparams_at_block(nuid, pin).await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::debug!(netuid = nuid.0, error = %e, "get_subnet_hyperparams failed (non-fatal)");
                        None
                    }
                },
            )
        },
        async {
            Ok::<_, anyhow::Error>(match client.get_subnet_identity_at_block(nuid, pin).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(netuid = nuid.0, error = %e, "get_subnet_identity failed (non-fatal)");
                    None
                }
            })
        },
    )?;

    let name = dynamic
        .as_ref()
        .map(|d| d.name.clone())
        .or_else(|| subnet_identity.as_ref().map(|i| i.subnet_name.clone()))
        .or_else(|| info.as_ref().map(|i| i.name.clone()))
        .unwrap_or_else(|| format!("SN{}", netuid));

    let n = neurons.len();
    let validators: Vec<_> = neurons.iter().filter(|n| n.validator_permit).collect();
    let miners: Vec<_> = neurons.iter().filter(|n| !n.validator_permit).collect();

    // Single subnet → one spot price (at the pinned block) converts the whole alpha column to τ.
    let price = match client.alpha_price_f64(nuid, Some(pin)).await {
        Ok(p) => Some(p),
        Err(e) => {
            tracing::warn!("alpha price fetch failed (non-fatal): {e:#}");
            None
        }
    };
    let total_stake_alpha: AlphaBalance = AlphaBalance::from_raw(
        neurons
            .iter()
            .fold(0u64, |acc, n| acc.saturating_add(n.stake.raw())),
    );
    // None (unknown price) propagates through to render as "—"/null, distinct from a real 0 τ.
    let total_stake_tao = price.map(|p| total_stake_alpha.to_tao(p));
    // Per-tempo emission in alpha (raw units summed across neurons).
    let total_emission_alpha: f64 = neurons.iter().map(|n| n.emission).sum();
    let avg_trust: f64 = if n > 0 {
        neurons.iter().map(|n| n.trust).sum::<f64>() / n as f64
    } else {
        0.0
    };
    let avg_incentive: f64 = if !miners.is_empty() {
        miners.iter().map(|n| n.incentive).sum::<f64>() / miners.len() as f64
    } else {
        0.0
    };
    let avg_dividends: f64 = if !validators.is_empty() {
        validators.iter().map(|n| n.dividends).sum::<f64>() / validators.len() as f64
    } else {
        0.0
    };

    // Sort by index to avoid cloning entire Vec of neuron records
    let mut miner_indices: Vec<usize> = (0..miners.len()).collect();
    miner_indices.sort_unstable_by(|&a, &b| {
        miners[b]
            .incentive
            .partial_cmp(&miners[a].incentive)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut val_indices: Vec<usize> = (0..validators.len()).collect();
    val_indices.sort_unstable_by(|&a, &b| {
        validators[b]
            .dividends
            .partial_cmp(&validators[a].dividends)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let unique_coldkeys: std::collections::HashSet<&String> =
        neurons.iter().map(|n| &n.coldkey).collect();

    if output.is_json() {
        print_json(&serde_json::json!({
            "netuid": netuid,
            "name": name,
            "total_neurons": n,
            "validators": validators.len(),
            "miners": miners.len(),
            "unique_owners": unique_coldkeys.len(),
            "total_stake_alpha": total_stake_alpha.units(),
            "total_stake_tao": total_stake_tao.map(|b| b.tao()),
            "total_emission_alpha": total_emission_alpha / 1e9,
            "avg_trust": avg_trust,
            "avg_miner_incentive": avg_incentive,
            "avg_validator_dividends": avg_dividends,
            "price": price,
            "tao_in": dynamic.as_ref().map(|d| d.tao_in.tao()).unwrap_or(0.0),
        }));
        return Ok(());
    }

    println!("=== Subnet Analytics: SN{} ({}) ===\n", netuid, name);

    if let Some(si) = &subnet_identity {
        if !si.description.is_empty() {
            println!("  {}\n", si.description);
        }
        if !si.github_repo.is_empty() {
            println!("  GitHub: {}", si.github_repo);
        }
        if !si.discord.is_empty() {
            println!("  Discord: {}", si.discord);
        }
        println!();
    }

    println!("  Neurons:         {}", n);
    println!(
        "  Validators:      {} ({:.0}%)",
        validators.len(),
        if n > 0 {
            validators.len() as f64 / n as f64 * 100.0
        } else {
            0.0
        }
    );
    println!(
        "  Miners:          {} ({:.0}%)",
        miners.len(),
        if n > 0 {
            miners.len() as f64 / n as f64 * 100.0
        } else {
            0.0
        }
    );
    println!("  Unique owners:   {}", unique_coldkeys.len());
    if let Some(ref i) = info {
        println!("  Max neurons:     {}", i.max_n);
        println!(
            "  Capacity:        {:.0}%",
            if i.max_n > 0 {
                n as f64 / i.max_n as f64 * 100.0
            } else {
                0.0
            }
        );
    }

    println!("\n  Economics:");
    println!(
        "    Total stake:         {:.4} α ≈ {}",
        total_stake_alpha.units(),
        total_stake_tao
            .map(|b| b.display_tao())
            .unwrap_or_else(|| "—".to_string())
    );
    println!(
        "    Total emission/tempo:{:.4} α",
        total_emission_alpha / 1e9
    );
    println!("    Avg trust:           {:.4}", avg_trust);
    println!("    Avg miner incentive: {:.4}", avg_incentive);
    println!("    Avg val dividends:   {:.4}", avg_dividends);

    if let Some(ref d) = dynamic {
        println!(
            "    Price:               {}",
            price
                .map(|p| format!("{p:.6} τ/α"))
                .unwrap_or_else(|| "—".to_string())
        );
        println!("    TAO in pool:         {}", d.tao_in.display_tao());
        println!(
            "    Subnet volume:       {:.4} τ",
            d.subnet_volume as f64 / 1e9
        );
    }

    if let Some(ref h) = hyperparams {
        println!("    Tempo:               {} blocks", h.tempo);
        println!(
            "    Commit-reveal:       {}",
            if h.commit_reveal_weights_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }

    if !miner_indices.is_empty() {
        let miners_top: Vec<_> = miner_indices.iter().take(5).map(|&i| miners[i]).collect();
        render_rows(
            OutputFormat::Table,
            &miners_top,
            "",
            |_| String::new(),
            &["UID", "Hotkey", "Incentive", "Trust", "Emission (α)"],
            |m| {
                vec![
                    format!("{}", m.uid),
                    crate::utils::short_ss58(&m.hotkey),
                    format!("{:.4}", m.incentive),
                    format!("{:.4}", m.trust),
                    format!("{:.4} α", m.emission / 1e9),
                ]
            },
            Some("\n  Top Miners (by incentive):"),
        );
    }

    if !val_indices.is_empty() {
        let vals_top: Vec<_> = val_indices.iter().take(5).map(|&i| validators[i]).collect();
        render_rows(
            OutputFormat::Table,
            &vals_top,
            "",
            |_| String::new(),
            &[
                "UID",
                "Hotkey",
                "Stake (α)",
                "≈ τ",
                "VTrust",
                "Dividends",
                "Emission (α)",
            ],
            |v| {
                vec![
                    format!("{}", v.uid),
                    crate::utils::short_ss58(&v.hotkey),
                    format!("{:.4} α", v.stake.units()),
                    price
                        .map(|p| v.stake.to_tao(p).display_tao())
                        .unwrap_or_else(|| "—".to_string()),
                    format!("{:.4}", v.validator_trust),
                    format!("{:.4}", v.dividends),
                    format!("{:.4} α", v.emission / 1e9),
                ]
            },
            Some("\n  Top Validators (by dividends):"),
        );
    }

    Ok(())
}

async fn handle_staking_analytics(
    client: &Client,
    address: &str,
    output: OutputFormat,
) -> Result<()> {
    let (stakes, dynamic, prices, block_emission) = tokio::try_join!(
        client.get_stake_for_coldkey(address),
        async {
            match client.get_all_dynamic_info().await {
                Ok(d) => Ok::<_, anyhow::Error>(d),
                Err(e) => {
                    tracing::warn!("get_all_dynamic_info failed (non-fatal): {e:#}");
                    Ok(Default::default())
                }
            }
        },
        // Price drives the whole APY/emission view, so a missing price RPC is fatal (like the stake fetch).
        client.current_alpha_price_all(),
        client.get_block_emission(),
    )?;
    let dynamic_map = build_dynamic_map(&dynamic);

    #[derive(serde::Serialize)]
    struct PositionAnalytics {
        netuid: u16,
        name: String,
        staked_tao: f64,
        price: f64,
        estimated_daily_emission_tao: f64,
        estimated_apy: f64,
    }

    let mut positions: Vec<PositionAnalytics> = Vec::new();

    for s in &stakes {
        let di = dynamic_map.get(&s.netuid.0);
        let price = prices.get(&s.netuid.0).copied().unwrap_or(0.0);
        let staked_tao = s.stake.to_tao(price).tao();
        let subnet_emission = di.map(|d| d.total_emission()).unwrap_or(0);
        let tao_in = di.map(|d| d.tao_in.tao()).unwrap_or(0.0);
        let name = di.map(|d| d.name.clone()).unwrap_or_default();

        let tempo = di.map(|d| d.tempo).unwrap_or(360);
        let (daily_emission, apy) =
            estimate_daily_emission_and_apy(staked_tao, tao_in, subnet_emission, tempo);

        positions.push(PositionAnalytics {
            netuid: s.netuid.0,
            name,
            staked_tao,
            price,
            estimated_daily_emission_tao: daily_emission,
            estimated_apy: apy,
        });
    }

    positions.sort_by(|a, b| {
        b.estimated_apy
            .partial_cmp(&a.estimated_apy)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // τ-equiv total = Σ per-subnet (alpha × price); never a raw alpha sum.
    let total_staked: f64 = stakes
        .iter()
        .map(|s| {
            s.stake
                .to_tao(prices.get(&s.netuid.0).copied().unwrap_or(0.0))
                .tao()
        })
        .sum();
    let total_daily: f64 = positions
        .iter()
        .map(|p| p.estimated_daily_emission_tao)
        .sum();
    let weighted_apy = if total_staked > 0.0 {
        total_daily / total_staked * 365.0 * 100.0
    } else {
        0.0
    };

    if output.is_json() {
        let pos_json: Vec<serde_json::Value> = positions
            .iter()
            .map(|p| {
                serde_json::json!({
                    "netuid": p.netuid,
                    "name": p.name,
                    "staked_tao": p.staked_tao,
                    "price": p.price,
                    "estimated_daily_tao": p.estimated_daily_emission_tao,
                    "estimated_apy_pct": p.estimated_apy,
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "address": address,
            "total_staked_tao": total_staked,
            "total_daily_emission_tao": total_daily,
            "weighted_apy_pct": weighted_apy,
            "block_emission_rao": block_emission.rao(),
            "positions": pos_json,
        }));
        return Ok(());
    }

    println!(
        "=== Staking Analytics for {} ===\n",
        crate::utils::short_ss58(address)
    );
    println!("  Total staked:       {:.4} τ", total_staked);
    println!("  Est. daily yield:   {:.6} τ", total_daily);
    println!("  Est. monthly yield: {:.4} τ", total_daily * 30.0);
    println!("  Est. yearly yield:  {:.4} τ", total_daily * 365.0);
    println!("  Weighted APY:       {:.2}%", weighted_apy);
    println!("  Block emission:     {}", block_emission.display_tao());

    if !positions.is_empty() {
        render_rows(
            OutputFormat::Table,
            &positions,
            "",
            |_| String::new(),
            &["Subnet", "Name", "Staked (τ)", "Price", "Daily (τ)", "APY"],
            |p| {
                vec![
                    format!("SN{}", p.netuid),
                    p.name.clone(),
                    format!("{:.4}", p.staked_tao),
                    format!("{:.6}", p.price),
                    format!("{:.6}", p.estimated_daily_emission_tao),
                    format!("{:.2}%", p.estimated_apy),
                ]
            },
            Some("\n  Position Breakdown:"),
        );
    }

    println!("\n  Note: APY estimates are based on current emission rates and pool sizes.");
    println!(
        "  Actual returns depend on validator performance, weight setting, and network changes."
    );

    Ok(())
}

async fn handle_swap_sim(
    client: &Client,
    netuid: u16,
    tao: Option<f64>,
    alpha: Option<f64>,
    output: OutputFormat,
) -> Result<()> {
    use crate::types::NetUid;
    let price_raw = client.current_alpha_price(NetUid(netuid)).await?;
    let price = price_raw as f64 / 1e9;

    // Determine direction and fetch simulation
    let sim = match (tao, alpha) {
        (Some(tao_amt), _) => {
            let (out, tf, af) = client
                .sim_swap_tao_for_alpha(NetUid(netuid), safe_rao(tao_amt))
                .await?;
            let out_f = out as f64 / 1e9;
            Some((
                "tao_to_alpha",
                "TAO → Alpha",
                tao_amt,
                "τ",
                out_f,
                "α",
                tf as f64 / 1e9,
                af as f64 / 1e9,
                if out_f > 0.0 { tao_amt / out_f } else { 0.0 },
            ))
        }
        (_, Some(alpha_amt)) => {
            let (out, tf, af) = client
                .sim_swap_alpha_for_tao(NetUid(netuid), safe_rao(alpha_amt))
                .await?;
            let out_f = out as f64 / 1e9;
            Some((
                "alpha_to_tao",
                "Alpha → TAO",
                alpha_amt,
                "α",
                out_f,
                "τ",
                tf as f64 / 1e9,
                af as f64 / 1e9,
                if alpha_amt > 0.0 {
                    out_f / alpha_amt
                } else {
                    0.0
                },
            ))
        }
        _ => None,
    };

    match sim {
        Some((dir, dir_label, amt_in, sym_in, amt_out, sym_out, tao_fee, alpha_fee, eff_price)) => {
            if output.is_json() {
                print_json(&serde_json::json!({
                    "direction": dir, "netuid": netuid,
                    "amount_in": amt_in, "amount_out": amt_out,
                    "tao_fee": tao_fee, "alpha_fee": alpha_fee,
                    "effective_price": eff_price, "current_price": price,
                }));
            } else {
                let slippage = if price > 0.0 {
                    ((eff_price - price) / price).abs() * 100.0
                } else {
                    0.0
                };
                println!("Swap Simulation — SN{}", netuid);
                println!("  Direction:       {}", dir_label);
                println!("  In:              {:.4} {}", amt_in, sym_in);
                println!("  Out:             {:.4} {}", amt_out, sym_out);
                println!("  TAO fee:         {:.6} τ", tao_fee);
                println!("  Alpha fee:       {:.6} α", alpha_fee);
                println!("  Effective price: {:.6} τ/α", eff_price);
                println!("  Current price:   {:.6} τ/α", price);
                println!("  Slippage:        {:.2}%", slippage);
            }
        }
        None => {
            if output.is_json() {
                print_json(&serde_json::json!({"netuid": netuid, "current_price": price}));
            } else {
                println!("SN{} current alpha price: {:.6} τ/α", netuid, price);
                println!(
                    "Use --tao <amount> to simulate TAO→Alpha, or --alpha <amount> for Alpha→TAO"
                );
            }
        }
    }
    Ok(())
}

async fn handle_nominations(client: &Client, hotkey: &str, output: OutputFormat) -> Result<()> {
    let delegates = client.get_delegated(hotkey).await?;
    // Per-subnet spot prices (τ/α) to convert each nominator's per-subnet alpha to TAO.
    // The delegate totals are the whole view, so a missing price RPC is fatal (like the delegate fetch).
    let prices = client.current_alpha_price_all().await?;
    let price_fn = |n: NetUid| prices.get(&n.0).copied().unwrap_or(0.0);
    if output.is_json() {
        print_json(&serde_json::json!({
            "hotkey": hotkey,
            "delegates": delegates,
        }));
        return Ok(());
    }
    if output.is_csv() {
        println!("delegate_hotkey,owner,take_pct,total_stake_tao_rao,nominator,stake_tao_rao");
        for d in &delegates {
            if d.nominators.is_empty() {
                println!(
                    "{},{},{:.6},{},,",
                    csv_escape(&d.hotkey),
                    csv_escape(&d.owner),
                    d.take * 100.0,
                    d.total_tao(price_fn).rao(),
                );
            } else {
                for (nominator, stakes) in &d.nominators {
                    println!(
                        "{},{},{:.6},{},{},{}",
                        csv_escape(&d.hotkey),
                        csv_escape(&d.owner),
                        d.take * 100.0,
                        d.total_tao(price_fn).rao(),
                        csv_escape(nominator),
                        DelegateInfo::nominator_tao(stakes, price_fn).rao(),
                    );
                }
            }
        }
        return Ok(());
    }

    if delegates.is_empty() {
        println!(
            "No delegation info found for {}",
            crate::utils::short_ss58(hotkey)
        );
        return Ok(());
    }

    println!(
        "Nominations for hotkey {}",
        crate::utils::short_ss58(hotkey)
    );
    for d in &delegates {
        println!("\n  Delegate: {}", crate::utils::short_ss58(&d.hotkey));
        println!("    Owner:       {}", crate::utils::short_ss58(&d.owner));
        println!("    Take:        {:.2}%", d.take * 100.0);
        println!(
            "    Total Stake: {} (τ-equiv)",
            d.total_tao(price_fn).display_tao()
        );
        println!("    Nominators:  {}", d.nominators.len());
        if !d.nominators.is_empty() {
            // Sort by τ-equivalent (per-subnet alpha × price) to avoid cloning the vector.
            let nom_tao =
                |i: usize| DelegateInfo::nominator_tao(&d.nominators[i].1, price_fn).rao();
            let mut indices: Vec<usize> = (0..d.nominators.len()).collect();
            indices.sort_unstable_by_key(|&i| std::cmp::Reverse(nom_tao(i)));
            println!("    Top nominators (τ-equiv):");
            for &i in indices.iter().take(10) {
                let (ref addr, ref stakes) = d.nominators[i];
                println!(
                    "      {} — {}",
                    crate::utils::short_ss58(addr),
                    DelegateInfo::nominator_tao(stakes, price_fn).display_tao()
                );
            }
        }
    }
    Ok(())
}

/// Full security audit of an account: proxies, delegates, stake exposure, permissions.
pub async fn handle_audit(client: &Client, address: &str, output: OutputFormat) -> Result<()> {
    // Pin a single block for consistency and to save 6 redundant at_latest() RPC round-trips.
    let pin = client.pin_latest_block().await?;
    let (balance, stakes, identity, proxies, delegate, dynamic, prices, coldkey_swap) = tokio::try_join!(
        client.get_balance_at_hash(address, pin),
        client.get_stake_for_coldkey_at_block(address, pin),
        client.get_identity_at_block(address, pin),
        client.list_proxies_at_block(address, pin),
        async {
            Ok::<_, anyhow::Error>(match client.get_delegate_at_block(address, pin).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!(error = %e, "get_delegate failed (non-fatal)");
                    None
                }
            })
        },
        async {
            match client.get_all_dynamic_info().await {
                Ok(d) => Ok::<_, anyhow::Error>(d),
                Err(e) => {
                    tracing::debug!(error = %e, "get_all_dynamic_info failed (non-fatal)");
                    Ok(Default::default())
                }
            }
        },
        async { Ok::<_, anyhow::Error>(client.current_alpha_price_all_at_block(pin).await.ok()) },
        async {
            Ok::<_, anyhow::Error>(
                match client
                    .get_coldkey_swap_scheduled_at_block(address, pin)
                    .await
                {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::debug!(error = %e, "get_coldkey_swap_scheduled failed (non-fatal)");
                        None
                    }
                },
            )
        },
    )?;
    let dynamic_map = build_dynamic_map(&dynamic);
    // Price is a secondary input: the security findings (coldkey swap, proxies, childkeys) are
    // price-free. When the price RPC is unavailable, value rows render as "—"/null and the
    // price-derived checks are skipped with a note — never silently shown as "no risk".
    let priced = prices.is_some();
    let prices = prices.unwrap_or_default();
    let pos_tao = |s: &crate::types::chain_data::StakeInfo| {
        prices
            .get(&s.netuid.0)
            .copied()
            .map(|p| s.stake.to_tao(p))
    };

    // Query childkey delegations + pending childkey changes at same pinned block (parallel)
    let child_key_futures: Vec<_> = stakes
        .iter()
        .map(|s| {
            let hotkey = s.hotkey.clone();
            let netuid = s.netuid;
            async move {
                let (children, pending) = tokio::join!(
                    async {
                        client
                            .get_child_keys_at_block(&hotkey, netuid, pin)
                            .await
                            .unwrap_or_default()
                    },
                    async {
                        match client.get_pending_child_keys_at_block(&hotkey, netuid, pin).await {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::debug!(hotkey = %crate::utils::short_ss58(&hotkey), netuid = netuid.0, error = %e, "Failed to fetch pending child keys");
                                None
                            }
                        }
                    },
                );
                (hotkey, netuid.0, children, pending)
            }
        })
        .collect();
    let child_key_results = futures::future::join_all(child_key_futures).await;

    // Compute risk findings
    let mut findings: Vec<serde_json::Value> = Vec::new();

    // Check scheduled coldkey swap
    if let Some((exec_block, new_ck_hash)) = &coldkey_swap {
        findings.push(serde_json::json!({
            "category": "coldkey_swap",
            "severity": "high",
            "message": format!("Coldkey swap scheduled! New coldkey hash: {} at block {}. If unauthorized, cancel immediately.", crate::utils::short_ss58(new_ck_hash), exec_block),
        }));
    }

    // Check proxies
    let has_any_proxy = proxies.iter().any(|(_, pt, _)| pt == "Any");
    if !proxies.is_empty() {
        for (delegate_ss58, proxy_type, delay) in &proxies {
            let severity = if proxy_type == "Any" && *delay == 0 {
                "high"
            } else if proxy_type == "Any" {
                "medium"
            } else {
                "info"
            };
            findings.push(serde_json::json!({
                "category": "proxy",
                "severity": severity,
                "message": format!("Proxy: {} type={} delay={}", crate::utils::short_ss58(delegate_ss58), proxy_type, delay),
            }));
        }
    }
    if has_any_proxy {
        findings.push(serde_json::json!({
            "category": "proxy",
            "severity": "high",
            "message": "Account has an 'Any' type proxy — this grants full control to another account.",
        }));
    }

    // Check stake concentration (τ-equivalent: convert each position at its subnet price).
    let total_staked: f64 = stakes.iter().filter_map(&pos_tao).map(|b| b.tao()).sum();
    let total_value = balance.tao() + total_staked;
    if priced && !stakes.is_empty() {
        let top_stake = stakes
            .iter()
            .filter_map(&pos_tao)
            .map(|b| b.tao())
            .fold(0.0_f64, f64::max);
        let concentration = if total_staked > 0.0 {
            top_stake / total_staked * 100.0
        } else {
            0.0
        };
        if concentration > 80.0 && stakes.len() > 1 {
            findings.push(serde_json::json!({
                "category": "stake",
                "severity": "medium",
                "message": format!("Stake concentration: {:.1}% of staked TAO is on a single subnet/hotkey", concentration),
            }));
        }
    }

    // Check low-liquidity exposure (skipped when price is unavailable; flagged below instead).
    if priced {
        for s in &stakes {
            if let Some(di) = dynamic_map.get(&s.netuid.0) {
                let tao_in = di.tao_in.tao();
                let stake_tao = pos_tao(s).unwrap_or(Balance::ZERO).tao();
                if tao_in > 0.0 && stake_tao > tao_in * 0.1 {
                    findings.push(serde_json::json!({
                        "category": "liquidity",
                        "severity": "medium",
                        "message": format!("SN{} ({}): stake is >{:.0}% of pool depth ({:.2}τ in pool). Large unstake will have high slippage.",
                            s.netuid.0, di.name, stake_tao / tao_in * 100.0, tao_in),
                    }));
                }
            }
        }
    } else if !stakes.is_empty() {
        // Don't let a missing price masquerade as "no risk" for the price-derived checks.
        findings.push(serde_json::json!({
            "category": "price",
            "severity": "info",
            "message": "Alpha price unavailable — stake value, concentration, and liquidity checks were skipped.",
        }));
    }

    // Check childkey delegations
    for (hotkey, netuid, children, pending) in &child_key_results {
        if !children.is_empty() {
            let total_proportion: f64 = children
                .iter()
                .map(|(p, _)| *p as f64 / u64::MAX as f64 * 100.0)
                .sum();
            findings.push(serde_json::json!({
                "category": "childkey",
                "severity": "info",
                "message": format!("SN{}: hotkey {} has {} childkey delegation(s) ({:.1}% delegated)",
                    netuid, crate::utils::short_ss58(hotkey), children.len(), total_proportion),
            }));
        }
        if let Some((pending_children, cooldown_block)) = pending {
            let total_pending_pct: f64 = pending_children
                .iter()
                .map(|(p, _)| *p as f64 / u64::MAX as f64 * 100.0)
                .sum();
            findings.push(serde_json::json!({
                "category": "pending_childkey",
                "severity": "medium",
                "message": format!("SN{}: hotkey {} has PENDING childkey change ({} children, {:.1}% delegated, cooldown block {})",
                    netuid, crate::utils::short_ss58(hotkey), pending_children.len(), total_pending_pct, cooldown_block),
            }));
        }
    }

    // Check if delegate with high take
    if let Some(ref d) = delegate {
        if d.take > 0.10 {
            findings.push(serde_json::json!({
                "category": "delegate",
                "severity": "info",
                "message": format!("Delegate take is {:.2}% (high — may deter nominators)", d.take * 100.0),
            }));
        }
    }

    // Check no identity set
    if identity.is_none() {
        findings.push(serde_json::json!({
            "category": "identity",
            "severity": "info",
            "message": "No on-chain identity set. Consider setting one for discoverability.",
        }));
    }

    // Check if balance is very low but has stakes (price-free: depends only on stake presence)
    if !stakes.is_empty() && balance.tao() < 0.1 {
        findings.push(serde_json::json!({
            "category": "balance",
            "severity": "low",
            "message": format!("Very low free balance ({:.4}τ) with active stakes. May not be able to pay tx fees.", balance.tao()),
        }));
    }

    if output.is_json() {
        let positions: Vec<serde_json::Value> = stakes
            .iter()
            .map(|s| {
                let di = dynamic_map.get(&s.netuid.0);
                serde_json::json!({
                    "netuid": s.netuid.0,
                    "hotkey": s.hotkey,
                    "alpha_raw": s.stake.raw(),
                    "tao_equiv_rao": pos_tao(s).map(|b| b.rao()),
                    "subnet_name": di.map(|d| d.name.clone()).unwrap_or_default(),
                    "price": prices.get(&s.netuid.0).copied(),
                    "tao_in_pool": di.map(|d| d.tao_in.tao()).unwrap_or(0.0),
                })
            })
            .collect();
        let proxy_json: Vec<serde_json::Value> = proxies
            .iter()
            .map(|(d, pt, delay)| serde_json::json!({"delegate": d, "proxy_type": pt, "delay": delay}))
            .collect();
        let childkey_json: Vec<serde_json::Value> = child_key_results
            .iter()
            .filter(|(_, _, c, p)| !c.is_empty() || p.is_some())
            .map(|(hk, nuid, children, pending)| {
                let mut obj = serde_json::json!({
                    "hotkey": hk,
                    "netuid": nuid,
                    "children": children.iter().map(|(p, c)| serde_json::json!({
                        "proportion_raw": p,
                        "proportion_pct": *p as f64 / u64::MAX as f64 * 100.0,
                        "child": c,
                    })).collect::<Vec<_>>(),
                });
                if let Some((pending_children, cooldown_block)) = pending {
                    obj["pending"] = serde_json::json!({
                        "children": pending_children.iter().map(|(p, c)| serde_json::json!({
                            "proportion_raw": p,
                            "proportion_pct": *p as f64 / u64::MAX as f64 * 100.0,
                            "child": c,
                        })).collect::<Vec<_>>(),
                        "cooldown_block": cooldown_block,
                    });
                }
                obj
            })
            .collect();
        print_json(&serde_json::json!({
            "address": address,
            "balance_tao": balance.tao(),
            "total_staked_tao": priced.then_some(total_staked),
            "total_value_tao": priced.then_some(total_value),
            "num_stakes": stakes.len(),
            "num_proxies": proxies.len(),
            "is_delegate": delegate.is_some(),
            "has_identity": identity.is_some(),
            "coldkey_swap_scheduled": coldkey_swap.as_ref().map(|(block, new_ck_hash)| serde_json::json!({
                "execution_block": block,
                "new_coldkey_hash": new_ck_hash,
            })),
            "childkey_delegations": childkey_json,
            "proxies": proxy_json,
            "stakes": positions,
            "findings": findings,
        }));
        return Ok(());
    }

    // Table output
    println!("=== Security Audit: {} ===\n", address);
    println!("  Free balance:  {}", balance.display_tao());
    println!(
        "  Total staked:  {}",
        if priced {
            format!("{:.4} τ (τ-equiv)", total_staked)
        } else {
            "—  (alpha price unavailable)".to_string()
        }
    );
    println!(
        "  Total value:   {}",
        if priced {
            format!("{:.4} τ", total_value)
        } else {
            "—".to_string()
        }
    );
    println!("  Stake positions: {}", stakes.len());
    println!("  Proxies:       {}", proxies.len());
    println!(
        "  Is delegate:   {}",
        if delegate.is_some() { "yes" } else { "no" }
    );
    println!(
        "  Has identity:  {}",
        if identity.is_some() { "yes" } else { "no" }
    );
    if let Some((exec_block, ref new_ck_hash)) = coldkey_swap {
        println!(
            "  CK Swap:       SCHEDULED → hash {} at block {}",
            crate::utils::short_ss58(new_ck_hash),
            exec_block
        );
    }

    if !proxies.is_empty() {
        render_rows(
            OutputFormat::Table,
            &proxies,
            "",
            |_| String::new(),
            &["Delegate", "Type", "Delay"],
            |(d, pt, delay)| {
                vec![
                    crate::utils::short_ss58(d),
                    pt.clone(),
                    format!("{}", delay),
                ]
            },
            Some("\n  Proxy Accounts:"),
        );
    }

    if !stakes.is_empty() {
        // Pair each stake with dynamic info for the table
        let exposure_rows: Vec<_> = stakes
            .iter()
            .map(|s| {
                let di = dynamic_map.get(&s.netuid.0);
                let pct = pos_tao(s).map(|b| {
                    if total_staked > 0.0 {
                        b.tao() / total_staked * 100.0
                    } else {
                        0.0
                    }
                });
                (
                    s,
                    di.map(|d| d.name.clone())
                        .unwrap_or_else(|| "?".to_string()),
                    pct,
                    di.map(|d| format!("{:.2}", d.tao_in.tao()))
                        .unwrap_or_else(|| "?".to_string()),
                )
            })
            .collect();
        render_rows(
            OutputFormat::Table,
            &exposure_rows,
            "",
            |_| String::new(),
            &[
                "Subnet",
                "Name",
                "Hotkey",
                "Stake τ",
                "% of Total",
                "Pool Depth (τ)",
            ],
            |(s, name, pct, depth)| {
                vec![
                    format!("SN{}", s.netuid.0),
                    name.clone(),
                    crate::utils::short_ss58(&s.hotkey),
                    pos_tao(s)
                        .map(|b| format!("{:.4}", b.tao()))
                        .unwrap_or_else(|| "—".to_string()),
                    pct.map(|p| format!("{:.1}%", p))
                        .unwrap_or_else(|| "—".to_string()),
                    depth.clone(),
                ]
            },
            Some("\n  Stake Exposure:"),
        );
    }

    if let Some(ref d) = delegate {
        println!("\n  Delegate Info:");
        println!("    Take:        {:.2}%", d.take * 100.0);
        println!("    Nominators:  {}", d.nominators.len());
        println!("    Subnets:     {:?}", d.registrations);
    }

    // Show childkey delegations
    let child_rows: Vec<_> = child_key_results
        .iter()
        .flat_map(|(hk, nuid, children, _)| {
            children.iter().map(move |(proportion, child)| {
                let pct = *proportion as f64 / u64::MAX as f64 * 100.0;
                (nuid, hk.clone(), child.clone(), pct)
            })
        })
        .collect();
    if !child_rows.is_empty() {
        render_rows(
            OutputFormat::Table,
            &child_rows,
            "",
            |_| String::new(),
            &["Subnet", "Parent Hotkey", "Child", "Proportion"],
            |(nuid, hk, child, pct)| {
                vec![
                    format!("SN{}", nuid),
                    crate::utils::short_ss58(hk),
                    crate::utils::short_ss58(child),
                    format!("{:.1}%", pct),
                ]
            },
            Some("\n  Childkey Delegations:"),
        );
    }

    // Show pending childkey changes
    let pending_rows: Vec<_> = child_key_results
        .iter()
        .flat_map(|(hk, nuid, _, pending)| {
            pending
                .iter()
                .flat_map(move |(pending_children, cooldown_block)| {
                    pending_children.iter().map(move |(proportion, child)| {
                        let pct = *proportion as f64 / u64::MAX as f64 * 100.0;
                        (nuid, hk.clone(), child.clone(), pct, *cooldown_block)
                    })
                })
        })
        .collect();
    if !pending_rows.is_empty() {
        render_rows(
            OutputFormat::Table,
            &pending_rows,
            "",
            |_| String::new(),
            &[
                "Subnet",
                "Parent Hotkey",
                "New Child",
                "Proportion",
                "Cooldown Block",
            ],
            |(nuid, hk, child, pct, cooldown)| {
                vec![
                    format!("SN{}", nuid),
                    crate::utils::short_ss58(hk),
                    crate::utils::short_ss58(child),
                    format!("{:.1}%", pct),
                    format!("{}", cooldown),
                ]
            },
            Some("\n  Pending Childkey Changes:"),
        );
    }

    if !findings.is_empty() {
        println!("\n  Findings:");
        for f in &findings {
            let severity = f["severity"].as_str().unwrap_or("info");
            let marker = match severity {
                "high" => "[!!]",
                "medium" => "[!] ",
                "low" => "[.] ",
                _ => "[i] ",
            };
            println!("    {} {}", marker, f["message"].as_str().unwrap_or(""));
        }
    } else {
        println!("\n  No findings — account looks clean.");
    }

    Ok(())
}

// ──────── Metagraph View ────────

async fn handle_metagraph_view(
    client: &Client,
    netuid: NetUid,
    since_block: Option<u32>,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    client.require_subnet_exists(netuid, None).await?;
    let neurons = client.get_neurons_lite(netuid).await?;

    if let Some(block_num) = since_block {
        // Diff mode: compare current vs historical
        // Parallelize block hash lookup and current block number (independent queries)
        let (block_hash, current_block) =
            tokio::try_join!(client.get_block_hash(block_num), client.get_block_number(),)?;
        let old_neurons = client.get_neurons_lite_at_block(netuid, block_hash).await?;

        let old_map: std::collections::HashMap<u16, &crate::types::chain_data::NeuronInfoLite> =
            old_neurons.iter().map(|n| (n.uid, n)).collect();

        // Deltas are reported in **alpha** (Δα): a τ-delta across two blocks would conflate
        // a real stake change with a price move. Quantity change is what this view answers.
        #[derive(serde::Serialize)]
        struct NeuronDiff {
            uid: u16,
            hotkey: String,
            change: String,
            stake_diff_alpha: f64,
            emission_diff_alpha: f64,
            incentive_diff: f64,
            trust_diff: f64,
        }

        let mut diffs = Vec::new();
        for n in neurons.iter() {
            if let Some(old) = old_map.get(&n.uid) {
                let stake_diff = n.stake.units() - old.stake.units();
                let emission_diff = (n.emission - old.emission) / 1e9;
                let incentive_diff = n.incentive - old.incentive;
                let trust_diff = n.trust - old.trust;
                if stake_diff.abs() > 0.001
                    || emission_diff.abs() > 0.0001
                    || incentive_diff.abs() > 0.0001
                    || trust_diff.abs() > 0.0001
                    || n.hotkey != old.hotkey
                {
                    diffs.push(NeuronDiff {
                        uid: n.uid,
                        hotkey: n.hotkey.clone(),
                        change: if n.hotkey != old.hotkey {
                            "replaced".into()
                        } else {
                            "changed".into()
                        },
                        stake_diff_alpha: stake_diff,
                        emission_diff_alpha: emission_diff,
                        incentive_diff,
                        trust_diff,
                    });
                }
            } else {
                diffs.push(NeuronDiff {
                    uid: n.uid,
                    hotkey: n.hotkey.clone(),
                    change: "new".into(),
                    stake_diff_alpha: n.stake.units(),
                    emission_diff_alpha: n.emission / 1e9,
                    incentive_diff: n.incentive,
                    trust_diff: n.trust,
                });
            }
        }

        let show = limit.unwrap_or(diffs.len());

        if output.is_json() {
            print_json(&serde_json::json!({
                "netuid": netuid.0,
                "since_block": block_num,
                "current_block": current_block,
                "total_neurons": neurons.len(),
                "changed": diffs.len(),
                "diffs": diffs.iter().take(show).collect::<Vec<_>>(),
            }));
        } else {
            println!(
                "Metagraph diff SN{} (block {} → {}): {} changed out of {}\n",
                netuid.0,
                block_num,
                current_block,
                diffs.len(),
                neurons.len()
            );
            for d in diffs.iter().take(show) {
                println!("  UID {:>4} [{}] ({}) stake:{:>+.4}α emission:{:>+.4}α incentive:{:>+.4} trust:{:>+.4}",
                    d.uid, d.change, crate::utils::short_ss58(&d.hotkey),
                    d.stake_diff_alpha, d.emission_diff_alpha, d.incentive_diff, d.trust_diff);
            }
        }
    } else {
        // Full metagraph dump. Single subnet → one price marks the alpha column in τ.
        let show = limit.unwrap_or(neurons.len());
        let (block, price) = tokio::try_join!(client.get_block_number(), async {
            Ok::<_, anyhow::Error>(match client.alpha_price_f64(netuid, None).await {
                Ok(p) => Some(p),
                Err(e) => {
                    tracing::warn!("alpha price fetch failed (non-fatal): {e:#}");
                    None
                }
            })
        })?;

        if output.is_json() {
            let entries: Vec<serde_json::Value> = neurons
                .iter()
                .take(show)
                .map(|n| {
                    serde_json::json!({
                        "uid": n.uid, "hotkey": n.hotkey, "coldkey": n.coldkey,
                        "stake_alpha": n.stake.units(),
                        "stake_tao_equiv": price.map(|p| n.stake.to_tao(p).tao()),
                        "emission_alpha": n.emission / 1e9,
                        "incentive": n.incentive, "consensus": n.consensus,
                        "trust": n.trust, "dividends": n.dividends,
                        "validator_trust": n.validator_trust,
                        "validator_permit": n.validator_permit,
                        "last_update": n.last_update, "active": n.active,
                    })
                })
                .collect();
            print_json(&serde_json::json!({
                "netuid": netuid.0, "block": block, "price": price, "n": neurons.len(), "neurons": entries,
            }));
        } else {
            println!(
                "Metagraph SN{} at block {} ({} neurons, price {})\n",
                netuid.0,
                block,
                neurons.len(),
                price
                    .map(|p| format!("{p:.6} τ/α"))
                    .unwrap_or_else(|| "—".to_string())
            );
            println!(
                "{:>5} {:>12} {:>12} {:>10} {:>10} {:>10} {:>10} {:>10} {:>3}",
                "UID",
                "Stake(α)",
                "≈τ",
                "Emis(α)",
                "Incentive",
                "Trust",
                "Consensus",
                "Dividends",
                "VP"
            );
            println!("{}", "-".repeat(96));
            // Sort by emission descending
            let mut indices: Vec<usize> = (0..neurons.len()).collect();
            indices.sort_unstable_by(|&a, &b| {
                neurons[b]
                    .emission
                    .partial_cmp(&neurons[a].emission)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for &i in indices.iter().take(show) {
                let n = &neurons[i];
                println!(
                    "{:>5} {:>12.4} {:>12} {:>10.4} {:>10.4} {:>10.4} {:>10.4} {:>10.4} {:>3}",
                    n.uid,
                    n.stake.units(),
                    price
                        .map(|p| format!("{:.4}", n.stake.to_tao(p).tao()))
                        .unwrap_or_else(|| "—".to_string()),
                    n.emission / 1e9,
                    n.incentive,
                    n.trust,
                    n.consensus,
                    n.dividends,
                    if n.validator_permit { "Y" } else { "" }
                );
            }
        }
    }
    Ok(())
}

// ──────── Axon Lookup ────────

async fn handle_axon_lookup(
    client: &Client,
    netuid: NetUid,
    uid: Option<u16>,
    hotkey: Option<&str>,
    output: OutputFormat,
) -> Result<()> {
    let target_uid = match (uid, hotkey) {
        (Some(u), _) => u,
        (None, Some(hk)) => {
            let neurons = client.get_neurons_lite(netuid).await?;
            neurons
                .iter()
                .find(|n| n.hotkey == hk)
                .map(|n| n.uid)
                .ok_or_else(|| anyhow::anyhow!("Hotkey {} not found on SN{}", hk, netuid.0))?
        }
        (None, None) => anyhow::bail!("Provide either --uid or --hotkey-address"),
    };

    let neuron = client.get_neuron(netuid, target_uid).await?;
    match neuron {
        Some(n) => {
            if output.is_json() {
                print_json(&serde_json::json!({
                    "netuid": netuid.0,
                    "uid": n.uid,
                    "hotkey": n.hotkey,
                    "axon": n.axon_info.as_ref().map(|a| serde_json::json!({
                        "ip": format_ip(a), "port": a.port,
                        "protocol": a.protocol, "version": a.version,
                    })),
                    "prometheus": n.prometheus_info.as_ref().map(|p| serde_json::json!({
                        "ip": format_prometheus_ip(p), "port": p.port, "version": p.version,
                    })),
                }));
            } else {
                println!("Axon for UID {} on SN{}", n.uid, netuid.0);
                println!("  Hotkey: {}", n.hotkey);
                match &n.axon_info {
                    Some(a) if a.port > 0 => {
                        println!("  IP:       {}", format_ip(a));
                        println!("  Port:     {}", a.port);
                        println!("  Protocol: {}", a.protocol);
                        println!("  Version:  {}", a.version);
                    }
                    _ => println!("  Axon: not serving"),
                }
                if let Some(p) = &n.prometheus_info {
                    if p.port > 0 {
                        println!("  Prometheus: {}:{}", format_prometheus_ip(p), p.port);
                    }
                }
            }
        }
        None => anyhow::bail!("Neuron UID {} not found on SN{}", target_uid, netuid.0),
    }
    Ok(())
}

/// Format IP from u128 string to dotted-quad.
fn format_ip(axon: &crate::types::chain_data::AxonInfo) -> String {
    if let Ok(ip_u128) = axon.ip.parse::<u128>() {
        if axon.ip_type == 4 && ip_u128 <= u32::MAX as u128 {
            let ip = ip_u128 as u32;
            return format!(
                "{}.{}.{}.{}",
                (ip >> 24) & 0xff,
                (ip >> 16) & 0xff,
                (ip >> 8) & 0xff,
                ip & 0xff
            );
        }
    }
    axon.ip.clone()
}

fn format_prometheus_ip(info: &crate::types::chain_data::PrometheusInfo) -> String {
    if let Ok(ip_u128) = info.ip.parse::<u128>() {
        if info.ip_type == 4 && ip_u128 <= u32::MAX as u128 {
            let ip = ip_u128 as u32;
            return format!(
                "{}.{}.{}.{}",
                (ip >> 24) & 0xff,
                (ip >> 16) & 0xff,
                (ip >> 8) & 0xff,
                ip & 0xff
            );
        }
    }
    info.ip.clone()
}

// ──────── Subnet Health ────────

async fn handle_subnet_health(
    client: &Client,
    netuid: NetUid,
    tcp_check: bool,
    probe_timeout_ms: u64,
    output: OutputFormat,
) -> Result<()> {
    let (neurons, dyn_info) = tokio::try_join!(
        client.get_neurons_lite(netuid),
        client.get_dynamic_info(netuid),
    )?;

    let total = neurons.len();
    let active = neurons.iter().filter(|n| n.active).count();
    let with_permit = neurons.iter().filter(|n| n.validator_permit).count();
    let block = client.get_block_number().await?;

    // If tcp_check requested, probe axons
    let mut reachable = 0u32;
    let mut unreachable = 0u32;
    let mut probes: Vec<serde_json::Value> = Vec::new();

    if tcp_check {
        // Need full neuron info for axon endpoints
        let timeout = std::time::Duration::from_millis(probe_timeout_ms);
        let mut futs = Vec::new();

        // Collect UIDs with axon info
        for n in neurons.iter() {
            let uid = n.uid;
            let hk = n.hotkey.clone();
            futs.push(async move {
                let neuron_full = match client.get_neuron(netuid, uid).await {
                    Ok(nf) => nf,
                    Err(e) => {
                        tracing::debug!(uid, error = %e, "health check: neuron fetch failed");
                        None
                    }
                };
                (uid, hk, neuron_full)
            });
        }

        // Probe up to 256 neurons to avoid overwhelming network
        let probe_limit = 256.min(futs.len());
        let batch: Vec<_> = futs.into_iter().take(probe_limit).collect();
        let results = futures::future::join_all(batch).await;

        for (uid, hk, neuron_full) in results {
            if let Some(nf) = neuron_full {
                if let Some(ref axon) = nf.axon_info {
                    if axon.port > 0 {
                        let ip_str = format_ip(axon);
                        let addr = format!("{}:{}", ip_str, axon.port);
                        let ok =
                            tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr))
                                .await
                                .map(|r| r.is_ok())
                                .unwrap_or(false);
                        if ok {
                            reachable += 1;
                        } else {
                            unreachable += 1;
                        }
                        probes.push(serde_json::json!({
                            "uid": uid, "hotkey": crate::utils::short_ss58(&hk),
                            "endpoint": addr, "reachable": ok,
                        }));
                        continue;
                    }
                }
            }
            // No axon or port=0 → not serving
        }
    }

    if output.is_json() {
        let mut obj = serde_json::json!({
            "netuid": netuid.0,
            "block": block,
            "total_neurons": total,
            "active": active,
            "active_pct": if total > 0 { active as f64 / total as f64 * 100.0 } else { 0.0 },
            "validators": with_permit,
        });
        if let Some(d) = &dyn_info {
            obj["name"] = serde_json::json!(d.name);
            obj["tempo"] = serde_json::json!(d.tempo);
            obj["price"] = serde_json::json!(d.price);
        }
        if tcp_check {
            obj["tcp_probed"] = serde_json::json!(probes.len());
            obj["reachable"] = serde_json::json!(reachable);
            obj["unreachable"] = serde_json::json!(unreachable);
            obj["probes"] = serde_json::json!(probes);
        }
        print_json(&obj);
    } else {
        let name = dyn_info.as_ref().map(|d| d.name.as_str()).unwrap_or("?");
        println!(
            "Subnet Health: SN{} ({}) at block {}\n",
            netuid.0, name, block
        );
        println!(
            "  Neurons:     {} total, {} active ({:.1}%)",
            total,
            active,
            if total > 0 {
                active as f64 / total as f64 * 100.0
            } else {
                0.0
            }
        );
        println!("  Validators:  {} with permits", with_permit);
        if let Some(d) = &dyn_info {
            println!("  Tempo:       {} blocks", d.tempo);
            println!("  Price:       {:.6} τ/α", d.price);
        }
        if tcp_check {
            println!("\n  TCP Probes ({} tested):", probes.len());
            println!("    Reachable:   {}", reachable);
            println!("    Unreachable: {}", unreachable);
            if reachable + unreachable > 0 {
                println!(
                    "    Reachability: {:.1}%",
                    reachable as f64 / (reachable + unreachable) as f64 * 100.0
                );
            }
            // Show unreachable nodes
            let unreachable_probes: Vec<_> =
                probes.iter().filter(|p| p["reachable"] == false).collect();
            if !unreachable_probes.is_empty() && unreachable_probes.len() <= 20 {
                println!("\n  Unreachable axons:");
                for p in &unreachable_probes {
                    println!("    UID {} ({}) — {}", p["uid"], p["hotkey"], p["endpoint"]);
                }
            }
        }
    }
    Ok(())
}

// ──────── Per-UID Emission Breakdown ────────

async fn handle_emissions(
    client: &Client,
    netuid: NetUid,
    limit: Option<usize>,
    output: OutputFormat,
) -> Result<()> {
    let (neurons, dyn_info) = tokio::try_join!(
        client.get_neurons_lite(netuid),
        client.get_dynamic_info(netuid),
    )?;

    let total_emission: f64 = neurons.iter().map(|n| n.emission).sum();
    let show = limit.unwrap_or(neurons.len());

    // Sort by emission descending
    let mut indices: Vec<usize> = (0..neurons.len()).collect();
    indices.sort_unstable_by(|&a, &b| {
        neurons[b]
            .emission
            .partial_cmp(&neurons[a].emission)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if output.is_json() {
        let entries: Vec<serde_json::Value> = indices
            .iter()
            .take(show)
            .map(|&i| {
                let n = &neurons[i];
                let pct = if total_emission > 0.0 {
                    n.emission / total_emission * 100.0
                } else {
                    0.0
                };
                serde_json::json!({
                    "uid": n.uid, "hotkey": n.hotkey,
                    "emission": n.emission, "emission_pct": pct,
                    "incentive": n.incentive, "dividends": n.dividends,
                    "validator_permit": n.validator_permit,
                })
            })
            .collect();
        print_json(&serde_json::json!({
            "netuid": netuid.0,
            "total_emission": total_emission,
            "name": dyn_info.as_ref().map(|d| d.name.as_str()).unwrap_or(""),
            "neurons": entries,
        }));
    } else {
        let name = dyn_info.as_ref().map(|d| d.name.as_str()).unwrap_or("?");
        println!(
            "Emission breakdown for SN{} ({}) — total emission: {:.4}\n",
            netuid.0, name, total_emission
        );
        println!(
            "{:>5} {:>12} {:>8} {:>10} {:>10} {:>3} Hotkey",
            "UID", "Emission", "%", "Incentive", "Dividends", "VP"
        );
        println!("{}", "-".repeat(80));
        for &i in indices.iter().take(show) {
            let n = &neurons[i];
            let pct = if total_emission > 0.0 {
                n.emission / total_emission * 100.0
            } else {
                0.0
            };
            println!(
                "{:>5} {:>12.4} {:>7.2}% {:>10.4} {:>10.4} {:>3} {}",
                n.uid,
                n.emission,
                pct,
                n.incentive,
                n.dividends,
                if n.validator_permit { "Y" } else { "" },
                crate::utils::short_ss58(&n.hotkey)
            );
        }
    }
    Ok(())
}

/// Estimate daily emission (in TAO) and annualized APY for a staking position.
///
/// - `staked_tao`: user's stake in TAO
/// - `tao_in`: total TAO in the subnet pool
/// - `subnet_emission_rao`: per-tempo emission in RAO (from `DynamicInfo::total_emission()`)
/// - `tempo`: subnet tempo in blocks (emission event period)
///
/// Returns `(daily_emission_tao, apy_pct)`.
fn estimate_daily_emission_and_apy(
    staked_tao: f64,
    tao_in: f64,
    subnet_emission_rao: u64,
    tempo: u16,
) -> (f64, f64) {
    let share = if tao_in > 0.0 {
        staked_tao / tao_in
    } else {
        return (0.0, 0.0);
    };
    let tempo_f = (tempo as f64).max(1.0);
    let emission_per_block_tao = subnet_emission_rao as f64 / 1e9 / tempo_f;
    let daily_emission = emission_per_block_tao * 7200.0 * share;
    let apy = if staked_tao > 0.0 {
        daily_emission / staked_tao * 365.0 * 100.0
    } else {
        0.0
    };
    (daily_emission, apy)
}

#[cfg(test)]
mod tests {
    use crate::types::chain_data::{AxonInfo, PrometheusInfo};

    #[test]
    fn module_compiles() {
        // If this test compiles, the view_cmds module is structurally valid.
    }

    #[test]
    fn handle_view_symbol_exists() {
        let _ = super::handle_view as fn(_, _, _) -> _;
    }

    #[test]
    fn format_ip_v4_basic() {
        let axon = AxonInfo {
            block: 0,
            version: 0,
            ip: "3232235777".to_string(), // 192.168.1.1 = 0xC0A80101
            port: 8080,
            ip_type: 4,
            protocol: 0,
        };
        assert_eq!(super::format_ip(&axon), "192.168.1.1");
    }

    #[test]
    fn format_ip_returns_raw_for_non_v4() {
        let axon = AxonInfo {
            block: 0,
            version: 0,
            ip: "some_ipv6_value".to_string(),
            port: 8080,
            ip_type: 6,
            protocol: 0,
        };
        // Non-IPv4: should return the raw string.
        assert_eq!(super::format_ip(&axon), "some_ipv6_value");
    }

    #[test]
    fn format_ip_loopback() {
        // 127.0.0.1 = 2130706433
        let axon = AxonInfo {
            block: 0,
            version: 0,
            ip: "2130706433".to_string(),
            port: 80,
            ip_type: 4,
            protocol: 0,
        };
        assert_eq!(super::format_ip(&axon), "127.0.0.1");
    }

    #[test]
    fn format_prometheus_ip_v4_basic() {
        let info = PrometheusInfo {
            block: 0,
            version: 0,
            ip: "3232235777".to_string(),
            port: 9090,
            ip_type: 4,
        };
        assert_eq!(super::format_prometheus_ip(&info), "192.168.1.1");
    }

    #[test]
    fn format_prometheus_ip_returns_raw_for_non_v4() {
        let info = PrometheusInfo {
            block: 0,
            version: 0,
            ip: "some_ipv6".to_string(),
            port: 9090,
            ip_type: 6,
        };
        assert_eq!(super::format_prometheus_ip(&info), "some_ipv6");
    }

    // ========== Issue 673: APY calculation — per-tempo vs per-block ==========

    #[test]
    fn apy_correctly_divides_by_tempo() {
        // Subnet with tempo=100, emission=1e9 RAO per tempo (= 1 TAO per tempo)
        // Per-block emission = 1 TAO / 100 blocks = 0.01 TAO/block
        // Daily = 0.01 * 7200 = 72 TAO daily * share
        let (daily, _apy) = super::estimate_daily_emission_and_apy(
            100.0,         // 100 TAO staked
            1000.0,        // 1000 TAO in pool
            1_000_000_000, // 1 TAO per tempo in RAO
            100,           // tempo = 100 blocks
        );
        let share = 100.0 / 1000.0; // 10%
        let expected_daily = (1.0 / 100.0) * 7200.0 * share; // 7.2
        assert!(
            (daily - expected_daily).abs() < 0.001,
            "Daily emission should be {:.4}, got {:.4}",
            expected_daily,
            daily
        );
    }

    #[test]
    fn apy_with_tempo_1_is_per_block() {
        // With tempo=1, emission IS per-block
        let (daily1, _) = super::estimate_daily_emission_and_apy(100.0, 1000.0, 1_000_000_000, 1);
        // Compare with tempo=100 — should be 100x different
        let (daily100, _) =
            super::estimate_daily_emission_and_apy(100.0, 1000.0, 1_000_000_000, 100);
        let ratio = daily1 / daily100;
        assert!(
            (ratio - 100.0).abs() < 0.01,
            "Tempo=1 should give 100x more daily emission than tempo=100, got ratio {:.2}",
            ratio
        );
    }

    #[test]
    fn apy_zero_staked_returns_zero() {
        let (daily, apy) = super::estimate_daily_emission_and_apy(0.0, 1000.0, 1_000_000_000, 100);
        assert_eq!(daily, 0.0);
        assert_eq!(apy, 0.0);
    }

    #[test]
    fn apy_zero_pool_returns_zero() {
        let (daily, apy) = super::estimate_daily_emission_and_apy(100.0, 0.0, 1_000_000_000, 100);
        assert_eq!(daily, 0.0);
        assert_eq!(apy, 0.0);
    }

    #[test]
    fn apy_tempo_zero_treated_as_one() {
        // tempo=0 should be clamped to 1 to avoid division by zero
        let (daily, _) = super::estimate_daily_emission_and_apy(100.0, 1000.0, 1_000_000_000, 0);
        let (daily_t1, _) = super::estimate_daily_emission_and_apy(100.0, 1000.0, 1_000_000_000, 1);
        assert!(
            (daily - daily_t1).abs() < 0.001,
            "tempo=0 should behave like tempo=1"
        );
    }

    #[test]
    fn apy_calculation_reasonable_range() {
        // Typical subnet: 10 TAO/tempo, tempo=360, pool=100k TAO, staked=1000 TAO
        let (_, apy) = super::estimate_daily_emission_and_apy(
            1000.0,         // 1k TAO staked
            100_000.0,      // 100k TAO pool
            10_000_000_000, // 10 TAO/tempo
            360,            // typical tempo
        );
        // Sanity check: APY should be positive and < 10000%
        assert!(apy > 0.0, "APY should be positive, got {}", apy);
        assert!(
            apy < 10000.0,
            "APY should be reasonable (<10000%), got {:.1}%",
            apy
        );
    }

    // ── Issue 83: u64 saturating_add prevents overflow ──

    #[test]
    fn saturating_add_prevents_u64_overflow() {
        let a: u64 = u64::MAX - 10;
        let b: u64 = 100;
        let result = a.saturating_add(b);
        assert_eq!(result, u64::MAX, "saturating_add should cap at u64::MAX");
    }

    #[test]
    fn saturating_fold_sums_correctly_within_range() {
        let values: Vec<u64> = vec![1_000_000_000, 2_000_000_000, 3_000_000_000];
        let total: u64 = values.iter().fold(0u64, |acc, &v| acc.saturating_add(v));
        assert_eq!(total, 6_000_000_000u64);
    }

    #[test]
    fn saturating_fold_caps_at_max_on_overflow() {
        let values: Vec<u64> = vec![u64::MAX, 1];
        let total: u64 = values.iter().fold(0u64, |acc, &v| acc.saturating_add(v));
        assert_eq!(total, u64::MAX);
    }

    // ── Issue 84: f64 precision — sum in RAO then convert ──

    #[test]
    fn rao_to_tao_conversion_preserves_precision() {
        // Summing in RAO (integer) then converting is more precise than f64 sum
        let rao_values: Vec<u64> = vec![1_000_000_001, 2_000_000_002, 3_000_000_003];
        let total_rao: u64 = rao_values
            .iter()
            .fold(0u64, |acc, &v| acc.saturating_add(v));
        let total_tao = total_rao as f64 / 1e9;
        assert!(
            (total_tao - 6.000000006).abs() < 1e-9,
            "RAO→TAO conversion should preserve precision: got {}",
            total_tao
        );
    }

    #[test]
    fn rao_sum_more_precise_than_f64_sum() {
        // Many small identical values: f64 sum accumulates error, RAO sum is exact
        let count = 10_000u64;
        let each_rao: u64 = 1_000_000_001; // 1.000000001 TAO

        // RAO path (what we do now)
        let total_rao: u64 = (0..count).fold(0u64, |acc, _| acc.saturating_add(each_rao));
        let tao_from_rao = total_rao as f64 / 1e9;

        // f64 path (old way)
        let each_tao = each_rao as f64 / 1e9;
        let tao_from_f64: f64 = (0..count).map(|_| each_tao).sum();

        // RAO path should be at least as accurate
        let expected = count as f64 * (each_rao as f64 / 1e9);
        let err_rao = (tao_from_rao - expected).abs();
        let err_f64 = (tao_from_f64 - expected).abs();
        assert!(
            err_rao <= err_f64,
            "RAO-based sum error ({:e}) should be <= f64 sum error ({:e})",
            err_rao,
            err_f64
        );
    }

    // ── Issue 141: hash_short uses chars().take() instead of byte-slicing ──

    #[test]
    fn hash_short_safe_for_ascii() {
        let hash = "0xabcdef1234567890abcdef";
        let hash_short: String = hash.chars().take(18).collect();
        assert_eq!(hash_short, "0xabcdef1234567890");
    }

    #[test]
    fn hash_short_safe_for_short_hash() {
        let hash = "0xabc";
        let hash_short: String = hash.chars().take(18).collect();
        assert_eq!(hash_short, "0xabc");
    }

    #[test]
    fn hash_short_safe_for_multibyte() {
        // Even if API returned non-ASCII (shouldn't happen), chars().take() is safe
        let hash = "\u{1F600}abcdefghij"; // emoji + ASCII
        let hash_short: String = hash.chars().take(18).collect();
        assert_eq!(hash_short.chars().count(), 11); // 1 emoji + 10 ascii chars
    }
}
