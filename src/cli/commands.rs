//! CLI command execution — thin dispatcher to focused handler modules.

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use crate::types::Balance;
use anyhow::Result;

/// Execute the parsed CLI command.
pub async fn execute(cli: Cli) -> Result<()> {
    let network = cli.resolve_network();
    let output = cli.output.as_str();
    let live_interval = cli.live_interval();
    let password = cli.password.clone();
    let yes = cli.yes;
    let batch = cli.batch;
    let pretty = cli.pretty;
    let mev = cli.mev;

    // Set global mode flags so helpers can check them
    set_batch_mode(batch || yes);
    set_pretty_mode(pretty);

    match cli.command {
        Commands::Wallet(cmd) => {
            wallet_cmds::handle_wallet(
                cmd,
                &cli.wallet_dir,
                &cli.wallet,
                password.as_deref(),
                output,
            )
            .await
        }
        Commands::Balance {
            address,
            watch,
            threshold,
            at_block,
        } => {
            let client = Client::connect_network(&network).await?;
            let addr = resolve_coldkey_address(address, &cli.wallet_dir, &cli.wallet);

            // Historical wayback mode
            if let Some(block_num) = at_block {
                let block_hash = client.get_block_hash(block_num).await?;
                let balance = client.get_balance_at_block(&addr, block_hash).await?;
                if output == "json" {
                    print_json(
                        &serde_json::json!({"address": addr, "block": block_num, "block_hash": format!("{:?}", block_hash), "balance_rao": balance.rao(), "balance_tao": balance.tao()}),
                    );
                } else {
                    println!("Address: {}", addr);
                    println!("Block:   {} ({:?})", block_num, block_hash);
                    println!("Balance: {}", balance.display_tao());
                }
                return Ok(());
            }

            // Watch mode
            if let Some(interval_opt) = watch {
                let interval = interval_opt.unwrap_or(60);
                let threshold_rao = threshold.map(Balance::from_tao);
                eprintln!(
                    "Watching balance for {} (every {}s{})",
                    crate::utils::short_ss58(&addr),
                    interval,
                    threshold_rao
                        .as_ref()
                        .map(|t| format!(", alert below {}", t.display_tao()))
                        .unwrap_or_default()
                );
                loop {
                    let balance = match client.get_balance_ss58(&addr).await {
                        Ok(b) => b,
                        Err(e) => {
                            eprintln!("[{}] Warning: balance query failed: {}", chrono::Local::now().format("%H:%M:%S"), e);
                            tokio::select! {
                                _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval)) => {},
                                _ = tokio::signal::ctrl_c() => {
                                    println!("\nStopping balance watch (received Ctrl+C)");
                                    return Ok(());
                                }
                            }
                            continue;
                        }
                    };
                    let below = threshold_rao
                        .as_ref()
                        .map(|t| balance.rao() < t.rao())
                        .unwrap_or(false);
                    if output == "json" {
                        print_json(&serde_json::json!({
                            "address": addr,
                            "balance_rao": balance.rao(),
                            "balance_tao": balance.tao(),
                            "below_threshold": below,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }));
                    } else {
                        let alert = if below {
                            " *** BELOW THRESHOLD ***"
                        } else {
                            ""
                        };
                        println!(
                            "[{}] {} — {}{}",
                            chrono::Local::now().format("%H:%M:%S"),
                            crate::utils::short_ss58(&addr),
                            balance.display_tao(),
                            alert
                        );
                    }
                    tokio::select! {
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(interval)) => {},
                        _ = tokio::signal::ctrl_c() => {
                            println!("\nStopping balance watch (received Ctrl+C)");
                            return Ok(());
                        }
                    }
                }
            }

            let balance = client.get_balance_ss58(&addr).await?;
            if output == "json" {
                print_json(
                    &serde_json::json!({"address": addr, "balance_rao": balance.rao(), "balance_tao": balance.tao()}),
                );
            } else {
                println!("Address: {}", addr);
                println!("Balance: {}", balance.display_tao());
            }
            Ok(())
        }
        Commands::Transfer { dest, amount } => {
            let client = Client::connect_network(&network).await?;
            let mut wallet = open_wallet(&cli.wallet_dir, &cli.wallet)?;
            unlock_coldkey(&mut wallet, password.as_deref())?;
            let balance = Balance::from_tao(amount);
            // Pre-flight balance check
            if let Some(ss58) = wallet.coldkey_ss58() {
                let current = client.get_balance_ss58(ss58).await?;
                if current.rao() < balance.rao() {
                    anyhow::bail!(
                        "Insufficient balance: you have {} but trying to transfer {}.",
                        current.display_tao(),
                        balance.display_tao()
                    );
                }
            }
            println!("Transferring {} to {}", balance.display_tao(), dest);
            let hash = client.transfer(wallet.coldkey()?, &dest, balance).await?;
            print_tx_result(output, &hash, "Transaction submitted.");
            Ok(())
        }
        Commands::TransferAll { dest, keep_alive } => {
            let client = Client::connect_network(&network).await?;
            let mut wallet = open_wallet(&cli.wallet_dir, &cli.wallet)?;
            unlock_coldkey(&mut wallet, password.as_deref())?;
            println!(
                "Transferring all balance to {} (keep_alive={})",
                dest, keep_alive
            );
            let hash = client
                .transfer_all(wallet.coldkey()?, &dest, keep_alive)
                .await?;
            print_tx_result(output, &hash, "All balance transferred.");
            Ok(())
        }
        Commands::Stake(cmd) => {
            let client = Client::connect_network(&network).await?;
            stake_cmds::handle_stake(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                output,
                password.as_deref(),
                yes,
                mev,
            )
            .await
        }
        Commands::Subnet(cmd) => {
            let client = Client::connect_network(&network).await?;
            subnet_cmds::handle_subnet(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                output,
                live_interval,
                password.as_deref(),
            )
            .await
        }
        Commands::Weights(cmd) => {
            let client = Client::connect_network(&network).await?;
            weights_cmds::handle_weights(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                password.as_deref(),
            )
            .await
        }
        Commands::Root(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_root(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                password.as_deref(),
            )
            .await
        }
        Commands::Delegate(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_delegate(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                output,
                password.as_deref(),
            )
            .await
        }
        Commands::View(cmd) => {
            let client = Client::connect_network(&network).await?;
            view_cmds::handle_view(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                output,
                live_interval,
            )
            .await
        }
        Commands::Identity(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_identity(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                password.as_deref(),
            )
            .await
        }
        Commands::Serve(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_serve(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                password.as_deref(),
            )
            .await
        }
        Commands::Proxy(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_proxy(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                output,
                password.as_deref(),
            )
            .await
        }
        Commands::Crowdloan(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_crowdloan(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                output,
                password.as_deref(),
            )
            .await
        }
        Commands::Liquidity(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_liquidity(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                &cli.hotkey,
                output,
                password.as_deref(),
            )
            .await
        }
        Commands::Swap(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_swap(
                cmd,
                &client,
                &cli.wallet_dir,
                &cli.wallet,
                password.as_deref(),
            )
            .await
        }
        Commands::Subscribe(cmd) => {
            let client = Client::connect_network(&network).await?;
            network_cmds::handle_subscribe(cmd, &client, output, batch).await
        }
        Commands::Multisig(cmd) => {
            network_cmds::handle_multisig(
                cmd,
                &cli.wallet_dir,
                &cli.wallet,
                &network,
                password.as_deref(),
            )
            .await
        }
        Commands::Config(cmd) => system_cmds::handle_config(cmd).await,
        Commands::Completions { shell } => {
            system_cmds::generate_completions(&shell);
            Ok(())
        }
        Commands::Update => system_cmds::handle_update().await,
        Commands::Doctor => {
            system_cmds::handle_doctor(&network, &cli.wallet_dir, &cli.wallet, output).await
        }
        Commands::Explain { topic } => {
            system_cmds::handle_explain(topic.as_deref(), output)
        }
        Commands::Audit { address } => {
            let client = Client::connect_network(&network).await?;
            let addr = resolve_coldkey_address(address, &cli.wallet_dir, &cli.wallet);
            view_cmds::handle_audit(&client, &addr, output).await
        }
        Commands::Block(cmd) => {
            let client = Client::connect_network(&network).await?;
            block_cmds::handle_block(cmd, &client, output).await
        }
        Commands::Diff(cmd) => {
            let client = Client::connect_network(&network).await?;
            block_cmds::handle_diff(cmd, &client, output, &cli.wallet_dir, &cli.wallet).await
        }
        Commands::Batch { file, no_atomic } => {
            let client = Client::connect_network(&network).await?;
            let mut wallet = open_wallet(&cli.wallet_dir, &cli.wallet)?;
            unlock_coldkey(&mut wallet, password.as_deref())?;
            system_cmds::handle_batch(&client, wallet.coldkey()?, &file, no_atomic, output).await
        }
    }
}
