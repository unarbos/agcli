<!-- orchestrate handoff
task: bootstrap-parity-env
branch: orch/agcli-parity/bootstrap-parity-env
agentId: bc-31a87ede-c62c-443a-94ac-7838c7b97364
runId: run-a92aa4d1-f027-40cd-82e1-7e4002e1085b
resultStatus: error
finishedAt: 2026-05-28T15:40:05.482Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m starting Phase 0 bootstrap now and will first read `.orchestrate/agcli-parity/discovery.md` as requested, then I’ll execute the environment setup steps in order and verify each scaffold variant by actually booting localnet.I’ve finished reading the discovery doc and next I’m aligning the git branch state, then I’ll run the bootstrap recipe step-by-step and capture version evidence as I go.I have the branch context and discovery details, so here’s the execution plan I’m following now:

1. **Bootstrap runtime/toolchain**: Rust stable, subtensor submodule, Docker (vfs daemon), localnet image pull, Python venv + `bittensor`/`bittensor-cli`.
2. **Build and baseline smoke**: build `agcli` release with `SKIP_METADATA_FETCH=1`, run default scaffold JSON smoke check, and capture required version/digest metadata.
3. **Variant scaffolds A–E**: create all five TOMLs under `examples/scaffold-variants/`, boot each one sequentially with unique container names/ports, validate JSON shape and record expected on-chain shape summaries.
4. **Write deliverables**: populate `docs/parity/versions.json` + `docs/parity/versions.md`, append the required `verified at ...` line in discovery.
5. **Finalize git**: commit on `orch/agcli-parity/bootstrap-parity-env`, push, and provide the requested handoff with measurements/findings/follow-ups.

I’m starting with branch creation plus the environment bootstrap commands now.I’ve switched to the required branch and completed Rust + submodule setup; next I’m handling Docker installation/daemon startup and image pull, checking existing terminal sessions first to avoid duplicate long-running daemons.