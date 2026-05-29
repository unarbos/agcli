<!-- orchestrate handoff
task: fix-p1-senate
branch: orch/agcli-parity/fix-p1-senate
agentId: bc-4d5b1c46-9825-4021-92bc-0ad5641a4308
runId: run-3ac46119-b2ce-43e7-92ce-0cff5452824e
resultStatus: error
finishedAt: 2026-05-29T06:41:50.949Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

Understood — I’ll first read the required orchestration context files (`.orchestrate/agcli-parity/discovery.md` and `docs/parity/gap-triage.md`), then locate the `btcli.sudo.senate` matrix row and current admin command implementation before making the scoped `senate list` parity change.I’ve confirmed you’re already on the required base branch and found the `btcli.sudo.senate` GAP row. Next I’m inspecting current admin CLI handling and parity test patterns so I can add `admin senate list` in the existing command surface and wire the matrix/docs updates cleanly.