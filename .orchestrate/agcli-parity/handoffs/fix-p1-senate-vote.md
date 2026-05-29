<!-- orchestrate handoff
task: fix-p1-senate-vote
branch: orch/agcli-parity/fix-p1-senate-vote
agentId: bc-9991f752-c6d3-4711-bff4-4e41b4ff5f09
runId: run-6863bb6d-675c-4538-bb2a-c8039e9d5320
resultStatus: error
finishedAt: 2026-05-29T06:39:42.228Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m implementing the `btcli.sudo.senate-vote` gap closure end to end: first I’ll read the required orchestrator docs, then inspect existing admin/raw governance call patterns, implement the senate-vote path, add a localnet parity write-path test, update the parity matrix row, and finish by committing, pushing `orch/agcli-parity/fix-p1-senate-vote`, and opening a draft PR.