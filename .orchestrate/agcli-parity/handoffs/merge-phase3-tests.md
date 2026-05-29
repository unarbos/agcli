<!-- orchestrate handoff
task: merge-phase3-tests
branch: orch/agcli-parity/phase3-tests
agentId: bc-93542a81-ff86-429a-869b-5687f8fcad27
runId: run-afde4b62-493d-445d-b123-7a0270dfd589
resultStatus: error
finishedAt: 2026-05-29T06:42:01.439Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

Understood — I’ll first read the required context files, then switch to `orch/agcli-parity/phase3-tests`, merge the specified parity branches one-by-one (keeping both test additions on conflicts), reconcile `docs/parity/matrix.json`, and finish with compile verification plus a pushed branch handoff.I’ve loaded the required discovery and triage context; next I’m moving to the required integration branch, fetching the parity source branches, and starting merges one at a time while capturing any conflicts and matrix-row flips.I confirmed all required remote branches exist; I’m now creating/switching to `orch/agcli-parity/phase3-tests`, merging dependency branch `orch/agcli-parity/env-setup-followup` first, then the five phase3 parity-suite branches in sequence.I hit the first conflict in `docs/parity/matrix.json` during `parity-balance-transfer`; I’ll resolve by preserving all existing rows and keeping/elevating `parity_test` + `COVERED_E2E` updates (never downgrading).