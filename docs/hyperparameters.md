# Subtensor Hyperparameters — Complete Reference

Every subnet on Bittensor has a set of tunable hyperparameters stored on-chain. Some can be changed by the **subnet owner** via `agcli subnet set-param`; others require the **chain sudo key** (root/governance) via `agcli admin`. This guide explains every parameter, what it actually does, and why you'd change it.

---

## Quick Reference Table

| Parameter | Type | Who Sets | agcli Command |
|-----------|------|----------|---------------|
| [tempo](#tempo) | u16 | sudo | `admin set-tempo` |
| [rho](#rho-ρ) | u16 | sudo | `subnet set-param --param rho` |
| [kappa](#kappa-κ) | u16 | sudo | `subnet set-param --param kappa` |
| [immunity_period](#immunity_period) | u16 | owner/sudo | `admin set-immunity-period` |
| [min_allowed_weights](#min_allowed_weights) | u16 | owner/sudo | `admin set-min-weights` |
| [max_allowed_uids](#max_allowed_uids) | u16 | sudo | `admin set-max-uids` |
| [max_allowed_validators](#max_allowed_validators) | u16 | sudo | `admin set-max-validators` |
| [max_weight_limit](#max_weight_limit) | u16 | owner/sudo | `admin set-max-weight-limit` |
| [min_difficulty](#min_difficulty) | u64 | sudo | `subnet set-param --param min_difficulty` |
| [max_difficulty](#max_difficulty) | u64 | sudo | `subnet set-param --param max_difficulty` |
| [difficulty](#difficulty) | u64 | sudo | `admin set-difficulty` |
| [weights_version](#weights_version) | u64 | owner | `subnet set-param --param weights_version` |
| [weights_rate_limit](#weights_rate_limit) | u64 | owner/sudo | `admin set-weights-rate-limit` |
| [adjustment_interval](#adjustment_interval) | u16 | sudo | `subnet set-param --param adjustment_interval` |
| [adjustment_alpha](#adjustment_alpha) | u64 | sudo | `subnet set-param --param adjustment_alpha` |
| [activity_cutoff](#activity_cutoff) | u16 | owner/sudo | `admin set-activity-cutoff` |
| [registration_allowed](#registration_allowed) | bool | sudo | `subnet set-param --param registration_allowed` |
| [pow_registration_allowed](#pow_registration_allowed) | bool | sudo | `subnet set-param --param pow_registration_allowed` |
| [target_regs_per_interval](#target_regs_per_interval) | u16 | sudo | `subnet set-param --param target_regs_per_interval` |
| [min_burn](#min_burn) | u64 | sudo | `subnet set-param --param min_burn` |
| [max_burn](#max_burn) | u64 | sudo | `subnet set-param --param max_burn` |
| [bonds_moving_average](#bonds_moving_average) | u64 | owner/sudo | `admin raw --call sudo_set_bonds_moving_average` |
| [max_regs_per_block](#max_regs_per_block) | u16 | sudo | `subnet set-param --param max_regs_per_block` |
| [serving_rate_limit](#serving_rate_limit) | u64 | owner/sudo | `admin raw --call sudo_set_serving_rate_limit` |
| [commit_reveal_weights_enabled](#commit_reveal_weights_enabled) | bool | owner/sudo | `admin set-commit-reveal` |
| [commit_reveal_weights_interval](#commit_reveal_weights_interval) | u64 | owner | `subnet set-param --param commit_reveal_weights_interval` |
| [commit_reveal_version](#commit_reveal_version) | u64 | owner | `subnet set-param --param commit_reveal_version` |
| [liquid_alpha_enabled](#liquid_alpha_enabled) | bool | owner | `subnet set-param --param liquid_alpha_enabled` |
| [bonds_penalty](#bonds_penalty) | u16 | owner | `subnet set-param --param bonds_penalty` |
| [bonds_reset_enabled](#bonds_reset_enabled) | bool | owner | `subnet set-param --param bonds_reset_enabled` |
| [yuma](#yuma) | bool | sudo | `subnet set-param --param yuma` |
| [min_allowed_uids](#min_allowed_uids) | u16 | sudo | `subnet set-param --param min_allowed_uids` |
| [min_non_immune_uids](#min_non_immune_uids) | u16 | sudo | `subnet set-param --param min_non_immune_uids` |

---

## Epoch & Timing

### tempo
**Blocks per epoch.** An epoch is one complete evaluation cycle: validators set weights, consensus runs, emissions are distributed.

- **Default**: 360 blocks (~72 minutes at 12s/block)
- **Range**: 1–65535
- **Effect**: Lower tempo = faster evaluation cycles = more frequent emission distribution. Higher tempo = less chain overhead but slower responsiveness.
- **Why change it**: Short tempo for fast-iterating subnets (e.g., real-time inference). Long tempo for subnets where evaluation is expensive (e.g., training runs).
- **Gotcha**: Tempo also acts as a rate-limit multiplier — subnet owners can only change hyperparams every `tempo × OwnerHyperparamRateLimit` blocks. Very short tempo means more frequent chain writes.

### activity_cutoff
**Blocks of inactivity before a neuron becomes deregistration-eligible.**

- **Default**: 5000 blocks (~16.6 hours)
- **Range**: 1–65535
- **Effect**: If a validator hasn't set weights (or a miner hasn't been scored) within this window, the neuron is flagged inactive and can be replaced by new registrants.
- **Why change it**: Increase for subnets with long evaluation cycles. Decrease to aggressively prune idle neurons.

### immunity_period
**Blocks of protection after a neuron registers.**

- **Default**: 4096 blocks (~13.6 hours)
- **Range**: 0–65535
- **Effect**: Newly registered neurons can't be deregistered during this window regardless of performance. Gives miners time to set up and validators time to start scoring.
- **Why change it**: Increase if your subnet needs significant setup time. Decrease if you want a competitive "prove yourself fast" environment.
- **Gotcha**: If `immunity_period > activity_cutoff`, neurons could be immune but flagged inactive simultaneously — the immunity takes precedence.

---

## Consensus Parameters

### rho (ρ)
**Inflation/emission adjustment parameter.**

- **Default**: 10 (all subnets)
- **Range**: 0–65535
- **Effect**: Used in the Yuma consensus emission formula. Higher rho amplifies the effect of stake-weighted ranking. The formula: `p = (Staking_Target / Staking_Actual) × Inflation_Target`. Rho controls how aggressively consensus rewards shift toward high-performing validators.
- **Why change it**: Rarely changed. Higher values make consensus more aggressive (winner-take-more). This is a deep protocol parameter — change with extreme caution.
- **Set by**: Sudo only (root governance).

### kappa (κ)
**Consensus majority threshold.**

- **Default**: 32767 (≈50% of u16 max, representing a 50% consensus threshold)
- **Range**: 0–65535
- **Effect**: Determines how much agreement validators need before a weight vector "wins" in Yuma consensus. At 32767, a miner needs to be ranked above median by >50% of stake-weighted validators to receive emissions. Higher kappa = stricter consensus = harder to earn emissions without broad agreement.
- **Why change it**: Increase to require stronger validator agreement (harder for any single validator to steer emissions). Decrease to make consensus more permissive.
- **Gotcha**: Kappa is a u16 where 65535 = 100%. The default 32767 ≈ 50%. Setting it to 0 makes consensus trivially easy; setting it near 65535 makes it nearly impossible.

### yuma
**Enable/disable Yuma consensus entirely.**

- **Default**: true
- **Effect**: When false, the subnet runs without stake-weighted consensus — emissions may be distributed differently or paused.
- **Why change it**: Experimental subnets may disable Yuma during development. Not normally changed in production.

---

## Weight Parameters

### min_allowed_weights
**Minimum number of UIDs a validator must include when setting weights.**

- **Default**: varies by subnet (often 1–16)
- **Range**: 0–65535
- **Effect**: Forces validators to evaluate at least N miners per weight-set call. Prevents validators from only rating one or two miners.
- **Why change it**: Increase to force broader evaluation. Set to 1 for small subnets or during bootstrap.
- **Gotcha**: If set higher than active miners, validators can't set weights at all.

### max_weight_limit
**Maximum weight value per UID in the weight vector.**

- **Default**: 65535 (no effective limit, since weights are u16-normalized)
- **Range**: 0–65535
- **Effect**: Caps the normalized weight any single miner can receive. At 65535, one miner can get 100% of a validator's weight. Lower values force more even distribution.
- **Why change it**: Decrease to prevent validators from concentrating all weight on a single miner (anti-collusion measure).

### weights_version
**Expected weights version key.**

- **Default**: 0 or subnet-specific
- **Range**: 0–u64::MAX
- **Effect**: When set, validators must pass this version number when calling `set_weights`. Mismatched versions are rejected. Used to force validators to upgrade their scoring code.
- **Why change it**: Bump when you release a new scoring algorithm and want to ensure all validators run the updated version.

### weights_rate_limit
**Minimum blocks between weight-set calls.**

- **Default**: varies (e.g., 100)
- **Range**: 0–u64::MAX (0 = unlimited)
- **Effect**: Rate-limits how often a validator can update weights. Prevents spamming the chain with weight updates.
- **Why change it**: Increase for subnets where evaluation is slow and frequent updates are noise. Set to 0 for testing or fast-feedback subnets.

---

## Commit-Reveal

Commit-reveal is a two-phase weight submission protocol that prevents validators from copying each other's weights. Validators first commit a hash, then reveal the actual weights later.

### commit_reveal_weights_enabled
**Toggle commit-reveal for weight submission.**

- **Default**: varies (some subnets have it enabled by default)
- **Effect**: When true, validators must use the two-phase commit-reveal protocol. `set_weights` is rejected — only `commit_weights` + `reveal_weights` work.
- **Why change it**: Enable to prevent weight-copying attacks. Disable if it adds too much complexity for a simple subnet.

### commit_reveal_weights_interval
**Blocks between commit and reveal phases.**

- **Default**: varies
- **Range**: 0–u64::MAX
- **Effect**: After committing, validators wait this many blocks before revealing. Longer interval = more protection from front-running but slower weight finalization.
- **Why change it**: Increase for subnets where weight-copying is a serious threat. Decrease to speed up consensus.

### commit_reveal_version
**Commit-reveal protocol version.**

- **Default**: 0
- **Range**: 0–u64::MAX
- **Effect**: Must match between commit and reveal calls. Allows upgrading the commit-reveal algorithm without breaking in-flight commits.
- **Why change it**: Bump when changing the serialization or hashing scheme for commit-reveal.

---

## Registration & Difficulty

These control how new neurons can join the subnet and how expensive it is.

### registration_allowed
**Master switch for new registrations (burn and PoW).**

- **Default**: true
- **Effect**: When false, no new neurons can register on this subnet. Existing neurons remain.
- **Why change it**: Disable during maintenance, subnet migration, or if the subnet is "full" and the owner wants to freeze membership.

### pow_registration_allowed
**Allow proof-of-work registration specifically.**

- **Default**: varies
- **Effect**: When false, only burn (TAO) registration works. PoW is disabled.
- **Why change it**: Some subnets disable PoW to ensure only economic commitment (burn) is required, preventing GPU farms from flooding registrations.

### difficulty
**Current proof-of-work difficulty.**

- **Default**: dynamically adjusted
- **Range**: 0–u64::MAX
- **Effect**: The hash difficulty target for PoW registration. Higher = harder = fewer successful PoW registrations per unit time.
- **Why change it**: Usually auto-adjusted, but can be overridden for testing or emergency situations.

### min_difficulty / max_difficulty
**Floor and ceiling for the auto-adjusted PoW difficulty.**

- **Defaults**: min ~10^18, max varies
- **Effect**: The difficulty auto-adjuster targets `target_regs_per_interval` registrations. These bounds prevent it from going too low (trivial) or too high (impossible).
- **Why change it**: Raise min_difficulty if PoW registrations are too easy. Lower max_difficulty if registrations become impossible.

### target_regs_per_interval
**Target number of registrations per adjustment interval.**

- **Default**: 1–3 (varies)
- **Range**: 0–65535
- **Effect**: The difficulty auto-adjuster tries to hit this target. If actual registrations exceed the target, difficulty goes up. If below, it goes down.
- **Why change it**: Increase to allow faster subnet growth. Decrease to throttle new entrants.

### adjustment_interval
**Blocks between difficulty/burn auto-adjustments.**

- **Default**: varies (e.g., 112)
- **Range**: 1–65535
- **Effect**: Every N blocks, the chain recalculates difficulty and burn registration cost based on actual vs target registrations.
- **Why change it**: Shorter interval = faster response to registration pressure. Longer = more stable costs.

### adjustment_alpha
**EMA smoothing factor for difficulty adjustment.**

- **Default**: varies
- **Range**: 0–u64::MAX
- **Effect**: Controls how aggressively difficulty changes. High alpha = slow, smooth adjustments. Low alpha = volatile, responsive.
- **Why change it**: Increase for stability. Decrease if you need rapid difficulty response (e.g., during a registration spike).

### min_burn / max_burn
**Floor and ceiling for TAO burn registration cost (in RAO).**

- **Defaults**: min ~1 TAO (10^9 RAO), max varies
- **Range**: 0–u64::MAX
- **Effect**: The burn cost auto-adjusts within these bounds. Setting min_burn high makes registration expensive. Setting max_burn low caps the cost.
- **Why change it**: Raise min_burn to increase economic barrier to entry. Lower max_burn to prevent runaway costs during high demand.
- **Gotcha**: Values are in RAO (1 TAO = 10^9 RAO). A min_burn of `1000000000` = 1 TAO.

### max_regs_per_block
**Maximum registrations allowed per block.**

- **Default**: 1
- **Range**: 0–65535
- **Effect**: Hard cap on how many neurons can register in a single block. Prevents registration spam even if difficulty is low.
- **Why change it**: Increase to allow burst registration. Usually kept at 1 to ensure orderly growth.

---

## Network Size Limits

### max_allowed_uids
**Maximum total neurons (UIDs) allowed on the subnet.**

- **Default**: 256 (varies by subnet, root subnet is 64)
- **Range**: 1–65535
- **Effect**: Hard cap on subnet size. When full, new registrations must replace existing neurons (lowest-performing are deregistered).
- **Why change it**: Increase for large subnets. Decrease to keep competition tight.

### min_allowed_uids
**Minimum neurons on the subnet.**

- **Default**: varies
- **Range**: 0–65535
- **Effect**: Prevents shrinking the subnet below this threshold via deregistration.
- **Why change it**: Set a floor to ensure minimum competition level.

### max_allowed_validators
**Maximum validators with validator permits.**

- **Default**: 128 (subnet 1), varies
- **Range**: 1–65535
- **Effect**: Limits how many neurons can act as validators. The top N by stake get permits. Others become miners.
- **Why change it**: Increase for more decentralized validation. Decrease to concentrate evaluation in fewer, higher-stake validators.

### min_non_immune_uids
**Minimum number of neurons that are NOT immune.**

- **Default**: varies
- **Range**: 0–65535
- **Effect**: Ensures there are always N neurons eligible for deregistration. Prevents a scenario where all neurons are immune and new registrants can never join.
- **Why change it**: Increase to ensure competitive churn. Decrease if registration is slow.

---

## Bonds & Dividends

Bonds determine how emissions are split between validators and miners. Validators accumulate bonds to miners they score well, and dividends flow proportionally.

### bonds_moving_average
**Smoothing factor for bond calculations.**

- **Default**: 900000
- **Range**: 0–u64::MAX
- **Effect**: Higher values = bonds change slowly (more historical weight). Lower values = bonds respond quickly to new weight vectors. This is the "memory" of the bond system.
- **Why change it**: Increase for stability — validators who have been scoring consistently keep more dividends. Decrease for responsiveness — recent performance matters more.
- **Gotcha**: This is a u64 fixed-point value. The actual smoothing fraction depends on the implementation.

### bonds_penalty
**Penalty factor applied to bond calculations.**

- **Default**: 0
- **Range**: 0–65535
- **Effect**: Penalizes validators whose weight vectors deviate too strongly from consensus. Higher penalty = more incentive to agree with other validators.
- **Why change it**: Increase to punish outlier validators. Keep at 0 for subnets where diverse opinions are valuable.

### bonds_reset_enabled
**Allow periodic bond resets.**

- **Default**: false
- **Effect**: When true, bonds can be reset (zeroed) periodically. This prevents entrenched validators from permanently dominating dividends through historical bond accumulation.
- **Why change it**: Enable in subnets where you want a more level playing field and prevent "first-mover" dividend capture.

### liquid_alpha_enabled
**Enable liquid alpha (dynamic dividend distribution).**

- **Default**: false
- **Effect**: When enabled, the alpha parameter in bond calculations (which controls the EMA between old bonds and new weights) becomes dynamic instead of fixed. It adjusts based on validator agreement — when validators agree strongly, alpha is low (bonds update fast); when they disagree, alpha is high (bonds are sticky).
- **Why change it**: Enable for more adaptive dividend dynamics. Particularly useful in subnets with variable miner quality.

---

## Serving (Axon)

### serving_rate_limit
**Minimum blocks between `serve_axon` calls.**

- **Default**: 50
- **Range**: 0–u64::MAX
- **Effect**: Rate-limits how often a neuron can update its on-chain IP/port/protocol info. Prevents spam updates.
- **Why change it**: Increase to reduce chain storage churn. Decrease for subnets where nodes change IP frequently.

---

## How Parameters Interact

### Registration cost loop
`target_regs_per_interval` + `adjustment_interval` + `adjustment_alpha` + `min_burn`/`max_burn` + `min_difficulty`/`max_difficulty` form a feedback loop:
1. Every `adjustment_interval` blocks, the chain checks how many neurons registered
2. If registrations > target → difficulty and burn cost go up
3. If registrations < target → difficulty and burn cost go down
4. `adjustment_alpha` controls the speed of change
5. `min_*`/`max_*` prevent extremes

### Deregistration eligibility
A neuron can be deregistered if ALL of these hold:
1. They've been registered longer than `immunity_period`
2. They've been inactive for longer than `activity_cutoff`
3. The subnet is full (`max_allowed_uids` reached)
4. At least `min_non_immune_uids` neurons remain non-immune
5. A new registrant is competing for the slot

### Consensus → Emissions pipeline
1. Validators set weights (subject to `min_allowed_weights`, `max_weight_limit`, `weights_rate_limit`, `commit_reveal_*`)
2. Every `tempo` blocks, Yuma consensus runs using `rho` and `kappa`
3. Bonds update using `bonds_moving_average`, `bonds_penalty`, and optionally `liquid_alpha_enabled`
4. Emissions split between validators (dividends via bonds) and miners (incentive via rank)

---

## Setting Parameters

### As subnet owner
```bash
# List all settable params
agcli subnet set-param --netuid 1 --param list

# Set a specific param
agcli subnet set-param --netuid 1 --param immunity_period --value 2000

# View current values
agcli subnet hyperparams --netuid 1
```

### As sudo (chain root)
```bash
# Typed commands (common params)
agcli admin set-tempo --netuid 1 --tempo 100 --sudo-key //Alice --network local

# Raw command (any AdminUtils call)
agcli admin raw --call sudo_set_bonds_moving_average --args '[1, 900000]' --sudo-key //Alice --network local

# List all known admin params
agcli admin list
```

### Via scaffold (automated setup)
```toml
# In scaffold.toml
[hyperparams]
tempo = 100
immunity_period = 500
max_allowed_validators = 8
min_allowed_weights = 1
weights_rate_limit = 0
commit_reveal_weights_enabled = false
```

---

## Source References

- **agcli admin commands**: `src/cli/admin_cmds.rs`, `src/admin.rs`
- **agcli subnet set-param**: `src/cli/subnet_cmds.rs` (SUBNET_PARAMS constant, line 2564)
- **On-chain storage**: `SubtensorModule` pallet in subtensor `pallets/subtensor/src/`
- **AdminUtils pallet**: `pallets/admin-utils/src/lib.rs` — all `sudo_set_*` extrinsics
- **Yuma consensus**: `pallets/subtensor/src/epoch/` — the weight→emission pipeline
