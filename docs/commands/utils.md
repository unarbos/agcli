# utils — Utility Commands

Miscellaneous tools: unit conversion, latency benchmarking, shell completions, self-update, diagnostics.

## Subcommands

### utils convert
Convert between TAO and RAO.

```bash
agcli utils convert --tao 1.5      # → 1500000000 RAO
agcli utils convert --rao 1000000000  # → 1.0 TAO
```

### utils latency
Benchmark RPC endpoint latency.

```bash
agcli utils latency [--count 10]
```

Measures round-trip time for chain queries.

### completions
Generate shell completions.

```bash
agcli completions --shell bash > ~/.bash_completion.d/agcli
agcli completions --shell zsh > ~/.zfunc/_agcli
agcli completions --shell fish > ~/.config/fish/completions/agcli.fish
agcli completions --shell powershell > _agcli.ps1
```

### update
Self-update agcli from GitHub.

```bash
agcli update
```

## Diagnostics (top-level)

**`agcli doctor`** is not a `utils` subcommand — it is a **top-level** command. See **[doctor.md](doctor.md)** for connectivity, chain pings, disk cache, wallet row semantics, JSON shape, and exit behaviour (always **0** with per-row OK/FAIL).

## Source Code
**agcli handler**: [`src/cli/system_cmds.rs`](https://github.com/unconst/agcli/blob/main/src/cli/system_cmds.rs) — `handle_utils()` (convert, latency), `generate_completions()`, `handle_update()`; **`handle_doctor()`** is separate (~`handle_doctor` in the same file).

**No on-chain interaction** for convert/completions/update. **`utils latency`** makes RPC test calls.

## Related Commands
- [`agcli doctor`](doctor.md) — Full connectivity / wallet smoke panel
- `agcli explain` — Built-in concept reference
- `agcli config show` — Current configuration
