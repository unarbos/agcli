<!-- orchestrate handoff
task: parity-subnet-register
branch: orch/phase3-parity-suite/parity-subnet-register
agentId: bc-2abd733a-840b-4f41-be64-891275667ef6
runId: run-7a1272be-13cc-4690-b593-6061a6bc632d
resultStatus: error
finishedAt: 2026-05-28T17:30:09.374Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

Understood — I’ll implement the subnet registration parity category tests in the allowed files, wire them to the existing harness patterns, and then run the requested `cargo test --features e2e --test parity_subnet_register -- --nocapture` verification before handing off. I’m starting by inspecting current parity test and matrix structure to match conventions exactly.