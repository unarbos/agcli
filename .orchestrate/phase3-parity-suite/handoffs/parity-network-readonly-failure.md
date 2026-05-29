<!-- orchestrate failure handoff
task: parity-network-readonly
branch: orch/phase3-parity-suite/parity-network-readonly
agentId: bc-35a60aa1-9203-45b2-8f1f-1365133a3286
runId: run-927052ca-c23f-4731-a3ec-b7b81d1b0558
failureMode: unknown
terminatedAt: 2026-05-28T17:43:16.084Z
-->

# parity-network-readonly failure handoff

Status: error (cloud agent terminated without writing a handoff)
Failure mode: unknown
Cloud agent: bc-35a60aa1-9203-45b2-8f1f-1365133a3286
Started: 2026-05-28T17:23:48.440Z
Terminated: 2026-05-28T17:43:16.084Z
Duration: 1167644ms
Last activity: 2026-05-28T17:43:15.941Z - respawned by self-planner (was error; attempts=2)
Last tool call: read_file
Branch: orch/phase3-parity-suite/parity-network-readonly
SDK error: (none recorded)

## Suggested next steps
- Retry as-is (treat as transient)
- Retry with smaller scope if this repeats
- Retry with different model if the same tool keeps failing
- Abandon: skip task, replan around it
