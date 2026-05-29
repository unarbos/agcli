# admin commands

`agcli admin` wraps privileged `AdminUtils` dispatchables through `Sudo.sudo`.
This page reflects the current code in:

- `src/cli/mod.rs` (`AdminCommands`)
- `src/cli/admin_cmds.rs` (handlers)
- `src/admin.rs` (subxt dynamic call wrappers)
- `subtensor/pallets/admin-utils/src/lib.rs` at commit `6844ee37...`

## Shared behavior

### Sudo key resolution

- `--sudo-key <String>` accepts a dev URI such as `//Alice`.
- If omitted, agcli falls back to the wallet coldkey.

### JSON output schema

For all write commands (`admin list` excluded), `--output json` prints:

```json
{
  "tx_hash": "0x..."
}
```

For `admin list`, `--output json` prints:

```json
[
  {
    "call": "sudo_set_tempo",
    "description": "Blocks per epoch",
    "args": ["netuid: u16", "tempo: u16"]
  }
]
```

### Exit codes (from `src/error.rs`)

- `0` success
- `10` network / websocket failure
- `11` auth / wallet unlock failure
- `12` validation (bad CLI value, bad JSON args, bad call name)
- `13` chain dispatch failure (`BadOrigin`, `SubnetDoesNotExist`, pallet errors)
- `14` local I/O failure (wallet/key file access)
- `15` timeout
- `1` uncategorized failure

### Value encoding (hyperparams)

Several admin setters mirror `subnet set-param` on-chain types. Prefer `agcli subnet set-param` for subnet owners; use `agcli admin` when you have sudo.

| Param | CLI flag | On-chain | How to pass |
|---|---|---|---|
| kappa | `--kappa` | u16, runtime ÷65535 → [0,1] | Raw u16 (e.g. `32767`) or use `subnet set-param --value 0.5` |
| rho | `--rho` | u16 sigmoid scale | Integer only (default ~10; **not** a 0–1 fraction) |
| min_burn / max_burn | `--burn` | u64 RAO | Raw RAO integer, or use `subnet set-param --value 1.0` (TAO) |
| alpha_low / alpha_high | `--alpha-low`, `--alpha-high` | u16 | Raw u16; see liquid-alpha pallet docs for semantics |

## Command reference

Notes:
- "SCALE sent by agcli" describes the exact dynamic values passed in `src/admin.rs`.
- All write calls are wrapped by `Sudo.sudo` and surface `Sudo::Sudid`.
- Admin-utils event emission is sparse. Most setters emit no `AdminUtils::*` event.

| Subcommand | Clap flags and types | Pallet ref and dispatchable | SCALE sent by agcli | Primary storage key(s) touched | Events on success |
|---|---|---|---|---|---|
| `set-tempo` | `--netuid <u16>` `--tempo <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_tempo(netuid: NetUid, tempo: u16)` | `(u128(netuid), u128(tempo))` | `SubtensorModule::Tempo[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::TempoSet` |
| `set-max-validators` | `--netuid <u16>` `--max <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_max_allowed_validators(netuid, max_allowed_validators)` | `(u128(netuid), u128(max))` | `SubtensorModule::MaxAllowedValidators[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::MaxAllowedValidatorsSet` |
| `set-max-uids` | `--netuid <u16>` `--max <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_max_allowed_uids(netuid, max_allowed_uids)` | `(u128(netuid), u128(max))` | `SubtensorModule::MaxAllowedUids[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::MaxAllowedUidsSet` |
| `set-immunity-period` | `--netuid <u16>` `--period <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_immunity_period(netuid, immunity_period)` | `(u128(netuid), u128(period))` | `SubtensorModule::ImmunityPeriod[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::ImmunityPeriodSet` |
| `set-min-weights` | `--netuid <u16>` `--min <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_min_allowed_weights(netuid, min_allowed_weights)` | `(u128(netuid), u128(min))` | `SubtensorModule::MinAllowedWeights[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::MinAllowedWeightSet` |
| `set-max-weight-limit` | `--netuid <u16>` `--limit <u16>` `--sudo-key <String?>` | agcli targets `AdminUtils::sudo_set_max_weight_limit`, but this dispatchable is not present in current `admin-utils` pallet | `(u128(netuid), u128(limit))` | No on-chain write in current runtime path. Related key exists: `SubtensorModule::MaxWeightsLimit[netuid]` | Fails before submit when metadata lacks call (`13`), no chain event |
| `set-weights-rate-limit` | `--netuid <u16>` `--limit <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_weights_set_rate_limit(netuid, weights_set_rate_limit)` | `(u128(netuid), u128(limit))` | `SubtensorModule::WeightsSetRateLimit[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::WeightsSetRateLimitSet` |
| `set-commit-reveal` | `--netuid <u16>` `--enabled <bool flag>` `--sudo-key <String?>` | `AdminUtils::sudo_set_commit_reveal_weights_enabled(netuid, enabled)` | `(u128(netuid), bool(enabled))` | `SubtensorModule::CommitRevealWeightsEnabled[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::CommitRevealEnabled` |
| `set-difficulty` | `--netuid <u16>` `--difficulty <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_difficulty(netuid, difficulty)` | `(u128(netuid), u128(difficulty))` | `SubtensorModule::Difficulty[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::DifficultySet` |
| `set-activity-cutoff` | `--netuid <u16>` `--cutoff <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_activity_cutoff(netuid, activity_cutoff)` | `(u128(netuid), u128(cutoff))` | `SubtensorModule::ActivityCutoff[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::ActivityCutoffSet` |
| `set-default-take` | `--take <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_default_take(default_take)` | `(u128(take))` | `SubtensorModule::MaxDelegateTake` | `Sudo::Sudid(Ok)`, `SubtensorModule::MaxDelegateTakeSet` |
| `set-tx-rate-limit` | `--limit <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_tx_rate_limit(tx_rate_limit)` | `(u128(limit))` | `SubtensorModule::TxRateLimit` | `Sudo::Sudid(Ok)`, `SubtensorModule::TxRateLimitSet` |
| `set-min-difficulty` | `--netuid <u16>` `--difficulty <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_min_difficulty(netuid, min_difficulty)` | `(u128(netuid), u128(difficulty))` | `SubtensorModule::MinDifficulty[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::MinDifficultySet` |
| `set-max-difficulty` | `--netuid <u16>` `--difficulty <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_max_difficulty(netuid, max_difficulty)` | `(u128(netuid), u128(difficulty))` | `SubtensorModule::MaxDifficulty[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::MaxDifficultySet` |
| `set-adjustment-interval` | `--netuid <u16>` `--interval <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_adjustment_interval(netuid, adjustment_interval)` | `(u128(netuid), u128(interval))` | `SubtensorModule::AdjustmentInterval[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::AdjustmentIntervalSet` |
| `set-kappa` | `--netuid <u16>` `--kappa <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_kappa(netuid, kappa)` | `(u128(netuid), u128(kappa))` | `SubtensorModule::Kappa[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::KappaSet` |
| `set-rho` | `--netuid <u16>` `--rho <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_rho(netuid, rho)` | `(u128(netuid), u128(rho))` | `SubtensorModule::Rho[netuid]` | `Sudo::Sudid(Ok)` (no dedicated `SubtensorModule` event in setter path) |
| `set-min-burn` | `--netuid <u16>` `--burn <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_min_burn(netuid, min_burn)` | `(u128(netuid), u128(burn))` | `SubtensorModule::MinBurn[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::MinBurnSet` |
| `set-max-burn` | `--netuid <u16>` `--burn <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_max_burn(netuid, max_burn)` | `(u128(netuid), u128(burn))` | `SubtensorModule::MaxBurn[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::MaxBurnSet` |
| `set-liquid-alpha` | `--netuid <u16>` `--enabled <bool flag>` `--sudo-key <String?>` | `AdminUtils::sudo_set_liquid_alpha_enabled(netuid, enabled)` | `(u128(netuid), bool(enabled))` | `SubtensorModule::LiquidAlphaOn[netuid]` | `Sudo::Sudid(Ok)` (no `AdminUtils` event for this call) |
| `set-alpha-values` | `--netuid <u16>` `--alpha-low <u16>` `--alpha-high <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_alpha_values(netuid, alpha_low, alpha_high)` | `(u128(netuid), u128(alpha_low), u128(alpha_high))` | `SubtensorModule::AlphaValues[netuid]` | `Sudo::Sudid(Ok)` (setter path logs, no dedicated event) |
| `set-yuma3` | `--netuid <u16>` `--enabled <bool flag>` `--sudo-key <String?>` | `AdminUtils::sudo_set_yuma3_enabled(netuid, enabled)` | `(u128(netuid), bool(enabled))` | `SubtensorModule::Yuma3On[netuid]` | `Sudo::Sudid(Ok)`, `AdminUtils::Yuma3EnableToggled` |
| `set-bonds-penalty` | `--netuid <u16>` `--penalty <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_bonds_penalty(netuid, bonds_penalty)` | `(u128(netuid), u128(penalty))` | `SubtensorModule::BondsPenalty[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::BondsPenaltySet` |
| `set-stake-threshold` | `--threshold <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_stake_threshold(min_stake)` | `(u128(threshold))` | `SubtensorModule::StakeThreshold` | `Sudo::Sudid(Ok)`, `SubtensorModule::StakeThresholdSet` |
| `set-network-registration` | `--netuid <u16>` `--allowed <bool flag>` `--sudo-key <String?>` | `AdminUtils::sudo_set_network_registration_allowed(netuid, registration_allowed)` | `(u128(netuid), bool(allowed))` | `SubtensorModule::NetworkRegistrationAllowed[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::RegistrationAllowed` |
| `set-pow-registration` | `--netuid <u16>` `--allowed <bool flag>` `--sudo-key <String?>` | `AdminUtils::sudo_set_network_pow_registration_allowed(netuid, registration_allowed)` | `(u128(netuid), bool(allowed))` | None in current runtime path because dispatchable returns `AdminUtils::POWRegistrationDisabled` | `Sudo::Sudid(Err(...POWRegistrationDisabled...))` |
| `set-adjustment-alpha` | `--netuid <u16>` `--alpha <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_adjustment_alpha(netuid, adjustment_alpha)` | `(u128(netuid), u128(alpha))` | `SubtensorModule::AdjustmentAlpha[netuid]` | `Sudo::Sudid(Ok)`, `SubtensorModule::AdjustmentAlphaSet` |
| `set-subnet-moving-alpha` | `--alpha <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_subnet_moving_alpha(alpha: I96F32)` | agcli sends `(u128(alpha))`, while pallet expects fixed-point `I96F32` | `SubtensorModule::SubnetMovingAlpha` | Can fail with type/dispatch error if encoding mismatches runtime expectation |
| `set-mechanism-count` | `--netuid <u16>` `--count <u16>` `--sudo-key <String?>` | `AdminUtils::sudo_set_mechanism_count(netuid, mechanism_count)` | `(u128(netuid), u128(count))` | `SubtensorModule::MechanismCountCurrent[netuid]` and possible reset of `MechanismEmissionSplit[netuid]` when count changes | `Sudo::Sudid(Ok)` (no dedicated admin-utils event) |
| `set-mechanism-emission-split` | `--netuid <u16>` `--weights <String>` `--sudo-key <String?>` | `AdminUtils::sudo_set_mechanism_emission_split(netuid, maybe_split: Option<Vec<u16>>)` | agcli parses CSV into `Vec<u64>` and sends unnamed composite vector, not explicit `Option<Vec<u16>>` | Intended target is `SubtensorModule::MechanismEmissionSplit[netuid]` | Can fail at dispatch/encoding if runtime rejects arg shape |
| `set-nominator-min-stake` | `--stake <u64>` `--sudo-key <String?>` | `AdminUtils::sudo_set_nominator_min_required_stake(min_stake)` | `(u128(stake))` | `SubtensorModule::NominatorMinRequiredStake` | `Sudo::Sudid(Ok)` |
| `raw` | `--call <String>` `--args <JSON array>` `--sudo-key <String?>` | `AdminUtils::<dynamic call name>` plus special alias `senate-vote` → `SubtensorModule::vote` | JSON numbers become `u128`, bools become bool, strings become string | Depends on call | `Sudo::Sudid(...)` for AdminUtils calls; direct `SubtensorModule::Voted` path for `senate-vote` |
| `list` | no args | local only, does not submit to chain | n/a | n/a | n/a |

## `admin raw` accepted call names

`admin raw` is restricted by `validate_admin_call_name` to agcli's local `known_params` list, not the full runtime call set. Current allowed names are:

`sudo_set_tempo`, `sudo_set_max_allowed_validators`, `sudo_set_max_allowed_uids`, `sudo_set_immunity_period`, `sudo_set_min_allowed_weights`, `sudo_set_max_weight_limit`, `sudo_set_weights_set_rate_limit`, `sudo_set_commit_reveal_weights_enabled`, `sudo_set_difficulty`, `sudo_set_bonds_moving_average`, `sudo_set_target_registrations_per_interval`, `sudo_set_activity_cutoff`, `sudo_set_serving_rate_limit`, `sudo_set_default_take`, `sudo_set_tx_rate_limit`, `sudo_set_min_difficulty`, `sudo_set_max_difficulty`, `sudo_set_adjustment_interval`, `sudo_set_adjustment_alpha`, `sudo_set_kappa`, `sudo_set_rho`, `sudo_set_min_burn`, `sudo_set_max_burn`, `sudo_set_liquid_alpha_enabled`, `sudo_set_alpha_values`, `sudo_set_yuma3_enabled`, `sudo_set_bonds_penalty`, `sudo_set_subnet_moving_alpha`, `sudo_set_mechanism_count`, `sudo_set_mechanism_emission_split`, `sudo_set_stake_threshold`, `sudo_set_nominator_min_required_stake`, `sudo_set_network_registration_allowed`, `sudo_set_network_pow_registration_allowed`.

`admin raw --call senate-vote` is also supported as a governance parity alias. It expects `--args '[\"0x<proposal_hash>\", <vote>]'`, where `<vote>` can be `true/false`, `yes/no`, `aye/nay`, or `1/0`.

## Practical examples

```bash
agcli --network local admin set-tempo --netuid 1 --tempo 120 --sudo-key //Alice
agcli --network local --output json admin set-default-take --take 32767 --sudo-key //Alice
agcli --network local admin raw --call sudo_set_target_registrations_per_interval --args '[1, 3]' --sudo-key //Alice
agcli --network local --yes --output json admin raw --call senate-vote --args '["0x<proposal_hash>", "yes"]' --sudo-key //Alice
agcli --output json admin list
```
