<!-- orchestrate failure handoff
task: parity-root
branch: orch/phase3-parity-suite/parity-root
agentId: bc-89dda57a-ea86-47be-8b03-f2bc06337037
runId: run-7b9e2ac3-7134-4b73-a983-48094f2d52c6
failureMode: unknown
terminatedAt: 2026-05-28T18:46:03.385Z
-->

# parity-root failure handoff

Status: error (cloud agent terminated without writing a handoff)
Failure mode: unknown
Cloud agent: bc-89dda57a-ea86-47be-8b03-f2bc06337037
Started: 2026-05-28T17:22:44.966Z
Terminated: 2026-05-28T18:46:03.385Z
Duration: 4998419ms
Last activity: 2026-05-28T18:46:03.375Z - respawned by self-planner (was error; attempts=1)
Last tool call: run_terminal_cmd
Branch: orch/phase3-parity-suite/parity-root
SDK error: (none recorded)

## Suggested next steps
- Retry as-is (treat as transient)
- Retry with smaller scope if this repeats
- Retry with different model if the same tool keeps failing
- Abandon: skip task, replan around it
