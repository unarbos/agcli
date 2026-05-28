<!-- orchestrate handoff
task: parity-network-readonly
branch: orch/phase3-parity-suite/parity-network-readonly
agentId: bc-35a60aa1-9203-45b2-8f1f-1365133a3286
runId: run-927052ca-c23f-4731-a3ec-b7b81d1b0558
resultStatus: error
finishedAt: 2026-05-28T17:43:16.084Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m implementing the scoped `network_readonly` parity worker task now: I’ll inspect existing parity harness/tests and matrix rows, add/adjust only the allowed files, run the required e2e command, then commit and push the exact branch requested before returning a single structured handoff.I found this repo doesn’t currently have a `tests/parity/` directory, so I’m locating the existing parity suite equivalents to map your requested files to the current layout before editing anything.