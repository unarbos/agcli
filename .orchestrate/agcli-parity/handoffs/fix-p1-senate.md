<!-- orchestrate handoff
task: fix-p1-senate
branch: orch/agcli-parity/fix-p1-senate
agentId: bc-b81410a0-2589-4677-bac8-b13dacba4c63
runId: run-11d4254e-79bf-4677-9c81-d14a806b6cb4
resultStatus: error
finishedAt: 2026-05-29T06:40:46.787Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m going to close the `btcli.sudo.senate` gap end-to-end: first I’ll read the required discovery and triage docs, then implement the admin senate list path, add/adjust parity coverage, update the matrix/docs, and finish with commit/push plus a draft PR on the required branch.I’ve loaded the required discovery and triage context; next I’m inspecting existing admin command handling and chain query helpers to wire a dedicated `admin senate list` path with matching JSON output and then add parity coverage/matrix updates.I found there’s no existing senate-query helper yet, and `admin` only has `raw` plus `list`; I’m now adding a read-only `raw --call senate` pathway (no sudo write) backed by a new chain query for senate membership, then I’ll wire docs/matrix and validate with build/tests.