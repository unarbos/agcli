<!-- orchestrate failure handoff
task: audit-delegate
branch: orch/agcli-audit/audit-delegate
agentId: bc-e18cf6f1-224e-426a-9110-51aaa0839734
runId: run-aa73e0b6-9eb8-40ea-8528-adcd43afcf9b
failureMode: unknown
terminatedAt: 2026-05-27T12:17:35.919Z
-->

# audit-delegate failure handoff

Status: error (cloud agent terminated without writing a handoff)
Failure mode: unknown
Cloud agent: bc-e18cf6f1-224e-426a-9110-51aaa0839734
Started: 2026-05-27T12:11:32.466Z
Terminated: 2026-05-27T12:17:35.919Z
Duration: 363453ms
Last activity: 2026-05-27T12:17:15.484Z - respawned by self-planner (was error; attempts=2)
Last tool call: run_terminal_cmd
Branch: orch/agcli-audit/audit-delegate
SDK error: (none recorded)

## Suggested next steps
- Retry as-is (treat as transient)
- Retry with smaller scope if this repeats
- Retry with different model if the same tool keeps failing
- Abandon: skip task, replan around it
