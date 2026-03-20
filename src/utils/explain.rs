//! Built-in Bittensor concept reference for `agcli explain <concept>`.

/// Return the explanation text for a concept, or None if not found.
pub fn explain(topic: &str) -> Option<&'static str> {
    match topic.to_lowercase().replace(['-', '_'], "").as_str() {
        "tempo" => Some(TEMPO),
        "commitreveal" | "cr" => Some(COMMIT_REVEAL),
        "yuma" | "yumaconsensus" => Some(YUMA),
        "ratelimit" | "ratelimits" | "weightsratelimit" => Some(RATE_LIMITS),
        "weights" | "settingweights" | "setweights" | "weightsetting" => Some(WEIGHTS),
        "stakeweight" | "stakeweightminimum" | "1000" => Some(STAKE_WEIGHT),
        "amm" | "dynamictao" | "dtao" | "pool" => Some(AMM),
        "bootstrap" => Some(BOOTSTRAP),
        "alpha" | "alphatoken" => Some(ALPHA),
        "emission" | "emissions" => Some(EMISSION),
        "registration" | "register" => Some(REGISTRATION),
        "subnet" | "subnets" => Some(SUBNETS),
        "validator" | "validators" => Some(VALIDATORS),
        "miner" | "miners" => Some(MINERS),
        "immunity" | "immunityperiod" => Some(IMMUNITY),
        "delegate" | "delegation" | "nominate" => Some(DELEGATION),
        "childkey" | "childkeys" => Some(CHILDKEYS),
        "root" | "rootnetwork" => Some(ROOT_NETWORK),
        "proxy" => Some(PROXY),
        "coldkeyswap" | "coldkey" | "ckswap" => Some(COLDKEY_SWAP),
        "governance" | "gov" | "proposals" => Some(GOVERNANCE),
        "senate" | "triumvirate" => Some(SENATE),
        "mevshield" | "mev" | "mevprotection" => Some(MEV_SHIELD),
        "limits" | "networklimits" | "chainlimits" => Some(LIMITS),
        "hyperparams" | "hyperparameters" | "params" => Some(HYPERPARAMS),
        "axon" | "axoninfo" | "serving" => Some(AXON),
        "take" | "delegatetake" | "validatortake" => Some(TAKE),
        "recycle" | "recyclealpha" | "burn" | "burnalpha" => Some(RECYCLE),
        "pow" | "powregistration" | "proofofwork" => Some(POW_REGISTRATION),
        "archive" | "archivenode" | "historical" | "wayback" => Some(ARCHIVE),
        "diff" | "compare" | "historicaldiff" => Some(DIFF),
        "ownerworkflow" | "ow" | "subnetowner" | "ownerguide" => Some(OWNER_WORKFLOW),
        topics if !topics.is_empty() => {
            // Fuzzy: check if the topic is a substring of any key
            let all = list_topics();
            for (key, _) in &all {
                if key.contains(topics) {
                    return explain(key);
                }
            }
            None
        }
        _ => None,
    }
}

/// List all available topics with short descriptions.
pub fn list_topics() -> Vec<(&'static str, &'static str)> {
    vec![
        ("tempo", "Block cadence for subnet weight evaluation"),
        ("commit-reveal", "Two-phase weight submission scheme"),
        ("yuma", "Yuma consensus — the incentive mechanism"),
        ("rate-limits", "Weight setting frequency constraints"),
        (
            "weights",
            "Setting weights: commands, commit-reveal, timeouts, common errors",
        ),
        ("stake-weight", "Minimum stake required to set weights"),
        ("amm", "Automated Market Maker (Dynamic TAO pools)"),
        ("bootstrap", "Getting started as a new subnet owner"),
        ("alpha", "Subnet-specific alpha tokens"),
        ("emission", "How TAO emissions are distributed"),
        ("registration", "Registering neurons on subnets"),
        ("subnets", "What subnets are and how they work"),
        ("validators", "Validator role and responsibilities"),
        ("miners", "Miner role and responsibilities"),
        ("immunity", "Immunity period for new registrations"),
        ("delegation", "Delegating/nominating stake to validators"),
        ("childkeys", "Childkey take and delegation within subnets"),
        ("root", "Root network (SN0) and root weights"),
        ("proxy", "Proxy accounts for delegated signing"),
        ("coldkey-swap", "Coldkey swap scheduling and security"),
        ("governance", "On-chain governance and proposals"),
        ("senate", "Senate / triumvirate governance body"),
        ("mev-shield", "MEV protection on Bittensor"),
        ("limits", "Network and chain operational limits"),
        ("hyperparams", "Subnet hyperparameters reference"),
        ("axon", "Axon serving endpoint for miners/validators"),
        ("take", "Validator/delegate take percentage"),
        ("recycle", "Recycling and burning alpha tokens"),
        ("pow", "Proof-of-work registration mechanics"),
        ("archive", "Archive nodes and historical data queries"),
        ("diff", "Compare chain state between two blocks"),
        ("owner-workflow", "Step-by-step guide for subnet owners"),
    ]
}

const TEMPO: &str = "\
TEMPO
=====
Tempo is the number of blocks between weight evaluation rounds on a subnet.

- Each subnet has its own tempo (e.g., 360 blocks ≈ 72 minutes at 12s/block).
- At each tempo boundary, Yuma consensus runs: weights are evaluated, ranks
  computed, and emissions distributed.
- Miners/validators are scored based on weights set during the tempo.
- Check a subnet's tempo: `agcli subnet hyperparams --netuid <N>`
- Blocks until next tempo = tempo - (current_block % tempo)

Practical impact:
- Weight changes only take effect at the next tempo boundary.
- If you set weights right after a tempo, you wait the full cycle.
- Plan your weight updates to land before the next tempo.";

const COMMIT_REVEAL: &str = "\
COMMIT-REVEAL
=============
A two-phase weight submission scheme that prevents weight copying.

Phase 1 — COMMIT: Hash your weights + a secret salt, submit the hash on-chain.
Phase 2 — REVEAL: After a waiting period, reveal the actual weights + salt.

Why it exists:
- Without commit-reveal, validators can observe others' weight transactions in
  the mempool and copy them before the tempo evaluates. This is a form of
  'weight mimicry' that undermines honest scoring.
- Commit-reveal ensures weights stay secret until the reveal window.

How to use it:
  # Commit (saves the salt — keep it!)
  agcli weights commit --netuid 97 \"0:100,1:200\" --salt mysecret

  # Wait for the reveal window (check commit_reveal_weights_interval in hyperparams)

  # Reveal (must use same weights + salt)
  agcli weights reveal --netuid 97 \"0:100,1:200\" mysecret

Check if a subnet uses commit-reveal:
  agcli subnet hyperparams --netuid <N>  →  commit_reveal_weights = true/false

The commit_reveal_weights_interval hyperparam controls how many tempos
you must wait before revealing.";

const YUMA: &str = "\
YUMA CONSENSUS
==============
Yuma consensus is Bittensor's incentive mechanism. It determines how emissions
are distributed based on validator weight-setting agreements.

How it works:
1. Validators set weights on miners based on perceived performance.
2. At each tempo, the chain aggregates all validator weights.
3. Consensus is reached: miners that multiple validators agree on get higher
   incentive scores. The consensus mechanism rewards agreement.
4. Emissions split: miners get incentive-based share, validators get
   dividends proportional to how well their weights matched consensus.

Key metrics (visible in metagraph):
- Trust:      How much a miner's performance is trusted by validators
- Consensus:  Degree of agreement on a miner's value
- Incentive:  Final score → determines miner's emission share
- Dividends:  Validator's emission share for accurate scoring
- VTrust:     Validator trust — how well a validator's weights match consensus

Why it matters:
- Validators who set accurate weights earn more dividends.
- Miners who deliver real value to multiple validators earn more incentive.
- Gaming the system (weight copying, collusion) is penalized by consensus.

View metagraph: `agcli subnet metagraph --netuid <N>`";

const RATE_LIMITS: &str = "\
RATE LIMITS
===========
The chain enforces rate limits on weight-setting to prevent spam and ensure stability.

weights_rate_limit: Number of blocks you must wait between set_weights calls.
  - Typical: 100 blocks (≈20 minutes at 12s/block)
  - If you try to set weights before the limit expires, the extrinsic fails.

tx_rate_limit: Global transaction rate limit per account.

How to check:
  agcli subnet hyperparams --netuid <N>  →  weights_rate_limit

Practical tips:
- Before calling set_weights, check when you last set weights.
- The error 'SettingWeightsTooFast' or 'TxRateLimitExceeded' means you need to wait.
- Rate limits apply per-hotkey per-subnet, not globally.
- Plan weight updates to be infrequent but well-timed (just before tempo).";

const WEIGHTS: &str = "\
SETTING WEIGHTS (agcli)
=======================
Discoverability (binary-only install):
  `agcli weights --help` — all weight subcommands
  `agcli explain --topic weights` — this summary (aliases: settingweights, setweights)
  `agcli explain --topic weights --full` — full `docs/commands/weights.md` when that tree is available
  `agcli explain` — list every built-in topic

Quick path:

1. Confirm the subnet exists (optional but fast)
   - `agcli subnet show --netuid <N>` — unknown netuid exits **12** (same class of error as bad `--netuid` on weight commands)
   - `agcli weights show --netuid <N>` — read-only view of on-chain weights; same **12** when the subnet is missing; no wallet; e2e **`weights_show_preflight`** + `get_all_weights` / `get_neurons_lite` / `get_weights_for_uid` in Phase 37 `test_all_weights`

2. Pick the flow for your subnet
   - `agcli subnet hyperparams --netuid <N>` → commit-reveal on/off and timing fields
   - If commit-reveal off: `agcli weights set --netuid N \"uid:wt,...\"`
   - If on: two-step `weights commit` + `weights reveal`, or one-shot `agcli weights commit-reveal ...`
   - Multi-mechanism subnets: direct **`weights set-mechanism`** or CR **`weights commit-mechanism`** + **`weights reveal-mechanism`** (hash/salt rules mirror global commit/reveal; see **`docs/commands/weights.md`**)
   - Hotkey: wallet hotkey *name* via `--hotkey` / `--hotkey-name`; use `--hotkey-address` only when a command expects an SS58.

3. Sanity-check before spending fees
   - `agcli weights set --netuid N ... --dry-run` (rate-limit context where available)

4. Extrinsics wait for finalization by default (~30s)
   - `--finalization-timeout`, env `AGCLI_FINALIZATION_TIMEOUT`, or `finalization_timeout` in ~/.agcli/config.toml
   - Global RPC wait: `--timeout`

5. When a transaction fails, agcli decodes Subtensor errors into plain language
   - Look for `Reason:` / `Hint:` after the raw message
   - Deeper context: `agcli explain commit-reveal`, `agcli explain rate-limits`, `agcli explain stake-weight`

Common chain errors (names vary by metadata; numeric codes are decoded too):
- Commit-reveal on vs off (use set vs commit/reveal / commit-reveal)
- Rate limits on `set_weights` and on `commit_weights`
- Min stake / validator permit / min UIDs / version key / weight sums (65535 cap)";

const STAKE_WEIGHT: &str = "\
STAKE-WEIGHT MINIMUM (1000τ)
=============================
To set weights on a subnet, your validator needs a minimum amount of effective
stake-weight. The typical threshold is 1000 TAO equivalent.

Why it exists:
- Prevents low-stake accounts from manipulating subnet scoring.
- Ensures validators have meaningful economic commitment.

What counts toward stake-weight:
- Direct stake from your coldkey to your hotkey on the subnet.
- Delegated (nominated) stake from other coldkeys.
- Childkey delegations from parent hotkeys.

If you're below the threshold:
  1. Ask others to stake/delegate to your validator hotkey.
  2. Move more of your own TAO to stake on that subnet.
  3. Use commit-reveal instead of direct set_weights (some subnets allow
     commit-reveal at lower thresholds).

Check your stake: `agcli stake list`
Check subnet requirements: `agcli subnet hyperparams --netuid <N>`";

const AMM: &str = "\
AMM (DYNAMIC TAO / ALPHA POOLS)
================================
Each subnet has a constant-product AMM (Automated Market Maker) pool that
creates a market between TAO and the subnet's alpha token.

Pool mechanics:
- Two-sided pool: TAO side (tao_in) and Alpha side (alpha_in).
- Price = tao_in / alpha_in (constant-product formula: x * y = k).
- When you stake TAO on a subnet, it swaps through the AMM → you get alpha.
- When you unstake, alpha swaps back → you get TAO.

Slippage:
- Small pools = high slippage. A 10τ stake on a pool with 100τ depth causes
  ~10% slippage (you get fewer alpha per TAO than the listed price).
- Check slippage before staking: `agcli view swap-sim --netuid N --tao 10`

Key metrics:
- price: Current τ/α exchange rate
- tao_in: TAO side of the pool (liquidity depth)
- alpha_in: Alpha side of the pool
- moving_price: Exponential moving average of price (32 fractional bits)

Tips for operators:
- Don't stake/unstake large amounts on shallow pools — use limit orders.
- Watch the pool depth: `agcli subnet show --netuid <N>` shows tao_in.
- The AMM means your alpha is always liquid — you can unstake anytime.";

const BOOTSTRAP: &str = "\
BOOTSTRAP GUIDE — New Subnet Owners
====================================
Getting a new subnet operational step-by-step:

1. REGISTER THE SUBNET
   agcli subnet register
   (Costs the current subnet registration price — check with `agcli view network`)

2. GET STAKE ON YOUR VALIDATOR
   Your hotkey needs enough stake to set weights (typically 1000τ stake-weight).
   agcli stake add <amount> --netuid <your_netuid>
   Or ask delegators/nominators to stake to your hotkey.

3. SET INITIAL WEIGHTS
   agcli weights set --netuid <your_netuid> \"0:100\"
   (Set weights on at least one UID — usually yourself for bootstrapping)

4. CONFIGURE HYPERPARAMS (as subnet owner)
   agcli subnet set-param --netuid <N> --param <name> --value <val>
   Use --param list to see all available parameters.

5. REGISTER MINERS
   Miners register via burn or POW:
   agcli subnet register-neuron --netuid <N>
   agcli subnet pow --netuid <N>

6. ONBOARD VALIDATORS
   Other validators register and start setting weights.
   Your subnet becomes healthy when multiple validators independently score miners.

7. MONITOR
   agcli subnet metagraph --netuid <N>       — see all UIDs and scores
   agcli view subnet-analytics --netuid <N>  — emission and performance stats

Common pitfalls:
- Forgetting to set weights initially (no emissions flow if no weights set)
- Not having enough stake to pass the stake-weight minimum
- Setting tempo too low (frequent evals) or too high (slow feedback)";

const ALPHA: &str = "\
ALPHA TOKENS
============
Each subnet issues its own alpha token. Alpha represents your share of the
subnet's staking pool and emission flow.

When you stake TAO on a subnet:
- Your TAO enters the AMM pool.
- You receive alpha tokens in return (at the current exchange rate).
- Your alpha entitles you to a share of the subnet's emissions.

When you unstake:
- Your alpha goes back through the AMM.
- You receive TAO (at the current exchange rate — may differ from when you staked).

Alpha operations:
  agcli stake add 10 --netuid 5         # TAO → alpha (stake)
  agcli stake remove 10 --netuid 5      # alpha → TAO (unstake)
  agcli stake recycle-alpha 10 --netuid 5   # recycle alpha back to TAO
  agcli stake burn-alpha 10 --netuid 5      # permanently burn alpha (reduce supply)
  agcli stake transfer-stake --dest <ss58> --amount 10 --from 5 --to 5  # transfer to another coldkey

Key insight: alpha is always liquid through the AMM, but slippage matters on
small pools. Use `agcli view swap-sim` to preview swap amounts.";

const EMISSION: &str = "\
EMISSIONS
=========
TAO is emitted every block and distributed across subnets and within each subnet.

Block emission: ~1τ per block (halving schedule applies).

Distribution chain:
1. BLOCK EMISSION → split across all subnets based on root weights.
2. SUBNET EMISSION → split between:
   - alpha_out_emission: goes to alpha holders (stakers)
   - alpha_in_emission: goes into the AMM pool
   - tao_in_emission: goes into the TAO side of the pool
3. WITHIN SUBNET → Yuma consensus distributes to validators (dividends)
   and miners (incentive) based on weights and consensus scores.

Check emission rates:
  agcli view network                    — block emission, total stake
  agcli subnet show --netuid <N>        — subnet emission per tempo
  agcli view subnet-analytics <netuid>  — detailed emission breakdown
  agcli view staking-analytics          — your personal emission estimates";

const REGISTRATION: &str = "\
REGISTRATION
============
Neurons (miners/validators) must register on a subnet to participate.

Two registration methods:

1. BURN REGISTRATION — Pay TAO to register instantly.
   agcli subnet register-neuron --netuid <N>
   Cost varies per subnet and adjusts with demand (check `agcli subnet show --netuid <N>`).

2. POW REGISTRATION — Solve a proof-of-work puzzle.
   agcli subnet pow --netuid <N> --threads 8
   Free but competitive — difficulty adjusts to target registration rate.

After registration:
- You get a UID (0 to max_n-1) on the subnet.
- New registrants have an immunity period (immunity_period blocks) where
  they cannot be deregistered.
- If the subnet is full, the lowest-score neuron gets replaced.

Prerequisites:
- A wallet with a coldkey and hotkey: `agcli wallet create`
- TAO balance (for burn registration) or CPU time (for POW)";

const SUBNETS: &str = "\
SUBNETS
=======
Subnets are the core unit of Bittensor. Each subnet defines an incentive game
where validators evaluate miners on a specific task.

Subnet properties:
- netuid: Unique identifier (0 = root network)
- tempo: Evaluation frequency (blocks between consensus rounds)
- max_n: Maximum number of neurons (UIDs) on the subnet
- emission_value: TAO emitted to this subnet per tempo
- Hyperparameters: rho, kappa, immunity_period, weights settings, etc.

Subnet lifecycle:
  1. Owner registers subnet (pays registration price)
  2. Owner configures identity and hyperparameters
  3. Miners and validators register
  4. Validators set weights → emissions flow
  5. Subnet grows or gets dissolved

List subnets: `agcli subnet list`
Subnet details: `agcli subnet show --netuid <N>` (same as `agcli subnet info --netuid <N>`)
Hyperparams: `agcli subnet hyperparams --netuid <N>`";

const VALIDATORS: &str = "\
VALIDATORS
==========
Validators evaluate miners and set weights that determine emission distribution.

Validator responsibilities:
1. Run scoring infrastructure (query miners, evaluate responses).
2. Set weights based on miner performance: `agcli weights set --netuid N \"uid:weight,...\"`
3. Participate in Yuma consensus — accurate weights earn dividends.

Becoming a validator:
1. Register on the subnet: `agcli subnet register-neuron --netuid <N>`
2. Accumulate enough stake-weight (typically 1000τ).
3. Get validator_permit = true (top N validators by stake get permits).
4. Set weights each tempo.

Key metrics:
- validator_trust (VTrust): How well your weights match consensus.
- dividends: Your share of validator emissions.
- validator_permit: Whether you can set weights (top staked validators).

Common issues:
- 'SettingWeightsTooFast' — wait for rate limit to expire
- 'CommitRevealEnabled' — use commit+reveal workflow instead
- Low VTrust — your weights diverge from other validators";

const MINERS: &str = "\
MINERS
======
Miners perform the actual work on a subnet and earn incentive-based emissions.

Miner responsibilities:
1. Serve an axon endpoint for validators to query.
2. Respond to validator queries with high-quality results.
3. Stay competitive — low-performing miners get deregistered.

Becoming a miner:
1. Register on the subnet: `agcli subnet register-neuron --netuid <N>`
   Or via POW: `agcli subnet pow --netuid <N>`
2. Set your axon endpoint: `agcli serve axon --netuid N --ip <ip> --port <port>`
3. Run your miner software (subnet-specific).

Key metrics:
- incentive: Your emission share based on validator weights.
- trust: How much validators trust your responses.
- rank: Your position relative to other miners.
- pruning_score: How likely you are to be replaced (low = at risk).

Protect your position:
- Consistently produce high-quality responses.
- Monitor your scores: `agcli view neuron --netuid N <uid>`
- Watch for adversarial actors (UIDs copying your work).";

const IMMUNITY: &str = "\
IMMUNITY PERIOD
===============
Newly registered neurons get a grace period where they cannot be deregistered.

- Measured in blocks (subnet-specific, typically 4096 blocks ≈ 13.6 hours).
- During immunity, even if your scores are low, you won't be pruned.
- After immunity expires, the lowest-scoring neuron is replaced when a new
  registration arrives and the subnet is full.

Check a subnet's immunity period:
  agcli subnet hyperparams --netuid <N>  →  immunity_period

Why it matters:
- New miners need time to set up their infrastructure and start responding.
- Without immunity, new registrants would be instantly replaced by existing neurons.
- Use the immunity period to get your miner running and serving.";

const DELEGATION: &str = "\
DELEGATION / NOMINATION
=======================
Delegation allows TAO holders to stake their TAO through a validator (delegate),
earning a share of that validator's emissions.

How it works:
1. Validator sets their delegate take (0-11.11%): `agcli delegate increase-take <pct>`
2. Nominator stakes through the validator's hotkey: `agcli stake add <amount> --netuid N --hotkey-address <validator_hotkey>`
3. Emissions earned by the validator are split: validator keeps their take,
   rest is distributed pro-rata to all nominators.

For nominators:
- Research validators: `agcli delegate list` or `agcli view validators`
- Check take %: lower take = more emissions for you
- Check validator performance: high VTrust = consistently accurate weights
- Diversify across subnets and validators to manage risk

For validators:
- Set a competitive take: `agcli delegate decrease-take <pct>`
- Low take attracts more delegation → more total stake → more influence
- Your reputation matters — consistent performance attracts long-term delegators";

const CHILDKEYS: &str = "\
CHILDKEYS
=========
Childkey delegation allows a parent validator hotkey to share its stake-weight
with child hotkeys on specific subnets.

Use cases:
- Run multiple validator instances with shared stake.
- Delegate your weight to specialized scoring infrastructure.
- Split your validator operations across teams/machines.

Set children: `agcli stake set-children --netuid N --children \"proportion:hotkey,...\"`
Set childkey take: `agcli stake childkey-take <pct> --netuid N`

The proportion determines how much of the parent's stake-weight flows to
each child. Proportions are u64 values — use relative ratios.";

const ROOT_NETWORK: &str = "\
ROOT NETWORK (SN0)
==================
The root network (netuid 0) controls emission distribution across all subnets.

Root validators set weights on subnet netuids to determine how much emission
each subnet receives. Higher root weight → more emission for that subnet.

Joining root:
  agcli root register    — register your hotkey on the root network

Setting root weights:
  agcli root weights \"1:100,5:50,97:200\"   — weight netuids, not UIDs

Root is special:
- Validators on root must have high total stake.
- Root weights directly control the economic incentives for all subnets.
- Changing root weights shifts emission flow across the entire network.";

const PROXY: &str = "\
PROXY ACCOUNTS
==============
Proxy accounts allow one account to act on behalf of another with restricted
permissions, enhancing security for validators and stakers.

Add a proxy:
  agcli proxy add <delegate_ss58> --proxy-type staking

Proxy types:
- any: Full access (dangerous — use sparingly)
- staking: Can stake/unstake but not transfer
- non_transfer: Can do anything except transfer TAO
- governance: Can participate in governance votes
- owner: Subnet owner operations

Why use proxies:
- Keep your coldkey on an air-gapped machine.
- Give your automation/bot limited permissions via a proxy.
- Revoke access without moving funds.

List proxies: `agcli proxy list`
Remove proxy: `agcli proxy remove <delegate_ss58>`";

const COLDKEY_SWAP: &str = "\
COLDKEY SWAP
============
Coldkey swap allows you to migrate your account to a new coldkey. This is a
scheduled operation — it does not execute immediately.

How it works:
1. SCHEDULE: Submit a swap request specifying the new coldkey.
   agcli swap coldkey --new-coldkey <new_ss58>
   The chain records the swap with an execution block (typically days away).

2. WAITING PERIOD: The swap is pending for ColdkeySwapScheduleDuration blocks.
   During this window, the original coldkey still controls the account.

3. EXECUTION: At the execution block, the chain automatically transfers all
   balances, stakes, and permissions from the old coldkey to the new one.

Security implications:
- If someone gains access to your coldkey, they can schedule a swap.
- This is a CRITICAL security event — monitor with `agcli audit`.
- The audit command checks for scheduled swaps and flags them as [!!] high severity.

Detection:
  agcli audit --address <your_coldkey>
  # Shows: 'Coldkey swap scheduled! New coldkey: ... at block ...'

Prevention:
- Use proxy accounts with limited permissions for daily operations.
- Keep your coldkey on an air-gapped or hardware-secured machine.
- Monitor your account regularly with `agcli audit`.
- Set up alerts: `agcli subscribe events --filter all --account <your_coldkey>`

Note: The chain does NOT currently expose a cancel-swap extrinsic. Once scheduled,
a coldkey swap will execute at the scheduled block unless chain governance intervenes.
If you detect an unauthorized swap, contact the Bittensor community immediately.";

const GOVERNANCE: &str = "\
GOVERNANCE
==========
Bittensor uses on-chain governance for protocol upgrades, parameter changes,
and treasury disbursements. Proposals go through a democratic process.

Governance flow:
1. PROPOSAL: A member of the senate (triumvirate) or a council member submits a proposal.
2. VOTING: Token-weighted voting — stake counts as voting power.
3. ENACTMENT: If the proposal passes the vote threshold and any required
   senate approval, it is enacted after a delay period.

Proposal types:
- Runtime upgrades (code changes to the chain)
- Parameter changes (emission schedule, registration costs, hyperparams)
- Treasury proposals (fund allocation from the treasury)

How to participate:
- Vote on proposals using your staked TAO weight.
- Delegate your vote to a trusted validator.
- Monitor active proposals through chain governance tools.

Key parameters:
- Proposals require supermajority or simple majority depending on type.
- Enactment delays give the community time to respond.
- Emergency proposals can bypass some delays with senate approval.";

const SENATE: &str = "\
SENATE (TRIUMVIRATE)
====================
The Senate (also called the Triumvirate) is a small governance body on Bittensor
with elevated permissions for critical chain decisions.

Composition:
- Members are the top validators by total delegated stake.
- Senate size is limited (typically 12 seats).
- Membership is dynamic — it updates as validator stake rankings change.

Powers:
- Can submit governance proposals directly.
- Some proposal types require senate approval to pass.
- Acts as a safety check on governance actions.
- Can fast-track emergency proposals.

How it works:
- Senate membership is automatic for top validators by delegation.
- No explicit application — rack up enough delegated stake and you qualify.
- Losing stake below the threshold means losing your senate seat.

Practical implications:
- Delegating to a validator also grants them governance influence.
- Consider a validator's governance track record when choosing who to delegate to.
- Senate votes are on-chain and transparent.";

const MEV_SHIELD: &str = "\
MEV SHIELD
==========
MEV (Maximal Extractable Value) Shield is a Bittensor-specific pallet that
protects users from transaction ordering manipulation by block producers.

What is MEV?
- Block producers can reorder, insert, or censor transactions within a block.
- On DeFi chains this enables front-running, sandwich attacks, and arbitrage.
- On Bittensor, MEV could affect staking, weight-setting, and AMM trades.

How MevShield works:
- The MevShield pallet adds protection against transaction ordering attacks.
- It uses commit-reveal patterns and timing constraints to make ordering
  manipulation unprofitable or impossible.
- Transactions within a protected window are processed fairly regardless of
  ordering within the block.

Protected operations:
- Stake/unstake operations through the AMM (prevents sandwich attacks).
- Weight commits/reveals (prevents front-running weight updates).
- Swap operations that interact with dynamic TAO pools.

For users:
- MEV protection is automatic — no extra flags needed.
- The protection is built into the chain runtime.
- Large AMM trades still face slippage from the constant-product formula,
  but won't face additional losses from block producer manipulation.
- Use limit orders (`agcli stake add-limit`) for additional price protection.";

const LIMITS: &str = "\
NETWORK & CHAIN LIMITS
======================
Bittensor enforces several limits at the chain level that affect miners,
validators, and stakers.

Weight setting:
- Minimum 1000 stake-weight to set weights directly (use commit-reveal otherwise).
- weights_rate_limit: minimum blocks between weight-set calls per validator per subnet.
- max_weights_limit: maximum number of UIDs that can be included in a single weight vector.
- min_allowed_weights: minimum UIDs required in a weight vector for it to be valid.

Registration:
- max_regs_per_block: cap on burn-registrations processed per block network-wide.
- target_regs_per_interval: target registrations per adjustment_interval, used to
  auto-adjust the burn cost.
- Immunity period: newly registered neurons cannot be deregistered for N blocks.

Staking:
- No minimum stake amount, but very small stakes earn negligible emission.
- Rate limit on stake/unstake operations may apply during high-traffic periods.
- Childkey delegation changes have a cooldown period before taking effect.

Serving:
- serving_rate_limit: minimum blocks between axon metadata updates.
- Axon IP/port must be publicly reachable for miners to receive queries.

General:
- Block time: ~12 seconds.
- Blocks per day: ~7200.
- Max subnets: determined by governance (currently ~64).
- Check current limits: `agcli subnet hyperparams --netuid <N>`";

const HYPERPARAMS: &str = "\
SUBNET HYPERPARAMETERS
======================
Each subnet has ~32 tunable parameters stored on-chain. Some are owner-settable,
others require the chain sudo key (root governance). Full reference: `agcli docs/hyperparameters.md`

View them: `agcli subnet hyperparams --netuid <N>`

EPOCH & TIMING
- tempo (u16, sudo): blocks per epoch. Default 360 (~72 min). Controls evaluation frequency.
- activity_cutoff (u16): blocks of inactivity before deregistration. Default ~5000.
- immunity_period (u16): blocks a new neuron is immune. Default ~4096.

CONSENSUS (deep protocol — change with caution)
- rho (u16, sudo): emission adjustment parameter. Default 10. Higher = more aggressive ranking.
- kappa (u16, sudo): consensus threshold. Default 32767 (≈50%). Higher = stricter agreement needed.
- yuma (bool, sudo): enable/disable Yuma consensus entirely.

WEIGHTS
- min_allowed_weights (u16): minimum UIDs per weight vector. Prevents narrow evaluation.
- max_weight_limit (u16): cap per-UID weight. 65535 = no cap, lower = forced distribution.
- weights_version (u64): validators must match this version or weight-set is rejected.
- weights_rate_limit (u64): min blocks between weight-set calls. 0 = unlimited.

COMMIT-REVEAL (anti-copying protection)
- commit_reveal_weights_enabled (bool): require two-phase weight submission.
- commit_reveal_weights_interval (u64): blocks between commit and reveal.
- commit_reveal_version (u64): protocol version for commit-reveal.

REGISTRATION & DIFFICULTY (feedback loop controls)
- registration_allowed (bool): master switch for new registrations.
- pow_registration_allowed (bool): allow PoW registration specifically.
- difficulty (u64): current PoW difficulty (usually auto-adjusted).
- min_difficulty / max_difficulty (u64): bounds for auto-adjustment.
- target_regs_per_interval (u16): target registrations per adjustment window.
- adjustment_interval (u16): blocks between difficulty recalculations.
- adjustment_alpha (u64): EMA smoothing — high = slow adjustment, low = volatile.
- min_burn / max_burn (u64): floor/ceiling for TAO burn cost (in RAO, 1 TAO = 10^9 RAO).
- max_regs_per_block (u16): hard cap on registrations per block.

NETWORK SIZE
- max_allowed_uids (u16, sudo): max neurons on subnet. Default 256.
- min_allowed_uids (u16, sudo): min neurons on subnet.
- max_allowed_validators (u16, sudo): max validator permit slots. Default 128.
- min_non_immune_uids (u16, sudo): ensures N neurons are always deregistration-eligible.

BONDS & DIVIDENDS
- bonds_moving_average (u64): smoothing for bond calcs. Higher = more historical weight.
- bonds_penalty (u16): penalizes validators deviating from consensus weights.
- bonds_reset_enabled (bool): allow periodic bond zeroing (prevents entrenchment).
- liquid_alpha_enabled (bool): dynamic bond alpha based on validator agreement.

SERVING
- serving_rate_limit (u64): min blocks between serve_axon (IP update) calls.

HOW THEY INTERACT
- Registration: target_regs + adjustment_interval + alpha + min/max burn/difficulty form a
  feedback loop that auto-tunes registration cost every adjustment_interval blocks.
- Deregistration: requires immunity expired + inactive > cutoff + subnet full + new registrant.
- Emissions: weights → Yuma consensus (rho, kappa) → bonds (moving avg, penalty, liquid alpha) → dividends.

Changing hyperparams:
- Owner: `agcli subnet set-param --netuid <N> --param <name> --value <val>`
- Sudo: `agcli admin set-tempo --netuid <N> --tempo <val> --sudo-key //Alice`
- List all: `agcli subnet set-param --netuid 1 --param list` or `agcli admin list`";

const AXON: &str = "\
AXON (SERVING ENDPOINT)
=======================
An axon is the network-facing endpoint that a miner or validator exposes
so other nodes can communicate with it.

What it stores on-chain:
- IP address (IPv4 or IPv6)
- Port number
- Protocol version
- Software version
- Placeholder (reserved field, usually 0)

How it works:
- Miners call `serve_axon` to register their IP:port on a specific subnet.
- Validators query on-chain axon info to discover miner endpoints.
- The serving_rate_limit hyperparameter controls how often axon info can be updated.

Viewing axon info:
- `agcli subnet metagraph --netuid <N> --uid <uid>` shows a neuron's axon details.
- Entries with IP 0.0.0.0 or port 0 indicate a neuron that hasn't set its axon.

For miners:
- Your axon must be reachable from the public internet.
- Set it early after registration — validators need it to send queries.
- Update if your IP changes (subject to serving_rate_limit).
- Common setup: run your miner behind a reverse proxy or directly with a public IP.

For validators:
- Axon info helps you verify that miners are actually online.
- Neurons with stale or missing axon info may be inactive.
- The `last_update` field in the metagraph shows when the neuron last interacted
  with the chain (not necessarily axon-specific).";

const TAKE: &str = "\
VALIDATOR / DELEGATE TAKE
=========================
Take is the percentage of emissions a validator keeps before distributing dividends
to their delegators (nominators).

How it works:
- A validator earns dividends from Yuma consensus based on weight accuracy.
- Before distributing to delegators, the validator takes a cut (the 'take').
- Remaining dividends are split proportionally among delegators by stake.

Take range:
- Minimum: 0% (validator keeps nothing — all dividends go to delegators)
- Maximum: 11.11% (18% of the u16 max, capped by chain logic)
- Default: typically 11.11% for new validators

Adjusting take:
  agcli delegate decrease-take <pct>    # Lower your take (attracts more delegation)
  agcli delegate increase-take <pct>    # Raise your take (takes effect after delay)

Important: take increases are delayed by the TakeDecreaseDelay hyperparameter
(typically ~7 days) to prevent bait-and-switch tactics. Take decreases
are instant — lowering take to attract stake is immediate.

Strategy:
- Low take attracts more delegators → more total stake → more influence.
- High take keeps more for yourself but discourages delegation.
- Top validators often run 5-9% take as a competitive balance.

Check take: `agcli delegate list` shows take % for all delegates.";

const RECYCLE: &str = "\
RECYCLE & BURN ALPHA
====================
Alpha tokens can be recycled (converted back to TAO) or burned (permanently
destroyed). Both operations reduce the alpha supply on a subnet.

RECYCLE ALPHA:
- Converts your alpha tokens back to TAO through the AMM.
- The alpha goes back into the pool, increasing alpha_in.
- You receive TAO from the pool, decreasing tao_in.
- Subject to AMM slippage on shallow pools.
  agcli stake recycle-alpha <amount> --netuid <N>

BURN ALPHA:
- Permanently destroys alpha tokens, reducing total supply.
- No TAO is returned — the tokens are gone forever.
- Burning increases the value of remaining alpha (deflationary).
- Used by subnet operators to manage token economics.
  agcli stake burn-alpha <amount> --netuid <N>

When to recycle vs burn:
- Recycle: You want your TAO back. Acts like a normal unstake through the AMM.
- Burn: You want to intentionally reduce supply to boost the subnet's alpha value.
  This is a deliberate economic action, not a recovery mechanism.

Slippage warning:
- Both recycle and large unstakes go through the AMM.
- Check the pool depth first: `agcli subnet show --netuid <N>` (look at tao_in).
- Simulate before acting: `agcli view swap-sim --netuid <N> --alpha <amount>`";

const POW_REGISTRATION: &str = "\
PROOF-OF-WORK REGISTRATION
===========================
PoW registration lets you register a neuron (miner/validator) on a subnet by
solving a computational puzzle instead of paying the burn fee.

How it works:
1. The chain publishes a target difficulty and a block hash as the 'input'.
2. Your node iterates through nonces until it finds one that, when hashed with
   the input, produces a hash below the target difficulty.
3. Submit the solution on-chain: `agcli subnet pow --netuid <N> --threads 8`
4. If valid and below difficulty, you get a UID on the subnet.

Difficulty adjustment:
- The chain adjusts difficulty based on the target_regs_per_interval parameter.
- More PoW registrations → higher difficulty → harder puzzles.
- Fewer registrations → lower difficulty → easier puzzles.
- Check current difficulty: `agcli subnet hyperparams --netuid <N>` → difficulty

Practical tips:
- Use `--threads` to set the number of CPU threads for parallel searching.
- PoW is competitive — someone else may solve it before you.
- Solutions expire quickly — compute and submit within the same block window.
- Some subnets have very high difficulty (hundreds of thousands), making PoW
  impractical. Check difficulty before spending CPU time.
- Energy cost: compare the electricity cost of PoW vs the burn registration fee.
  Often burn is cheaper for established subnets.

When to use PoW:
- You have spare CPU/GPU capacity and want to avoid spending TAO.
- The burn cost is high relative to your TAO holdings.
- You're running a bootstrapping operation on a new (low-difficulty) subnet.

Key hyperparams:
- difficulty: current PoW target difficulty
- min_difficulty / max_difficulty: difficulty bounds
- adjustment_interval: blocks between difficulty adjustments
- target_regs_per_interval: target registrations that drive adjustment";

const ARCHIVE: &str = "\
ARCHIVE NODES & HISTORICAL DATA
================================
Standard Bittensor nodes prune old block state to save disk space. Archive nodes
retain the full state for every block, enabling historical queries.

Why archive nodes matter:
- Standard (pruned) nodes only keep recent state (~256 blocks).
- Querying old blocks on a pruned node returns 'State already discarded' errors.
- Archive nodes store every block's state, so you can query any historical block.

Using archive nodes in agcli:
  # Use the built-in archive network preset
  agcli balance --at-block 3000000 --network archive

  # Or specify a custom archive endpoint
  agcli subnet metagraph --netuid 1 --at-block 3000000 --endpoint wss://your-archive:443

  # Set as default in config
  agcli config set --key network --value archive

Commands that support --at-block (historical wayback):
  agcli balance --at-block N
  agcli stake list --at-block N
  agcli subnet list --at-block N
  agcli subnet show --netuid X --at-block N
  agcli subnet metagraph --netuid X --at-block N
  agcli view network --at-block N
  agcli view portfolio --at-block N
  agcli view dynamic --at-block N
  agcli view neuron --netuid X --uid Y --at-block N
  agcli view validators --at-block N
  agcli view account --at-block N

Block explorer:
  agcli block latest               # Head: same RPC order as `handle_block` Latest; e2e `block_latest_preflight` in Phase 20 test_block_queries
  agcli block info --number N      # get_block_hash → header + extrinsics + timestamp; e2e `block_info_preflight` in Phase 20 test_block_queries
  agcli block range --from A --to B  # Concurrent hash batch then per-block ext+ts; e2e `block_range_preflight` in Phase 20 test_block_queries

Historical diff (compare state between two blocks):
  agcli diff portfolio --block1 A --block2 B [--address SS58]
  agcli diff subnet --netuid X --block1 A --block2 B
  agcli diff network --block1 A --block2 B
  agcli diff metagraph --netuid X --block1 A --block2 B   # changed/new neurons only; e2e Phase 20 `test_diff_queries`

Known archive providers:
- OnFinality:  wss://bittensor-finney.api.onfinality.io/public-ws (built-in)
- Self-hosted: Run a subtensor node with --pruning=archive

Tips:
- Archive queries are slower than standard queries due to state retrieval.
- The --network archive flag automatically uses a public archive endpoint.
- For heavy historical analysis, consider running your own archive node.
- Auto-detection: if --at-block hits pruned state, agcli suggests using --network archive.";

const DIFF: &str = "\
HISTORICAL DIFF
===============
Compare chain state between two blocks to see what changed over time.

The `agcli diff` command fetches state snapshots at two block heights and shows
a side-by-side comparison with deltas. Requires an archive node for older blocks.

Sub-commands:

  agcli diff portfolio --block1 4000000 --block2 5000000 [--address SS58]
    Compare an account's free balance and total stake between two blocks.
    If no --address is given, the default wallet coldkey is used.
    Shows: free balance, total stake, total value, and stake position count.

  agcli diff subnet --netuid 1 --block1 4000000 --block2 5000000
    Compare a subnet's economic state between two blocks.
    Shows: TAO in pool, alpha price, emission, tempo, and owner.

  agcli diff network --block1 4000000 --block2 5000000
    Compare network-wide stats between two blocks.
    Shows: total issuance, total stake, staking ratio, and subnet count.

  agcli diff metagraph --netuid 1 --block1 4000000 --block2 5000000
    Compare lite metagraph snapshots: lists neurons with stake/emission/incentive
    deltas (or new UIDs / hotkey replacements). Empty output means no changes
    above CLI thresholds.

Tips:
- Use --network archive for blocks older than the pruning window (~256 blocks).
- All diff commands support --output json for machine-readable output.
- Use `agcli block range --from A --to B` to scan block metadata first,
  then drill into specific blocks with diff commands.
- Queries run in parallel (block1 and block2 fetched concurrently).
- See also: `agcli explain archive` for archive node setup.";

const OWNER_WORKFLOW: &str = "\
SUBNET OWNER WORKFLOW
=====================
Complete guide for registering, configuring, and managing a subnet with agcli.

PHASE 1: PREPARATION
---------------------
  # Check current registration cost
  agcli view network

  # Ensure your wallet has enough TAO for registration lock
  agcli balance

  # Understand key concepts first
  agcli explain subnet
  agcli explain hyperparams

PHASE 2: REGISTER YOUR SUBNET
-------------------------------
  # Read-only: current lock amount for subnet register / register-leased
  agcli subnet create-cost

  # Register a new subnet (costs the current subnet lock amount)
  agcli subnet register

  # Or register with identity metadata in one extrinsic (optional --github, --url, …)
  agcli subnet register-with-identity --name 'My Subnet' --github opentensor/subtensor

  # Note the netuid printed in the output — you'll use it everywhere.

  # Set on-chain identity (helps validators and miners find you)
  agcli identity set-subnet --netuid <N> --name 'My Subnet' --github 'https://github.com/...'

PHASE 3: CONFIGURE HYPERPARAMETERS
------------------------------------
  # View current hyperparameters
  agcli subnet hyperparams --netuid <N>

  # List all settable parameters
  agcli subnet set-param --netuid <N> --param list

  # Common initial configuration:
  agcli subnet set-param --netuid <N> --param tempo --value 360
  agcli subnet set-param --netuid <N> --param max_allowed_uids --value 256
  agcli subnet set-param --netuid <N> --param immunity_period --value 4096
  agcli subnet set-param --netuid <N> --param min_allowed_weights --value 1
  agcli subnet set-param --netuid <N> --param max_allowed_validators --value 64
  agcli subnet set-param --netuid <N> --param registration_allowed --value true
  agcli subnet set-param --netuid <N> --param weights_rate_limit --value 100

  # Enable commit-reveal for weight privacy
  agcli subnet set-param --netuid <N> --param commit_reveal_weights_enabled --value true
  agcli subnet set-param --netuid <N> --param commit_reveal_period --value 1

  # Enable liquid alpha for dynamic incentives
  agcli subnet set-param --netuid <N> --param liquid_alpha_enabled --value true

PHASE 4: MONITOR YOUR SUBNET
------------------------------
  # Watch live tempo countdown and rate limits
  agcli subnet watch --netuid <N>

  # Check metagraph: who's registered, their weights, stake
  agcli subnet metagraph --netuid <N>

  # Check health: miner status, weight coverage
  agcli subnet health --netuid <N>

  # Monitor registrations, weight changes, anomalies in real-time
  agcli subnet monitor --netuid <N>

  # View emission distribution across UIDs
  agcli subnet emissions --netuid <N>

  # Check current registration cost and difficulty trend
  agcli subnet cost --netuid <N>

  # Probe all miners' axon endpoints for health
  agcli subnet probe --netuid <N>

  # View pending weight commits
  agcli subnet commits --netuid <N>

PHASE 5: LIQUIDITY MANAGEMENT
-------------------------------
  # View AMM pool state for your subnet
  agcli subnet liquidity --netuid <N>

  # Toggle user liquidity participation
  agcli liquidity toggle --netuid <N> --enable true

  # Simulate a TAO-to-alpha swap to check pricing
  agcli view swap-sim --netuid <N> --tao 100

PHASE 6: ONGOING OPERATIONS
-----------------------------
  # Compare subnet state between blocks (track changes over time)
  agcli diff subnet --netuid <N> --block1 <old> --block2 <new>

  # Save metagraph snapshots for historical comparison
  agcli subnet metagraph --netuid <N> --save
  agcli subnet cache-list --netuid <N>
  agcli subnet cache-diff --netuid <N>   # unknown --netuid → exit 12 like subnet show
  agcli subnet emission-split --netuid <N>   # mechanism split; unknown --netuid → exit 12 like subnet show
  agcli subnet mechanism-count --netuid <N>
  agcli subnet set-mechanism-count --netuid <N> --count K   # owner; unknown --netuid → exit 12 before wallet (e2e `subnet_owner_mechanism_writes`)
  agcli subnet set-emission-split --netuid <N> --weights 50,50   # owner; parse/weight validation then exit 12 before wallet if SN missing (--yes skips confirm)
  agcli subnet check-start --netuid <N>   # active / can_start / tempo; unknown --netuid → exit 12 like subnet show
  agcli subnet set-param --netuid <N> --param list   # owner hyperparams; unknown --netuid → exit 12 before wallet (incl. list mode)
  agcli subnet set-symbol --netuid <N> --symbol ALPHA   # owner token symbol; unknown --netuid → exit 12 after local symbol validation
  agcli subnet trim --netuid <N> --max-uids 256   # owner max UID cap; unknown --netuid → exit 12 before wallet (--yes skips confirm)
  agcli subnet register-neuron --netuid <N>   # burn register; unknown --netuid → exit 12 before hotkey unlock
  agcli subnet list   # all subnets at pinned head; e2e `subnet_list` → `list_subnets` (no wallet / no --netuid)
  agcli subnet create-cost   # subnet creation lock (read-only); same RPC as e2e `subnet_create_cost` line
  agcli subnet register   # plain new subnet (no pre-submit cost read); e2e `subnet_register_plain` → same `get_subnet_registration_cost` as create-cost
  agcli subnet register-with-identity --name '...'   # register + identity; e2e `subnet_register_with_identity` → get_subnet_identity
  agcli subnet register-leased [--end-block N]   # new leased subnet; lock cost: `subnet create-cost` (same RPC as e2e log)
  agcli subnet pow --netuid <N> --threads 4   # POW register; same preflight as register-neuron
  agcli subnet snipe --netuid <N> --watch   # require_subnet_exists before stream; register modes + e2e sections 6b–6g
  agcli subnet dissolve --netuid <N>   # owner schedule dissolve; unknown --netuid → exit 12 before wallet (--yes skips confirm)
  agcli subnet terminate-lease --netuid <N>   # owner end leased subnet; unknown --netuid → exit 12 before wallet
  agcli weights show --netuid <N> [--hotkey-address SS58] [--limit L]   # read-only; require_subnet_exists_for_weights_cmd then get_all_weights + get_neurons_lite (+ get_weights_for_uid); e2e `weights_show_preflight` + Phase 37 `test_all_weights`
  agcli weights set --netuid <N> --weights 0:100   # direct set_weights when CR off; hyperparams before wallet; e2e `weights_set_preflight` + Phase 7
  agcli weights commit --netuid <N> --weights 0:100   # commit-reveal phase 1; hyperparams existence check before wallet; e2e `weights_commit_preflight` + Phase 17
  agcli weights reveal --netuid <N> --weights 0:100 --salt S   # commit-reveal phase 2; same preflight as commit; salt → u16 pairs; e2e `weights_reveal_preflight` + Phase 17
  agcli weights commit-reveal --netuid <N> --weights 0:100 [--wait]   # one-shot CR or set_weights fallback; strict hyperparams RPC; e2e `weights_commit_reveal_preflight` + Phase 17
  agcli weights status --netuid <N>   # pending CR commits for default hotkey; preflight like commit/reveal; post-wallet try_join reads; e2e `weights_status_preflight` in Phase 17 `test_reveal_weights_rejected_without_prior_commit`
  agcli weights commit-timelocked --netuid <N> --weights 0:100 --round R   # drand timelock commit; hyperparams preflight then wallet; SDK loads CommitRevealWeightsVersion at submit; e2e `weights_commit_timelocked_preflight` in Phase 17 `test_commit_timelocked_weights_rejected_when_incorrect_commit_reveal_version`
  agcli weights set-mechanism --netuid <N> --mechanism-id 0 --weights 0:100   # set_mechanism_weights; require_subnet_exists_for_weights_cmd before wallet; --dry-run JSON only; e2e `weights_set_mechanism_preflight` in Phase 5 `test_set_mechanism_weights`
  agcli weights commit-mechanism --netuid <N> --mechanism-id 0 --hash 0x...   # commit_mechanism_weights; same preflight; --hash = 32-byte hex (blake2 over uids+weights+salt like `weights commit`); e2e `weights_commit_mechanism_preflight` in Phase 5 `test_commit_mechanism_weights`
  agcli weights reveal-mechanism --netuid <N> --mechanism-id 0 --weights 0:65535 --salt S   # reveal_mechanism_weights; same preflight; salt → u16 pairs like `weights reveal`; e2e `weights_reveal_mechanism_preflight` in Phase 5 `test_reveal_mechanism_weights`
  agcli block latest   # read-only head: get_block_number → get_block_hash → extrinsic_count + timestamp; e2e `block_latest_preflight` in Phase 20 `test_block_queries`
  agcli block info --number N   # get_block_hash(N) → try_join!(header, extrinsic_count, timestamp); e2e `block_info_preflight` in Phase 20 `test_block_queries`
  agcli block range --from A --to B   # span ≤1000; try_join_all(get_block_hash) then try_join_all per-hash ext+ts; e2e `block_range_preflight` in Phase 20 `test_block_queries`
  agcli diff portfolio --block1 A --block2 B [--address SS58]   # try_join!(get_block_hash×2) then balance + stake maps at each hash; e2e `diff_portfolio_preflight` in Phase 20 `test_diff_queries`
  agcli diff subnet --netuid N --block1 A --block2 B   # same hashes → try_join!(get_dynamic_info_at_block×2); missing subnet at a height → exit 12; e2e `diff_subnet_preflight` in Phase 20 `test_diff_queries`
  agcli diff network --block1 A --block2 B   # six-way try_join: issuance, total_stake, all_subnets ×2 blocks; e2e `diff_network_preflight` in Phase 20 `test_diff_queries`
  agcli diff metagraph --netuid N --block1 A --block2 B   # try_join!(get_neurons_lite_at_block×2) then local UID diff; e2e `diff_metagraph_preflight` in Phase 20 `test_diff_queries`
  agcli subscribe blocks   # long-running: subscribe_finalized → extrinsics count per block; Ctrl+C; e2e Phase 26 `test_subscribe_blocks`
  agcli subscribe events --filter all [--netuid <N>] [--account SS58]   # validate_event_filter + optional SS58 before subscribe_finalized → block.events(); e2e Phase 26 `subscribe_events_preflight` log in `test_subscribe_events_preflight`
  agcli doctor   # top-level: connect_network → block# + total_networks + 3×get_block_number + disk cache + wallet path; always exit 0 (read OK/FAIL rows or JSON); e2e Phase 20 `doctor_preflight` in `test_doctor_preflight`
  agcli balance [--address SS58]   # get_balance_ss58; --at-block → get_block_hash + get_balance_at_block; --watch polls; invalid --threshold → exit 12; e2e Phase 20 `balance_preflight` in `test_balance_preflight`
  agcli transfer --dest SS58 --amount τ   # transfer_allow_death; validate_ss58 + validate_amount + get_balance_ss58 preflight; `transfer-all` / `transfer-keep-alive` variants; invalid dest/amount → exit 12; e2e Phase 20 `transfer_preflight` in `test_transfer_preflight`
  agcli stake list [--address SS58]   # get_stake_for_coldkey; --at-block → get_block_hash + get_stake_for_coldkey_at_block; invalid --address → exit 12 + stake.md hint; e2e Phase 20 `stake_list_preflight` in `test_stake_list_preflight`
  agcli stake add --amount τ --netuid N [--max-slippage PCT]   # validate_netuid + validate_amount + check_spending_limit → unlock → get_balance → optional slippage try_join (alpha price + sim swap); insufficient/slippage → exit 13; e2e Phase 20 `stake_add_preflight` in `test_stake_add_preflight`
  agcli stake remove --amount τ --netuid N [--max-slippage PCT]   # validate_netuid + validate_amount (`unstake amount`) → unlock → optional sell-path slippage try_join (`current_alpha_price` + `sim_swap_alpha_for_tao`); slippage → exit 13; e2e Phase 20 `stake_remove_preflight` in `test_stake_remove_preflight`
  agcli stake move --amount α --from SRC --to DST [--hotkey-address SS58]   # validate_netuid×2 → same SN bail → validate_amount (`move amount`) → check_spending_limit(`--to`) → unlock → move_stake; no slippage/balance pre-read; invalid amount → exit 12 + stake.md hint; e2e Phase 20 `stake_move_preflight` in `test_stake_move_preflight`
  agcli stake swap --amount α --from SRC --to DST [--hotkey-address SS58]   # validate_netuid×2 → same SN bail → validate_amount (`swap amount`) → check_spending_limit(`--to`) → unlock → swap_stake; no slippage/balance pre-read; invalid amount → exit 12 + stake.md hint; e2e Phase 20 `stake_swap_preflight` in `test_stake_swap_preflight`
  agcli stake unstake-all [--hotkey-address SS58]   # unlock_and_resolve only (validate_ss58 `hotkey-address` when flag set); no netuid/amount/spending-limit pre-read; bad hotkey SS58 → exit 12 + stake.md hint; e2e Phase 20 `stake_unstake_all_preflight` in `test_stake_unstake_all_preflight`
  agcli view portfolio [--address SS58]   # resolve/validate coldkey; latest: pin_latest_block → try_join(balance, stakes, dynamic); --at-block: get_block_hash → try_join(balance, stakes); invalid --address → exit 12 + view.md hint; e2e Phase 20 `view_portfolio_preflight` in `test_view_portfolio_preflight`

  # Security audit your account
  agcli audit

TIPS FOR OWNERS:
- Use --dry-run on any write command to preview without submitting.
- Use --output json for automation pipelines.
- Set AGCLI_NETWORK=finney and AGCLI_WALLET=<name> in your shell profile.
- Set-param shows the current value before confirming changes.
- Run `agcli doctor` to verify connectivity and wallet health (`docs/commands/doctor.md`; exit 0 with per-row OK/FAIL).
- Run `agcli balance` or `agcli balance --address …` for free TAO (`docs/commands/balance.md`; e2e `balance_preflight`).
- Use `agcli transfer` / `transfer-all` / `transfer-keep-alive` for coldkey TAO moves (`docs/commands/transfer.md`; e2e `transfer_preflight`).
- Use `agcli stake list` / `stake list --address …` for staked positions (`docs/commands/stake.md`; e2e `stake_list_preflight`).
- Use `agcli stake add` to lock TAO as alpha on a subnet (`docs/commands/stake.md`; e2e `stake_add_preflight`, extrinsic coverage `test_add_remove_stake`).
- Use `agcli stake remove` to convert alpha back to free TAO (`docs/commands/stake.md`; e2e `stake_remove_preflight`, same Phase 8 extrinsic test).
- Use `agcli stake move` to shift alpha between subnets on the same hotkey (`docs/commands/stake.md`; e2e `stake_move_preflight`).
- Use `agcli stake swap` for the AMM **`swap_stake`** path between subnets on the same hotkey (`docs/commands/stake.md`; e2e `stake_swap_preflight`).
- Use `agcli stake unstake-all` to exit every subnet position for one hotkey in one extrinsic (`docs/commands/stake.md`; e2e `stake_unstake_all_preflight`).
- Use `agcli view portfolio` for balance + priced stake positions (`docs/commands/view.md`; e2e `view_portfolio_preflight`).
- Use `agcli subnet monitor --netuid <N> --json` for structured event streaming.";

#[cfg(test)]
mod tests {
    use super::*;

    // --- Known topics return Some ---

    #[test]
    fn known_topic_tempo() {
        assert!(explain("tempo").is_some());
    }

    #[test]
    fn known_topic_yuma() {
        assert!(explain("yuma").is_some());
    }

    #[test]
    fn known_topic_amm() {
        assert!(explain("amm").is_some());
    }

    #[test]
    fn known_topic_emission() {
        assert!(explain("emission").is_some());
    }

    #[test]
    fn known_topic_subnet() {
        assert!(explain("subnet").is_some());
    }

    #[test]
    fn known_topic_registration() {
        assert!(explain("registration").is_some());
    }

    #[test]
    fn known_topic_validator() {
        assert!(explain("validator").is_some());
    }

    #[test]
    fn known_topic_miner() {
        assert!(explain("miner").is_some());
    }

    #[test]
    fn known_topic_proxy() {
        assert!(explain("proxy").is_some());
    }

    #[test]
    fn known_topic_governance() {
        assert!(explain("governance").is_some());
    }

    #[test]
    fn known_topic_root() {
        assert!(explain("root").is_some());
    }

    #[test]
    fn known_topic_bootstrap() {
        assert!(explain("bootstrap").is_some());
    }

    #[test]
    fn known_topic_alpha() {
        assert!(explain("alpha").is_some());
    }

    #[test]
    fn known_topic_immunity() {
        assert!(explain("immunity").is_some());
    }

    #[test]
    fn known_topic_delegate() {
        assert!(explain("delegate").is_some());
    }

    #[test]
    fn known_topic_childkey() {
        assert!(explain("childkey").is_some());
    }

    #[test]
    fn known_topic_senate() {
        assert!(explain("senate").is_some());
    }

    #[test]
    fn known_topic_hyperparams() {
        assert!(explain("hyperparams").is_some());
    }

    #[test]
    fn known_topic_axon() {
        assert!(explain("axon").is_some());
    }

    #[test]
    fn known_topic_take() {
        assert!(explain("take").is_some());
    }

    #[test]
    fn known_topic_recycle() {
        assert!(explain("recycle").is_some());
    }

    #[test]
    fn known_topic_pow() {
        assert!(explain("pow").is_some());
    }

    #[test]
    fn known_topic_archive() {
        assert!(explain("archive").is_some());
    }

    #[test]
    fn known_topic_diff() {
        assert!(explain("diff").is_some());
    }

    #[test]
    fn known_topic_owner_workflow() {
        assert!(explain("ow").is_some());
    }

    #[test]
    fn owner_workflow_mentions_view_portfolio_preflight() {
        let t = explain("ow").expect("owner workflow topic");
        assert!(
            t.contains("view_portfolio_preflight"),
            "Phase 6 cheat sheet should reference e2e view portfolio preflight"
        );
    }

    #[test]
    fn owner_workflow_mentions_stake_move_preflight() {
        let t = explain("ow").expect("owner workflow topic");
        assert!(
            t.contains("stake_move_preflight"),
            "Phase 6 cheat sheet should reference e2e stake move preflight"
        );
    }

    #[test]
    fn owner_workflow_mentions_stake_swap_preflight() {
        let t = explain("ow").expect("owner workflow topic");
        assert!(
            t.contains("stake_swap_preflight"),
            "Phase 6 cheat sheet should reference e2e stake swap preflight"
        );
    }

    #[test]
    fn owner_workflow_mentions_stake_unstake_all_preflight() {
        let t = explain("ow").expect("owner workflow topic");
        assert!(
            t.contains("stake_unstake_all_preflight"),
            "Phase 6 cheat sheet should reference e2e stake unstake-all preflight"
        );
    }

    // --- Alias tests ---

    #[test]
    fn alias_cr_for_commit_reveal() {
        let cr = explain("cr");
        let full = explain("commit-reveal");
        assert!(cr.is_some());
        assert_eq!(cr, full);
    }

    #[test]
    fn alias_dtao_for_amm() {
        let dtao = explain("dtao");
        let amm = explain("amm");
        assert!(dtao.is_some());
        assert_eq!(dtao, amm);
    }

    #[test]
    fn alias_pool_for_amm() {
        let pool = explain("pool");
        let amm = explain("amm");
        assert!(pool.is_some());
        assert_eq!(pool, amm);
    }

    #[test]
    fn alias_dynamic_tao_for_amm() {
        let dtao = explain("dynamic-tao");
        let amm = explain("amm");
        assert!(dtao.is_some());
        assert_eq!(dtao, amm);
    }

    #[test]
    fn alias_yuma_consensus() {
        let yc = explain("yuma-consensus");
        let yuma = explain("yuma");
        assert!(yc.is_some());
        assert_eq!(yc, yuma);
    }

    #[test]
    fn alias_gov_for_governance() {
        let gov = explain("gov");
        let full = explain("governance");
        assert!(gov.is_some());
        assert_eq!(gov, full);
    }

    #[test]
    fn alias_proposals_for_governance() {
        assert_eq!(explain("proposals"), explain("governance"));
    }

    #[test]
    fn alias_triumvirate_for_senate() {
        assert_eq!(explain("triumvirate"), explain("senate"));
    }

    #[test]
    fn alias_ckswap_for_coldkeyswap() {
        let ck = explain("ckswap");
        let full = explain("coldkey-swap");
        assert!(ck.is_some());
        assert_eq!(ck, full);
    }

    #[test]
    fn alias_mev_for_mevshield() {
        let mev = explain("mev");
        let full = explain("mev-shield");
        assert!(mev.is_some());
        assert_eq!(mev, full);
    }

    #[test]
    fn alias_nominate_for_delegation() {
        assert_eq!(explain("nominate"), explain("delegation"));
    }

    #[test]
    fn alias_1000_for_stake_weight() {
        assert_eq!(explain("1000"), explain("stake-weight"));
    }

    #[test]
    fn alias_params_for_hyperparams() {
        assert_eq!(explain("params"), explain("hyperparams"));
    }

    #[test]
    fn alias_serving_for_axon() {
        assert_eq!(explain("serving"), explain("axon"));
    }

    #[test]
    fn alias_burn_for_recycle() {
        assert_eq!(explain("burn"), explain("recycle"));
    }

    #[test]
    fn alias_wayback_for_archive() {
        assert_eq!(explain("wayback"), explain("archive"));
    }

    #[test]
    fn alias_subnet_owner_for_owner_workflow() {
        assert_eq!(explain("subnet-owner"), explain("ow"));
    }

    // --- Case insensitivity ---

    #[test]
    fn case_insensitive_tempo() {
        assert_eq!(explain("TEMPO"), explain("tempo"));
    }

    #[test]
    fn case_insensitive_yuma() {
        assert_eq!(explain("YUMA"), explain("yuma"));
    }

    #[test]
    fn case_insensitive_amm() {
        assert_eq!(explain("AMM"), explain("amm"));
    }

    #[test]
    fn case_insensitive_mixed() {
        assert_eq!(explain("Emission"), explain("emission"));
    }

    #[test]
    fn case_insensitive_subnet() {
        assert_eq!(explain("SUBNET"), explain("subnet"));
    }

    #[test]
    fn case_insensitive_dtao() {
        assert_eq!(explain("DTAO"), explain("dtao"));
    }

    #[test]
    fn case_insensitive_cr() {
        assert_eq!(explain("CR"), explain("cr"));
    }

    #[test]
    fn case_insensitive_mev_shield() {
        assert_eq!(explain("MEV-SHIELD"), explain("mev-shield"));
    }

    // --- Hyphen/underscore stripping ---

    #[test]
    fn strip_hyphens_commit_reveal() {
        assert_eq!(explain("commit-reveal"), explain("commitreveal"));
    }

    #[test]
    fn strip_underscores_commit_reveal() {
        assert_eq!(explain("commit_reveal"), explain("commitreveal"));
    }

    #[test]
    fn strip_hyphens_rate_limit() {
        assert_eq!(explain("rate-limit"), explain("ratelimit"));
    }

    #[test]
    fn strip_underscores_rate_limit() {
        assert_eq!(explain("rate_limit"), explain("ratelimit"));
    }

    #[test]
    fn strip_hyphens_stake_weight() {
        assert_eq!(explain("stake-weight"), explain("stakeweight"));
    }

    #[test]
    fn strip_hyphens_dynamic_tao() {
        assert_eq!(explain("dynamic-tao"), explain("dynamictao"));
    }

    #[test]
    fn strip_underscores_dynamic_tao() {
        assert_eq!(explain("dynamic_tao"), explain("dynamictao"));
    }

    #[test]
    fn strip_hyphens_alpha_token() {
        assert_eq!(explain("alpha-token"), explain("alphatoken"));
    }

    #[test]
    fn strip_hyphens_cold_key_swap() {
        assert_eq!(explain("coldkey-swap"), explain("coldkeyswap"));
    }

    #[test]
    fn strip_hyphens_mev_protection() {
        assert_eq!(explain("mev-protection"), explain("mevprotection"));
    }

    #[test]
    fn strip_hyphens_network_limits() {
        assert_eq!(explain("network-limits"), explain("networklimits"));
    }

    #[test]
    fn strip_hyphens_immunity_period() {
        assert_eq!(explain("immunity-period"), explain("immunityperiod"));
    }

    #[test]
    fn strip_hyphens_root_network() {
        assert_eq!(explain("root-network"), explain("rootnetwork"));
    }

    #[test]
    fn strip_hyphens_pow_registration() {
        assert_eq!(explain("pow-registration"), explain("powregistration"));
    }

    #[test]
    fn strip_hyphens_archive_node() {
        assert_eq!(explain("archive-node"), explain("archivenode"));
    }

    #[test]
    fn strip_mixed_hyphens_and_case() {
        assert_eq!(explain("Commit-Reveal"), explain("commitreveal"));
    }

    // --- Unknown topics return None ---

    #[test]
    fn unknown_topic_returns_none() {
        assert!(explain("xyzzy_nonexistent_topic_999").is_none());
    }

    #[test]
    fn unknown_topic_empty_string_returns_none() {
        assert!(explain("").is_none());
    }

    #[test]
    fn unknown_topic_random_returns_none() {
        assert!(explain("notarealconcept").is_none());
    }

    #[test]
    fn unknown_topic_number_returns_none() {
        // "1000" is a known alias for stake-weight, but other numbers should not match
        assert!(explain("9999").is_none());
    }

    // --- list_topics is non-empty ---

    #[test]
    fn list_topics_not_empty() {
        let topics = list_topics();
        assert!(!topics.is_empty());
    }

    #[test]
    fn list_topics_contains_tempo() {
        let topics = list_topics();
        assert!(topics.iter().any(|(k, _)| *k == "tempo"));
    }

    #[test]
    fn list_topics_contains_amm() {
        let topics = list_topics();
        assert!(topics.iter().any(|(k, _)| *k == "amm"));
    }

    // --- Content sanity: known topics contain expected keywords ---

    #[test]
    fn tempo_content_mentions_blocks() {
        let text = explain("tempo").unwrap();
        let lower = text.to_lowercase();
        assert!(lower.contains("block") || lower.contains("tempo"));
    }

    #[test]
    fn amm_content_mentions_pool_or_market() {
        let text = explain("amm").unwrap();
        let lower = text.to_lowercase();
        assert!(lower.contains("pool") || lower.contains("market") || lower.contains("amm"));
    }

    #[test]
    fn emission_content_mentions_emission() {
        let text = explain("emission").unwrap();
        let lower = text.to_lowercase();
        assert!(lower.contains("emission") || lower.contains("tao"));
    }
}
