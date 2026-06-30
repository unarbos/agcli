//! Portfolio queries — aggregate all stakes, balances, and subnet positions.

use crate::chain::Client;
use crate::types::Balance;
use anyhow::Result;
use serde::Serialize;

/// A user's complete portfolio across all subnets.
#[derive(Debug, Serialize)]
pub struct Portfolio {
    pub coldkey_ss58: String,
    pub free_balance: Balance,
    pub total_staked: Balance,
    pub positions: Vec<SubnetPosition>,
}

/// Stake position on a single subnet.
#[derive(Debug, Serialize)]
pub struct SubnetPosition {
    pub netuid: u16,
    pub subnet_name: String,
    pub hotkey_ss58: String,
    pub alpha_stake: u64,
    pub tao_equivalent: Balance,
    pub price: f64,
}

/// Fetch the full portfolio for a coldkey (subnet names from DynamicInfo, spot prices from
/// the swap runtime API). Uses a single pinned block hash for all queries — saves redundant
/// at_latest() RPC round-trips and ensures every value is from the same block.
pub async fn fetch_portfolio(client: &Client, coldkey_ss58: &str) -> Result<Portfolio> {
    // Pin a single block for consistency across all queries
    let block_hash = client.pin_latest_block().await?;

    // Parallel fetch at the pinned block: balance, stakes, dynamic info (names), spot prices
    let (balance, stakes, dynamic, prices) = tokio::try_join!(
        client.get_balance_at_hash(coldkey_ss58, block_hash),
        client.get_stake_for_coldkey_at_block(coldkey_ss58, block_hash),
        async {
            match client.get_all_dynamic_info_at_block(block_hash).await {
                Ok(d) => Ok::<_, anyhow::Error>(std::sync::Arc::new(d)),
                Err(e) => {
                    tracing::warn!("Failed to fetch dynamic info for portfolio: {e:#}");
                    Ok(std::sync::Arc::new(vec![]))
                }
            }
        },
        async {
            Ok::<_, anyhow::Error>(
                client
                    .current_alpha_price_all_at_block(block_hash)
                    .await
                    .unwrap_or_default(),
            )
        },
    )?;
    let dynamic_map: std::collections::HashMap<u16, &crate::types::chain_data::DynamicInfo> =
        dynamic.iter().map(|d| (d.netuid.0, d)).collect();

    let positions: Vec<SubnetPosition> = stakes
        .iter()
        .map(|s| {
            let di = dynamic_map.get(&s.netuid.0);
            let price = prices.get(&s.netuid.0).copied().unwrap_or(0.0);
            SubnetPosition {
                netuid: s.netuid.0,
                subnet_name: di.map(|d| d.name.clone()).unwrap_or_default(),
                hotkey_ss58: s.hotkey.clone(),
                alpha_stake: s.stake.raw(),
                tao_equivalent: s.stake.to_tao(price),
                price,
            }
        })
        .collect();

    let total_staked = positions
        .iter()
        .fold(Balance::ZERO, |acc, p| acc + p.tao_equivalent);

    Ok(Portfolio {
        coldkey_ss58: coldkey_ss58.to_string(),
        free_balance: balance,
        total_staked,
        positions,
    })
}
