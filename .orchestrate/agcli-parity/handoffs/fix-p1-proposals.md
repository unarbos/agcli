<!-- orchestrate handoff
task: fix-p1-proposals
branch: orch/agcli-parity/fix-p1-proposals
agentId: bc-62dee864-6b93-490d-b643-df5837c893f4
runId: run-55d1205e-1449-4813-a876-dfcb3a862852
resultStatus: error
finishedAt: 2026-05-29T06:39:43.685Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m starting by reading the orchestrator bootstrap docs first as requested, then I’ll inspect the scoped parity row for `btcli.sudo.proposals` and implement the command/test/matrix updates on a dedicated branch.I’ve loaded both required context docs and confirmed this row is explicitly scoped as a P1 governance GAP. Next I’m checking the existing admin command surface and parity test patterns to add `senate proposals` with localnet-backed verification.