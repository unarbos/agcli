<!-- orchestrate failure handoff
task: bootstrap-env
branch: orch/agcli-audit/bootstrap-env
agentId: bc-ad166916-bce6-4acc-9653-4be1c1579983
runId: run-a6223dad-e15e-436a-8c05-2dc44a90590a
failureMode: unknown
terminatedAt: 2026-05-27T12:17:46.876Z
-->

# bootstrap-env failure handoff

Status: error (cloud agent terminated without writing a handoff)
Failure mode: unknown
Cloud agent: bc-ad166916-bce6-4acc-9653-4be1c1579983
Started: 2026-05-27T12:09:49.037Z
Terminated: 2026-05-27T12:17:46.876Z
Duration: 477839ms
Last activity: 2026-05-27T12:17:35.916Z - respawned by self-planner (was error; attempts=3)
Last tool call: run_terminal_cmd
Branch: orch/agcli-audit/bootstrap-env
SDK error: (none recorded)

## Suggested next steps
- Retry as-is (treat as transient)
- Retry with smaller scope if this repeats
- Retry with different model if the same tool keeps failing
- Abandon: skip task, replan around it
