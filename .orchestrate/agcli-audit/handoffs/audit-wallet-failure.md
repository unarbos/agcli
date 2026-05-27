<!-- orchestrate failure handoff
task: audit-wallet
branch: orch/agcli-audit/audit-wallet
agentId: bc-18e4d458-bba6-403b-a12d-569e05c9c9da
runId: run-6b8b5769-c8b4-4888-bb0c-65855e612827
failureMode: unknown
terminatedAt: 2026-05-27T12:14:13.008Z
-->

# audit-wallet failure handoff

Status: error (cloud agent terminated without writing a handoff)
Failure mode: unknown
Cloud agent: bc-18e4d458-bba6-403b-a12d-569e05c9c9da
Started: 2026-05-27T12:10:58.329Z
Terminated: 2026-05-27T12:14:13.008Z
Duration: 194679ms
Last activity: 2026-05-27T12:14:12.935Z - respawned by self-planner (was error; attempts=2)
Last tool call: grep_search
Branch: orch/agcli-audit/audit-wallet
SDK error: (none recorded)

## Suggested next steps
- Retry as-is (treat as transient)
- Retry with smaller scope if this repeats
- Retry with different model if the same tool keeps failing
- Abandon: skip task, replan around it
