# contracts — WASM Smart Contract Operations

Deploy and interact with WASM smart contracts on the Bittensor chain via pallet-contracts.

## Subcommands

### contracts upload
Upload WASM contract code to the chain.

```bash
agcli contracts upload --code /path/to/contract.wasm [--storage-deposit-limit 1000000]
```

Returns the tx hash. The code hash can be found in chain events.

### contracts instantiate
Create a contract instance from an already-uploaded code hash.

```bash
agcli contracts instantiate --code-hash 0x... \
  [--value 0] [--data 0x...] [--salt 0x...] \
  [--gas-ref-time 10000000000] [--gas-proof-size 1048576] \
  [--storage-deposit-limit 1000000]
```

- `--code-hash`: Hash from a previous `upload`
- `--data`: Constructor selector + args (hex-encoded)
- `--salt`: Unique salt for address derivation
- `--value`: TAO (in RAO) to transfer to the new contract

### contracts call
Call an existing contract.

```bash
agcli contracts call --contract 5Contract... --data 0xSelectorArgs... \
  [--value 0] [--gas-ref-time 10000000000] [--gas-proof-size 1048576]
```

- `--contract`: SS58 address of the deployed contract
- `--data`: Method selector + encoded arguments (hex)

### contracts remove-code
Remove previously uploaded contract code.

```bash
agcli contracts remove-code --code-hash 0x...
```

## On-chain Pallet
- `Contracts::upload_code` — Upload WASM bytecode
- `Contracts::instantiate` — Create contract instance
- `Contracts::call` — Invoke contract method
- `Contracts::remove_code` — Remove uploaded code

## Related Commands
- `agcli evm call` — EVM (Solidity) contract interaction
