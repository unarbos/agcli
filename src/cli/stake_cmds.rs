//! Stake command handlers.

use crate::cli::StakeCommands;
use crate::cli::helpers::*;
use crate::chain::Client;
use crate::types::{Balance, NetUid};
use anyhow::Result;

pub async fn handle_stake(
    cmd: StakeCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    output: &str,
) -> Result<()> {
    match cmd {
        StakeCommands::List { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            let stakes = client.get_stake_for_coldkey(&addr).await?;
            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&stakes)?);
            } else if output == "csv" {
                println!("netuid,hotkey,stake_rao,alpha_raw");
                for s in &stakes {
                    println!("{},{},{},{}", s.netuid, s.hotkey, s.stake.rao(), s.alpha_stake.raw());
                }
            } else if stakes.is_empty() {
                println!("No stakes found for {}", crate::utils::short_ss58(&addr));
            } else {
                println!("Stakes for {}:", crate::utils::short_ss58(&addr));
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Subnet", "Hotkey", "Stake (τ)", "Alpha"]);
                for s in &stakes {
                    table.add_row(vec![
                        format!("SN{}", s.netuid),
                        crate::utils::short_ss58(&s.hotkey),
                        s.stake.display_tao(),
                        format!("{}", s.alpha_stake),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        StakeCommands::Add { amount, netuid, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let bal = Balance::from_tao(amount);
            println!("Adding stake: {} to {} on SN{}", bal.display_tao(), crate::utils::short_ss58(&hk), netuid);
            let hash = client.add_stake(&pair, &hk, NetUid(netuid), bal).await?;
            println!("Stake added. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::Remove { amount, netuid, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let bal = Balance::from_tao(amount);
            println!("Removing stake: {} from {} on SN{}", bal.display_tao(), crate::utils::short_ss58(&hk), netuid);
            let hash = client.remove_stake(&pair, &hk, NetUid(netuid), bal).await?;
            println!("Stake removed. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::Move { amount, from, to, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let bal = Balance::from_tao(amount);
            println!("Moving stake: {} from SN{} to SN{} on {}", bal.display_tao(), from, to, crate::utils::short_ss58(&hk));
            let hash = client.move_stake(&pair, &hk, NetUid(from), NetUid(to), bal).await?;
            println!("Stake moved. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::Swap { amount, netuid, from_hotkey, to_hotkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let bal = Balance::from_tao(amount);
            println!("Swapping stake: {} on SN{} from {} to {}", bal.display_tao(), netuid, crate::utils::short_ss58(&from_hotkey), crate::utils::short_ss58(&to_hotkey));
            let hash = client.swap_stake(wallet.coldkey()?, &from_hotkey, NetUid(netuid), NetUid(netuid), bal).await?;
            println!("Stake swapped. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::UnstakeAll { hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            println!("Unstaking all from {}", crate::utils::short_ss58(&hk));
            let hash = client.unstake_all(&pair, &hk).await?;
            println!("All stake removed. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::ClaimRoot { hotkey: _, netuid } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hash = client.claim_root(wallet.coldkey()?, &[netuid]).await?;
            println!("Root claimed for SN{}. Tx: {}", netuid, hash);
            Ok(())
        }
        StakeCommands::AddLimit { amount, netuid, price, partial, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let bal = Balance::from_tao(amount);
            let lp = (price * 1_000_000_000.0) as u64;
            println!("Adding stake limit: {} at {:.4} on SN{} (partial={})", bal.display_tao(), price, netuid, partial);
            let hash = client.add_stake_limit(&pair, &hk, NetUid(netuid), bal, lp, partial).await?;
            println!("Limit stake added. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::RemoveLimit { amount, netuid, price, partial, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let lp = (price * 1_000_000_000.0) as u64;
            let amt = (amount * 1_000_000_000.0) as u64;
            println!("Removing stake limit: {:.4} at {:.4} on SN{} (partial={})", amount, price, netuid, partial);
            let hash = client.remove_stake_limit(&pair, &hk, NetUid(netuid), amt, lp, partial).await?;
            println!("Limit stake removed. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::ChildkeyTake { take, netuid, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!("Setting childkey take to {:.2}% on SN{} for {}", take, netuid, crate::utils::short_ss58(&hk));
            let hash = client.set_childkey_take(&pair, &hk, NetUid(netuid), take_u16).await?;
            println!("Childkey take set. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::SetChildren { netuid, children } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None)?;
            let children_parsed = parse_children(&children)?;
            println!("Setting {} children on SN{} for {}", children_parsed.len(), netuid, crate::utils::short_ss58(&hk));
            let hash = client.set_children(&pair, &hk, NetUid(netuid), &children_parsed).await?;
            println!("Children set. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::RecycleAlpha { amount, netuid, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let amt = (amount * 1_000_000_000.0) as u64;
            println!("Recycling {:.4} alpha on SN{} for TAO via {}", amount, netuid, crate::utils::short_ss58(&hk));
            let hash = client.recycle_alpha(&pair, &hk, NetUid(netuid), amt).await?;
            println!("Alpha recycled. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::Wizard => {
            staking_wizard(client, wallet_dir, wallet_name, hotkey_name).await
        }
    }
}

async fn staking_wizard(
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
) -> Result<()> {
    println!("=== Staking Wizard ===\n");

    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    let coldkey_ss58 = wallet
        .coldkey_ss58()
        .map(|s| s.to_string())
        .unwrap_or_default();
    println!("Wallet: {} ({})", wallet_name, crate::utils::short_ss58(&coldkey_ss58));

    let balance = client.get_balance_ss58(&coldkey_ss58).await?;
    println!("Balance: {}\n", balance.display_tao());

    if balance.rao() == 0 {
        println!("You need TAO to stake. Transfer some TAO to your coldkey first.");
        return Ok(());
    }

    println!("Fetching subnet data...");
    let dynamic = client.get_all_dynamic_info().await?;
    let mut subnets_with_pool: Vec<_> = dynamic.iter()
        .filter(|d| d.tao_in.rao() > 0)
        .collect();
    subnets_with_pool.sort_by(|a, b| b.tao_in.rao().cmp(&a.tao_in.rao()));

    println!("\nTop subnets by TAO pool:");
    let display_count = subnets_with_pool.len().min(15);
    for (i, d) in subnets_with_pool.iter().take(display_count).enumerate() {
        println!(
            "  {:>2}. SN{:<3} {:<20} price={:.6} τ/α  pool={:.2} τ",
            i + 1, d.netuid, &d.name, d.price, d.tao_in.tao(),
        );
    }

    let netuid_input: String = dialoguer::Input::new()
        .with_prompt("\nEnter subnet netuid to stake on")
        .interact_text()?;
    let netuid: u16 = netuid_input.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid netuid"))?;

    let max_tao = balance.tao();
    let amount_input: String = dialoguer::Input::new()
        .with_prompt(&format!("Amount of TAO to stake (max {:.4})", max_tao))
        .interact_text()?;
    let amount: f64 = amount_input.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount"))?;

    if amount <= 0.0 || amount > max_tao {
        anyhow::bail!("Amount must be between 0 and {:.4}", max_tao);
    }

    let hotkey_ss58 = resolve_hotkey_ss58(None, &mut wallet, hotkey_name)?;
    println!("\nStaking {:.4} τ on SN{} with hotkey {}", amount, netuid, crate::utils::short_ss58(&hotkey_ss58));

    let confirm = dialoguer::Confirm::new()
        .with_prompt("Proceed?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("Cancelled.");
        return Ok(());
    }

    unlock_coldkey(&mut wallet)?;
    let stake_balance = Balance::from_tao(amount);
    let hash = client
        .add_stake(wallet.coldkey()?, &hotkey_ss58, NetUid(netuid), stake_balance)
        .await?;
    println!("Stake added! Tx: {}", hash);

    println!("\nUpdated portfolio:");
    let portfolio = crate::queries::portfolio::fetch_portfolio(client, &coldkey_ss58).await?;
    println!("  Free:   {}", portfolio.free_balance.display_tao());
    println!("  Staked: {}", portfolio.total_staked.display_tao());

    Ok(())
}
