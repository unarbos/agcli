# evm — Ethereum Virtual Machine Operations

Interact with the EVM layer on Bittensor. Supports Ethereum-compatible smart contract calls and balance withdrawals.

## Subcommands

### evm call
Execute an EVM call (message call to a contract or EOA).

```bash
agcli evm call --source 0xSourceAddr --target 0xTargetAddr \
  --input 0xCalldata --gas-limit 100000 \
  [--value 0x...] [--max-fee-per-gas 0x...]
```

- `--source`: Sender EVM address (20 bytes hex)
- `--target`: Contract/destination EVM address (20 bytes hex)
- `--input`: ABI-encoded calldata (hex)
- `--value`: Wei to send (U256, 32 bytes hex)
- `--gas-limit`: Gas limit (default 21000)
- `--max-fee-per-gas`: Max fee per gas (U256, 32 bytes hex)

### evm withdraw
Withdraw balance from an EVM address back to the Substrate side.

```bash
agcli evm withdraw --address 0xEvmAddr --amount 1000000000
```

- `--address`: EVM address to withdraw from (20 bytes hex)
- `--amount`: Amount in RAO to withdraw

## On-chain Pallets
- `EVM::call` — Execute EVM message call
- `EVM::withdraw` — Bridge funds from EVM to Substrate
- `Ethereum::transact` — Raw Ethereum transaction (not wrapped yet)

## Related Commands
- `agcli contracts call` — WASM contract interaction (alternative to EVM)
