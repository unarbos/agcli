<!-- orchestrate handoff
task: fix-p1-senate-vote
branch: orch/agcli-parity/fix-p1-senate-vote
agentId: bc-f1141234-958b-46a1-a13f-e2961ff1dddc
runId: run-d110fcf1-03f0-4fc5-981b-e7a5edf123ba
resultStatus: error
finishedAt: 2026-05-29T06:42:08.871Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

Understood: I’ll close the `btcli.sudo.senate-vote` gap end-to-end by first reading the orchestrator docs you specified, then implementing the admin senate vote path, adding/parsing a localnet parity test if needed, and finally updating the matrix row plus opening a draft PR on the required branch.I’ve loaded the orchestration context and confirmed the repo is on `cursor/agcli-parity-orchestrate-88b1`; next I’m inspecting the current admin/governance command wiring so I can add the senate vote path in-scope and mirror existing localnet parity test patterns.