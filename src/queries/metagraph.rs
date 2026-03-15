//! Metagraph query — fetch full subnet state.

use crate::chain::Client;
use crate::types::chain_data::Metagraph;
use crate::types::NetUid;
use anyhow::Result;

/// Fetch the metagraph for a subnet.
pub async fn fetch_metagraph(client: &Client, netuid: NetUid) -> Result<Metagraph> {
    // Parallel fetch: neurons and block number are independent queries
    let (neurons, block) = tokio::try_join!(
        client.get_neurons_lite(netuid),
        client.get_block_number(),
    )?;
    let n = neurons.len() as u16;

    // Single-pass extraction: iterate once instead of 12 times
    let mut stake = Vec::with_capacity(neurons.len());
    let mut ranks = Vec::with_capacity(neurons.len());
    let mut trust = Vec::with_capacity(neurons.len());
    let mut consensus = Vec::with_capacity(neurons.len());
    let mut incentive = Vec::with_capacity(neurons.len());
    let mut dividends = Vec::with_capacity(neurons.len());
    let mut emission = Vec::with_capacity(neurons.len());
    let mut validator_trust = Vec::with_capacity(neurons.len());
    let mut validator_permit = Vec::with_capacity(neurons.len());
    let mut uids = Vec::with_capacity(neurons.len());
    let mut active = Vec::with_capacity(neurons.len());
    let mut last_update = Vec::with_capacity(neurons.len());

    for neuron in &neurons {
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
