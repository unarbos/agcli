<!-- orchestrate failure handoff
task: audit-proxy
branch: orch/agcli-audit/audit-proxy
agentId: bc-5e867a47-60bb-4bf4-b104-28de11b9ba29
runId: run-caa63c5e-d761-49b1-a2f1-2595095dae07
failureMode: unknown
terminatedAt: 2026-05-27T12:16:20.139Z
-->

# audit-proxy failure handoff

Status: error (cloud agent terminated without writing a handoff)
Failure mode: unknown
Cloud agent: bc-5e867a47-60bb-4bf4-b104-28de11b9ba29
Started: 2026-05-27T12:12:48.372Z
Terminated: 2026-05-27T12:16:20.139Z
Duration: 211767ms
Last activity: 2026-05-27T12:16:20.050Z - respawned by self-planner (was error; attempts=2)
Last tool call: run_terminal_cmd
Branch: orch/agcli-audit/audit-proxy
SDK error: (none recorded)

## Suggested next steps
- Retry as-is (treat as transient)
- Retry with smaller scope if this repeats
- Retry with different model if the same tool keeps failing
- Abandon: skip task, replan around it
