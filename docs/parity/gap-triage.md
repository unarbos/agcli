## Executive summary
- Total matrix rows: **397**.
- Counts by `agcli_status`:
  - `COVERED_CLI_ONLY`: **302**
  - `COVERED_UNIQUE`: **26**
  - `GAP`: **65**
  - `N/A`: **4**
  - `COVERED_E2E`: **0** (this branch snapshot does not yet include merged Phase 3 row flips)

Top 5 highest-impact `GAP` rows (operator workflows blocked):
1. `btcli.sudo.stake-burn` (`SubtensorModule.add_stake_burn`) — missing write-path command for stake-burn economics changes.
2. `btcli.sudo.senate-vote` (`SubtensorModule.vote`) — missing senate vote submit path.
3. `btcli.sudo.proposals` — no senate proposal listing command.
4. `btcli.sudo.senate` — no senate membership/introspection command.
5. `btcli.stake.child.get` — no child-stake routing read helper for stake automation flows.

Top 5 highest-impact `COVERED_CLI_ONLY` drift items from Phase 3:
1. `btcli.wallet.associate-hotkey` — **confirmed divergence**: btcli no-op vs agcli fee-spending extrinsic.
2. `btcli.wallet.regen-coldkey` — **confirmed divergence**: btcli non-interactive `success=false` JSON while exiting 0 vs stricter agcli error semantics.
3. `btcli.weights.commit` — **high-risk pending drift diagnosis** (category `parity-weights` failed after retries).
4. `btcli.subnets.register` — **high-risk pending drift diagnosis** (category `parity-subnet-register` failed after retries).
5. `btcli.stake.move` — **high-risk pending drift diagnosis** (category `parity-stake-advanced` failed after retries).

Quoted Phase 3 evidence:
- From `.orchestrate/agcli-parity/handoffs/phase3-parity-suite.md`: "`associate-hotkey` behavior diverges (btcli no-op vs agcli fee-spending extrinsic on baseline localnet)."
- From `.orchestrate/agcli-parity/handoffs/phase3-parity-suite.md`: "`regen-coldkey` non-interactive drift: btcli produced `success=false` JSON while exiting 0; row left `COVERED_CLI_ONLY`."

## Gap rows by priority tier
Tier counts applied in this triage:
- **P0**: 1 row
- **P1**: 19 rows
- **P2**: 45 rows

### P0 (blocker)
| Matrix id | ref_extrinsic | Gap summary | Suggested agcli command surface | Difficulty | Suggested PR title | Handler pointer |
|---|---|---|---|---|---|---|
| `btcli.sudo.stake-burn` | `SubtensorModule.add_stake_burn` | btcli submits add_stake_burn for subnet economics; agcli has no named command for this write path. | `agcli admin stake-burn --netuid <u16> --amount <tao> --yes --output json` | `moderate` | `feat(admin): close GAP btcli.sudo.stake-burn` | `src/cli/admin_cmds.rs` |

### P1 (important)
| Matrix id | ref_extrinsic | Gap summary | Suggested agcli command surface | Difficulty | Suggested PR title | Handler pointer |
|---|---|---|---|---|---|---|
| `btcli.sudo.proposals` | `—` | btcli lists current senate proposals for governance ops; agcli has no dedicated proposal-list surface. | `agcli admin senate proposals --output json` | `moderate` | `feat(admin): close GAP btcli.sudo.proposals` | `src/cli/admin_cmds.rs` |
| `btcli.sudo.senate` | `—` | btcli lists senate members and status; agcli lacks a dedicated senate-members view. | `agcli admin senate list --output json` | `moderate` | `feat(admin): close GAP btcli.sudo.senate` | `src/cli/admin_cmds.rs` |
| `btcli.sudo.senate-vote` | `SubtensorModule.vote` | btcli submits a senate vote extrinsic; agcli cannot submit this governance vote directly. | `agcli admin senate vote --proposal-hash <0x...> --vote <yes|no> --yes --output json` | `moderate` | `feat(admin): close GAP btcli.sudo.senate-vote` | `src/cli/admin_cmds.rs` |
| `sdk.async_subtensor.compose_call` | `—` | SDK composes arbitrary pallet calls without submitting; agcli has no call-composition command for scripted pipelines. | `agcli utils compose-call --pallet <name> --call <name> --args-json <json>` | `large` | `feat(view): close GAP sdk.async_subtensor.compose_call` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.async_subtensor.get_admin_freeze_window` | `—` | SDK provides admin freeze window as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk admin-freeze-window --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_admin_freeze_window` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_root_claim_type` | `—` | SDK provides root claim type as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk root-claim-type --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_root_claim_type` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_root_claimable_all_rates` | `—` | SDK provides root claimable all rates as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk root-claimable-all-rates --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_root_claimable_all_rates` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_root_claimed` | `—` | SDK provides root claimed as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk root-claimed --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_root_claimed` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_vote_data` | `—` | SDK provides vote data as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk vote-data --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_vote_data` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.query_runtime_api` | `—` | SDK exposes generic runtime API queries; agcli has no equivalent programmable runtime-api command. | `agcli view runtime-api call --api <name> --method <name> --params-json <json>` | `large` | `feat(view): close GAP sdk.async_subtensor.query_runtime_api` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.async_subtensor.validate_extrinsic_params` | `—` | SDK validates nonce/era/tip parameter sets pre-submit; agcli lacks a standalone preflight validator. | `agcli utils validate-extrinsic-params --nonce <n> --era <mortal|immortal> --tip <tao>` | `moderate` | `feat(view): close GAP sdk.async_subtensor.validate_extrinsic_params` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.subtensor.compose_call` | `—` | SDK composes arbitrary pallet calls without submitting; agcli has no call-composition command for scripted pipelines. | `agcli utils compose-call --pallet <name> --call <name> --args-json <json>` | `large` | `feat(view): close GAP sdk.subtensor.compose_call` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.subtensor.get_admin_freeze_window` | `—` | SDK provides admin freeze window as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk admin-freeze-window --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_admin_freeze_window` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_root_claim_type` | `—` | SDK provides root claim type as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk root-claim-type --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_root_claim_type` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_root_claimable_all_rates` | `—` | SDK provides root claimable all rates as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk root-claimable-all-rates --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_root_claimable_all_rates` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_root_claimed` | `—` | SDK provides root claimed as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk root-claimed --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_root_claimed` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_vote_data` | `—` | SDK provides vote data as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk vote-data --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_vote_data` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.query_runtime_api` | `—` | SDK exposes generic runtime API queries; agcli has no equivalent programmable runtime-api command. | `agcli view runtime-api call --api <name> --method <name> --params-json <json>` | `large` | `feat(view): close GAP sdk.subtensor.query_runtime_api` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.subtensor.validate_extrinsic_params` | `—` | SDK validates nonce/era/tip parameter sets pre-submit; agcli lacks a standalone preflight validator. | `agcli utils validate-extrinsic-params --nonce <n> --era <mortal|immortal> --tip <tao>` | `moderate` | `feat(view): close GAP sdk.subtensor.validate_extrinsic_params` | new file needed (`src/cli/sdk_cmds.rs`) |

### P2 (nice-to-have)
| Matrix id | ref_extrinsic | Gap summary | Suggested agcli command surface | Difficulty | Suggested PR title | Handler pointer |
|---|---|---|---|---|---|---|
| `btcli.config.add-proxy` | `—` | btcli stores a named proxy endpoint in local config; agcli has no config-level proxy profile command. | `agcli config add-proxy --name <alias> --ss58 <addr>` | `trivial` | `feat(config): close GAP btcli.config.add-proxy` | `src/cli/system_cmds.rs` |
| `btcli.config.clear` | `—` | btcli can reset all persisted CLI config state; agcli has no equivalent full reset command. | `agcli config clear --yes` | `trivial` | `feat(config): close GAP btcli.config.clear` | `src/cli/system_cmds.rs` |
| `btcli.config.clear-proxies` | `—` | btcli can remove all saved proxy endpoints at once; agcli lacks a bulk clear-proxies action. | `agcli config clear-proxies --yes` | `trivial` | `feat(config): close GAP btcli.config.clear-proxies` | `src/cli/system_cmds.rs` |
| `btcli.config.proxies` | `—` | btcli lists configured proxy endpoints; agcli has no command to inspect saved proxy profile state. | `agcli config proxies --output json` | `trivial` | `feat(config): close GAP btcli.config.proxies` | `src/cli/system_cmds.rs` |
| `btcli.config.remove-proxy` | `—` | btcli deletes one saved proxy endpoint from local config; agcli cannot remove a named proxy profile. | `agcli config remove-proxy --name <alias>` | `trivial` | `feat(config): close GAP btcli.config.remove-proxy` | `src/cli/system_cmds.rs` |
| `btcli.config.update-proxy` | `—` | btcli mutates an existing proxy profile in local config; agcli has no update-proxy equivalent. | `agcli config update-proxy --name <alias> --ss58 <addr>` | `trivial` | `feat(config): close GAP btcli.config.update-proxy` | `src/cli/system_cmds.rs` |
| `btcli.stake.child.get` | `—` | btcli returns current child-hotkey stake routing/delegation state; agcli lacks a read helper for this child map. | `agcli stake child get --hotkey <ss58> --output json` | `moderate` | `feat(stake): close GAP btcli.stake.child.get` | `src/cli/stake_cmds.rs` |
| `btcli.wallet.regen-coldkeypub` | `—` | btcli can regenerate/import a coldkey public key file without private material; agcli has no equivalent recovery helper. | `agcli wallet regen-coldkeypub --public-key <hex|ss58> --output json` | `trivial` | `feat(wallet): close GAP btcli.wallet.regen-coldkeypub` | `src/cli/wallet_cmds.rs` |
| `btcli.wallet.regen-hotkeypub` | `—` | btcli can regenerate/import a hotkey public key file only; agcli lacks this public-key recovery command. | `agcli wallet regen-hotkeypub --public-key <hex|ss58> --output json` | `trivial` | `feat(wallet): close GAP btcli.wallet.regen-hotkeypub` | `src/cli/wallet_cmds.rs` |
| `sdk.async_subtensor.get_all_neuron_certificates` | `—` | SDK provides all neuron certificates as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk all-neuron-certificates --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_all_neuron_certificates` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_children_pending` | `—` | SDK provides children pending as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk children-pending --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_children_pending` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_coldkey_swap_announcement_delay` | `—` | SDK provides coldkey swap announcement delay as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-announcement-delay --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_coldkey_swap_announcement_delay` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_coldkey_swap_announcements` | `—` | SDK provides coldkey swap announcements as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-announcements --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_coldkey_swap_announcements` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_coldkey_swap_dispute` | `—` | SDK provides coldkey swap dispute as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-dispute --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_coldkey_swap_dispute` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_coldkey_swap_disputes` | `—` | SDK provides coldkey swap disputes as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-disputes --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_coldkey_swap_disputes` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_coldkey_swap_reannouncement_delay` | `—` | SDK provides coldkey swap reannouncement delay as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-reannouncement-delay --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_coldkey_swap_reannouncement_delay` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_extrinsic_fee` | `—` | SDK estimates extrinsic fees before submission; agcli has no standalone extrinsic fee estimator. | `agcli utils fee extrinsic --call-hex <0x...> --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_extrinsic_fee` | `src/cli/system_cmds.rs` |
| `sdk.async_subtensor.get_last_bonds_reset` | `—` | SDK provides last bonds reset as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk last-bonds-reset --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_last_bonds_reset` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_mev_shield_current_key` | `—` | SDK provides mev shield current key as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk mev-shield-current-key --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_mev_shield_current_key` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_mev_shield_next_key` | `—` | SDK provides mev shield next key as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk mev-shield-next-key --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_mev_shield_next_key` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_root_alpha_dividends_per_subnet` | `—` | SDK provides root alpha dividends per subnet as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk root-alpha-dividends-per-subnet --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_root_alpha_dividends_per_subnet` | `src/cli/view_cmds.rs` |
| `sdk.async_subtensor.get_transfer_fee` | `—` | SDK estimates transfer fees; agcli lacks a direct transfer-fee estimation command. | `agcli utils fee transfer --dest <ss58> --amount <tao> --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.get_transfer_fee` | `src/cli/system_cmds.rs` |
| `sdk.async_subtensor.last_drand_round` | `—` | SDK returns latest drand round index; agcli has no direct drand-round read helper. | `agcli drand current-round --output json` | `moderate` | `feat(view): close GAP sdk.async_subtensor.last_drand_round` | `src/cli/network_cmds.rs` |
| `sdk.async_subtensor.query_map` | `—` | SDK iterates arbitrary storage maps by prefix; agcli lacks a generic map query/scan command. | `agcli view storage query-map --module <name> --storage <name> --prefix-json <json>` | `large` | `feat(view): close GAP sdk.async_subtensor.query_map` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.async_subtensor.query_map_subtensor` | `—` | SDK iterates subtensor storage maps by prefix; agcli has no map-iteration query surface for subtensor keys. | `agcli view storage query-map-subtensor --storage <name> --prefix-json <json>` | `large` | `feat(view): close GAP sdk.async_subtensor.query_map_subtensor` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.async_subtensor.query_module` | `—` | SDK exposes generic module/storage queries; agcli cannot issue arbitrary module storage lookups. | `agcli view storage query-module --module <name> --storage <name> --key-json <json>` | `large` | `feat(view): close GAP sdk.async_subtensor.query_module` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.async_subtensor.query_subtensor` | `—` | SDK exposes generic subtensor storage queries; agcli lacks a generic subtensor storage query interface. | `agcli view storage query-subtensor --storage <name> --key-json <json>` | `large` | `feat(view): close GAP sdk.async_subtensor.query_subtensor` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.subtensor.get_all_neuron_certificates` | `—` | SDK provides all neuron certificates as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk all-neuron-certificates --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_all_neuron_certificates` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_children_pending` | `—` | SDK provides children pending as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk children-pending --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_children_pending` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_coldkey_swap_announcement_delay` | `—` | SDK provides coldkey swap announcement delay as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-announcement-delay --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_coldkey_swap_announcement_delay` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_coldkey_swap_announcements` | `—` | SDK provides coldkey swap announcements as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-announcements --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_coldkey_swap_announcements` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_coldkey_swap_dispute` | `—` | SDK provides coldkey swap dispute as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-dispute --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_coldkey_swap_dispute` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_coldkey_swap_disputes` | `—` | SDK provides coldkey swap disputes as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-disputes --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_coldkey_swap_disputes` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_coldkey_swap_reannouncement_delay` | `—` | SDK provides coldkey swap reannouncement delay as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk coldkey-swap-reannouncement-delay --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_coldkey_swap_reannouncement_delay` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_extrinsic_fee` | `—` | SDK estimates extrinsic fees before submission; agcli has no standalone extrinsic fee estimator. | `agcli utils fee extrinsic --call-hex <0x...> --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_extrinsic_fee` | `src/cli/system_cmds.rs` |
| `sdk.subtensor.get_last_bonds_reset` | `—` | SDK provides last bonds reset as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk last-bonds-reset --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_last_bonds_reset` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_mev_shield_current_key` | `—` | SDK provides mev shield current key as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk mev-shield-current-key --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_mev_shield_current_key` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_mev_shield_next_key` | `—` | SDK provides mev shield next key as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk mev-shield-next-key --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_mev_shield_next_key` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_root_alpha_dividends_per_subnet` | `—` | SDK provides root alpha dividends per subnet as a direct read helper; agcli has no dedicated command for that chain value. | `agcli view sdk root-alpha-dividends-per-subnet --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_root_alpha_dividends_per_subnet` | `src/cli/view_cmds.rs` |
| `sdk.subtensor.get_transfer_fee` | `—` | SDK estimates transfer fees; agcli lacks a direct transfer-fee estimation command. | `agcli utils fee transfer --dest <ss58> --amount <tao> --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.get_transfer_fee` | `src/cli/system_cmds.rs` |
| `sdk.subtensor.last_drand_round` | `—` | SDK returns latest drand round index; agcli has no direct drand-round read helper. | `agcli drand current-round --output json` | `moderate` | `feat(view): close GAP sdk.subtensor.last_drand_round` | `src/cli/network_cmds.rs` |
| `sdk.subtensor.query_map` | `—` | SDK iterates arbitrary storage maps by prefix; agcli lacks a generic map query/scan command. | `agcli view storage query-map --module <name> --storage <name> --prefix-json <json>` | `large` | `feat(view): close GAP sdk.subtensor.query_map` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.subtensor.query_map_subtensor` | `—` | SDK iterates subtensor storage maps by prefix; agcli has no map-iteration query surface for subtensor keys. | `agcli view storage query-map-subtensor --storage <name> --prefix-json <json>` | `large` | `feat(view): close GAP sdk.subtensor.query_map_subtensor` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.subtensor.query_module` | `—` | SDK exposes generic module/storage queries; agcli cannot issue arbitrary module storage lookups. | `agcli view storage query-module --module <name> --storage <name> --key-json <json>` | `large` | `feat(view): close GAP sdk.subtensor.query_module` | new file needed (`src/cli/sdk_cmds.rs`) |
| `sdk.subtensor.query_subtensor` | `—` | SDK exposes generic subtensor storage queries; agcli lacks a generic subtensor storage query interface. | `agcli view storage query-subtensor --storage <name> --key-json <json>` | `large` | `feat(view): close GAP sdk.subtensor.query_subtensor` | new file needed (`src/cli/sdk_cmds.rs`) |

## Behavior-drift rows from Phase 3
Only two rows were explicitly left `COVERED_CLI_ONLY` due confirmed divergence diagnostics in completed Phase 3 worker output.

| Matrix row | Phase 3 worker handoff | Divergence diagnosis | Suggested fix scope |
|---|---|---|---|
| `btcli.wallet.associate-hotkey` | `.orchestrate/phase3-parity-suite/handoffs/parity-wallet.md` | Wallet worker reported btcli no-op when already associated, but agcli still submits extrinsic and burns fee on baseline localnet. | Add pre-submit idempotency check in `wallet associate-hotkey` (query association first), and short-circuit without extrinsic when already linked; add localnet parity assertion on fee delta = 0 for no-op case. |
| `btcli.wallet.regen-coldkey` | `.orchestrate/phase3-parity-suite/handoffs/parity-wallet.md` | Non-interactive behavior drift: btcli emits `success=false` in JSON while process exits 0; agcli currently uses stricter error semantics. | Decide contract (match btcli loose semantics vs keep strict agcli semantics), then normalize exit code + JSON fields and pin behavior in parity tests for non-interactive regen failures. |

## Phase 3 categories that did not complete
The Phase 3 subplanner marked these categories as `error after retries` or `cancelled`: `parity-stake-advanced`, `parity-subnet-register`, `parity-subnet-hyperparams`, `parity-weights`, `parity-network-readonly`, `parity-root`, `parity-identity-commitment`.

| Category | Why it likely failed | Narrower respawn scope (rows + scaffold variant) |
|---|---|---|
| `parity-stake-advanced` | Failure handoff shows repeated `failureMode: unknown`; raw output indicates path/layout mismatch before meaningful test execution. | Respawn only `btcli.stake.move`, `btcli.stake.swap`, `btcli.stake.transfer`, plus SDK `move_stake`/`swap_stake`/`transfer_stake`; use **variant C** (multi-subnet) and split write rows into 2 workers (`move/swap` and `transfer/claim`). |
| `parity-subnet-register` | Failed quickly (`~10s`) with `failureMode: unknown`; raw output starts at discovery/path checks and never reaches command execution. | Respawn only registration rows (`btcli.subnets.register`, SDK `root_register`, `register`, `register_limit`), on **variant A** first; separate burned-register coverage to a second worker on **variant B** if commit-reveal gating is needed. |
| `parity-subnet-hyperparams` | Repeated unknown failure + raw output path mismatch while locating parity files. | Respawn with only `btcli.subnets.set-identity` + SDK `set_subnet_identity` + highest-risk hyperparam writes; run on **variant B** (commit-reveal on) to exercise hyperparam-sensitive paths. |
| `parity-weights` | Unknown failure after retries; raw output confirms expected parity paths were missing and branch alignment consumed attempts. | Respawn as two workers: (1) `btcli.weights.commit` + SDK `commit_weights`; (2) SDK `set_weights` paths; run on **variant B** (short reveal interval) and require explicit `parity_weights` target wiring before execution. |
| `parity-network-readonly` | Unknown failure after retries, stuck at repo layout mismatch (`tests/parity/` absent). | Respawn read-only rows only (no writes), split by module (`view network state` vs `fees/runtime API`); run on **variant A** and enforce preflight check that matrix + parity targets are present before tests. |
| `parity-root` | Cancelled after prolonged running state (`~83 min`) and repeated environment/bootstrap churn; raw output shows heavy setup before final verification. | Respawn into three micro-workers on **variant A**: (a) `root_register`, (b) root identity path, (c) root weights path; hard-cap each worker to <=3 rows and require reusable pre-warmed Docker chain fixture. |
| `parity-identity-commitment` | Unknown failure; raw output centered on missing parity scaffold and branch mismatch. | Respawn with only `btcli.wallet.set-identity` and SDK `set_commitment`/`set_reveal_commitment` rows on **variant A**; separate identity and commitment into two workers if either exceeds cap. |

Cross-cutting blocker called out by Phase 3 aggregator (`.orchestrate/agcli-parity/handoffs/phase3-parity-suite.md`): requested verifier commands such as `cargo test --features e2e --test parity_<category>` were missing for multiple workers. Ensure test target wiring lands before respawn.

## Recommended follow-up orchestrate plan
1. **Land env-setup agent first (single worker, prerequisite gate).**
   - Goal: pre-warm cloud image with Rust `>=1.95`, Docker daemon + pulled `ghcr.io/opentensor/subtensor-localnet:devnet-ready`, `uv`, project `.venv` with `bittensor` + `bittensor-cli`, and Cargo compatibility for edition2024 deps.
   - Suggested env-setup prompt:
     - `Prepare cloud agent parity environment: Rust stable >=1.95, Docker with working daemon and pre-pulled ghcr.io/opentensor/subtensor-localnet:devnet-ready, uv installed, project .venv with bittensor and bittensor-cli, and cargo-compatible toolchain for edition2024 dependencies.`

2. **Re-run the 7 incomplete Phase 3 categories, one worker per category, each with narrowed scope above.**
   - `parity-stake-advanced`
   - `parity-subnet-register`
   - `parity-subnet-hyperparams`
   - `parity-weights`
   - `parity-network-readonly`
   - `parity-root`
   - `parity-identity-commitment`
   - Gate: each worker must pass its concrete `cargo test --features e2e --test parity_<category>` target with localnet assertions.

3. **Land each P0 gap as its own PR (<=8 workers, `openPR=true`).**
   - Worker 1: `feat(admin): close GAP btcli.sudo.stake-burn`.
   - Include localnet write-path parity test + matrix flip for all shared-dispatchable rows.

4. **Land P1 gaps as second wave (batched by subsystem, still one-row-per-PR discipline).**
   - Wave A (governance): senate list/proposals/vote + vote-data/root-claim/admin-freeze helpers.
   - Wave B (SDK power surfaces): compose-call, query-runtime-api, validate-extrinsic-params.
   - Every PR must include localnet chain-state verification for write paths and explicit output-contract checks for read helpers.

5. **Re-run Phase 5 then Phase 6.**
   - Phase 5: refresh UX/fault-tolerance scorecard after divergence fixes.
   - Phase 6: wire parity-localnet CI job and regenerate parity report from updated matrix + Phase 3 reruns.
