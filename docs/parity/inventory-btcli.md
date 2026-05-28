# btcli command inventory (v9.21.2)

Generated from installed `bittensor-cli` sources in `.venv` and grouped by top-level command.

## wallet

- [`btcli.wallet.associate-hotkey`](inventory-btcli.json#btcli.wallet.associate-hotkey) — `write` — extrinsic: `SubtensorModule.try_associate_hotkey` — Associate a hotkey with a wallet(coldkey).
- [`btcli.wallet.balance`](inventory-btcli.json#btcli.wallet.balance) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.wallet.create`](inventory-btcli.json#btcli.wallet.create) — `mixed` — extrinsic: `null` — Local wallet/key management operation; no chain transaction submitted.
- [`btcli.wallet.get-identity`](inventory-btcli.json#btcli.wallet.get-identity) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.wallet.list`](inventory-btcli.json#btcli.wallet.list) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.wallet.new-coldkey`](inventory-btcli.json#btcli.wallet.new-coldkey) — `mixed` — extrinsic: `null` — Local wallet/key management operation; no chain transaction submitted.
- [`btcli.wallet.new-hotkey`](inventory-btcli.json#btcli.wallet.new-hotkey) — `mixed` — extrinsic: `null` — Local wallet/key management operation; no chain transaction submitted.
- [`btcli.wallet.overview`](inventory-btcli.json#btcli.wallet.overview) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.wallet.regen-coldkey`](inventory-btcli.json#btcli.wallet.regen-coldkey) — `mixed` — extrinsic: `null` — Local wallet/key management operation; no chain transaction submitted.
- [`btcli.wallet.regen-coldkeypub`](inventory-btcli.json#btcli.wallet.regen-coldkeypub) — `mixed` — extrinsic: `null` — Local wallet/key management operation; no chain transaction submitted.
- [`btcli.wallet.regen-hotkey`](inventory-btcli.json#btcli.wallet.regen-hotkey) — `mixed` — extrinsic: `null` — Local wallet/key management operation; no chain transaction submitted.
- [`btcli.wallet.regen-hotkeypub`](inventory-btcli.json#btcli.wallet.regen-hotkeypub) — `mixed` — extrinsic: `null` — Local wallet/key management operation; no chain transaction submitted.
- [`btcli.wallet.set-identity`](inventory-btcli.json#btcli.wallet.set-identity) — `write` — extrinsic: `SubtensorModule.set_identity` — Create or update the on-chain identity of a coldkey or a hotkey on the Bittensor network.
- [`btcli.wallet.sign`](inventory-btcli.json#btcli.wallet.sign) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.wallet.swap-check`](inventory-btcli.json#btcli.wallet.swap-check) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.wallet.swap-coldkey`](inventory-btcli.json#btcli.wallet.swap-coldkey) — `mixed` — extrinsic: `SubtensorModule.announce_coldkey_swap, SubtensorModule.dispute_coldkey_swap, SubtensorModule.clear_coldkey_swap_announcement, SubtensorModule.swap_coldkey_announced` — Action-based command: check is read-only, while announce/dispute/clear/execute submit extrinsics.
- [`btcli.wallet.swap-hotkey`](inventory-btcli.json#btcli.wallet.swap-hotkey) — `write` — extrinsic: `SubtensorModule.swap_hotkey` — Swap hotkeys of a given wallet on the blockchain.
- [`btcli.wallet.transfer`](inventory-btcli.json#btcli.wallet.transfer) — `write` — extrinsic: `Balances.transfer_allow_death, Balances.transfer_keep_alive` — Send TAO tokens from one wallet to another wallet on the Bittensor network.
- [`btcli.wallet.verify`](inventory-btcli.json#btcli.wallet.verify) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.

## subnets

- [`btcli.subnets.burn-cost`](inventory-btcli.json#btcli.subnets.burn-cost) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.check-start`](inventory-btcli.json#btcli.subnets.check-start) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.create`](inventory-btcli.json#btcli.subnets.create) — `write` — extrinsic: `SubtensorModule.register_network, SubtensorModule.register_network_with_identity` — Registers a new subnet on the network.
- [`btcli.subnets.get-identity`](inventory-btcli.json#btcli.subnets.get-identity) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.hyperparameters`](inventory-btcli.json#btcli.subnets.hyperparameters) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.list`](inventory-btcli.json#btcli.subnets.list) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.mechanisms.count`](inventory-btcli.json#btcli.subnets.mechanisms.count) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.mechanisms.emissions`](inventory-btcli.json#btcli.subnets.mechanisms.emissions) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.mechanisms.set`](inventory-btcli.json#btcli.subnets.mechanisms.set) — `write` — extrinsic: `AdminUtils.sudo_set_mechanism_count` — Configure how many mechanisms are registered for a subnet.
- [`btcli.subnets.mechanisms.split-emissions`](inventory-btcli.json#btcli.subnets.mechanisms.split-emissions) — `write` — extrinsic: `AdminUtils.sudo_set_mechanism_emission_split` — Update the emission split across mechanisms for a subnet.
- [`btcli.subnets.price`](inventory-btcli.json#btcli.subnets.price) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.register`](inventory-btcli.json#btcli.subnets.register) — `write` — extrinsic: `SubtensorModule.root_register` — Register a neuron (a subnet validator or a subnet miner) in the specified subnet by recycling some TAO.
- [`btcli.subnets.set-identity`](inventory-btcli.json#btcli.subnets.set-identity) — `write` — extrinsic: `SubtensorModule.set_subnet_identity` — Set or update the identity information for a subnet.
- [`btcli.subnets.set-symbol`](inventory-btcli.json#btcli.subnets.set-symbol) — `write` — extrinsic: `SubtensorModule.update_symbol` — Allows the user to update their subnet symbol to a different available symbol.
- [`btcli.subnets.show`](inventory-btcli.json#btcli.subnets.show) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.subnets.start`](inventory-btcli.json#btcli.subnets.start) — `write` — extrinsic: `SubtensorModule.start_call` — Starts a subnet's emission schedule.

## stake

- [`btcli.stake.add`](inventory-btcli.json#btcli.stake.add) — `write` — extrinsic: `SubtensorModule.add_stake, SubtensorModule.add_stake_limit` — Stake TAO to one or more hotkeys on specific netuids with your coldkey.
- [`btcli.stake.auto`](inventory-btcli.json#btcli.stake.auto) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.stake.child.get`](inventory-btcli.json#btcli.stake.child.get) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.stake.child.revoke`](inventory-btcli.json#btcli.stake.child.revoke) — `write` — extrinsic: `SubtensorModule.set_children` — Remove all children hotkeys on a specified subnet (or all).
- [`btcli.stake.child.set`](inventory-btcli.json#btcli.stake.child.set) — `write` — extrinsic: `SubtensorModule.set_children` — Set child hotkeys on a specified subnet (or all).
- [`btcli.stake.child.take`](inventory-btcli.json#btcli.stake.child.take) — `write` — extrinsic: `SubtensorModule.set_childkey_take` — Get and set your child hotkey take on a specified subnet.
- [`btcli.stake.list`](inventory-btcli.json#btcli.stake.list) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.stake.move`](inventory-btcli.json#btcli.stake.move) — `write` — extrinsic: `SubtensorModule.move_stake` — Move staked TAO between hotkeys while keeping the same coldkey ownership.
- [`btcli.stake.process-claim`](inventory-btcli.json#btcli.stake.process-claim) — `write` — extrinsic: `SubtensorModule.claim_root` — Manually claim accumulated root network emissions for your coldkey.
- [`btcli.stake.remove`](inventory-btcli.json#btcli.stake.remove) — `write` — extrinsic: `SubtensorModule.remove_stake, SubtensorModule.remove_stake_limit, SubtensorModule.unstake_all_alpha` — Unstake TAO from one or more hotkeys and transfer them back to the user's coldkey wallet.
- [`btcli.stake.set-auto`](inventory-btcli.json#btcli.stake.set-auto) — `write` — extrinsic: `SubtensorModule.set_coldkey_auto_stake_hotkey` — Set the auto-stake destination hotkey for a coldkey.
- [`btcli.stake.set-claim`](inventory-btcli.json#btcli.stake.set-claim) — `write` — extrinsic: `SubtensorModule.set_root_claim_type` — Set the root claim type for your coldkey.
- [`btcli.stake.swap`](inventory-btcli.json#btcli.stake.swap) — `write` — extrinsic: `SubtensorModule.swap_stake, SubtensorModule.swap_stake_limit` — Swap stake between different subnets while keeping the same coldkey-hotkey pair ownership.
- [`btcli.stake.transfer`](inventory-btcli.json#btcli.stake.transfer) — `write` — extrinsic: `SubtensorModule.transfer_stake` — Transfer stake between coldkeys while keeping the same hotkey ownership.
- [`btcli.stake.wizard`](inventory-btcli.json#btcli.stake.wizard) — `mixed` — extrinsic: `SubtensorModule.move_stake, SubtensorModule.transfer_stake, SubtensorModule.swap_stake, SubtensorModule.swap_stake_limit` — Interactive planner that dispatches move/transfer/swap stake extrinsics depending on selected flow.

## sudo

- [`btcli.sudo.get`](inventory-btcli.json#btcli.sudo.get) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.sudo.get-take`](inventory-btcli.json#btcli.sudo.get-take) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.sudo.proposals`](inventory-btcli.json#btcli.sudo.proposals) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.sudo.senate`](inventory-btcli.json#btcli.sudo.senate) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.sudo.senate-vote`](inventory-btcli.json#btcli.sudo.senate-vote) — `write` — extrinsic: `SubtensorModule.vote` — Cast a vote on an active proposal in Bittensor's governance protocol.
- [`btcli.sudo.set`](inventory-btcli.json#btcli.sudo.set) — `write` — extrinsic: `Sudo.sudo(inner AdminUtils.*|SubtensorModule.*)` — Used to set hyperparameters for a specific subnet.
- [`btcli.sudo.set-take`](inventory-btcli.json#btcli.sudo.set-take) — `write` — extrinsic: `SubtensorModule.increase_take, SubtensorModule.decrease_take` — Allows users to change their delegate take percentage.
- [`btcli.sudo.stake-burn`](inventory-btcli.json#btcli.sudo.stake-burn) — `write` — extrinsic: `SubtensorModule.add_stake_burn` — Allows subnet owners to buy back alpha on their subnet by staking TAO and immediately burning the acquired alpha.
- [`btcli.sudo.trim`](inventory-btcli.json#btcli.sudo.trim) — `write` — extrinsic: `AdminUtils.sudo_trim_to_max_allowed_uids` — Allows subnet owners to trim UIDs on their subnet to a specified max number of netuids.

## weights

- [`btcli.weights.commit`](inventory-btcli.json#btcli.weights.commit) — `write` — extrinsic: `SubtensorModule.commit_weights` — Commit weights for specific subnet.
- [`btcli.weights.reveal`](inventory-btcli.json#btcli.weights.reveal) — `write` — extrinsic: `SubtensorModule.reveal_weights` — Reveal weights for a specific subnet.

## view

- [`btcli.view.dashboard`](inventory-btcli.json#btcli.view.dashboard) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.

## config

- [`btcli.config.add-proxy`](inventory-btcli.json#btcli.config.add-proxy) — `mixed` — extrinsic: `null` — Mutates local btcli config/proxy address book only; no on-chain extrinsic.
- [`btcli.config.clear`](inventory-btcli.json#btcli.config.clear) — `mixed` — extrinsic: `null` — Mutates local btcli config/proxy address book only; no on-chain extrinsic.
- [`btcli.config.clear-proxies`](inventory-btcli.json#btcli.config.clear-proxies) — `mixed` — extrinsic: `null` — Mutates local btcli config/proxy address book only; no on-chain extrinsic.
- [`btcli.config.get`](inventory-btcli.json#btcli.config.get) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.config.proxies`](inventory-btcli.json#btcli.config.proxies) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.config.remove-proxy`](inventory-btcli.json#btcli.config.remove-proxy) — `mixed` — extrinsic: `null` — Mutates local btcli config/proxy address book only; no on-chain extrinsic.
- [`btcli.config.set`](inventory-btcli.json#btcli.config.set) — `mixed` — extrinsic: `null` — Mutates local btcli config/proxy address book only; no on-chain extrinsic.
- [`btcli.config.update-proxy`](inventory-btcli.json#btcli.config.update-proxy) — `mixed` — extrinsic: `null` — Mutates local btcli config/proxy address book only; no on-chain extrinsic.

## utils

- [`btcli.utils.convert`](inventory-btcli.json#btcli.utils.convert) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.utils.latency`](inventory-btcli.json#btcli.utils.latency) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.

## axon

- [`btcli.axon.reset`](inventory-btcli.json#btcli.axon.reset) — `write` — extrinsic: `SubtensorModule.serve_axon` — Reset the axon information for a neuron on the network.
- [`btcli.axon.set`](inventory-btcli.json#btcli.axon.set) — `write` — extrinsic: `SubtensorModule.serve_axon` — Set the axon information for a neuron on the network.

## proxy

- [`btcli.proxy.add`](inventory-btcli.json#btcli.proxy.add) — `write` — extrinsic: `Proxy.add_proxy` — Registers an existing account as a standard proxy for the delegator.
- [`btcli.proxy.create`](inventory-btcli.json#btcli.proxy.create) — `write` — extrinsic: `Proxy.create_pure` — Creates a new pure proxy account.
- [`btcli.proxy.execute`](inventory-btcli.json#btcli.proxy.execute) — `write` — extrinsic: `Proxy.proxy_announced` — Executes a previously announced proxy call.
- [`btcli.proxy.kill`](inventory-btcli.json#btcli.proxy.kill) — `write` — extrinsic: `Proxy.kill_pure` — Permanently removes a pure proxy account.
- [`btcli.proxy.remove`](inventory-btcli.json#btcli.proxy.remove) — `write` — extrinsic: `Proxy.remove_proxy, Proxy.remove_proxies` — Unregisters a proxy from an account.

## crowd

- [`btcli.crowd.contribute`](inventory-btcli.json#btcli.crowd.contribute) — `write` — extrinsic: `Crowdloan.contribute` — Contribute TAO to an active crowdloan.
- [`btcli.crowd.contributors`](inventory-btcli.json#btcli.crowd.contributors) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.crowd.create`](inventory-btcli.json#btcli.crowd.create) — `write` — extrinsic: `Crowdloan.create, SubtensorModule.register_leased_network` — Start a new crowdloan campaign for fundraising or subnet leasing.
- [`btcli.crowd.dissolve`](inventory-btcli.json#btcli.crowd.dissolve) — `write` — extrinsic: `Crowdloan.dissolve` — Dissolve a crowdloan after all contributors have been refunded.
- [`btcli.crowd.finalize`](inventory-btcli.json#btcli.crowd.finalize) — `write` — extrinsic: `Crowdloan.finalize` — Finalize a successful crowdloan that has reached its cap.
- [`btcli.crowd.info`](inventory-btcli.json#btcli.crowd.info) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.crowd.list`](inventory-btcli.json#btcli.crowd.list) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.crowd.refund`](inventory-btcli.json#btcli.crowd.refund) — `write` — extrinsic: `Crowdloan.refund` — Refund contributors of a non-finalized crowdloan.
- [`btcli.crowd.update`](inventory-btcli.json#btcli.crowd.update) — `write` — extrinsic: `Crowdloan.update_min_contribution, Crowdloan.update_end, Crowdloan.update_cap` — Update one mutable field on a non-finalized crowdloan.
- [`btcli.crowd.withdraw`](inventory-btcli.json#btcli.crowd.withdraw) — `write` — extrinsic: `Crowdloan.withdraw` — Withdraw contributions from a non-finalized crowdloan.

## liquidity

- [`btcli.liquidity.add`](inventory-btcli.json#btcli.liquidity.add) — `write` — extrinsic: `Swap.add_liquidity` — Add liquidity to the swap (as a combination of TAO + Alpha).
- [`btcli.liquidity.list`](inventory-btcli.json#btcli.liquidity.list) — `read` — extrinsic: `null` — Read-only or local-only command path; no on-chain extrinsic is submitted.
- [`btcli.liquidity.modify`](inventory-btcli.json#btcli.liquidity.modify) — `write` — extrinsic: `Swap.modify_position` — Modifies the liquidity position for the given subnet.
- [`btcli.liquidity.remove`](inventory-btcli.json#btcli.liquidity.remove) — `write` — extrinsic: `Swap.remove_liquidity` — Remove liquidity from the swap (as a combination of TAO + Alpha).
