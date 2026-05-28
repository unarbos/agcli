<!-- orchestrate handoff
task: parity-weights
branch: orch/phase3-parity-suite/parity-weights
agentId: bc-a573c2ce-3e15-4327-ba0d-7c69b0f05913
runId: run-c7dd0e63-cfe8-4732-886b-e1a730fd9077
resultStatus: error
finishedAt: 2026-05-28T17:50:59.549Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’ll implement the weights parity category end-to-end in the allowed test/matrix files, then run the required `cargo test --features e2e --test parity_weights -- --nocapture` verification and produce the handoff in the exact format. I’m starting by inspecting the existing parity harness patterns and current matrix rows for weights operations.I found this repo’s current layout differs from the paths in your prompt, so I’m locating the existing parity-style weight tests and harness modules first, then I’ll map your requested operations into that structure while staying within the allowed files if they exist.I confirmed the expected parity paths don’t exist on the current detached commit, so I’m checking out the orchestrator branch you specified and re-scanning there before making any edits.