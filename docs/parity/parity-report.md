## Executive summary
- `docs/parity/matrix.json` currently has 397 rows and status counts: `COVERED_E2E=0`, `COVERED_CLI_ONLY=302`, `COVERED_UNIQUE=26`, `GAP=65`, `N/A=4`.
- This Phase 6 pass filled 38 previously-null `parity_test` cells from landed Phase 3 worker outputs, with no status flips.
- Write-path parity is still incomplete. There are 160 `COVERED_CLI_ONLY` rows that map to a non-empty `ref_extrinsic`.
- Remaining gaps by triage tier are unchanged at `P0=1`, `P1=19`, `P2=45`.
- Result versus mission DoD is partial and currently failing.
- Failing criteria: no rows are `COVERED_E2E`, write-path `COVERED_CLI_ONLY` is non-zero, and full e2e green status has not been demonstrated.
- Completed criteria in this phase are infrastructure-oriented: parity-localnet CI workflow added, versions pinned, and final report assembled with evidence.

## Versions
```json
{
  "agcli": {
    "git_sha": "dc6ac77a39e6a9236bdca45219433fb46b46adce",
    "version": "agcli 0.1.0",
    "build": "cargo --release"
  },
  "btcli": {
    "pypi_version": "9.21.2",
    "command": "pip install bittensor-cli==9.21.2",
    "homepage": "https://github.com/opentensor/btcli"
  },
  "bittensor": {
    "pypi_version": "10.3.2",
    "command": "pip install bittensor==10.3.2",
    "homepage": "https://github.com/opentensor/bittensor"
  },
  "subtensor_localnet_image": {
    "tag": "ghcr.io/opentensor/subtensor-localnet:devnet-ready",
    "digest": "sha256:10848ffc226f6156ef831463937299e1f4d34732c7ab1e97f348bca3f8e751c9"
  },
  "subtensor_submodule_sha": "6844ee37f0b8cb02baf9ff8d3ca4319cfb33f361",
  "rustc": "rustc 1.95.0 (59807616e 2026-04-14)",
  "docker": "Docker version 29.1.3, build 29.1.3-0ubuntu3~24.04.2",
  "python": "Python 3.12.3",
  "recorded_at": "2026-05-28T16:13:46Z"
}
```

## Phase-by-phase summary
### Phase 0
- Toolchain/bootstrap phase recorded the pinned versions in `docs/parity/versions.json` and established the localnet image digest for reproducible parity runs.
- Phase 0 outputs are present and consumed directly by this workflow/report pass (`docs/parity/versions.json`, `docs/parity/versions.md` from `orch/agcli-parity/bootstrap-toolchain`).

### Phase 1
- btcli and SDK inventories were produced as separate artifacts (`docs/parity/inventory-btcli.*`, `docs/parity/inventory-sdk.*`).
- Those inventories became the source input to the Phase 2 parity matrix.

### Phase 2
- Matrix baseline exists with 397 rows (`docs/parity/matrix.json`).
- This cycle preserved all row statuses and only filled missing `parity_test` pointers where Phase 3 evidence existed.

### Phase 3
- Aggregator handoff reports 5 landed categories (`parity-balance-transfer`, `parity-wallet`, `parity-stake-basic`, `parity-governance`, `parity-misc`) and 7 incomplete categories (`.orchestrate/agcli-parity/handoffs/phase3-parity-suite.md`).
- Confirmed drifts from landed work remain: `associate-hotkey` chain-effect mismatch and `regen-coldkey` non-interactive behavior mismatch (`.orchestrate/phase3-parity-suite/handoffs/parity-wallet.md`).

### Phase 4
- Gap-closure subplanner did not converge. Attention log evidence: "phase4-gap-closure: tool_call idle 1609907ms; last=none since wait start (2026-05-28T18:51:23.927Z)" followed by "phase4-gap-closure: run.wait returned status=error; recovery probe found terminal status=error, accepting authoritative status" (`.orchestrate/agcli-parity/state.json`).
- Triage worker succeeded with tiered follow-up plan and `GAP` tier tagging (`docs/parity/gap-triage.md`).
- Cap-hit context is recorded verbatim in the triage failure handoff: "renamed from phase4-gap-closure (subplanner cap-hit) to a single triage worker" (`.orchestrate/agcli-parity/handoffs/phase4-gap-triage-failure.md`).

### Phase 5+6
- Phase 5 retries exhausted. Attention log repeats the same failure pattern: "phase5-ux-scorecard: run.wait returned status=error; recovery probe found terminal status=error, accepting authoritative status" and "phase5-ux-scorecard: respawned by self-planner (was error)" (`.orchestrate/agcli-parity/state.json`).
- This combined pass delivered the CI workflow (`.github/workflows/parity-localnet.yml`), final report (`docs/parity/parity-report.md`), and 38 `parity_test` fill-ins in `docs/parity/matrix.json` without status mutations.

## UX scorecard
| UX axis | btcli | agcli | Verdict |
|---|---|---|---|
| non-interactive default | Interactive by default, automation path is `--no-prompt` (`docs/why-agcli.md`: L17, L81). | Interactive by default, automation path is `--batch` plus `--yes` (`docs/why-agcli.md`: L17, L77-L83). | tie |
| hard-error / batch mode | No equivalent hard-error mode (`docs/why-agcli.md`: L82). | `--batch` enforces non-blocking behavior and structured failures (`docs/why-agcli.md`: L82-L91). | agcli wins |
| structured error output | Exit-code-first behavior (`docs/why-agcli.md`: L83). | JSON structured errors in batch/JSON mode (`docs/why-agcli.md`: L83) and asserted in Phase 3 parity tests (`tests/parity/balance_transfer.rs::transfer_and_balance_parity_btcli_sdk_vs_agcli`, `tests/parity/misc.rs::misc_group_ux_claims_json_errors`). | agcli wins |
| `--dry-run` previews | Not available (`docs/why-agcli.md`: L85). | Available on write paths (`docs/why-agcli.md`: L85, L321) and exercised in Phase 3 (`tests/parity/balance_transfer.rs::transfer_and_balance_parity_btcli_sdk_vs_agcli`, `tests/parity/wallet.rs::wallet_filesystem_and_sdk_workflows_parity`). | agcli wins |
| output formats | Table and JSON (`docs/why-agcli.md`: L26, L87). | Table, JSON, and CSV (`docs/why-agcli.md`: L26, L87). | agcli wins |
| spending limits | No equivalent safety cap (`docs/why-agcli.md`: L325). | Spending-limit controls are explicit (`docs/why-agcli.md`: L17, L325). | agcli wins |
| password handling | Requires tty or keyring path (`docs/why-agcli.md`: L86). | Supports `AGCLI_PASSWORD` / `--password` non-tty operation (`docs/why-agcli.md`: L86). | agcli wins |
| exit codes | Known non-interactive inconsistency: `regen-coldkey` emitted `success=false` JSON with exit 0 (`.orchestrate/phase3-parity-suite/handoffs/parity-wallet.md`). | Stricter error semantics were kept and called out in triage (`docs/parity/gap-triage.md`: L117). | agcli wins |
| shell completions | Not available (`docs/why-agcli.md`: L27, L88). | bash, zsh, fish, PowerShell completions (`docs/why-agcli.md`: L27, L88). | agcli wins |
| cold-start speed | 2-5s startup class (`docs/why-agcli.md`: L13). | ~50ms startup class (`docs/why-agcli.md`: L13, L38, L50). | agcli wins |
| reconnect/retry | Partial retry behavior on some operations (`docs/why-agcli.md`: L180). | Built-in backoff/retry positioning (`docs/why-agcli.md`: L180-L186) and harness retry patterns (`tests/e2e_modules/harness.rs`: `is_retryable`, `retry_extrinsic!`). | agcli wins |
| at-block historical queries | Not supported (`docs/why-agcli.md`: L20, L135). | `--at-block` on read commands (`docs/why-agcli.md`: L20, L135). | agcli wins |
| live streaming | Not supported (`docs/why-agcli.md`: L21, L238). | Built-in live and subscribe flows (`docs/why-agcli.md`: L21, L104, L119, L227-L236). | agcli wins |
| cache hit-rate / coalescing | Disk cache but no request coalescing (`docs/why-agcli.md`: L62-L73). | 3-layer cache with request coalescing (`docs/why-agcli.md`: L15, L52-L73). | agcli wins |
| Windows native support | WSL2-only (`docs/why-agcli.md`: L30). | Native Windows support (`docs/why-agcli.md`: L30). | agcli wins |

- Tie follow-up: both CLIs still require an explicit non-interactive flag, so default invocation behavior is not yet "agent-safe" without policy flags.

### Fault-tolerance
| scenario | evidence | current read |
|---|---|---|
| chain RPC drop mid-extrinsic | Harness retries transient errors and reconnects (`tests/e2e_modules/harness.rs`: `is_retryable`, `needs_fresh_conn`, `retry_delay_ms`, `retry_extrinsic!`). | agcli path has explicit resilience scaffolding; btcli evidence is weaker/partial in this repo snapshot. |
| conflicting nonce | No direct landed parity test in Phase 3; triage still lists extrinsic-param validation as open SDK parity work (`docs/parity/gap-triage.md`: L52, L60). | not yet parity-verified |
| hotkey not registered | Harness includes registration/liveness retry loop around subnet presence checks (`tests/e2e_modules/harness.rs`: L150-L184). | partially covered by harness behavior, not closed as explicit parity case |
| insufficient balance | Phase 3 validates JSON error contracts on transfer/wallet/misc invalid flows (`.orchestrate/phase3-parity-suite/handoffs/parity-balance-transfer.md`, `.orchestrate/phase3-parity-suite/handoffs/parity-wallet.md`). | partially covered, but insufficient-balance specific parity assertion is still missing |
| wrong network endpoint | Chain bootstrap/liveness checks exist (`tests/e2e_modules/harness.rs`: `ensure_local_chain`, `wait_for_chain`, `ensure_alive`). | guarded for localnet lifecycle, not explicitly parity-tested against wrong endpoint inputs |
| password mismatch | Only related evidence is wallet regen semantics drift (`.orchestrate/phase3-parity-suite/handoffs/parity-wallet.md`, `docs/parity/gap-triage.md`: L117). | not yet parity-verified |
| container not running | Harness auto-boots localnet container before tests (`tests/e2e_modules/harness.rs`: `ensure_local_chain`). | agcli test harness coverage exists; parity suite still incomplete overall |

## Gap closure table
| row id | phase 4 closure status | evidence |
|---|---|---|
| (none closed in this cycle) | 0 rows moved GAP -> COVERED_E2E | Phase 4 produced triage only and explicitly deferred implementation to follow-up waves (`docs/parity/gap-triage.md`, `## Recommended follow-up orchestrate plan`). |

## Remaining gaps
- Triage-tier counts from `docs/parity/gap-triage.md`: `P0=1`, `P1=19`, `P2=45`.
- Quoted triage executive summary:

> "Total matrix rows: **397**."
>
> "Counts by `agcli_status`:"
> - "`COVERED_CLI_ONLY`: **302**"
> - "`COVERED_UNIQUE`: **26**"
> - "`GAP`: **65**"
> - "`N/A`: **4**"
> - "`COVERED_E2E`: **0** (this branch snapshot does not yet include merged Phase 3 row flips)"

- The recommended closure sequence remains unchanged: env setup, respawn 7 incomplete Phase 3 categories, then P0/P1 implementation waves (`docs/parity/gap-triage.md`, `## Recommended follow-up orchestrate plan`).

## Definition-of-done checklist
- [x] btcli command inventory is mapped into the matrix (93 btcli rows present in `docs/parity/matrix.json`).
- [ ] 100% btcli workflows are localnet-verified as `COVERED_E2E` (currently `COVERED_E2E=0`). Follow-up: `docs/parity/gap-triage.md`, `## Recommended follow-up orchestrate plan`.
- [x] SDK helper inventory is mapped into the matrix (278 SDK rows present in `docs/parity/matrix.json`).
- [ ] 100% SDK extrinsic helpers are localnet-verified as `COVERED_E2E` (currently `COVERED_E2E=0`). Follow-up: `docs/parity/gap-triage.md`, `## Recommended follow-up orchestrate plan`.
- [ ] zero `COVERED_CLI_ONLY` rows for writes (currently 160 write-like rows are still `COVERED_CLI_ONLY`). Follow-up: `docs/parity/gap-triage.md`, `## Recommended follow-up orchestrate plan`.
- [ ] agcli UX score is confidently >= btcli across all required axes with complete parity evidence (Phase 5 retries exhausted; this scorecard is evidence-backed but not end-state complete). Follow-up: `docs/parity/gap-triage.md`, `## Recommended follow-up orchestrate plan`.
- [ ] all e2e suites pass in final integrated state (7 parity categories remain incomplete per Phase 3 aggregator). Follow-up: `docs/parity/gap-triage.md`, `## Recommended follow-up orchestrate plan`.
- [x] Phase 6 CI baseline now exists with version-pinned installs, digest-pinned localnet image pull, and artifact upload (`.github/workflows/parity-localnet.yml`).

## Reproduction recipe
```bash
git clone https://github.com/unarbos/agcli.git
cd agcli

# Python tools
python3 -m venv .venv
source .venv/bin/activate
pip install uv
uv pip install "bittensor-cli==9.21.2" "bittensor==10.3.2"

# Localnet image (digest-pinned)
sudo docker pull ghcr.io/opentensor/subtensor-localnet:devnet-ready@sha256:10848ffc226f6156ef831463937299e1f4d34732c7ab1e97f348bca3f8e751c9

# Build agcli
SKIP_METADATA_FETCH=1 cargo build --release --bin agcli

# Run only parity targets that actually exist under tests/parity/
if [ -d tests/parity ]; then
  for f in tests/parity/*.rs; do
    stem=$(basename "$f" .rs)
    [ "$stem" = "mod" ] && continue
    cargo test --features e2e --test "parity_${stem}" -- --nocapture
  done
else
  echo "tests/parity not present; nothing to run"
fi
```

Local worker run evidence for this recipe shape:
- `source .venv/bin/activate && sudo docker pull ...@sha256:10848... && SKIP_METADATA_FETCH=1 cargo build --release --bin agcli && ...`
- Result: completed successfully on this branch, `Finished release profile` and `tests/parity not present; nothing to run`.
