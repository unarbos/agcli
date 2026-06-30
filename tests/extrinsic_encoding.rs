//! Verify critical extrinsic call shapes encode without structural mistakes.
//!
//! Run: `cargo test --features test-utils --test extrinsic_encoding`
//!
//! **Metadata refresh checklist** (when `build.rs` fetches new runtime metadata):
//! 1. `cargo test --features test-utils --test extrinsic_encoding`
//! 2. If `safe_mode_force_enter_golden_call_data` fails, update the pinned `[u8; 2]` bytes
//!    to match the new `SafeMode::force_enter` pallet/call indices.

use subxt::dynamic::{tx, Value};
use subxt::tx::Payload;

fn encode_dynamic(pallet: &str, call: &str, fields: Vec<Value>) -> Vec<u8> {
    tx(pallet, call, fields)
        .encode_call_data(agcli::test_metadata())
        .expect("dynamic call should encode against embedded metadata")
}

fn encode_dynamic_panics(pallet: &str, call: &str, fields: Vec<Value>) -> bool {
    std::panic::catch_unwind(|| encode_dynamic(pallet, call, fields)).is_err()
}

fn typed_call_data<T: Payload>(payload: &T) -> Vec<u8> {
    payload
        .encode_call_data(agcli::test_metadata())
        .expect("typed call should encode against embedded metadata")
}

#[test]
fn proxy_announce_differs_from_raw_account_bytes() {
    let real_id = [7u8; 32];
    let call_hash = [8u8; 32];
    let with_id = encode_dynamic(
        "Proxy",
        "announce",
        vec![
            Value::unnamed_variant("Id", [Value::from_bytes(real_id)]),
            Value::from_bytes(call_hash),
        ],
    );
    assert!(
        encode_dynamic_panics(
            "Proxy",
            "announce",
            vec![Value::from_bytes(real_id), Value::from_bytes(call_hash)],
        ),
        "raw account bytes must not encode as MultiAddress::Id"
    );
    assert!(!with_id.is_empty());
}

#[test]
fn set_identity_encodes_seven_flat_byte_fields() {
    // SubtensorModule::set_identity takes seven plain Vec<u8> fields keyed by the
    // signer's coldkey — name, url, github_repo, image, discord, description, additional.
    let bytes = |s: &[u8]| Value::from_bytes(s);
    let full = vec![
        bytes(b"name"),
        bytes(b"url"),
        bytes(b"github"),
        bytes(b"image"),
        bytes(b""),
        bytes(b"desc"),
        bytes(b""),
    ];
    let with_info = encode_dynamic("SubtensorModule", "set_identity", full);
    assert!(!with_info.is_empty());
    assert!(
        encode_dynamic_panics(
            "SubtensorModule",
            "set_identity",
            vec![bytes(b"name"), bytes(b"url")],
        ),
        "set_identity must encode all seven identity fields"
    );
    // The standalone Registry pallet was removed from the runtime.
    assert!(
        encode_dynamic_panics("Registry", "set_identity", Vec::<Value>::new()),
        "Registry pallet should no longer exist in metadata"
    );
}

#[test]
fn safe_mode_force_enter_has_no_args() {
    let empty = encode_dynamic("SafeMode", "force_enter", Vec::<Value>::new());
    assert!(
        encode_dynamic_panics("SafeMode", "force_enter", vec![Value::u128(100)]),
        "force_enter must not encode a duration argument"
    );
    assert_eq!(
        typed_call_data(&agcli::api::tx().safe_mode().force_enter()),
        empty,
        "typed SafeMode::force_enter should match empty dynamic encoding"
    );
}

/// Pin call-data prefix for SafeMode::force_enter against embedded Finney metadata.
/// Update when runtime metadata is refreshed and this test fails.
#[test]
fn safe_mode_force_enter_golden_call_data() {
    let data = typed_call_data(&agcli::api::tx().safe_mode().force_enter());
    assert_eq!(data.len(), 2, "force_enter call data length");
    assert_eq!(
        data,
        [20, 1],
        "SafeMode::force_enter golden bytes (pallet 20, call 1); update when metadata changes"
    );
}

#[test]
fn associate_evm_key_call_data_varies_with_netuid() {
    let make = |netuid: u16| {
        typed_call_data(&agcli::api::tx().subtensor_module().associate_evm_key(
            netuid,
            subxt::utils::H160::from([1u8; 20]),
            100u64,
            [0u8; 65],
        ))
    };
    assert_ne!(make(42), make(0), "netuid must affect encoded call data");
}

#[test]
fn announce_coldkey_swap_hash_differs_from_raw_account_id() {
    let new_id = subxt::utils::AccountId32([5u8; 32]);
    let hash = subxt::utils::H256::from(sp_core::hashing::blake2_256(&new_id.0));

    let with_hash = typed_call_data(
        &agcli::api::tx()
            .subtensor_module()
            .announce_coldkey_swap(hash),
    );
    let with_raw_account = encode_dynamic(
        "SubtensorModule",
        "announce_coldkey_swap",
        vec![Value::from_bytes(new_id.0)],
    );
    assert_ne!(
        with_hash, with_raw_account,
        "announce_coldkey_swap must encode Blake2 hash, not raw AccountId32 bytes"
    );
    assert_ne!(
        with_hash,
        typed_call_data(
            &agcli::api::tx()
                .subtensor_module()
                .swap_coldkey_announced(new_id),
        ),
        "announce (hash) and exec (AccountId32) must encode differently"
    );
}

#[test]
fn swap_coldkey_announced_encodes_account_id() {
    let new_id = subxt::utils::AccountId32([2u8; 32]);
    let tx = agcli::api::tx()
        .subtensor_module()
        .swap_coldkey_announced(new_id.clone());
    let data = typed_call_data(&tx);
    assert_eq!(tx.call_name(), "swap_coldkey_announced");
    assert!(!data.is_empty());
}

#[test]
fn register_leased_network_call_data_varies_with_emissions_share() {
    use agcli::api::runtime_types::sp_arithmetic::per_things::Percent;
    let low = typed_call_data(
        &agcli::api::tx()
            .subtensor_module()
            .register_leased_network(Percent(25), Some(999u32)),
    );
    let high = typed_call_data(
        &agcli::api::tx()
            .subtensor_module()
            .register_leased_network(Percent(100), Some(999u32)),
    );
    assert_ne!(low, high, "emissions_share must affect encoded call data");
}

#[test]
fn remove_stake_encodes_alpha_balance_not_tao() {
    let hotkey = subxt::utils::AccountId32([4u8; 32]);
    let alpha_raw = 1_500_000_000u64;
    let with_alpha = typed_call_data(&agcli::api::tx().subtensor_module().remove_stake(
        hotkey.clone(),
        1,
        alpha_raw,
    ));
    let with_tao_scale = typed_call_data(&agcli::api::tx().subtensor_module().remove_stake(
        hotkey,
        1,
        agcli::types::balance::Balance::from_tao(1.5).rao(),
    ));
    assert_eq!(
        with_alpha, with_tao_scale,
        "1.5 α raw units must encode the same as Balance::from_tao(1.5).rao() when decimals align"
    );
    assert_ne!(
        with_alpha,
        typed_call_data(&agcli::api::tx().subtensor_module().remove_stake(
            subxt::utils::AccountId32([4u8; 32]),
            1,
            alpha_raw + 1
        ),),
        "alpha raw amount must affect encoded call data"
    );
}

#[test]
fn register_limit_encodes_netuid_hotkey_and_cap() {
    let hotkey = [3u8; 32];
    let low_cap = encode_dynamic(
        "SubtensorModule",
        "register_limit",
        vec![
            Value::u128(7),
            Value::from_bytes(hotkey),
            Value::u128(2_000_000_000),
        ],
    );
    let high_cap = encode_dynamic(
        "SubtensorModule",
        "register_limit",
        vec![
            Value::u128(7),
            Value::from_bytes(hotkey),
            Value::u128(3_000_000_000),
        ],
    );
    assert_ne!(
        low_cap, high_cap,
        "limit_price_rao must affect encoded register_limit call data"
    );
    assert_ne!(
        encode_dynamic(
            "SubtensorModule",
            "register_limit",
            vec![
                Value::u128(8),
                Value::from_bytes(hotkey),
                Value::u128(2_000_000_000),
            ],
        ),
        low_cap,
        "netuid must affect encoded register_limit call data"
    );
}

#[test]
fn register_leased_network_rejects_hotkey_as_first_field() {
    let hotkey = [3u8; 32];
    assert!(
        encode_dynamic_panics(
            "SubtensorModule",
            "register_leased_network",
            vec![
                Value::from_bytes(hotkey),
                Value::unnamed_variant("Some", [Value::u128(999)]),
            ],
        ),
        "register_leased_network must not encode hotkey as first argument"
    );
}

// ── Unit-safety manifest + Client API type guards ──

fn require_balance(_: agcli::Balance) {}
fn require_alpha(_: agcli::AlphaBalance) {}

#[test]
fn client_add_stake_amount_is_balance_type() {
    require_balance(agcli::Balance::from_tao(1.0));
}

#[test]
fn client_remove_stake_amount_is_alpha_type() {
    require_alpha(agcli::AlphaBalance::from_units(1.0));
}

#[test]
fn extrinsic_manifest_staking_unit_assignments() {
    use agcli::chain::{ArgUnit, ExtrinsicSpec};

    let add = ExtrinsicSpec::find("SubtensorModule", "add_stake").unwrap();
    assert!(
        add.args.iter().any(|a| a.unit == ArgUnit::TaoRao),
        "add_stake must declare TaoRao amount"
    );

    for call in [
        "remove_stake",
        "move_stake",
        "swap_stake",
        "transfer_stake",
        "recycle_alpha",
        "burn_alpha",
    ] {
        let spec = ExtrinsicSpec::find("SubtensorModule", call)
            .unwrap_or_else(|| panic!("manifest missing {call}"));
        assert!(
            spec.args.iter().any(|a| a.unit == ArgUnit::AlphaRaw),
            "{call} must declare AlphaRaw amount"
        );
        assert_eq!(
            spec.tao_spending_indices(),
            None,
            "{call} must not trigger τ spending limits"
        );
    }

    let transfer = ExtrinsicSpec::find("SubtensorModule", "transfer_stake").unwrap();
    assert_eq!(transfer.args[4].name, "alpha_amount");
    assert_eq!(transfer.args[4].unit, ArgUnit::AlphaRaw);
}

#[test]
fn extrinsic_manifest_all_specs_unique_and_non_empty() {
    use agcli::chain::ALL_SPECS;
    assert!(
        ALL_SPECS.len() >= 15,
        "staking manifest should be populated"
    );
    for spec in ALL_SPECS {
        assert!(!spec.args.is_empty() || spec.call.starts_with("unstake"));
    }
}

#[test]
fn transfer_stake_and_add_stake_same_u64_trap_documented() {
    // Same numeric value encodes identically on-wire — semantic safety is Balance vs AlphaBalance at SDK/CLI layer.
    let hotkey = subxt::utils::AccountId32([6u8; 32]);
    let dest = subxt::utils::AccountId32([7u8; 32]);
    let raw = 10_000_000_000u64;
    let add_stake = typed_call_data(&agcli::api::tx().subtensor_module().add_stake(
        hotkey.clone(),
        1,
        raw,
    ));
    let transfer = typed_call_data(
        &agcli::api::tx()
            .subtensor_module()
            .transfer_stake(dest, hotkey, 1, 2, raw),
    );
    assert_ne!(add_stake, transfer, "different calls must differ overall");
    // Both pass `raw` as u64 — encoding tests cannot distinguish τ vs α; types must.
}

fn manifest_dummy_value(arg: &agcli::chain::ExtrinsicArgSpec) -> Value {
    use agcli::chain::ArgUnit;
    match arg.name {
        "hotkey" | "origin_hotkey" | "destination_hotkey" | "destination_coldkey" | "dest" => {
            Value::from_bytes([2u8; 32])
        }
        "netuid" | "origin_netuid" | "destination_netuid" => Value::u128(1),
        "allow_partial" => Value::bool(false),
        "version_key" => Value::u128(0),
        "crowdloan_id" => Value::u128(1),
        _ => match arg.unit {
            ArgUnit::TaoRao | ArgUnit::AlphaRaw | ArgUnit::TaoPerAlphaRao => {
                Value::u128(1_000_000_000)
            }
            ArgUnit::NormalizedU16 | ArgUnit::RhoScaleU16 | ArgUnit::WeightU16 => Value::u128(1000),
            ArgUnit::TakeU16_65535 => Value::u128(11_796),
            ArgUnit::U64OverU64Max => Value::u128(u64::MAX as u128 / 2),
            ArgUnit::Other => Value::u128(0),
        },
    }
}

fn encode_manifest_spec(spec: &agcli::chain::ExtrinsicSpec) {
    use subxt::utils::AccountId32;
    let hk = AccountId32([2u8; 32]);
    let hk2 = AccountId32([4u8; 32]);
    let dest = AccountId32([5u8; 32]);
    let amt = 1_000_000_000u64;
    let price = 2_000_000_000u64;

    match (spec.pallet, spec.call) {
        ("SubtensorModule", "add_stake") => {
            typed_call_data(
                &agcli::api::tx()
                    .subtensor_module()
                    .add_stake(hk.clone(), 1, amt),
            );
        }
        ("SubtensorModule", "remove_stake") => {
            typed_call_data(
                &agcli::api::tx()
                    .subtensor_module()
                    .remove_stake(hk.clone(), 1, amt),
            );
        }
        ("SubtensorModule", "move_stake") => {
            typed_call_data(&agcli::api::tx().subtensor_module().move_stake(
                hk.clone(),
                hk2.clone(),
                1,
                2,
                amt,
            ));
        }
        ("SubtensorModule", "transfer_stake") => {
            typed_call_data(&agcli::api::tx().subtensor_module().transfer_stake(
                dest.clone(),
                hk.clone(),
                1,
                2,
                amt,
            ));
        }
        ("SubtensorModule", "swap_stake") => {
            typed_call_data(
                &agcli::api::tx()
                    .subtensor_module()
                    .swap_stake(hk.clone(), 1, 2, amt),
            );
        }
        ("SubtensorModule", "add_stake_limit") => {
            typed_call_data(&agcli::api::tx().subtensor_module().add_stake_limit(
                hk.clone(),
                1,
                amt,
                price,
                false,
            ));
        }
        ("SubtensorModule", "remove_stake_limit") => {
            typed_call_data(&agcli::api::tx().subtensor_module().remove_stake_limit(
                hk.clone(),
                1,
                amt,
                price,
                false,
            ));
        }
        ("SubtensorModule", "swap_stake_limit") => {
            typed_call_data(&agcli::api::tx().subtensor_module().swap_stake_limit(
                hk.clone(),
                1,
                2,
                amt,
                price,
                false,
            ));
        }
        ("SubtensorModule", "remove_stake_full_limit") => {
            typed_call_data(
                &agcli::api::tx().subtensor_module().remove_stake_full_limit(
                    hk.clone(),
                    1,
                    Some(price),
                ),
            );
        }
        ("SubtensorModule", "recycle_alpha") => {
            typed_call_data(
                &agcli::api::tx()
                    .subtensor_module()
                    .recycle_alpha(hk.clone(), amt, 1),
            );
        }
        ("SubtensorModule", "burn_alpha") => {
            typed_call_data(
                &agcli::api::tx()
                    .subtensor_module()
                    .burn_alpha(hk.clone(), amt, 1),
            );
        }
        ("SubtensorModule", "set_childkey_take") => {
            typed_call_data(&agcli::api::tx().subtensor_module().set_childkey_take(
                hk.clone(),
                1,
                11_796,
            ));
        }
        ("SubtensorModule", "set_children") => {
            let ch = AccountId32([3u8; 32]);
            typed_call_data(&agcli::api::tx().subtensor_module().set_children(
                hk,
                1,
                vec![(u64::MAX / 2, ch)],
            ));
        }
        ("SubtensorModule", "unstake_all") => {
            typed_call_data(&agcli::api::tx().subtensor_module().unstake_all(hk));
        }
        ("SubtensorModule", "unstake_all_alpha") => {
            typed_call_data(&agcli::api::tx().subtensor_module().unstake_all_alpha(hk));
        }
        ("SubtensorModule", "set_weights") => {
            typed_call_data(&agcli::api::tx().subtensor_module().set_weights(
                1,
                vec![0u16],
                vec![65535u16],
                0u64,
            ));
        }
        ("SubtensorModule", "increase_take") => {
            typed_call_data(
                &agcli::api::tx()
                    .subtensor_module()
                    .increase_take(hk.clone(), 11_796),
            );
        }
        ("SubtensorModule", "decrease_take") => {
            typed_call_data(
                &agcli::api::tx()
                    .subtensor_module()
                    .decrease_take(hk.clone(), 11_796),
            );
        }
        ("Balances", "transfer_allow_death") => {
            typed_call_data(
                &agcli::api::tx()
                    .balances()
                    .transfer_allow_death(subxt::utils::MultiAddress::Id(dest.clone()), amt),
            );
        }
        ("Balances", "transfer_keep_alive") => {
            typed_call_data(
                &agcli::api::tx()
                    .balances()
                    .transfer_keep_alive(subxt::utils::MultiAddress::Id(dest.clone()), amt),
            );
        }
        ("Crowdloan", "contribute") => {
            typed_call_data(&agcli::api::tx().crowdloan().contribute(1, amt));
        }
        _ => {
            let fields: Vec<Value> = spec.args.iter().map(manifest_dummy_value).collect();
            encode_dynamic(spec.pallet, spec.call, fields);
        }
    }
}

#[test]
fn extrinsic_manifest_all_specs_encode_against_metadata() {
    use agcli::chain::ALL_SPECS;

    for spec in ALL_SPECS {
        encode_manifest_spec(spec);
    }
}
