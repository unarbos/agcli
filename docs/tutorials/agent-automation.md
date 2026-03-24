# Agent Automation with agcli

How to use agcli from AI agents, scripts, and automation pipelines with zero interactive prompts.

## Setup

```bash
# Install
cargo install --git https://github.com/unarbos/agcli

# Set persistent defaults (optional)
agcli config set --key batch --value true          # Never prompt for missing args
agcli config set --key output --value json         # Always output JSON
agcli config set --key network --value finney      # Default to mainnet
```

## Core Principles

1. **`--batch` mode** — all missing required args become hard errors with hints, never stdin prompts
2. **`--output json`** — structured JSON on stdout; errors as `{"error": true, "message": "..."}` on stderr
3. **`--yes`** — skip confirmation prompts (combine with `--batch` for fully non-interactive)
4. **`--password`** or `AGCLI_PASSWORD` — provide wallet password without prompt
5. **Exit codes** — 0=success, 1=error. Parse stderr JSON for error details.

## Wallet Management

```bash
# Create wallet
agcli wallet create --name agent_wallet --password "$WALLET_PASS" --yes

# Import from mnemonic
agcli wallet import --name agent_wallet \
  --mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" \
  --password "$WALLET_PASS"

# List wallets (JSON)
agcli --output json wallet list

# Derive address from public key (no wallet needed)
agcli --output json wallet derive 0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d

# Sign a message
agcli --output json wallet sign --message "my message" --password "$WALLET_PASS" -w agent_wallet

# Verify a signature
agcli --output json wallet verify --message "my message" --signature 0xABCDEF... --signer 5Gx...
# Exit code 0 = valid, 1 = invalid
```

## Querying Chain State

```bash
# Balance
agcli --output json balance --address 5Gx...

# All subnets
agcli --output json subnet list

# Single subnet details
agcli --output json subnet show --netuid 1

# Metagraph (all neurons)
agcli --output json subnet metagraph --netuid 1

# Single neuron
agcli --output json subnet metagraph --netuid 1 --uid 42

# Portfolio
agcli --output json view portfolio --address 5Gx...

# Staking analytics with APY
agcli --output json view staking-analytics --address 5Gx...

# Dynamic TAO prices
agcli --output json view dynamic

# Network overview
agcli --output json view network

# Swap simulation
agcli --output json view swap-sim --netuid 1 --tao 10.0
```

## Staking with Safety

```bash
# Set spending limits FIRST
agcli config set --key spending_limit.97 --value 100.0    # Max 100 TAO on SN97
agcli config set --key spending_limit.* --value 500.0     # Global max

# Stake with slippage protection
agcli stake add --amount 10.0 --netuid 1 --password "$WALLET_PASS" --yes --max-slippage 2.0

# Unstake
agcli stake remove --amount 5.0 --netuid 1 --password "$WALLET_PASS" --yes --max-slippage 2.0

# Check liquidity before staking
agcli --output json subnet liquidity --netuid 1
```

## Weight Setting

```bash
# Dry-run first (check pre-conditions without submitting)
agcli --output json weights set --netuid 97 --weights "0:100,1:200,2:50" --dry-run --password "$WALLET_PASS"
# Returns: stake_sufficient, rate_limit_ok, commit_reveal_required, blocks_until_eligible

# Set weights
agcli weights set --netuid 97 --weights "0:100,1:200,2:50" --password "$WALLET_PASS" --yes

# Atomic commit-reveal (one command, no babysitting)
agcli weights commit-reveal --netuid 97 --weights "0:100,1:200,2:50" --wait --password "$WALLET_PASS" --yes
# Commits, waits for reveal window, auto-reveals, returns result
```

## Monitoring

```bash
# Watch balance (alerts to stdout as JSON when below threshold)
agcli --output json balance --watch 60 --threshold 10.0 --address 5Gx... &

# Monitor subnet for anomalies (JSON streaming)
agcli subnet monitor --netuid 97 --json &

# Subscribe to staking events on your subnet
agcli --output json subscribe events staking --netuid 97 --account 5Gx... &

# Subnet health check
agcli --output json subnet health --netuid 97

# Registration cost
agcli --output json subnet cost --netuid 97
```

## Error Handling Pattern

```python
import subprocess, json

def agcli(args):
    """Run agcli command, return parsed JSON or raise on error."""
    result = subprocess.run(
        ["agcli", "--output", "json", "--batch"] + args,
        capture_output=True, text=True
    )
    if result.returncode != 0:
        err = json.loads(result.stderr) if result.stderr.strip() else {"message": "unknown error"}
        raise RuntimeError(err.get("message", result.stderr))
    return json.loads(result.stdout) if result.stdout.strip() else None

# Usage
balance = agcli(["balance", "--address", "5Gx..."])
subnets = agcli(["subnet", "list"])
```

```bash
# Shell pattern
if ! output=$(agcli --output json --batch balance --address 5Gx... 2>/tmp/agcli_err); then
    error=$(cat /tmp/agcli_err)
    echo "Error: $error" >&2
    exit 1
fi
echo "$output" | jq '.balance'
```

## Environment Variables

All flags have env var equivalents for containerized/CI use:

```bash
export AGCLI_NETWORK=finney
export AGCLI_PASSWORD=mypass
export AGCLI_WALLET=agent_wallet
export AGCLI_HOTKEY=default
export AGCLI_YES=1
export AGCLI_BATCH=1

# Now all commands run fully non-interactive
agcli --output json balance
agcli stake add --amount 10.0 --netuid 1
```

## Built-in Reference

```bash
# Get help on any Bittensor concept
agcli explain tempo
agcli explain commit-reveal
agcli explain amm
agcli explain rate-limits
agcli explain bootstrap

# List all topics
agcli explain
```
