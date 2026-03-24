use crate::common::*;

#[test]
fn validate_ipv4_rejects_loopback_range() {
    assert!(validate_ipv4("127.0.0.1").is_err());
    assert!(validate_ipv4("127.255.255.254").is_err());
    assert!(validate_ipv4("127.1.2.3").is_err());
}

#[test]
fn validate_ipv4_warns_private_but_allows() {
    // Private ranges should succeed (just warn to stderr)
    assert!(validate_ipv4("10.0.0.1").is_ok());
    assert!(validate_ipv4("10.255.255.255").is_ok());
    assert!(validate_ipv4("172.16.0.1").is_ok());
    assert!(validate_ipv4("172.31.255.255").is_ok());
    assert!(validate_ipv4("192.168.0.1").is_ok());
    assert!(validate_ipv4("192.168.255.255").is_ok());
}

#[test]
fn validate_ipv4_rejects_too_few_octets() {
    assert!(validate_ipv4("1.2.3").is_err());
    assert!(validate_ipv4("1.2").is_err());
    assert!(validate_ipv4("1").is_err());
}

#[test]
fn validate_ipv4_rejects_too_many_octets() {
    assert!(validate_ipv4("1.2.3.4.5").is_err());
}

#[test]
fn validate_ipv4_rejects_octet_overflow() {
    assert!(validate_ipv4("256.0.0.1").is_err());
    assert!(validate_ipv4("1.2.3.999").is_err());
}

#[test]
fn validate_ipv4_rejects_non_numeric() {
    assert!(validate_ipv4("abc.def.ghi.jkl").is_err());
    assert!(validate_ipv4("1.2.3.x").is_err());
}

#[test]
fn validate_ipv4_rejects_empty() {
    assert!(validate_ipv4("").is_err());
}

#[test]
fn validate_ipv4_rejects_leading_zeros() {
    let result = validate_ipv4("01.02.03.04");
    assert!(result.is_err(), "leading zeros should fail");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("leading zeros"), "msg: {}", msg);
}

#[test]
fn validate_ipv4_rejects_negative() {
    assert!(validate_ipv4("-1.0.0.1").is_err());
}

#[test]
fn validate_ipv4_rejects_spaces() {
    assert!(validate_ipv4("1.2.3. 4").is_err());
    assert!(validate_ipv4(" 1.2.3.4").is_err());
}

#[test]
fn validate_ipv4_rejects_hostname() {
    assert!(validate_ipv4("example.com").is_err());
    assert!(validate_ipv4("localhost").is_err());
}

// ── validate_ss58 ──

use agcli::cli::helpers::validate_ss58;

#[test]
fn validate_ss58_valid_alice() {
    assert!(validate_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test").is_ok());
}

#[test]
fn validate_ss58_valid_bob() {
    assert!(validate_ss58("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty", "test").is_ok());
}

#[test]
fn validate_ss58_empty_rejects() {
    let err = validate_ss58("", "destination").unwrap_err().to_string();
    assert!(err.contains("empty"), "msg: {}", err);
    assert!(err.contains("destination"), "should include label: {}", err);
}

#[test]
fn validate_ss58_whitespace_only_rejects() {
    assert!(validate_ss58("   ", "dest").is_err());
}

#[test]
fn validate_ss58_too_short_rejects() {
    let err = validate_ss58("5Grw", "hotkey").unwrap_err().to_string();
    assert!(err.contains("too short"), "msg: {}", err);
}

#[test]
fn validate_ss58_too_long_rejects() {
    let long = "5".to_string() + &"a".repeat(60);
    let err = validate_ss58(&long, "test").unwrap_err().to_string();
    assert!(err.contains("too long"), "msg: {}", err);
}

#[test]
fn validate_ss58_ethereum_address_rejects() {
    let err = validate_ss58("0x742d35Cc6634C0532925a3b844Bc9e7595f2bD18", "destination")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Ethereum") || err.contains("hex"),
        "should detect 0x prefix: {}",
        err
    );
}

#[test]
fn validate_ss58_uppercase_0x_rejects() {
    let err = validate_ss58("0X742d35Cc6634C0532925a3b844Bc9e7595f2bD18", "test")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Ethereum") || err.contains("hex"),
        "msg: {}",
        err
    );
}

#[test]
fn validate_ss58_contains_spaces_rejects() {
    let err = validate_ss58("5Grwva EF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test")
        .unwrap_err()
        .to_string();
    assert!(err.contains("whitespace"), "msg: {}", err);
}

#[test]
fn validate_ss58_tabs_rejects() {
    assert!(validate_ss58("5Grwva\tEF5z", "test").is_err());
}

#[test]
fn validate_ss58_invalid_base58_chars_rejects() {
    // 'O' is not in Base58 (0, I, O, l are excluded)
    let err = validate_ss58("5GrwvaOF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Base58") || err.contains("'O'"),
        "msg: {}",
        err
    );
}

#[test]
fn validate_ss58_zero_char_rejects() {
    // '0' is not valid Base58
    let err = validate_ss58("50rwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Base58") || err.contains("'0'"),
        "msg: {}",
        err
    );
}

#[test]
fn validate_ss58_lowercase_l_rejects() {
    // 'l' is not valid Base58
    let err = validate_ss58("5GrwvalF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "test")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Base58") || err.contains("'l'"),
        "msg: {}",
        err
    );
}

#[test]
fn validate_ss58_bad_checksum_rejects() {
    // Change last char to invalidate checksum
    let err = validate_ss58("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQZ", "test")
        .unwrap_err()
        .to_string();
    assert!(err.contains("checksum"), "msg: {}", err);
}

#[test]
fn validate_ss58_random_string_rejects() {
    assert!(validate_ss58("notanaddressatall12345678901234567890123456", "test").is_err());
}

#[test]
fn validate_ss58_label_in_error() {
    let err = validate_ss58("", "my-delegate").unwrap_err().to_string();
    assert!(
        err.contains("my-delegate"),
        "error should include label: {}",
        err
    );
}

#[test]
fn validate_ss58_leading_trailing_whitespace_trimmed() {
    // Trimmed version of Alice should be valid
    assert!(validate_ss58(" 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY ", "test").is_ok());
}

// ── validate_password_strength ──

use agcli::cli::helpers::validate_password_strength;

#[test]
fn validate_password_strength_strong_no_panic() {
    validate_password_strength("Str0ng!Pass#2024").unwrap();
}

#[test]
fn validate_password_strength_short_no_panic() {
    // Short passwords are rejected (min 8 chars); function returns Err, does not panic
    let result = validate_password_strength("ab");
    assert!(result.is_err(), "short password should be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("short") || msg.contains("8"), "got: {}", msg);
}

#[test]
fn validate_password_strength_empty_rejects() {
    // Empty password must be rejected (Issue 729)
    let result = validate_password_strength("");
    assert!(result.is_err(), "Empty password should be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Empty password"),
        "Error should mention empty password, got: {}",
        msg
    );
}

#[test]
fn validate_password_strength_common_no_panic() {
    // Common or short passwords are rejected; function returns Err, does not panic
    for pw in ["password", "12345678", "qwerty"] {
        let result = validate_password_strength(pw);
        assert!(result.is_err(), "weak password {:?} should be rejected", pw);
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("commonly")
                || msg.contains("dictionary")
                || msg.contains("short")
                || msg.contains("8"),
            "got: {}",
            msg
        );
    }
}

#[test]
fn validate_password_strength_single_type_no_panic() {
    // Single character type: "abcdefgh" and "ABCDEFGH" pass (warning only); "12345678" is common -> rejected
    validate_password_strength("abcdefgh").unwrap();
    validate_password_strength("ABCDEFGH").unwrap();
    let result = validate_password_strength("12345678");
    assert!(result.is_err(), "12345678 is common and should be rejected");
}

#[test]
fn validate_password_strength_mixed_no_panic() {
    // Mixed but too short (4 chars); rejected, function returns Err
    let result = validate_password_strength("aB1!");
    assert!(result.is_err(), "4-char password should be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("short") || msg.contains("8"), "got: {}", msg);
}

// ── validate_port ──

use agcli::cli::helpers::validate_port;

#[test]
fn validate_port_normal_ok() {
    assert!(validate_port(8091, "axon").is_ok());
    assert!(validate_port(443, "https").is_ok());
    assert!(validate_port(65535, "max").is_ok());
    assert!(validate_port(1024, "user").is_ok());
}

#[test]
fn validate_port_zero_rejects() {
    let err = validate_port(0, "axon").unwrap_err().to_string();
    assert!(err.contains("0"), "msg: {}", err);
    assert!(err.contains("axon"), "should include label: {}", err);
}

#[test]
fn validate_port_privileged_warns_but_ok() {
    // Ports < 1024 should succeed but print a warning
    assert!(validate_port(80, "http").is_ok());
    assert!(validate_port(1, "min").is_ok());
    assert!(validate_port(22, "ssh").is_ok());
}

#[test]
fn validate_port_one_ok() {
    assert!(validate_port(1, "test").is_ok());
}

// ── validate_netuid ──

#[test]
fn validate_netuid_normal_ok() {
    assert!(validate_netuid(1).is_ok());
    assert!(validate_netuid(100).is_ok());
    assert!(validate_netuid(65535).is_ok());
}

#[test]
fn validate_netuid_zero_rejects() {
    let err = validate_netuid(0).unwrap_err().to_string();
    assert!(err.contains("0") || err.contains("Root"), "msg: {}", err);
}

// ── validate_batch_axon_json ──

use agcli::cli::helpers::validate_batch_axon_json;

#[test]
fn validate_batch_axon_json_valid_single() {
    let json = r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 8091}]"#;
    let entries = validate_batch_axon_json(json).unwrap();
    assert_eq!(entries.len(), 1);
}

#[test]
fn validate_batch_axon_json_valid_multiple() {
    let json = r#"[
        {"netuid": 1, "ip": "1.2.3.4", "port": 8091},
        {"netuid": 2, "ip": "5.6.7.8", "port": 9092, "protocol": 4, "version": 1}
    ]"#;
    let entries = validate_batch_axon_json(json).unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn validate_batch_axon_json_valid_with_all_fields() {
    let json = r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 8091, "protocol": 6, "version": 42}]"#;
    assert!(validate_batch_axon_json(json).is_ok());
}

#[test]
fn validate_batch_axon_json_empty_array_rejects() {
    let err = validate_batch_axon_json("[]").unwrap_err().to_string();
    assert!(err.contains("empty"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_not_array_rejects() {
    assert!(validate_batch_axon_json(r#"{"netuid": 1}"#).is_err());
}

#[test]
fn validate_batch_axon_json_invalid_json_rejects() {
    let err = validate_batch_axon_json("not json")
        .unwrap_err()
        .to_string();
    assert!(err.contains("Invalid"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_missing_netuid_rejects() {
    let err = validate_batch_axon_json(r#"[{"ip": "1.2.3.4", "port": 8091}]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("netuid"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_missing_ip_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "port": 8091}]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("ip"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_missing_port_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "1.2.3.4"}]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("port"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_netuid_not_number_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": "one", "ip": "1.2.3.4", "port": 8091}]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("netuid"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_ip_not_string_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": 123, "port": 8091}]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("ip"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_port_zero_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 0}]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("port"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_port_too_large_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 70000}]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("port"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_netuid_too_large_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 100000, "ip": "1.2.3.4", "port": 8091}]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("netuid"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_protocol_overflow_rejects() {
    let err = validate_batch_axon_json(
        r#"[{"netuid": 1, "ip": "1.2.3.4", "port": 8091, "protocol": 256}]"#,
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("protocol"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_invalid_ip_rejects() {
    let err = validate_batch_axon_json(r#"[{"netuid": 1, "ip": "127.0.0.1", "port": 8091}]"#)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("loopback") || err.contains("IP") || err.contains("ip"),
        "msg: {}",
        err
    );
}

#[test]
fn validate_batch_axon_json_entry_not_object_rejects() {
    let err = validate_batch_axon_json(r#"[42]"#).unwrap_err().to_string();
    assert!(err.contains("not a JSON object"), "msg: {}", err);
}

#[test]
fn validate_batch_axon_json_string_entry_rejects() {
    let err = validate_batch_axon_json(r#"["hello"]"#)
        .unwrap_err()
        .to_string();
    assert!(err.contains("not a JSON object"), "msg: {}", err);
}

// ──── validate_mnemonic tests ────

#[test]
fn validate_mnemonic_valid_12_words() {
    // Generate a valid 12-word mnemonic using bip39 crate
    let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 16]).unwrap();
    assert!(validate_mnemonic(&mnemonic.to_string()).is_ok());
}

#[test]
fn validate_mnemonic_valid_24_words() {
    let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 32]).unwrap();
    let phrase = mnemonic.to_string();
    let words: Vec<&str> = phrase.split_whitespace().collect();
    assert_eq!(words.len(), 24);
    assert!(validate_mnemonic(&phrase).is_ok());
}

#[test]
fn validate_mnemonic_valid_15_words() {
    let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 20]).unwrap();
    let phrase = mnemonic.to_string();
    let words: Vec<&str> = phrase.split_whitespace().collect();
    assert_eq!(words.len(), 15);
    assert!(validate_mnemonic(&phrase).is_ok());
}

#[test]
fn validate_mnemonic_empty_rejects() {
    let err = validate_mnemonic("").unwrap_err().to_string();
    assert!(err.contains("empty"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_whitespace_only_rejects() {
    let err = validate_mnemonic("   \t  ").unwrap_err().to_string();
    assert!(err.contains("empty"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_wrong_word_count_rejects() {
    let err = validate_mnemonic("abandon abandon abandon")
        .unwrap_err()
        .to_string();
    assert!(err.contains("3 words"), "msg: {}", err);
    assert!(err.contains("12, 15, 18, 21, or 24"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_11_words_rejects() {
    let err = validate_mnemonic(
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon",
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("11 words"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_13_words_rejects() {
    let err = validate_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap_err().to_string();
    assert!(err.contains("13 words"), "msg: {}", err);
}

#[test]
fn validate_mnemonic_invalid_word_rejects() {
    // 12 words but one is not BIP-39
    let err = validate_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon xylophone").unwrap_err().to_string();
    assert!(
        err.contains("xylophone"),
        "should mention bad word: {}",
        err
    );
    assert!(err.contains("BIP-39"), "should mention BIP-39: {}", err);
}

#[test]
fn validate_mnemonic_bad_checksum_rejects() {
    // 12 valid BIP-39 words but wrong checksum
    let err = validate_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon").unwrap_err().to_string();
    assert!(err.contains("checksum"), "should mention checksum: {}", err);
}

#[test]
fn validate_mnemonic_extra_spaces_ok() {
    // Valid mnemonic with extra whitespace should be accepted
    let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 16]).unwrap();
    let phrase = mnemonic.to_string();
    let with_spaces = format!("  {}  ", phrase.replace(' ', "  "));
    assert!(
        validate_mnemonic(&with_spaces).is_ok(),
        "extra whitespace should be tolerated"
    );
}

#[test]
fn validate_mnemonic_misspelled_word_suggests() {
    // "abandn" is close to "abandon"
    let err = validate_mnemonic("abandn abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap_err().to_string();
    assert!(
        err.contains("abandn"),
        "should mention misspelled word: {}",
        err
    );
    assert!(err.contains("BIP-39"), "should mention BIP-39: {}", err);
}

#[test]
fn validate_mnemonic_numbers_reject() {
    let err = validate_mnemonic("1 2 3 4 5 6 7 8 9 10 11 12")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("BIP-39") || err.contains("not in"),
        "msg: {}",
        err
    );
}

#[test]
fn validate_mnemonic_single_word_rejects() {
    let err = validate_mnemonic("abandon").unwrap_err().to_string();
    assert!(
        err.contains("1 words") || err.contains("1 word"),
        "msg: {}",
        err
    );
}

#[test]
fn validate_mnemonic_passphrase_not_mnemonic() {
    // Common mistake: entering a password instead of mnemonic
    let err = validate_mnemonic("MySecretPassword123!")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("1 word") || err.contains("expected"),
        "msg: {}",
        err
    );
}

#[test]
fn validate_mnemonic_25_words_rejects() {
    // More than 24 words
    let mnemonic24 =
        bip39::Mnemonic::from_entropy_in(bip39::Language::English, &[0u8; 32]).unwrap();
    let phrase = format!("{} abandon", mnemonic24);
    let err = validate_mnemonic(&phrase).unwrap_err().to_string();
    assert!(err.contains("25 words"), "msg: {}", err);
}

// ──── Error message quality tests ────
// Verify that all user-facing errors contain actionable tips

#[test]
fn error_quality_validate_amount_zero_has_tip() {
    let err = validate_amount(0.0, "stake").unwrap_err().to_string();
    assert!(
        err.contains("Tip:"),
        "zero amount error should have tip: {}",
        err
    );
    assert!(err.contains("RAO"), "should mention RAO minimum: {}", err);
}

#[test]
fn error_quality_validate_name_empty_has_tip() {
    let err = validate_name("", "wallet").unwrap_err().to_string();
    assert!(
        err.contains("Tip:"),
        "empty name error should have tip: {}",
        err
    );
    assert!(
        err.contains("alphanumeric") || err.contains("mywallet"),
        "should suggest valid name: {}",
        err
    );
}

#[test]
fn error_quality_validate_name_path_traversal_has_tip() {
    let err = validate_name("../../../etc/passwd", "wallet")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Tip:"),
        "path traversal error should have tip: {}",
        err
    );
}

#[test]
fn error_quality_validate_ss58_empty_has_tip() {
    let err = agcli::cli::helpers::validate_ss58("", "destination")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Tip:"),
        "empty SS58 error should have tip: {}",
        err
    );
    assert!(err.contains("48"), "should mention address length: {}", err);
}

#[test]
fn error_quality_validate_ss58_ethereum_has_tip() {
    let err = agcli::cli::helpers::validate_ss58(
        "0x742d35Cc6634C0532925a3b844BcEfe0390a94e0",
        "destination",
    )
    .unwrap_err()
    .to_string();
    assert!(
        err.contains("Tip:"),
        "Ethereum address error should have tip: {}",
        err
    );
    assert!(
        err.contains("Ethereum") || err.contains("SS58"),
        "should explain format: {}",
        err
    );
}

#[test]
fn error_quality_validate_ipv4_loopback_has_tip() {
    let err = validate_ipv4("127.0.0.1").unwrap_err().to_string();
    assert!(
        err.contains("public"),
        "loopback error should suggest public IP: {}",
        err
    );
}

#[test]
fn error_quality_validate_take_over_max_has_tip() {
    let err = validate_take_pct(20.0).unwrap_err().to_string();
    assert!(
        err.contains("Tip:"),
        "over-max take error should have tip: {}",
        err
    );
    assert!(err.contains("18"), "should mention maximum: {}", err);
}

#[test]
fn error_quality_validate_port_zero_has_tip() {
    let err = agcli::cli::helpers::validate_port(0, "axon")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Tip:"),
        "port zero error should have tip: {}",
        err
    );
    assert!(
        err.contains("8091") || err.contains("443"),
        "should suggest common ports: {}",
        err
    );
}

#[test]
fn error_quality_validate_netuid_zero_has_tip() {
    let err = agcli::cli::helpers::validate_netuid(0)
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Tip:"),
        "netuid zero error should have tip: {}",
        err
    );
    assert!(
        err.contains("netuid 1"),
        "should mention subnets start at 1: {}",
        err
    );
}

#[test]
fn error_quality_validate_symbol_empty_has_tip() {
    let err = validate_symbol("").unwrap_err().to_string();
    assert!(
        err.contains("Tip:"),
        "empty symbol error should have tip: {}",
        err
    );
    assert!(
        err.contains("ALPHA") || err.contains("SN1"),
        "should suggest example: {}",
        err
    );
}

#[test]
fn error_quality_validate_mnemonic_wrong_count_has_tip() {
    let err = validate_mnemonic("abandon abandon abandon")
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("Tip:"),
        "wrong word count error should have tip: {}",
        err
    );
}

#[test]
fn error_quality_validate_mnemonic_bad_word_has_tip() {
    let err = validate_mnemonic("xylophone abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").unwrap_err().to_string();
    assert!(
        err.contains("BIP-39") || err.contains("dictionary"),
        "bad word error should mention BIP-39: {}",
        err
    );
}

#[test]
fn error_quality_parse_weight_invalid_has_format() {
    let err = parse_weight_pairs("bad").unwrap_err().to_string();
    assert!(
        err.contains("uid:weight") || err.contains("Format:"),
        "should show format: {}",
        err
    );
}

#[test]
fn error_quality_parse_children_invalid_has_format() {
    let err = parse_children("bad").unwrap_err().to_string();
    assert!(
        err.contains("proportion:hotkey") || err.contains("Format:"),
        "should show format: {}",
        err
    );
}

#[test]
fn error_quality_validate_max_cost_negative_mentions_value() {
    let err = validate_max_cost(-5.0).unwrap_err().to_string();
    assert!(err.contains("-5"), "should show the invalid value: {}", err);
    assert!(
        err.contains("negative"),
        "should explain why invalid: {}",
        err
    );
}

#[test]
fn error_quality_validate_delegate_take_over_max_has_tip() {
    let err = validate_delegate_take(25.0).unwrap_err().to_string();
    assert!(
        err.contains("Tip:"),
        "over-max delegate take should have tip: {}",
        err
    );
    assert!(err.contains("18"), "should mention maximum: {}", err);
}

// ──── Dry-run related parse tests ────
// These verify --dry-run flag is parseable across different command positions

#[test]
fn dry_run_flag_parses_with_transfer() {
    use clap::Parser;
    let args = vec![
        "agcli",
        "--dry-run",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "1.0",
    ];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run before subcommand: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_flag_parses_after_subcommand() {
    use clap::Parser;
    let args = vec![
        "agcli",
        "transfer",
        "--dry-run",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "1.0",
    ];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run after subcommand: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_flag_absent_means_false() {
    use clap::Parser;
    // When --dry-run is not passed and no env var, should be false
    // Note: clean env for this test
    let saved = std::env::var("AGCLI_DRY_RUN").ok();
    std::env::remove_var("AGCLI_DRY_RUN");
    let args = vec!["agcli", "balance"];
    let cli = agcli::cli::Cli::try_parse_from(args).unwrap();
    assert!(!cli.dry_run, "dry_run should be false when flag absent");
    // Restore
    if let Some(v) = saved {
        std::env::set_var("AGCLI_DRY_RUN", v);
    }
}

#[test]
fn dry_run_with_stake_add() {
    use clap::Parser;
    let args = vec![
        "agcli",
        "--dry-run",
        "stake",
        "add",
        "--netuid",
        "1",
        "--amount",
        "1.0",
    ];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with stake add: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_subnet_register() {
    use clap::Parser;
    // subnet register takes no args — it creates a new network
    let args = vec!["agcli", "--dry-run", "subnet", "register"];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "--dry-run with subnet register: {:?}",
        cli.err()
    );
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_weights_set() {
    use clap::Parser;
    let args = vec![
        "agcli",
        "--dry-run",
        "weights",
        "set",
        "--netuid",
        "1",
        "--weights",
        "0:100,1:200",
    ];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with weights set: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_delegate_increase() {
    use clap::Parser;
    let args = vec![
        "agcli",
        "--dry-run",
        "delegate",
        "increase-take",
        "--take",
        "10.0",
    ];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(
        cli.is_ok(),
        "--dry-run with delegate increase-take: {:?}",
        cli.err()
    );
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_proxy_add() {
    use clap::Parser;
    let args = vec![
        "agcli",
        "--dry-run",
        "proxy",
        "add",
        "--delegate",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with proxy add: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn dry_run_with_serve_axon() {
    use clap::Parser;
    let args = vec![
        "agcli",
        "--dry-run",
        "serve",
        "axon",
        "--netuid",
        "1",
        "--ip",
        "1.2.3.4",
        "--port",
        "8091",
    ];
    let cli = agcli::cli::Cli::try_parse_from(args);
    assert!(cli.is_ok(), "--dry-run with serve axon: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

// ──── parse_children enhanced edge cases ────

#[test]
fn parse_children_trailing_comma_ok() {
    let alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let result = parse_children(&format!("1000:{},", alice));
    assert!(
        result.is_ok(),
        "trailing comma should be tolerated: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().len(), 1);
}

// ──── validate_derive_input tests ────
#[test]
fn validate_derive_input_valid_hex_32_bytes() {
    let hex = "0x0000000000000000000000000000000000000000000000000000000000000001";
    assert!(validate_derive_input(hex).is_ok());
}

#[test]
fn validate_derive_input_valid_mnemonic() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    assert!(validate_derive_input(mnemonic).is_ok());
}

#[test]
fn validate_derive_input_empty() {
    let err = validate_derive_input("").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("cannot be empty"), "got: {}", msg);
    assert!(msg.contains("Tip:"), "should have tip: {}", msg);
}

#[test]
fn validate_derive_input_whitespace_only() {
    let err = validate_derive_input("   ").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"));
}

#[test]
fn validate_derive_input_hex_empty_after_prefix() {
    let err = validate_derive_input("0x").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("empty after"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_odd_length() {
    let err =
        validate_derive_input("0x012345678901234567890123456789012345678901234567890123456789012")
            .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("odd length"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_too_short() {
    let err = validate_derive_input("0x0123456789abcdef").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("32 bytes"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_too_long() {
    let err = validate_derive_input(
        "0x00000000000000000000000000000000000000000000000000000000000000000000",
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("32 bytes"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_invalid_chars() {
    let err =
        validate_derive_input("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG")
            .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Invalid hex character"), "got: {}", msg);
}

#[test]
fn validate_derive_input_hex_uppercase_0_x() {
    // 0X prefix should also be recognized as hex
    let err = validate_derive_input("0X0123").unwrap_err();
    // Should treat as hex path and reject for wrong length
    let msg = err.to_string();
    assert!(
        msg.contains("odd length") || msg.contains("32 bytes"),
        "got: {}",
        msg
    );
}

#[test]
fn validate_derive_input_hex_with_spaces() {
    // Hex with trailing spaces — trimmed first
    let hex = "0x0000000000000000000000000000000000000000000000000000000000000001  ";
    assert!(
        validate_derive_input(hex).is_ok(),
        "trailing spaces should be trimmed"
    );
}

#[test]
fn validate_derive_input_invalid_mnemonic() {
    // Something that's not hex but also not a valid mnemonic
    let err = validate_derive_input("not a valid mnemonic at all").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("mnemonic") || msg.contains("word"),
        "got: {}",
        msg
    );
}

// ──── validate_multisig_json_args tests ────

#[test]
fn validate_multisig_json_args_valid_simple() {
    let result = validate_multisig_json_args(r#"[1, "hello", true]"#);
    assert!(result.is_ok(), "simple array: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 3);
}

#[test]
fn validate_multisig_json_args_valid_hex_bytes() {
    let result = validate_multisig_json_args(r#"["0xdeadbeef", 42]"#);
    assert!(result.is_ok());
}

#[test]
fn validate_multisig_json_args_valid_nested_object() {
    let result = validate_multisig_json_args(
        r#"[{"Id": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"}, 1000]"#,
    );
    assert!(result.is_ok());
}

#[test]
fn validate_multisig_json_args_empty_string() {
    let err = validate_multisig_json_args("").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Empty JSON"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_not_json() {
    let err = validate_multisig_json_args("not json at all").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Invalid JSON"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_json_object_not_array() {
    let err = validate_multisig_json_args(r#"{"key": "value"}"#).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Expected a JSON array"), "got: {}", msg);
    assert!(
        msg.contains("object"),
        "should say it got an object: {}",
        msg
    );
}

#[test]
fn validate_multisig_json_args_json_string_not_array() {
    let err = validate_multisig_json_args(r#""just a string""#).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Expected a JSON array"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_json_number_not_array() {
    let err = validate_multisig_json_args("42").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Expected a JSON array"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_null_element() {
    let err = validate_multisig_json_args("[1, null, 3]").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("null"), "got: {}", msg);
    assert!(
        msg.contains("index 1"),
        "should identify the index: {}",
        msg
    );
}

#[test]
fn validate_multisig_json_args_deeply_nested() {
    let deep = r#"[[[[[["too deep"]]]]]]"#;
    let err = validate_multisig_json_args(deep).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("nesting too deep"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_long_string() {
    let long_str = format!(r#"["{}"]"#, "a".repeat(2000));
    let err = validate_multisig_json_args(&long_str).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("too long"), "got: {}", msg);
}

#[test]
fn validate_multisig_json_args_empty_array() {
    let result = validate_multisig_json_args("[]");
    assert!(
        result.is_ok(),
        "empty array should be valid: {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn validate_multisig_json_args_whitespace_around() {
    let result = validate_multisig_json_args("  [1, 2]  ");
    assert!(
        result.is_ok(),
        "whitespace should be trimmed: {:?}",
        result.err()
    );
}

#[test]
fn validate_multisig_json_args_nested_null() {
    let err = validate_multisig_json_args(r#"[{"key": null}]"#).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("null"), "nested null: {}", msg);
}

#[test]
fn validate_multisig_json_args_valid_bool_and_negative() {
    let result = validate_multisig_json_args(r#"[true, false, -1, 0]"#);
    assert!(result.is_ok());
}

#[test]
fn validate_multisig_json_args_nested_array_ok() {
    // 3 levels deep — should be fine (limit is 4)
    let result = validate_multisig_json_args(r#"[[[1, 2]], "ok"]"#);
    assert!(result.is_ok(), "3 levels of nesting: {:?}", result.err());
}

// ──── json_to_subxt_value tests ────

#[test]
fn json_to_subxt_value_large_u64() {
    let v = serde_json::json!(u64::MAX);
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_negative_i64() {
    let v = serde_json::json!(i64::MIN);
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_hex_string_short() {
    let v = serde_json::json!("0xab");
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_invalid_hex_string_fallback() {
    // "0xZZ" should fail hex decode and fall back to string
    let v = serde_json::json!("0xZZ");
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_bool_false() {
    let v = serde_json::json!(false);
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_nested_array() {
    let v = serde_json::json!([[1, 2], [3, 4]]);
    let result = json_to_subxt_value(&v);
    let _ = result;
}

#[test]
fn json_to_subxt_value_null_to_string() {
    let v = serde_json::json!(null);
    let result = json_to_subxt_value(&v);
    let _ = result; // null maps to string "null"
}

#[test]
fn json_to_subxt_value_object_to_string() {
    let v = serde_json::json!({"key": "val"});
    let result = json_to_subxt_value(&v);
    let _ = result; // object falls through to string
}

#[test]
fn parse_children_duplicate_hotkeys_allowed() {
    let alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
    let result = parse_children(&format!("500:{},500:{}", alice, alice));
    // Duplicate hotkeys should still parse — the chain will reject if needed
    assert!(result.is_ok(), "duplicate hotkeys: {:?}", result.err());
    assert_eq!(result.unwrap().len(), 2);
}

// =====================================================================
// validate_evm_address tests
// =====================================================================

#[test]
fn evm_address_valid_with_0x_prefix() {
    assert!(validate_evm_address("0x1234567890abcdef1234567890abcdef12345678", "test").is_ok());
}

#[test]
fn evm_address_valid_without_0x_prefix() {
    assert!(validate_evm_address("1234567890abcdef1234567890abcdef12345678", "test").is_ok());
}

#[test]
fn evm_address_valid_uppercase() {
    assert!(validate_evm_address("0x1234567890ABCDEF1234567890ABCDEF12345678", "test").is_ok());
}

#[test]
fn evm_address_valid_mixed_case() {
    assert!(validate_evm_address("0xABCDef1234567890abcdef1234567890AbCdEf12", "test").is_ok());
}

#[test]
fn evm_address_valid_all_zeros() {
    assert!(validate_evm_address("0x0000000000000000000000000000000000000000", "test").is_ok());
}

#[test]
fn evm_address_valid_all_f() {
    assert!(validate_evm_address("0xffffffffffffffffffffffffffffffffffffffff", "test").is_ok());
}

#[test]
fn evm_address_empty() {
    let err = validate_evm_address("", "source").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "got: {}", err);
}

#[test]
fn evm_address_just_0x() {
    let err = validate_evm_address("0x", "source").unwrap_err();
    assert!(err.to_string().contains("empty after '0x'"), "got: {}", err);
}

#[test]
fn evm_address_too_short() {
    let err = validate_evm_address("0x1234", "target").unwrap_err();
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_too_long() {
    let err =
        validate_evm_address("0x1234567890abcdef1234567890abcdef1234567800", "target").unwrap_err();
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_odd_length() {
    let err = validate_evm_address("0x1234567890abcdef1234567890abcdef1234567", "src").unwrap_err();
    assert!(err.to_string().contains("odd hex length"), "got: {}", err);
}

#[test]
fn evm_address_invalid_hex_char() {
    let err =
        validate_evm_address("0x1234567890abcdef1234567890abcdef1234567g", "src").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "got: {}", err);
}

#[test]
fn evm_address_with_spaces() {
    let err = validate_evm_address("  0x1234  ", "src").unwrap_err();
    // trimmed = "0x1234" → too short
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_0_x_prefix() {
    assert!(validate_evm_address("0X1234567890abcdef1234567890abcdef12345678", "test").is_ok());
}

#[test]
fn evm_address_19_bytes() {
    let err = validate_evm_address("0x1234567890abcdef1234567890abcdef123456", "test").unwrap_err();
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

#[test]
fn evm_address_21_bytes() {
    let err =
        validate_evm_address("0x1234567890abcdef1234567890abcdef123456789a", "test").unwrap_err();
    assert!(err.to_string().contains("20 bytes"), "got: {}", err);
}

