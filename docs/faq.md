# Frequently Asked Questions

Things agcli doesn't cover on its own — and where to find the answers.

---

## 1. What do miners actually *do*?

agcli shows you how to register, serve an axon, and receive emissions. But the actual mining logic — what service you run, how you respond to queries — is **entirely subnet-specific**.

Miners solve different problems on every subnet. What they do is fully defined by how validators evaluate them and set weights. To understand what a miner does on any given subnet, **walk backwards from how validators score**:

1. Look at the validator's scoring logic (usually in their repo's `validator.py` or equivalent)
2. Understand what metric is being measured (latency, accuracy, data quality, compute, etc.)
3. Build something that optimizes for that metric

That's it. The chain doesn't care what the game is — it only cares about the weight matrix validators submit. Yuma consensus aggregates those weights and distributes emissions accordingly.

```bash
agcli explain yuma        # How consensus turns weights into emissions
agcli explain validators  # Validator role and responsibilities
agcli explain miners      # Miner role and registration
```

**Where to find subnet-specific miner code**: each subnet has its own repository. Browse subnets and find their GitHub links at [taostats.io](https://taostats.io) or [taomarketcap.com](https://taomarketcap.com).

---

## 2. How does Yuma Consensus actually work?

agcli's `explain yuma` topic gives a summary. For the full math, you need the subtensor source code. Here's the core algorithm:

### The weight→emission pipeline

Every `tempo` blocks (subnet-specific cadence), the chain runs Yuma consensus:

1. **Collect the weight matrix** — each validator `v` has submitted a weight vector `W[v]` over all miners `m`. Weights are normalized per-validator so they sum to 1.

2. **Stake weighting** — each validator's weights are scaled by their stake fraction:
   ```
   S[v] = stake[v] / total_stake
   ```

3. **Compute consensus (median)** — for each miner `m`, compute the stake-weighted median of all validators' weights on that miner. This is the **consensus weight** `C[m]`. The median (not mean) is critical — it means a minority of validators cannot manipulate scores.

4. **Clip outliers** — validators whose weights deviate significantly from consensus get clipped. Specifically, for each validator-miner pair, the effective weight is:
   ```
   W_clipped[v][m] = min(W[v][m], C[m] * kappa)
   ```
   where `kappa` controls how much deviation is tolerated. This punishes validators who set weights that disagree with the majority.

5. **Calculate rank (miner incentive)** — each miner's rank is the sum of stake-weighted clipped weights:
   ```
   rank[m] = sum_v(S[v] * W_clipped[v][m])
   ```
   Rank is then normalized to sum to 1. This determines each miner's share of miner emissions.

6. **Calculate trust** — trust for miner `m` is the fraction of stake that gave it non-zero weight:
   ```
   trust[m] = sum_v(S[v] * (W[v][m] > 0))
   ```

7. **Update bonds (EMA)** — bonds track the historical relationship between validators and miners. Updated each epoch using exponential moving average:
   ```
   bond[v][m] = alpha * bond_prev[v][m] + (1 - alpha) * (S[v] * W_clipped[v][m])
   ```
   The `alpha` parameter is "liquid alpha" — per-subnet, controlling how fast bonds adapt.

8. **Calculate dividends (validator incentive)** — each validator's dividend is proportional to their bonds with high-incentive miners:
   ```
   dividend[v] = sum_m(bond[v][m] * rank[m])
   ```
   This means validators earn more by consistently identifying the best miners early.

9. **Distribute emissions** — the subnet's total emissions for this epoch are split:
   - 41% to miners (proportional to rank)
   - 41% to validators (proportional to dividends)
   - 18% to subnet owner

### Key properties

- **Stake = voting power**: validators with more stake have more influence on consensus.
- **Median resists manipulation**: unlike a mean, the median requires >50% of stake to move.
- **Bonds create loyalty**: validators who discover good miners early build bonds and earn more dividends from them over time. This incentivizes active evaluation, not copying other validators' weights.
- **Liquid alpha controls responsiveness**: low alpha = bonds shift quickly (good for fast-changing subnets), high alpha = bonds shift slowly (good for stable subnets).

### Source code reference

The canonical implementation lives in the subtensor Rust codebase:
- `pallets/subtensor/src/epoch/` — the epoch function that runs Yuma
- Key entry point: `epoch()` function, called every `tempo` blocks per subnet
- Weight processing, consensus calculation, bond updates, and emission distribution all happen within this function

```bash
agcli explain yuma          # Summary of consensus mechanics
agcli explain tempo         # Weight-setting cadence per subnet
agcli explain commit-reveal # Two-phase weight submission (anti-MEV)
agcli explain emission      # How emissions flow from block → subnet → participant
agcli explain stake-weight  # Minimum stake to set weights
```

---

## 3. How do I pick a subnet?

This is hard to answer generally because profitability depends on:

- **Competition difficulty** — how much work (compute, data, model quality) other miners are expending to reach the top
- **Your hardware** — some subnets need H100s, others need CPUs and good data pipelines
- **Your expertise** — ML inference, data scraping, weather modeling, etc.
- **Registration cost** — rises with demand, creating economic equilibrium
- **Emission allocation** — how much TAO the subnet receives (determined by root network weights)

There is no universal "best subnet." The profitable play is finding a subnet where your skills and hardware give you an edge over existing miners.

**How to research subnets:**

1. Browse all active subnets at [taomarketcap.com](https://taomarketcap.com) — see emission rates, miner counts, registration costs
2. Read the subnet's GitHub repo and docs to understand what's being measured
3. Check the validator scoring code to understand exactly how miners are ranked
4. Look at current miner performance to gauge competition level

```bash
agcli view dynamic                    # Pool economics, TAO reserves, alpha pricing for all subnets
agcli subnet cost --netuid <N>        # Current registration cost
agcli subnet health --netuid <N>      # Per-subnet diagnostics
agcli view metagraph --netuid <N>     # See all miners/validators, their stakes, and emissions
```

---

## 4. How do validators communicate with miners?

This is **completely subnet-specific**. agcli handles the chain interaction layer — registration, staking, weights, commitments. The actual validator↔miner communication happens off-chain and varies by subnet.

Common patterns:

| Protocol | How it works | Example subnets |
|----------|-------------|-----------------|
| **HTTP + Epistula** | Validators send signed HTTP requests to miner endpoints. Authentication via `X-Epistula-*` headers (timestamp, signature, hotkey). This is the modern standard. | Most new subnets |
| **WebSocket** | Persistent connections for streaming or real-time workloads | Streaming inference subnets |
| **External submission** | Miners submit work to external platforms (HuggingFace, GitHub, S3 buckets). Validators read from those platforms. No direct miner↔validator connection. | Gittensor (SN74), Affine (SN120) |
| **Legacy Axon/Dendrite** | The old Bittensor RPC system using Synapse objects. Deprecated but still in use on older subnets. | Some legacy subnets |

Miners commit their connection information (IP, port, protocol) to chain via `serve_axon`. Validators read the metagraph to discover miner endpoints.

```bash
agcli serve axon --netuid <N> --ip <IP> --port <PORT>   # Commit endpoint to chain
agcli view metagraph --netuid <N>                         # See all committed endpoints
agcli explain axon                                        # Axon serving mechanics
```

See the [Communication: Epistula, not Axon](philosophy.md#communication-epistula-not-axon) section in philosophy.md for more on the Epistula signing protocol.

---

## 5. Where do I find subnet-specific code?

agcli is the **chain interaction layer** — it handles everything that touches the Bittensor blockchain (registration, staking, weights, transfers, governance). But each subnet (SN1, SN2, SN3, ...) has its own separate codebase with:

- Miner implementation (or reference miner)
- Validator scoring logic
- Subnet-specific communication protocols
- Setup guides and hardware requirements

**Where to find them:**

- **[taomarketcap.com](https://taomarketcap.com)** — browse all subnets, each listing links to the subnet's GitHub and website
- **[taostats.io](https://taostats.io)** — alternative explorer with subnet details and links
- **GitHub search** — most subnet repos are under the team's GitHub org

```bash
agcli view metagraph --netuid <N>    # See subnet participants
agcli subnet info --netuid <N>       # Subnet metadata and owner
agcli explain subnets                # How subnets work on the chain
```

agcli gives you everything you need to interact with the chain. For what to actually *run* as a miner or validator on a specific subnet, go to that subnet's repo.
