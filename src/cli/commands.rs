//! CLI command execution handlers.

use crate::cli::*;
use crate::chain::Client;
use crate::wallet::Wallet;
use crate::types::Balance;
use crate::types::NetUid;
use anyhow::Result;
use clap::CommandFactory;
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
        Commands::Multisig(cmd) => {
            handle_multisig(cmd, &cli.wallet_dir, &cli.wallet, network.ws_url()).await
        }
        Commands::Config(cmd) => handle_config(cmd).await,
        Commands::Completions { shell } => {
            generate_completions(&shell);
            Ok(())
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

/// Shortcut: open wallet, unlock, resolve hotkey, return (pair, hotkey_ss58).
fn unlock_and_resolve(
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    hotkey_arg: Option<String>,
) -> Result<(sp_core::sr25519::Pair, String)> {
    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    unlock_coldkey(&mut wallet)?;
    let hotkey_ss58 = resolve_hotkey_ss58(hotkey_arg, &mut wallet, hotkey_name)?;
    let pair = wallet.coldkey()?.clone();
    Ok((pair, hotkey_ss58))
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
        StakeCommands::Wizard => {
            staking_wizard(client, wallet_dir, wallet_name, hotkey_name).await
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
                    let rows: Vec<(&str, String)> = vec![
                        ("rho", format!("{}", h.rho)),
                        ("kappa", format!("{}", h.kappa)),
                        ("immunity_period", format!("{}", h.immunity_period)),
                        ("min_allowed_weights", format!("{}", h.min_allowed_weights)),
                        ("max_weights_limit", format!("{}", h.max_weights_limit)),
                        ("tempo", format!("{}", h.tempo)),
                        ("min_difficulty", format!("{}", h.min_difficulty)),
                        ("max_difficulty", format!("{}", h.max_difficulty)),
                        ("weights_version", format!("{}", h.weights_version)),
                        ("weights_rate_limit", format!("{}", h.weights_rate_limit)),
                        ("adjustment_interval", format!("{}", h.adjustment_interval)),
                        ("activity_cutoff", format!("{}", h.activity_cutoff)),
                        ("registration_allowed", format!("{}", h.registration_allowed)),
                        ("target_regs_per_interval", format!("{}", h.target_regs_per_interval)),
                        ("min_burn", h.min_burn.display_tao()),
                        ("max_burn", h.max_burn.display_tao()),
                        ("bonds_moving_avg", format!("{}", h.bonds_moving_avg)),
                        ("max_regs_per_block", format!("{}", h.max_regs_per_block)),
                        ("serving_rate_limit", format!("{}", h.serving_rate_limit)),
                        ("max_validators", format!("{}", h.max_validators)),
                        ("adjustment_alpha", format!("{}", h.adjustment_alpha)),
                        ("difficulty", format!("{}", h.difficulty)),
                        ("commit_reveal_weights", format!("{}", h.commit_reveal_weights_enabled)),
                        ("commit_reveal_interval", format!("{}", h.commit_reveal_weights_interval)),
                        ("liquid_alpha_enabled", format!("{}", h.liquid_alpha_enabled)),
                    ];
                    for (k, v) in &rows { table.add_row(vec![k, v.as_str()]); }
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
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None)?;
            println!("Registering new subnet...");
            let hash = client.register_network(&pair, &hk).await?;
            println!("Subnet registered. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::RegisterNeuron { netuid } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None)?;
            println!("Burn-registering on SN{} with hotkey {}", netuid, crate::utils::short_ss58(&hk));
            let hash = client.burned_register(&pair, NetUid(netuid), &hk).await?;
            println!("Neuron registered. Tx: {}", hash);
            Ok(())
        }
        SubnetCommands::Pow { netuid, threads } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None)?;
            let hotkey_pk = crate::wallet::keypair::from_ss58(&hk)?;
            println!("POW registration on SN{} with {} threads", netuid, threads);
            let (block_number, block_hash) = client.get_block_info_for_pow().await?;
            let difficulty = client.get_difficulty(NetUid(netuid)).await?;
            println!("Difficulty: {}, Block: #{}", difficulty, block_number);

            let attempts_per_thread = 10_000_000u64;
            let mut handles = Vec::new();
            for t in 0..threads {
                let (bh, hk_bytes, diff) = (block_hash, hotkey_pk.0, difficulty);
                let offset = t as u64 * attempts_per_thread;
                handles.push(std::thread::spawn(move || {
                    crate::utils::pow::solve_pow_range(&bh, &hk_bytes, diff, offset, attempts_per_thread)
                }));
            }
            let result = handles.into_iter()
                .find_map(|h| h.join().ok().flatten());
            match result {
                Some((nonce, work)) => {
                    println!("POW solved! Nonce: {}", nonce);
                    let hash = client.pow_register(&pair, NetUid(netuid), &hk, block_number, nonce, work).await?;
                    println!("POW registered. Tx: {}", hash);
                }
                None => println!("POW not found after {} attempts/thread. Try burn registration.", attempts_per_thread),
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
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None)?;
            println!("Registering on root network with hotkey {}", crate::utils::short_ss58(&hk));
            let hash = client.root_register(&pair, &hk).await?;
            println!("Root registered. Tx: {}", hash);
            Ok(())
        }
        RootCommands::Weights { weights } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            println!("Setting {} root weights", uids.len());
            let hash = client.set_weights(wallet.hotkey()?, NetUid::ROOT, &uids, &wts, 0).await?;
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
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!("Decreasing take to {:.2}% for {}", take, crate::utils::short_ss58(&hk));
            let hash = client.decrease_take(&pair, &hk, take_u16).await?;
            println!("Take decreased. Tx: {}", hash);
            Ok(())
        }
        DelegateCommands::IncreaseTake { take, hotkey } => {
            let (pair, hk) = unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, hotkey)?;
            let take_u16 = (take / 100.0 * 65535.0).min(65535.0) as u16;
            println!("Increasing take to {:.2}% for {}", take, crate::utils::short_ss58(&hk));
            let hash = client.increase_take(&pair, &hk, take_u16).await?;
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
        ViewCommands::Validators { netuid, limit } => {
            handle_validators(client, output, netuid, limit).await
        }
        ViewCommands::History { address, limit } => {
            let addr = resolve_coldkey_address(address, wallet_dir, wallet_name);
            handle_history(&addr, output, limit).await
        }
    }
}

// ──────── Validators ────────

async fn handle_validators(
    client: &Client,
    output: &str,
    netuid: Option<u16>,
    limit: usize,
) -> Result<()> {
    // Collect validators: either from a specific subnet or top delegates
    if let Some(nuid) = netuid {
        // Show validators on a specific subnet
        let neurons = client.get_neurons_lite(NetUid(nuid)).await?;
        let mut validators: Vec<_> = neurons.into_iter()
            .filter(|n| n.validator_permit)
            .collect();
        validators.sort_by(|a, b| b.stake.rao().cmp(&a.stake.rao()));
        validators.truncate(limit);

        if output == "json" {
            println!("{}", serde_json::to_string_pretty(&validators)?);
        } else if output == "csv" {
            println!("uid,hotkey,coldkey,stake_rao,trust,vtrust,dividends,emission");
            for v in &validators {
                println!("{},{},{},{},{:.6},{:.6},{:.6},{:.0}", v.uid, v.hotkey, v.coldkey, v.stake.rao(), v.trust, v.validator_trust, v.dividends, v.emission);
            }
        } else {
            println!("Validators on SN{} ({} with permits)", nuid, validators.len());
            let mut table = comfy_table::Table::new();
            table.set_header(vec!["UID", "Hotkey", "Stake", "VTrust", "Dividends", "Emission"]);
            for v in &validators {
                table.add_row(vec![
                    format!("{}", v.uid),
                    crate::utils::short_ss58(&v.hotkey),
                    format!("{:.4}τ", v.stake.tao()),
                    format!("{:.4}", v.validator_trust),
                    format!("{:.4}", v.dividends),
                    format!("{:.0}", v.emission),
                ]);
            }
            println!("{table}");
        }
    } else {
        // Show top delegates (global validators overview)
        let delegates = client.get_delegates().await?;
        let mut sorted = delegates;
        sorted.sort_by(|a, b| b.total_stake.rao().cmp(&a.total_stake.rao()));
        sorted.truncate(limit);

        if output == "json" {
            println!("{}", serde_json::to_string_pretty(&sorted)?);
        } else if output == "csv" {
            println!("hotkey,owner,take_pct,total_stake_rao,nominators,registrations");
            for d in &sorted {
                println!("{},{},{:.2},{},{},{:?}", d.hotkey, d.owner, d.take * 100.0, d.total_stake.rao(), d.nominators.len(), d.registrations);
            }
        } else {
            println!("Top {} validators by total stake", sorted.len());
            let mut table = comfy_table::Table::new();
            table.set_header(vec!["#", "Hotkey", "Owner", "Take", "Total Stake", "Nominators", "Subnets"]);
            for (i, d) in sorted.iter().enumerate() {
                table.add_row(vec![
                    format!("{}", i + 1),
                    crate::utils::short_ss58(&d.hotkey),
                    crate::utils::short_ss58(&d.owner),
                    format!("{:.2}%", d.take * 100.0),
                    d.total_stake.display_tao(),
                    format!("{}", d.nominators.len()),
                    format!("{}", d.registrations.len()),
                ]);
            }
            println!("{table}");
        }
    }
    Ok(())
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

// ──────── Config ────────

async fn handle_config(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Show => {
            let cfg = crate::config::Config::load();
            let s = toml::to_string_pretty(&cfg)?;
            if s.trim().is_empty() {
                println!("No configuration set. Use 'agcli config set <key> <value>' to configure.");
            } else {
                println!("{}", s);
            }
            Ok(())
        }
        ConfigCommands::Set { key, value } => {
            let mut cfg = crate::config::Config::load();
            match key.as_str() {
                "network" => cfg.network = Some(value),
                "endpoint" => cfg.endpoint = Some(value),
                "wallet_dir" => cfg.wallet_dir = Some(value),
                "wallet" => cfg.wallet = Some(value),
                "hotkey" => cfg.hotkey = Some(value),
                "output" => {
                    if !["table", "json", "csv"].contains(&value.as_str()) {
                        anyhow::bail!("Invalid output format '{}'. Must be: table, json, csv", value);
                    }
                    cfg.output = Some(value);
                }
                "proxy" => cfg.proxy = Some(value),
                "live_interval" => {
                    let v: u64 = value.parse().map_err(|_| anyhow::anyhow!("Invalid interval '{}'", value))?;
                    cfg.live_interval = Some(v);
                }
                _ => anyhow::bail!("Unknown config key '{}'. Valid keys: network, endpoint, wallet_dir, wallet, hotkey, output, proxy, live_interval", key),
            }
            cfg.save()?;
            println!("Set {} = {}", key, cfg_value_display(&key, &cfg));
            Ok(())
        }
        ConfigCommands::Unset { key } => {
            let mut cfg = crate::config::Config::load();
            match key.as_str() {
                "network" => cfg.network = None,
                "endpoint" => cfg.endpoint = None,
                "wallet_dir" => cfg.wallet_dir = None,
                "wallet" => cfg.wallet = None,
                "hotkey" => cfg.hotkey = None,
                "output" => cfg.output = None,
                "proxy" => cfg.proxy = None,
                "live_interval" => cfg.live_interval = None,
                _ => anyhow::bail!("Unknown config key '{}'", key),
            }
            cfg.save()?;
            println!("Unset {}", key);
            Ok(())
        }
        ConfigCommands::Path => {
            println!("{}", crate::config::Config::default_path().display());
            Ok(())
        }
    }
}

// ──────── Multisig ────────

async fn handle_multisig(
    cmd: MultisigCommands,
    wallet_dir: &str,
    wallet_name: &str,
    ws_url: &str,
) -> Result<()> {
    match cmd {
        MultisigCommands::Address { signatories, threshold } => {
            let addrs: Vec<&str> = signatories.split(',').map(|s| s.trim()).collect();
            if addrs.len() < 2 {
                anyhow::bail!("Need at least 2 signatories for a multisig");
            }
            // Sort account IDs lexicographically (Substrate requirement)
            let mut account_ids: Vec<crate::AccountId> = addrs.iter()
                .map(|s| Client::ss58_to_account_id_pub(s))
                .collect::<Result<_>>()?;
            account_ids.sort();

            // Derive multisig address: blake2_256(b"modlpy/teleport" ++ threshold ++ sorted_accounts)
            use blake2::digest::{Update, VariableOutput};
            let mut hasher = blake2::Blake2bVar::new(32)
                .map_err(|e| anyhow::anyhow!("blake2 error: {:?}", e))?;
            hasher.update(b"modlpy/teleport");
            hasher.update(&(threshold as u16).to_le_bytes());
            for id in &account_ids {
                hasher.update(id.as_ref());
            }
            let mut hash = [0u8; 32];
            hasher.finalize_variable(&mut hash)
                .map_err(|e| anyhow::anyhow!("blake2 finalize error: {:?}", e))?;

            let multisig_account = sp_core::crypto::AccountId32::from(hash);
            let ms_ss58 = multisig_account.to_string();
            println!("Multisig address: {}", ms_ss58);
            println!("  Threshold: {}/{}", threshold, addrs.len());
            println!("  Signatories:");
            for addr in &addrs {
                println!("    {}", addr);
            }
            Ok(())
        }
        MultisigCommands::Submit { others, threshold, pallet, call, args } => {
            let client = Client::connect(ws_url).await?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;

            let other_addrs: Vec<&str> = others.split(',').map(|s| s.trim()).collect();
            let mut other_ids: Vec<crate::AccountId> = other_addrs.iter()
                .map(|s| Client::ss58_to_account_id_pub(s))
                .collect::<Result<_>>()?;
            other_ids.sort();

            // Build the inner call dynamically
            let fields: Vec<subxt::dynamic::Value> = if let Some(ref args_json) = args {
                let parsed: Vec<serde_json::Value> = serde_json::from_str(args_json)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON args: {}", e))?;
                parsed.iter().map(json_to_subxt_value).collect()
            } else {
                vec![]
            };

            println!("Submitting multisig call: {}.{} (threshold {}/{})",
                pallet, call, threshold, other_ids.len() + 1);
            let hash = client.submit_multisig_call(
                wallet.coldkey()?,
                &other_ids,
                threshold,
                &pallet,
                &call,
                fields,
            ).await?;
            println!("Multisig call submitted. Tx: {}", hash);
            Ok(())
        }
        MultisigCommands::Approve { others, threshold, call_hash } => {
            let client = Client::connect(ws_url).await?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet)?;

            let other_addrs: Vec<&str> = others.split(',').map(|s| s.trim()).collect();
            let mut other_ids: Vec<crate::AccountId> = other_addrs.iter()
                .map(|s| Client::ss58_to_account_id_pub(s))
                .collect::<Result<_>>()?;
            other_ids.sort();

            let hash_hex = call_hash.strip_prefix("0x").unwrap_or(&call_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Call hash must be 32 bytes"))?;

            println!("Approving multisig call (threshold {}/{})", threshold, other_ids.len() + 1);
            let tx_hash = client.approve_multisig(
                wallet.coldkey()?,
                &other_ids,
                threshold,
                hash_bytes,
            ).await?;
            println!("Multisig approval submitted. Tx: {}", tx_hash);
            Ok(())
        }
    }
}

/// Convert a serde_json::Value to a subxt dynamic Value for multisig call args.
fn json_to_subxt_value(v: &serde_json::Value) -> subxt::dynamic::Value {
    use subxt::dynamic::Value;
    match v {
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Value::u128(u as u128)
            } else if let Some(i) = n.as_i64() {
                Value::i128(i as i128)
            } else {
                Value::string(n.to_string())
            }
        }
        serde_json::Value::String(s) => {
            if s.starts_with("0x") {
                if let Ok(bytes) = hex::decode(s.strip_prefix("0x").unwrap()) {
                    return Value::from_bytes(bytes);
                }
            }
            Value::string(s.clone())
        }
        serde_json::Value::Bool(b) => Value::bool(*b),
        serde_json::Value::Array(arr) => {
            Value::unnamed_composite(arr.iter().map(json_to_subxt_value))
        }
        _ => Value::string(v.to_string()),
    }
}

// ──────── Transaction History ────────

async fn handle_history(address: &str, output: &str, limit: usize) -> Result<()> {
    println!("Fetching transaction history for {}...", crate::utils::short_ss58(address));
    let url = "https://bittensor.api.subscan.io/api/v2/scan/extrinsics";
    let body = serde_json::json!({
        "address": address,
        "row": limit.min(100),
        "page": 0,
    });
    let client = reqwest::Client::new();
    let resp = client.post(url)
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
            if output == "json" {
                println!("{}", serde_json::to_string_pretty(&txs)?);
            } else if output == "csv" {
                println!("block,hash,module,call,success,timestamp");
                for tx in txs {
                    println!("{},{},{},{},{},{}",
                        tx.get("block_num").and_then(|v| v.as_u64()).unwrap_or(0),
                        tx.get("extrinsic_hash").and_then(|v| v.as_str()).unwrap_or(""),
                        tx.get("call_module").and_then(|v| v.as_str()).unwrap_or(""),
                        tx.get("call_module_function").and_then(|v| v.as_str()).unwrap_or(""),
                        tx.get("success").and_then(|v| v.as_bool()).unwrap_or(false),
                        tx.get("block_timestamp").and_then(|v| v.as_u64()).unwrap_or(0),
                    );
                }
            } else {
                println!("Recent transactions ({}):", txs.len());
                let mut table = comfy_table::Table::new();
                table.set_header(vec!["Block", "Module", "Call", "Success", "Hash"]);
                for tx in txs {
                    let block = tx.get("block_num").and_then(|v| v.as_u64()).unwrap_or(0);
                    let module = tx.get("call_module").and_then(|v| v.as_str()).unwrap_or("?");
                    let call = tx.get("call_module_function").and_then(|v| v.as_str()).unwrap_or("?");
                    let success = tx.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                    let hash = tx.get("extrinsic_hash").and_then(|v| v.as_str()).unwrap_or("?");
                    let hash_short = if hash.len() > 18 { &hash[..18] } else { hash };
                    table.add_row(vec![
                        format!("{}", block),
                        module.to_string(),
                        call.to_string(),
                        if success { "OK" } else { "FAIL" }.to_string(),
                        format!("{}...", hash_short),
                    ]);
                }
                println!("{table}");
            }
        }
        _ => {
            println!("No transactions found for {}", crate::utils::short_ss58(address));
            println!("Note: Subscan API may have rate limits or the address may have no activity.");
        }
    }
    Ok(())
}

// ──────── Shell Completions ────────

fn generate_completions(shell: &str) {
    use clap_complete::{generate, Shell};
    let mut cmd = Cli::command();
    let shell_enum = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        _ => {
            eprintln!("Unsupported shell: {}. Use: bash, zsh, fish, powershell", shell);
            return;
        }
    };
    generate(shell_enum, &mut cmd, "agcli", &mut std::io::stdout());
}

fn cfg_value_display(key: &str, cfg: &crate::config::Config) -> String {
    match key {
        "network" => cfg.network.clone().unwrap_or_default(),
        "endpoint" => cfg.endpoint.clone().unwrap_or_default(),
        "wallet_dir" => cfg.wallet_dir.clone().unwrap_or_default(),
        "wallet" => cfg.wallet.clone().unwrap_or_default(),
        "hotkey" => cfg.hotkey.clone().unwrap_or_default(),
        "output" => cfg.output.clone().unwrap_or_default(),
        "proxy" => cfg.proxy.clone().unwrap_or_default(),
        "live_interval" => cfg.live_interval.map(|v| v.to_string()).unwrap_or_default(),
        _ => String::new(),
    }
}

// ──────── Staking Wizard ────────

async fn staking_wizard(
    client: &Client,
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
) -> Result<()> {
    println!("=== Staking Wizard ===\n");

    // Step 1: Open wallet
    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    let coldkey_ss58 = wallet
        .coldkey_ss58()
        .map(|s| s.to_string())
        .unwrap_or_default();
    println!("Wallet: {} ({})", wallet_name, crate::utils::short_ss58(&coldkey_ss58));

    // Step 2: Show balance
    let balance = client.get_balance_ss58(&coldkey_ss58).await?;
    println!("Balance: {}\n", balance.display_tao());

    if balance.rao() == 0 {
        println!("You need TAO to stake. Transfer some TAO to your coldkey first.");
        return Ok(());
    }

    // Step 3: Show top subnets by TAO pool
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
            i + 1,
            d.netuid,
            &d.name,
            d.price,
            d.tao_in.tao(),
        );
    }

    // Step 4: Ask which subnet
    let netuid_input: String = dialoguer::Input::new()
        .with_prompt("\nEnter subnet netuid to stake on")
        .interact_text()?;
    let netuid: u16 = netuid_input.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid netuid"))?;

    // Step 5: Ask amount
    let max_tao = balance.tao();
    let amount_input: String = dialoguer::Input::new()
        .with_prompt(&format!("Amount of TAO to stake (max {:.4})", max_tao))
        .interact_text()?;
    let amount: f64 = amount_input.trim().parse()
        .map_err(|_| anyhow::anyhow!("Invalid amount"))?;

    if amount <= 0.0 || amount > max_tao {
        anyhow::bail!("Amount must be between 0 and {:.4}", max_tao);
    }

    // Step 6: Resolve hotkey
    let hotkey_ss58 = resolve_hotkey_ss58(None, &mut wallet, hotkey_name)?;
    println!("\nStaking {:.4} τ on SN{} with hotkey {}", amount, netuid, crate::utils::short_ss58(&hotkey_ss58));

    // Step 7: Confirm
    let confirm = dialoguer::Confirm::new()
        .with_prompt("Proceed?")
        .default(true)
        .interact()?;

    if !confirm {
        println!("Cancelled.");
        return Ok(());
    }

    // Step 8: Unlock and submit
    unlock_coldkey(&mut wallet)?;
    let stake_balance = Balance::from_tao(amount);
    let hash = client
        .add_stake(wallet.coldkey()?, &hotkey_ss58, NetUid(netuid), stake_balance)
        .await?;
    println!("Stake added! Tx: {}", hash);

    // Step 9: Show updated portfolio
    println!("\nUpdated portfolio:");
    let portfolio = crate::queries::portfolio::fetch_portfolio(client, &coldkey_ss58).await?;
    println!("  Free:   {}", portfolio.free_balance.display_tao());
    println!("  Staked: {}", portfolio.total_staked.display_tao());

    Ok(())
}
