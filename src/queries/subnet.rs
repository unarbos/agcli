//! Subnet queries.

use crate::chain::Client;
use crate::types::chain_data::SubnetInfo;
use anyhow::Result;

/// List all subnets with basic info.
/// Pins a single block hash and fetches subnets + dynamic info at that block,
/// ensuring data consistency (no cross-block joins).
pub async fn list_subnets(client: &Client) -> Result<Vec<SubnetInfo>> {
    // Pin a single block so both queries read from the same chain state.
    let block_hash = client.pin_latest_block().await?;
    let (subnets_result, dynamic_result) =
        tokio::try_join!(client.get_all_subnets_at_block(block_hash), async {
            Ok::<_, anyhow::Error>(client.get_all_dynamic_info_at_block(block_hash).await)
        },)?;
    let mut subnets = subnets_result;
    // Enrich subnet list with real names from DynamicInfo (one call vs N identity queries)
    if let Ok(dynamic) = dynamic_result {
        let name_map: std::collections::HashMap<u16, (String, u64)> = dynamic
            .iter()
            .filter(|d| !d.name.is_empty())
            .map(|d| (d.netuid.0, (d.name.clone(), d.total_emission())))
            .collect();
        for s in &mut subnets {
            if let Some((name, emission)) = name_map.get(&s.netuid.0) {
                s.name = name.clone();
                if s.emission_value == 0 {
                    s.emission_value = *emission;
                }
            }
        }
    }
    Ok(subnets)
}
