<!-- orchestrate handoff
task: audit-config
branch: orch/agcli-audit/audit-config
agentId: bc-80ad2493-a4af-4e14-ba78-c5296094118e
runId: run-9ba11aa0-6557-4b43-92c2-c43fcc906927
resultStatus: error
finishedAt: 2026-05-27T12:18:20.795Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

Understood: I’ll audit the full `config` command surface, update `docs/commands/config.md`, add `tests/audit_config.rs`, and verify with `cargo check --all-targets` and `cargo test --no-run --test audit_config` while keeping changes limited to those two files. I’m starting by reading the required discovery file and relevant command/handler definitions.I’ve read the required discovery notes and am now mapping every `ConfigCommands` variant to `handle_config` and its underlying call paths so I can verify arguments, output shape, and on-chain relevance before editing docs/tests.