//! CLI command execution handlers.

use crate::cli::*;
use crate::chain::Client;
use crate::wallet::Wallet;
use crate::types::Balance;
use crate::types::NetUid;
use anyhow::Result;
use sp_core::Pair as _;

/// Execute the parsed CLI command.
pub async fn execute(cli: Cli) -> Result<()> {
    let network = cli.resolve_network();
    let output = cli.output.as_str();
    let live_interval = cli.live_interval();

    match cli.command {
        Commands::Wallet(cmd) => handle_wallet(cmd, &cli.wallet_dir).await,
        Commands::Balance { address } => {
            let client = Client::connect(network.ws_url()).await?;
            let addr = resolve_coldkey_address(address, &cli.wallet_dir, &cli.wallet);
            let balance = client.get_balance_ss58(&addr).await?;
            if output == "json" {
                println!("{}", serde_json::json!({"address": addr, "balance_rao": balance.rao(), "balance_tao": balance.tao()}));
            } else {
                println!("Address: {}", addr);
                println!("Balance: {}", balance.display_tao());
            }
            Ok(())
        }
        Commands::Transfer { dest, amount } => {
            let client = Client::connect(network.ws_url()).await?;
            let mut wallet = open_wallet(&cli.wallet_dir, &cli.wallet)?;
            unlock_coldkey(&mut wallet)?;
            let balance = Balance::from_tao(amount);
            println!("Transferring {} to {}", balance.display_tao(), dest);
            let hash = client.transfer(wallet.coldkey()?, &dest, balance).await?;
            if output == "json" {
                println!("{}", serde_json::json!({"tx_hash": hash}));
            } else {
                println!("Transaction submitted: {}", hash);
            }
            Ok(())
        }
        Commands::Stake(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_stake(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, output).await
        }
        Commands::Subnet(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_subnet(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, output, live_interval).await
        }
        Commands::Weights(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_weights(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey).await
        }
        Commands::Root(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_root(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey).await
        }
        Commands::Delegate(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_delegate(cmd, &client, &cli.wallet_dir, &cli.wallet, &cli.hotkey, output).await
        }
        Commands::View(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_view(cmd, &client, &cli.wallet_dir, &cli.wallet, output, live_interval).await
        }
        Commands::Identity(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_identity(cmd, &client, &cli.wallet_dir, &cli.wallet).await
        }
        Commands::Swap(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_swap(cmd, &client, &cli.wallet_dir, &cli.wallet).await
        }
        Commands::Subscribe(cmd) => {
            let client = Client::connect(network.ws_url()).await?;
            handle_subscribe(cmd, &client, output).await
        }
    }
}

// ──────── Helpers ────────

fn open_wallet(wallet_dir: &str, wallet_name: &str) -> Result<Wallet> {
    Wallet::open(&format!("{}/{}", wallet_dir, wallet_name))
}

fn unlock_coldkey(wallet: &mut Wallet) -> Result<()> {
    let password = dialoguer::Password::new()
        .with_prompt("Coldkey password")
        .interact()?;
    wallet.unlock_coldkey(&password)
}

fn resolve_coldkey_address(address: Option<String>, wallet_dir: &str, wallet_name: &str) -> String {
    address.unwrap_or_else(|| {
        open_wallet(wallet_dir, wallet_name)
            .ok()
            .and_then(|w| w.coldkey_ss58().map(|s| s.to_string()))
            .unwrap_or_default()
    })
}

fn resolve_hotkey_ss58(
    hotkey_arg: Option<String>,
    wallet: &mut Wallet,
    hotkey_name: &str,
) -> Result<String> {
    if let Some(hk) = hotkey_arg {
        return Ok(hk);
    }
    wallet.load_hotkey(hotkey_name)?;
    wallet
        .hotkey_ss58()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not resolve hotkey SS58 address"))
}

fn parse_weight_pairs(weights_str: &str) -> Result<(Vec<u16>, Vec<u16>)> {
    let mut uids = Vec::new();
    let mut weights = Vec::new();
    for pair in weights_str.split(',') {
        let parts: Vec<&str> = pair.trim().split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid weight pair '{}', expected 'uid:weight'", pair);
        }
        uids.push(parts[0].trim().parse::<u16>()?);
        weights.push(parts[1].trim().parse::<u16>()?);
    }
    Ok((uids, weights))
}

fn parse_children(children_str: &str) -> Result<Vec<(u64, String)>> {
    let mut result = Vec::new();
    for pair in children_str.split(',') {
        let parts: Vec<&str> = pair.trim().split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid child pair '{}', expected 'proportion:hotkey_ss58'",
                pair
            );
        }
        let proportion = parts[0].trim().parse::<u64>()?;
        let hotkey = parts[1].trim().to_string();
        result.push((proportion, hotkey));
    }
    Ok(result)
}

// ──────── Wallet ────────

async fn handle_wallet(cmd: WalletCommands, wallet_dir: &str) -> Result<()> {
    match cmd {
        WalletCommands::Create { name } => {
            let password = dialoguer::Password::new()
                .with_prompt("Set coldkey password")
                .with_confirmation("Confirm password", "Passwords don't match")
                .interact()?;
            let wallet = Wallet::create(wallet_dir, &name, &password, "default")?;
            println!("Wallet '{}' created.", name);
            if let Some(addr) = wallet.coldkey_ss58() {
                println!("Coldkey: {}", addr);
            }
            if let Some(addr) = wallet.hotkey_ss58() {
                println!("Hotkey:  {}", addr);
            }
            Ok(())
        }
        WalletCommands::List => {
            let wallets = Wallet::list_wallets(wallet_dir)?;
            if wallets.is_empty() {
                println!("No wallets found in {}", wallet_dir);
            } else {
                println!("Wallets in {}:", wallet_dir);
                for name in wallets {
                    let w = Wallet::open(&format!("{}/{}", wallet_dir, name)).ok();
                    let addr = w
                        .as_ref()
                        .and_then(|w| w.coldkey_ss58().map(|s| s.to_string()))
                        .unwrap_or_else(|| "?".to_string());
                    println!("  {} ({})", name, crate::utils::short_ss58(&addr));
                }
            }
            Ok(())
        }
        WalletCommands::Show { all } => {
            let wallets = Wallet::list_wallets(wallet_dir)?;
            for name in &wallets {
                let w = Wallet::open(&format!("{}/{}", wallet_dir, name));
                if let Ok(w) = w {
                    println!("Wallet: {}", name);
                    if let Some(addr) = w.coldkey_ss58() {
                        println!("  Coldkey: {}", addr);
                    }
                    if all {
                        if let Ok(hotkeys) = w.list_hotkeys() {
                            for hk_name in &hotkeys {
                                let mut w2 =
                                    Wallet::open(&format!("{}/{}", wallet_dir, name)).unwrap();
                                if w2.load_hotkey(hk_name).is_ok() {
                                    if let Some(hk_addr) = w2.hotkey_ss58() {
                                        println!("  Hotkey '{}': {}", hk_name, hk_addr);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        WalletCommands::Import { name } => {
            let mnemonic = dialoguer::Input::<String>::new()
                .with_prompt("Enter mnemonic phrase")
                .interact_text()?;
            let password = dialoguer::Password::new()
                .with_prompt("Set password")
                .with_confirmation("Confirm", "Mismatch")
                .interact()?;
            let wallet = Wallet::import_from_mnemonic(wallet_dir, &name, &mnemonic, &password)?;
            println!("Wallet '{}' imported.", name);
            if let Some(addr) = wallet.coldkey_ss58() {
                println!("Coldkey: {}", addr);
            }
            Ok(())
        }
        WalletCommands::RegenColdkey => {
            println!("Regenerating coldkey from mnemonic...");
            let mnemonic = dialoguer::Input::<String>::new()
                .with_prompt("Enter mnemonic phrase")
                .interact_text()?;
            let password = dialoguer::Password::new()
                .with_prompt("Set password")
                .with_confirmation("Confirm", "Mismatch")
                .interact()?;
            let wallet =
                Wallet::import_from_mnemonic(wallet_dir, "default", &mnemonic, &password)?;
            println!("Coldkey regenerated.");
            if let Some(addr) = wallet.coldkey_ss58() {
                println!("Coldkey: {}", addr);
            }
            Ok(())
        }
        WalletCommands::RegenHotkey { name } => {
            println!("Regenerating hotkey '{}' from mnemonic...", name);
            let mnemonic = dialoguer::Input::<String>::new()
                .with_prompt("Enter hotkey mnemonic phrase")
                .interact_text()?;
            let pair = crate::wallet::keypair::pair_from_mnemonic(&mnemonic)?;
            let ss58 = crate::wallet::keypair::to_ss58(&pair.public(), 42);
            // Write hotkey file
            let hotkey_path =
                std::path::PathBuf::from(wallet_dir).join("default").join("hotkeys").join(&name);
            std::fs::create_dir_all(hotkey_path.parent().unwrap())?;
            crate::wallet::keyfile::write_keyfile(&hotkey_path, &mnemonic)?;
            println!("Hotkey '{}' regenerated: {}", name, ss58);
            Ok(())
        }
        WalletCommands::NewHotkey { name } => {
            let (pair, mnemonic) = crate::wallet::keypair::generate_mnemonic_keypair()?;
            let ss58 = crate::wallet::keypair::to_ss58(&pair.public(), 42);
            let hotkey_path =
                std::path::PathBuf::from(wallet_dir).join("default").join("hotkeys").join(&name);
            std::fs::create_dir_all(hotkey_path.parent().unwrap())?;
            crate::wallet::keyfile::write_keyfile(&hotkey_path, &mnemonic)?;
            println!("New hotkey '{}' created: {}", name, ss58);
            Ok(())
        }
    }
}

// ──────── Stake ────────

async fn handle_stake(
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
        StakeCommands::Add {
            amount,
            netuid,
            hotkey,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let balance = Balance::from_tao(amount);
            println!(
                "Adding stake: {} to {} on SN{}",
                balance.display_tao(),
                crate::utils::short_ss58(&hotkey_ss58),
                netuid
            );
            let hash = client
                .add_stake(wallet.coldkey()?, &hotkey_ss58, NetUid(netuid), balance)
                .await?;
            println!("Stake added. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::Remove {
            amount,
            netuid,
            hotkey,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let balance = Balance::from_tao(amount);
            println!(
                "Removing stake: {} from {} on SN{}",
                balance.display_tao(),
                crate::utils::short_ss58(&hotkey_ss58),
                netuid
            );
            let hash = client
                .remove_stake(wallet.coldkey()?, &hotkey_ss58, NetUid(netuid), balance)
                .await?;
            println!("Stake removed. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::Move {
            amount,
            from,
            to,
            hotkey,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let balance = Balance::from_tao(amount);
            println!(
                "Moving stake: {} from SN{} to SN{} on {}",
                balance.display_tao(),
                from,
                to,
                crate::utils::short_ss58(&hotkey_ss58)
            );
            let hash = client
                .move_stake(
                    wallet.coldkey()?,
                    &hotkey_ss58,
                    NetUid(from),
                    NetUid(to),
                    balance,
                )
                .await?;
            println!("Stake moved. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::Swap {
            amount,
            netuid,
            from_hotkey,
            to_hotkey,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let balance = Balance::from_tao(amount);
            println!(
                "Swapping stake: {} on SN{} from {} to {}",
                balance.display_tao(),
                netuid,
                crate::utils::short_ss58(&from_hotkey),
                crate::utils::short_ss58(&to_hotkey)
            );
            let hash = client
                .swap_stake(
                    wallet.coldkey()?,
                    &from_hotkey,
                    NetUid(netuid),
                    NetUid(netuid),
                    balance,
                )
                .await?;
            println!("Stake swapped. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::UnstakeAll { hotkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            println!(
                "Unstaking all from {}",
                crate::utils::short_ss58(&hotkey_ss58)
            );
            let hash = client.unstake_all(wallet.coldkey()?, &hotkey_ss58).await?;
            println!("All stake removed. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::ClaimRoot { hotkey: _, netuid } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            println!("Claiming root dividends for SN{}", netuid);
            let hash = client.claim_root(wallet.coldkey()?, &[netuid]).await?;
            println!("Root claimed. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::AddLimit {
            amount,
            netuid,
            price,
            partial,
            hotkey,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let balance = Balance::from_tao(amount);
            let limit_price = (price * 1_000_000_000.0) as u64;
            println!(
                "Adding stake with limit: {} at price {:.4} on SN{} (partial={})",
                balance.display_tao(),
                price,
                netuid,
                partial
            );
            let hash = client
                .add_stake_limit(
                    wallet.coldkey()?,
                    &hotkey_ss58,
                    NetUid(netuid),
                    balance,
                    limit_price,
                    partial,
                )
                .await?;
            println!("Limit stake added. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::RemoveLimit {
            amount,
            netuid,
            price,
            partial,
            hotkey,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let amount_raw = (amount * 1_000_000_000.0) as u64;
            let limit_price = (price * 1_000_000_000.0) as u64;
            println!(
                "Removing stake with limit: {:.4} at price {:.4} on SN{} (partial={})",
                amount, price, netuid, partial
            );
            let hash = client
                .remove_stake_limit(
                    wallet.coldkey()?,
                    &hotkey_ss58,
                    NetUid(netuid),
                    amount_raw,
                    limit_price,
                    partial,
                )
                .await?;
            println!("Limit stake removed. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::ChildkeyTake {
            take,
            netuid,
            hotkey,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!(
                "Setting childkey take to {:.2}% on SN{} for {}",
                take,
                netuid,
                crate::utils::short_ss58(&hotkey_ss58)
            );
            let hash = client
                .set_childkey_take(wallet.coldkey()?, &hotkey_ss58, NetUid(netuid), take_u16)
                .await?;
            println!("Childkey take set. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::SetChildren { netuid, children } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = {
                wallet.load_hotkey(hotkey_name)?;
                wallet
                    .hotkey_ss58()
                    .map(|s| s.to_string())
                    .ok_or_else(|| anyhow::anyhow!("Could not resolve hotkey"))?
            };
            let children_parsed = parse_children(&children)?;
            println!(
                "Setting {} children on SN{} for {}",
                children_parsed.len(),
                netuid,
                crate::utils::short_ss58(&hotkey_ss58)
            );
            let hash = client
                .set_children(
                    wallet.coldkey()?,
                    &hotkey_ss58,
                    NetUid(netuid),
                    &children_parsed,
                )
                .await?;
            println!("Children set. Tx: {}", hash);
            Ok(())
        }
        StakeCommands::Wizard => {
            println!("Interactive staking wizard not yet implemented.");
            println!("Use 'agcli stake add' or 'agcli stake remove' for now.");
            Ok(())
        }
    }
}

// ──────── Subnet ────────

async fn handle_subnet(
    cmd: SubnetCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    output: &str,
    live_interval: Option<u64>,
) -> Result<()> {
    match cmd {
        SubnetCommands::List => {
            let subnets = crate::queries::subnet::list_subnets(client).await?;
            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&subnets)?);
            } else if output == "csv" {
                println!("netuid,name,n,max_n,tempo,emission,burn_rao,owner");
                for s in &subnets {
                    println!("{},{},{},{},{},{},{},{}", s.netuid, s.name, s.n, s.max_n, s.tempo, s.emission_value, s.burn.rao(), s.owner);
                }
            } else if subnets.is_empty() {
                println!("No subnets found.");
            } else {
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "NetUID", "Name", "N", "Max", "Tempo", "Emission", "Burn", "Owner",
                ]);
                for s in &subnets {
                    table.add_row(vec![
                        format!("{}", s.netuid),
                        s.name.clone(),
                        format!("{}", s.n),
                        format!("{}", s.max_n),
                        format!("{}", s.tempo),
                        format!("{}", s.emission_value),
                        s.burn.display_tao(),
                        crate::utils::short_ss58(&s.owner),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        SubnetCommands::Show { netuid } => {
            let info = client.get_subnet_info(NetUid(netuid)).await?;
            let dynamic = client.get_dynamic_info(NetUid(netuid)).await.ok().flatten();
            match info {
                Some(mut s) => {
                    // Resolve real name from DynamicInfo
                    if let Some(ref di) = dynamic {
                        if !di.name.is_empty() {
                            s.name = di.name.clone();
                        }
                    }
                    if output == "json" {
                        println!("{}", serde_json::to_string_pretty(&s)?);
                    } else {
                        println!("Subnet {} ({})", s.netuid, s.name);
                        println!("  Symbol:        {}", s.symbol);
                        println!("  Neurons:       {}/{}", s.n, s.max_n);
                        println!("  Tempo:         {}", s.tempo);
                        println!("  Emission:      {}", s.emission_value);
                        println!("  Burn:          {}", s.burn.display_tao());
                        println!("  Difficulty:    {}", s.difficulty);
                        println!("  Immunity:      {} blocks", s.immunity_period);
                        println!("  Owner:         {}", s.owner);
                        println!("  Registration:  {}", if s.registration_allowed { "open" } else { "closed" });
                        if let Some(ref di) = dynamic {
                            println!("  Price:         {:.6} τ/α", di.price);
                            println!("  TAO in pool:   {}", di.tao_in.display_tao());
                            println!("  Alpha in:      {}", di.alpha_in);
                            println!("  Alpha out:     {}", di.alpha_out);
                            println!("  Volume:        {}", di.subnet_volume);
                        }
                    }
                }
                None => println!("Subnet {} not found.", netuid),
            }
            Ok(())
        }
        SubnetCommands::Hyperparams { netuid } => {
            let params = client.get_subnet_hyperparams(NetUid(netuid)).await?;
            match params {
                Some(h) => {
                    println!("Hyperparameters for SN{}", netuid);
                    let mut table = comfy_table::Table::new();
                    table.set_header(vec!["Parameter", "Value"]);
                    table.add_row(vec!["rho", &format!("{}", h.rho)]);
                    table.add_row(vec!["kappa", &format!("{}", h.kappa)]);
                    table.add_row(vec!["immunity_period", &format!("{}", h.immunity_period)]);
                    table.add_row(vec![
                        "min_allowed_weights",
                        &format!("{}", h.min_allowed_weights),
                    ]);
                    table.add_row(vec![
                        "max_weights_limit",
                        &format!("{}", h.max_weights_limit),
                    ]);
                    table.add_row(vec!["tempo", &format!("{}", h.tempo)]);
                    table.add_row(vec!["min_difficulty", &format!("{}", h.min_difficulty)]);
                    table.add_row(vec!["max_difficulty", &format!("{}", h.max_difficulty)]);
                    table.add_row(vec!["weights_version", &format!("{}", h.weights_version)]);
                    table.add_row(vec![
                        "weights_rate_limit",
                        &format!("{}", h.weights_rate_limit),
                    ]);
                    table.add_row(vec![
                        "adjustment_interval",
                        &format!("{}", h.adjustment_interval),
                    ]);
                    table.add_row(vec!["activity_cutoff", &format!("{}", h.activity_cutoff)]);
                    table.add_row(vec![
                        "registration_allowed",
                        &format!("{}", h.registration_allowed),
                    ]);
                    table.add_row(vec![
                        "target_regs_per_interval",
                        &format!("{}", h.target_regs_per_interval),
                    ]);
                    table.add_row(vec!["min_burn", &h.min_burn.display_tao()]);
                    table.add_row(vec!["max_burn", &h.max_burn.display_tao()]);
                    table.add_row(vec![
                        "bonds_moving_avg",
                        &format!("{}", h.bonds_moving_avg),
                    ]);
                    table.add_row(vec![
                        "max_regs_per_block",
                        &format!("{}", h.max_regs_per_block),
                    ]);
                    table.add_row(vec![
                        "serving_rate_limit",
                        &format!("{}", h.serving_rate_limit),
                    ]);
                    table.add_row(vec!["max_validators", &format!("{}", h.max_validators)]);
                    table.add_row(vec![
                        "adjustment_alpha",
                        &format!("{}", h.adjustment_alpha),
                    ]);
                    table.add_row(vec!["difficulty", &format!("{}", h.difficulty)]);
                    table.add_row(vec![
                        "commit_reveal_weights",
                        &format!("{}", h.commit_reveal_weights_enabled),
                    ]);
                    table.add_row(vec![
                        "commit_reveal_interval",
                        &format!("{}", h.commit_reveal_weights_interval),
                    ]);
                    table.add_row(vec![
                        "liquid_alpha_enabled",
                        &format!("{}", h.liquid_alpha_enabled),
                    ]);
                    println!("{table}");
                }
                None => println!("Hyperparameters not found for SN{}.", netuid),
            }
            Ok(())
        }
        SubnetCommands::Metagraph { netuid } => {
            if let Some(interval) = live_interval {
                return crate::live::live_metagraph(client, netuid.into(), interval).await;
            }
            let mg = crate::queries::fetch_metagraph(client, netuid.into()).await?;
            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&mg)?);
            } else if output == "csv" {
                println!("uid,hotkey,coldkey,stake_rao,rank,trust,consensus,incentive,dividends,emission,validator_permit");
                for n in &mg.neurons {
                    println!("{},{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.0},{}", n.uid, n.hotkey, n.coldkey, n.stake.rao(), n.rank, n.trust, n.consensus, n.incentive, n.dividends, n.emission, n.validator_permit);
                }
            } else {
                println!(
                    "Metagraph SN{} — {} neurons, block {}",
                    netuid, mg.n, mg.block
                );
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "UID", "Hotkey", "Stake", "Rank", "Trust", "Incentive", "Emission", "VP",
                ]);
                for n in &mg.neurons {
                    table.add_row(vec![
                        format!("{}", n.uid),
                        crate::utils::short_ss58(&n.hotkey),
                        format!("{:.4}τ", n.stake.tao()),
                        format!("{:.4}", n.rank),
                        format!("{:.4}", n.trust),
                        format!("{:.4}", n.incentive),
                        format!("{:.0}", n.emission),
                        if n.validator_permit { "Y" } else { "" }.to_string(),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        SubnetCommands::Register => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(None, &mut wallet, hotkey_name)?;
            println!("Registering new subnet...");
            let hash = client
                .register_network(wallet.coldkey()?, &hotkey_ss58)
                .await?;
            println!("Subnet registered. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::RegisterNeuron { netuid } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(None, &mut wallet, hotkey_name)?;
            println!(
                "Burn-registering on SN{} with hotkey {}",
                netuid,
                crate::utils::short_ss58(&hotkey_ss58)
            );
            let hash = client
                .burned_register(wallet.coldkey()?, NetUid(netuid), &hotkey_ss58)
                .await?;
            println!("Neuron registered. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::Pow { netuid, threads } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(None, &mut wallet, hotkey_name)?;
            let hotkey_pk = crate::wallet::keypair::from_ss58(&hotkey_ss58)?;

            println!("POW registration on SN{} with {} threads", netuid, threads);
            println!("Fetching block info and difficulty...");

            let (block_number, block_hash) = client.get_block_info_for_pow().await?;
            let difficulty = client.get_difficulty(NetUid(netuid)).await?;
            println!("Difficulty: {}, Block: #{} (0x{})", difficulty, block_number, hex::encode(block_hash));

            let attempts_per_thread = 10_000_000u64;
            let mut handles = Vec::new();
            for t in 0..threads {
                let bh = block_hash;
                let hk = hotkey_pk.0;
                let diff = difficulty;
                let offset = t as u64 * attempts_per_thread;
                handles.push(std::thread::spawn(move || {
                    crate::utils::pow::solve_pow_range(&bh, &hk, diff, offset, attempts_per_thread)
                }));
            }

            let mut result = None;
            for handle in handles {
                if let Some(found) = handle.join().map_err(|_| anyhow::anyhow!("thread panic"))? {
                    result = Some(found);
                    break;
                }
            }

            match result {
                Some((nonce, work)) => {
                    println!("POW solved! Nonce: {}", nonce);
                    let hash = client
                        .pow_register(
                            wallet.coldkey()?,
                            NetUid(netuid),
                            &hotkey_ss58,
                            block_number,
                            nonce,
                            work,
                        )
                        .await?;
                    println!("POW registered. Tx: {}", hash);
                }
                None => {
                    println!(
                        "POW not found after {} attempts per thread. Try again or use burn registration.",
                        attempts_per_thread
                    );
                }
            }
            Ok(())
        }
    }
}

// ──────── Weights ────────

async fn handle_weights(
    cmd: WeightCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
) -> Result<()> {
    match cmd {
        WeightCommands::Set {
            netuid,
            weights,
            version_key,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            println!(
                "Setting {} weights on SN{} (version_key={})",
                uids.len(),
                netuid,
                version_key
            );
            let hash = client
                .set_weights(wallet.hotkey()?, NetUid(netuid), &uids, &wts, version_key)
                .await?;
            println!("Weights set. Tx: {}", hash);
            Ok(())
        }
        WeightCommands::Commit {
            netuid,
            weights,
            salt,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            let salt_str = salt.unwrap_or_else(|| {
                use rand::Rng;
                let s: String = rand::thread_rng()
                    .sample_iter(&rand::distributions::Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect();
                println!("Generated salt: {}", s);
                s
            });
            // Compute commit hash = blake2b(uids || weights || salt)
            use blake2::digest::{Update, VariableOutput};
            let mut hasher = blake2::Blake2bVar::new(32)
                .map_err(|e| anyhow::anyhow!("blake2 init error: {:?}", e))?;
            for u in &uids {
                hasher.update(&u.to_le_bytes());
            }
            for w in &wts {
                hasher.update(&w.to_le_bytes());
            }
            hasher.update(salt_str.as_bytes());
            let mut hash_out = [0u8; 32];
            hasher
                .finalize_variable(&mut hash_out)
                .map_err(|e| anyhow::anyhow!("blake2 finalize error: {:?}", e))?;
            println!("Committing weights on SN{}", netuid);
            println!("  Commit hash: 0x{}", hex::encode(hash_out));
            println!("  Save this salt for reveal: {}", salt_str);
            let hash = client
                .commit_weights(wallet.hotkey()?, NetUid(netuid), hash_out)
                .await?;
            println!("Weights committed. Tx: {}", hash);
            Ok(())
        }
        WeightCommands::Reveal {
            netuid,
            weights,
            salt,
            version_key,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            let salt_u16: Vec<u16> = salt
                .as_bytes()
                .chunks(2)
                .map(|chunk| {
                    let b0 = chunk[0] as u16;
                    let b1 = if chunk.len() > 1 { chunk[1] as u16 } else { 0 };
                    (b1 << 8) | b0
                })
                .collect();
            println!(
                "Revealing {} weights on SN{} (version_key={})",
                uids.len(),
                netuid,
                version_key
            );
            let hash = client
                .reveal_weights(
                    wallet.hotkey()?,
                    NetUid(netuid),
                    &uids,
                    &wts,
                    &salt_u16,
                    version_key,
                )
                .await?;
            println!("Weights revealed. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Root ────────

async fn handle_root(
    cmd: RootCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
) -> Result<()> {
    match cmd {
        RootCommands::Register => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(None, &mut wallet, hotkey_name)?;
            println!(
                "Registering on root network with hotkey {}",
                crate::utils::short_ss58(&hotkey_ss58)
            );
            let hash = client
                .root_register(wallet.coldkey()?, &hotkey_ss58)
                .await?;
            println!("Root registered. Tx: {}", hash);
            Ok(())
        }
        RootCommands::Weights { weights } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            println!("Setting {} root weights", uids.len());
            let hash = client
                .set_weights(wallet.hotkey()?, NetUid::ROOT, &uids, &wts, 0)
                .await?;
            println!("Root weights set. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Delegate ────────

async fn handle_delegate(
    cmd: DelegateCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    output: &str,
) -> Result<()> {
    match cmd {
        DelegateCommands::List => {
            let delegates = client.get_delegates().await?;
            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&delegates)?);
            } else if output == "csv" {
                println!("hotkey,owner,take_pct,total_stake_rao,nominators");
                for d in &delegates {
                    println!("{},{},{:.4},{},{}", d.hotkey, d.owner, d.take * 100.0, d.total_stake.rao(), d.nominators.len());
                }
            } else {
                println!("{} delegates", delegates.len());
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "Hotkey",
                    "Owner",
                    "Take",
                    "Total Stake",
                    "Nominators",
                ]);
                for d in delegates.iter().take(50) {
                    table.add_row(vec![
                        crate::utils::short_ss58(&d.hotkey),
                        crate::utils::short_ss58(&d.owner),
                        format!("{:.2}%", d.take * 100.0),
                        d.total_stake.display_tao(),
                        format!("{}", d.nominators.len()),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        DelegateCommands::Show { hotkey } => {
            let hotkey_ss58 = match hotkey {
                Some(hk) => hk,
                None => {
                    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
                    resolve_hotkey_ss58(None, &mut wallet, hotkey_name)?
                }
            };
            let delegate = client.get_delegate(&hotkey_ss58).await?;
            match delegate {
                Some(d) => {
                    println!("Delegate: {}", d.hotkey);
                    println!("  Owner:       {}", d.owner);
                    println!("  Take:        {:.2}%", d.take * 100.0);
                    println!("  Total stake: {}", d.total_stake.display_tao());
                    println!("  Nominators:  {}", d.nominators.len());
                    println!("  Registrations: {:?}", d.registrations);
                    println!("  VP subnets:    {:?}", d.validator_permits);
                    if !d.nominators.is_empty() {
                        println!("  Top nominators:");
                        let mut sorted = d.nominators.clone();
                        sorted.sort_by(|a, b| b.1.rao().cmp(&a.1.rao()));
                        for (addr, stake) in sorted.iter().take(10) {
                            println!(
                                "    {} — {}",
                                crate::utils::short_ss58(addr),
                                stake.display_tao()
                            );
                        }
                    }
                }
                None => println!("Delegate not found for {}", hotkey_ss58),
            }
            Ok(())
        }
        DelegateCommands::DecreaseTake { take, hotkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!(
                "Decreasing take to {:.2}% for {}",
                take,
                crate::utils::short_ss58(&hotkey_ss58)
            );
            let hash = client
                .decrease_take(wallet.coldkey()?, &hotkey_ss58, take_u16)
                .await?;
            println!("Take decreased. Tx: {}", hash);
            Ok(())
        }
        DelegateCommands::IncreaseTake { take, hotkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let hotkey_ss58 = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!(
                "Increasing take to {:.2}% for {}",
                take,
                crate::utils::short_ss58(&hotkey_ss58)
            );
            let hash = client
                .increase_take(wallet.coldkey()?, &hotkey_ss58, take_u16)
                .await?;
            println!("Take increased. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── View ────────

async fn handle_view(
    cmd: ViewCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    output: &str,
    live_interval: Option<u64>,
) -> Result<()> {
    match cmd {
        ViewCommands::Portfolio { address } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            if let Some(interval) = live_interval {
                return crate::live::live_portfolio(client, &addr, interval).await;
            }
            let portfolio = crate::queries::portfolio::fetch_portfolio(client, &addr).await?;
            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&portfolio)?);
            } else if output == "csv" {
                println!("netuid,subnet_name,hotkey,alpha_stake,tao_equiv_rao,price");
                for p in &portfolio.positions {
                    println!("{},{},{},{},{},{:.6}", p.netuid, p.subnet_name, p.hotkey_ss58, p.alpha_stake, p.tao_equivalent.rao(), p.price);
                }
            } else {
                println!("Portfolio for {}", crate::utils::short_ss58(&addr));
                println!("  Free:   {}", portfolio.free_balance.display_tao());
                println!("  Staked: {}", portfolio.total_staked.display_tao());
                println!(
                    "  Total:  {}",
                    (portfolio.free_balance + portfolio.total_staked).display_tao()
                );
                if !portfolio.positions.is_empty() {
                    let mut table = comfy_table::Table::new();
                    table.set_header(vec!["Subnet", "Name", "Hotkey", "Alpha", "TAO Equiv", "Price"]);
                    for p in &portfolio.positions {
                        table.add_row(vec![
                            format!("SN{}", p.netuid),
                            p.subnet_name.clone(),
                            crate::utils::short_ss58(&p.hotkey_ss58),
                            format!("{}", p.alpha_stake),
                            format!("{}", p.tao_equivalent),
                            format!("{:.4}", p.price),
                        ]);
                    }
                    println!("{table}");
                }
            }
            Ok(())
        }
        ViewCommands::Network => {
            let block = client.get_block_number().await?;
            let total_stake = client.get_total_stake().await?;
            let total_networks = client.get_total_networks().await?;
            let total_issuance = client.get_total_issuance().await?;
            let emission = client.get_block_emission().await?;
            let staking_ratio = if total_issuance.rao() > 0 {
                total_stake.tao() / total_issuance.tao() * 100.0
            } else {
                0.0
            };
            if output == "json" {
                println!("{}", serde_json::json!({
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
        ViewCommands::Dynamic => {
            if let Some(interval) = live_interval {
                return crate::live::live_dynamic(client, interval).await;
            }
            let dynamic = client.get_all_dynamic_info().await?;
            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&dynamic)?);
            } else if output == "csv" {
                println!("netuid,name,symbol,tempo,price,tao_in_rao,alpha_in,alpha_out,emission,volume");
                for d in &dynamic {
                    println!("{},{},{},{},{:.6},{},{},{},{},{}", d.netuid, d.name, d.symbol, d.tempo, d.price, d.tao_in.rao(), d.alpha_in.raw(), d.alpha_out.raw(), d.emission, d.subnet_volume);
                }
            } else {
                println!("Dynamic TAO — {} subnets", dynamic.len());
                let mut table = comfy_table::Table::new();
                table.set_header(vec![
                    "NetUID", "Name", "Symbol", "Price (τ/α)", "TAO In", "Alpha In", "Alpha Out", "Emission", "Tempo",
                ]);
                for d in &dynamic {
                    table.add_row(vec![
                        format!("{}", d.netuid),
                        d.name.clone(),
                        d.symbol.clone(),
                        format!("{:.6}", d.price),
                        d.tao_in.display_tao(),
                        format!("{}", d.alpha_in),
                        format!("{}", d.alpha_out),
                        format!("{}", d.emission),
                        format!("{}", d.tempo),
                    ]);
                }
                println!("{table}");
            }
            Ok(())
        }
        ViewCommands::Neuron { netuid, uid } => {
            let neuron = client.get_neuron(NetUid(netuid), uid).await?;
            match neuron {
                Some(n) => {
                    println!("Neuron UID {} on SN{}", uid, netuid);
                    println!("  Hotkey:          {}", n.hotkey);
                    println!("  Coldkey:         {}", n.coldkey);
                    println!("  Active:          {}", n.active);
                    println!("  Stake:           {}", n.stake.display_tao());
                    println!("  Rank:            {:.6}", n.rank);
                    println!("  Trust:           {:.6}", n.trust);
                    println!("  Consensus:       {:.6}", n.consensus);
                    println!("  Incentive:       {:.6}", n.incentive);
                    println!("  Dividends:       {:.6}", n.dividends);
                    println!("  Emission:        {:.0}", n.emission);
                    println!("  Val. Trust:      {:.6}", n.validator_trust);
                    println!("  Val. Permit:     {}", n.validator_permit);
                    println!("  Pruning Score:   {:.6}", n.pruning_score);
                    println!("  Last Update:     {}", n.last_update);
                    if let Some(axon) = &n.axon_info {
                        println!("  Axon:            {}:{} (v{}, proto {})",
                            axon.ip, axon.port, axon.version, axon.protocol);
                    }
                    if let Some(prom) = &n.prometheus_info {
                        println!("  Prometheus:      {}:{} (v{})",
                            prom.ip, prom.port, prom.version);
                    }
                }
                None => println!("Neuron UID {} not found on SN{}", uid, netuid),
            }
            Ok(())
        }
    }
}

// ──────── Identity ────────

async fn handle_identity(
    cmd: IdentityCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
) -> Result<()> {
    match cmd {
        IdentityCommands::Show { address } => {
            let identity = client.get_identity(&address).await?;
            match identity {
                Some(id) => {
                    println!("Identity for {}", address);
                    println!("  Name:        {}", id.name);
                    println!("  URL:         {}", id.url);
                    println!("  GitHub:      {}", id.github);
                    println!("  Discord:     {}", id.discord);
                    println!("  Description: {}", id.description);
                    if !id.image.is_empty() {
                        println!("  Image:       {}", id.image);
                    }
                }
                None => println!("No identity found for {}", address),
            }
            Ok(())
        }
        IdentityCommands::Set {
            name,
            url,
            github,
            description,
        } => {
            // Identity::Set maps to SubtensorModule::set_identity for subnet identity
            // For account-level identity, the Registry pallet is used (not currently wired)
            println!("Note: Account-level identity uses the Registry pallet.");
            println!("Use 'agcli identity set-subnet' for subnet identity.");
            println!("Name: {}, URL: {:?}, GitHub: {:?}", name, url, github);
            let _ = description;
            Ok(())
        }
        IdentityCommands::SetSubnet {
            netuid,
            name,
            github,
            url,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let identity = crate::types::chain_data::SubnetIdentity {
                subnet_name: name.clone(),
                github_repo: github.unwrap_or_default(),
                subnet_contact: String::new(),
                subnet_url: url.unwrap_or_default(),
                discord: String::new(),
                description: String::new(),
                additional: String::new(),
            };
            println!("Setting subnet identity for SN{}: {}", netuid, name);
            let hash = client
                .set_subnet_identity(wallet.coldkey()?, NetUid(netuid), &identity)
                .await?;
            println!("Subnet identity set. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Swap ────────

async fn handle_swap(
    cmd: SwapCommands,
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
) -> Result<()> {
    match cmd {
        SwapCommands::Hotkey { new_hotkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            let old_hotkey = match wallet.hotkey_ss58().map(|s| s.to_string()) {
                Some(hk) => hk,
                None => {
                    wallet.load_hotkey("default")?;
                    wallet
                        .hotkey_ss58()
                        .map(|s| s.to_string())
                        .ok_or_else(|| anyhow::anyhow!("Could not resolve current hotkey"))?
                }
            };
            println!(
                "Swapping hotkey {} -> {}",
                crate::utils::short_ss58(&old_hotkey),
                crate::utils::short_ss58(&new_hotkey)
            );
            let hash = client
                .swap_hotkey(wallet.coldkey()?, &old_hotkey, &new_hotkey)
                .await?;
            println!("Hotkey swapped. Tx: {}", hash);
            Ok(())
        }
        SwapCommands::Coldkey { new_coldkey } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            println!(
                "Scheduling coldkey swap to {}",
                crate::utils::short_ss58(&new_coldkey)
            );
            let hash = client
                .schedule_swap_coldkey(wallet.coldkey()?, &new_coldkey)
                .await?;
            println!("Coldkey swap scheduled. Tx: {}", hash);
            Ok(())
        }
    }
}

// ──────── Subscribe (events/blocks) ────────

async fn handle_subscribe(
    cmd: SubscribeCommands,
    client: &Client,
    output: &str,
) -> Result<()> {
    let json = output == "json";
    match cmd {
        SubscribeCommands::Blocks => {
            crate::events::subscribe_blocks(client.inner_client(), json).await
        }
        SubscribeCommands::Events { filter } => {
            let f = crate::events::EventFilter::from_str(&filter);
            crate::events::subscribe_events(client.inner_client(), f, json).await
        }
    }
}
