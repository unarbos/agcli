//! Subnet queries.

use crate::chain::Client;
use crate::types::chain_data::{DynamicInfo, SubnetInfo};
use crate::types::NetUid;
use anyhow::Result;

/// List all subnets with basic info.
pub async fn list_subnets(client: &Client) -> Result<Vec<SubnetInfo>> {
    // Enrich subnet list with real names from DynamicInfo (one call vs N identity queries)
    let mut subnets = client.get_all_subnets().await?;
    if let Ok(dynamic) = client.get_all_dynamic_info().await {
        let name_map: std::collections::HashMap<u16, String> = dynamic
            .iter()
            .filter(|d| !d.name.is_empty())
            .map(|d| (d.netuid.0, d.name.clone()))
            .collect();
        for s in &mut subnets {
            if let Some(name) = name_map.get(&s.netuid.0) {
                s.name = name.clone();
            }
        }
    }
    Ok(subnets)
}

/// List all subnets with dynamic (pricing) info.
pub async fn list_dynamic_subnets(client: &Client) -> Result<Vec<DynamicInfo>> {
    client.get_all_dynamic_info().await
}

/// Get details for a specific subnet.
pub async fn subnet_detail(client: &Client, netuid: NetUid) -> Result<Option<SubnetInfo>> {
    client.get_subnet_info(netuid).await
}
