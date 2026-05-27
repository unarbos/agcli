<!-- orchestrate failure handoff
task: audit-contracts
branch: orch/agcli-audit/audit-contracts
agentId: bc-3adcf2aa-482f-4079-b9f9-851a68fc471e
runId: run-5c8ee49b-aae0-4cb8-909a-52c4063dd533
failureMode: unknown
terminatedAt: 2026-05-27T12:17:53.817Z
-->

# audit-contracts failure handoff

Status: error (cloud agent terminated without writing a handoff)
Failure mode: unknown
Cloud agent: bc-3adcf2aa-482f-4079-b9f9-851a68fc471e
Started: 2026-05-27T12:15:24.712Z
Terminated: 2026-05-27T12:17:53.817Z
Duration: 149105ms
Last activity: 2026-05-27T12:17:46.874Z - respawned by self-planner (was error; attempts=2)
Last tool call: run_terminal_cmd
Branch: orch/agcli-audit/audit-contracts
SDK error: (none recorded)

## Suggested next steps
- Retry as-is (treat as transient)
- Retry with smaller scope if this repeats
- Retry with different model if the same tool keeps failing
- Abandon: skip task, replan around it
