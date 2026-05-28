# agcli parity matrix

Generated from upstream inventories: btcli=93, sdk=278; agcli-unique rows=26.

## Status summary

| Status | Count |
|---|---:|
| COVERED_CLI_ONLY | 302 |
| COVERED_UNIQUE | 26 |
| GAP | 65 |
| COVERED_E2E | 0 |
| N/A | 4 |

## Source summary

| Source | Count |
|---|---:|
| btcli | 93 |
| sdk | 278 |
| agcli-unique | 26 |

## Phase 3 candidates (COVERED_CLI_ONLY + COVERED_UNIQUE)

Rows: 328

| id | source | ref_extrinsic | agcli_status | agcli_invocation |
|---|---|---|---|---|
| agcli.audit.audit | agcli-unique |  | COVERED_UNIQUE | agcli audit --address <ss58> --output json |
| agcli.batch.file | agcli-unique |  | COVERED_UNIQUE | agcli batch --file ops.json --yes |
| agcli.block.info | agcli-unique |  | COVERED_UNIQUE | agcli block info --number 12345 --output json |
| agcli.block.latest | agcli-unique |  | COVERED_UNIQUE | agcli block latest --output json |
| agcli.block.range | agcli-unique |  | COVERED_UNIQUE | agcli block range --from 1000 --to 1100 --output json |
| agcli.diff.network | agcli-unique |  | COVERED_UNIQUE | agcli diff network --block1 1000 --block2 2000 --output json |
| agcli.diff.portfolio | agcli-unique |  | COVERED_UNIQUE | agcli diff portfolio --block1 1000 --block2 2000 --output json |
| agcli.diff.subnet | agcli-unique |  | COVERED_UNIQUE | agcli diff subnet --netuid 1 --block1 1000 --block2 2000 --output json |
| agcli.doctor.doctor | agcli-unique |  | COVERED_UNIQUE | agcli doctor --output json |
| agcli.explain.topic | agcli-unique |  | COVERED_UNIQUE | agcli explain --topic tempo |
| agcli.global.at-block | agcli-unique |  | COVERED_UNIQUE | agcli subnet list --at-block 3500000 --network archive --output json |
| agcli.subnet.cache-diff | agcli-unique |  | COVERED_UNIQUE | agcli subnet cache-diff --netuid 1 --from a --to b |
| agcli.subnet.emissions | agcli-unique |  | COVERED_UNIQUE | agcli subnet emissions --netuid 1 --output json |
| agcli.subnet.health | agcli-unique |  | COVERED_UNIQUE | agcli subnet health --netuid 1 --output json |
| agcli.subnet.monitor | agcli-unique |  | COVERED_UNIQUE | agcli subnet monitor --netuid 1 --json |
| agcli.subnet.probe | agcli-unique |  | COVERED_UNIQUE | agcli subnet probe --netuid 1 |
| agcli.subscribe.blocks | agcli-unique |  | COVERED_UNIQUE | agcli subscribe blocks |
| agcli.subscribe.events | agcli-unique |  | COVERED_UNIQUE | agcli subscribe events --filter staking --netuid 1 |
| agcli.utils.convert | agcli-unique |  | COVERED_UNIQUE | agcli utils convert --tao 10 --netuid 1 |
| agcli.utils.latency | agcli-unique |  | COVERED_UNIQUE | agcli utils latency |
| agcli.view.dynamic | agcli-unique |  | COVERED_UNIQUE | agcli view dynamic --output json |
| agcli.view.history | agcli-unique |  | COVERED_UNIQUE | agcli view history --address <ss58> --output json |
| agcli.view.network | agcli-unique |  | COVERED_UNIQUE | agcli view network --output json |
| agcli.view.portfolio | agcli-unique |  | COVERED_UNIQUE | agcli view portfolio --output json |
| agcli.view.validators | agcli-unique |  | COVERED_UNIQUE | agcli view validators --output json |
| agcli.weights.commit-reveal | agcli-unique |  | COVERED_UNIQUE | agcli weights commit-reveal --netuid 1 --weights "0:100" --wait --yes |
| btcli.config.get | btcli |  | COVERED_CLI_ONLY | agcli config show |
| btcli.config.set | btcli |  | COVERED_CLI_ONLY | agcli config set --key <key> --value <value> |
| btcli.crowd.contribute | btcli | Crowdloan.contribute | COVERED_CLI_ONLY | agcli crowdloan contribute --id 0 --amount 5 --yes |
| btcli.crowd.contributors | btcli |  | COVERED_CLI_ONLY | agcli crowdloan contributors --id 0 --output json |
| btcli.crowd.create | btcli | Crowdloan.create \| SubtensorModule.register_leased_network | COVERED_CLI_ONLY | agcli crowdloan create --deposit 10 --min-contribution 1 --cap 100 --end 1000 --yes |
| btcli.crowd.dissolve | btcli | Crowdloan.dissolve | COVERED_CLI_ONLY | agcli crowdloan dissolve --id 0 --yes |
| btcli.crowd.finalize | btcli | Crowdloan.finalize | COVERED_CLI_ONLY | agcli crowdloan finalize --id 0 --yes |
| btcli.crowd.info | btcli |  | COVERED_CLI_ONLY | agcli crowdloan info --id 0 --output json |
| btcli.crowd.list | btcli |  | COVERED_CLI_ONLY | agcli crowdloan list --output json |
| btcli.crowd.refund | btcli | Crowdloan.refund | COVERED_CLI_ONLY | agcli crowdloan refund --id 0 --yes |
| btcli.crowd.update | btcli | Crowdloan.update_cap \| Crowdloan.update_end \| Crowdloan.update_min_contribution | COVERED_CLI_ONLY | agcli crowdloan update-cap --id 0 --cap 200 --yes |
| btcli.crowd.withdraw | btcli | Crowdloan.withdraw | COVERED_CLI_ONLY | agcli crowdloan withdraw --id 0 --yes |
| btcli.liquidity.add | btcli | Swap.add_liquidity | COVERED_CLI_ONLY | agcli liquidity add --netuid 1 --price-low 1 --price-high 2 --amount 1000000000 --yes |
| btcli.liquidity.list | btcli |  | COVERED_CLI_ONLY | agcli subnet liquidity --netuid 1 |
| btcli.liquidity.modify | btcli | Swap.modify_position | COVERED_CLI_ONLY | agcli liquidity modify --netuid 1 --position-id 1 --delta 1000 --yes |
| btcli.liquidity.remove | btcli | Swap.remove_liquidity | COVERED_CLI_ONLY | agcli liquidity remove --netuid 1 --position-id 1 --yes |
| btcli.proxy.add | btcli | Proxy.add_proxy | COVERED_CLI_ONLY | agcli proxy add --delegate <ss58> --proxy-type any --delay 0 --yes |
| btcli.proxy.create | btcli | Proxy.create_pure | COVERED_CLI_ONLY | agcli proxy create-pure --proxy-type any --delay 0 --yes |
| btcli.proxy.execute | btcli | Proxy.proxy_announced | COVERED_CLI_ONLY | agcli proxy proxy-announced --delegate <ss58> --real <ss58> --pallet SubtensorModule --call transfer_stake --yes |
| btcli.proxy.kill | btcli | Proxy.kill_pure | COVERED_CLI_ONLY | agcli proxy kill-pure --spawner <ss58> --proxy-type any --index 0 --height 0 --ext-index 0 --yes |
| btcli.proxy.remove | btcli | Proxy.remove_proxies \| Proxy.remove_proxy | COVERED_CLI_ONLY | agcli proxy remove --delegate <ss58> --proxy-type any --delay 0 --yes |
| btcli.stake.add | btcli | SubtensorModule.add_stake \| SubtensorModule.add_stake_limit | COVERED_CLI_ONLY | agcli stake add --amount 10 --netuid 1 --yes |
| btcli.stake.auto | btcli |  | COVERED_CLI_ONLY | agcli stake show-auto --address <ss58> |
| btcli.stake.child.revoke | btcli | SubtensorModule.set_children | COVERED_CLI_ONLY | agcli stake set-children --netuid 1 --children "" --yes |
| btcli.stake.child.set | btcli | SubtensorModule.set_children | COVERED_CLI_ONLY | agcli stake set-children --netuid 1 --children "1:<hotkey>" --yes |
| btcli.stake.child.take | btcli | SubtensorModule.set_childkey_take | COVERED_CLI_ONLY | agcli stake childkey-take --netuid 1 --take 0.1 --yes |
| btcli.stake.list | btcli |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| btcli.stake.move | btcli | SubtensorModule.move_stake | COVERED_CLI_ONLY | agcli stake move --amount 1 --from 1 --to 2 --yes |
| btcli.stake.process-claim | btcli | SubtensorModule.claim_root | COVERED_CLI_ONLY | agcli stake process-claim --netuid 1 --hotkey-address <ss58> --yes |
| btcli.stake.remove | btcli | SubtensorModule.remove_stake \| SubtensorModule.remove_stake_limit \| SubtensorModule.unstake_all_alpha | COVERED_CLI_ONLY | agcli stake remove --amount 1 --netuid 1 --yes |
| btcli.stake.set-auto | btcli | SubtensorModule.set_coldkey_auto_stake_hotkey | COVERED_CLI_ONLY | agcli stake set-auto --netuid 1 --hotkey-address <ss58> --yes |
| btcli.stake.set-claim | btcli | SubtensorModule.set_root_claim_type | COVERED_CLI_ONLY | agcli stake set-claim --netuid 1 --yes |
| btcli.stake.swap | btcli | SubtensorModule.swap_stake \| SubtensorModule.swap_stake_limit | COVERED_CLI_ONLY | agcli stake swap --amount 1 --from 1 --to 2 --yes |
| btcli.stake.transfer | btcli | SubtensorModule.transfer_stake | COVERED_CLI_ONLY | agcli stake transfer-stake --netuid 1 --amount 1 --dest-hotkey <ss58> --yes |
| btcli.stake.wizard | btcli | SubtensorModule.move_stake \| SubtensorModule.swap_stake \| SubtensorModule.swap_stake_limit \| SubtensorModule.transfer_stake | COVERED_CLI_ONLY | agcli stake wizard --netuid 1 --amount 1 --yes |
| btcli.subnets.burn-cost | btcli |  | COVERED_CLI_ONLY | agcli subnet create-cost |
| btcli.subnets.check-start | btcli |  | COVERED_CLI_ONLY | agcli subnet check-start --netuid 1 |
| btcli.subnets.create | btcli | SubtensorModule.register_network \| SubtensorModule.register_network_with_identity | COVERED_CLI_ONLY | agcli subnet register --yes |
| btcli.subnets.get-identity | btcli |  | COVERED_CLI_ONLY | agcli subnet show --netuid 1 --output json |
| btcli.subnets.hyperparameters | btcli |  | COVERED_CLI_ONLY | agcli subnet hyperparams --netuid 1 --output json |
| btcli.subnets.list | btcli |  | COVERED_CLI_ONLY | agcli subnet list --output json |
| btcli.subnets.mechanisms.count | btcli |  | COVERED_CLI_ONLY | agcli subnet mechanism-count --netuid 1 |
| btcli.subnets.mechanisms.emissions | btcli |  | COVERED_CLI_ONLY | agcli subnet emission-split --netuid 1 |
| btcli.subnets.mechanisms.set | btcli | AdminUtils.sudo_set_mechanism_count | COVERED_CLI_ONLY | agcli subnet set-mechanism-count --netuid 1 --count 2 --yes |
| btcli.subnets.mechanisms.split-emissions | btcli | AdminUtils.sudo_set_mechanism_emission_split | COVERED_CLI_ONLY | agcli subnet set-emission-split --netuid 1 --weights "50,50" --yes |
| btcli.subnets.price | btcli |  | COVERED_CLI_ONLY | agcli subnet show --netuid 1 --output json |
| btcli.subnets.register | btcli | SubtensorModule.root_register | COVERED_CLI_ONLY | agcli subnet register-neuron --netuid 1 --yes |
| btcli.subnets.set-identity | btcli | SubtensorModule.set_subnet_identity | COVERED_CLI_ONLY | agcli identity set-subnet --netuid 1 --name demo --yes |
| btcli.subnets.set-symbol | btcli | SubtensorModule.update_symbol | COVERED_CLI_ONLY | agcli subnet set-symbol --netuid 1 --symbol TEST --yes |
| btcli.subnets.show | btcli |  | COVERED_CLI_ONLY | agcli subnet show --netuid 1 --output json |
| btcli.subnets.start | btcli | SubtensorModule.start_call | COVERED_CLI_ONLY | agcli subnet start --netuid 1 --yes |
| btcli.sudo.get | btcli |  | COVERED_CLI_ONLY | agcli admin list --output json |
| btcli.sudo.get-take | btcli |  | COVERED_CLI_ONLY | agcli delegate show --hotkey-address <ss58> --output json |
| btcli.sudo.set | btcli | SubtensorModule.*) \| Sudo.sudo | COVERED_CLI_ONLY | agcli admin raw --call <admin_call> --args "[...]" --yes |
| btcli.sudo.set-take | btcli | SubtensorModule.decrease_take \| SubtensorModule.increase_take | COVERED_CLI_ONLY | agcli delegate increase-take --take 0.18 --yes |
| btcli.sudo.trim | btcli | AdminUtils.sudo_trim_to_max_allowed_uids | COVERED_CLI_ONLY | agcli subnet trim --netuid 1 --max-uids 64 --yes |
| btcli.utils.convert | btcli |  | COVERED_CLI_ONLY | agcli utils convert --amount 1.0 |
| btcli.utils.latency | btcli |  | COVERED_CLI_ONLY | agcli utils latency |
| btcli.view.dashboard | btcli |  | COVERED_CLI_ONLY | agcli view network --output json |
| btcli.wallet.associate-hotkey | btcli | SubtensorModule.try_associate_hotkey | COVERED_CLI_ONLY | agcli wallet associate-hotkey --hotkey-address <ss58> --yes |
| btcli.wallet.balance | btcli |  | COVERED_CLI_ONLY | agcli balance --address <ss58> --output json |
| btcli.wallet.create | btcli |  | COVERED_CLI_ONLY | agcli wallet create --name mywallet --yes |
| btcli.wallet.get-identity | btcli |  | COVERED_CLI_ONLY | agcli identity show --address <ss58> --output json |
| btcli.wallet.list | btcli |  | COVERED_CLI_ONLY | agcli wallet list --output json |
| btcli.wallet.new-coldkey | btcli |  | COVERED_CLI_ONLY | agcli wallet create --name mywallet --yes |
| btcli.wallet.new-hotkey | btcli |  | COVERED_CLI_ONLY | agcli wallet new-hotkey --name mywallet --yes |
| btcli.wallet.overview | btcli |  | COVERED_CLI_ONLY | agcli view portfolio --address <ss58> --output json |
| btcli.wallet.regen-coldkey | btcli |  | COVERED_CLI_ONLY | agcli wallet regen-coldkey --yes |
| btcli.wallet.regen-hotkey | btcli |  | COVERED_CLI_ONLY | agcli wallet regen-hotkey --name mywallet --yes |
| btcli.wallet.set-identity | btcli | SubtensorModule.set_identity | COVERED_CLI_ONLY | agcli identity set --name operator --yes |
| btcli.wallet.sign | btcli |  | COVERED_CLI_ONLY | agcli wallet sign --message "hello" |
| btcli.wallet.swap-check | btcli |  | COVERED_CLI_ONLY | agcli wallet check-swap --output json |
| btcli.wallet.swap-coldkey | btcli | SubtensorModule.announce_coldkey_swap \| SubtensorModule.clear_coldkey_swap_announcement \| SubtensorModule.dispute_coldkey_swap \| SubtensorModule.swap_coldkey_announced | COVERED_CLI_ONLY | agcli swap coldkey --new-coldkey <ss58> --yes |
| btcli.wallet.swap-hotkey | btcli | SubtensorModule.swap_hotkey | COVERED_CLI_ONLY | agcli swap hotkey --new-hotkey <ss58> --yes |
| btcli.wallet.transfer | btcli | Balances.transfer_allow_death \| Balances.transfer_keep_alive | COVERED_CLI_ONLY | agcli transfer --dest <ss58> --amount 1 --yes |
| btcli.wallet.verify | btcli |  | COVERED_CLI_ONLY | agcli wallet verify --message "hello" --signature 0x... |
| btcli.weights.commit | btcli | SubtensorModule.commit_weights | COVERED_CLI_ONLY | agcli weights commit --netuid 1 --weights "0:100" --yes |
| btcli.weights.reveal | btcli | SubtensorModule.reveal_weights | COVERED_CLI_ONLY | agcli weights reveal --netuid 1 --weights "0:100" --salt s --yes |
| sdk.async_subtensor.add_liquidity | sdk | Swap.add_liquidity | COVERED_CLI_ONLY | agcli liquidity add --netuid 1 --price-low 1 --price-high 2 --amount 1000000000 --yes |
| sdk.async_subtensor.add_proxy | sdk | Proxy.add_proxy | COVERED_CLI_ONLY | agcli proxy add --delegate <ss58> --proxy-type any --delay 0 --yes |
| sdk.async_subtensor.add_stake | sdk | SubtensorModule.add_stake \| SubtensorModule.add_stake_limit | COVERED_CLI_ONLY | agcli stake add --amount 10 --netuid 1 --yes |
| sdk.async_subtensor.add_stake_burn | sdk | SubtensorModule.add_stake_burn | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.async_subtensor.add_stake_multiple | sdk | SubtensorModule.add_stake \| SubtensorModule.add_stake_limit | COVERED_CLI_ONLY | agcli stake add --amount 10 --netuid 1 --yes |
| sdk.async_subtensor.announce_coldkey_swap | sdk | SubtensorModule.announce_coldkey_swap | COVERED_CLI_ONLY | agcli swap coldkey --new-coldkey <ss58> --yes |
| sdk.async_subtensor.announce_proxy | sdk | Proxy.announce | COVERED_CLI_ONLY | agcli proxy announce --real <ss58> --call-hex 0x00 --yes |
| sdk.async_subtensor.bonds | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --full --output json |
| sdk.async_subtensor.burned_register | sdk | SubtensorModule.burned_register \| SubtensorModule.root_register | COVERED_CLI_ONLY | agcli subnet register-neuron --netuid 1 --yes |
| sdk.async_subtensor.claim_root | sdk | SubtensorModule.claim_root | COVERED_CLI_ONLY | agcli stake claim-root --netuid 1 --yes |
| sdk.async_subtensor.clear_coldkey_swap_announcement | sdk | SubtensorModule.clear_coldkey_swap_announcement | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.async_subtensor.commit_weights | sdk | SubtensorModule.commit_mechanism_weights | COVERED_CLI_ONLY | agcli weights commit-mechanism --netuid 1 --mechanism-id 0 --hash 0x... --yes |
| sdk.async_subtensor.contribute_crowdloan | sdk | Crowdloan.contribute | COVERED_CLI_ONLY | agcli crowdloan contribute --id 0 --amount 5 --yes |
| sdk.async_subtensor.create_crowdloan | sdk | Crowdloan.create | COVERED_CLI_ONLY | agcli crowdloan create --deposit 10 --min-contribution 1 --cap 100 --end 1000 --yes |
| sdk.async_subtensor.create_pure_proxy | sdk | Proxy.create_pure | COVERED_CLI_ONLY | agcli proxy create-pure --proxy-type any --delay 0 --yes |
| sdk.async_subtensor.dispute_coldkey_swap | sdk | SubtensorModule.dispute_coldkey_swap | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.async_subtensor.dissolve_crowdloan | sdk | Crowdloan.dissolve | COVERED_CLI_ONLY | agcli crowdloan dissolve --id 0 --yes |
| sdk.async_subtensor.does_hotkey_exist | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.async_subtensor.finalize_crowdloan | sdk | Crowdloan.finalize | COVERED_CLI_ONLY | agcli crowdloan finalize --id 0 --yes |
| sdk.async_subtensor.get_all_commitments | sdk |  | COVERED_CLI_ONLY | agcli commitment list --netuid 1 --output json |
| sdk.async_subtensor.get_all_ema_tao_inflow | sdk |  | COVERED_CLI_ONLY | agcli view dynamic --output json |
| sdk.async_subtensor.get_all_revealed_commitments | sdk |  | COVERED_CLI_ONLY | agcli subnet commits --netuid 1 --output json |
| sdk.async_subtensor.get_all_subnets_info | sdk |  | COVERED_CLI_ONLY | agcli subnet list --output json |
| sdk.async_subtensor.get_all_subnets_netuid | sdk |  | COVERED_CLI_ONLY | agcli subnet list --output json |
| sdk.async_subtensor.get_auto_stakes | sdk |  | COVERED_CLI_ONLY | agcli stake show-auto --address <ss58> --output json |
| sdk.async_subtensor.get_balance | sdk |  | COVERED_CLI_ONLY | agcli balance --address <ss58> --output json |
| sdk.async_subtensor.get_children | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.async_subtensor.get_coldkey_swap_announcement | sdk |  | COVERED_CLI_ONLY | agcli wallet check-swap --output json |
| sdk.async_subtensor.get_commitment_metadata | sdk |  | COVERED_CLI_ONLY | agcli commitment get --netuid 1 --hotkey-address <ss58> --output json |
| sdk.async_subtensor.get_crowdloan_by_id | sdk |  | COVERED_CLI_ONLY | agcli crowdloan info --id 0 --output json |
| sdk.async_subtensor.get_crowdloan_contributions | sdk |  | COVERED_CLI_ONLY | agcli crowdloan contributors --id 0 --output json |
| sdk.async_subtensor.get_crowdloan_next_id | sdk |  | COVERED_CLI_ONLY | agcli crowdloan list --output json |
| sdk.async_subtensor.get_crowdloans | sdk |  | COVERED_CLI_ONLY | agcli crowdloan list --output json |
| sdk.async_subtensor.get_delegate_by_hotkey | sdk |  | COVERED_CLI_ONLY | agcli delegate show --hotkey-address <ss58> --output json |
| sdk.async_subtensor.get_delegate_identities | sdk |  | COVERED_CLI_ONLY | agcli delegate list --output json |
| sdk.async_subtensor.get_delegated | sdk |  | COVERED_CLI_ONLY | agcli delegate show --hotkey-address <ss58> --output json |
| sdk.async_subtensor.get_delegates | sdk |  | COVERED_CLI_ONLY | agcli delegate list --output json |
| sdk.async_subtensor.get_ema_tao_inflow | sdk |  | COVERED_CLI_ONLY | agcli view dynamic --output json |
| sdk.async_subtensor.get_hotkey_owner | sdk |  | COVERED_CLI_ONLY | agcli view account --address <ss58> --output json |
| sdk.async_subtensor.get_hyperparameter | sdk |  | COVERED_CLI_ONLY | agcli subnet hyperparams --netuid 1 --output json |
| sdk.async_subtensor.get_liquidity_list | sdk |  | COVERED_CLI_ONLY | agcli subnet liquidity --netuid 1 --output json |
| sdk.async_subtensor.get_minimum_required_stake | sdk |  | COVERED_CLI_ONLY | agcli subnet hyperparams --netuid 1 --output json |
| sdk.async_subtensor.get_netuids_for_hotkey | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.async_subtensor.get_neuron_for_pubkey_and_subnet | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.async_subtensor.get_owned_hotkeys | sdk |  | COVERED_CLI_ONLY | agcli wallet show --all --output json |
| sdk.async_subtensor.get_parents | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.async_subtensor.get_proxies | sdk |  | COVERED_CLI_ONLY | agcli proxy list --address <ss58> --output json |
| sdk.async_subtensor.get_proxies_for_real_account | sdk |  | COVERED_CLI_ONLY | agcli proxy list --address <ss58> --output json |
| sdk.async_subtensor.get_proxy_announcement | sdk |  | COVERED_CLI_ONLY | agcli proxy list-announcements --address <ss58> --output json |
| sdk.async_subtensor.get_proxy_announcements | sdk |  | COVERED_CLI_ONLY | agcli proxy list-announcements --address <ss58> --output json |
| sdk.async_subtensor.get_stake | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.async_subtensor.get_stake_for_coldkey_and_hotkey | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.async_subtensor.get_stake_info_for_coldkey | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.async_subtensor.get_stake_info_for_coldkeys | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.async_subtensor.get_stake_weight | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.async_subtensor.get_staking_hotkeys | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.async_subtensor.get_subnet_burn_cost | sdk |  | COVERED_CLI_ONLY | agcli subnet create-cost |
| sdk.async_subtensor.get_subnet_hyperparameters | sdk |  | COVERED_CLI_ONLY | agcli subnet hyperparams --netuid 1 --output json |
| sdk.async_subtensor.get_subnet_info | sdk |  | COVERED_CLI_ONLY | agcli subnet show --netuid 1 --output json |
| sdk.async_subtensor.get_subnet_prices | sdk |  | COVERED_CLI_ONLY | agcli subnet liquidity --netuid 1 --output json |
| sdk.async_subtensor.get_timelocked_weight_commits | sdk |  | COVERED_CLI_ONLY | agcli weights status --netuid 1 --output json |
| sdk.async_subtensor.get_total_subnets | sdk |  | COVERED_CLI_ONLY | agcli subnet list --output json |
| sdk.async_subtensor.get_uid_for_hotkey_on_subnet | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.async_subtensor.kill_pure_proxy | sdk | Proxy.kill_pure | COVERED_CLI_ONLY | agcli proxy kill-pure --spawner <ss58> --proxy-type any --index 0 --height 0 --ext-index 0 --yes |
| sdk.async_subtensor.mev_submit_encrypted | sdk | MevShield.submit_encrypted | COVERED_CLI_ONLY | agcli --mev batch --file calls.json --yes |
| sdk.async_subtensor.modify_liquidity | sdk | Swap.modify_position | COVERED_CLI_ONLY | agcli liquidity modify --netuid 1 --position-id 1 --delta 1000 --yes |
| sdk.async_subtensor.move_stake | sdk | SubtensorModule.move_stake | COVERED_CLI_ONLY | agcli stake move --amount 1 --from 1 --to 2 --yes |
| sdk.async_subtensor.neuron_for_uid | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --uid 0 --output json |
| sdk.async_subtensor.neurons | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.async_subtensor.neurons_lite | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.async_subtensor.poke_deposit | sdk | Proxy.poke_deposit | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.async_subtensor.proxy | sdk | Proxy.proxy | COVERED_CLI_ONLY | agcli --proxy <delegate_ss58> transfer --dest <ss58> --amount 1 --yes |
| sdk.async_subtensor.proxy_announced | sdk | Proxy.proxy_announced | COVERED_CLI_ONLY | agcli proxy proxy-announced --delegate <ss58> --real <ss58> --pallet SubtensorModule --call transfer_stake --yes |
| sdk.async_subtensor.query_identity | sdk |  | COVERED_CLI_ONLY | agcli identity show --address <ss58> --output json |
| sdk.async_subtensor.refund_crowdloan | sdk | Crowdloan.refund | COVERED_CLI_ONLY | agcli crowdloan refund --id 0 --yes |
| sdk.async_subtensor.register | sdk | SubtensorModule.register_limit \| SubtensorModule.root_register | COVERED_CLI_ONLY | agcli subnet register-neuron --netuid 1 --yes |
| sdk.async_subtensor.register_limit | sdk | SubtensorModule.register_limit | COVERED_CLI_ONLY | agcli subnet register-neuron --netuid 1 --yes |
| sdk.async_subtensor.register_subnet | sdk | SubtensorModule.register_network | COVERED_CLI_ONLY | agcli subnet register --yes |
| sdk.async_subtensor.reject_proxy_announcement | sdk | Proxy.reject_announcement | COVERED_CLI_ONLY | agcli proxy reject-announcement --delegate <ss58> --call-hash 0x... --yes |
| sdk.async_subtensor.remove_liquidity | sdk | Swap.remove_liquidity | COVERED_CLI_ONLY | agcli liquidity remove --netuid 1 --position-id 1 --yes |
| sdk.async_subtensor.remove_proxies | sdk | Proxy.remove_proxies | COVERED_CLI_ONLY | agcli proxy remove-all --yes |
| sdk.async_subtensor.remove_proxy | sdk | Proxy.remove_proxy | COVERED_CLI_ONLY | agcli proxy remove --delegate <ss58> --proxy-type any --delay 0 --yes |
| sdk.async_subtensor.remove_proxy_announcement | sdk | Proxy.remove_announcement | COVERED_CLI_ONLY | agcli proxy remove-announcement --real <ss58> --call-hash 0x... --yes |
| sdk.async_subtensor.reveal_weights | sdk | SubtensorModule.reveal_mechanism_weights | COVERED_CLI_ONLY | agcli weights reveal-mechanism --netuid 1 --mechanism-id 0 --weights "0:100" --salt s --yes |
| sdk.async_subtensor.root_register | sdk | SubtensorModule.root_register | COVERED_CLI_ONLY | agcli root register --netuid 1 --yes |
| sdk.async_subtensor.root_set_pending_childkey_cooldown | sdk | SubtensorModule.set_pending_childkey_cooldown \| Sudo.sudo | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.async_subtensor.set_auto_stake | sdk | SubtensorModule.set_coldkey_auto_stake_hotkey | COVERED_CLI_ONLY | agcli stake set-auto --netuid 1 --hotkey-address <ss58> --yes |
| sdk.async_subtensor.set_children | sdk | SubtensorModule.set_children | COVERED_CLI_ONLY | agcli stake set-children --netuid 1 --children "1:<hotkey>" --yes |
| sdk.async_subtensor.set_commitment | sdk | Commitments.set_commitment | COVERED_CLI_ONLY | agcli commitment set --netuid 1 --data "endpoint:https://example" --yes |
| sdk.async_subtensor.set_delegate_take | sdk | SubtensorModule.decrease_take \| SubtensorModule.increase_take | COVERED_CLI_ONLY | agcli delegate decrease-take --take 0.05 --yes |
| sdk.async_subtensor.set_reveal_commitment | sdk | Commitments.set_commitment | COVERED_CLI_ONLY | agcli commitment set --netuid 1 --data "endpoint:https://example" --yes |
| sdk.async_subtensor.set_root_claim_type | sdk | SubtensorModule.set_root_claim_type | COVERED_CLI_ONLY | agcli stake set-claim --netuid 1 --yes |
| sdk.async_subtensor.set_subnet_identity | sdk | SubtensorModule.set_subnet_identity | COVERED_CLI_ONLY | agcli identity set-subnet --netuid 1 --name demo --yes |
| sdk.async_subtensor.set_weights | sdk | SubtensorModule.commit_timelocked_mechanism_weights \| SubtensorModule.set_mechanism_weights | COVERED_CLI_ONLY | agcli weights commit-timelocked --netuid 1 --weights "0:100" --round 1 --yes |
| sdk.async_subtensor.sign_and_send_extrinsic | sdk | generic::<runtime call supplied by caller> | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.async_subtensor.sim_swap | sdk |  | COVERED_CLI_ONLY | agcli view swap-sim --netuid 1 --tao 1 --output json |
| sdk.async_subtensor.start_call | sdk | SubtensorModule.start_call | COVERED_CLI_ONLY | agcli subnet start --netuid 1 --yes |
| sdk.async_subtensor.subnet_exists | sdk |  | COVERED_CLI_ONLY | agcli subnet show --netuid 1 --output json |
| sdk.async_subtensor.swap_coldkey_announced | sdk | SubtensorModule.swap_coldkey_announced | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.async_subtensor.swap_stake | sdk | SubtensorModule.swap_stake \| SubtensorModule.swap_stake_limit | COVERED_CLI_ONLY | agcli stake swap --amount 1 --from 1 --to 2 --yes |
| sdk.async_subtensor.toggle_user_liquidity | sdk | Swap.toggle_user_liquidity | COVERED_CLI_ONLY | agcli liquidity toggle --netuid 1 --enable --yes |
| sdk.async_subtensor.transfer | sdk | Balances.transfer_all \| Balances.transfer_allow_death \| Balances.transfer_keep_alive | COVERED_CLI_ONLY | agcli transfer-all --dest <ss58> --yes |
| sdk.async_subtensor.transfer_stake | sdk | SubtensorModule.transfer_stake | COVERED_CLI_ONLY | agcli stake transfer-stake --netuid 1 --amount 1 --dest-hotkey <ss58> --yes |
| sdk.async_subtensor.unstake | sdk | SubtensorModule.remove_stake \| SubtensorModule.remove_stake_limit | COVERED_CLI_ONLY | agcli stake remove --amount 1 --netuid 1 --yes |
| sdk.async_subtensor.unstake_all | sdk | SubtensorModule.remove_stake_full_limit | COVERED_CLI_ONLY | agcli stake unstake-all --yes |
| sdk.async_subtensor.unstake_multiple | sdk | SubtensorModule.remove_stake \| SubtensorModule.remove_stake_full_limit \| SubtensorModule.remove_stake_limit | COVERED_CLI_ONLY | agcli stake remove --amount 1 --netuid 1 --yes |
| sdk.async_subtensor.update_cap_crowdloan | sdk | Crowdloan.update_cap | COVERED_CLI_ONLY | agcli crowdloan update-cap --id 0 --cap 200 --yes |
| sdk.async_subtensor.update_end_crowdloan | sdk | Crowdloan.update_end | COVERED_CLI_ONLY | agcli crowdloan update-end --id 0 --end 2000 --yes |
| sdk.async_subtensor.update_min_contribution_crowdloan | sdk | Crowdloan.update_min_contribution | COVERED_CLI_ONLY | agcli crowdloan update-min-contribution --id 0 --min 1 --yes |
| sdk.async_subtensor.weights | sdk |  | COVERED_CLI_ONLY | agcli weights show --netuid 1 --output json |
| sdk.async_subtensor.withdraw_crowdloan | sdk | Crowdloan.withdraw | COVERED_CLI_ONLY | agcli crowdloan withdraw --id 0 --yes |
| sdk.subtensor.add_liquidity | sdk | Swap.add_liquidity | COVERED_CLI_ONLY | agcli liquidity add --netuid 1 --price-low 1 --price-high 2 --amount 1000000000 --yes |
| sdk.subtensor.add_proxy | sdk | Proxy.add_proxy | COVERED_CLI_ONLY | agcli proxy add --delegate <ss58> --proxy-type any --delay 0 --yes |
| sdk.subtensor.add_stake | sdk | SubtensorModule.add_stake \| SubtensorModule.add_stake_limit | COVERED_CLI_ONLY | agcli stake add --amount 10 --netuid 1 --yes |
| sdk.subtensor.add_stake_burn | sdk | SubtensorModule.add_stake_burn | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.subtensor.add_stake_multiple | sdk | SubtensorModule.add_stake \| SubtensorModule.add_stake_limit | COVERED_CLI_ONLY | agcli stake add --amount 10 --netuid 1 --yes |
| sdk.subtensor.announce_coldkey_swap | sdk | SubtensorModule.announce_coldkey_swap | COVERED_CLI_ONLY | agcli swap coldkey --new-coldkey <ss58> --yes |
| sdk.subtensor.announce_proxy | sdk | Proxy.announce | COVERED_CLI_ONLY | agcli proxy announce --real <ss58> --call-hex 0x00 --yes |
| sdk.subtensor.bonds | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --full --output json |
| sdk.subtensor.burned_register | sdk | SubtensorModule.burned_register \| SubtensorModule.root_register | COVERED_CLI_ONLY | agcli subnet register-neuron --netuid 1 --yes |
| sdk.subtensor.claim_root | sdk | SubtensorModule.claim_root | COVERED_CLI_ONLY | agcli stake claim-root --netuid 1 --yes |
| sdk.subtensor.clear_coldkey_swap_announcement | sdk | SubtensorModule.clear_coldkey_swap_announcement | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.subtensor.commit_weights | sdk | SubtensorModule.commit_mechanism_weights | COVERED_CLI_ONLY | agcli weights commit-mechanism --netuid 1 --mechanism-id 0 --hash 0x... --yes |
| sdk.subtensor.contribute_crowdloan | sdk | Crowdloan.contribute | COVERED_CLI_ONLY | agcli crowdloan contribute --id 0 --amount 5 --yes |
| sdk.subtensor.create_crowdloan | sdk | Crowdloan.create | COVERED_CLI_ONLY | agcli crowdloan create --deposit 10 --min-contribution 1 --cap 100 --end 1000 --yes |
| sdk.subtensor.create_pure_proxy | sdk | Proxy.create_pure | COVERED_CLI_ONLY | agcli proxy create-pure --proxy-type any --delay 0 --yes |
| sdk.subtensor.dispute_coldkey_swap | sdk | SubtensorModule.dispute_coldkey_swap | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.subtensor.dissolve_crowdloan | sdk | Crowdloan.dissolve | COVERED_CLI_ONLY | agcli crowdloan dissolve --id 0 --yes |
| sdk.subtensor.does_hotkey_exist | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.subtensor.finalize_crowdloan | sdk | Crowdloan.finalize | COVERED_CLI_ONLY | agcli crowdloan finalize --id 0 --yes |
| sdk.subtensor.get_all_commitments | sdk |  | COVERED_CLI_ONLY | agcli commitment list --netuid 1 --output json |
| sdk.subtensor.get_all_ema_tao_inflow | sdk |  | COVERED_CLI_ONLY | agcli view dynamic --output json |
| sdk.subtensor.get_all_revealed_commitments | sdk |  | COVERED_CLI_ONLY | agcli subnet commits --netuid 1 --output json |
| sdk.subtensor.get_all_subnets_info | sdk |  | COVERED_CLI_ONLY | agcli subnet list --output json |
| sdk.subtensor.get_all_subnets_netuid | sdk |  | COVERED_CLI_ONLY | agcli subnet list --output json |
| sdk.subtensor.get_auto_stakes | sdk |  | COVERED_CLI_ONLY | agcli stake show-auto --address <ss58> --output json |
| sdk.subtensor.get_balance | sdk |  | COVERED_CLI_ONLY | agcli balance --address <ss58> --output json |
| sdk.subtensor.get_children | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.subtensor.get_coldkey_swap_announcement | sdk |  | COVERED_CLI_ONLY | agcli wallet check-swap --output json |
| sdk.subtensor.get_commitment_metadata | sdk |  | COVERED_CLI_ONLY | agcli commitment get --netuid 1 --hotkey-address <ss58> --output json |
| sdk.subtensor.get_crowdloan_by_id | sdk |  | COVERED_CLI_ONLY | agcli crowdloan info --id 0 --output json |
| sdk.subtensor.get_crowdloan_contributions | sdk |  | COVERED_CLI_ONLY | agcli crowdloan contributors --id 0 --output json |
| sdk.subtensor.get_crowdloan_next_id | sdk |  | COVERED_CLI_ONLY | agcli crowdloan list --output json |
| sdk.subtensor.get_crowdloans | sdk |  | COVERED_CLI_ONLY | agcli crowdloan list --output json |
| sdk.subtensor.get_delegate_by_hotkey | sdk |  | COVERED_CLI_ONLY | agcli delegate show --hotkey-address <ss58> --output json |
| sdk.subtensor.get_delegate_identities | sdk |  | COVERED_CLI_ONLY | agcli delegate list --output json |
| sdk.subtensor.get_delegated | sdk |  | COVERED_CLI_ONLY | agcli delegate show --hotkey-address <ss58> --output json |
| sdk.subtensor.get_delegates | sdk |  | COVERED_CLI_ONLY | agcli delegate list --output json |
| sdk.subtensor.get_ema_tao_inflow | sdk |  | COVERED_CLI_ONLY | agcli view dynamic --output json |
| sdk.subtensor.get_hotkey_owner | sdk |  | COVERED_CLI_ONLY | agcli view account --address <ss58> --output json |
| sdk.subtensor.get_hyperparameter | sdk |  | COVERED_CLI_ONLY | agcli subnet hyperparams --netuid 1 --output json |
| sdk.subtensor.get_liquidity_list | sdk |  | COVERED_CLI_ONLY | agcli subnet liquidity --netuid 1 --output json |
| sdk.subtensor.get_mechanism_count | sdk |  | COVERED_CLI_ONLY | agcli subnet mechanism-count --netuid 1 --output json |
| sdk.subtensor.get_mechanism_emission_split | sdk |  | COVERED_CLI_ONLY | agcli subnet emission-split --netuid 1 --output json |
| sdk.subtensor.get_minimum_required_stake | sdk |  | COVERED_CLI_ONLY | agcli subnet hyperparams --netuid 1 --output json |
| sdk.subtensor.get_netuids_for_hotkey | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.subtensor.get_neuron_for_pubkey_and_subnet | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.subtensor.get_owned_hotkeys | sdk |  | COVERED_CLI_ONLY | agcli wallet show --all --output json |
| sdk.subtensor.get_parents | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.subtensor.get_proxies | sdk |  | COVERED_CLI_ONLY | agcli proxy list --address <ss58> --output json |
| sdk.subtensor.get_proxies_for_real_account | sdk |  | COVERED_CLI_ONLY | agcli proxy list --address <ss58> --output json |
| sdk.subtensor.get_proxy_announcement | sdk |  | COVERED_CLI_ONLY | agcli proxy list-announcements --address <ss58> --output json |
| sdk.subtensor.get_proxy_announcements | sdk |  | COVERED_CLI_ONLY | agcli proxy list-announcements --address <ss58> --output json |
| sdk.subtensor.get_stake | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.subtensor.get_stake_for_coldkey_and_hotkey | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.subtensor.get_stake_info_for_coldkey | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.subtensor.get_stake_info_for_coldkeys | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.subtensor.get_stake_weight | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.subtensor.get_staking_hotkeys | sdk |  | COVERED_CLI_ONLY | agcli stake list --address <ss58> --output json |
| sdk.subtensor.get_subnet_burn_cost | sdk |  | COVERED_CLI_ONLY | agcli subnet create-cost |
| sdk.subtensor.get_subnet_hyperparameters | sdk |  | COVERED_CLI_ONLY | agcli subnet hyperparams --netuid 1 --output json |
| sdk.subtensor.get_subnet_info | sdk |  | COVERED_CLI_ONLY | agcli subnet show --netuid 1 --output json |
| sdk.subtensor.get_subnet_prices | sdk |  | COVERED_CLI_ONLY | agcli subnet liquidity --netuid 1 --output json |
| sdk.subtensor.get_timelocked_weight_commits | sdk |  | COVERED_CLI_ONLY | agcli weights status --netuid 1 --output json |
| sdk.subtensor.get_total_subnets | sdk |  | COVERED_CLI_ONLY | agcli subnet list --output json |
| sdk.subtensor.get_uid_for_hotkey_on_subnet | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.subtensor.kill_pure_proxy | sdk | Proxy.kill_pure | COVERED_CLI_ONLY | agcli proxy kill-pure --spawner <ss58> --proxy-type any --index 0 --height 0 --ext-index 0 --yes |
| sdk.subtensor.mev_submit_encrypted | sdk | MevShield.submit_encrypted | COVERED_CLI_ONLY | agcli --mev batch --file calls.json --yes |
| sdk.subtensor.modify_liquidity | sdk | Swap.modify_position | COVERED_CLI_ONLY | agcli liquidity modify --netuid 1 --position-id 1 --delta 1000 --yes |
| sdk.subtensor.move_stake | sdk | SubtensorModule.move_stake | COVERED_CLI_ONLY | agcli stake move --amount 1 --from 1 --to 2 --yes |
| sdk.subtensor.neuron_for_uid | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --uid 0 --output json |
| sdk.subtensor.neurons | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.subtensor.neurons_lite | sdk |  | COVERED_CLI_ONLY | agcli subnet metagraph --netuid 1 --output json |
| sdk.subtensor.poke_deposit | sdk | Proxy.poke_deposit | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.subtensor.proxy | sdk | Proxy.proxy | COVERED_CLI_ONLY | agcli --proxy <delegate_ss58> transfer --dest <ss58> --amount 1 --yes |
| sdk.subtensor.proxy_announced | sdk | Proxy.proxy_announced | COVERED_CLI_ONLY | agcli proxy proxy-announced --delegate <ss58> --real <ss58> --pallet SubtensorModule --call transfer_stake --yes |
| sdk.subtensor.query_identity | sdk |  | COVERED_CLI_ONLY | agcli identity show --address <ss58> --output json |
| sdk.subtensor.refund_crowdloan | sdk | Crowdloan.refund | COVERED_CLI_ONLY | agcli crowdloan refund --id 0 --yes |
| sdk.subtensor.register | sdk | SubtensorModule.register_limit \| SubtensorModule.root_register | COVERED_CLI_ONLY | agcli subnet register-neuron --netuid 1 --yes |
| sdk.subtensor.register_limit | sdk | SubtensorModule.register_limit | COVERED_CLI_ONLY | agcli subnet register-neuron --netuid 1 --yes |
| sdk.subtensor.register_subnet | sdk | SubtensorModule.register_network | COVERED_CLI_ONLY | agcli subnet register --yes |
| sdk.subtensor.reject_proxy_announcement | sdk | Proxy.reject_announcement | COVERED_CLI_ONLY | agcli proxy reject-announcement --delegate <ss58> --call-hash 0x... --yes |
| sdk.subtensor.remove_liquidity | sdk | Swap.remove_liquidity | COVERED_CLI_ONLY | agcli liquidity remove --netuid 1 --position-id 1 --yes |
| sdk.subtensor.remove_proxies | sdk | Proxy.remove_proxies | COVERED_CLI_ONLY | agcli proxy remove-all --yes |
| sdk.subtensor.remove_proxy | sdk | Proxy.remove_proxy | COVERED_CLI_ONLY | agcli proxy remove --delegate <ss58> --proxy-type any --delay 0 --yes |
| sdk.subtensor.remove_proxy_announcement | sdk | Proxy.remove_announcement | COVERED_CLI_ONLY | agcli proxy remove-announcement --real <ss58> --call-hash 0x... --yes |
| sdk.subtensor.reveal_weights | sdk | SubtensorModule.reveal_mechanism_weights | COVERED_CLI_ONLY | agcli weights reveal-mechanism --netuid 1 --mechanism-id 0 --weights "0:100" --salt s --yes |
| sdk.subtensor.root_register | sdk | SubtensorModule.root_register | COVERED_CLI_ONLY | agcli root register --netuid 1 --yes |
| sdk.subtensor.root_set_pending_childkey_cooldown | sdk | SubtensorModule.set_pending_childkey_cooldown \| Sudo.sudo | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.subtensor.set_auto_stake | sdk | SubtensorModule.set_coldkey_auto_stake_hotkey | COVERED_CLI_ONLY | agcli stake set-auto --netuid 1 --hotkey-address <ss58> --yes |
| sdk.subtensor.set_children | sdk | SubtensorModule.set_children | COVERED_CLI_ONLY | agcli stake set-children --netuid 1 --children "1:<hotkey>" --yes |
| sdk.subtensor.set_commitment | sdk | Commitments.set_commitment | COVERED_CLI_ONLY | agcli commitment set --netuid 1 --data "endpoint:https://example" --yes |
| sdk.subtensor.set_delegate_take | sdk | SubtensorModule.decrease_take \| SubtensorModule.increase_take | COVERED_CLI_ONLY | agcli delegate decrease-take --take 0.05 --yes |
| sdk.subtensor.set_reveal_commitment | sdk | Commitments.set_commitment | COVERED_CLI_ONLY | agcli commitment set --netuid 1 --data "endpoint:https://example" --yes |
| sdk.subtensor.set_root_claim_type | sdk | SubtensorModule.set_root_claim_type | COVERED_CLI_ONLY | agcli stake set-claim --netuid 1 --yes |
| sdk.subtensor.set_subnet_identity | sdk | SubtensorModule.set_subnet_identity | COVERED_CLI_ONLY | agcli identity set-subnet --netuid 1 --name demo --yes |
| sdk.subtensor.set_weights | sdk | SubtensorModule.commit_timelocked_mechanism_weights \| SubtensorModule.set_mechanism_weights | COVERED_CLI_ONLY | agcli weights commit-timelocked --netuid 1 --weights "0:100" --round 1 --yes |
| sdk.subtensor.sign_and_send_extrinsic | sdk | generic::<runtime call supplied by caller> | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.subtensor.sim_swap | sdk |  | COVERED_CLI_ONLY | agcli view swap-sim --netuid 1 --tao 1 --output json |
| sdk.subtensor.start_call | sdk | SubtensorModule.start_call | COVERED_CLI_ONLY | agcli subnet start --netuid 1 --yes |
| sdk.subtensor.subnet_exists | sdk |  | COVERED_CLI_ONLY | agcli subnet show --netuid 1 --output json |
| sdk.subtensor.swap_coldkey_announced | sdk | SubtensorModule.swap_coldkey_announced | COVERED_CLI_ONLY | agcli batch --file calls.json --yes |
| sdk.subtensor.swap_stake | sdk | SubtensorModule.swap_stake \| SubtensorModule.swap_stake_limit | COVERED_CLI_ONLY | agcli stake swap --amount 1 --from 1 --to 2 --yes |
| sdk.subtensor.toggle_user_liquidity | sdk | Swap.toggle_user_liquidity | COVERED_CLI_ONLY | agcli liquidity toggle --netuid 1 --enable --yes |
| sdk.subtensor.transfer | sdk | Balances.transfer_all \| Balances.transfer_allow_death \| Balances.transfer_keep_alive | COVERED_CLI_ONLY | agcli transfer-all --dest <ss58> --yes |
| sdk.subtensor.transfer_stake | sdk | SubtensorModule.transfer_stake | COVERED_CLI_ONLY | agcli stake transfer-stake --netuid 1 --amount 1 --dest-hotkey <ss58> --yes |
| sdk.subtensor.unstake | sdk | SubtensorModule.remove_stake \| SubtensorModule.remove_stake_limit | COVERED_CLI_ONLY | agcli stake remove --amount 1 --netuid 1 --yes |
| sdk.subtensor.unstake_all | sdk | SubtensorModule.remove_stake_full_limit | COVERED_CLI_ONLY | agcli stake unstake-all --yes |
| sdk.subtensor.unstake_multiple | sdk | SubtensorModule.remove_stake \| SubtensorModule.remove_stake_full_limit \| SubtensorModule.remove_stake_limit | COVERED_CLI_ONLY | agcli stake remove --amount 1 --netuid 1 --yes |
| sdk.subtensor.update_cap_crowdloan | sdk | Crowdloan.update_cap | COVERED_CLI_ONLY | agcli crowdloan update-cap --id 0 --cap 200 --yes |
| sdk.subtensor.update_end_crowdloan | sdk | Crowdloan.update_end | COVERED_CLI_ONLY | agcli crowdloan update-end --id 0 --end 2000 --yes |
| sdk.subtensor.update_min_contribution_crowdloan | sdk | Crowdloan.update_min_contribution | COVERED_CLI_ONLY | agcli crowdloan update-min-contribution --id 0 --min 1 --yes |
| sdk.subtensor.weights | sdk |  | COVERED_CLI_ONLY | agcli weights show --netuid 1 --output json |
| sdk.subtensor.withdraw_crowdloan | sdk | Crowdloan.withdraw | COVERED_CLI_ONLY | agcli crowdloan withdraw --id 0 --yes |

## Phase 4 candidates (GAP)

Rows: 65

| id | source | ref_extrinsic | agcli_status | agcli_invocation |
|---|---|---|---|---|
| btcli.config.add-proxy | btcli |  | GAP |  |
| btcli.config.clear | btcli |  | GAP |  |
| btcli.config.clear-proxies | btcli |  | GAP |  |
| btcli.config.proxies | btcli |  | GAP |  |
| btcli.config.remove-proxy | btcli |  | GAP |  |
| btcli.config.update-proxy | btcli |  | GAP |  |
| btcli.stake.child.get | btcli |  | GAP |  |
| btcli.sudo.proposals | btcli |  | GAP |  |
| btcli.sudo.senate | btcli |  | GAP |  |
| btcli.sudo.senate-vote | btcli | SubtensorModule.vote | GAP |  |
| btcli.sudo.stake-burn | btcli | SubtensorModule.add_stake_burn | GAP |  |
| btcli.wallet.regen-coldkeypub | btcli |  | GAP |  |
| btcli.wallet.regen-hotkeypub | btcli |  | GAP |  |
| sdk.async_subtensor.compose_call | sdk |  | GAP |  |
| sdk.async_subtensor.get_admin_freeze_window | sdk |  | GAP |  |
| sdk.async_subtensor.get_all_neuron_certificates | sdk |  | GAP |  |
| sdk.async_subtensor.get_children_pending | sdk |  | GAP |  |
| sdk.async_subtensor.get_coldkey_swap_announcement_delay | sdk |  | GAP |  |
| sdk.async_subtensor.get_coldkey_swap_announcements | sdk |  | GAP |  |
| sdk.async_subtensor.get_coldkey_swap_dispute | sdk |  | GAP |  |
| sdk.async_subtensor.get_coldkey_swap_disputes | sdk |  | GAP |  |
| sdk.async_subtensor.get_coldkey_swap_reannouncement_delay | sdk |  | GAP |  |
| sdk.async_subtensor.get_extrinsic_fee | sdk |  | GAP |  |
| sdk.async_subtensor.get_last_bonds_reset | sdk |  | GAP |  |
| sdk.async_subtensor.get_mev_shield_current_key | sdk |  | GAP |  |
| sdk.async_subtensor.get_mev_shield_next_key | sdk |  | GAP |  |
| sdk.async_subtensor.get_root_alpha_dividends_per_subnet | sdk |  | GAP |  |
| sdk.async_subtensor.get_root_claim_type | sdk |  | GAP |  |
| sdk.async_subtensor.get_root_claimable_all_rates | sdk |  | GAP |  |
| sdk.async_subtensor.get_root_claimed | sdk |  | GAP |  |
| sdk.async_subtensor.get_transfer_fee | sdk |  | GAP |  |
| sdk.async_subtensor.get_vote_data | sdk |  | GAP |  |
| sdk.async_subtensor.last_drand_round | sdk |  | GAP |  |
| sdk.async_subtensor.query_map | sdk |  | GAP |  |
| sdk.async_subtensor.query_map_subtensor | sdk |  | GAP |  |
| sdk.async_subtensor.query_module | sdk |  | GAP |  |
| sdk.async_subtensor.query_runtime_api | sdk |  | GAP |  |
| sdk.async_subtensor.query_subtensor | sdk |  | GAP |  |
| sdk.async_subtensor.validate_extrinsic_params | sdk |  | GAP |  |
| sdk.subtensor.compose_call | sdk |  | GAP |  |
| sdk.subtensor.get_admin_freeze_window | sdk |  | GAP |  |
| sdk.subtensor.get_all_neuron_certificates | sdk |  | GAP |  |
| sdk.subtensor.get_children_pending | sdk |  | GAP |  |
| sdk.subtensor.get_coldkey_swap_announcement_delay | sdk |  | GAP |  |
| sdk.subtensor.get_coldkey_swap_announcements | sdk |  | GAP |  |
| sdk.subtensor.get_coldkey_swap_dispute | sdk |  | GAP |  |
| sdk.subtensor.get_coldkey_swap_disputes | sdk |  | GAP |  |
| sdk.subtensor.get_coldkey_swap_reannouncement_delay | sdk |  | GAP |  |
| sdk.subtensor.get_extrinsic_fee | sdk |  | GAP |  |
| sdk.subtensor.get_last_bonds_reset | sdk |  | GAP |  |
| sdk.subtensor.get_mev_shield_current_key | sdk |  | GAP |  |
| sdk.subtensor.get_mev_shield_next_key | sdk |  | GAP |  |
| sdk.subtensor.get_root_alpha_dividends_per_subnet | sdk |  | GAP |  |
| sdk.subtensor.get_root_claim_type | sdk |  | GAP |  |
| sdk.subtensor.get_root_claimable_all_rates | sdk |  | GAP |  |
| sdk.subtensor.get_root_claimed | sdk |  | GAP |  |
| sdk.subtensor.get_transfer_fee | sdk |  | GAP |  |
| sdk.subtensor.get_vote_data | sdk |  | GAP |  |
| sdk.subtensor.last_drand_round | sdk |  | GAP |  |
| sdk.subtensor.query_map | sdk |  | GAP |  |
| sdk.subtensor.query_map_subtensor | sdk |  | GAP |  |
| sdk.subtensor.query_module | sdk |  | GAP |  |
| sdk.subtensor.query_runtime_api | sdk |  | GAP |  |
| sdk.subtensor.query_subtensor | sdk |  | GAP |  |
| sdk.subtensor.validate_extrinsic_params | sdk |  | GAP |  |

## Informational (COVERED_E2E + N/A)

Rows: 4

| id | source | ref_extrinsic | agcli_status | agcli_invocation |
|---|---|---|---|---|
| btcli.axon.reset | btcli | SubtensorModule.serve_axon | N/A | agcli serve reset --netuid 1 |
| btcli.axon.set | btcli | SubtensorModule.serve_axon | N/A | agcli serve axon --netuid 1 --ip 127.0.0.1 --port 8091 |
| sdk.async_subtensor.serve_axon | sdk | SubtensorModule.serve_axon \| SubtensorModule.serve_axon_tls | N/A | agcli serve axon --netuid 1 --ip 127.0.0.1 --port 8091 |
| sdk.subtensor.serve_axon | sdk | SubtensorModule.serve_axon \| SubtensorModule.serve_axon_tls | N/A | agcli serve axon --netuid 1 --ip 127.0.0.1 --port 8091 |

