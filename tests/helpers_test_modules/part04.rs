use crate::common::*;

#[test]
fn validate_subnet_name_empty() {
    let err = validate_subnet_name("", "subnet name").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "err: {}", err);
}

#[test]
fn validate_subnet_name_whitespace_only() {
    let err = validate_subnet_name("   ", "subnet name").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "err: {}", err);
}

#[test]
fn validate_subnet_name_too_long() {
    let name = "a".repeat(257);
    let err = validate_subnet_name(&name, "name").unwrap_err();
    assert!(err.to_string().contains("too long"), "err: {}", err);
    assert!(err.to_string().contains("max 256"), "err: {}", err);
}

#[test]
fn validate_subnet_name_control_chars() {
    let err = validate_subnet_name("My\x00Subnet", "name").unwrap_err();
    assert!(
        err.to_string().contains("control character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_subnet_name_newline_rejected() {
    let err = validate_subnet_name("My\nSubnet", "name").unwrap_err();
    assert!(
        err.to_string().contains("control character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_subnet_name_tab_rejected() {
    let err = validate_subnet_name("My\tSubnet", "name").unwrap_err();
    assert!(
        err.to_string().contains("control character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_subnet_name_unicode_ok() {
    assert!(validate_subnet_name("Subnet-日本語", "name").is_ok());
}

#[test]
fn validate_subnet_name_label_in_error() {
    let err = validate_subnet_name("", "custom-label").unwrap_err();
    assert!(err.to_string().contains("custom-label"), "label: {}", err);
}

#[test]
fn validate_subnet_name_special_chars_ok() {
    assert!(validate_subnet_name("My-Subnet_v2.0 (beta)", "name").is_ok());
}

// ── validate_github_repo ──

#[test]
fn validate_github_repo_valid() {
    assert!(validate_github_repo("opentensor/subtensor").is_ok());
}

#[test]
fn validate_github_repo_valid_with_dots() {
    assert!(validate_github_repo("user.name/repo.rs").is_ok());
}

#[test]
fn validate_github_repo_valid_with_hyphens() {
    assert!(validate_github_repo("my-org/my-repo").is_ok());
}

#[test]
fn validate_github_repo_valid_with_underscores() {
    assert!(validate_github_repo("my_org/my_repo").is_ok());
}

#[test]
fn validate_github_repo_empty_ok() {
    assert!(validate_github_repo("").is_ok());
}

#[test]
fn validate_github_repo_whitespace_ok() {
    assert!(validate_github_repo("   ").is_ok());
}

#[test]
fn validate_github_repo_missing_slash() {
    let err = validate_github_repo("justarepo").unwrap_err();
    assert!(err.to_string().contains("owner/repo"), "err: {}", err);
}

#[test]
fn validate_github_repo_empty_owner() {
    let err = validate_github_repo("/repo").unwrap_err();
    assert!(err.to_string().contains("owner/repo"), "err: {}", err);
}

#[test]
fn validate_github_repo_empty_repo() {
    let err = validate_github_repo("owner/").unwrap_err();
    assert!(err.to_string().contains("owner/repo"), "err: {}", err);
}

#[test]
fn validate_github_repo_too_many_slashes() {
    let err = validate_github_repo("a/b/c").unwrap_err();
    assert!(err.to_string().contains("owner/repo"), "err: {}", err);
}

#[test]
fn validate_github_repo_special_chars() {
    let err = validate_github_repo("user@/repo").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_github_repo_spaces_in_name() {
    let err = validate_github_repo("my org/my repo").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_github_repo_too_long() {
    let long = format!("{}/{}", "a".repeat(128), "b".repeat(128));
    let err = validate_github_repo(&long).unwrap_err();
    assert!(err.to_string().contains("too long"), "err: {}", err);
}

#[test]
fn validate_github_repo_unicode_rejected() {
    let err = validate_github_repo("日本/語").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_github_repo_hash_character() {
    let err = validate_github_repo("user/repo#1").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_github_repo_single_chars() {
    assert!(validate_github_repo("a/b").is_ok());
}

// ── validate_proxy_type ──

use agcli::cli::helpers::validate_proxy_type;

#[test]
fn validate_proxy_type_any() {
    assert!(validate_proxy_type("any").is_ok());
    assert!(validate_proxy_type("Any").is_ok());
    assert!(validate_proxy_type("ANY").is_ok());
}

#[test]
fn validate_proxy_type_owner() {
    assert!(validate_proxy_type("owner").is_ok());
    assert!(validate_proxy_type("Owner").is_ok());
}

#[test]
fn validate_proxy_type_staking() {
    assert!(validate_proxy_type("staking").is_ok());
    assert!(validate_proxy_type("Staking").is_ok());
    assert!(validate_proxy_type("STAKING").is_ok());
}

#[test]
fn validate_proxy_type_transfer() {
    assert!(validate_proxy_type("transfer").is_ok());
    assert!(validate_proxy_type("Transfer").is_ok());
}

#[test]
fn validate_proxy_type_nontransfer_variants() {
    assert!(validate_proxy_type("nontransfer").is_ok());
    assert!(validate_proxy_type("NonTransfer").is_ok());
    assert!(validate_proxy_type("non_transfer").is_ok());
}

#[test]
fn validate_proxy_type_noncritical_variants() {
    assert!(validate_proxy_type("noncritical").is_ok());
    assert!(validate_proxy_type("non_critical").is_ok());
    assert!(validate_proxy_type("NonCritical").is_ok());
}

#[test]
fn validate_proxy_type_governance() {
    assert!(validate_proxy_type("governance").is_ok());
    assert!(validate_proxy_type("Governance").is_ok());
}

#[test]
fn validate_proxy_type_senate() {
    assert!(validate_proxy_type("senate").is_ok());
    assert!(validate_proxy_type("Senate").is_ok());
}

#[test]
fn validate_proxy_type_registration() {
    assert!(validate_proxy_type("registration").is_ok());
    assert!(validate_proxy_type("Registration").is_ok());
}

#[test]
fn validate_proxy_type_nonfungible() {
    assert!(validate_proxy_type("nonfungible").is_ok());
    assert!(validate_proxy_type("non_fungible").is_ok());
}

#[test]
fn validate_proxy_type_smalltransfer() {
    assert!(validate_proxy_type("smalltransfer").is_ok());
    assert!(validate_proxy_type("small_transfer").is_ok());
}

#[test]
fn validate_proxy_type_rootweights() {
    assert!(validate_proxy_type("rootweights").is_ok());
    assert!(validate_proxy_type("root_weights").is_ok());
}

#[test]
fn validate_proxy_type_childkeys() {
    assert!(validate_proxy_type("childkeys").is_ok());
    assert!(validate_proxy_type("child_keys").is_ok());
}

#[test]
fn validate_proxy_type_triumvirate() {
    assert!(validate_proxy_type("triumvirate").is_ok());
    assert!(validate_proxy_type("Triumvirate").is_ok());
}

#[test]
fn validate_proxy_type_swaphotkey() {
    assert!(validate_proxy_type("swaphotkey").is_ok());
    assert!(validate_proxy_type("swap_hotkey").is_ok());
}

#[test]
fn validate_proxy_type_subnetleasebeneficiary() {
    assert!(validate_proxy_type("subnetleasebeneficiary").is_ok());
    assert!(validate_proxy_type("subnet_lease_beneficiary").is_ok());
}

#[test]
fn validate_proxy_type_rootclaim() {
    assert!(validate_proxy_type("rootclaim").is_ok());
    assert!(validate_proxy_type("root_claim").is_ok());
}

#[test]
fn validate_proxy_type_sudo_unchecked_set_code() {
    assert!(validate_proxy_type("sudouncheckedsetcode").is_ok());
    assert!(validate_proxy_type("sudo_unchecked_set_code").is_ok());
}

#[test]
fn validate_proxy_type_empty_rejected() {
    let err = validate_proxy_type("").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "err: {}", err);
}

#[test]
fn validate_proxy_type_typo_rejected() {
    let err = validate_proxy_type("Stakking").unwrap_err();
    assert!(
        err.to_string().contains("Unknown proxy type"),
        "err: {}",
        err
    );
    assert!(
        err.to_string().contains("Staking"),
        "should suggest valid types: {}",
        err
    );
}

#[test]
fn validate_proxy_type_random_string_rejected() {
    let err = validate_proxy_type("foobar123").unwrap_err();
    assert!(
        err.to_string().contains("Unknown proxy type"),
        "err: {}",
        err
    );
}

#[test]
fn validate_proxy_type_numeric_rejected() {
    let err = validate_proxy_type("42").unwrap_err();
    assert!(
        err.to_string().contains("Unknown proxy type"),
        "err: {}",
        err
    );
}

#[test]
fn validate_proxy_type_whitespace_rejected() {
    let err = validate_proxy_type("  ").unwrap_err();
    assert!(
        err.to_string().contains("Unknown proxy type"),
        "err: {}",
        err
    );
}

#[test]
fn validate_proxy_type_partial_match_rejected() {
    // "stake" is not a valid type — must be "staking"
    let err = validate_proxy_type("stake").unwrap_err();
    assert!(
        err.to_string().contains("Unknown proxy type"),
        "err: {}",
        err
    );
}

#[test]
fn validate_proxy_type_special_chars_rejected() {
    let err = validate_proxy_type("any;drop").unwrap_err();
    assert!(
        err.to_string().contains("Unknown proxy type"),
        "err: {}",
        err
    );
}

// ── validate_spending_limit ──

use agcli::cli::helpers::validate_spending_limit;

#[test]
fn validate_spending_limit_valid() {
    assert!(validate_spending_limit(100.0, "1").is_ok());
}

#[test]
fn validate_spending_limit_zero() {
    assert!(validate_spending_limit(0.0, "1").is_ok());
}

#[test]
fn validate_spending_limit_large_valid() {
    assert!(validate_spending_limit(1_000_000.0, "0").is_ok());
}

#[test]
fn validate_spending_limit_max_netuid() {
    assert!(validate_spending_limit(50.0, "65535").is_ok());
}

#[test]
fn validate_spending_limit_negative_rejected() {
    let err = validate_spending_limit(-100.0, "1").unwrap_err();
    assert!(err.to_string().contains("negative"), "err: {}", err);
}

#[test]
fn validate_spending_limit_tiny_negative_rejected() {
    let err = validate_spending_limit(-0.001, "1").unwrap_err();
    assert!(err.to_string().contains("negative"), "err: {}", err);
}

#[test]
fn validate_spending_limit_infinity_rejected() {
    let err = validate_spending_limit(f64::INFINITY, "1").unwrap_err();
    assert!(err.to_string().contains("finite"), "err: {}", err);
}

#[test]
fn validate_spending_limit_neg_infinity_rejected() {
    let err = validate_spending_limit(f64::NEG_INFINITY, "1").unwrap_err();
    assert!(err.to_string().contains("finite"), "err: {}", err);
}

#[test]
fn validate_spending_limit_nan_rejected() {
    let err = validate_spending_limit(f64::NAN, "1").unwrap_err();
    assert!(err.to_string().contains("finite"), "err: {}", err);
}

#[test]
fn validate_spending_limit_non_numeric_netuid() {
    let err = validate_spending_limit(100.0, "abc").unwrap_err();
    assert!(err.to_string().contains("Invalid netuid"), "err: {}", err);
}

#[test]
fn validate_spending_limit_empty_netuid() {
    let err = validate_spending_limit(100.0, "").unwrap_err();
    assert!(err.to_string().contains("Invalid netuid"), "err: {}", err);
}

#[test]
fn validate_spending_limit_netuid_overflow() {
    let err = validate_spending_limit(100.0, "99999").unwrap_err();
    assert!(err.to_string().contains("Invalid netuid"), "err: {}", err);
}

#[test]
fn validate_spending_limit_netuid_negative() {
    let err = validate_spending_limit(100.0, "-1").unwrap_err();
    assert!(err.to_string().contains("Invalid netuid"), "err: {}", err);
}

#[test]
fn validate_spending_limit_netuid_with_text() {
    let err = validate_spending_limit(100.0, "1abc").unwrap_err();
    assert!(err.to_string().contains("Invalid netuid"), "err: {}", err);
}

#[test]
fn validate_spending_limit_fractional_valid() {
    assert!(validate_spending_limit(0.5, "1").is_ok());
    assert!(validate_spending_limit(99.99, "2").is_ok());
}

// ── validate_call_hash ──────────────────────────────────────────────

#[test]
fn validate_call_hash_valid_with_0x_prefix() {
    let hash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    assert!(validate_call_hash(hash, "test").is_ok());
}

#[test]
fn validate_call_hash_valid_bare_hex() {
    let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    assert!(validate_call_hash(hash, "test").is_ok());
}

#[test]
fn validate_call_hash_valid_uppercase_prefix() {
    let hash = "0Xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    assert!(validate_call_hash(hash, "test").is_ok());
}

#[test]
fn validate_call_hash_valid_mixed_case_hex() {
    let hash = "0xABCDEF1234567890abcdef1234567890ABCDEF1234567890abcdef1234567890";
    assert!(validate_call_hash(hash, "test").is_ok());
}

#[test]
fn validate_call_hash_valid_all_zeros() {
    let hash = "0x0000000000000000000000000000000000000000000000000000000000000000";
    assert!(validate_call_hash(hash, "test").is_ok());
}

#[test]
fn validate_call_hash_valid_all_f() {
    let hash = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
    assert!(validate_call_hash(hash, "test").is_ok());
}

#[test]
fn validate_call_hash_empty_rejected() {
    let err = validate_call_hash("", "test").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "err: {}", err);
}

#[test]
fn validate_call_hash_whitespace_only_rejected() {
    let err = validate_call_hash("   ", "test").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "err: {}", err);
}

#[test]
fn validate_call_hash_only_0x_prefix_rejected() {
    let err = validate_call_hash("0x", "test").unwrap_err();
    assert!(err.to_string().contains("empty after '0x'"), "err: {}", err);
}

#[test]
fn validate_call_hash_too_short_rejected() {
    let err = validate_call_hash("0xabcdef", "test").unwrap_err();
    assert!(err.to_string().contains("64 hex chars"), "err: {}", err);
    assert!(err.to_string().contains("got 6"), "err: {}", err);
}

#[test]
fn validate_call_hash_too_long_rejected() {
    let hash = "0x".to_string() + &"ab".repeat(33); // 66 hex chars
    let err = validate_call_hash(&hash, "test").unwrap_err();
    assert!(err.to_string().contains("64 hex chars"), "err: {}", err);
}

#[test]
fn validate_call_hash_non_hex_rejected() {
    let hash = "0xgggggg1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
    let err = validate_call_hash(hash, "test").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "err: {}", err);
    assert!(err.to_string().contains("'g'"), "err: {}", err);
}

#[test]
fn validate_call_hash_one_byte_short_rejected() {
    // 62 hex chars = 31 bytes
    let hash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef12345678";
    let err = validate_call_hash(hash, "test").unwrap_err();
    assert!(err.to_string().contains("64 hex chars"), "err: {}", err);
}

#[test]
fn validate_call_hash_one_byte_long_rejected() {
    // 66 hex chars = 33 bytes
    let hash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab";
    let err = validate_call_hash(hash, "test").unwrap_err();
    assert!(err.to_string().contains("64 hex chars"), "err: {}", err);
}

#[test]
fn validate_call_hash_spaces_around_hash() {
    let hash = "  0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890  ";
    assert!(
        validate_call_hash(hash, "test").is_ok(),
        "should trim whitespace"
    );
}

#[test]
fn validate_call_hash_evm_address_length_rejected() {
    // 40 hex chars = 20 bytes (EVM address size, not call hash)
    let hash = "0xabcdef1234567890abcdef1234567890abcdef12";
    let err = validate_call_hash(hash, "test").unwrap_err();
    assert!(err.to_string().contains("64 hex chars"), "err: {}", err);
}

#[test]
fn validate_call_hash_special_chars_rejected() {
    let hash = "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef123456789!";
    let err = validate_call_hash(hash, "test").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "err: {}", err);
}

#[test]
fn validate_call_hash_embedded_spaces_rejected() {
    let hash = "0xabcdef12 34567890abcdef1234567890abcdef1234567890abcdef1234567890";
    let err = validate_call_hash(hash, "test").unwrap_err();
    assert!(err.to_string().contains("not valid hex"), "err: {}", err);
}

#[test]
fn validate_call_hash_label_in_error() {
    let err = validate_call_hash("", "proxy announce").unwrap_err();
    assert!(err.to_string().contains("proxy announce"), "err: {}", err);
}

// ── validate_config_network ──────────────────────────────────────────

#[test]
fn validate_config_network_finney() {
    assert!(validate_config_network("finney").is_ok());
}

#[test]
fn validate_config_network_test() {
    assert!(validate_config_network("test").is_ok());
}

#[test]
fn validate_config_network_local() {
    assert!(validate_config_network("local").is_ok());
}

#[test]
fn validate_config_network_archive() {
    assert!(validate_config_network("archive").is_ok());
}

#[test]
fn validate_config_network_case_insensitive() {
    assert!(validate_config_network("Finney").is_ok());
    assert!(validate_config_network("FINNEY").is_ok());
    assert!(validate_config_network("Test").is_ok());
    assert!(validate_config_network("TEST").is_ok());
    assert!(validate_config_network("Local").is_ok());
    assert!(validate_config_network("LOCAL").is_ok());
    assert!(validate_config_network("Archive").is_ok());
    assert!(validate_config_network("ARCHIVE").is_ok());
}

#[test]
fn validate_config_network_with_whitespace() {
    assert!(validate_config_network("  finney  ").is_ok());
    assert!(validate_config_network(" test ").is_ok());
}

#[test]
fn validate_config_network_unknown_rejected() {
    let err = validate_config_network("mainnet").unwrap_err();
    assert!(err.to_string().contains("Unknown network"), "err: {}", err);
    assert!(err.to_string().contains("mainnet"), "err: {}", err);
}

#[test]
fn validate_config_network_empty_rejected() {
    let err = validate_config_network("").unwrap_err();
    assert!(err.to_string().contains("Unknown network"), "err: {}", err);
}

#[test]
fn validate_config_network_custom_url_rejected() {
    // Users should use --endpoint for custom URLs, not --network
    let err = validate_config_network("wss://example.com").unwrap_err();
    assert!(err.to_string().contains("Unknown network"), "err: {}", err);
    assert!(err.to_string().contains("--endpoint"), "err: {}", err);
}

#[test]
fn validate_config_network_misspelled_rejected() {
    let err = validate_config_network("finey").unwrap_err();
    assert!(err.to_string().contains("Unknown network"), "err: {}", err);
}

#[test]
fn validate_config_network_devnet_rejected() {
    let err = validate_config_network("devnet").unwrap_err();
    assert!(err.to_string().contains("Unknown network"), "err: {}", err);
}

#[test]
fn validate_config_network_testnet_vs_test() {
    // "testnet" is wrong, "test" is correct
    let err = validate_config_network("testnet").unwrap_err();
    assert!(err.to_string().contains("Unknown network"), "err: {}", err);
}

// ── validate_admin_call_name (upgraded with known_params check) ─────

#[test]
fn validate_admin_call_name_known_calls_accepted() {
    let known_calls = [
        "sudo_set_tempo",
        "sudo_set_max_allowed_validators",
        "sudo_set_max_allowed_uids",
        "sudo_set_immunity_period",
        "sudo_set_min_allowed_weights",
        "sudo_set_max_weight_limit",
        "sudo_set_weights_set_rate_limit",
        "sudo_set_commit_reveal_weights_enabled",
        "sudo_set_difficulty",
        "sudo_set_bonds_moving_average",
        "sudo_set_target_registrations_per_interval",
        "sudo_set_activity_cutoff",
        "sudo_set_serving_rate_limit",
    ];
    for call in &known_calls {
        assert!(
            validate_admin_call_name(call).is_ok(),
            "known call '{}' should be accepted",
            call
        );
    }
}

#[test]
fn validate_admin_call_name_empty_rejected() {
    let err = validate_admin_call_name("").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "err: {}", err);
}

#[test]
fn validate_admin_call_name_whitespace_only_rejected() {
    let err = validate_admin_call_name("   ").unwrap_err();
    assert!(err.to_string().contains("cannot be empty"), "err: {}", err);
}

#[test]
fn validate_admin_call_name_too_long_rejected() {
    let long_name = "a".repeat(129);
    let err = validate_admin_call_name(&long_name).unwrap_err();
    assert!(err.to_string().contains("too long"), "err: {}", err);
}

#[test]
fn validate_admin_call_name_starts_with_digit_rejected() {
    let err = validate_admin_call_name("1sudo_set_tempo").unwrap_err();
    assert!(
        err.to_string().contains("must start with a letter"),
        "err: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_starts_with_underscore_rejected() {
    let err = validate_admin_call_name("_sudo_set_tempo").unwrap_err();
    assert!(
        err.to_string().contains("must start with a letter"),
        "err: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_special_chars_rejected() {
    let err = validate_admin_call_name("sudo-set-tempo").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_spaces_rejected() {
    let err = validate_admin_call_name("sudo set tempo").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_dots_rejected() {
    let err = validate_admin_call_name("sudo.set.tempo").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_unknown_still_accepted_with_warning() {
    // Unknown calls are rejected (Issue 711) to prevent typos executing sudo
    let err = validate_admin_call_name("sudo_set_some_future_param").unwrap_err();
    assert!(
        err.to_string().contains("Unknown admin call"),
        "got: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_case_sensitive() {
    // Matching is case-sensitive; known list uses lowercase snake_case
    let err = validate_admin_call_name("Sudo_Set_Tempo").unwrap_err();
    assert!(
        err.to_string().contains("Unknown admin call"),
        "got: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_with_whitespace_trimmed() {
    assert!(validate_admin_call_name("  sudo_set_tempo  ").is_ok());
}

#[test]
fn validate_admin_call_name_max_length_boundary() {
    // 128-char unknown name is rejected (format ok, not in known list)
    let name = "a".repeat(128);
    let err = validate_admin_call_name(&name).unwrap_err();
    assert!(
        err.to_string().contains("Unknown admin call") || err.to_string().contains("too long"),
        "got: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_unicode_rejected() {
    let err = validate_admin_call_name("sudo_set_tempö").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_emoji_rejected() {
    let err = validate_admin_call_name("sudo_🔥").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

#[test]
fn validate_admin_call_name_newline_rejected() {
    let err = validate_admin_call_name("sudo\nset").unwrap_err();
    assert!(
        err.to_string().contains("invalid character"),
        "err: {}",
        err
    );
}

// ── validate_netuid edge cases ──────────────────────────────────────

#[test]
fn validate_netuid_zero_rejected() {
    let err = validate_netuid(0).unwrap_err();
    assert!(err.to_string().contains("Root network"), "err: {}", err);
}

#[test]
fn validate_netuid_one_accepted() {
    assert!(validate_netuid(1).is_ok());
}

#[test]
fn validate_netuid_max_u16_accepted() {
    assert!(validate_netuid(65535).is_ok());
}

#[test]
fn validate_netuid_typical_values_accepted() {
    for netuid in [1, 2, 3, 5, 10, 18, 32, 100, 256, 1000] {
        assert!(
            validate_netuid(netuid).is_ok(),
            "netuid {} should be valid",
            netuid
        );
    }
}

#[test]
fn validate_netuid_error_message_helpful() {
    let err = validate_netuid(0).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Tip:") || msg.contains("netuid 1"),
        "Error should guide user: {}",
        msg
    );
}

// ═══════════════════════════════════════════════════════════════════
// validate_threshold (new in Step 20)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn validate_threshold_zero_accepted() {
    assert!(validate_threshold(0.0, "test").is_ok());
}

#[test]
fn validate_threshold_positive_accepted() {
    assert!(validate_threshold(1.0, "test").is_ok());
    assert!(validate_threshold(0.5, "test").is_ok());
    assert!(validate_threshold(100_000.0, "test").is_ok());
}

#[test]
fn validate_threshold_negative_rejected() {
    let err = validate_threshold(-1.0, "test").unwrap_err();
    assert!(err.to_string().contains("negative"), "Error: {}", err);
}

#[test]
fn validate_threshold_negative_small_rejected() {
    assert!(validate_threshold(-0.001, "test").is_err());
}

#[test]
fn validate_threshold_nan_rejected() {
    assert!(validate_threshold(f64::NAN, "test").is_err());
}

#[test]
fn validate_threshold_infinity_rejected() {
    assert!(validate_threshold(f64::INFINITY, "test").is_err());
    assert!(validate_threshold(f64::NEG_INFINITY, "test").is_err());
}

#[test]
fn validate_threshold_error_mentions_tip() {
    let err = validate_threshold(-5.0, "balance --threshold").unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("Tip:") || msg.contains("threshold"),
        "Error should be helpful: {}",
        msg
    );
}

// ═══════════════════════════════════════════════════════════════════
// resolve_and_validate_coldkey_address (new in Step 20)
// ═══════════════════════════════════════════════════════════════════

#[test]
fn resolve_and_validate_rejects_invalid_ss58() {
    let result = resolve_and_validate_coldkey_address(
        Some("not-a-real-address".to_string()),
        "/tmp/nonexistent",
        "default",
        "test --address",
    );
    assert!(result.is_err(), "invalid SS58 should be rejected");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("test --address"),
        "Error should mention parameter: {}",
        msg
    );
}

#[test]
fn resolve_and_validate_accepts_valid_ss58() {
    // This is a valid Bittensor SS58 address (Alice)
    let result = resolve_and_validate_coldkey_address(
        Some("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY".to_string()),
        "/tmp/nonexistent",
        "default",
        "test --address",
    );
    assert!(
        result.is_ok(),
        "valid SS58 should be accepted: {:?}",
        result.err()
    );
}

#[test]
fn resolve_and_validate_none_falls_back_to_wallet() {
    // With None address and nonexistent wallet dir, fallback tries to open wallet and fails -> Err
    let result = resolve_and_validate_coldkey_address(
        None,
        "/tmp/nonexistent_wallet_dir_xyz",
        "default",
        "test --address",
    );
    assert!(result.is_err(), "nonexistent wallet dir should yield Err");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Could not resolve") || msg.contains("wallet"),
        "got: {}",
        msg
    );
}

#[test]
fn resolve_and_validate_rejects_empty_string() {
    let result = resolve_and_validate_coldkey_address(
        Some("".to_string()),
        "/tmp/nonexistent",
        "default",
        "test --address",
    );
    assert!(result.is_err(), "empty address string should be rejected");
}

#[test]
fn resolve_and_validate_rejects_short_garbage() {
    let result = resolve_and_validate_coldkey_address(
        Some("abc".to_string()),
        "/tmp/nonexistent",
        "default",
        "test --address",
    );
    assert!(result.is_err(), "short garbage should be rejected");
}

#[test]
fn resolve_and_validate_rejects_ethereum_address() {
    let result = resolve_and_validate_coldkey_address(
        Some("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string()),
        "/tmp/nonexistent",
        "default",
        "portfolio --address",
    );
    assert!(result.is_err(), "Ethereum address should be rejected");
}

// ——— validate_limit_price tests ———

#[test]
fn limit_price_valid_small() {
    assert!(validate_limit_price(0.001, "test").is_ok());
}

#[test]
fn limit_price_valid_one() {
    assert!(validate_limit_price(1.0, "test").is_ok());
}

#[test]
fn limit_price_valid_large_but_safe() {
    // 1 billion * 1e9 = 1e18, fits in u64 (max ~1.84e19)
    assert!(validate_limit_price(1_000_000_000.0, "test").is_ok());
}

#[test]
fn limit_price_rejects_zero() {
    let r = validate_limit_price(0.0, "test");
    assert!(r.is_err());
    assert!(
        format!("{}", r.unwrap_err()).contains("must be positive"),
        "should explain positive requirement"
    );
}

#[test]
fn limit_price_rejects_negative() {
    let r = validate_limit_price(-1.0, "test");
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("must be positive"));
}

#[test]
fn limit_price_rejects_nan() {
    let r = validate_limit_price(f64::NAN, "test");
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("finite"));
}

#[test]
fn limit_price_rejects_infinity() {
    let r = validate_limit_price(f64::INFINITY, "test");
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("finite"));
}

#[test]
fn limit_price_rejects_neg_infinity() {
    assert!(validate_limit_price(f64::NEG_INFINITY, "test").is_err());
}

#[test]
fn limit_price_rejects_overflow() {
    // 20 billion * 1e9 = 2e19 > u64::MAX (~1.84e19)
    let r = validate_limit_price(20_000_000_000.0, "test");
    assert!(r.is_err());
    assert!(
        format!("{}", r.unwrap_err()).contains("too large"),
        "should explain overflow"
    );
}

#[test]
fn limit_price_boundary_just_below_max() {
    // u64::MAX / 1e9 ≈ 18.446... so 18.0 should be fine
    assert!(validate_limit_price(18.0, "test").is_ok());
}

#[test]
fn limit_price_boundary_just_above_max() {
    // 18.5e9 > u64::MAX? 18.5 * 1e9 = 1.85e10, fits. But 18_446_744_074.0 * 1e9 > u64::MAX
    assert!(validate_limit_price(18_446_744_074.0, "test").is_err());
}

// ——— validate_block_number tests ———

#[test]
fn block_number_valid() {
    assert!(validate_block_number(1, "test").is_ok());
    assert!(validate_block_number(100, "test").is_ok());
    assert!(validate_block_number(u32::MAX, "test").is_ok());
}

#[test]
fn block_number_rejects_zero() {
    let r = validate_block_number(0, "test");
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("genesis"));
}

// ——— validate_repeat_params tests ———

#[test]
fn repeat_params_valid() {
    assert!(validate_repeat_params(10, 5).is_ok());
    assert!(validate_repeat_params(1, 1).is_ok());
    assert!(validate_repeat_params(100, 100).is_ok());
}

#[test]
fn repeat_params_rejects_zero_every() {
    let r = validate_repeat_params(0, 5);
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("cannot be 0"));
}

#[test]
fn repeat_params_rejects_zero_count() {
    let r = validate_repeat_params(10, 0);
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("cannot be 0"));
}

#[test]
fn repeat_params_rejects_overflow() {
    // u32::MAX * u32::MAX overflows u64
    let r = validate_repeat_params(u32::MAX, u32::MAX);
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("overflow"));
}

#[test]
fn repeat_params_large_but_valid() {
    // 1_000_000 * 1_000 = 1e9, fits in u32::MAX (4.29e9)
    assert!(validate_repeat_params(1_000_000, 1_000).is_ok());
}

#[test]
fn repeat_params_boundary_overflow() {
    // 65536 * 65536 = 4_294_967_296 > u32::MAX
    let r = validate_repeat_params(65536, 65536);
    assert!(r.is_err());
}

// ——— validate_price_range tests ———

#[test]
fn price_range_valid() {
    assert!(validate_price_range(0.001, 1.0).is_ok());
    assert!(validate_price_range(0.5, 0.6).is_ok());
}

#[test]
fn price_range_rejects_equal() {
    let r = validate_price_range(1.0, 1.0);
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("strictly less"));
}

#[test]
fn price_range_rejects_inverted() {
    let r = validate_price_range(2.0, 1.0);
    assert!(r.is_err());
    assert!(format!("{}", r.unwrap_err()).contains("strictly less"));
}

#[test]
fn price_range_very_close_values() {
    // Extremely close but different
    assert!(validate_price_range(1.0, 1.0 + f64::EPSILON).is_ok());
}

#[test]
fn price_range_very_small_low() {
    assert!(validate_price_range(f64::MIN_POSITIVE, 1.0).is_ok());
}

// ──── Batch 6: Spending limit enforcement regression tests ────
// These verify check_spending_limit is callable for all the stake operations
// that now enforce it (move, swap, add-limit, remove-limit, swap-limit,
// transfer-stake, wizard). Without a config file, all should pass.

#[test]
fn spending_limit_move_stake_dest_subnet() {
    // move stake checks destination subnet — should pass with no config
    let result = agcli::cli::helpers::check_spending_limit(42, 1000.0);
    assert!(
        result.is_ok(),
        "move-stake spending check should pass without config"
    );
}

#[test]
fn spending_limit_swap_stake_dest_subnet() {
    let result = agcli::cli::helpers::check_spending_limit(18, 500.0);
    assert!(
        result.is_ok(),
        "swap-stake spending check should pass without config"
    );
}

#[test]
fn spending_limit_add_limit_order() {
    let result = agcli::cli::helpers::check_spending_limit(1, 250.0);
    assert!(
        result.is_ok(),
        "add-limit spending check should pass without config"
    );
}

#[test]
fn spending_limit_remove_limit_order() {
    let result = agcli::cli::helpers::check_spending_limit(3, 750.0);
    assert!(
        result.is_ok(),
        "remove-limit spending check should pass without config"
    );
}

#[test]
fn spending_limit_swap_limit_order() {
    let result = agcli::cli::helpers::check_spending_limit(99, 333.0);
    assert!(
        result.is_ok(),
        "swap-limit spending check should pass without config"
    );
}

#[test]
fn spending_limit_transfer_stake() {
    let result = agcli::cli::helpers::check_spending_limit(7, 100.0);
    assert!(
        result.is_ok(),
        "transfer-stake spending check should pass without config"
    );
}

#[test]
fn spending_limit_wizard() {
    // wizard now checks spending limit before staking
    let result = agcli::cli::helpers::check_spending_limit(12, 50.0);
    assert!(
        result.is_ok(),
        "wizard spending check should pass without config"
    );
}

// ──── Issue 636/637/638: Spending limit enforcement for raw calls ────

#[test]
fn raw_call_spending_limit_add_stake_passes_without_config() {
    // Without any spending limits configured, all calls should pass
    let args: Vec<serde_json::Value> = vec![
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!(1),                 // netuid
        serde_json::json!(50_000_000_000u64), // 50 TAO in rao
    ];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "add_stake",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_non_subtensor_passes() {
    // Non-SubtensorModule calls should always pass
    let args: Vec<serde_json::Value> = vec![serde_json::json!(1000)];
    let result =
        agcli::cli::helpers::check_spending_limit_for_raw_call("Balances", "transfer", &args);
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_unknown_call_passes() {
    // Unknown calls on SubtensorModule should pass (we only gate known staking calls)
    let args: Vec<serde_json::Value> = vec![serde_json::json!(1000)];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "unknown_extrinsic",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_remove_stake() {
    let args: Vec<serde_json::Value> = vec![
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!(2),                  // netuid
        serde_json::json!(100_000_000_000u64), // 100 TAO
    ];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "remove_stake",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_move_stake() {
    // move_stake(hotkey_o, hotkey_d, origin_netuid, dest_netuid, amount_rao)
    let args: Vec<serde_json::Value> = vec![
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!(1),                 // from
        serde_json::json!(2),                 // to
        serde_json::json!(25_000_000_000u64), // 25 TAO
    ];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "move_stake",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_swap_stake() {
    // swap_stake(hotkey, origin_netuid, dest_netuid, amount_rao)
    let args: Vec<serde_json::Value> = vec![
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!(1),
        serde_json::json!(2),
        serde_json::json!(10_000_000_000u64), // 10 TAO
    ];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "swap_stake",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_transfer_stake() {
    // transfer_stake(dest, hotkey, origin_netuid, dest_netuid, amount_rao)
    let args: Vec<serde_json::Value> = vec![
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!(1),
        serde_json::json!(2),
        serde_json::json!(5_000_000_000u64), // 5 TAO
    ];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "transfer_stake",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_add_stake_limit() {
    // add_stake_limit(hotkey, netuid, amount_rao, limit_price, allow_partial)
    let args: Vec<serde_json::Value> = vec![
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!(3),
        serde_json::json!(75_000_000_000u64), // 75 TAO
        serde_json::json!(1000),
        serde_json::json!(true),
    ];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "add_stake_limit",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_too_few_args() {
    // If args are too few, should pass (let encoding fail later)
    let args: Vec<serde_json::Value> = vec![serde_json::json!("hotkey")];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "add_stake",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_zero_amount() {
    // 0 TAO should always pass
    let args: Vec<serde_json::Value> = vec![
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!(1),
        serde_json::json!(0),
    ];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "add_stake",
        &args,
    );
    assert!(result.is_ok());
}

#[test]
fn raw_call_spending_limit_swap_stake_limit() {
    // swap_stake_limit(hotkey, origin_netuid, dest_netuid, amount_rao, ...)
    let args: Vec<serde_json::Value> = vec![
        serde_json::json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
        serde_json::json!(1),
        serde_json::json!(2),
        serde_json::json!(20_000_000_000u64), // 20 TAO
        serde_json::json!(500),
        serde_json::json!(true),
    ];
    let result = agcli::cli::helpers::check_spending_limit_for_raw_call(
        "SubtensorModule",
        "swap_stake_limit",
        &args,
    );
    assert!(result.is_ok());
}
