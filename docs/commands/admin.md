# admin — Sudo AdminUtils

Set subnet hyperparameters via the chain's `AdminUtils` pallet. Requires the sudo key (Alice on localnet, root key on mainnet).

These commands close the gap where agents can register subnets but can't configure them without writing Rust — every AdminUtils call is now a one-liner.

## Typed Commands

### admin set-tempo
Set the tempo (blocks per epoch) for a subnet.

```bash
agcli admin set-tempo --netuid 1 --tempo 100 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_tempo(origin, netuid, tempo)`

### admin set-max-validators
Set max validator slots.

```bash
agcli admin set-max-validators --netuid 1 --max 8 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_max_allowed_validators(origin, netuid, max)`

### admin set-max-uids
Set max total UID slots.

```bash
agcli admin set-max-uids --netuid 1 --max 256 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_max_allowed_uids(origin, netuid, max)`

### admin set-immunity-period
Set immunity period (blocks of immunity after registration).

```bash
agcli admin set-immunity-period --netuid 1 --period 100 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_immunity_period(origin, netuid, period)`

### admin set-min-weights
Set minimum weights a validator must set.

```bash
agcli admin set-min-weights --netuid 1 --min 1 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_min_allowed_weights(origin, netuid, min)`

### admin set-max-weight-limit
Set maximum weight value.

```bash
agcli admin set-max-weight-limit --netuid 1 --limit 65535 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_max_weight_limit(origin, netuid, limit)`

### admin set-weights-rate-limit
Set blocks between weight submissions (0 = unlimited).

```bash
agcli admin set-weights-rate-limit --netuid 1 --limit 0 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_weights_set_rate_limit(origin, netuid, limit)`

### admin set-commit-reveal
Enable or disable commit-reveal weights.

```bash
agcli admin set-commit-reveal --netuid 1 --enabled false --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_commit_reveal_weights_enabled(origin, netuid, enabled)`

### admin set-difficulty
Set POW registration difficulty.

```bash
agcli admin set-difficulty --netuid 1 --difficulty 1000000 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_difficulty(origin, netuid, difficulty)`

### admin set-activity-cutoff
Set activity cutoff (blocks before a neuron is considered inactive).

```bash
agcli admin set-activity-cutoff --netuid 1 --cutoff 5000 --sudo-key //Alice --network local
```

**On-chain**: `AdminUtils::sudo_set_activity_cutoff(origin, netuid, cutoff)`

## Generic Commands

### admin raw
Execute any AdminUtils call by name — escape hatch for parameters without a typed command.

```bash
# Set bonds moving average
agcli admin raw --call sudo_set_bonds_moving_average --args '[1, 900000]' --sudo-key //Alice --network local

# Set target registrations per interval
agcli admin raw --call sudo_set_target_registrations_per_interval --args '[1, 3]' --sudo-key //Alice --network local

# Set serving rate limit
agcli admin raw --call sudo_set_serving_rate_limit --args '[1, 50]' --sudo-key //Alice --network local
```

Args must be a JSON array. Supported value types: numbers (u128), booleans, strings.

### admin list
Show all known AdminUtils parameters with descriptions and argument types.

```bash
agcli admin list
# JSON: [{"call", "description", "args"}]
```

**Known parameters:**
| Call | Description | Args |
|------|-------------|------|
| `sudo_set_tempo` | Blocks per epoch | `netuid: u16, tempo: u16` |
| `sudo_set_max_allowed_validators` | Max validator slots | `netuid: u16, max: u16` |
| `sudo_set_max_allowed_uids` | Max total UID slots | `netuid: u16, max: u16` |
| `sudo_set_immunity_period` | Blocks of immunity after registration | `netuid: u16, period: u16` |
| `sudo_set_min_allowed_weights` | Minimum weights a validator must set | `netuid: u16, min: u16` |
| `sudo_set_max_weight_limit` | Maximum weight value | `netuid: u16, limit: u16` |
| `sudo_set_weights_set_rate_limit` | Blocks between weight submissions (0=unlimited) | `netuid: u16, limit: u64` |
| `sudo_set_commit_reveal_weights_enabled` | Enable/disable commit-reveal weights | `netuid: u16, enabled: bool` |
| `sudo_set_difficulty` | POW registration difficulty | `netuid: u16, difficulty: u64` |
| `sudo_set_bonds_moving_average` | Bonds moving average | `netuid: u16, avg: u64` |
| `sudo_set_target_registrations_per_interval` | Target registrations per interval | `netuid: u16, target: u16` |
| `sudo_set_activity_cutoff` | Blocks before neuron is inactive | `netuid: u16, cutoff: u16` |
| `sudo_set_serving_rate_limit` | Axon serving rate limit | `netuid: u16, limit: u64` |

## Sudo Key

On **localnet**, Alice (`//Alice`) is the sudo account. Pass `--sudo-key //Alice`.

If `--sudo-key` is omitted, the command falls back to the wallet coldkey. On mainnet, only the chain's root key can execute AdminUtils calls.

## Common Errors
| Error | Cause | Fix |
|-------|-------|-----|
| `Invalid sudo key URI` | Bad URI format | Use `//Alice` or `//Bob` |
| `BadOrigin` / extrinsic failed | Caller is not the sudo account | Verify `--sudo-key` is the chain's sudo key |
| `SubnetDoesNotExist` | Invalid netuid | Check `agcli subnet list` |
| `Invalid JSON args` | Malformed `--args` in `raw` | Must be JSON array: `'[1, 100]'` |

## Source Code
**agcli handler**: [`src/cli/admin_cmds.rs`](https://github.com/unarbos/agcli/blob/main/src/cli/admin_cmds.rs) — `handle_admin()` L34, `resolve_sudo_key()` L12, `parse_raw_args()` L232

**SDK**: [`src/admin.rs`](https://github.com/unarbos/agcli/blob/main/src/admin.rs) — `set_tempo()` L25, `set_max_allowed_validators()` L42, `set_max_allowed_uids()` L59, `set_immunity_period()` L76, `set_min_allowed_weights()` L93, `set_max_weight_limit()` L110, `set_weights_set_rate_limit()` L127, `set_commit_reveal_weights_enabled()` L144, `set_difficulty()` L161, `set_activity_cutoff()` L212, `set_serving_rate_limit()` L229, `raw_admin_call()` L249, `known_params()` L262

**Subtensor pallet**: [`pallets/admin-utils/src/lib.rs`](https://github.com/opentensor/subtensor/blob/main/pallets/admin-utils/src/lib.rs) — All `sudo_set_*` dispatch entry points

## Related Commands
- `agcli localnet start` — Start a local chain for testing
- `agcli localnet scaffold` — Full test environment with admin calls included
- `agcli subnet set-param` — Set hyperparameters as subnet owner (not sudo)
- `agcli subnet hyperparams` — View current hyperparameters
