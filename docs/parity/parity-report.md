# agcli parity report (phase 5+6 refresh)

Generated from `docs/parity/matrix.json` (post-follow-up refresh).

## Matrix snapshot

- Total rows: **397**
- Status distribution:
  - `COVERED_E2E`: **2**
  - `COVERED_CLI_ONLY`: **306**
  - `GAP`: **59**
  - `COVERED_UNIQUE`: **26**
  - `N/A`: **4**
- Rows with `parity_test` populated: **2**

### Source breakdown

| Source | Total | COVERED_E2E | COVERED_CLI_ONLY | GAP | COVERED_UNIQUE | N/A |
|---|---:|---:|---:|---:|---:|---:|
| btcli | 93 | 0 | 78 | 13 | 0 | 2 |
| sdk | 278 | 2 | 228 | 46 | 0 | 2 |
| agcli-unique | 26 | 0 | 0 | 0 | 26 | 0 |

### Follow-up delta vs prior triage snapshot

- `COVERED_E2E`: **0 → 2**
- `COVERED_CLI_ONLY`: **302 → 306**
- `GAP`: **65 → 59**
- `parity_test` populated: **0 → 2**

## DoD checklist (honest state)

- [ ] **100% btcli mapped + localnet verified**  
  Current: `0 / 93` btcli rows are `COVERED_E2E`.
- [ ] **100% SDK extrinsic helpers mapped**  
  Current: `230 / 278` sdk rows are non-`GAP`; only `2 / 278` are `COVERED_E2E`.
- [ ] **Zero `COVERED_CLI_ONLY` for write paths**  
  Current: `158` strict write rows (non-empty `ref_extrinsic`) remain `COVERED_CLI_ONLY`.
- [ ] **agcli UX >= btcli**  
  No refreshed UX scorecard artifact was present in this branch snapshot.
- [ ] **All e2e suites pass**  
  Not executed in this step-5/6 refresh task.

## Notes

- This refresh applied the documented follow-up row changes from:
  - `orch/agcli-parity/phase3-rerun-identity-commitment`
  - `orch/agcli-parity/fix-p1-sdk-power-surface`
- The parity-localnet workflow now discovers parity targets automatically and fails fast if none are discoverable, preventing false-green CI runs.
