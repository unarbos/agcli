<!-- orchestrate handoff
task: parity-stake-advanced
branch: orch/phase3-parity-suite/parity-stake-advanced
agentId: bc-a97c2187-c4f1-4075-b5f8-2e7a5d66b9b3
runId: run-8286166e-3d49-45e4-9476-fe65f22a4db8
resultStatus: error
finishedAt: 2026-05-28T17:58:57.809Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m going to inspect the existing parity harness and stake-related tests first, then implement `tests/parity/stake_advanced.rs` and update the matrix entries for the advanced staking rows, followed by running the requested targeted e2e test command.I didn’t find those paths at `/workspace`, so I’m locating the actual repository layout next and then I’ll map the allowed file paths to the real location before editing.