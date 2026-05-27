<!-- orchestrate failure handoff
task: audit-config
branch: orch/agcli-audit/audit-config
agentId: bc-80ad2493-a4af-4e14-ba78-c5296094118e
runId: run-9ba11aa0-6557-4b43-92c2-c43fcc906927
failureMode: unknown
terminatedAt: 2026-05-27T12:18:20.795Z
-->

# audit-config failure handoff

Status: error (cloud agent terminated without writing a handoff)
Failure mode: unknown
Cloud agent: bc-80ad2493-a4af-4e14-ba78-c5296094118e
Started: 2026-05-27T12:17:15.495Z
Terminated: 2026-05-27T12:18:20.795Z
Duration: 65300ms
Last activity: 2026-05-27T12:18:20.714Z - respawned by self-planner (was error; attempts=2)
Last tool call: read_file
Branch: orch/agcli-audit/audit-config
SDK error: (none recorded)

## Suggested next steps
- Retry as-is (treat as transient)
- Retry with smaller scope if this repeats
- Retry with different model if the same tool keeps failing
- Abandon: skip task, replan around it
