<!-- orchestrate handoff
task: env-setup-followup
branch: orch/agcli-parity/env-setup-followup
agentId: bc-1b6e9353-a0d7-4286-ac5e-625f9058da2f
runId: run-49670d7a-de2c-40f1-8a46-ff6826a508cb
resultStatus: error
finishedAt: 2026-05-29T05:58:51.034Z
-->

> ⚠️ Run ended with `status=error`. No structured handoff produced — the content below is the worker's raw output up to the point of failure.

I’m going to do the environment-only bootstrap exactly as scoped: first I’ll read the required parity docs (`discovery.md` then `gap-triage.md`) and inspect the current git/branch state before running setup commands.I’ve read both required docs and will now execute the scoped bootstrap steps in order on a dedicated branch (`orch/agcli-parity/env-setup-followup`), then run the scaffold-A smoke test and only touch the allowed docs/handoff paths.