# agcli-subtensor-audit — Root Planner Discovery Notes

This file is the bootstrap reference material for descendants. Workers and subplanners read this **first** to ground their scoped tasks. Do not mutate freely; the root planner curates it.

## Top-level layout (verified at audit start)

- `src/main.rs`, `src/lib.rs`: CLI entrypoint and SDK re-exports.
- `src/cli/mod.rs`: clap `Cli` struct + 28 `*Commands` enums (top-level command groups). ~3204 LOC.
- `src/cli/commands.rs`: top-level dispatch (`match cli.command`), forwards to `*_cmds.rs`.
- `src/cli/<group>_cmds.rs`: per-group `pub async fn handle_*` implementations.
- `src/chain/extrinsics.rs`: 129 `pub (async )fn`s — the actual subxt extrinsic submitters.
- `src/chain/queries.rs`: 71 `pub (async )fn`s — read-only chain queries.
- `src/chain/rpc_types.rs`: SCALE-decodable storage types.
- `src/chain/mod.rs`: `Client` glue.
- `src/error.rs`: 1447 LOC — Error classification + exit-code mapping (and PalletError <-> human messages).
- `src/events.rs`: 1710 LOC — real-time block/event subscription + filter taxonomy.
- `src/live.rs`: 665 LOC — live polling with delta tracking.
- `src/scaffold.rs`: 818 LOC — declarative test environment scaffolding (TOML → fully-configured local chain).
- `src/localnet.rs`: 393 LOC — Docker-based local chain management.
- `src/extrinsics/{mev_shield.rs,weights.rs}`: shared payload helpers.
- `src/queries/{cache,disk_cache,metagraph,portfolio,query_cache,subnet}.rs`: caches and projections.
- `src/types/{balance,chain_data,network}.rs`: Balance/SS58/NetUid newtypes + chain-data structs.
- `src/utils/{explain,format,pow}.rs`: explain texts, formatters, PoW helpers.
- `src/wallet/{keyfile,keypair,mod}.rs`: bittensor-compatible wallet store + sr25519 keypair.

## Command groups (1 worker per group)

The CLI has these top-level command groups (`pub enum *Commands` in `src/cli/mod.rs`):

| Group        | enum                | cmds module                       | docs                              | tests bundle                          |
|--------------|---------------------|-----------------------------------|-----------------------------------|---------------------------------------|
| wallet       | WalletCommands      | src/cli/wallet_cmds.rs            | docs/commands/wallet.md           | tests/wallet_test.rs                  |
| stake        | StakeCommands       | src/cli/stake_cmds.rs             | docs/commands/stake.md            | tests/stake_binary_stress.rs etc.     |
| subnet       | SubnetCommands      | src/cli/subnet_cmds.rs            | docs/commands/subnet.md           | tests/cli_test_modules/*              |
| weights      | WeightCommands      | src/cli/weights_cmds.rs           | docs/commands/weights.md          | tests/cli_weights.rs, weights_binary_stress.rs |
| view         | ViewCommands        | src/cli/view_cmds.rs              | docs/commands/view.md             | tests/cli_test_modules/*              |
| admin        | AdminCommands       | src/cli/admin_cmds.rs             | docs/commands/admin.md            | tests/cli_test_modules/*              |
| network      | (Crowdloan/Liquidity/Swap/Subscribe/Multisig/Drand/SafeMode/EVM/Contracts/Scheduler/Preimage/Block/Diff/Utils/Config/Identity/Serve/Proxy/Delegate/Root/Commitment/Batch/Audit) | src/cli/network_cmds.rs | many docs       | mixed                                  |
| system       | (Doctor/Explain/Update/Completions) | src/cli/system_cmds.rs    | docs/commands/{doctor,explain}.md | tests/cli_test_modules/*              |
| block        | BlockCommands       | src/cli/block_cmds.rs             | docs/commands/block.md            | tests/cli_test_modules/*              |
| localnet     | LocalnetCommands    | src/cli/localnet_cmds.rs          | docs/commands/localnet.md         | tests/localnet_e2e_test.rs            |

`network_cmds.rs` is overloaded: it carries Crowdloan, Liquidity, Swap, Subscribe, Multisig, Drand, SafeMode, EVM, Contracts, Scheduler, Preimage, Diff, Utils, Config, Identity, Serve, Proxy, Delegate, Root, Commitment, Batch, Audit. We split that file into multiple worker scopes by `pathsAllowed` + line-range guidance in `scopedGoal` rather than by file boundary.

## Subtensor pallets (mapping)

`subtensor/pallets/` (subtensor submodule, branch pinned at `6844ee37f0b8cb02baf9ff8d3ca4319cfb33f361`):

- `subtensor` — main pallet, ~146 dispatchables in `subtensor/pallets/subtensor/src/macros/dispatches.rs` (set_weights, add_stake, register, swap_hotkey, set_children, register_network, …).
- `admin-utils` — sudo hyperparameter setters (~75 fns, e.g. `sudo_set_tempo`, `sudo_set_min_allowed_weights`).
- `commitments` — `set_commitment`, `set_max_space`, `reveal_timelocked_commitments`, `get_commitments`, `purge_netuid`.
- `crowdloan` — `create`, `contribute`, `withdraw`, `finalize`, `refund`, `dissolve`, `update_*`.
- `proxy` — `proxy`, `add_proxy`, `remove_proxy`, `remove_proxies`, `create_pure`, `kill_pure`, `announce`, …
- `swap` — `set_fee_rate`, `toggle_user_liquidity`, `add_liquidity`, `remove_liquidity`, `modify_position`, `disable_lp`.
- `utility` — `batch`, `batch_all`, `force_batch`, `as_derivative`, `dispatch_as`, `with_weight`, `if_else`.
- `drand` — `write_pulse`, `set_beacon_config`, `set_oldest_stored_round`, `random_at`, `message`.
- `transaction-fee`, `registry`, `shield` — fee/identity/MEV-shield infrastructure.

When auditing a CLI command group, cross-reference its extrinsic to the corresponding pallet dispatchable to validate argument completeness, encoding, error codes, and event names.

## Cross-cutting modules (each gets its own worker)

- `src/error.rs` — pallet-error → human-readable + exit-code map. Many on-chain errors not covered.
- `src/events.rs` — event filter taxonomy and pretty printers; check against `subtensor` pallet `events.rs`.
- `src/utils/explain.rs` — built-in concept docs (32 topics per docs/llm.txt); cross-check with on-chain reality.
- `src/scaffold.rs` + `src/localnet.rs` — bootstrap chain scaffolding for tests.
- `src/queries/*` — cached chain reads; verify TTLs and at-block staleness.
- `docs/commands/*.md` — per-command reference, often drifts from clap surface.
- `docs/llm.txt` — agent-facing reference; should match every available CLI flag.
- `docs/hyperparameters.md` — should match every `admin-utils` `sudo_set_*`.

## Known environment constraints (root planner observations)

- Repo's `subtensor/rust-toolchain.toml` pins `1.89`; the cloud-agent VM ships with `1.83`. Workers must run `rustup default stable` (which lands a >=1.89 stable) before `cargo check` / `cargo build`.
- `cargo check --bin agcli` succeeds in ~1m25s on a cold cache.
- `docker` is **not** installed in the cloud-agent VM. Every test that invokes Docker (every `localnet`, every `*_e2e` test gated on the `e2e` feature, `localnet scaffold`) will fail in this environment. Workers must:
  1. Install `docker.io` (or `docker-ce`) themselves — the cloud-agent VM's user has `sudo` (Doppler installation succeeded earlier with sudo).
  2. OR mark their acceptance criterion "verified with localnet" as `not-verified` and explain in their handoff `Notes` that the environment lacks Docker.
- The `e2e` cargo feature gates Docker-dependent integration tests in `tests/{e2e_test,user_flows_e2e,localnet_e2e_test,chain_integration_test}.rs`. A worker that wants to exercise these needs Docker.
- `CURSOR_API_KEY` is not in the env. The orchestrate loop spawns cloud agents through the Cursor SDK, which requires `CURSOR_API_KEY`. If you read this file as a worker, you have already been spawned successfully — that means the operator has populated the secret since this discovery file was written.

## How to use this discovery in a worker

1. Read this file end-to-end before doing anything else.
2. Find your group in the table above; open the listed cmds module + docs file + tests file.
3. Cross-reference each subcommand with the pallet dispatchable from the mapping above.
4. Run `cargo check` (not full release build) to validate any code change quickly.
5. Use `agcli localnet scaffold` (after `docker` is installed) to get a chain to drive your green-path tests against. Fall back to read-only assertions if Docker is unavailable.
6. Write a focused green-path integration test for at least one of your subcommands — name it `green_path_<group>` and put it in the worker's `pathsAllowed` test file.
7. Update `docs/commands/<group>.md` so the on-chain behavior, pallet ref, storage keys, events, and exit codes are accurate.
8. Mention every subcommand you decided **not** to verify in the `## Notes, concerns, deviations, findings, thoughts, feedback` section of your handoff, with the reason.

## Acceptance shape every worker should meet

- `cargo check --all-targets` passes on the worker's branch.
- `cargo clippy -- -D warnings` is clean for the files the worker touched (best-effort if cross-module fixes are out of scope).
- Worker's group docs file documents every subcommand under that group — exit codes, args, output schema, pallet ref.
- At least one new green-path integration test added, with a `not-verified` marker if Docker is unavailable.
- Handoff includes `Findings:` bullets — concrete drift between code and docs, missing args, ill-formatted output, panicking paths, and any pallet dispatchable that has no CLI surface.

## Cloud-agent VM environment recipe (verified by root planner)

Workers should run this at the start of their session before doing audit work, because the cloud-agent VM is not preconfigured for this repo:

```bash
# 1) Rust >= 1.89 (Cargo.toml deps require edition2024 features).
rustup install stable && rustup default stable

# 2) Bun (only needed if you spawn nested orchestrate calls — most workers don't).
curl -fsSL https://bun.sh/install | bash && export PATH="$HOME/.bun/bin:$PATH"

# 3) Docker (cloud-agent VM lacks systemd; storage driver overlayfs fails inside
#    the unprivileged container — use vfs).
sudo apt-get update -y
sudo apt-get install -y docker.io
sudo mkdir -p /etc/docker
echo '{"features":{"containerd-snapshotter":false},"storage-driver":"vfs"}' | sudo tee /etc/docker/daemon.json
sudo dockerd > /tmp/dockerd.log 2>&1 &
sleep 5
sudo docker run --rm hello-world  # smoke test

# 4) Submodules — `subtensor/` is a git submodule pinned at a specific commit;
#    init it shallowly to avoid timeouts.
git submodule update --init --depth=1 -- subtensor

# 5) Build cache — `cargo check --bin agcli` takes ~1m30s cold, then is fast.
SKIP_METADATA_FETCH=1 cargo check --bin agcli
```

`build.rs` fetches chain metadata from `wss://entrypoint-finney.opentensor.ai:443` unless `SKIP_METADATA_FETCH=1` is set and a cached `metadata.rs` exists in `OUT_DIR`. Workers without internet egress to finney must set the env or supply a cached metadata file.

## Localnet image

`src/localnet.rs` defaults to `ghcr.io/opentensor/subtensor-localnet:devnet-ready` on host port 9944. Workers running localnet-backed tests must `sudo docker pull` that image before invoking `agcli localnet start` or `agcli localnet scaffold`.

## What worker handoffs MUST contain

- `## Status` — `success`, `partial`, or `blocked`.
- `## Branch` — the actual branch they pushed (the spawn already named it; do not rename).
- `## What I did` — bullets per file, naming each subcommand audited.
- `## Findings` — concrete drift, missing args, panicking paths, undocumented exit codes, bad output formatting (numbers as strings, missing fields, inconsistent JSON shapes), pallet dispatchables not surfaced in agcli, args mistyped vs the SCALE codec, etc. **This is the primary audit deliverable.**
- `## Verification` — `unit-test-verified` if the new green-path test passes locally; `not-verified` if Docker / network / sudo were unavailable.
- `## Suggested follow-ups` — things you noticed but were out of scope (e.g. cross-cutting changes to `src/error.rs` that should be a separate task).
verified at 2026-05-27T12:16:19Z UTC on rust rustc 1.95.0 (59807616e 2026-04-14)
verified at 2026-05-27T12:25:50Z on rust rustc 1.95.0 (59807616e 2026-04-14)
