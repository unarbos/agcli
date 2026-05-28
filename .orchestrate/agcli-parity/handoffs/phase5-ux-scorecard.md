<!-- orchestrate handoff
task: phase5-ux-scorecard
branch: orch/agcli-parity/phase5-ux-scorecard
agentId: bc-0e67d4bd-8614-47df-b3f4-392d237c8f67
runId: run-8374cfc0-23cd-4f38-a7d3-c21fd3b96d1f
resultStatus: error
finishedAt: 2026-05-28T20:43:12.811Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m going to assemble the UX scorecard from the required upstream artifacts first (`discovery.md`, Phase 3/4 handoffs), then validate a few representative agcli/btcli command behaviors locally and synthesize everything into `docs/parity/ux-scorecard.md`. I’ll start by reading the relevant source files and mapping evidence to each required UX axis.