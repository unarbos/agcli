# safe-mode — Safe Mode Operations

Enter, extend, or exit the chain's safe mode. Safe mode restricts which extrinsics can be executed, useful for emergency situations.

## Subcommands

### safe-mode enter
Enter safe mode permissionlessly. Requires a deposit.

```bash
agcli safe-mode enter
```

### safe-mode extend
Extend the current safe mode duration. Requires a deposit.

```bash
agcli safe-mode extend
```

### safe-mode force-enter
Force enter safe mode for a specified duration (requires sudo).

```bash
agcli safe-mode force-enter --duration 1000
```

### safe-mode force-exit
Force exit safe mode immediately (requires sudo).

```bash
agcli safe-mode force-exit
```

## On-chain Pallet
- `SafeMode::enter` — Permissionless entry (with deposit)
- `SafeMode::extend` — Permissionless extension (with deposit)
- `SafeMode::force_enter` — Privileged entry (sudo)
- `SafeMode::force_exit` — Privileged exit (sudo)

## Related Commands
- `agcli admin` — Other sudo-level operations
