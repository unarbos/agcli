//! Metagraph query — fetch full subnet state.

use crate::chain::Client;
use crate::types::chain_data::Metagraph;
use crate::types::NetUid;
use anyhow::Result;

/// Fetch the metagraph for a subnet.
pub async fn fetch_metagraph(client: &Client, netuid: NetUid) -> Result<Metagraph> {
    let neurons = client.get_neurons_lite(netuid).await?;
    let n = neurons.len() as u16;
    let block = client.get_block_number().await?;

    Ok(Metagraph {
        netuid,
        n,
        block,
        stake: neurons.iter().map(|n| n.stake).collect(),
        ranks: neurons.iter().map(|n| n.rank).collect(),
        trust: neurons.iter().map(|n| n.trust).collect(),
        consensus: neurons.iter().map(|n| n.consensus).collect(),
        incentive: neurons.iter().map(|n| n.incentive).collect(),
        dividends: neurons.iter().map(|n| n.dividends).collect(),
        emission: neurons.iter().map(|n| n.emission).collect(),
        validator_trust: neurons.iter().map(|n| n.validator_trust).collect(),
        validator_permit: neurons.iter().map(|n| n.validator_permit).collect(),
        uids: neurons.iter().map(|n| n.uid).collect(),
        active: neurons.iter().map(|n| n.active).collect(),
        last_update: neurons.iter().map(|n| n.last_update).collect(),
        neurons,
    })
}
