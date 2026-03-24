use crate::common::*;

#[test]
fn evm_address_error_includes_tip() {
    let err = validate_evm_address("", "source").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

#[test]
fn evm_address_unicode() {
    let err = validate_evm_address("0x123こんにちは", "test").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "got: {}", err);
}

// =====================================================================
// validate_hex_data tests
// =====================================================================

#[test]
fn hex_data_valid_empty_0x() {
    assert!(validate_hex_data("0x", "test").is_ok());
}

#[test]
fn hex_data_valid_short() {
    assert!(validate_hex_data("0xdeadbeef", "test").is_ok());
}

#[test]
fn hex_data_valid_long() {
    assert!(validate_hex_data(
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "test"
    )
    .is_ok());
}

#[test]
fn hex_data_valid_no_prefix() {
    assert!(validate_hex_data("cafebabe", "test").is_ok());
}

#[test]
fn hex_data_empty_string() {
    let err = validate_hex_data("", "code-hash").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn hex_data_odd_length() {
    let err = validate_hex_data("0xabc", "data").unwrap_err();
    assert!(err.to_string().contains("odd length"), "got: {}", err);
}

#[test]
fn hex_data_invalid_chars() {
    let err = validate_hex_data("0xnothex", "salt").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "got: {}", err);
}

#[test]
fn hex_data_spaces_only() {
    let err = validate_hex_data("   ", "test").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn hex_data_0_x_prefix() {
    assert!(validate_hex_data("0Xabcd", "test").is_ok());
}

#[test]
fn hex_data_single_byte() {
    assert!(validate_hex_data("0xff", "test").is_ok());
}

#[test]
fn hex_data_error_includes_tip() {
    let err = validate_hex_data("0xabc", "salt").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

// =====================================================================
// validate_pallet_call tests
// =====================================================================

#[test]
fn pallet_call_valid_pascal_case() {
    assert!(validate_pallet_call("System", "pallet").is_ok());
}

#[test]
fn pallet_call_valid_pascal_multi_word() {
    assert!(validate_pallet_call("SubtensorModule", "pallet").is_ok());
}

#[test]
fn pallet_call_valid_snake_case() {
    assert!(validate_pallet_call("remark", "call").is_ok());
}

#[test]
fn pallet_call_valid_snake_multi_word() {
    assert!(validate_pallet_call("transfer_keep_alive", "call").is_ok());
}

#[test]
fn pallet_call_valid_with_numbers() {
    assert!(validate_pallet_call("Erc20", "pallet").is_ok());
}

#[test]
fn pallet_call_empty() {
    let err = validate_pallet_call("", "pallet").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn pallet_call_spaces_only() {
    let err = validate_pallet_call("   ", "call").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn pallet_call_starts_with_number() {
    let err = validate_pallet_call("1System", "pallet").unwrap_err();
    assert!(
        err.to_string().contains("must start with a letter"),
        "got: {}",
        err
    );
}

#[test]
fn pallet_call_starts_with_underscore() {
    let err = validate_pallet_call("_private", "call").unwrap_err();
    assert!(
        err.to_string().contains("must start with a letter"),
        "got: {}",
        err
    );
}

#[test]
fn pallet_call_contains_dash() {
    let err = validate_pallet_call("my-pallet", "pallet").unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {}", err);
}

#[test]
fn pallet_call_contains_space() {
    let err = validate_pallet_call("my pallet", "pallet").unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {}", err);
}

#[test]
fn pallet_call_contains_dot() {
    let err = validate_pallet_call("System.remark", "pallet").unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {}", err);
}

#[test]
fn pallet_call_too_long() {
    let long = "A".repeat(129);
    let err = validate_pallet_call(&long, "pallet").unwrap_err();
    assert!(err.to_string().contains("too long"), "got: {}", err);
}

#[test]
fn pallet_call_exactly_128() {
    let ok = "A".repeat(128);
    assert!(validate_pallet_call(&ok, "pallet").is_ok());
}

#[test]
fn pallet_call_unicode() {
    let err = validate_pallet_call("Sÿstem", "pallet").unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {}", err);
}

#[test]
fn pallet_call_error_includes_tip() {
    let err = validate_pallet_call("", "pallet").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

// =====================================================================
// validate_schedule_id tests
// =====================================================================

#[test]
fn schedule_id_valid_short() {
    assert!(validate_schedule_id("my_task").is_ok());
}

#[test]
fn schedule_id_valid_32_bytes() {
    assert!(validate_schedule_id(&"x".repeat(32)).is_ok());
}

#[test]
fn schedule_id_empty() {
    let err = validate_schedule_id("").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn schedule_id_too_long() {
    let err = validate_schedule_id(&"a".repeat(33)).unwrap_err();
    assert!(err.to_string().contains("too long"), "got: {}", err);
}

#[test]
fn schedule_id_single_char() {
    assert!(validate_schedule_id("x").is_ok());
}

#[test]
fn schedule_id_with_special_chars() {
    // The chain interprets id as bytes; any non-empty ≤32 is valid
    assert!(validate_schedule_id("my-task-#1!").is_ok());
}

#[test]
fn schedule_id_error_includes_tip() {
    let err = validate_schedule_id("").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

// =====================================================================
// validate_crowdloan_amount tests
// =====================================================================

use agcli::cli::helpers::validate_crowdloan_amount;

#[test]
fn crowdloan_amount_valid_small() {
    assert!(validate_crowdloan_amount(0.001, "deposit").is_ok());
}

#[test]
fn crowdloan_amount_valid_large() {
    assert!(validate_crowdloan_amount(1_000_000.0, "cap").is_ok());
}

#[test]
fn crowdloan_amount_valid_one() {
    assert!(validate_crowdloan_amount(1.0, "contribution amount").is_ok());
}

#[test]
fn crowdloan_amount_zero_rejected() {
    let err = validate_crowdloan_amount(0.0, "deposit").unwrap_err();
    assert!(
        err.to_string().contains("greater than zero"),
        "got: {}",
        err
    );
}

#[test]
fn crowdloan_amount_negative_rejected() {
    let err = validate_crowdloan_amount(-1.0, "cap").unwrap_err();
    assert!(err.to_string().contains("negative"), "got: {}", err);
}

#[test]
fn crowdloan_amount_nan_rejected() {
    let err = validate_crowdloan_amount(f64::NAN, "deposit").unwrap_err();
    assert!(err.to_string().contains("finite"), "got: {}", err);
}

#[test]
fn crowdloan_amount_inf_rejected() {
    let err = validate_crowdloan_amount(f64::INFINITY, "cap").unwrap_err();
    assert!(err.to_string().contains("finite"), "got: {}", err);
}

#[test]
fn crowdloan_amount_neg_inf_rejected() {
    let err = validate_crowdloan_amount(f64::NEG_INFINITY, "cap").unwrap_err();
    assert!(err.to_string().contains("finite"), "got: {}", err);
}

#[test]
fn crowdloan_amount_tiny_valid() {
    // Smallest representable positive value — should pass
    assert!(validate_crowdloan_amount(f64::MIN_POSITIVE, "deposit").is_ok());
}

#[test]
fn crowdloan_amount_error_includes_tip() {
    let err = validate_crowdloan_amount(0.0, "deposit").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

#[test]
fn crowdloan_amount_negative_error_includes_tip() {
    let err = validate_crowdloan_amount(-5.0, "cap").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

// =====================================================================
// validate_price tests
// =====================================================================

use agcli::cli::helpers::validate_price;

#[test]
fn price_valid_small() {
    assert!(validate_price(0.001, "price-low").is_ok());
}

#[test]
fn price_valid_large() {
    assert!(validate_price(1_000_000.0, "price-high").is_ok());
}

#[test]
fn price_valid_one() {
    assert!(validate_price(1.0, "price-low").is_ok());
}

#[test]
fn price_zero_rejected() {
    let err = validate_price(0.0, "price-low").unwrap_err();
    assert!(err.to_string().contains("positive"), "got: {}", err);
}

#[test]
fn price_negative_rejected() {
    let err = validate_price(-0.5, "price-high").unwrap_err();
    assert!(err.to_string().contains("positive"), "got: {}", err);
}

#[test]
fn price_nan_rejected() {
    let err = validate_price(f64::NAN, "price-low").unwrap_err();
    assert!(err.to_string().contains("finite"), "got: {}", err);
}

#[test]
fn price_inf_rejected() {
    let err = validate_price(f64::INFINITY, "price-high").unwrap_err();
    assert!(err.to_string().contains("finite"), "got: {}", err);
}

#[test]
fn price_neg_inf_rejected() {
    let err = validate_price(f64::NEG_INFINITY, "price-low").unwrap_err();
    assert!(err.to_string().contains("finite"), "got: {}", err);
}

#[test]
fn price_tiny_valid() {
    assert!(validate_price(f64::MIN_POSITIVE, "price-low").is_ok());
}

#[test]
fn price_error_includes_tip() {
    let err = validate_price(0.0, "price-low").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

// =====================================================================
// validate_commitment_data tests
// =====================================================================

use agcli::cli::helpers::validate_commitment_data;

#[test]
fn commitment_data_valid_simple() {
    assert!(validate_commitment_data("endpoint:http://localhost:8080").is_ok());
}

#[test]
fn commitment_data_valid_multi() {
    assert!(validate_commitment_data("endpoint:http://my.server,version:1.0,type:miner").is_ok());
}

#[test]
fn commitment_data_empty_rejected() {
    let err = validate_commitment_data("").unwrap_err();
    assert!(err.to_string().contains("empty"), "got: {}", err);
}

#[test]
fn commitment_data_whitespace_only_rejected() {
    let err = validate_commitment_data("   ").unwrap_err();
    assert!(err.to_string().contains("empty"), "got: {}", err);
}

#[test]
fn commitment_data_too_long_rejected() {
    let long = "x".repeat(1025);
    let err = validate_commitment_data(&long).unwrap_err();
    assert!(err.to_string().contains("too long"), "got: {}", err);
}

#[test]
fn commitment_data_exactly_1024_ok() {
    let data = "x".repeat(1024);
    assert!(validate_commitment_data(&data).is_ok());
}

#[test]
fn commitment_data_single_char_valid() {
    assert!(validate_commitment_data("a").is_ok());
}

#[test]
fn commitment_data_error_includes_tip() {
    let err = validate_commitment_data("").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

// =====================================================================
// validate_event_filter tests
// =====================================================================

use agcli::cli::helpers::validate_event_filter;

#[test]
fn event_filter_all_valid() {
    assert!(validate_event_filter("all").is_ok());
}

#[test]
fn event_filter_staking_valid() {
    assert!(validate_event_filter("staking").is_ok());
}

#[test]
fn event_filter_registration_valid() {
    assert!(validate_event_filter("registration").is_ok());
}

#[test]
fn event_filter_transfer_valid() {
    assert!(validate_event_filter("transfer").is_ok());
}

#[test]
fn event_filter_weights_valid() {
    assert!(validate_event_filter("weights").is_ok());
}

#[test]
fn event_filter_subnet_valid() {
    assert!(validate_event_filter("subnet").is_ok());
}

#[test]
fn event_filter_case_insensitive() {
    assert!(validate_event_filter("ALL").is_ok());
    assert!(validate_event_filter("Staking").is_ok());
    assert!(validate_event_filter("TRANSFER").is_ok());
}

#[test]
fn event_filter_invalid_rejected() {
    let err = validate_event_filter("blocks").unwrap_err();
    assert!(err.to_string().contains("Valid filters"), "got: {}", err);
}

#[test]
fn event_filter_empty_rejected() {
    let err = validate_event_filter("").unwrap_err();
    assert!(err.to_string().contains("Valid filters"), "got: {}", err);
}

#[test]
fn event_filter_nonsense_rejected() {
    let err = validate_event_filter("foobar").unwrap_err();
    assert!(
        err.to_string().contains("Invalid event filter"),
        "got: {}",
        err
    );
}

#[test]
fn event_filter_with_spaces() {
    // Leading/trailing spaces should be trimmed
    assert!(validate_event_filter("  all  ").is_ok());
}

#[test]
fn event_filter_error_includes_tip() {
    let err = validate_event_filter("bad").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

// =====================================================================
// validate_wasm_file
// =====================================================================

#[test]
fn wasm_file_valid_minimal() {
    // Minimal valid WASM: magic + version + empty sections
    let mut data = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    data.extend_from_slice(&[0u8; 100]); // padding to make it realistic
    assert!(validate_wasm_file(&data, "test.wasm").is_ok());
}

#[test]
fn wasm_file_empty() {
    let err = validate_wasm_file(&[], "empty.wasm").unwrap_err();
    assert!(err.to_string().contains("empty"), "got: {}", err);
}

#[test]
fn wasm_file_too_small() {
    let data = vec![0x00, 0x61, 0x73];
    let err = validate_wasm_file(&data, "tiny.wasm").unwrap_err();
    assert!(err.to_string().contains("too small"), "got: {}", err);
}

#[test]
fn wasm_file_bad_magic() {
    let data = vec![0x7f, 0x45, 0x4c, 0x46, 0x01, 0x00, 0x00, 0x00]; // ELF magic
    let err = validate_wasm_file(&data, "not.wasm").unwrap_err();
    assert!(
        err.to_string().contains("not a WASM module"),
        "got: {}",
        err
    );
}

#[test]
fn wasm_file_pdf_magic() {
    let mut data = b"%PDF-1.4 ".to_vec();
    data.extend_from_slice(&[0u8; 100]);
    let err = validate_wasm_file(&data, "doc.pdf").unwrap_err();
    assert!(
        err.to_string().contains("not a WASM module"),
        "got: {}",
        err
    );
}

#[test]
fn wasm_file_too_large() {
    // Build just the header for a huge file
    let mut data = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    data.resize(16 * 1024 * 1024 + 1, 0x00); // 16MB + 1
    let err = validate_wasm_file(&data, "huge.wasm").unwrap_err();
    assert!(err.to_string().contains("too large"), "got: {}", err);
}

#[test]
fn wasm_file_exactly_max_size() {
    let mut data = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    data.resize(16 * 1024 * 1024, 0x00); // exactly 16MB
    assert!(validate_wasm_file(&data, "max.wasm").is_ok());
}

#[test]
fn wasm_file_error_includes_tip() {
    let data = vec![0xff; 100];
    let err = validate_wasm_file(&data, "bad.wasm").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

#[test]
fn wasm_file_error_shows_filename() {
    let err = validate_wasm_file(&[], "my_contract.wasm").unwrap_err();
    assert!(
        err.to_string().contains("my_contract.wasm"),
        "error should include filename: {}",
        err
    );
}

#[test]
fn wasm_file_json_bytes() {
    // Someone passes a JSON file instead of WASM
    let data = b"[{\"pallet\":\"Test\"}]";
    let err = validate_wasm_file(data, "calls.json").unwrap_err();
    assert!(
        err.to_string().contains("not a WASM module"),
        "got: {}",
        err
    );
}

#[test]
fn wasm_file_7_bytes() {
    // Edge case: exactly 7 bytes, under minimum 8
    let data = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00];
    let err = validate_wasm_file(&data, "short.wasm").unwrap_err();
    assert!(err.to_string().contains("too small"), "got: {}", err);
}

#[test]
fn wasm_file_8_bytes_valid() {
    // Exactly 8 bytes with valid magic
    let data = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    assert!(validate_wasm_file(&data, "min.wasm").is_ok());
}

// =====================================================================
// validate_gas_limit
// =====================================================================

#[test]
fn gas_limit_valid_21000() {
    assert!(validate_gas_limit(21000, "gas limit").is_ok());
}

#[test]
fn gas_limit_valid_1() {
    assert!(validate_gas_limit(1, "gas limit").is_ok());
}

#[test]
fn gas_limit_valid_max_u64() {
    assert!(validate_gas_limit(u64::MAX, "gas limit").is_ok());
}

#[test]
fn gas_limit_zero_rejected() {
    let err = validate_gas_limit(0, "gas limit").unwrap_err();
    assert!(err.to_string().contains("cannot be zero"), "got: {}", err);
}

#[test]
fn gas_limit_zero_error_includes_tip() {
    let err = validate_gas_limit(0, "gas limit").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

#[test]
fn gas_limit_error_includes_label() {
    let err = validate_gas_limit(0, "my gas").unwrap_err();
    assert!(
        err.to_string().contains("my gas"),
        "error should include label: {}",
        err
    );
}

// =====================================================================
// validate_batch_file
// =====================================================================

#[test]
fn batch_file_valid_single_call() {
    let json = r#"[{"pallet":"Balances","call":"transfer_allow_death","args":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",1000000000]}]"#;
    let calls = validate_batch_file(json, "test.json").unwrap();
    assert_eq!(calls.len(), 1);
}

#[test]
fn batch_file_valid_multi_call() {
    let json = r#"[
        {"pallet":"Balances","call":"transfer_allow_death","args":["addr1",100]},
        {"pallet":"SubtensorModule","call":"add_stake","args":["hk",1,100]}
    ]"#;
    let calls = validate_batch_file(json, "test.json").unwrap();
    assert_eq!(calls.len(), 2);
}

#[test]
fn batch_file_valid_empty_args() {
    let json = r#"[{"pallet":"System","call":"remark","args":[]}]"#;
    assert!(validate_batch_file(json, "test.json").is_ok());
}

#[test]
fn batch_file_empty_array() {
    let json = "[]";
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("empty"), "got: {}", err);
}

#[test]
fn batch_file_not_array_object() {
    let json = r#"{"pallet":"Balances","call":"transfer","args":[]}"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("JSON array"), "got: {}", err);
    assert!(err.to_string().contains("forget to wrap"), "got: {}", err);
}

#[test]
fn batch_file_not_array_string() {
    let json = r#""hello""#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("JSON array"), "got: {}", err);
}

#[test]
fn batch_file_not_array_number() {
    let json = "42";
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("JSON array"), "got: {}", err);
}

#[test]
fn batch_file_not_array_null() {
    let json = "null";
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("JSON array"), "got: {}", err);
}

#[test]
fn batch_file_not_array_bool() {
    let json = "true";
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("JSON array"), "got: {}", err);
}

#[test]
fn batch_file_invalid_json() {
    let json = "{invalid json";
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("Invalid JSON"), "got: {}", err);
}

#[test]
fn batch_file_missing_pallet() {
    let json = r#"[{"call":"transfer","args":[]}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(
        err.to_string().contains("missing \"pallet\""),
        "got: {}",
        err
    );
}

#[test]
fn batch_file_missing_call() {
    let json = r#"[{"pallet":"Balances","args":[]}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("missing \"call\""), "got: {}", err);
}

#[test]
fn batch_file_missing_args() {
    let json = r#"[{"pallet":"Balances","call":"transfer"}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("missing \"args\""), "got: {}", err);
}

#[test]
fn batch_file_pallet_not_string() {
    let json = r#"[{"pallet":123,"call":"transfer","args":[]}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("pallet"), "got: {}", err);
}

#[test]
fn batch_file_call_not_string() {
    let json = r#"[{"pallet":"Balances","call":true,"args":[]}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("call"), "got: {}", err);
}

#[test]
fn batch_file_args_not_array() {
    let json = r#"[{"pallet":"Balances","call":"transfer","args":"bad"}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("args"), "got: {}", err);
}

#[test]
fn batch_file_element_not_object_null() {
    let json = r#"[null]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("not an object"), "got: {}", err);
}

#[test]
fn batch_file_element_not_object_string() {
    let json = r#"["hello"]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("not an object"), "got: {}", err);
}

#[test]
fn batch_file_element_not_object_number() {
    let json = r#"[42]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("not an object"), "got: {}", err);
}

#[test]
fn batch_file_element_not_object_array() {
    let json = r#"[[1,2,3]]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(err.to_string().contains("not an object"), "got: {}", err);
}

#[test]
fn batch_file_error_shows_index() {
    let json = r#"[{"pallet":"OK","call":"ok","args":[]},{"pallet":"Bad","args":[]}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(
        err.to_string().contains("#1"),
        "error should show index: {}",
        err
    );
}

#[test]
fn batch_file_error_includes_tip() {
    let json = r#"[{"pallet":"X","call":"y"}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    assert!(
        err.to_string().contains("Tip:"),
        "error should include Tip: {}",
        err
    );
}

#[test]
fn batch_file_error_shows_filename() {
    let json = "not-json";
    let err = validate_batch_file(json, "my_batch.json").unwrap_err();
    assert!(
        err.to_string().contains("my_batch.json"),
        "error should include filename: {}",
        err
    );
}

#[test]
fn batch_file_too_many_calls() {
    // Build JSON array with 1001 valid calls
    let call = r#"{"pallet":"System","call":"remark","args":[]}"#;
    let calls: Vec<&str> = (0..1001).map(|_| call).collect();
    let json = format!("[{}]", calls.join(","));
    let err = validate_batch_file(&json, "huge.json").unwrap_err();
    assert!(err.to_string().contains("too many calls"), "got: {}", err);
}

#[test]
fn batch_file_exactly_1000_calls() {
    let call = r#"{"pallet":"System","call":"remark","args":[]}"#;
    let calls: Vec<&str> = (0..1000).map(|_| call).collect();
    let json = format!("[{}]", calls.join(","));
    let result = validate_batch_file(&json, "max.json");
    assert!(
        result.is_ok(),
        "1000 calls should be ok: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().len(), 1000);
}

#[test]
fn batch_file_mixed_valid_invalid() {
    let json = r#"[{"pallet":"OK","call":"ok","args":[]},{"not":"a call"}]"#;
    let err = validate_batch_file(json, "test.json").unwrap_err();
    // Second element should fail for missing pallet
    assert!(err.to_string().contains("#1"), "got: {}", err);
}

#[test]
fn batch_file_extra_fields_ok() {
    // Extra fields besides pallet/call/args should be tolerated
    let json =
        r#"[{"pallet":"System","call":"remark","args":[],"comment":"my note","priority":1}]"#;
    assert!(validate_batch_file(json, "test.json").is_ok());
}

// =====================================================================
// validate_weight_input()
// =====================================================================

#[test]
fn weight_input_valid_pairs() {
    assert!(validate_weight_input("0:100,1:200").is_ok());
}

#[test]
fn weight_input_single_pair() {
    assert!(validate_weight_input("0:100").is_ok());
}

#[test]
fn weight_input_with_spaces() {
    assert!(validate_weight_input("  0:100 , 1:200  ").is_ok());
}

#[test]
fn weight_input_stdin() {
    assert!(validate_weight_input("-").is_ok());
}

#[test]
fn weight_input_file_ref() {
    assert!(validate_weight_input("@weights.json").is_ok());
}

#[test]
fn weight_input_json_array() {
    assert!(validate_weight_input(r#"[{"uid":0,"weight":100}]"#).is_ok());
}

#[test]
fn weight_input_json_object() {
    assert!(validate_weight_input(r#"{"0":100}"#).is_ok());
}

#[test]
fn weight_input_empty() {
    let err = validate_weight_input("").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn weight_input_whitespace_only() {
    let err = validate_weight_input("   ").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn weight_input_missing_colon() {
    let err = validate_weight_input("0100").unwrap_err();
    assert!(err.to_string().contains("missing ':'"), "got: {}", err);
}

#[test]
fn weight_input_trailing_comma() {
    let err = validate_weight_input("0:100,").unwrap_err();
    assert!(
        err.to_string().contains("Empty weight pair"),
        "got: {}",
        err
    );
}

#[test]
fn weight_input_double_colon() {
    let err = validate_weight_input("0:1:2").unwrap_err();
    assert!(err.to_string().contains("exactly one ':'"), "got: {}", err);
}

#[test]
fn weight_input_leading_comma() {
    let err = validate_weight_input(",0:100").unwrap_err();
    assert!(
        err.to_string().contains("Empty weight pair"),
        "got: {}",
        err
    );
}

#[test]
fn weight_input_middle_empty() {
    let err = validate_weight_input("0:100,,1:200").unwrap_err();
    assert!(
        err.to_string().contains("Empty weight pair"),
        "got: {}",
        err
    );
}

#[test]
fn weight_input_no_value() {
    // "0:" has a colon but no value — this passes pre-validation, parse_weight_pairs catches it
    assert!(validate_weight_input("0:").is_ok());
}

// =====================================================================
// validate_view_limit()
// =====================================================================

#[test]
fn view_limit_valid() {
    assert!(validate_view_limit(1, "test").is_ok());
    assert!(validate_view_limit(50, "test").is_ok());
    assert!(validate_view_limit(10_000, "test").is_ok());
}

#[test]
fn view_limit_zero() {
    let err = validate_view_limit(0, "test").unwrap_err();
    assert!(err.to_string().contains("at least 1"), "got: {}", err);
}

#[test]
fn view_limit_too_large() {
    let err = validate_view_limit(10_001, "test").unwrap_err();
    assert!(err.to_string().contains("too large"), "got: {}", err);
}

#[test]
fn view_limit_max_boundary() {
    assert!(validate_view_limit(10_000, "test").is_ok());
    assert!(validate_view_limit(10_001, "test").is_err());
}

#[test]
fn view_limit_label_in_error() {
    let err = validate_view_limit(0, "validators --limit").unwrap_err();
    assert!(
        err.to_string().contains("validators --limit"),
        "got: {}",
        err
    );
}

#[test]
fn view_limit_huge() {
    let err = validate_view_limit(usize::MAX, "test").unwrap_err();
    assert!(err.to_string().contains("too large"), "got: {}", err);
}

// =====================================================================
// validate_admin_call_name()
// =====================================================================

#[test]
fn admin_call_valid_names() {
    // Only known AdminUtils call names (from admin::known_params()) are accepted
    assert!(validate_admin_call_name("sudo_set_tempo").is_ok());
    assert!(validate_admin_call_name("sudo_set_max_allowed_validators").is_ok());
    assert!(validate_admin_call_name("sudo_set_difficulty").is_ok());
}

#[test]
fn admin_call_empty() {
    let err = validate_admin_call_name("").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn admin_call_whitespace_only() {
    let err = validate_admin_call_name("   ").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn admin_call_starts_with_number() {
    let err = validate_admin_call_name("1set_tempo").unwrap_err();
    assert!(
        err.to_string().contains("must start with a letter"),
        "got: {}",
        err
    );
}

#[test]
fn admin_call_starts_with_underscore() {
    let err = validate_admin_call_name("_hidden").unwrap_err();
    assert!(
        err.to_string().contains("must start with a letter"),
        "got: {}",
        err
    );
}

#[test]
fn admin_call_special_chars() {
    let err = validate_admin_call_name("sudo.set.tempo").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "got: {}",
        err
    );
}

#[test]
fn admin_call_spaces() {
    let err = validate_admin_call_name("sudo set tempo").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "got: {}",
        err
    );
}

#[test]
fn admin_call_too_long() {
    let long = "a".repeat(129);
    let err = validate_admin_call_name(&long).unwrap_err();
    assert!(err.to_string().contains("too long"), "got: {}", err);
}

#[test]
fn admin_call_exact_max_length() {
    // 128-char unknown name passes length check but is rejected as unknown
    let name = "a".repeat(128);
    let err = validate_admin_call_name(&name).unwrap_err();
    assert!(
        err.to_string().contains("Unknown admin call") || err.to_string().contains("too long"),
        "got: {}",
        err
    );
}

#[test]
fn admin_call_with_hyphen() {
    let err = validate_admin_call_name("sudo-set-tempo").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "got: {}",
        err
    );
}

#[test]
fn admin_call_unicode() {
    let err = validate_admin_call_name("südö_set").unwrap_err();
    assert!(
        err.to_string().contains("must start with a letter")
            || err.to_string().contains("invalid character"),
        "got: {}",
        err
    );
}

#[test]
fn admin_call_with_numbers_ok() {
    // Known call with numbers in name
    assert!(validate_admin_call_name("sudo_set_max_allowed_uids").is_ok());
}

#[test]
fn admin_call_tip_mentions_list() {
    let err = validate_admin_call_name("").unwrap_err();
    assert!(err.to_string().contains("agcli admin list"), "got: {}", err);
}

// =====================================================================
// parse_weight_pairs — extended edge cases
// =====================================================================

#[test]
fn parse_weight_pairs_max_uid() {
    let (uids, weights) = parse_weight_pairs("65535:100").unwrap();
    assert_eq!(uids, vec![65535]);
    assert_eq!(weights, vec![100]);
}

#[test]
fn parse_weight_pairs_zero_weight() {
    let (uids, weights) = parse_weight_pairs("0:0").unwrap();
    assert_eq!(uids, vec![0]);
    assert_eq!(weights, vec![0]);
}

#[test]
fn parse_weight_pairs_uid_overflow() {
    let err = parse_weight_pairs("65536:100").unwrap_err();
    assert!(err.to_string().contains("Invalid UID"), "got: {}", err);
}

#[test]
fn parse_weight_pairs_weight_overflow() {
    let err = parse_weight_pairs("0:65536").unwrap_err();
    assert!(err.to_string().contains("Invalid weight"), "got: {}", err);
}

#[test]
fn parse_weight_pairs_negative_uid_v2() {
    let err = parse_weight_pairs("-1:100").unwrap_err();
    assert!(err.to_string().contains("Invalid UID"), "got: {}", err);
}

#[test]
fn parse_weight_pairs_negative_weight_v2() {
    let err = parse_weight_pairs("0:-100").unwrap_err();
    assert!(err.to_string().contains("Invalid weight"), "got: {}", err);
}

#[test]
fn parse_weight_pairs_float_uid() {
    let err = parse_weight_pairs("0.5:100").unwrap_err();
    assert!(err.to_string().contains("Invalid UID"), "got: {}", err);
}

#[test]
fn parse_weight_pairs_text_uid() {
    let err = parse_weight_pairs("abc:100").unwrap_err();
    assert!(err.to_string().contains("Invalid UID"), "got: {}", err);
}

#[test]
fn parse_weight_pairs_text_weight() {
    let err = parse_weight_pairs("0:abc").unwrap_err();
    assert!(err.to_string().contains("Invalid weight"), "got: {}", err);
}

#[test]
fn parse_weight_pairs_max_weight() {
    let (_, weights) = parse_weight_pairs("0:65535").unwrap();
    assert_eq!(weights, vec![65535]);
}

#[test]
fn parse_weight_pairs_many() {
    let pairs: Vec<String> = (0..100).map(|i| format!("{}:{}", i, i * 10)).collect();
    let input = pairs.join(",");
    let (uids, weights) = parse_weight_pairs(&input).unwrap();
    assert_eq!(uids.len(), 100);
    assert_eq!(weights.len(), 100);
    assert_eq!(uids[99], 99);
    assert_eq!(weights[99], 990);
}

#[test]
fn parse_weight_pairs_duplicate_uid() {
    // Duplicates are allowed at parse level (chain will handle)
    let (uids, _) = parse_weight_pairs("0:100,0:200").unwrap();
    assert_eq!(uids, vec![0, 0]);
}

#[test]
fn parse_weight_pairs_spaces_around_values() {
    let (uids, weights) = parse_weight_pairs(" 0 : 100 , 1 : 200 ").unwrap();
    assert_eq!(uids, vec![0, 1]);
    assert_eq!(weights, vec![100, 200]);
}

// ── validate_threads ──

#[test]
fn validate_threads_valid_one() {
    assert!(validate_threads(1, "POW").is_ok());
}

#[test]
fn validate_threads_valid_four() {
    assert!(validate_threads(4, "POW").is_ok());
}

#[test]
fn validate_threads_valid_max() {
    assert!(validate_threads(256, "POW").is_ok());
}

#[test]
fn validate_threads_zero() {
    let err = validate_threads(0, "POW").unwrap_err();
    assert!(err.to_string().contains("cannot be zero"), "err: {}", err);
}

#[test]
fn validate_threads_too_many() {
    let err = validate_threads(257, "POW").unwrap_err();
    assert!(err.to_string().contains("too high"), "err: {}", err);
    assert!(err.to_string().contains("max 256"), "err: {}", err);
}

#[test]
fn validate_threads_way_too_many() {
    let err = validate_threads(10000, "mining").unwrap_err();
    assert!(err.to_string().contains("mining"), "label shown: {}", err);
}

#[test]
fn validate_threads_boundary_255() {
    assert!(validate_threads(255, "t").is_ok());
}

#[test]
fn validate_threads_label_in_error() {
    let err = validate_threads(0, "custom-label").unwrap_err();
    assert!(err.to_string().contains("custom-label"), "label: {}", err);
}

// ── validate_url ──

#[test]
fn validate_url_valid_https() {
    assert!(validate_url("https://example.com", "test").is_ok());
}

#[test]
fn validate_url_valid_http() {
    assert!(validate_url("http://example.com/path?q=1", "test").is_ok());
}

#[test]
fn validate_url_valid_localhost() {
    assert!(validate_url("http://localhost:8080/api", "test").is_ok());
}

#[test]
fn validate_url_empty_ok() {
    assert!(validate_url("", "test").is_ok());
}

#[test]
fn validate_url_whitespace_empty_ok() {
    assert!(validate_url("   ", "test").is_ok());
}

#[test]
fn validate_url_missing_scheme() {
    let err = validate_url("example.com", "subnet URL").unwrap_err();
    assert!(err.to_string().contains("http://"), "err: {}", err);
    assert!(err.to_string().contains("https://"), "err: {}", err);
}

#[test]
fn validate_url_ftp_scheme_rejected() {
    let err = validate_url("ftp://files.example.com", "test").unwrap_err();
    assert!(err.to_string().contains("http://"), "err: {}", err);
}

#[test]
fn validate_url_missing_host() {
    let err = validate_url("https://", "test").unwrap_err();
    assert!(err.to_string().contains("missing a host"), "err: {}", err);
}

#[test]
fn validate_url_missing_host_with_path() {
    let err = validate_url("https:///path", "test").unwrap_err();
    assert!(err.to_string().contains("missing a host"), "err: {}", err);
}

#[test]
fn validate_url_too_long() {
    let long_url = format!("https://example.com/{}", "a".repeat(2040));
    let err = validate_url(&long_url, "test").unwrap_err();
    assert!(err.to_string().contains("too long"), "err: {}", err);
    assert!(err.to_string().contains("max 2048"), "err: {}", err);
}

#[test]
fn validate_url_label_in_error() {
    let err = validate_url("badurl", "my-field").unwrap_err();
    assert!(err.to_string().contains("my-field"), "label: {}", err);
}

#[test]
fn validate_url_http_missing_host_query() {
    let err = validate_url("http://?query", "test").unwrap_err();
    assert!(err.to_string().contains("missing a host"), "err: {}", err);
}

#[test]
fn validate_url_valid_with_port() {
    assert!(validate_url("https://example.com:443/path", "test").is_ok());
}

#[test]
fn validate_url_valid_ip_address() {
    assert!(validate_url("http://192.168.1.1:9944", "test").is_ok());
}

// ── validate_subnet_name ──

#[test]
fn validate_subnet_name_valid_simple() {
    assert!(validate_subnet_name("MySubnet", "name").is_ok());
}

#[test]
fn validate_subnet_name_valid_with_spaces() {
    assert!(validate_subnet_name("My Cool Subnet", "name").is_ok());
}

#[test]
fn validate_subnet_name_valid_single_char() {
    assert!(validate_subnet_name("A", "name").is_ok());
}

#[test]
fn validate_subnet_name_valid_max_length() {
    let name = "a".repeat(256);
    assert!(validate_subnet_name(&name, "name").is_ok());
}

