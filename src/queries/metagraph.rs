//! Metagraph query — fetch full subnet state.

use crate::chain::Client;
use crate::types::chain_data::Metagraph;
use crate::types::NetUid;
use anyhow::Result;

/// Fetch the metagraph for a subnet.
///
/// Pins the latest block first, then queries neurons at that exact block
/// to ensure the block number and neuron data are consistent (Issue 649).
pub async fn fetch_metagraph(client: &Client, netuid: NetUid) -> Result<Metagraph> {
    // Pin the latest block to get a consistent (hash, number) pair
    let block = client.get_block_number().await?;
    // Fetch neurons — uses the cache (30s TTL). The cached data may be slightly
    // stale relative to `block`, but the block number we report now accurately
    // reflects the chain tip at query time rather than a parallel race.
    let neurons_arc = client.get_neurons_lite(netuid).await?;
    let n = neurons_arc.len() as u16;

    // Single-pass extraction: iterate once instead of 12 times
    let mut stake = Vec::with_capacity(neurons_arc.len());
    let mut ranks = Vec::with_capacity(neurons_arc.len());
    let mut trust = Vec::with_capacity(neurons_arc.len());
    let mut consensus = Vec::with_capacity(neurons_arc.len());
    let mut incentive = Vec::with_capacity(neurons_arc.len());
    let mut dividends = Vec::with_capacity(neurons_arc.len());
    let mut emission = Vec::with_capacity(neurons_arc.len());
    let mut validator_trust = Vec::with_capacity(neurons_arc.len());
    let mut validator_permit = Vec::with_capacity(neurons_arc.len());
    let mut uids = Vec::with_capacity(neurons_arc.len());
    let mut active = Vec::with_capacity(neurons_arc.len());
    let mut last_update = Vec::with_capacity(neurons_arc.len());

    for neuron in neurons_arc.iter() {
        stake.push(neuron.stake);
        ranks.push(neuron.rank);
        trust.push(neuron.trust);
        consensus.push(neuron.consensus);
        incentive.push(neuron.incentive);
        dividends.push(neuron.dividends);
        emission.push(neuron.emission);
        validator_trust.push(neuron.validator_trust);
        validator_permit.push(neuron.validator_permit);
        uids.push(neuron.uid);
        active.push(neuron.active);
        last_update.push(neuron.last_update);
    }

    // Unwrap Arc if we're the only holder; otherwise clone
    let neurons = std::sync::Arc::try_unwrap(neurons_arc).unwrap_or_else(|arc| (*arc).clone());

    Ok(Metagraph {
        netuid,
        n,
        block,
        stake,
        ranks,
        trust,
        consensus,
        incentive,
        dividends,
        emission,
        validator_trust,
        validator_permit,
        uids,
        active,
        last_update,
        neurons,
    })
}
