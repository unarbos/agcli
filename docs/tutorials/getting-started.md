# Getting Started with agcli

## 1. Install

```bash
# From source
git clone https://github.com/unconst/agcli && cd agcli
cargo install --path .

# Or directly
cargo install --git https://github.com/unconst/agcli
```

## 2. Create a Wallet

```bash
agcli wallet create --name my_wallet
# You'll be prompted for a password to encrypt the coldkey.
# A 12-word mnemonic is generated — SAVE IT SECURELY.
```

## 3. Check Your Balance

```bash
agcli balance
# Or for any address:
agcli balance --address 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY
```

## 4. View Subnets

```bash
# List all subnets
agcli subnet list

# View a specific subnet's metagraph
agcli subnet metagraph 1
```

## 5. Stake TAO

```bash
# Stake 10 TAO on subnet 1
agcli stake add 10.0 --netuid 1

# View your stakes
agcli stake list
```

## 6. Transfer TAO

```bash
agcli transfer 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY 1.5
```

## 7. Interactive Staking Wizard

```bash
# Guided staking flow — shows subnets, lets you pick, confirms before submitting
agcli stake wizard
```

## Configuration

### Persistent config file (recommended)
```bash
# Set defaults once — persisted to ~/.agcli/config.toml
agcli config set network finney
agcli config set wallet my_wallet
agcli config set output json

# View current config
agcli config show

# Remove a setting
agcli config unset output
```

### Environment variables
```bash
export AGCLI_NETWORK=finney
export AGCLI_WALLET=my_wallet
export AGCLI_WALLET_DIR=~/.bittensor/wallets
```

### CLI flags (override everything)
```bash
agcli --network test --wallet my_wallet subnet list
```

Priority: CLI flags > environment variables > config file > defaults

## Shell Completions

```bash
# Bash
agcli completions bash > /etc/bash_completion.d/agcli

# Zsh
agcli completions zsh > ~/.zfunc/_agcli

# Fish
agcli completions fish > ~/.config/fish/completions/agcli.fish
```

## Advanced: Validators & History

```bash
# View top validators across the network
agcli view validators --limit 20

# Validators on a specific subnet
agcli view validators --netuid 1

# Transaction history
agcli view history --limit 10
```
