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

## Configuration

Set defaults via environment variables:
```bash
export AGCLI_NETWORK=finney
export AGCLI_WALLET=my_wallet
export AGCLI_WALLET_DIR=~/.bittensor/wallets
```

Or pass them as CLI flags:
```bash
agcli --network test --wallet my_wallet subnet list
```
