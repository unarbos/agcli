# agcli-parity — Root Planner Discovery Notes

This file is the bootstrap reference material for every worker, subplanner, and verifier under the `agcli-parity` orchestrate root. **Read it end-to-end before doing anything else**, then revisit your task's `scopedGoal` with this context in mind.

Mission: bring **agcli** to functional parity with **btcli** (latent-to/btcli) and the **Bittensor Python SDK** (opentensor/bittensor) on every chain-relevant operation, then exceed them on coverage, agent-friendliness, and fault tolerance. Every write-path claim must be validated against a real local subtensor chain (Docker), not mocked.

## North star

For every workflow an operator can do with btcli or `bittensor` Python, there is an agcli path that:

1. works on the localnet image `ghcr.io/opentensor/subtensor-localnet:devnet-ready`,
2. produces equivalent on-chain effects (same storage delta, equivalent events, same balance/stake outcome),
3. is easier to script (`--batch`, `--yes`, `--output json`, `--dry-run`, structured errors, spending limits).

## Non-negotiable constraints

- **Localnet is mandatory for write-path validation.** Unit/CLI-parse tests do not count as covered.
- **Chain effects are source of truth.** Compare storage / events / balances before and after — not just exit codes or stdout shape.
- **Use the existing agcli harness** at `tests/e2e_modules/harness.rs`. Reuse `ensure_local_chain`, `wait_for_chain`, `dev_pair`, `ensure_alive`, `sudo_admin_call`, `setup_subnet`, `setup_global_rate_limits` rather than reimplementing.
- **Pin reference tool versions.** Record `btcli --version`, `pip show bittensor`, and the resolved git SHAs in `docs/parity/versions.json` so reruns are reproducible.
- **Do not skip btcli.** Even where agcli claims superset, run the btcli command on the same chain state to confirm equivalence.
- **Out of scope**: miner/validator application runtime (Axon/Dendrite HTTP servers, Yuma math internals, subnet-specific scoring). Focus on chain operations the CLI/SDK expose.

## Repo layout pointers (verified)

- `src/cli/mod.rs` (~3204 LOC) defines `Cli` + 28 `*Commands` enums (the agcli surface).
- `src/cli/commands.rs` dispatches to per-group handlers; per-group files in `src/cli/{wallet,stake,subnet,view,weights,admin,system,block,localnet,network}_cmds.rs`.
- `src/chain/extrinsics.rs` (~129 `pub fn`s) — every subxt extrinsic submitter (mapping target).
- `src/chain/queries.rs` (~71 `pub fn`s) — read-only chain queries.
- `src/error.rs` — exit code map + pallet error → human message.
- `src/scaffold.rs` (~818 LOC) + `src/localnet.rs` (~393 LOC) — scaffold orchestration & Docker lifecycle.
- `docs/why-agcli.md` — claimed parity & superset (415 lines). **This document is what we are validating.**
- `docs/llm.txt` — agent-facing reference; the agcli surface as advertised to LLMs.
- `docs/commands/` — 31 per-group `.md` files (produced by the prior `agcli-audit` orchestrate run; trustworthy starting point for what agcli claims to do).
- `subtensor/pallets/` (git submodule, pinned at `6844ee37f0b8cb02baf9ff8d3ca4319cfb33f361`) — pallet sources; cross-reference dispatchables here.
- `examples/scaffold.toml` — single-subnet default scaffold; Phase 0 adds variant configs A-E.

## Existing localnet test infrastructure

- `tests/e2e_test.rs` (~320 LOC) — top-level e2e suite.
- `tests/user_flows_e2e.rs` (~7087 LOC) — heavyweight scenario suite. Read before adding parallel flows.
- `tests/localnet_e2e_test.rs` (~692 LOC) — Docker chain lifecycle tests.
- `tests/e2e_modules/harness.rs` (~766 LOC) — shared harness. **Always reuse, never duplicate.**
  - Public API: `ensure_local_chain()`, `wait_for_chain()`, `dev_pair("//Alice")`, `to_ss58(...)`, `ensure_alive(&mut client)`, `ensure_alice_on_subnet(...)`, `wait_blocks(...)`, `sudo_admin_call(...)`, `setup_subnet(...)`, `setup_global_rate_limits(...)`, `salt_bytes_to_reveal_vec(...)`, `sudo_set_commit_reveal_weights_or_fail(...)`.
  - Constants: `LOCAL_WS = "ws://127.0.0.1:9944"`, `CONTAINER_NAME = "agcli_e2e_test"`, `DOCKER_IMAGE`, `ALICE_URI`, `ALICE_SS58`, `BOB_URI`, `BOB_SS58`.
  - Retry helpers handle transient WS disconnects (read `is_retryable`, `needs_fresh_conn`, `retry_delay_ms`).
- `tests/e2e_modules/cases_part01..03.rs` — split parameterized cases; parity tests should add a `tests/parity/` directory rather than expanding these files.

## Cloud-agent VM environment recipe

The cloud-agent VM is bare. Every worker needing to actually run code should bootstrap its own toolchain unless Phase 0 has already populated this discovery file's "verified at" stamp:

```bash
# 1) Rust toolchain (Cargo.toml requires >= 1.89)
rustup install stable && rustup default stable

# 2) Submodules (subtensor pallet sources for cross-reference)
git submodule update --init --depth=1 -- subtensor

# 3) Docker (VM lacks systemd; use vfs storage driver)
sudo apt-get update -y
sudo apt-get install -y docker.io
sudo mkdir -p /etc/docker
echo '{"features":{"containerd-snapshotter":false},"storage-driver":"vfs"}' | sudo tee /etc/docker/daemon.json
sudo dockerd > /tmp/dockerd.log 2>&1 &
sleep 5
sudo docker run --rm hello-world  # smoke test

# 4) Pull the localnet image (needed before any *_e2e test runs)
sudo docker pull ghcr.io/opentensor/subtensor-localnet:devnet-ready

# 5) Python venv with reference tools (Phase 0 only; later phases consume their pins)
python3 -m venv .venv && source .venv/bin/activate
pip install --upgrade pip
pip install bittensor bittensor-cli   # Phase 0 records resolved versions
btcli --version

# 6) Build agcli release binary (every parity test invokes the binary)
SKIP_METADATA_FETCH=1 cargo build --release --bin agcli
target/release/agcli --version
```

`build.rs` reads `SKIP_METADATA_FETCH=1` to skip fetching live chain metadata from `wss://entrypoint-finney.opentensor.ai:443` and use the cached `metadata.rs`. Always set it unless you intentionally need fresh finney metadata.

## Scaffold variants (Phase 0 produces these)

Phase 0 (`bootstrap-parity-env`) writes the following scaffold configs to `examples/scaffold-variants/`:

| Variant | File | What it exercises |
|---------|------|-------------------|
| A | `scaffold-A-baseline.toml` | 1 subnet, 3 neurons (1 validator + 2 miners), commit-reveal **off** (default-ish baseline). |
| B | `scaffold-B-commit-reveal.toml` | 1 subnet, 3 neurons, commit-reveal **on**, short reveal interval for fast tests. |
| C | `scaffold-C-multi-subnet.toml` | 2 subnets with distinct tempos/hyperparams; tests cross-subnet ops (move/swap stake). |
| D | `scaffold-D-mechanisms.toml` | 1 subnet with `mechanism_count > 1` (Yuma + alt mechanism); tests mechanism-aware queries. |
| E | `scaffold-E-user-liquidity.toml` | 1 subnet with `toggle_user_liquidity` enabled and seeded LP positions; tests `swap`/`liquidity` commands. |

Each variant must boot via `agcli localnet scaffold --config examples/scaffold-variants/scaffold-<X>-*.toml --output json` and have its `result.subnets`/`result.neurons` field validated.

## Reference tools (Phase 0 pins, all later phases consume)

`docs/parity/versions.json` (Phase 0 deliverable) records:

```json
{
  "agcli": { "git_sha": "...", "build": "cargo --release", "version": "..." },
  "btcli": { "pypi_version": "...", "git_sha": "..." },
  "bittensor": { "pypi_version": "...", "git_sha": "..." },
  "subtensor_localnet_image": "ghcr.io/opentensor/subtensor-localnet:devnet-ready",
  "subtensor_image_digest": "sha256:...",
  "subtensor_submodule_sha": "6844ee37f0b8cb02baf9ff8d3ca4319cfb33f361",
  "rustc": "...",
  "python": "3.12.x",
  "docker": "..."
}
```

## Phase-by-phase task overview

| Phase | Task name | Purpose | Deliverables |
|---|---|---|---|
| 0 | `bootstrap-parity-env` | Toolchain + Docker + reference tools + scaffold variants + version manifest. | `docs/parity/versions.{json,md}`, `examples/scaffold-variants/*.toml`, updated discovery stamp. |
| 1 | `inventory-btcli` | Exhaustive enumeration of every `btcli` subcommand, args, on-chain mapping. | `docs/parity/inventory-btcli.{md,json}` |
| 1 | `inventory-sdk` | Exhaustive enumeration of chain-relevant `bittensor` SDK helpers (subtensor extrinsics, async_subtensor). | `docs/parity/inventory-sdk.{md,json}` |
| 2 | `parity-matrix` | Join the two inventories with agcli's command tree (src/cli/mod.rs + docs/llm.txt + docs/commands/*). | `docs/parity/matrix.json`, `docs/parity/matrix.md` (human view). |
| 3 | `phase3-parity-suite` (subplanner) | One parity test worker per category; each boots fresh localnet and runs btcli/SDK + agcli + on-chain assertions. | `tests/parity/<category>.rs`, per-category handoffs. |
| 4 | `phase4-gap-closure` (subplanner) | One narrow worker per `GAP` row in `matrix.json`; implements + tests + commits + (optionally) opens its own PR. | Updates to src/cli/**, src/chain/**, docs/commands/**, tests/parity/**; matrix.json row flipped GAP → COVERED_E2E. |
| 5 | `phase5-ux-scorecard` | UX & fault-tolerance scorecard vs btcli (--batch, --dry-run, structured errors, spending limits, preflight). | `docs/parity/ux-scorecard.md`. |
| 6 | `phase6-ci-and-report` | CI parity-localnet job + final parity-report.md. | `.github/workflows/parity-localnet.yml`, `docs/parity/parity-report.md`. |

## Matrix row schema (Phase 2 deliverable, every later phase consumes)

`docs/parity/matrix.json` is an array of rows. Every row has:

```json
{
  "id": "btcli.wallet.transfer",                // stable id; "btcli.<group>.<cmd>" or "sdk.<module>.<fn>"
  "source": "btcli|sdk",
  "ref_invocation": "btcli wallet transfer --destination 5G... --amount 1.0",
  "ref_extrinsic": "Balances.transfer_keep_alive",
  "agcli_invocation": "agcli transfer --dest 5G... --amount 1.0",
  "agcli_status": "COVERED_E2E|COVERED_CLI_ONLY|COVERED_UNIQUE|GAP|N/A",
  "agcli_flag_diffs": ["btcli: --destination ↔ agcli: --dest"],
  "parity_test": "tests/parity/balance.rs::transfer_parity",  // null until phase 3
  "notes": "free text"
}
```

`agcli_status`:

- `COVERED_E2E` — agcli has the command **and** a `tests/parity/` test validates equivalent on-chain effects.
- `COVERED_CLI_ONLY` — agcli has the command but no localnet parity test (Phase 3 target).
- `COVERED_UNIQUE` — agcli has the command and no btcli/SDK equivalent (still needs a localnet test where it touches chain state; the "ref" half is N/A).
- `GAP` — agcli has no equivalent (Phase 4 target).
- `N/A` — out-of-scope per the mission (e.g. Axon HTTP runtime, Yuma internals).

Definition of done (mission-level): every btcli command is `COVERED_E2E`, every SDK extrinsic helper is `COVERED_E2E`, no `COVERED_CLI_ONLY` for write paths, UX scorecard says agcli ≥ btcli on every axis, full e2e suite green.

## Worker guardrails

- **`pathsAllowed` is law.** Each task lists only the files it may modify. If a finding requires source changes outside your scope, list it as `## Suggested follow-ups` in your handoff — do not modify out-of-scope files.
- **No mocked chain for sign-off.** Compile-only verification (`type-check-only`) does not satisfy any Phase 3+ acceptance criterion. Boot a localnet, run the commands, assert the storage delta.
- **Use scaffold variants for keys.** Do not hand-roll dev wallets. Read the JSON output of `agcli localnet scaffold --output json --config examples/scaffold-variants/<variant>.toml` to get pre-funded SS58s and pre-registered UIDs.
- **Document flag differences explicitly.** When btcli says `--destination` and agcli says `--dest`, record the diff in the matrix row's `agcli_flag_diffs`. Do not silently rename agcli flags to match btcli; agcli's flag choices are intentional (read `docs/why-agcli.md`).
- **One handoff per task.** Follow the prompt templates exactly; the `## Findings` section is the substantive deliverable for inventory + matrix tasks, and `## Verification` is the substantive deliverable for parity-test workers.
- **Reuse `tests/e2e_modules/harness.rs` patterns** to absorb flakiness (reconnect on WS drop, wait for finalization, retry on retryable errors).

## Existing prior orchestrate run (read-only reference)

`.orchestrate/agcli-audit/` (sibling workspace, already completed) contains 48 handoffs documenting agcli's existing surface group-by-group. Workers should treat those handoffs as a **factual snapshot** of agcli today (what it claims to expose and what its docs say). The parity mission validates those claims against btcli/SDK and surfaces gaps.

## Acceptance for every worker (default)

- `cargo check --all-targets` clean on the worker branch.
- For parity-test workers: a `tests/parity/<name>.rs` (or appended file) that **actually runs against localnet** and asserts the equivalent on-chain delta.
- Handoff `## Verification` is one of `live-ui-verified`, `unit-test-verified`, `type-check-only`, `verifier-blocked`, `verifier-failed`. Phase 3+ rejects anything weaker than `unit-test-verified` for the parity tests themselves, with a strong preference for evidence that the test ran against a real localnet.
- Handoff `## Findings` enumerates concrete drift, flag rename suggestions, missing commands, ill-formed output, panics, exit-code mismatches.

## Verified at

(Phase 0 appends a one-line `verified at <UTC iso> on rustc <version>, docker <version>, btcli <version>, bittensor <version>, agcli <git sha>` line below.)
verified at 2026-05-28T16:13:46Z on rustc 1.95.0, docker 29.1.3, btcli 9.21.2, bittensor 10.3.2, agcli dc6ac77a39e6a9236bdca45219433fb46b46adce, localnet image digest sha256:10848ffc226f6156ef831463937299e1f4d34732c7ab1e97f348bca3f8e751c9.
