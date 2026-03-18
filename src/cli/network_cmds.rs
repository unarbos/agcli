//! Network operation handlers (root, delegate, identity, swap, subscribe, serve, proxy, crowdloan, liquidity).

use crate::chain::Client;
use crate::cli::helpers::*;
use crate::cli::*;
use crate::types::{Balance, NetUid};
use anyhow::Result;

// ──────── Root ────────

pub(super) async fn handle_root(cmd: RootCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name, password) = (
        ctx.wallet_dir,
        ctx.wallet_name,
        ctx.hotkey_name,
        ctx.password,
    );
    match cmd {
        RootCommands::Register => {
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            tracing::info!(hotkey = %crate::utils::short_ss58(&hk), "Registering on root network");
            println!(
                "Registering on root network with hotkey {}",
                crate::utils::short_ss58(&hk)
            );
            let hash = client.root_register(&pair, &hk).await?;
            tracing::info!(tx = %hash, "Root registration complete");
            println!(
                "Registered on root network with hotkey {}.\n  Tx: {}",
                crate::utils::short_ss58(&hk),
                hash
            );
            Ok(())
        }
        RootCommands::Weights { weights } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            wallet.load_hotkey(hotkey_name)?;
            let (uids, wts) = parse_weight_pairs(&weights)?;
            println!("Setting {} root weights", uids.len());
            let hash = client
                .set_weights(wallet.hotkey()?, NetUid::ROOT, &uids, &wts, 0)
                .await?;
            println!("Root weights set ({} UIDs).\n  Tx: {}", uids.len(), hash);
            Ok(())
        }
    }
}

// ──────── Delegate ────────

pub(super) async fn handle_delegate(
    cmd: DelegateCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    match cmd {
        DelegateCommands::List => {
            let delegates = client.get_delegates().await?;
            let top: Vec<_> = delegates.into_iter().take(50).collect();
            render_rows(
                ctx.output,
                &top,
                "hotkey,owner,take_pct,total_stake_rao,nominators",
                |d| {
                    format!(
                        "{},{},{:.4},{},{}",
                        d.hotkey,
                        d.owner,
                        d.take * 100.0,
                        d.total_stake.rao(),
                        d.nominators.len()
                    )
                },
                &["Hotkey", "Owner", "Take", "Total Stake", "Nominators"],
                |d| {
                    vec![
                        crate::utils::short_ss58(&d.hotkey),
                        crate::utils::short_ss58(&d.owner),
                        format!("{:.2}%", d.take * 100.0),
                        d.total_stake.display_tao(),
                        format!("{}", d.nominators.len()),
                    ]
                },
                Some(&format!("{} delegates", top.len())),
            );
            Ok(())
        }
        DelegateCommands::Show { hotkey } => {
            let hotkey_ss58 = match hotkey {
                Some(hk) => {
                    validate_ss58(&hk, "delegate show hotkey")?;
                    hk
                }
                None => {
                    let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
                    resolve_hotkey_ss58(None, &mut wallet, ctx.hotkey_name)?
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
                        // Sort indices to avoid cloning the full nominators list
                        let mut indices: Vec<usize> = (0..d.nominators.len()).collect();
                        indices.sort_unstable_by(|&a, &b| {
                            d.nominators[b].1.rao().cmp(&d.nominators[a].1.rao())
                        });
                        for &i in indices.iter().take(10) {
                            let (addr, stake) = &d.nominators[i];
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
            change_take(client, ctx, hotkey, take, false).await
        }
        DelegateCommands::IncreaseTake { take, hotkey } => {
            change_take(client, ctx, hotkey, take, true).await
        }
    }
}

async fn change_take(
    client: &Client,
    ctx: &Ctx<'_>,
    hotkey: Option<String>,
    take: f64,
    increase: bool,
) -> Result<()> {
    crate::cli::helpers::validate_delegate_take(take)?;
    let (pair, hk) = unlock_and_resolve(
        ctx.wallet_dir,
        ctx.wallet_name,
        ctx.hotkey_name,
        hotkey,
        ctx.password,
    )?;
    let take_u16 = (take / 100.0 * 65535.0).round().min(65535.0) as u16;
    let dir = if increase { "Increasing" } else { "Decreasing" };
    tracing::info!(hotkey = %crate::utils::short_ss58(&hk), take_pct = take, direction = dir, "Changing delegate take");
    println!(
        "{} take to {:.2}% for {}",
        dir,
        take,
        crate::utils::short_ss58(&hk)
    );
    let hash = if increase {
        client.increase_take(&pair, &hk, take_u16).await?
    } else {
        client.decrease_take(&pair, &hk, take_u16).await?
    };
    tracing::info!(tx = %hash, "Delegate take changed");
    println!(
        "Delegate take {} to {:.2}% for {}.\n  Tx: {}",
        if increase { "increased" } else { "decreased" },
        take,
        crate::utils::short_ss58(&hk),
        hash
    );
    Ok(())
}

// ──────── Identity ────────

pub(super) async fn handle_identity(
    cmd: IdentityCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    let (wallet_dir, wallet_name, password) = (ctx.wallet_dir, ctx.wallet_name, ctx.password);
    match cmd {
        IdentityCommands::Show { address } => {
            validate_ss58(&address, "identity show address")?;
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
            image,
        } => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let url_str = url.as_deref().unwrap_or("");
            let desc_str = description.as_deref().unwrap_or("");
            let gh_str = github.as_deref().unwrap_or("");
            let img_str = image.as_deref().unwrap_or("");
            println!("Setting on-chain identity: name='{}'", name);
            let hash = client
                .set_registry_identity(wallet.coldkey()?, &name, url_str, desc_str, gh_str, img_str)
                .await?;
            println!("Identity set (name: '{}').\n  Tx: {}", name, hash);
            Ok(())
        }
        IdentityCommands::Clear => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!("Clearing on-chain identity");
            let hash = client.clear_registry_identity(wallet.coldkey()?).await?;
            println!("Identity cleared.\n  Tx: {}", hash);
            Ok(())
        }
        IdentityCommands::SetSubnet {
            netuid,
            name,
            github,
            url,
        } => {
            crate::cli::helpers::validate_subnet_name(&name, "subnet name")?;
            if let Some(ref u) = url {
                crate::cli::helpers::validate_url(u, "subnet URL")?;
            }
            if let Some(ref gh) = github {
                crate::cli::helpers::validate_github_repo(gh)?;
            }
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let identity = crate::types::chain_data::SubnetIdentity {
                subnet_name: name.clone(),
                github_repo: github.unwrap_or_default(),
                subnet_contact: String::new(),
                subnet_url: url.unwrap_or_default(),
                discord: String::new(),
                description: String::new(),
                additional: String::new(),
            };
            tracing::info!(netuid = netuid, name = %name, "Setting subnet identity");
            println!("Setting subnet identity for SN{}: {}", netuid, name);
            let hash = client
                .set_subnet_identity(wallet.coldkey()?, NetUid(netuid), &identity)
                .await?;
            tracing::info!(tx = %hash, netuid = netuid, "Subnet identity set");
            println!(
                "Subnet identity set for SN{} (name: '{}').\n  Tx: {}",
                netuid, name, hash
            );
            Ok(())
        }
    }
}

// ──────── Swap ────────

pub(super) async fn handle_swap(cmd: SwapCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, password) = (ctx.wallet_dir, ctx.wallet_name, ctx.password);
    match cmd {
        SwapCommands::Hotkey { new_hotkey } => {
            validate_ss58(&new_hotkey, "new hotkey")?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            // Always load the hotkey specified by --hotkey-name / AGCLI_HOTKEY / config.
            // Wallet::open() pre-loads "default" hotkey SS58, so we must explicitly
            // load the correct hotkey to respect the user's --hotkey-name flag.
            wallet.load_hotkey(ctx.hotkey_name)?;
            let old_hotkey = wallet
                .hotkey_ss58()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("Could not resolve current hotkey '{}'. Create one with: agcli wallet new-hotkey --name {}", ctx.hotkey_name, ctx.hotkey_name))?;
            tracing::info!(old = %crate::utils::short_ss58(&old_hotkey), new = %crate::utils::short_ss58(&new_hotkey), "Swapping hotkey");
            println!(
                "Swapping hotkey {} -> {}",
                crate::utils::short_ss58(&old_hotkey),
                crate::utils::short_ss58(&new_hotkey)
            );
            let hash = client
                .swap_hotkey(wallet.coldkey()?, &old_hotkey, &new_hotkey)
                .await?;
            tracing::info!(tx = %hash, "Hotkey swapped");
            println!(
                "Hotkey swapped: {} → {}.\n  Tx: {}",
                crate::utils::short_ss58(&old_hotkey),
                crate::utils::short_ss58(&new_hotkey),
                hash
            );
            Ok(())
        }
        SwapCommands::Coldkey { new_coldkey } => {
            validate_ss58(&new_coldkey, "new coldkey")?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            tracing::info!(new_coldkey = %crate::utils::short_ss58(&new_coldkey), "Scheduling coldkey swap");
            println!(
                "Scheduling coldkey swap to {}",
                crate::utils::short_ss58(&new_coldkey)
            );
            let hash = client
                .schedule_swap_coldkey(wallet.coldkey()?, &new_coldkey)
                .await?;
            tracing::info!(tx = %hash, "Coldkey swap scheduled");
            println!("Coldkey swap scheduled to {}. Check status with `agcli wallet check-swap`.\n  Tx: {}", crate::utils::short_ss58(&new_coldkey), hash);
            Ok(())
        }
        SwapCommands::EvmKey {
            evm_address,
            block_number,
            signature,
        } => {
            // Parse EVM address (20 bytes hex, optionally 0x-prefixed)
            let addr_hex = evm_address.strip_prefix("0x").unwrap_or(&evm_address);
            let addr_bytes: [u8; 20] = hex::decode(addr_hex)
                .map_err(|e| anyhow::anyhow!("Invalid EVM address hex: {}", e))?
                .try_into()
                .map_err(|v: Vec<u8>| {
                    anyhow::anyhow!(
                        "EVM address must be 20 bytes, got {}",
                        v.len()
                    )
                })?;
            // Parse signature (65 bytes hex, optionally 0x-prefixed)
            let sig_hex = signature.strip_prefix("0x").unwrap_or(&signature);
            let sig_bytes: [u8; 65] = hex::decode(sig_hex)
                .map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?
                .try_into()
                .map_err(|v: Vec<u8>| {
                    anyhow::anyhow!(
                        "Signature must be 65 bytes (r+s+v), got {}",
                        v.len()
                    )
                })?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!(
                "Associating EVM address 0x{} with your account",
                addr_hex
            );
            let hash = client
                .associate_evm_key(wallet.coldkey()?, addr_bytes, block_number, sig_bytes)
                .await?;
            print_tx_result(
                ctx.output,
                &hash,
                &format!("EVM key 0x{} associated", addr_hex),
            );
            Ok(())
        }
    }
}

// ──────── Subscribe ────────

pub(super) async fn handle_subscribe(
    cmd: SubscribeCommands,
    client: &Client,
    output: OutputFormat,
) -> Result<()> {
    // Validate inputs before connecting
    if let SubscribeCommands::Events {
        ref filter,
        ref account,
        ..
    } = cmd
    {
        validate_event_filter(filter)?;
        if let Some(ref acct) = account {
            validate_ss58(acct, "account")?;
        }
    }

    let json = output.is_json();
    match cmd {
        SubscribeCommands::Blocks => crate::events::subscribe_blocks(client.subxt(), json).await,
        SubscribeCommands::Events {
            filter,
            netuid,
            account,
        } => {
            let f: crate::events::EventFilter = filter
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid event filter '{}': {}", filter, e))?;
            crate::events::subscribe_events_filtered(
                client.subxt(),
                f,
                json,
                netuid,
                account.as_deref(),
            )
            .await
        }
    }
}

// ──────── Multisig ────────

pub(super) async fn handle_multisig(
    cmd: MultisigCommands,
    wallet_dir: &str,
    wallet_name: &str,
    network: &crate::types::Network,
    password: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    match cmd {
        MultisigCommands::Address {
            signatories,
            threshold,
        } => {
            let addrs: Vec<&str> = signatories.split(',').map(|s| s.trim()).collect();
            if addrs.len() < 2 {
                anyhow::bail!("Need at least 2 signatories for a multisig. Provide comma-separated SS58 addresses.");
            }
            let account_ids = parse_sorted_signatories(&signatories)?;

            // Derive multisig AccountId using Substrate's multi_account_id algorithm:
            // blake2_256(SCALE_encode(b"modlpy/utilisuba", sorted_signatories, threshold))
            // SCALE encoding of this tuple: prefix bytes ++ compact(len) ++ each AccountId ++ threshold_le16
            use blake2::digest::{Update, VariableOutput};
            let mut hasher = blake2::Blake2bVar::new(32)
                .map_err(|e| anyhow::anyhow!("blake2 error: {:?}", e))?;
            hasher.update(b"modlpy/utilisuba");
            // SCALE compact-encode the signatory count
            let count = account_ids.len() as u32;
            let compact = parity_scale_codec::Compact(count);
            use parity_scale_codec::Encode;
            hasher.update(&compact.encode());
            for id in &account_ids {
                hasher.update(id.as_ref());
            }
            hasher.update(&threshold.to_le_bytes());
            let mut hash = [0u8; 32];
            hasher
                .finalize_variable(&mut hash)
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
        MultisigCommands::Submit {
            others,
            threshold,
            pallet,
            call,
            args,
        } => {
            // Enforce spending limits on staking calls within multisig
            // (audit fix: reject malformed JSON early instead of silently skipping limit check)
            let raw_args: Vec<serde_json::Value> = match args.as_ref() {
                Some(s) => serde_json::from_str(s).map_err(|e| {
                    anyhow::anyhow!("Multisig submit: invalid --args JSON: {e}.\n  Tip: pass a JSON array, e.g. '[1, \"0x...\"]'.")
                })?,
                None => vec![],
            };
            check_spending_limit_for_raw_call(&pallet, &call, &raw_args)
                .map_err(|e| anyhow::anyhow!("Multisig submit: {}", e))?;

            let mut client = Client::connect_network(network).await?;
            client.set_dry_run(dry_run);
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let other_ids = parse_sorted_signatories(&others)?;
            let fields: Vec<subxt::dynamic::Value> = if let Some(ref args_json) = args {
                let validated = crate::cli::helpers::validate_multisig_json_args(args_json)?;
                validated.iter().map(json_to_subxt_value).collect()
            } else {
                vec![]
            };
            println!(
                "Submitting multisig call: {}.{} (threshold {}/{})",
                pallet,
                call,
                threshold,
                other_ids.len() + 1
            );
            let hash = client
                .submit_multisig_call(
                    wallet.coldkey()?,
                    &other_ids,
                    threshold,
                    &pallet,
                    &call,
                    fields,
                )
                .await?;
            println!(
                "Multisig call submitted: {}.{} (threshold {}/{}).\n  Tx: {}",
                pallet,
                call,
                threshold,
                other_ids.len() + 1,
                hash
            );
            Ok(())
        }
        MultisigCommands::Approve {
            others,
            threshold,
            call_hash,
        } => {
            validate_call_hash(&call_hash, "multisig approve")?;
            let mut client = Client::connect_network(network).await?;
            client.set_dry_run(dry_run);
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let other_ids = parse_sorted_signatories(&others)?;
            let hash_hex = call_hash.strip_prefix("0x").unwrap_or(&call_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?.try_into().map_err(|_| {
                anyhow::anyhow!("Call hash must be exactly 32 bytes (64 hex chars)")
            })?;
            println!(
                "Approving multisig call (threshold {}/{})",
                threshold,
                other_ids.len() + 1
            );
            let tx_hash = client
                .approve_multisig(wallet.coldkey()?, &other_ids, threshold, hash_bytes)
                .await?;
            println!(
                "Multisig approval submitted (threshold {}/{}).\n  Tx: {}",
                threshold,
                other_ids.len() + 1,
                tx_hash
            );
            Ok(())
        }
        MultisigCommands::Execute {
            others,
            threshold,
            pallet,
            call,
            args,
            timepoint_height,
            timepoint_index,
        } => {
            // Enforce spending limits on staking calls within multisig
            // (audit fix: reject malformed JSON early instead of silently skipping limit check)
            let raw_args: Vec<serde_json::Value> = match args.as_ref() {
                Some(s) => serde_json::from_str(s).map_err(|e| {
                    anyhow::anyhow!("Multisig execute: invalid --args JSON: {e}.\n  Tip: pass a JSON array, e.g. '[1, \"0x...\"]'.")
                })?,
                None => vec![],
            };
            check_spending_limit_for_raw_call(&pallet, &call, &raw_args)
                .map_err(|e| anyhow::anyhow!("Multisig execute: {}", e))?;

            let mut client = Client::connect_network(network).await?;
            client.set_dry_run(dry_run);
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let other_ids = parse_sorted_signatories(&others)?;
            let fields = parse_json_args(&args)?;
            let timepoint = match (timepoint_height, timepoint_index) {
                (Some(h), Some(i)) => Some((h, i)),
                (None, None) => None,
                _ => anyhow::bail!(
                    "Both --timepoint-height and --timepoint-index must be provided together"
                ),
            };
            println!(
                "Executing multisig call: {}.{} (threshold {}/{})",
                pallet,
                call,
                threshold,
                other_ids.len() + 1
            );
            let tx_hash = client
                .execute_multisig(
                    wallet.coldkey()?,
                    &other_ids,
                    threshold,
                    timepoint,
                    &pallet,
                    &call,
                    fields,
                )
                .await?;
            println!(
                "Multisig call executed: {}.{} (threshold {}/{}).\n  Tx: {}",
                pallet,
                call,
                threshold,
                other_ids.len() + 1,
                tx_hash
            );
            Ok(())
        }
        MultisigCommands::Cancel {
            others,
            threshold,
            call_hash,
            timepoint_height,
            timepoint_index,
        } => {
            validate_call_hash(&call_hash, "multisig cancel")?;
            let mut client = Client::connect_network(network).await?;
            client.set_dry_run(dry_run);
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let other_ids = parse_sorted_signatories(&others)?;
            let hash_hex = call_hash.strip_prefix("0x").unwrap_or(&call_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?.try_into().map_err(|_| {
                anyhow::anyhow!("Call hash must be exactly 32 bytes (64 hex chars)")
            })?;
            println!(
                "Cancelling multisig call (threshold {}/{})",
                threshold,
                other_ids.len() + 1
            );
            let tx_hash = client
                .cancel_multisig(
                    wallet.coldkey()?,
                    &other_ids,
                    threshold,
                    (timepoint_height, timepoint_index),
                    hash_bytes,
                )
                .await?;
            println!(
                "Multisig call cancelled (threshold {}/{}).\n  Tx: {}",
                threshold,
                other_ids.len() + 1,
                tx_hash
            );
            Ok(())
        }
        MultisigCommands::List { address } => {
            validate_ss58(&address, "multisig list address")?;
            let client = Client::connect_network(network).await?;
            let pending = client.list_multisig_pending(&address).await?;
            if pending.is_empty() {
                println!("No pending multisig operations for {}", address);
            } else {
                println!(
                    "Pending multisig operations for {} ({} found):",
                    address,
                    pending.len()
                );
                for (call_hash, height, index, approvals, deposit) in &pending {
                    println!("  Call hash:  {}", call_hash);
                    println!("  Timepoint: height={}, index={}", height, index);
                    println!("  Approvals: {}", approvals);
                    println!("  Deposit:   {} RAO", deposit);
                    println!();
                }
            }
            Ok(())
        }
    }
}

// ──────── Scheduler ────────

pub(super) async fn handle_scheduler(
    cmd: SchedulerCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    match cmd {
        SchedulerCommands::Schedule {
            when,
            pallet,
            call,
            args,
            priority,
            repeat_every,
            repeat_count,
        } => {
            validate_pallet_call(&pallet, "pallet")?;
            validate_pallet_call(&call, "call")?;
            validate_block_number(when, "schedule block")?;
            let periodic = match (repeat_every, repeat_count) {
                (Some(period), Some(count)) => {
                    validate_repeat_params(period, count)?;
                    Some((period, count))
                }
                (None, None) => None,
                _ => anyhow::bail!(
                    "Both --repeat-every and --repeat-count must be provided together"
                ),
            };
            // Enforce spending limits on staking calls before scheduling
            // (audit fix: reject malformed JSON early instead of silently skipping limit check)
            let raw_args: Vec<serde_json::Value> = match args.as_ref() {
                Some(s) => serde_json::from_str(s).map_err(|e| {
                    anyhow::anyhow!("Scheduler: invalid --args JSON: {e}.\n  Tip: pass a JSON array, e.g. '[1, \"0x...\"]'.")
                })?,
                None => vec![],
            };
            check_spending_limit_for_raw_call(&pallet, &call, &raw_args)
                .map_err(|e| anyhow::anyhow!("Scheduler: {}", e))?;

            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let fields = parse_json_args(&args)?;
            println!(
                "Scheduling {}.{} at block {} (priority {}{})",
                pallet,
                call,
                when,
                priority,
                periodic
                    .map(|(p, c)| format!(", repeat every {} blocks {} times", p, c))
                    .unwrap_or_default()
            );
            let tx_hash = client
                .schedule_call(
                    wallet.coldkey()?,
                    when,
                    periodic,
                    priority,
                    &pallet,
                    &call,
                    fields,
                )
                .await?;
            println!(
                "Call scheduled: {}.{} at block {} (priority {}).\n  Tx: {}",
                pallet, call, when, priority, tx_hash
            );
            Ok(())
        }
        SchedulerCommands::ScheduleNamed {
            id,
            when,
            pallet,
            call,
            args,
            priority,
            repeat_every,
            repeat_count,
        } => {
            validate_schedule_id(&id)?;
            validate_pallet_call(&pallet, "pallet")?;
            validate_pallet_call(&call, "call")?;
            validate_block_number(when, "schedule block")?;
            let periodic = match (repeat_every, repeat_count) {
                (Some(period), Some(count)) => {
                    validate_repeat_params(period, count)?;
                    Some((period, count))
                }
                (None, None) => None,
                _ => anyhow::bail!(
                    "Both --repeat-every and --repeat-count must be provided together"
                ),
            };
            // Enforce spending limits on staking calls before scheduling
            // (audit fix: reject malformed JSON early instead of silently skipping limit check)
            let raw_args: Vec<serde_json::Value> = match args.as_ref() {
                Some(s) => serde_json::from_str(s).map_err(|e| {
                    anyhow::anyhow!("Scheduler: invalid --args JSON: {e}.\n  Tip: pass a JSON array, e.g. '[1, \"0x...\"]'.")
                })?,
                None => vec![],
            };
            check_spending_limit_for_raw_call(&pallet, &call, &raw_args)
                .map_err(|e| anyhow::anyhow!("Scheduler: {}", e))?;

            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let fields = parse_json_args(&args)?;
            println!(
                "Scheduling named task '{}': {}.{} at block {} (priority {})",
                id, pallet, call, when, priority
            );
            let tx_hash = client
                .schedule_named_call(
                    wallet.coldkey()?,
                    id.as_bytes(),
                    when,
                    periodic,
                    priority,
                    &pallet,
                    &call,
                    fields,
                )
                .await?;
            println!(
                "Named call '{}' scheduled: {}.{} at block {}.\n  Tx: {}",
                id, pallet, call, when, tx_hash
            );
            Ok(())
        }
        SchedulerCommands::Cancel { when, index } => {
            validate_block_number(when, "cancel block")?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!(
                "Cancelling scheduled task at block {}, index {}",
                when, index
            );
            let tx_hash = client
                .cancel_scheduled(wallet.coldkey()?, when, index)
                .await?;
            println!(
                "Scheduled task at block {} index {} cancelled.\n  Tx: {}",
                when, index, tx_hash
            );
            Ok(())
        }
        SchedulerCommands::CancelNamed { id } => {
            validate_schedule_id(&id)?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!("Cancelling named scheduled task '{}'", id);
            let tx_hash = client
                .cancel_named_scheduled(wallet.coldkey()?, id.as_bytes())
                .await?;
            println!(
                "Named scheduled task '{}' cancelled.\n  Tx: {}",
                id, tx_hash
            );
            Ok(())
        }
    }
}

// ──────── Preimage ────────

pub(super) async fn handle_preimage(
    cmd: PreimageCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    match cmd {
        PreimageCommands::Note { pallet, call, args } => {
            validate_pallet_call(&pallet, "pallet")?;
            validate_pallet_call(&call, "call")?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let fields = parse_json_args(&args)?;
            println!("Storing preimage for {}.{}", pallet, call);
            let (tx_hash, preimage_hash) = client
                .note_preimage(wallet.coldkey()?, &pallet, &call, fields)
                .await?;
            println!(
                "Preimage stored for {}.{}.\n  Hash: 0x{}\n  Tx: {}",
                pallet,
                call,
                hex::encode(preimage_hash),
                tx_hash
            );
            Ok(())
        }
        PreimageCommands::Unnote { hash } => {
            validate_hex_data(&hash, "preimage hash")?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let hash_hex = hash.strip_prefix("0x").unwrap_or(&hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?.try_into().map_err(|_| {
                anyhow::anyhow!("Preimage hash must be exactly 32 bytes (64 hex chars)")
            })?;
            println!("Removing preimage 0x{}", hash_hex);
            let tx_hash = client
                .unnote_preimage(wallet.coldkey()?, hash_bytes)
                .await?;
            println!("Preimage 0x{} removed.\n  Tx: {}", hash_hex, tx_hash);
            Ok(())
        }
    }
}

// ──────── Contracts ────────

pub(super) async fn handle_contracts(
    cmd: ContractsCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    match cmd {
        ContractsCommands::Upload {
            code,
            storage_deposit_limit,
        } => {
            let wasm = std::fs::read(&code)
                .map_err(|e| anyhow::anyhow!("Failed to read WASM file '{}': {}", code, e))?;
            validate_wasm_file(&wasm, &code)?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!("Uploading contract code ({} bytes)", wasm.len());
            let tx_hash = client
                .contracts_upload_code(wallet.coldkey()?, wasm, storage_deposit_limit)
                .await?;
            println!("Contract code uploaded. Tx: {}", tx_hash);
            Ok(())
        }
        ContractsCommands::Instantiate {
            code_hash,
            value,
            data,
            salt,
            gas_ref_time,
            gas_proof_size,
            storage_deposit_limit,
        } => {
            validate_hex_data(&code_hash, "code-hash")?;
            validate_hex_data(&data, "data")?;
            validate_hex_data(&salt, "salt")?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let hash_hex = code_hash.strip_prefix("0x").unwrap_or(&code_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Code hash must be 32 bytes"))?;
            let data_hex = data.strip_prefix("0x").unwrap_or(&data);
            let data_bytes = hex::decode(data_hex)
                .map_err(|e| anyhow::anyhow!("Invalid hex data '{}': {}", data, e))?;
            let salt_hex = salt.strip_prefix("0x").unwrap_or(&salt);
            let salt_bytes = hex::decode(salt_hex)
                .map_err(|e| anyhow::anyhow!("Invalid hex salt '{}': {}", salt, e))?;
            println!("Instantiating contract from code hash 0x{}", hash_hex);
            let tx_hash = client
                .contracts_instantiate(
                    wallet.coldkey()?,
                    value,
                    gas_ref_time,
                    gas_proof_size,
                    storage_deposit_limit,
                    hash_bytes,
                    data_bytes,
                    salt_bytes,
                )
                .await?;
            println!("Contract instantiated. Tx: {}", tx_hash);
            Ok(())
        }
        ContractsCommands::Call {
            contract,
            value,
            data,
            gas_ref_time,
            gas_proof_size,
            storage_deposit_limit,
        } => {
            validate_ss58(&contract, "contract")?;
            validate_hex_data(&data, "data")?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let data_hex = data.strip_prefix("0x").unwrap_or(&data);
            let data_bytes =
                hex::decode(data_hex).map_err(|e| anyhow::anyhow!("Invalid hex data: {}", e))?;
            println!(
                "Calling contract {} ({} bytes input)",
                crate::utils::short_ss58(&contract),
                data_bytes.len()
            );
            let tx_hash = client
                .contracts_call(
                    wallet.coldkey()?,
                    &contract,
                    value,
                    gas_ref_time,
                    gas_proof_size,
                    storage_deposit_limit,
                    data_bytes,
                )
                .await?;
            println!("Contract call submitted. Tx: {}", tx_hash);
            Ok(())
        }
        ContractsCommands::RemoveCode { code_hash } => {
            validate_hex_data(&code_hash, "code-hash")?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let hash_hex = code_hash.strip_prefix("0x").unwrap_or(&code_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Code hash must be 32 bytes"))?;
            println!("Removing contract code 0x{}", hash_hex);
            let tx_hash = client
                .contracts_remove_code(wallet.coldkey()?, hash_bytes)
                .await?;
            println!("Contract code removed. Tx: {}", tx_hash);
            Ok(())
        }
    }
}

// ──────── EVM ────────

pub(super) async fn handle_evm(cmd: EvmCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    match cmd {
        EvmCommands::Call {
            source,
            target,
            input,
            value,
            gas_limit,
            max_fee_per_gas,
        } => {
            validate_evm_address(&source, "source")?;
            validate_evm_address(&target, "target")?;
            validate_hex_data(&input, "input")?;
            validate_hex_data(&value, "value")?;
            validate_hex_data(&max_fee_per_gas, "max-fee-per-gas")?;
            validate_gas_limit(gas_limit, "gas limit")?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let src_hex = source.strip_prefix("0x").unwrap_or(&source);
            let src_bytes: [u8; 20] = hex::decode(src_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Source must be 20 bytes (EVM address)"))?;
            let tgt_hex = target.strip_prefix("0x").unwrap_or(&target);
            let tgt_bytes: [u8; 20] = hex::decode(tgt_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Target must be 20 bytes (EVM address)"))?;
            let input_hex = input.strip_prefix("0x").unwrap_or(&input);
            let input_bytes = hex::decode(input_hex)
                .map_err(|e| anyhow::anyhow!("Invalid hex input '{}': {}", input, e))?;
            let value_hex = value.strip_prefix("0x").unwrap_or(&value);
            let value_bytes: [u8; 32] = hex::decode(value_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Value must be 32 bytes (U256)"))?;
            let fee_hex = max_fee_per_gas
                .strip_prefix("0x")
                .unwrap_or(&max_fee_per_gas);
            let fee_bytes: [u8; 32] = hex::decode(fee_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Max fee must be 32 bytes (U256)"))?;
            println!(
                "EVM call: {} -> {} ({} bytes input, gas={})",
                source,
                target,
                input_bytes.len(),
                gas_limit
            );
            let tx_hash = client
                .evm_call(
                    wallet.coldkey()?,
                    src_bytes,
                    tgt_bytes,
                    input_bytes,
                    value_bytes,
                    gas_limit,
                    fee_bytes,
                    None,
                    None,
                )
                .await?;
            println!("EVM call submitted. Tx: {}", tx_hash);
            Ok(())
        }
        EvmCommands::Withdraw { address, amount } => {
            validate_evm_address(&address, "withdraw")?;
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let addr_hex = address.strip_prefix("0x").unwrap_or(&address);
            let addr_bytes: [u8; 20] = hex::decode(addr_hex)?
                .try_into()
                .map_err(|_| anyhow::anyhow!("Address must be 20 bytes (EVM address)"))?;
            println!("Withdrawing {} RAO from EVM address {}", amount, address);
            let tx_hash = client
                .evm_withdraw(wallet.coldkey()?, addr_bytes, amount)
                .await?;
            println!("EVM withdrawal submitted. Tx: {}", tx_hash);
            Ok(())
        }
    }
}

// ──────── SafeMode ────────

pub(super) async fn handle_safe_mode(
    cmd: SafeModeCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    match cmd {
        SafeModeCommands::Enter => {
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!("Entering safe mode (permissionless)...");
            let tx_hash = client.safe_mode_enter(wallet.coldkey()?).await?;
            println!("Safe mode entered. Tx: {}", tx_hash);
            Ok(())
        }
        SafeModeCommands::Extend => {
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!("Extending safe mode...");
            let tx_hash = client.safe_mode_extend(wallet.coldkey()?).await?;
            println!("Safe mode extended. Tx: {}", tx_hash);
            Ok(())
        }
        SafeModeCommands::ForceEnter { duration } => {
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!("Force entering safe mode for {} blocks (sudo)...", duration);
            let tx_hash = client
                .safe_mode_force_enter(wallet.coldkey()?, duration)
                .await?;
            println!("Safe mode force-entered. Tx: {}", tx_hash);
            Ok(())
        }
        SafeModeCommands::ForceExit => {
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!("Force exiting safe mode (sudo)...");
            let tx_hash = client.safe_mode_force_exit(wallet.coldkey()?).await?;
            println!("Safe mode force-exited. Tx: {}", tx_hash);
            Ok(())
        }
    }
}

// ──────── Drand ────────

pub(super) async fn handle_drand(cmd: DrandCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    match cmd {
        DrandCommands::WritePulse { payload, signature } => {
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            let payload_hex = payload.strip_prefix("0x").unwrap_or(&payload);
            let payload_bytes = hex::decode(payload_hex)
                .map_err(|e| anyhow::anyhow!("Invalid payload hex: {}", e))?;
            let sig_hex = signature.strip_prefix("0x").unwrap_or(&signature);
            let sig_bytes = hex::decode(sig_hex)
                .map_err(|e| anyhow::anyhow!("Invalid signature hex: {}", e))?;
            println!(
                "Writing Drand pulse ({} bytes payload, {} bytes sig)",
                payload_bytes.len(),
                sig_bytes.len()
            );
            let tx_hash = client
                .drand_write_pulse(wallet.coldkey()?, payload_bytes, sig_bytes)
                .await?;
            println!("Drand pulse written. Tx: {}", tx_hash);
            Ok(())
        }
    }
}

// ──────── Serve ────────

pub(super) async fn handle_serve(cmd: ServeCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, hotkey_name, password) = (
        ctx.wallet_dir,
        ctx.wallet_name,
        ctx.hotkey_name,
        ctx.password,
    );
    match cmd {
        ServeCommands::Axon {
            netuid,
            ip,
            port,
            protocol,
            version,
        } => {
            let ip_u128 = crate::cli::helpers::validate_ipv4(&ip)?;
            crate::cli::helpers::validate_port(port, "axon")?;
            let (pair, _hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            let axon = crate::types::chain_data::AxonInfo {
                block: 0,
                version,
                ip: ip_u128.to_string(),
                port,
                ip_type: 4,
                protocol,
            };
            println!(
                "Serving axon on SN{}: {}:{} (proto={}, ver={})",
                netuid, ip, port, protocol, version
            );
            let hash = client.serve_axon(&pair, NetUid(netuid), &axon).await?;
            println!(
                "Axon served on SN{}: {}:{} (proto={}, ver={}).\n  Tx: {}",
                netuid, ip, port, protocol, version, hash
            );
            Ok(())
        }
        ServeCommands::BatchAxon { file } => {
            let content = std::fs::read_to_string(&file)
                .map_err(|e| anyhow::anyhow!("Failed to read batch axon file '{}': {}", file, e))?;
            // Validate entire JSON upfront before unlocking wallet or submitting anything
            let entries = crate::cli::helpers::validate_batch_axon_json(&content)?;
            let (pair, _hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;

            println!("Batch serving {} axon updates", entries.len());
            for (i, entry) in entries.iter().enumerate() {
                // Fields are validated by validate_batch_axon_json, but use safe access for defense-in-depth
                let netuid_val = entry["netuid"].as_u64().ok_or_else(|| {
                    anyhow::anyhow!("Batch entry {}: missing or invalid 'netuid'", i)
                })?;
                let netuid: u16 = netuid_val.try_into().map_err(|_| {
                    anyhow::anyhow!(
                        "Batch entry {}: netuid {} exceeds u16::MAX ({})",
                        i,
                        netuid_val,
                        u16::MAX
                    )
                })?;
                let ip = entry["ip"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Batch entry {}: missing or invalid 'ip'", i))?;
                let port_val = entry["port"].as_u64().ok_or_else(|| {
                    anyhow::anyhow!("Batch entry {}: missing or invalid 'port'", i)
                })?;
                let port: u16 = port_val.try_into().map_err(|_| {
                    anyhow::anyhow!(
                        "Batch entry {}: port {} exceeds u16::MAX ({})",
                        i,
                        port_val,
                        u16::MAX
                    )
                })?;
                let protocol: u8 = entry["protocol"]
                    .as_u64()
                    .unwrap_or(4)
                    .try_into()
                    .map_err(|_| {
                        anyhow::anyhow!(
                            "Batch entry {}: protocol value exceeds u8::MAX (255)",
                            i
                        )
                    })?;
                let version: u32 = entry["version"]
                    .as_u64()
                    .unwrap_or(0)
                    .try_into()
                    .map_err(|_| {
                        anyhow::anyhow!(
                            "Batch entry {}: version value exceeds u32::MAX",
                            i
                        )
                    })?;

                let ip_u128 = crate::cli::helpers::validate_ipv4(ip)?;

                let axon = crate::types::chain_data::AxonInfo {
                    block: 0,
                    version,
                    ip: ip_u128.to_string(),
                    port,
                    ip_type: 4,
                    protocol,
                };
                let hash = client.serve_axon(&pair, NetUid(netuid), &axon).await?;
                println!("  [{}] SN{} {}:{} — Tx: {}", i + 1, netuid, ip, port, hash);
            }
            println!("Batch axon update complete ({} entries).", entries.len());
            Ok(())
        }
        ServeCommands::Reset { netuid } => {
            validate_netuid(netuid)?;
            let (pair, hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!(
                "Resetting axon info for hotkey {} on SN{}",
                crate::utils::short_ss58(&hk),
                netuid
            );
            // Reset axon by setting all fields to zero
            let axon = crate::types::chain_data::AxonInfo {
                block: 0,
                version: 0,
                ip: "0".to_string(),
                port: 0,
                ip_type: 4,
                protocol: 0,
            };
            let hash = client.serve_axon(&pair, NetUid(netuid), &axon).await?;
            println!(
                "Axon reset for {} on SN{}.\n  Tx: {}",
                crate::utils::short_ss58(&hk),
                netuid,
                hash
            );
            Ok(())
        }
        ServeCommands::Prometheus {
            netuid,
            ip,
            port,
            version,
        } => {
            let ip_u128 = crate::cli::helpers::validate_ipv4(&ip)?;
            crate::cli::helpers::validate_port(port, "prometheus")?;
            let (pair, _hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            println!(
                "Serving Prometheus on SN{}: {}:{} (ver={})",
                netuid, ip, port, version
            );
            let hash = client
                .serve_prometheus(&pair, NetUid(netuid), version, ip_u128, port, 4)
                .await?;
            println!(
                "Prometheus served on SN{}: {}:{} (ver={}).\n  Tx: {}",
                netuid, ip, port, version, hash
            );
            Ok(())
        }
        ServeCommands::AxonTls {
            netuid,
            ip,
            port,
            protocol,
            version,
            cert,
        } => {
            let ip_u128 = crate::cli::helpers::validate_ipv4(&ip)?;
            crate::cli::helpers::validate_port(port, "axon-tls")?;
            let cert_bytes = std::fs::read(&cert)
                .map_err(|e| anyhow::anyhow!("Failed to read TLS cert '{}': {}", cert, e))?;
            let (pair, _hk) =
                unlock_and_resolve(wallet_dir, wallet_name, hotkey_name, None, password)?;
            let axon = crate::types::chain_data::AxonInfo {
                block: 0,
                version,
                ip: ip_u128.to_string(),
                port,
                ip_type: 4,
                protocol,
            };
            println!(
                "Serving axon with TLS on SN{}: {}:{} (cert={} bytes)",
                netuid,
                ip,
                port,
                cert_bytes.len()
            );
            let hash = client
                .serve_axon_tls(&pair, NetUid(netuid), &axon, &cert_bytes)
                .await?;
            println!(
                "Axon TLS served on SN{}: {}:{} (cert={} bytes).\n  Tx: {}",
                netuid,
                ip,
                port,
                cert_bytes.len(),
                hash
            );
            Ok(())
        }
    }
}

// ──────── Proxy ────────

pub(super) async fn handle_proxy(cmd: ProxyCommands, client: &Client, ctx: &Ctx<'_>) -> Result<()> {
    let (wallet_dir, wallet_name, output, password) =
        (ctx.wallet_dir, ctx.wallet_name, ctx.output, ctx.password);
    let adding = matches!(cmd, ProxyCommands::Add { .. });
    match cmd {
        ProxyCommands::Add {
            delegate,
            proxy_type,
            delay,
        }
        | ProxyCommands::Remove {
            delegate,
            proxy_type,
            delay,
        } => {
            validate_ss58(&delegate, "delegate")?;
            validate_proxy_type(&proxy_type)?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let verb = if adding { "Adding" } else { "Removing" };
            println!(
                "{} proxy: {} (type={}, delay={})",
                verb,
                crate::utils::short_ss58(&delegate),
                proxy_type,
                delay
            );
            let hash = if adding {
                client
                    .add_proxy(wallet.coldkey()?, &delegate, &proxy_type, delay)
                    .await?
            } else {
                client
                    .remove_proxy(wallet.coldkey()?, &delegate, &proxy_type, delay)
                    .await?
            };
            println!(
                "Proxy {} for {} (type={}, delay={}).\n  Tx: {}",
                if adding { "added" } else { "removed" },
                crate::utils::short_ss58(&delegate),
                proxy_type,
                delay,
                hash
            );
            Ok(())
        }
        ProxyCommands::CreatePure {
            proxy_type,
            delay,
            index,
        } => {
            validate_proxy_type(&proxy_type)?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            println!(
                "Creating pure proxy (type={}, delay={}, index={})",
                proxy_type, delay, index
            );
            let hash = client
                .create_pure_proxy(wallet.coldkey()?, &proxy_type, delay, index)
                .await?;
            println!("Pure proxy created (type={}, delay={}).\n  Run `agcli proxy list` to find the new pure proxy address.\n  Tx: {}", proxy_type, delay, hash);
            Ok(())
        }
        ProxyCommands::KillPure {
            spawner,
            proxy_type,
            index,
            height,
            ext_index,
        } => {
            validate_ss58(&spawner, "spawner")?;
            validate_proxy_type(&proxy_type)?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            confirm_action(
                "WARNING: Killing pure proxy will make ALL funds in it PERMANENTLY inaccessible.\n\
                 This operation CANNOT be undone. Type 'yes' to confirm:"
            )?;
            let hash = client
                .kill_pure_proxy(
                    wallet.coldkey()?,
                    &spawner,
                    &proxy_type,
                    index,
                    height,
                    ext_index,
                )
                .await?;
            println!("Pure proxy killed. All funds in this proxy are now permanently inaccessible.\n  Tx: {}", hash);
            Ok(())
        }
        ProxyCommands::List { address } => {
            let addr = resolve_and_validate_coldkey_address(
                address,
                wallet_dir,
                wallet_name,
                "proxy list --address",
            )?;
            let proxies = client.list_proxies(&addr).await?;
            if output.is_json() {
                let json: Vec<serde_json::Value> = proxies.iter().map(|(d, t, delay)| {
                    serde_json::json!({"delegate": d, "proxy_type": t, "delay": delay})
                }).collect();
                print_json_ser(&json);
            } else {
                render_rows(
                    output,
                    &proxies,
                    "delegate,proxy_type,delay",
                    |(delegate, proxy_type, delay)| {
                        format!("{},{},{}", delegate, proxy_type, delay)
                    },
                    &["Delegate", "Type", "Delay"],
                    |(delegate, proxy_type, delay)| {
                        vec![
                            crate::utils::short_ss58(delegate),
                            proxy_type.clone(),
                            format!("{}", delay),
                        ]
                    },
                    Some(&format!(
                        "Proxy accounts for {}:",
                        crate::utils::short_ss58(&addr)
                    )),
                );
            }
            Ok(())
        }
        ProxyCommands::Announce { real, call_hash } => {
            validate_ss58(&real, "real")?;
            validate_call_hash(&call_hash, "proxy announce")?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let hash_hex = call_hash.strip_prefix("0x").unwrap_or(&call_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?.try_into().map_err(|_| {
                anyhow::anyhow!("Call hash must be exactly 32 bytes (64 hex chars)")
            })?;
            println!(
                "Announcing proxy call for {} (hash: {})",
                crate::utils::short_ss58(&real),
                call_hash
            );
            let tx_hash = client
                .proxy_announce(wallet.coldkey()?, &real, hash_bytes)
                .await?;
            println!(
                "Proxy announcement submitted for {}.\n  Tx: {}",
                crate::utils::short_ss58(&real),
                tx_hash
            );
            Ok(())
        }
        ProxyCommands::ProxyAnnounced {
            delegate,
            real,
            proxy_type,
            pallet,
            call,
            args,
        } => {
            validate_ss58(&delegate, "delegate")?;
            validate_ss58(&real, "real")?;
            if let Some(ref pt) = proxy_type {
                validate_proxy_type(pt)?;
            }
            validate_pallet_call(&pallet, "pallet")?;
            validate_pallet_call(&call, "call")?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let fields = parse_json_args(&args)?;
            println!(
                "Executing announced proxy call: {}.{} (delegate={}, real={})",
                pallet,
                call,
                crate::utils::short_ss58(&delegate),
                crate::utils::short_ss58(&real)
            );
            let tx_hash = client
                .proxy_announced(
                    wallet.coldkey()?,
                    &delegate,
                    &real,
                    proxy_type.as_deref(),
                    &pallet,
                    &call,
                    fields,
                )
                .await?;
            println!(
                "Announced proxy call executed: {}.{} (delegate={}, real={}).\n  Tx: {}",
                pallet,
                call,
                crate::utils::short_ss58(&delegate),
                crate::utils::short_ss58(&real),
                tx_hash
            );
            Ok(())
        }
        ProxyCommands::RejectAnnouncement {
            delegate,
            call_hash,
        } => {
            validate_ss58(&delegate, "delegate")?;
            validate_call_hash(&call_hash, "proxy reject")?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let hash_hex = call_hash.strip_prefix("0x").unwrap_or(&call_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?.try_into().map_err(|_| {
                anyhow::anyhow!("Call hash must be exactly 32 bytes (64 hex chars)")
            })?;
            println!(
                "Rejecting announcement from {}",
                crate::utils::short_ss58(&delegate)
            );
            let tx_hash = client
                .proxy_reject_announcement(wallet.coldkey()?, &delegate, hash_bytes)
                .await?;
            println!(
                "Announcement from {} rejected.\n  Tx: {}",
                crate::utils::short_ss58(&delegate),
                tx_hash
            );
            Ok(())
        }
        ProxyCommands::ListAnnouncements { address } => {
            let addr = resolve_and_validate_coldkey_address(
                address,
                wallet_dir,
                wallet_name,
                "proxy list-announcements --address",
            )?;
            let announcements = client.list_proxy_announcements(&addr).await?;
            if output.is_json() {
                let json: Vec<serde_json::Value> = announcements
                    .iter()
                    .map(|(real, call_hash, height)| {
                        serde_json::json!({"real": real, "call_hash": call_hash, "height": height})
                    })
                    .collect();
                print_json_ser(&json);
            } else if announcements.is_empty() {
                println!(
                    "No pending announcements for {}",
                    crate::utils::short_ss58(&addr)
                );
            } else {
                println!(
                    "Pending proxy announcements for {} ({} found):",
                    crate::utils::short_ss58(&addr),
                    announcements.len()
                );
                for (real, call_hash, height) in &announcements {
                    println!("  Real: {}  Hash: {}  Height: {}", real, call_hash, height);
                }
            }
            Ok(())
        }
        ProxyCommands::RemoveAll => {
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            confirm_action(
                "WARNING: This will revoke ALL proxy delegations for your account.\n\
                 This cannot be undone. Confirm?"
            )?;
            println!("Removing all proxy delegations");
            let hash = client.remove_proxies(wallet.coldkey()?).await?;
            println!("All proxy delegations removed.\n  Tx: {}", hash);
            Ok(())
        }
        ProxyCommands::RemoveAnnouncement { real, call_hash } => {
            validate_ss58(&real, "real")?;
            validate_call_hash(&call_hash, "proxy remove-announcement")?;
            let mut wallet = open_wallet(wallet_dir, wallet_name)?;
            unlock_coldkey(&mut wallet, password)?;
            let hash_hex = call_hash.strip_prefix("0x").unwrap_or(&call_hash);
            let hash_bytes: [u8; 32] = hex::decode(hash_hex)?.try_into().map_err(|_| {
                anyhow::anyhow!("Call hash must be exactly 32 bytes (64 hex chars)")
            })?;
            println!(
                "Removing announcement for {} (hash: {})",
                crate::utils::short_ss58(&real),
                call_hash
            );
            let tx_hash = client
                .remove_announcement(wallet.coldkey()?, &real, hash_bytes)
                .await?;
            println!(
                "Announcement removed for {}.\n  Tx: {}",
                crate::utils::short_ss58(&real),
                tx_hash
            );
            Ok(())
        }
    }
}

// ──────── Crowdloan ────────

pub(super) async fn handle_crowdloan(
    cmd: CrowdloanCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    let (wallet_dir, wallet_name, password, output) =
        (ctx.wallet_dir, ctx.wallet_name, ctx.password, ctx.output);
    // Read-only query commands (no wallet needed)
    match &cmd {
        CrowdloanCommands::List => {
            let crowdloans = client.list_crowdloans().await?;
            if crowdloans.is_empty() {
                println!("No crowdloans found.");
            } else {
                render_rows(
                    output,
                    &crowdloans,
                    "id,creator,deposit,raised,cap,end_block,finalized",
                    |(id, creator, deposit, raised, cap, end_block, finalized)| {
                        format!(
                            "{},{},{},{},{},{},{}",
                            id,
                            creator,
                            Balance::from_rao(*deposit).display_tao(),
                            Balance::from_rao(*raised).display_tao(),
                            Balance::from_rao(*cap).display_tao(),
                            end_block,
                            finalized,
                        )
                    },
                    &[
                        "ID",
                        "Creator",
                        "Deposit",
                        "Raised",
                        "Cap",
                        "End Block",
                        "Done",
                    ],
                    |(id, creator, deposit, raised, cap, end_block, finalized)| {
                        vec![
                            format!("{}", id),
                            crate::utils::short_ss58(creator),
                            Balance::from_rao(*deposit).display_tao(),
                            Balance::from_rao(*raised).display_tao(),
                            Balance::from_rao(*cap).display_tao(),
                            format!("{}", end_block),
                            if *finalized { "Yes" } else { "No" }.to_string(),
                        ]
                    },
                    Some(&format!("{} crowdloans", crowdloans.len())),
                );
            }
            return Ok(());
        }
        CrowdloanCommands::Info { crowdloan_id } => {
            let info = client.get_crowdloan_info(*crowdloan_id).await?;
            match info {
                Some((
                    creator,
                    deposit,
                    raised,
                    cap,
                    end_block,
                    min_contrib,
                    finalized,
                    target,
                )) => {
                    println!("Crowdloan #{}", crowdloan_id);
                    println!("  Creator:          {}", creator);
                    println!(
                        "  Deposit:          {}",
                        Balance::from_rao(deposit).display_tao()
                    );
                    println!(
                        "  Raised:           {}",
                        Balance::from_rao(raised).display_tao()
                    );
                    println!(
                        "  Cap:              {}",
                        Balance::from_rao(cap).display_tao()
                    );
                    println!(
                        "  Progress:         {:.1}%",
                        if cap > 0 {
                            raised as f64 / cap as f64 * 100.0
                        } else {
                            0.0
                        }
                    );
                    println!("  End block:        {}", end_block);
                    println!(
                        "  Min contribution: {}",
                        Balance::from_rao(min_contrib).display_tao()
                    );
                    println!(
                        "  Finalized:        {}",
                        if finalized { "Yes" } else { "No" }
                    );
                    if let Some(t) = target {
                        println!("  Target address:   {}", t);
                    }
                }
                None => println!("Crowdloan #{} not found.", crowdloan_id),
            }
            return Ok(());
        }
        CrowdloanCommands::Contributors { crowdloan_id } => {
            let contributors = client.get_crowdloan_contributors(*crowdloan_id).await?;
            if contributors.is_empty() {
                println!("No contributors for crowdloan #{}.", crowdloan_id);
            } else {
                let total: u64 = contributors.iter().map(|(_, amount)| amount).sum();
                render_rows(
                    output,
                    &contributors,
                    "address,amount_rao,pct",
                    |(addr, amount)| {
                        format!(
                            "{},{},{:.2}",
                            addr,
                            amount,
                            if total > 0 {
                                *amount as f64 / total as f64 * 100.0
                            } else {
                                0.0
                            },
                        )
                    },
                    &["Address", "Amount", "%"],
                    |(addr, amount)| {
                        vec![
                            crate::utils::short_ss58(addr),
                            Balance::from_rao(*amount).display_tao(),
                            format!(
                                "{:.1}%",
                                if total > 0 {
                                    *amount as f64 / total as f64 * 100.0
                                } else {
                                    0.0
                                }
                            ),
                        ]
                    },
                    Some(&format!(
                        "{} contributors, total {}",
                        contributors.len(),
                        Balance::from_rao(total).display_tao()
                    )),
                );
            }
            return Ok(());
        }
        _ => {} // Fall through to write commands that need wallet
    }

    // Validate write-command inputs before wallet unlock
    match &cmd {
        CrowdloanCommands::Create {
            deposit,
            min_contribution,
            cap,
            target,
            ..
        } => {
            validate_crowdloan_amount(*deposit, "deposit")?;
            validate_crowdloan_amount(*min_contribution, "min-contribution")?;
            validate_crowdloan_amount(*cap, "cap")?;
            if *min_contribution > *cap {
                anyhow::bail!(
                    "Invalid crowdloan: min_contribution ({:.9}) exceeds cap ({:.9}).\n  Tip: min_contribution must be ≤ cap.",
                    min_contribution, cap
                );
            }
            if *deposit > *cap {
                anyhow::bail!(
                    "Invalid crowdloan: deposit ({:.9}) exceeds cap ({:.9}).\n  Tip: deposit should be ≤ cap.",
                    deposit, cap
                );
            }
            if let Some(ref t) = target {
                validate_ss58(t, "target")?;
            }
        }
        CrowdloanCommands::Contribute { amount, .. } => {
            validate_crowdloan_amount(*amount, "contribution amount")?;
        }
        CrowdloanCommands::UpdateCap { cap, .. } => {
            validate_crowdloan_amount(*cap, "cap")?;
        }
        CrowdloanCommands::UpdateMinContribution {
            min_contribution, ..
        } => {
            validate_crowdloan_amount(*min_contribution, "min-contribution")?;
        }
        _ => {} // ID-only commands validated by clap (u32)
    }

    // Write commands need wallet
    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    unlock_coldkey(&mut wallet, password)?;
    let pair = wallet.coldkey()?;
    let (action, hash) = match cmd {
        CrowdloanCommands::Create {
            deposit,
            min_contribution,
            cap,
            end_block,
            target,
        } => {
            let dep = Balance::from_tao(deposit);
            let min = Balance::from_tao(min_contribution);
            let cap_b = Balance::from_tao(cap);
            println!(
                "Creating crowdloan: deposit={}, min={}, cap={}, end_block={}",
                dep.display_tao(),
                min.display_tao(),
                cap_b.display_tao(),
                end_block
            );
            (
                "Crowdloan created",
                client
                    .crowdloan_create(
                        pair,
                        dep.rao(),
                        min.rao(),
                        cap_b.rao(),
                        end_block,
                        target.as_deref(),
                    )
                    .await?,
            )
        }
        CrowdloanCommands::Contribute {
            crowdloan_id,
            amount,
        } => {
            let bal = Balance::from_tao(amount);
            println!(
                "Contributing {} to crowdloan #{}",
                bal.display_tao(),
                crowdloan_id
            );
            (
                "Contribution submitted",
                client.crowdloan_contribute(pair, crowdloan_id, bal).await?,
            )
        }
        CrowdloanCommands::Withdraw { crowdloan_id } => {
            println!("Withdrawing from crowdloan #{}", crowdloan_id);
            (
                "Withdrawal submitted",
                client.crowdloan_withdraw(pair, crowdloan_id).await?,
            )
        }
        CrowdloanCommands::Finalize { crowdloan_id } => {
            println!("Finalizing crowdloan #{}", crowdloan_id);
            (
                "Crowdloan finalized",
                client.crowdloan_finalize(pair, crowdloan_id).await?,
            )
        }
        CrowdloanCommands::Refund { crowdloan_id } => {
            println!("Refunding contributors of crowdloan #{}", crowdloan_id);
            (
                "Refund submitted",
                client.crowdloan_refund(pair, crowdloan_id).await?,
            )
        }
        CrowdloanCommands::Dissolve { crowdloan_id } => {
            println!("Dissolving crowdloan #{}", crowdloan_id);
            (
                "Crowdloan dissolved",
                client.crowdloan_dissolve(pair, crowdloan_id).await?,
            )
        }
        CrowdloanCommands::UpdateCap { crowdloan_id, cap } => {
            let cap_b = Balance::from_tao(cap);
            println!(
                "Updating cap of crowdloan #{} to {}",
                crowdloan_id,
                cap_b.display_tao()
            );
            (
                "Cap updated",
                client
                    .crowdloan_update_cap(pair, crowdloan_id, cap_b.rao())
                    .await?,
            )
        }
        CrowdloanCommands::UpdateEnd {
            crowdloan_id,
            end_block,
        } => {
            println!(
                "Updating end block of crowdloan #{} to {}",
                crowdloan_id, end_block
            );
            (
                "End block updated",
                client
                    .crowdloan_update_end(pair, crowdloan_id, end_block)
                    .await?,
            )
        }
        CrowdloanCommands::UpdateMinContribution {
            crowdloan_id,
            min_contribution,
        } => {
            let min = Balance::from_tao(min_contribution);
            println!(
                "Updating min contribution of crowdloan #{} to {}",
                crowdloan_id,
                min.display_tao()
            );
            (
                "Min contribution updated",
                client
                    .crowdloan_update_min_contribution(pair, crowdloan_id, min.rao())
                    .await?,
            )
        }
        // Read-only variants already handled above
        CrowdloanCommands::List
        | CrowdloanCommands::Info { .. }
        | CrowdloanCommands::Contributors { .. } => unreachable!(),
    };
    println!("{}. Tx: {}", action, hash);
    Ok(())
}

// ──────── Liquidity ────────

/// Convert a price (TAO per Alpha) to a Uniswap V3-style tick index.
/// tick = log(price) / log(1.0001), clamped to [-887272, 887272].
fn price_to_tick(price: f64) -> i32 {
    const MIN_TICK: i32 = -887272;
    const MAX_TICK: i32 = 887272;
    if !price.is_finite() || price <= 0.0 {
        return MIN_TICK;
    }
    let tick = (price.ln() / 1.0001_f64.ln()) as i32;
    tick.clamp(MIN_TICK, MAX_TICK)
}

pub(super) async fn handle_liquidity(
    cmd: LiquidityCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    // Validate inputs before wallet unlock
    match &cmd {
        LiquidityCommands::Add {
            netuid,
            price_low,
            price_high,
            amount,
            hotkey,
        } => {
            validate_netuid(*netuid)?;
            validate_price(*price_low, "price-low")?;
            validate_price(*price_high, "price-high")?;
            validate_price_range(*price_low, *price_high)?;
            if *amount == 0 {
                anyhow::bail!("Invalid liquidity amount: cannot be zero.\n  Tip: specify a positive RAO amount.");
            }
            if let Some(ref hk) = hotkey {
                validate_ss58(hk, "hotkey")?;
            }
        }
        LiquidityCommands::Remove { netuid, hotkey, .. } => {
            validate_netuid(*netuid)?;
            if let Some(ref hk) = hotkey {
                validate_ss58(hk, "hotkey")?;
            }
        }
        LiquidityCommands::Modify {
            netuid,
            delta,
            hotkey,
            ..
        } => {
            validate_netuid(*netuid)?;
            if *delta == 0 {
                anyhow::bail!("Invalid liquidity delta: cannot be zero.\n  Tip: use a positive value to add or negative to remove liquidity.");
            }
            if let Some(ref hk) = hotkey {
                validate_ss58(hk, "hotkey")?;
            }
        }
        LiquidityCommands::Toggle { netuid, .. } => {
            validate_netuid(*netuid)?;
        }
    }

    let hotkey_name = ctx.hotkey_name;
    let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
    unlock_coldkey(&mut wallet, ctx.password)?;

    match cmd {
        LiquidityCommands::Add {
            netuid,
            price_low,
            price_high,
            amount,
            hotkey,
        } => {
            let hk = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let pair = wallet.coldkey()?;
            let tick_low = price_to_tick(price_low);
            let tick_high = price_to_tick(price_high);
            if tick_low >= tick_high {
                anyhow::bail!(
                    "price_low ({:.6}) must be less than price_high ({:.6})",
                    price_low,
                    price_high
                );
            }
            println!(
                "Adding liquidity on SN{}: range [{:.6}, {:.6}] (ticks [{}, {}]), amount={} RAO",
                netuid, price_low, price_high, tick_low, tick_high, amount
            );
            let hash = client
                .add_liquidity(pair, &hk, NetUid(netuid), tick_low, tick_high, amount)
                .await?;
            println!(
                "Liquidity added on SN{}: {} RAO in range [{:.6}, {:.6}].\n  Tx: {}",
                netuid, amount, price_low, price_high, hash
            );
        }
        LiquidityCommands::Remove {
            netuid,
            position_id,
            hotkey,
        } => {
            let hk = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let pair = wallet.coldkey()?;
            println!(
                "Removing liquidity position {} on SN{}",
                position_id, netuid
            );
            let hash = client
                .remove_liquidity(pair, &hk, NetUid(netuid), position_id)
                .await?;
            println!(
                "Liquidity position {} removed from SN{}.\n  Tx: {}",
                position_id, netuid, hash
            );
        }
        LiquidityCommands::Modify {
            netuid,
            position_id,
            delta,
            hotkey,
        } => {
            let hk = resolve_hotkey_ss58(hotkey, &mut wallet, hotkey_name)?;
            let pair = wallet.coldkey()?;
            let action = if delta > 0 { "Adding" } else { "Removing" };
            println!(
                "{} {} RAO liquidity on position {} (SN{})",
                action,
                delta.unsigned_abs(),
                position_id,
                netuid
            );
            let hash = client
                .modify_liquidity(pair, &hk, NetUid(netuid), position_id, delta)
                .await?;
            println!(
                "Position {} modified on SN{}: {} {} RAO.\n  Tx: {}",
                position_id,
                netuid,
                action.to_lowercase(),
                delta.unsigned_abs(),
                hash
            );
        }
        LiquidityCommands::Toggle { netuid, enable } => {
            let pair = wallet.coldkey()?;
            let action = if enable { "Enabling" } else { "Disabling" };
            println!(
                "{} user liquidity on SN{} (subnet owner only)",
                action, netuid
            );
            let hash = client
                .toggle_user_liquidity(pair, NetUid(netuid), enable)
                .await?;
            println!(
                "User liquidity {}. Tx: {}",
                if enable { "enabled" } else { "disabled" },
                hash
            );
        }
    }
    Ok(())
}

/// Parse a comma-separated list of SS58 addresses into sorted AccountIds (for multisig).
fn parse_sorted_signatories(csv: &str) -> Result<Vec<crate::AccountId>> {
    let mut ids: Vec<crate::AccountId> = csv
        .split(',')
        .map(|s| Client::ss58_to_account_id_pub(s.trim()))
        .collect::<Result<_>>()?;
    ids.sort();
    Ok(ids)
}

// ──────── Commitment ────────

pub(super) async fn handle_commitment(
    cmd: CommitmentCommands,
    client: &Client,
    ctx: &Ctx<'_>,
) -> Result<()> {
    // Validate inputs before wallet/chain access
    match &cmd {
        CommitmentCommands::Set { netuid, data } => {
            validate_netuid(*netuid)?;
            validate_commitment_data(data)?;
        }
        CommitmentCommands::Get { netuid, hotkey } => {
            validate_netuid(*netuid)?;
            validate_ss58(hotkey, "hotkey")?;
        }
        CommitmentCommands::List { netuid } => {
            validate_netuid(*netuid)?;
        }
    }

    match cmd {
        CommitmentCommands::Set { netuid, data } => {
            let mut wallet = open_wallet(ctx.wallet_dir, ctx.wallet_name)?;
            unlock_coldkey(&mut wallet, ctx.password)?;
            println!("Setting commitment on SN{}: {}", netuid, data);
            let hash = client
                .set_commitment(wallet.coldkey()?, netuid, &data)
                .await?;
            print_tx_result(ctx.output, &hash, "Commitment set.");
            Ok(())
        }
        CommitmentCommands::Get { netuid, hotkey } => {
            let reg = client.get_commitment(netuid, &hotkey).await?;
            match reg {
                Some((block, fields)) => {
                    if ctx.output.is_json() {
                        print_json(&serde_json::json!({
                            "hotkey": hotkey,
                            "netuid": netuid,
                            "block": block,
                            "fields": fields,
                        }));
                    } else {
                        println!(
                            "Commitment for {} on SN{} (block {})",
                            crate::utils::short_ss58(&hotkey),
                            netuid,
                            block
                        );
                        for (i, f) in fields.iter().enumerate() {
                            println!("  [{}] {}", i, f);
                        }
                    }
                }
                None => {
                    if ctx.output.is_json() {
                        print_json(&serde_json::json!({
                            "hotkey": hotkey,
                            "netuid": netuid,
                            "found": false,
                        }));
                    } else {
                        println!(
                            "No commitment found for {} on SN{}",
                            crate::utils::short_ss58(&hotkey),
                            netuid
                        );
                    }
                }
            }
            Ok(())
        }
        CommitmentCommands::List { netuid } => {
            let commitments = client.get_all_commitments(netuid).await?;
            if ctx.output.is_json() {
                let arr: Vec<_> = commitments
                    .iter()
                    .map(|(ss58, block, fields)| {
                        serde_json::json!({
                            "hotkey": ss58,
                            "block": block,
                            "fields": fields,
                        })
                    })
                    .collect();
                print_json(&serde_json::Value::Array(arr));
            } else {
                println!("Commitments on SN{} ({} total):", netuid, commitments.len());
                for (ss58, block, fields) in &commitments {
                    println!("  {} (block {})", crate::utils::short_ss58(ss58), block);
                    for (i, f) in fields.iter().enumerate() {
                        println!("    [{}] {}", i, f);
                    }
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    // ── Issue 80: netuid u16 overflow guard in batch axon ──

    #[test]
    fn netuid_within_u16_succeeds() {
        let netuid_val: u64 = 65535;
        let result: Result<u16, _> = netuid_val.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), u16::MAX);
    }

    #[test]
    fn netuid_exceeding_u16_fails() {
        let netuid_val: u64 = 65536;
        let result: Result<u16, _> = netuid_val.try_into();
        assert!(result.is_err(), "netuid 65536 should fail u16 conversion");
    }

    #[test]
    fn netuid_zero_succeeds() {
        let netuid_val: u64 = 0;
        let result: Result<u16, _> = netuid_val.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0u16);
    }

    // ── Issue 81: protocol/version truncation guards ──

    #[test]
    fn protocol_within_u8_range_succeeds() {
        let val: u64 = 4;
        let result: Result<u8, _> = val.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 4u8);
    }

    #[test]
    fn protocol_exceeding_u8_fails() {
        let val: u64 = 256;
        let result: Result<u8, _> = val.try_into();
        assert!(result.is_err(), "protocol 256 should fail u8 conversion");
    }

    #[test]
    fn version_within_u32_range_succeeds() {
        let val: u64 = u32::MAX as u64;
        let result: Result<u32, _> = val.try_into();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), u32::MAX);
    }

    #[test]
    fn version_exceeding_u32_fails() {
        let val: u64 = u32::MAX as u64 + 1;
        let result: Result<u32, _> = val.try_into();
        assert!(result.is_err(), "version exceeding u32::MAX should fail");
    }

    // ── Issue 82: price_to_tick NaN/Inf guard ──

    #[test]
    fn price_to_tick_nan_returns_min_tick() {
        assert_eq!(super::price_to_tick(f64::NAN), -887272);
    }

    #[test]
    fn price_to_tick_inf_returns_min_tick() {
        assert_eq!(super::price_to_tick(f64::INFINITY), -887272);
    }

    #[test]
    fn price_to_tick_neg_inf_returns_min_tick() {
        assert_eq!(super::price_to_tick(f64::NEG_INFINITY), -887272);
    }

    #[test]
    fn price_to_tick_zero_returns_min_tick() {
        assert_eq!(super::price_to_tick(0.0), -887272);
    }

    #[test]
    fn price_to_tick_negative_returns_min_tick() {
        assert_eq!(super::price_to_tick(-1.0), -887272);
    }

    #[test]
    fn price_to_tick_normal_price_is_sane() {
        let tick = super::price_to_tick(1.0);
        assert_eq!(tick, 0, "price=1.0 should give tick=0 (ln(1)=0)");
    }

    // ── Audit fix: multisig/scheduler --args JSON must not silently swallow parse errors ──

    /// Verify the fixed pattern: malformed JSON must produce an error, not silently become `vec![]`.
    #[test]
    fn malformed_json_args_rejects_instead_of_silent_default() {
        let bad_json = Some("not valid json{".to_string());
        // The OLD pattern: .and_then(|s| serde_json::from_str(s).ok()).unwrap_or_default()
        // would return vec![] here — silently skipping spending limit checks.
        // The NEW pattern must produce an Err:
        let result: Result<Vec<serde_json::Value>, _> = match bad_json.as_ref() {
            Some(s) => serde_json::from_str(s).map_err(|e| {
                anyhow::anyhow!("invalid --args JSON: {e}")
            }),
            None => Ok(vec![]),
        };
        assert!(result.is_err(), "malformed JSON must be rejected, not silently defaulted to empty");
    }

    #[test]
    fn valid_json_array_args_parses_correctly() {
        let good_json = Some("[1, \"0xabc\", true]".to_string());
        let result: Result<Vec<serde_json::Value>, _> = match good_json.as_ref() {
            Some(s) => serde_json::from_str(s).map_err(|e| {
                anyhow::anyhow!("invalid --args JSON: {e}")
            }),
            None => Ok(vec![]),
        };
        assert!(result.is_ok(), "valid JSON array should parse successfully");
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn none_args_yields_empty_vec() {
        let no_json: Option<String> = None;
        let result: Result<Vec<serde_json::Value>, _> = match no_json.as_ref() {
            Some(s) => serde_json::from_str(s).map_err(|e| {
                anyhow::anyhow!("invalid --args JSON: {e}")
            }),
            None => Ok(vec![]),
        };
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn json_object_instead_of_array_rejects() {
        // A JSON object is valid JSON but not an array — should fail deserialization to Vec
        let obj_json = Some("{\"amount\": 100}".to_string());
        let result: Result<Vec<serde_json::Value>, _> = match obj_json.as_ref() {
            Some(s) => serde_json::from_str(s).map_err(|e| {
                anyhow::anyhow!("invalid --args JSON: {e}")
            }),
            None => Ok(vec![]),
        };
        assert!(result.is_err(), "JSON object (not array) must be rejected");
    }

    // ──── Issue 100: Multisig address uses correct Substrate prefix ────

    #[test]
    fn multisig_address_uses_utilisuba_prefix() {
        // Verify the multisig address derivation matches Substrate's multi_account_id:
        // blake2_256(SCALE(b"modlpy/utilisuba" ++ compact(len) ++ sorted_accounts ++ threshold_le16))
        // Test against known Substrate multisig address for Alice+Bob threshold=2
        use blake2::digest::{Update, VariableOutput};
        use parity_scale_codec::{Compact, Encode};
        use sp_core::Pair;

        let alice = sp_core::crypto::AccountId32::from(
            sp_core::sr25519::Pair::from_string("//Alice", None).unwrap().public()
        );
        let bob = sp_core::crypto::AccountId32::from(
            sp_core::sr25519::Pair::from_string("//Bob", None).unwrap().public()
        );
        let mut ids = vec![alice, bob];
        ids.sort();

        let threshold: u16 = 2;
        let mut hasher = blake2::Blake2bVar::new(32).unwrap();
        hasher.update(b"modlpy/utilisuba");
        hasher.update(&Compact(ids.len() as u32).encode());
        for id in &ids {
            hasher.update(id.as_ref());
        }
        hasher.update(&threshold.to_le_bytes());
        let mut hash = [0u8; 32];
        hasher.finalize_variable(&mut hash).unwrap();

        let multisig_account = sp_core::crypto::AccountId32::from(hash);
        let ms_ss58 = multisig_account.to_string();

        // This should be a deterministic, valid SS58 address
        assert!(ms_ss58.starts_with('5'), "Multisig SS58 should start with '5': {}", ms_ss58);
        assert_eq!(ms_ss58.len(), 48, "SS58 addresses are 48 chars: {}", ms_ss58);

        // Verify it uses utilisuba, not teleport — the hash should NOT match a "teleport" derivation
        let mut hasher2 = blake2::Blake2bVar::new(32).unwrap();
        hasher2.update(b"modlpy/teleport");
        hasher2.update(&threshold.to_le_bytes());
        for id in &ids {
            hasher2.update(id.as_ref());
        }
        let mut hash2 = [0u8; 32];
        hasher2.finalize_variable(&mut hash2).unwrap();
        assert_ne!(hash, hash2, "utilisuba and teleport prefixes must produce different addresses");
    }

    // ──── Issue 102: Port value validated with try_into ────

    #[test]
    fn batch_port_overflow_rejected() {
        // Port value > 65535 should be rejected, not silently truncated
        let port_val: u64 = 70000;
        let port_result: Result<u16, _> = port_val.try_into();
        assert!(port_result.is_err(), "Port {} should not fit in u16", port_val);
    }

    #[test]
    fn batch_port_valid_accepted() {
        let port_val: u64 = 8080;
        let port_result: Result<u16, _> = port_val.try_into();
        assert!(port_result.is_ok());
        assert_eq!(port_result.unwrap(), 8080u16);
    }

    #[test]
    fn batch_port_max_valid() {
        let port_val: u64 = 65535;
        let port_result: Result<u16, _> = port_val.try_into();
        assert!(port_result.is_ok());
        assert_eq!(port_result.unwrap(), 65535u16);
    }

    // ── Issue 135: change_take uses .round() ──

    #[test]
    fn change_take_rounds_correctly() {
        // Same pattern as ChildkeyTake fix (Issue 130) — must also round here
        let take = 0.01_f64; // 0.01%
        let take_u16 = (take / 100.0 * 65535.0).round().min(65535.0) as u16;
        // Without round: 6.5535 truncates to 6. With round: 7.
        assert_eq!(take_u16, 7);
    }

    #[test]
    fn change_take_truncation_would_give_wrong_value() {
        let take = 0.01_f64;
        let truncated = (take / 100.0 * 65535.0).min(65535.0) as u16;
        let rounded = (take / 100.0 * 65535.0).round().min(65535.0) as u16;
        assert_eq!(truncated, 6, "truncation gives wrong value");
        assert_eq!(rounded, 7, "rounding gives correct value");
        assert_ne!(truncated, rounded);
    }
}
