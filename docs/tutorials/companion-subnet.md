# Tutorial: Running a Companion Subnet with agcli

A step-by-step guide for subnet owners building **consumer-facing AI products** on Bittensor — chatbots, companions, assistants — where miners compete on response quality, memory recall, and personality consistency rather than raw compute throughput.

This tutorial uses [Project Nobi](https://github.com/ProjectNobi/project-nobi) (SN272, testnet) as a concrete example, but the patterns apply to any subnet where the commodity is *conversation quality*.

---

## What makes companion subnets different?

Most Bittensor subnets measure infrastructure: inference speed, storage capacity, prediction accuracy. Companion subnets measure something harder to quantify: **"Is this a good AI companion?"**

| Dimension | Infrastructure subnet | Companion subnet |
|-----------|----------------------|-----------------|
| Commodity | Compute, storage, predictions | Conversation quality |
| Scoring | Deterministic (latency, accuracy) | LLM-as-judge + memory recall |
| Users | Developers, other subnets | Everyday people |
| Success metric | Throughput, cost | User satisfaction, retention |
| Revenue model | API fees (B2B) | Emissions-funded (public good) |

This changes how you operate: you care about **personality consistency** across conversations, **memory recall** accuracy, **safety filtering**, and **emotional intelligence** — not just tokens per second.

---

## Prerequisites

```bash
# Install agcli
cargo install --git https://github.com/unconst/agcli

# Verify
agcli doctor
```

You'll also need:
- A Bittensor wallet with enough TAO for subnet registration
- An LLM API key (e.g., Chutes.ai) for your miners
- Your subnet's validator and miner code

---

## Step 1: Register your subnet

```bash
# Check current registration cost
agcli subnet cost --network test

# Register (testnet)
agcli subnet register --network test --password p --yes

# Verify
agcli --output json subnet list --network test | jq '.[] | select(.owner == "YOUR_COLDKEY")'
```

---

## Step 2: Configure hyperparameters for companion quality

Companion subnets need different hyperparameters than infrastructure subnets. Key settings:

```bash
# Set tempo — longer tempos give validators more time for multi-turn evaluation
agcli admin set-tempo --netuid YOUR_NETUID --tempo 360 \
  --sudo-key //Alice --network local

# Set max validators — companion scoring needs consistency, fewer validators is better
agcli admin set-max-validators --netuid YOUR_NETUID --max 8 \
  --sudo-key //Alice --network local

# Set min weights — ensure validators score enough miners per epoch
agcli admin set-min-weights --netuid YOUR_NETUID --min 4 \
  --sudo-key //Alice --network local

# Enable commit-reveal for weight privacy
agcli admin set-commit-reveal --netuid YOUR_NETUID --enabled true \
  --sudo-key //Alice --network local
```

**Why these matter for companions:**
- **Longer tempo**: Multi-turn conversations take time to evaluate (memory recall requires multiple exchanges)
- **Fewer validators**: LLM-as-judge scoring is expensive; fewer validators keeps validation costs manageable
- **Commit-reveal**: Prevents miners from gaming scores by watching validator weight submissions

```bash
# Verify your hyperparameters
agcli subnet hyperparams --netuid YOUR_NETUID --network test
```

---

## Step 3: Monitor your miners

Companion miners are scored on multiple dimensions. Use agcli to monitor their performance:

```bash
# Live metagraph monitoring — watch registrations, emissions, weight changes
agcli subnet monitor --netuid YOUR_NETUID --network test --json

# Check per-UID emissions
agcli subnet emissions --netuid YOUR_NETUID --network test

# Health check — registration status, active miners, validator coverage
agcli subnet health --netuid YOUR_NETUID --network test

# TCP reachability — verify miners are actually serving
agcli subnet probe --netuid YOUR_NETUID --network test
```

### Tracking companion-specific metrics

agcli shows on-chain metrics (emissions, weights, stakes). Your companion-specific metrics (memory recall scores, quality ratings, safety probe pass rates) live in your validator's scoring pipeline. Use agcli's JSON output to combine both:

```bash
# Export metagraph as JSON for your scoring dashboard
agcli --output json subnet metagraph --netuid YOUR_NETUID --network test > metagraph.json

# Compare metagraph state over time
agcli diff subnet --netuid YOUR_NETUID --from-block 1000 --to-block 2000 --network test
```

---

## Step 4: Manage staking and emissions

```bash
# View your portfolio across subnets
agcli view portfolio

# Stake TAO on your subnet (with slippage protection)
agcli stake add --amount 100 --netuid YOUR_NETUID --max-slippage 2.0 --password p --yes

# Check pool economics
agcli view dynamic --netuid YOUR_NETUID

# Simulate a swap before executing
agcli view swap-sim --netuid YOUR_NETUID --tao 50
```

### Owner emission burns

If your subnet burns owner emissions (like Nobi burns 100% via `burn_alpha()`), you can monitor the burn on-chain:

```bash
# Subscribe to your subnet's events — watch for burn transactions
agcli subscribe events --filter staking --netuid YOUR_NETUID

# Check current alpha supply and pool depth
agcli view dynamic --netuid YOUR_NETUID --output json
```

---

## Step 5: Agent automation

Companion subnets benefit heavily from automation — validators need to run 24/7, miners need monitoring, and the subnet owner needs to track health continuously.

```bash
# Configure agcli for non-interactive operation
agcli config set network test
agcli config set wallet default
agcli config set hotkey default

# Set spending limits (safety net for automated operations)
agcli config set spending_limit.YOUR_NETUID 50

# Run in batch mode — structured errors, never hangs
export AGCLI_BATCH=1
export AGCLI_PASSWORD=your_password
export AGCLI_YES=1

# Automated health check (cron-friendly)
agcli --output json subnet health --netuid YOUR_NETUID
agcli --output json subnet emissions --netuid YOUR_NETUID
```

### Example: automated monitoring script

```bash
#!/bin/bash
# companion-monitor.sh — run via cron every 5 minutes

export AGCLI_BATCH=1
NETUID=272
NETWORK=test

# Check subnet health
health=$(agcli --output json subnet health --netuid $NETUID --network $NETWORK)
active_miners=$(echo "$health" | jq '.active_miners')

if [ "$active_miners" -lt 5 ]; then
    echo "WARNING: Only $active_miners active miners on SN$NETUID"
    # Send alert...
fi

# Check emissions
agcli --output json subnet emissions --netuid $NETUID --network $NETWORK > /tmp/emissions.json
```

---

## Step 6: Debugging with historical queries

When companion quality drops or emissions shift unexpectedly:

```bash
# What changed in the last 500 blocks?
agcli diff subnet --netuid YOUR_NETUID --from-block -500 --network test

# Check metagraph at a specific block
agcli subnet metagraph --netuid YOUR_NETUID --at-block 12345 --network test

# Audit a specific validator or miner
agcli audit --address 5Gx...

# View transaction history
agcli view history --address 5Gx...
```

---

## Companion subnet design patterns

Based on production experience with companion subnets:

### 1. Multi-dimensional scoring
Companions can't be scored on a single metric. Use weighted composite scores:
```
multi_turn_score = 0.60 * quality + 0.30 * memory_recall + 0.10 * reliability
single_turn_score = 0.90 * quality + 0.10 * reliability
```

### 2. Safety as a multiplier
Adversarial safety probes should be ~10% of validator queries. A miner that fails a safety probe gets **zero for the entire round** — safety isn't a dimension, it's a gate.

### 3. Burn for credibility
If your subnet is a public good (no user fees), burning 100% of owner emissions via `burn_alpha()` signals alignment. Every burn transaction is on-chain and verifiable by stakers.

### 4. Memory isolation
If your companion serves both group chats and private DMs, **tag memories with their source** and filter DM recall to exclude group-sourced data. This prevents cross-contamination of private information.

---

## Further reading

- `agcli explain subnets` — subnet lifecycle, registration, tempo
- `agcli explain yuma` — how consensus works
- `agcli explain alpha` — alpha tokens and AMM mechanics
- `agcli explain dynamic-tao` — dTAO economics
- [philosophy.md](../philosophy.md) — subnet design principles, the eight canonical patterns
- [Project Nobi](https://github.com/ProjectNobi/project-nobi) — open-source companion subnet (MIT licensed)

---

*Contributed by T68Bot from [Project Nobi](https://projectnobi.ai) — building personal AI companions for everyone on Bittensor.*
