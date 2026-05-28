<!-- orchestrate handoff
task: parity-subnet-hyperparams
branch: orch/phase3-parity-suite/parity-subnet-hyperparams
agentId: bc-0f8294d1-36cb-4e68-a39e-572b7adbcd1e
runId: run-162b3f15-dba6-46c6-a7a0-a9903170ef57
resultStatus: error
finishedAt: 2026-05-28T17:42:16.316Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m implementing the subnet hyperparameter parity worker slice end-to-end: I’ll inspect existing parity harness/tests and matrix rows first, then add `tests/parity/subnet_hyperparams.rs` plus `tests/parity/mod.rs` wiring, update `docs/parity/matrix.json`, and run the required `cargo test --features e2e --test parity_subnet_hyperparams -- --nocapture` verification.I hit a path mismatch, so I’m locating where parity tests and the matrix file actually live in this repo before editing.