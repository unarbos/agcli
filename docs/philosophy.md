# Philosophy: Subnets, incentives, and the Bittensor machine

This document distills the design philosophy behind Bittensor subnets — drawn from the [Chi subnet knowledge base](https://github.com/unconst/Chi), production subnet patterns, and agcli's own design principles. It is written for operators, subnet builders, and AI agents that need to understand *why* the network works the way it does, not just *how* to interact with it.

---

## The one-sentence rule

Every subnet must answer **"What am I measuring?"** in a single sentence. If the answer contains the word "and," simplify.

- **Valid**: "Who can serve the cheapest GPU inference?" (Targon SN4)
- **Valid**: "Who can predict weather most accurately?" (Zeus SN18)
- **Invalid**: "AI" — too vague
- **Invalid**: "Compute and storage and inference" — too many things

A subnet that measures one thing well is better than a subnet that measures many things poorly. The chain doesn't care what the game is. It only cares about the weight matrix validators submit. Yuma consensus aggregates those weights, rewards agreement, penalizes outliers, and distributes emissions. The game lives entirely in the validator's scoring logic.

```bash
agcli explain yuma        # How consensus works
agcli explain emissions   # How emissions flow from block → subnet → miner/validator
agcli explain subnets     # Lifecycle, netuid, tempo, hyperparams
```

---

## Only write the validator

Chi's most distinctive doctrine: **only write `validator.py`**. Never write miner code.

The validator defines what gets measured and how it gets scored. Miners infer how to win. Publishing reference miner implementations constrains the solution space and stifles innovation — miners copy it verbatim instead of finding better solutions.

This has a direct operational consequence: **validators are the critical infrastructure**. They need speed (weight-setting within tempo windows), reliability (stale-while-error caching when the chain hiccups), auditability (`diff subnet`, `audit`, `--at-block`), and safety (`--dry-run`, spending limits). agcli was built for this role.

---

## The trust model

### Validators are not individually trustworthy

Any secret held by a validator **will** leak. A validator running its own miner will share evaluation data with it. This attack is invisible and certain on any valuable subnet.

What prevents it: **stake-weighted consensus**. Yuma clips outlier weights and rewards agreement across the full validator set. No single actor can dominate. Economic alignment, not individual honesty, is the security model.

```bash
agcli subscribe events --filter staking    # Watch validator behavior in real time
agcli diff subnet --netuid 1               # Compare metagraph state across blocks
agcli audit --address 5Gx...               # Check proxy permissions, delegate exposure
```

### Secret eval sets always fail

If a validator holds a secret test set and scores miners against it, the validator can simply share the answers with their own miner. Chi marks this pattern as **FORBIDDEN**.

Trustworthy ground truth comes from:

| Source | How it works | Example |
|---|---|---|
| **Deterministic generation** | Tasks from public seeds, reproducible by any validator | Molecular binding scores (Nova SN68) |
| **Delayed real-world data** | Ground truth arrives after miners commit answers | Weather forecasts vs ERA5 actuals (Zeus SN18) |
| **Adversarial competition** | One team generates challenges, another solves them | Deepfake generation vs detection (BitMind SN34) |
| **External APIs** | Third-party platform provides ground truth | GitHub PR verification (Gittensor SN74) |
| **Hardware attestation** | Cryptographic TEE proofs | GPU verification via GraVal (Chutes SN64) |

### Coldkeys are not sybil-proof

Anyone can create unlimited coldkeys. "One best hotkey per coldkey" does nothing — the attacker just creates more coldkeys.

The actual sybil protection: **256 UID slots + dynamic registration costs**. Registration cost rises proportionally to expected rewards, creating economic equilibrium. The cost of N slots exceeds the reward from N slots.

**Design implication**: mechanisms where N miners is NOT N times more profitable.

```bash
agcli subnet cost --netuid 1     # Current registration cost
agcli view dynamic               # Pool economics, TAO reserves, alpha pricing
agcli diff network               # Track issuance and economic pressure over time
```

### Similarity detection is unreliable

Fine-tuning a few steps changes model weights while preserving capability. Paraphrasing defeats output similarity. If evasion is cheaper than honest work, rational miners will evade.

Instead of trying to detect copies, design mechanisms where **copying has diminishing returns** — Pareto scoring (copies can only tie, never dominate), delayed scoring after submissions close, and diversity requirements.

---

## Push compute to miners, not validators

Validators should be cheap to run. Expensive validators cause centralization — only well-funded operators can afford to validate, which shrinks the validator set and weakens consensus.

- **Bad**: Validators download models and run inference locally
- **Good**: Miners host endpoints, validators make lightweight API calls

agcli embodies this. A validator using agcli doesn't need a beefy machine to interact with the chain. The CLI is a ~10 MB static binary. Three-layer caching minimizes RPC calls. Parallel queries use Tokio's work-stealing scheduler across whatever cores are available.

---

## The eight subnet patterns

Every production subnet maps to one of these canonical designs:

### 1. Compute Auction

Miners serve fungible compute (GPUs, VMs) and compete on price. Validators run auction clearing — cheapest providers are allocated first.

*Example*: **Targon (SN4)** — miners advertise H100/A100/V100 with bid prices, hardware attestation prevents GPU spoofing, scoring is `(1 - bid/max_bid) * gpu_multiplier`.

### 2. Capacity Market

Always-on infrastructure with usage-based rewards. Miners connect to a central orchestrator; no public endpoint needed.

*Example*: **Chutes (SN64)** — GPU inference marketplace. GraVal cryptographic proofs verify hardware. Scoring is total billed compute delivered over a rolling 7-day window.

### 3. Data Indexing

Unique datasets with sample verification. Miners scrape and store data, commit a bucket URL to chain once, validators pull samples for deep validation.

*Example*: **Data Universe (SN13)** — scorable bytes formula `(miner_bytes² / total_bytes)` penalizes duplication and rewards unique data. Credibility EMA builds slowly and is lost quickly on validation failures.

### 4. Prediction Market

Future predictions with delayed ground truth. Miners submit distribution samples, validators store predictions and score against actual outcomes after the horizon passes.

*Example*: **Synth Subnet** — Monte Carlo predictions scored by CRPS (Continuous Ranked Probability Score). Lower CRPS = better calibrated distributions.

### 5. Time-Series Forecasting

Sequential predictions against a known baseline. Score = improvement over baseline + speed bonus.

*Example*: **Zeus (SN18)** — weather forecasts scored against ERA5 reanalysis. Difficulty scaling adjusts for temporal lead time and geographic complexity. Mixture-of-Experts enables miner specialization by region/variable.

### 6. External Activity Verification

Third-party platform work where the external API provides ground truth. No miner server needed.

*Example*: **Gittensor (SN74)** — miners submit GitHub PRs, validators query the GitHub API. Scoring uses language weights, test-file downweighting, repository star multipliers, and issue-linking bonuses. Self-merge and maintainer PRs are filtered.

### 7. Adversarial Red/Blue

Detection vs generation tasks where ground truth is expensive to curate. Red team generates challenges, blue team solves them, truth emerges from competition.

*Example*: **BitMind (SN34)** — generators create deepfakes with C2PA provenance, discriminators submit ONNX detection models. The benchmark evolves continuously as generator outputs feed future discriminator tests.

**Key insight**: validators don't need ground truth if miners create the challenges. The correct answer emerges from the competitive process. This removes validators from the trust equation entirely.

### 8. Container Execution

Miners compete on code quality, not hardware. Submit Docker images, validators pull and run in sandboxes, score deterministically.

*Example*: **Affine (SN120)** — reasoning model competition. Miners upload to HuggingFace, deploy to Chutes. Epsilon-Pareto dominance with winners-take-all: copies can only tie, never dominate. Must *improve* to earn. Open-source flywheel — best model is public, copied, improved by next miner.

```bash
agcli explain registration   # Burn vs PoW, immunity period, UID recycling
agcli explain alpha           # Alpha tokens, subnet-specific staking
agcli explain amm             # Constant-product AMM math, slippage, pool depth
agcli subnet health --netuid 1   # Per-subnet diagnostics
agcli subnet monitor --netuid 1  # Live metagraph delta stream
```

---

## Subnet design ideas from production

### Score models, not predictions

Numinous (SN6) evaluates the underlying agent over many events rather than scoring individual predictions. This prevents lucky one-off guesses and rewards consistent calibration. The question is not "was this prediction good?" but "is this predictor good?"

### Deterministic oracles eliminate consensus disputes

When ground truth can be computed deterministically (Nova SN68 uses PSICHIC for molecular binding), all validators agree on identical scores. This makes consensus trivial and eliminates validator subjectivity as an attack vector.

### The copy-improve flywheel

Open source creates an improvement cycle. The best model is public. Miners download it, improve it, redeploy. The network gets better over time. This trades individual competitive advantage for ecosystem growth — and the ecosystem benefits outweigh the individual cost.

Affine (SN120) demonstrates this: epsilon-Pareto scoring means copying the leader gives you zero advantage (copies tie, ties don't dominate). You must *beat* the leader to earn. But the leader's code is public, so you have a foundation to build on.

### Usage-based rewards over flat emissions

Hippius (SN75) shifted from flat emissions to usage-based: only actively-used storage and compute earns rewards. Idle capacity doesn't get the allocation bonus. This rewards **actual utility**, not just claimed capacity.

### Geographic diversity as an explicit incentive

Hippius also dedicates 5% of scoring weight to geographic diversity, explicitly incentivizing decentralization and preventing regional concentration. Small but meaningful.

### Adversarial evolution

BitMind (SN34) builds an **evolving benchmark** — generator outputs become part of future detector test data. The benchmark never goes stale because it continuously evolves with the arms race between red and blue teams.

### Specialization through Mixture of Experts

Zeus (SN18) dynamically selects the best miners per context (region, variable, forecast horizon). Miners can specialize in niches rather than building monolithic generalists. Competition drives quality within each niche; the network provides coverage across all niches. This mirrors the division of labor in economics.

---

## Communication: Epistula, not Axon

The old Bittensor communication system (Axon, Dendrite, Synapse) is deprecated. New subnets use **HTTP APIs with Epistula signing**.

Miners commit connection information to the chain (API endpoints, S3 URLs, Docker images). Validators read chain commitments for discovery. Requests are authenticated with Epistula headers:

- `X-Epistula-Timestamp` — nonce
- `X-Epistula-Signature` — signed `{nonce}.{sha256(body)}`
- `X-Epistula-Hotkey` — hotkey SS58 address

Chain commitments are rate-limited to 1 per 100 blocks (~20 minutes). This means you commit a **storage location** (a bucket URL, an API endpoint), not individual data items.

---

## The incentive design process

Building a subnet is a seven-step mechanism design exercise:

1. **Define the commodity** — what value do miners provide? (inference, compute, data, predictions, external work, availability)
2. **Define quality** — how is "good" measured? (accuracy, speed, reliability, uniqueness, cost efficiency)
3. **Design verification** — how do validators confirm quality? (direct, delayed, sampling, external, hardware, adversarial)
4. **Identify attacks** — how could miners cheat?
5. **Add anti-gaming** — credibility EMA (`score * credibility^2.5`), time decay, uniqueness scoring, coldkey dedup
6. **Convert to weights** — normalization, softmax temperature (low = winner-take-most, high = more equal), inverse softmax for "lower is better" metrics
7. **Test and iterate** — simulate with honest and gaming miners, deploy to testnet, monitor emission concentration

**Decision tree checkpoints**:
- Secret eval sets? → Redesign
- Similarity detection as primary defense? → Redesign
- Coldkey dedup for sybil resistance? → Redesign
- Expensive validators? → Push compute to miners

```bash
agcli explain tempo          # Weight-setting cadence
agcli explain commit-reveal  # Two-phase weight submission
agcli explain hyperparams    # What each hyperparameter controls
agcli explain validators     # Validator role, permits, consensus
agcli explain miners         # Miner role, registration, serving
```

---

## The network architecture

### Participants

| Role | What they do | Emission share |
|---|---|---|
| **Subnet owner** | Deploys subnet, configures hyperparams, sets identity | 18% |
| **Validator** | Queries miners, scores quality, submits weights | 41% |
| **Miner** | Provides the commodity, competes on quality | 41% |
| **Staker** | Locks TAO on hotkeys, receives proportional emissions | Via delegation |

### The epoch cycle

Every `tempo` blocks:
1. Filter active neurons, identify validators
2. Process the weight matrix
3. Calculate stake-weighted median consensus (outlier weights clipped)
4. Calculate trust, rank, and incentive
5. Update bonds using EMA with liquid alpha
6. Calculate dividends: `dividend[v] = sum(bond[v][m] * incentive[m])`
7. Distribute emissions: 41% miners, 41% validators, 18% owner

### Keys

- **Coldkey**: cold storage — staking, transfers, subnet creation, ownership. Keep offline.
- **Hotkey**: operational — registration, weight-setting, request signing. Can be on servers.

One coldkey controls many hotkeys. agcli reads existing Python bittensor-wallet keyfiles directly.

### Alpha tokens and the AMM

Each subnet has an alpha token. Staking TAO into a subnet goes through a constant-product AMM (`TAO_reserve * Alpha_reserve = k`). The deeper the pool, the less slippage. agcli surfaces this:

```bash
agcli view dynamic --netuid 1    # Pool reserves, price, 24h volume
agcli utils convert --netuid 1 --tao 100   # Simulate swap
agcli explain amm                # The math behind it
agcli explain alpha              # Alpha token mechanics
```

---

## Why tooling matters

Understanding these mechanics is not optional — it is the difference between operating effectively and flying blind. This is why agcli embeds 31 `explain` topics in the binary, provides `diff` commands for forensic analysis, surfaces economic data through `view dynamic` and `subnet cost`, and streams real-time events through `subscribe`.

The philosophy is simple: **make Bittensor legible, not just usable**. An operator who understands the incentive game they're participating in will always outperform one who is just typing CLI commands.

```bash
agcli explain --list    # See all 31+ embedded topics
```
