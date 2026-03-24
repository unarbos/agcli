use agcli::cli::OutputFormat;
use clap::Parser;

#[test]
fn parse_drand_write_pulse_missing_signature() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "drand",
        "write-pulse",
        "--payload",
        "0xdeadbeef",
    ]);
    assert!(
        cli.is_err(),
        "drand write-pulse without --signature should fail"
    );
}

// =====================================================================
// Admin commands — boundary value tests
// =====================================================================

#[test]
fn parse_admin_set_tempo_max_u16() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-tempo",
        "--netuid",
        "65535",
        "--tempo",
        "65535",
    ]);
    assert!(cli.is_ok(), "admin set-tempo max u16: {:?}", cli.err());
}

#[test]
fn parse_admin_set_tempo_overflow_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-tempo",
        "--netuid",
        "65536",
        "--tempo",
        "360",
    ]);
    assert!(
        cli.is_err(),
        "admin set-tempo netuid > u16::MAX should fail"
    );
}

#[test]
fn parse_admin_set_tempo_negative_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-tempo",
        "--netuid",
        "-1",
        "--tempo",
        "360",
    ]);
    assert!(cli.is_err(), "admin set-tempo negative netuid should fail");
}

#[test]
fn parse_admin_set_max_validators_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-max-validators",
        "--netuid",
        "1",
        "--max",
        "0",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-max-validators zero: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_weights_rate_limit_max_u64() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-weights-rate-limit",
        "--netuid",
        "1",
        "--limit",
        "18446744073709551615",
    ]);
    assert!(
        cli.is_ok(),
        "admin set-weights-rate-limit max u64: {:?}",
        cli.err()
    );
}

#[test]
fn parse_admin_set_weights_rate_limit_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-weights-rate-limit",
        "--netuid",
        "1",
        "--limit",
        "18446744073709551616",
    ]);
    assert!(
        cli.is_err(),
        "admin set-weights-rate-limit > u64::MAX should fail"
    );
}

#[test]
fn parse_admin_set_difficulty_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "admin",
        "set-difficulty",
        "--netuid",
        "1",
        "--difficulty",
        "0",
    ]);
    assert!(cli.is_ok(), "admin set-difficulty zero: {:?}", cli.err());
}

// =====================================================================
// Scheduler — boundary / edge-case tests
// =====================================================================

#[test]
fn parse_scheduler_schedule_priority_min() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "100",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--priority",
        "0",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler priority 0 (highest): {:?}",
        cli.err()
    );
}

#[test]
fn parse_scheduler_schedule_priority_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "100",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--priority",
        "255",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler priority 255 (lowest): {:?}",
        cli.err()
    );
}

#[test]
fn parse_scheduler_schedule_priority_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "100",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--priority",
        "256",
    ]);
    assert!(cli.is_err(), "scheduler priority > 255 should fail");
}

#[test]
fn parse_scheduler_schedule_when_zero() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "0",
        "--pallet",
        "System",
        "--call",
        "remark",
    ]);
    assert!(cli.is_ok(), "scheduler when=0: {:?}", cli.err());
}

#[test]
fn parse_scheduler_cancel_large_index() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "cancel",
        "--when",
        "999999",
        "--index",
        "4294967295",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler cancel max u32 index: {:?}",
        cli.err()
    );
}

#[test]
fn parse_scheduler_schedule_named_empty_id() {
    // Clap will accept an empty string for --id; runtime should validate
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule-named",
        "--id",
        "",
        "--when",
        "100",
        "--pallet",
        "System",
        "--call",
        "remark",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler schedule-named empty id (parses, runtime validates): {:?}",
        cli.err()
    );
}

// =====================================================================
// Contracts — boundary tests
// =====================================================================

#[test]
fn parse_contracts_instantiate_defaults_only() {
    // Just code-hash; value, data, salt, gas all have defaults
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(
        cli.is_ok(),
        "contracts instantiate defaults: {:?}",
        cli.err()
    );
}

#[test]
fn parse_contracts_call_value_and_gas() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "call",
        "--contract",
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--data",
        "0x",
        "--value",
        "0",
        "--gas-ref-time",
        "1",
        "--gas-proof-size",
        "1",
    ]);
    assert!(cli.is_ok(), "contracts call min gas: {:?}", cli.err());
}

// =====================================================================
// EVM — boundary tests
// =====================================================================

#[test]
fn parse_evm_call_default_gas() {
    // Defaults: input="0x", value=0x00...00, gas_limit=21000, max_fee_per_gas=0x00...01
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000000",
        "--target",
        "0x0000000000000000000000000000000000000000",
    ]);
    assert!(
        cli.is_ok(),
        "evm call zero addresses with defaults: {:?}",
        cli.err()
    );
}

#[test]
fn parse_evm_withdraw_zero_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "withdraw",
        "--address",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--amount",
        "0",
    ]);
    assert!(cli.is_ok(), "evm withdraw zero amount: {:?}", cli.err());
}

// =====================================================================
// Localnet commands — CLI parsing tests
// =====================================================================

#[test]
fn parse_localnet_start_defaults() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start"]);
    assert!(cli.is_ok(), "localnet start defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "start",
        "--image",
        "my-image:latest",
        "--container",
        "my_container",
        "--port",
        "9955",
        "--wait",
        "false",
        "--timeout",
        "300",
    ]);
    assert!(cli.is_ok(), "localnet start all args: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_custom_port() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--port", "8844"]);
    assert!(cli.is_ok(), "localnet start port 8844: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_port_zero() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--port", "0"]);
    assert!(
        cli.is_ok(),
        "localnet start port 0 (clap parse succeeds, runtime validates): {:?}",
        cli.err()
    );
}

#[test]
fn parse_localnet_start_port_max() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--port", "65535"]);
    assert!(cli.is_ok(), "localnet start port 65535: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_port_overflow() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--port", "65536"]);
    assert!(cli.is_err(), "localnet start port > 65535 should fail");
}

#[test]
fn parse_localnet_start_timeout_zero() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--timeout", "0"]);
    assert!(cli.is_ok(), "localnet start timeout 0: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_wait_true() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--wait", "true"]);
    assert!(cli.is_ok(), "localnet start wait true: {:?}", cli.err());
}

#[test]
fn parse_localnet_stop_defaults() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "stop"]);
    assert!(cli.is_ok(), "localnet stop defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_stop_custom_container() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "stop",
        "--container",
        "my_localnet",
    ]);
    assert!(
        cli.is_ok(),
        "localnet stop custom container: {:?}",
        cli.err()
    );
}

#[test]
fn parse_localnet_status_defaults() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "status"]);
    assert!(cli.is_ok(), "localnet status defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_status_with_port() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "status",
        "--port",
        "9944",
        "--container",
        "agcli_localnet",
    ]);
    assert!(cli.is_ok(), "localnet status with port: {:?}", cli.err());
}

#[test]
fn parse_localnet_reset_defaults() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "reset"]);
    assert!(cli.is_ok(), "localnet reset defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_reset_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "reset",
        "--image",
        "custom:tag",
        "--container",
        "my_chain",
        "--port",
        "10000",
        "--timeout",
        "60",
    ]);
    assert!(cli.is_ok(), "localnet reset all args: {:?}", cli.err());
}

#[test]
fn parse_localnet_logs_defaults() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "logs"]);
    assert!(cli.is_ok(), "localnet logs defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_logs_with_tail() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "logs", "--tail", "100"]);
    assert!(cli.is_ok(), "localnet logs tail 100: {:?}", cli.err());
}

#[test]
fn parse_localnet_logs_with_container() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "logs",
        "--container",
        "my_chain",
        "--tail",
        "50",
    ]);
    assert!(cli.is_ok(), "localnet logs container+tail: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_defaults() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "scaffold"]);
    assert!(cli.is_ok(), "localnet scaffold defaults: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "localnet",
        "scaffold",
        "--config",
        "scaffold.toml",
        "--image",
        "my-image:v1",
        "--port",
        "9955",
        "--no-start",
    ]);
    assert!(cli.is_ok(), "localnet scaffold all args: {:?}", cli.err());
}

#[test]
fn parse_localnet_scaffold_no_start_flag() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "scaffold", "--no-start"]);
    assert!(cli.is_ok(), "localnet scaffold no-start: {:?}", cli.err());
}

#[test]
fn parse_localnet_start_negative_port() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "start", "--port", "-1"]);
    assert!(cli.is_err(), "localnet start negative port should fail");
}

#[test]
fn parse_localnet_logs_tail_zero() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "logs", "--tail", "0"]);
    assert!(cli.is_ok(), "localnet logs tail 0: {:?}", cli.err());
}

#[test]
fn parse_localnet_reset_port_string() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "localnet", "reset", "--port", "abc"]);
    assert!(cli.is_err(), "localnet reset non-numeric port should fail");
}

// =====================================================================
// Doctor command — CLI parsing tests
// =====================================================================

#[test]
fn parse_doctor_plain() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "doctor"]);
    assert!(cli.is_ok(), "doctor plain: {:?}", cli.err());
}

#[test]
fn parse_doctor_json_output() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "doctor"]);
    assert!(cli.is_ok(), "doctor json output: {:?}", cli.err());
}

#[test]
fn parse_doctor_with_network() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--network", "test", "doctor"]);
    assert!(cli.is_ok(), "doctor with network: {:?}", cli.err());
}

#[test]
fn parse_doctor_with_wallet() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--wallet", "mywallet", "doctor"]);
    assert!(cli.is_ok(), "doctor with wallet: {:?}", cli.err());
}

// =====================================================================
// EVM — additional boundary tests with validation
// =====================================================================

#[test]
fn parse_evm_call_max_gas_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--gas-limit",
        "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "evm call max u64 gas: {:?}", cli.err());
}

#[test]
fn parse_evm_call_overflow_gas_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--gas-limit",
        "18446744073709551616",
    ]);
    assert!(cli.is_err(), "evm call gas > u64::MAX should fail");
}

#[test]
fn parse_evm_withdraw_max_u128() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "withdraw",
        "--address",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--amount",
        "340282366920938463463374607431768211455",
    ]);
    assert!(cli.is_ok(), "evm withdraw max u128: {:?}", cli.err());
}

#[test]
fn parse_evm_withdraw_overflow_u128() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "withdraw",
        "--address",
        "0x1234567890abcdef1234567890abcdef12345678",
        "--amount",
        "340282366920938463463374607431768211456",
    ]);
    assert!(cli.is_err(), "evm withdraw > u128::MAX should fail");
}

#[test]
fn parse_evm_call_with_input_data() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--input",
        "0xa9059cbb0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(
        cli.is_ok(),
        "evm call with ABI-encoded input: {:?}",
        cli.err()
    );
}

#[test]
fn parse_evm_call_with_value() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--value",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
    ]);
    assert!(cli.is_ok(), "evm call with value: {:?}", cli.err());
}

// =====================================================================
// Scheduler — additional validation-related tests
// =====================================================================

#[test]
fn parse_scheduler_schedule_repeat_both() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "1000",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--repeat-every",
        "100",
        "--repeat-count",
        "5",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler schedule with repeat: {:?}",
        cli.err()
    );
}

#[test]
fn parse_scheduler_schedule_repeat_every_only() {
    // Clap accepts partial repeats, runtime validates pair
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "1000",
        "--pallet",
        "System",
        "--call",
        "remark",
        "--repeat-every",
        "100",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler repeat-every alone (runtime validates): {:?}",
        cli.err()
    );
}

#[test]
fn parse_scheduler_schedule_max_when() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "4294967295",
        "--pallet",
        "System",
        "--call",
        "remark",
    ]);
    assert!(cli.is_ok(), "scheduler max u32 when: {:?}", cli.err());
}

#[test]
fn parse_scheduler_schedule_when_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "schedule",
        "--when",
        "4294967296",
        "--pallet",
        "System",
        "--call",
        "remark",
    ]);
    assert!(cli.is_err(), "scheduler when > u32::MAX should fail");
}

#[test]
fn parse_scheduler_cancel_named_long_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "scheduler",
        "cancel-named",
        "--id",
        "a_very_long_descriptive_task_name",
    ]);
    assert!(
        cli.is_ok(),
        "scheduler cancel-named long id: {:?}",
        cli.err()
    );
}

// =====================================================================
// Contracts — additional boundary + missing field tests
// =====================================================================

#[test]
fn parse_contracts_instantiate_with_storage_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--storage-deposit-limit",
        "1000000",
    ]);
    assert!(
        cli.is_ok(),
        "contracts instantiate with storage limit: {:?}",
        cli.err()
    );
}

#[test]
fn parse_contracts_upload_with_storage_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "upload",
        "--code",
        "/tmp/contract.wasm",
        "--storage-deposit-limit",
        "5000000",
    ]);
    assert!(
        cli.is_ok(),
        "contracts upload with storage limit: {:?}",
        cli.err()
    );
}

#[test]
fn parse_contracts_instantiate_max_gas() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "instantiate",
        "--code-hash",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        "--gas-ref-time",
        "18446744073709551615",
        "--gas-proof-size",
        "18446744073709551615",
    ]);
    assert!(
        cli.is_ok(),
        "contracts instantiate max u64 gas: {:?}",
        cli.err()
    );
}

// =====================================================================
// Crowdloan — expanded boundary + edge case tests
// =====================================================================

#[test]
fn parse_crowdloan_list() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "list"]);
    assert!(cli.is_ok(), "crowdloan list: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_info_with_id() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "info", "--crowdloan-id", "42"]);
    assert!(cli.is_ok(), "crowdloan info: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_contributors_with_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "contributors",
        "--crowdloan-id",
        "1",
    ]);
    assert!(cli.is_ok(), "crowdloan contributors: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_create_all_fields() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--deposit",
        "10.5",
        "--min-contribution",
        "0.1",
        "--cap",
        "1000.0",
        "--end-block",
        "500000",
        "--target",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "crowdloan create all fields: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_create_without_target() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--deposit",
        "1",
        "--min-contribution",
        "0.01",
        "--cap",
        "100",
        "--end-block",
        "100000",
    ]);
    assert!(
        cli.is_ok(),
        "crowdloan create without target: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_create_max_end_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--deposit",
        "1",
        "--min-contribution",
        "0.01",
        "--cap",
        "100",
        "--end-block",
        "4294967295",
    ]);
    assert!(
        cli.is_ok(),
        "crowdloan create max u32 end_block: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_create_end_block_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--deposit",
        "1",
        "--min-contribution",
        "0.01",
        "--cap",
        "100",
        "--end-block",
        "4294967296",
    ]);
    assert!(cli.is_err(), "end_block u32 overflow should fail");
}

#[test]
fn parse_crowdloan_contribute_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "contribute",
        "--crowdloan-id",
        "5",
        "--amount",
        "10.0",
    ]);
    assert!(cli.is_ok(), "crowdloan contribute: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_contribute_max_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "contribute",
        "--crowdloan-id",
        "4294967295",
        "--amount",
        "1.0",
    ]);
    assert!(
        cli.is_ok(),
        "crowdloan contribute max u32 id: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_contribute_id_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "contribute",
        "--crowdloan-id",
        "4294967296",
        "--amount",
        "1.0",
    ]);
    assert!(cli.is_err(), "crowdloan-id u32 overflow should fail");
}

#[test]
fn parse_crowdloan_contribute_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "contribute",
        "--crowdloan-id",
        "1",
    ]);
    assert!(
        cli.is_err(),
        "crowdloan contribute missing amount should fail"
    );
}

#[test]
fn parse_crowdloan_contribute_missing_id() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "crowdloan", "contribute", "--amount", "10.0"]);
    assert!(cli.is_err(), "crowdloan contribute missing id should fail");
}

#[test]
fn parse_crowdloan_create_missing_deposit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--min-contribution",
        "0.01",
        "--cap",
        "100",
        "--end-block",
        "100000",
    ]);
    assert!(cli.is_err(), "crowdloan create missing deposit should fail");
}

#[test]
fn parse_crowdloan_create_missing_cap() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "create",
        "--deposit",
        "1",
        "--min-contribution",
        "0.01",
        "--end-block",
        "100000",
    ]);
    assert!(cli.is_err(), "crowdloan create missing cap should fail");
}

#[test]
fn parse_crowdloan_update_cap_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-cap",
        "--crowdloan-id",
        "3",
        "--cap",
        "500.0",
    ]);
    assert!(cli.is_ok(), "crowdloan update-cap: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_update_end_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-end",
        "--crowdloan-id",
        "3",
        "--end-block",
        "200000",
    ]);
    assert!(cli.is_ok(), "crowdloan update-end: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_update_min_contribution_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "crowdloan",
        "update-min-contribution",
        "--crowdloan-id",
        "3",
        "--min-contribution",
        "0.5",
    ]);
    assert!(
        cli.is_ok(),
        "crowdloan update-min-contribution: {:?}",
        cli.err()
    );
}

#[test]
fn parse_crowdloan_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--wallet",
        "mywallet",
        "crowdloan",
        "info",
        "--crowdloan-id",
        "1",
    ]);
    assert!(cli.is_ok(), "crowdloan with global flags: {:?}", cli.err());
}

#[test]
fn parse_crowdloan_with_json_output() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "crowdloan", "list"]);
    assert!(cli.is_ok(), "crowdloan list json: {:?}", cli.err());
}

// =====================================================================
// Commitment — expanded boundary + edge case tests
// =====================================================================

#[test]
fn parse_commitment_set_with_global() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "commitment",
        "set",
        "--netuid",
        "1",
        "--data",
        "endpoint:http://my.server:8080,version:2.0",
    ]);
    assert!(cli.is_ok(), "commitment set with global: {:?}", cli.err());
}

#[test]
fn parse_commitment_set_missing_data() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "commitment", "set", "--netuid", "1"]);
    assert!(cli.is_err(), "commitment set missing data should fail");
}

#[test]
fn parse_commitment_set_missing_netuid() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "commitment", "set", "--data", "key:value"]);
    assert!(cli.is_err(), "commitment set missing netuid should fail");
}

#[test]
fn parse_commitment_get_missing_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "commitment", "get", "--netuid", "1"]);
    assert!(cli.is_err(), "commitment get missing hotkey should fail");
}

#[test]
fn parse_commitment_get_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "commitment",
        "get",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "commitment get missing netuid should fail");
}

#[test]
fn parse_commitment_list_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "commitment", "list"]);
    assert!(cli.is_err(), "commitment list missing netuid should fail");
}

#[test]
fn parse_commitment_get_with_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "commitment",
        "get",
        "--netuid",
        "1",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "commitment get json: {:?}", cli.err());
}

#[test]
fn parse_commitment_set_netuid_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "commitment",
        "set",
        "--netuid",
        "65535",
        "--data",
        "endpoint:http://test",
    ]);
    assert!(
        cli.is_ok(),
        "commitment set max u16 netuid: {:?}",
        cli.err()
    );
}

#[test]
fn parse_commitment_set_netuid_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "commitment",
        "set",
        "--netuid",
        "65536",
        "--data",
        "endpoint:http://test",
    ]);
    assert!(
        cli.is_err(),
        "commitment set netuid u16 overflow should fail"
    );
}

// =====================================================================
// Liquidity — expanded boundary + edge case tests
// =====================================================================

#[test]
fn parse_liquidity_add_all_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "1",
        "--price-low",
        "0.001",
        "--price-high",
        "1.5",
        "--amount",
        "1000000",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "liquidity add all args: {:?}", cli.err());
}

#[test]
fn parse_liquidity_add_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--price-low",
        "0.001",
        "--price-high",
        "1.0",
        "--amount",
        "1000",
    ]);
    assert!(cli.is_err(), "liquidity add missing netuid should fail");
}

#[test]
fn parse_liquidity_add_missing_price_low() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "1",
        "--price-high",
        "1.0",
        "--amount",
        "1000",
    ]);
    assert!(cli.is_err(), "liquidity add missing price-low should fail");
}

#[test]
fn parse_liquidity_add_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "1",
        "--price-low",
        "0.001",
        "--price-high",
        "1.0",
    ]);
    assert!(cli.is_err(), "liquidity add missing amount should fail");
}

#[test]
fn parse_liquidity_add_max_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "1",
        "--price-low",
        "0.001",
        "--price-high",
        "1.0",
        "--amount",
        "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "liquidity add max u64 amount: {:?}", cli.err());
}

#[test]
fn parse_liquidity_add_amount_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "add",
        "--netuid",
        "1",
        "--price-low",
        "0.001",
        "--price-high",
        "1.0",
        "--amount",
        "18446744073709551616",
    ]);
    assert!(cli.is_err(), "liquidity add u64 overflow should fail");
}

#[test]
fn parse_liquidity_remove_basic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "remove",
        "--netuid",
        "1",
        "--position-id",
        "42",
    ]);
    assert!(cli.is_ok(), "liquidity remove: {:?}", cli.err());
}

#[test]
fn parse_liquidity_remove_with_hotkey() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "remove",
        "--netuid",
        "1",
        "--position-id",
        "42",
        "--hotkey-address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "liquidity remove with hotkey: {:?}", cli.err());
}

#[test]
fn parse_liquidity_remove_missing_position() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "liquidity", "remove", "--netuid", "1"]);
    assert!(
        cli.is_err(),
        "liquidity remove missing position should fail"
    );
}

#[test]
fn parse_liquidity_remove_max_position_id() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "remove",
        "--netuid",
        "1",
        "--position-id",
        "340282366920938463463374607431768211455",
    ]);
    assert!(cli.is_ok(), "liquidity remove max u128: {:?}", cli.err());
}

#[test]
fn parse_liquidity_modify_negative_delta() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--position-id",
        "10",
        "--delta",
        "-5000",
    ]);
    assert!(
        cli.is_ok(),
        "liquidity modify negative delta: {:?}",
        cli.err()
    );
}

#[test]
fn parse_liquidity_modify_positive_delta() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--position-id",
        "10",
        "--delta",
        "5000",
    ]);
    assert!(
        cli.is_ok(),
        "liquidity modify positive delta: {:?}",
        cli.err()
    );
}

#[test]
fn parse_liquidity_modify_missing_delta() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "modify",
        "--netuid",
        "1",
        "--position-id",
        "10",
    ]);
    assert!(cli.is_err(), "liquidity modify missing delta should fail");
}

#[test]
fn parse_liquidity_toggle_enable() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "liquidity",
        "toggle",
        "--netuid",
        "1",
        "--enable",
    ]);
    assert!(cli.is_ok(), "liquidity toggle enable: {:?}", cli.err());
}

#[test]
fn parse_liquidity_toggle_disable() {
    // Without --enable flag, enable defaults to false
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "liquidity", "toggle", "--netuid", "1"]);
    assert!(cli.is_ok(), "liquidity toggle disable: {:?}", cli.err());
}

#[test]
fn parse_liquidity_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--wallet",
        "mywallet",
        "liquidity",
        "add",
        "--netuid",
        "5",
        "--price-low",
        "0.01",
        "--price-high",
        "10.0",
        "--amount",
        "500",
    ]);
    assert!(cli.is_ok(), "liquidity with global flags: {:?}", cli.err());
}

// =====================================================================
// Subscribe — expanded tests
// =====================================================================

#[test]
fn parse_subscribe_blocks_with_json() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "subscribe", "blocks"]);
    assert!(cli.is_ok(), "subscribe blocks json: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_default_filter() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "events"]);
    assert!(cli.is_ok(), "subscribe events default: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_all_filters() {
    for filter in [
        "all",
        "staking",
        "stake",
        "registration",
        "register",
        "reg",
        "transfer",
        "transfers",
        "weights",
        "weight",
        "subnet",
        "subnets",
        "delegation",
        "delegate",
        "delegates",
        "keys",
        "key",
        "swap",
        "dex",
        "liquidity",
        "governance",
        "gov",
        "sudo",
        "safemode",
        "crowdloan",
        "crowdloans",
        "fund",
    ] {
        let cli =
            agcli::cli::Cli::try_parse_from(["agcli", "subscribe", "events", "--filter", filter]);
        assert!(
            cli.is_ok(),
            "subscribe events --filter {}: {:?}",
            filter,
            cli.err()
        );
    }
}

#[test]
fn parse_subscribe_events_with_account_and_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subscribe",
        "events",
        "--filter",
        "staking",
        "--netuid",
        "1",
        "--account",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "subscribe events all opts: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_netuid_max() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "subscribe",
        "events",
        "--filter",
        "all",
        "--netuid",
        "65535",
    ]);
    assert!(cli.is_ok(), "subscribe events max netuid: {:?}", cli.err());
}

#[test]
fn parse_subscribe_events_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "finney",
        "--output",
        "json",
        "subscribe",
        "events",
        "--filter",
        "transfer",
    ]);
    assert!(
        cli.is_ok(),
        "subscribe events with globals: {:?}",
        cli.err()
    );
}

// =====================================================================
// Diff — expanded boundary tests
// =====================================================================

#[test]
fn parse_diff_portfolio_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--block1",
        "100",
        "--block2",
        "200",
    ]);
    assert!(cli.is_ok(), "diff portfolio with address: {:?}", cli.err());
}

#[test]
fn parse_diff_portfolio_max_blocks() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--block1",
        "0",
        "--block2",
        "4294967295",
    ]);
    assert!(cli.is_ok(), "diff portfolio max u32: {:?}", cli.err());
}

#[test]
fn parse_diff_portfolio_block_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "portfolio",
        "--block1",
        "0",
        "--block2",
        "4294967296",
    ]);
    assert!(cli.is_err(), "diff portfolio block overflow should fail");
}

#[test]
fn parse_diff_subnet_missing_block2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "subnet", "--netuid", "1", "--block1", "100",
    ]);
    assert!(cli.is_err(), "diff subnet missing block2 should fail");
}

#[test]
fn parse_diff_metagraph_missing_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "diff",
        "metagraph",
        "--block1",
        "100",
        "--block2",
        "200",
    ]);
    assert!(cli.is_err(), "diff metagraph missing netuid should fail");
}

#[test]
fn parse_diff_network_zero_blocks() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "network", "--block1", "0", "--block2", "0",
    ]);
    assert!(cli.is_ok(), "diff network zero blocks: {:?}", cli.err());
}

#[test]
fn parse_diff_subnet_max_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "subnet", "--netuid", "65535", "--block1", "100", "--block2", "200",
    ]);
    assert!(cli.is_ok(), "diff subnet max netuid: {:?}", cli.err());
}

// =====================================================================
// Block — expanded boundary tests
// =====================================================================

#[test]
fn parse_block_info_max() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "info", "--number", "4294967295"]);
    assert!(cli.is_ok(), "block info max u32: {:?}", cli.err());
}

#[test]
fn parse_block_info_zero() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "info", "--number", "0"]);
    assert!(cli.is_ok(), "block info zero: {:?}", cli.err());
}

#[test]
fn parse_block_info_overflow() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "info", "--number", "4294967296"]);
    assert!(cli.is_err(), "block info u32 overflow should fail");
}

#[test]
fn parse_block_info_missing_number() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "info"]);
    assert!(cli.is_err(), "block info missing number should fail");
}

#[test]
fn parse_block_latest_with_json() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "block", "latest"]);
    assert!(cli.is_ok(), "block latest json: {:?}", cli.err());
}

#[test]
fn parse_block_range_same_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "block", "range", "--from", "100", "--to", "100",
    ]);
    assert!(cli.is_ok(), "block range same block: {:?}", cli.err());
}

#[test]
fn parse_block_range_max_values() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "block",
        "range",
        "--from",
        "4294967294",
        "--to",
        "4294967295",
    ]);
    assert!(cli.is_ok(), "block range max u32: {:?}", cli.err());
}

#[test]
fn parse_block_range_zero_start() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "block", "range", "--from", "0", "--to", "10"]);
    assert!(cli.is_ok(), "block range from zero: {:?}", cli.err());
}

#[test]
fn parse_block_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--network", "finney", "block", "latest"]);
    assert!(cli.is_ok(), "block with global flags: {:?}", cli.err());
}

// =====================================================================
// Utils — expanded boundary tests
// =====================================================================

#[test]
fn parse_utils_convert_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "convert", "--amount", "1.0"]);
    assert!(cli.is_ok(), "utils convert default: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_with_to_rao() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--amount", "1.0", "--to-rao",
    ]);
    assert!(cli.is_ok(), "utils convert to-rao: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_no_args() {
    // All convert args are optional
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "convert"]);
    assert!(cli.is_ok(), "utils convert no args: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_alpha_with_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--alpha", "100.0", "--netuid", "1",
    ]);
    assert!(cli.is_ok(), "utils convert alpha: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_tao_with_netuid() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "utils", "convert", "--tao", "10.0", "--netuid", "5",
    ]);
    assert!(cli.is_ok(), "utils convert tao to alpha: {:?}", cli.err());
}

#[test]
fn parse_utils_latency_custom_pings() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "latency", "--pings", "10"]);
    assert!(cli.is_ok(), "utils latency custom pings: {:?}", cli.err());
}

#[test]
fn parse_utils_latency_one_ping() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "latency", "--pings", "1"]);
    assert!(cli.is_ok(), "utils latency one ping: {:?}", cli.err());
}

#[test]
fn parse_utils_latency_with_extra_endpoints() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "utils",
        "latency",
        "--extra",
        "ws://127.0.0.1:9944,ws://custom:9945",
        "--pings",
        "3",
    ]);
    assert!(
        cli.is_ok(),
        "utils latency extra endpoints: {:?}",
        cli.err()
    );
}

#[test]
fn parse_utils_latency_zero_pings() {
    // Zero pings should parse (runtime may fail, but CLI should accept)
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "latency", "--pings", "0"]);
    assert!(cli.is_ok(), "utils latency zero pings: {:?}", cli.err());
}

// =====================================================================
// Root — expanded tests
// =====================================================================

#[test]
fn parse_root_weights_multi_uids() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "root",
        "weights",
        "--weights",
        "0:100,1:50,2:25",
    ]);
    assert!(cli.is_ok(), "root weights multi uids: {:?}", cli.err());
}

#[test]
fn parse_root_weights_missing_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "weights"]);
    assert!(cli.is_err(), "root weights missing arg should fail");
}

#[test]
fn parse_root_register_with_json() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "root", "register"]);
    assert!(cli.is_ok(), "root register json: {:?}", cli.err());
}

#[test]
fn parse_root_weights_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "finney",
        "--wallet",
        "mywallet",
        "root",
        "weights",
        "--weights",
        "0:100",
    ]);
    assert!(cli.is_ok(), "root weights with global: {:?}", cli.err());
}

// =====================================================================
// Swap — expanded boundary tests
// =====================================================================

#[test]
fn parse_swap_hotkey_with_global() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--wallet",
        "mywallet",
        "swap",
        "hotkey",
        "--new-hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "swap hotkey with global: {:?}", cli.err());
}

#[test]
fn parse_swap_coldkey_with_global() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "swap",
        "coldkey",
        "--new-coldkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "swap coldkey with global: {:?}", cli.err());
}

#[test]
fn parse_swap_hotkey_with_password() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--password",
        "test123",
        "swap",
        "hotkey",
        "--new-hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "swap hotkey with password: {:?}", cli.err());
}

// =====================================================================
// Swap — extended edge cases
// =====================================================================

#[test]
fn parse_swap_hotkey_missing_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "hotkey"]);
    assert!(cli.is_err(), "swap hotkey missing --new-hotkey should fail");
}

#[test]
fn parse_swap_coldkey_missing_arg() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap", "coldkey"]);
    assert!(
        cli.is_err(),
        "swap coldkey missing --new-coldkey should fail"
    );
}

#[test]
fn parse_swap_hotkey_with_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--dry-run",
        "swap",
        "hotkey",
        "--new-hotkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "swap hotkey dry-run: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn parse_swap_coldkey_with_yes_batch() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "-y",
        "--batch",
        "swap",
        "coldkey",
        "--new-coldkey",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "swap coldkey yes+batch: {:?}", cli.err());
    let c = cli.unwrap();
    assert!(c.yes);
    assert!(c.batch);
}

#[test]
fn parse_swap_no_subcommand() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "swap"]);
    assert!(cli.is_err(), "swap without subcommand should fail");
}

// =====================================================================
// Block — comprehensive tests
// =====================================================================

#[test]
fn parse_block_info_large_number() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "info", "--number", "4294967295"]);
    assert!(cli.is_ok(), "block info max u32: {:?}", cli.err());
}

#[test]
fn parse_block_info_negative() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block", "info", "--number", "-1"]);
    assert!(cli.is_err(), "block info negative should fail");
}

#[test]
fn parse_block_latest_json() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "block", "latest"]);
    assert!(cli.is_ok(), "block latest json: {:?}", cli.err());
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

#[test]
fn parse_block_no_subcommand() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "block"]);
    assert!(cli.is_err(), "block without subcommand should fail");
}

#[test]
fn parse_block_info_with_csv() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "--output", "csv", "block", "info", "--number", "50",
    ]);
    assert!(cli.is_ok(), "block info csv: {:?}", cli.err());
    assert_eq!(cli.unwrap().output, OutputFormat::Csv);
}

// =====================================================================
// Diff — comprehensive tests
// =====================================================================

#[test]
fn parse_diff_portfolio_missing_block1() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "diff", "portfolio", "--block2", "200"]);
    assert!(cli.is_err(), "diff portfolio missing block1 should fail");
}

#[test]
fn parse_diff_portfolio_missing_block2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "diff", "portfolio", "--block1", "100"]);
    assert!(cli.is_err(), "diff portfolio missing block2 should fail");
}

#[test]
fn parse_diff_network_same_block() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "diff", "network", "--block1", "500", "--block2", "500",
    ]);
    assert!(cli.is_ok(), "diff network same block: {:?}", cli.err());
}

#[test]
fn parse_diff_portfolio_json() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "diff",
        "portfolio",
        "--block1",
        "100",
        "--block2",
        "200",
    ]);
    assert!(cli.is_ok(), "diff portfolio json: {:?}", cli.err());
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

#[test]
fn parse_diff_no_subcommand() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "diff"]);
    assert!(cli.is_err(), "diff without subcommand should fail");
}

// =====================================================================
// Utils — comprehensive tests
// =====================================================================

#[test]
fn parse_utils_convert_from_rao() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "utils", "convert", "--amount", "1000000000"]);
    assert!(cli.is_ok(), "utils convert from rao: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_zero() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "convert", "--amount", "0"]);
    assert!(cli.is_ok(), "utils convert zero: {:?}", cli.err());
}

#[test]
fn parse_utils_convert_large_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "utils",
        "convert",
        "--amount",
        "999999999.999999999",
    ]);
    assert!(cli.is_ok(), "utils convert large: {:?}", cli.err());
}

#[test]
fn parse_utils_latency_pings_one() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils", "latency", "--pings", "1"]);
    assert!(cli.is_ok(), "utils latency 1 ping: {:?}", cli.err());
}

#[test]
fn parse_utils_no_subcommand() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "utils"]);
    assert!(cli.is_err(), "utils without subcommand should fail");
}

// =====================================================================
// Batch — comprehensive tests
// =====================================================================

#[test]
fn parse_batch_default_atomic() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "batch", "--file", "/tmp/calls.json"]);
    assert!(cli.is_ok(), "batch default: {:?}", cli.err());
    if let agcli::cli::Commands::Batch {
        file,
        no_atomic,
        force,
    } = &cli.unwrap().command
    {
        assert_eq!(file, "/tmp/calls.json");
        assert!(!no_atomic);
        assert!(!force);
    } else {
        panic!("expected Batch command");
    }
}

#[test]
fn parse_batch_force() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "batch", "--file", "/tmp/calls.json", "--force"]);
    assert!(cli.is_ok(), "batch force: {:?}", cli.err());
    if let agcli::cli::Commands::Batch { force, .. } = &cli.unwrap().command {
        assert!(force);
    } else {
        panic!("expected Batch command");
    }
}

#[test]
fn parse_batch_force_and_no_atomic() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "batch",
        "--file",
        "/tmp/calls.json",
        "--force",
        "--no-atomic",
    ]);
    assert!(cli.is_ok(), "batch force+no-atomic: {:?}", cli.err());
    if let agcli::cli::Commands::Batch {
        force, no_atomic, ..
    } = &cli.unwrap().command
    {
        assert!(force);
        assert!(no_atomic);
    } else {
        panic!("expected Batch command");
    }
}

#[test]
fn parse_batch_missing_file() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "batch"]);
    assert!(cli.is_err(), "batch missing file should fail");
}

#[test]
fn parse_batch_with_global_flags() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--wallet",
        "mywallet",
        "--password",
        "test123",
        "-y",
        "--batch",
        "batch",
        "--file",
        "/tmp/calls.json",
    ]);
    assert!(cli.is_ok(), "batch with globals: {:?}", cli.err());
    let c = cli.unwrap();
    assert!(c.yes);
    assert!(c.batch);
    assert_eq!(c.password, Some("test123".to_string()));
}

#[test]
fn parse_batch_with_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--dry-run",
        "batch",
        "--file",
        "/tmp/calls.json",
    ]);
    assert!(cli.is_ok(), "batch dry-run: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn parse_batch_with_json_output() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "batch",
        "--file",
        "/tmp/calls.json",
    ]);
    assert!(cli.is_ok(), "batch json output: {:?}", cli.err());
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

// =====================================================================
// Audit — comprehensive tests
// =====================================================================

#[test]
fn parse_audit_json() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "audit"]);
    assert!(cli.is_ok(), "audit json: {:?}", cli.err());
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

#[test]
fn parse_audit_csv() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "csv", "audit"]);
    assert!(cli.is_ok(), "audit csv: {:?}", cli.err());
}

#[test]
fn parse_audit_with_network_wallet() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--wallet",
        "mywallet",
        "audit",
    ]);
    assert!(cli.is_ok(), "audit with network: {:?}", cli.err());
}

// =====================================================================
// Explain — comprehensive tests
// =====================================================================

#[test]
fn parse_explain_commit_reveal() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "commit-reveal"]);
    assert!(cli.is_ok(), "explain commit-reveal: {:?}", cli.err());
}

#[test]
fn parse_explain_weights() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "weights"]);
    assert!(cli.is_ok(), "explain weights: {:?}", cli.err());
}

#[test]
fn parse_explain_weights_alias_settingweights() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "settingweights"]);
    assert!(cli.is_ok(), "explain settingweights: {:?}", cli.err());
}

#[test]
fn explain_weights_builtin_mentions_help_and_commit_reveal() {
    let text = agcli::utils::explain::explain("weights").expect("weights topic");
    assert!(
        text.contains("weights --help") && text.contains("commit-reveal"),
        "expected discoverability strings: {}",
        text
    );
}

#[test]
fn parse_explain_amm() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "amm"]);
    assert!(cli.is_ok(), "explain amm: {:?}", cli.err());
}

#[test]
fn parse_explain_full() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "explain", "--topic", "tempo", "--full"]);
    assert!(cli.is_ok(), "explain full: {:?}", cli.err());
}

// =====================================================================
// Completions — comprehensive tests
// =====================================================================

#[test]
fn parse_completions_bash() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "bash"]);
    assert!(cli.is_ok(), "completions bash: {:?}", cli.err());
}

#[test]
fn parse_completions_zsh() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "zsh"]);
    assert!(cli.is_ok(), "completions zsh: {:?}", cli.err());
}

#[test]
fn parse_completions_fish() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "fish"]);
    assert!(cli.is_ok(), "completions fish: {:?}", cli.err());
}

#[test]
fn parse_completions_powershell() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "powershell"]);
    assert!(cli.is_ok(), "completions powershell: {:?}", cli.err());
}

#[test]
fn parse_completions_invalid_shell() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions", "--shell", "tcsh"]);
    assert!(cli.is_err(), "completions invalid shell should fail");
}

#[test]
fn parse_completions_missing_shell() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "completions"]);
    assert!(cli.is_err(), "completions missing shell should fail");
}

// =====================================================================
// Doctor — tests
// =====================================================================

#[test]
fn parse_doctor() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "doctor"]);
    assert!(cli.is_ok(), "doctor: {:?}", cli.err());
}

#[test]
fn parse_doctor_json() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--output", "json", "doctor"]);
    assert!(cli.is_ok(), "doctor json: {:?}", cli.err());
    assert_eq!(cli.unwrap().output, OutputFormat::Json);
}

// =====================================================================
// Update — tests
// =====================================================================

// =====================================================================
// Balance — extended edge cases
// =====================================================================

#[test]
fn parse_balance_with_watch() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch"]);
    assert!(cli.is_ok(), "balance watch: {:?}", cli.err());
}

#[test]
fn parse_balance_with_watch_interval() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--watch", "30"]);
    assert!(cli.is_ok(), "balance watch interval: {:?}", cli.err());
}

#[test]
fn parse_balance_with_threshold() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--threshold", "10.5"]);
    assert!(cli.is_ok(), "balance threshold: {:?}", cli.err());
}

#[test]
fn parse_balance_with_at_block() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "balance", "--at-block", "5000000"]);
    assert!(cli.is_ok(), "balance at-block: {:?}", cli.err());
}

#[test]
fn parse_balance_with_address() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "balance",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "balance with address: {:?}", cli.err());
}

#[test]
fn parse_balance_watch_with_threshold() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "balance",
        "--watch",
        "60",
        "--threshold",
        "5.0",
    ]);
    assert!(cli.is_ok(), "balance watch+threshold: {:?}", cli.err());
}

#[test]
fn parse_balance_all_options() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--output",
        "json",
        "balance",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--watch",
        "10",
        "--threshold",
        "1.0",
        "--at-block",
        "100",
    ]);
    assert!(cli.is_ok(), "balance all options: {:?}", cli.err());
}

// =====================================================================
// Transfer — extended edge cases
// =====================================================================

#[test]
fn parse_transfer_small_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "0.000000001",
    ]);
    assert!(cli.is_ok(), "transfer tiny: {:?}", cli.err());
}

#[test]
fn parse_transfer_missing_dest() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "transfer", "--amount", "1.0"]);
    assert!(cli.is_err(), "transfer missing dest should fail");
}

#[test]
fn parse_transfer_missing_amount() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_err(), "transfer missing amount should fail");
}

#[test]
fn parse_transfer_all_keep_alive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer-all",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--keep-alive",
    ]);
    assert!(cli.is_ok(), "transfer-all keep-alive: {:?}", cli.err());
}

#[test]
fn parse_transfer_all_no_keep_alive() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "transfer-all",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
    ]);
    assert!(cli.is_ok(), "transfer-all no keep: {:?}", cli.err());
}

#[test]
fn parse_transfer_all_missing_dest() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "transfer-all"]);
    assert!(cli.is_err(), "transfer-all missing dest should fail");
}

#[test]
fn parse_transfer_with_dry_run_mev() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--dry-run",
        "--mev",
        "transfer",
        "--dest",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--amount",
        "1.0",
    ]);
    assert!(cli.is_ok(), "transfer dry+mev: {:?}", cli.err());
    let c = cli.unwrap();
    assert!(c.dry_run);
    assert!(c.mev);
}

// =====================================================================
// Global flags — extended edge cases
// =====================================================================

#[test]
fn parse_global_verbose() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "-v", "balance"]);
    assert!(cli.is_ok(), "verbose: {:?}", cli.err());
    assert!(cli.unwrap().verbose);
}

#[test]
fn parse_global_debug() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--debug", "balance"]);
    assert!(cli.is_ok(), "debug: {:?}", cli.err());
    assert!(cli.unwrap().debug);
}

#[test]
fn parse_global_timeout() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--timeout", "30", "balance"]);
    assert!(cli.is_ok(), "timeout: {:?}", cli.err());
    assert_eq!(cli.unwrap().timeout, Some(30));
}

#[test]
fn parse_global_time() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--time", "balance"]);
    assert!(cli.is_ok(), "time: {:?}", cli.err());
    assert!(cli.unwrap().time);
}

#[test]
fn parse_global_best() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--best", "balance"]);
    assert!(cli.is_ok(), "best: {:?}", cli.err());
    assert!(cli.unwrap().best);
}

#[test]
fn parse_global_pretty() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--pretty", "--output", "json", "balance"]);
    assert!(cli.is_ok(), "pretty: {:?}", cli.err());
    let c = cli.unwrap();
    assert!(c.pretty);
    assert_eq!(c.output, OutputFormat::Json);
}

#[test]
fn parse_global_proxy() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--proxy",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "balance",
    ]);
    assert!(cli.is_ok(), "proxy: {:?}", cli.err());
    assert!(cli.unwrap().proxy.is_some());
}

#[test]
fn parse_global_endpoint_overrides_network() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "finney",
        "--endpoint",
        "ws://custom:9944",
        "balance",
    ]);
    assert!(cli.is_ok(), "endpoint override: {:?}", cli.err());
    let c = cli.unwrap();
    assert_eq!(c.endpoint, Some("ws://custom:9944".to_string()));
}

#[test]
fn parse_global_all_flags_combined() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--endpoint",
        "ws://127.0.0.1:9944",
        "--wallet",
        "mywal",
        "--hotkey-name",
        "myhk",
        "--output",
        "json",
        "--pretty",
        "-v",
        "--debug",
        "--time",
        "--best",
        "--dry-run",
        "--mev",
        "-y",
        "--batch",
        "--timeout",
        "60",
        "--password",
        "secret",
        "--log-file",
        "/tmp/all.log",
        "--proxy",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "balance",
    ]);
    assert!(cli.is_ok(), "all flags: {:?}", cli.err());
    let c = cli.unwrap();
    assert!(c.verbose);
    assert!(c.debug);
    assert!(c.time);
    assert!(c.best);
    assert!(c.dry_run);
    assert!(c.mev);
    assert!(c.yes);
    assert!(c.batch);
    assert!(c.pretty);
    assert_eq!(c.timeout, Some(60));
    assert_eq!(c.output, OutputFormat::Json);
}

// =====================================================================
// Root — extended edge cases
// =====================================================================

#[test]
fn parse_root_register_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "--dry-run", "root", "register"]);
    assert!(cli.is_ok(), "root register dry-run: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn parse_root_register_with_password() {
    let cli =
        agcli::cli::Cli::try_parse_from(["agcli", "--password", "secret", "root", "register"]);
    assert!(cli.is_ok(), "root register with password: {:?}", cli.err());
}

#[test]
fn parse_root_weights_multi_pair() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "root",
        "weights",
        "--weights",
        "0:50,1:30,2:20",
    ]);
    assert!(cli.is_ok(), "root weights multi: {:?}", cli.err());
}

#[test]
fn parse_root_weights_single_pair() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root", "weights", "--weights", "0:100"]);
    assert!(cli.is_ok(), "root weights single: {:?}", cli.err());
}

#[test]
fn parse_root_weights_with_dry_run() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--dry-run",
        "root",
        "weights",
        "--weights",
        "0:50,1:50",
    ]);
    assert!(cli.is_ok(), "root weights dry: {:?}", cli.err());
    assert!(cli.unwrap().dry_run);
}

#[test]
fn parse_root_no_subcommand() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "root"]);
    assert!(cli.is_err(), "root without subcommand should fail");
}

// =====================================================================
// Config — extended edge cases
// =====================================================================

#[test]
fn parse_config_set_missing_key() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "set", "--value", "test"]);
    assert!(cli.is_err(), "config set missing key should fail");
}

#[test]
fn parse_config_set_missing_value() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "set", "--key", "network"]);
    assert!(cli.is_err(), "config set missing value should fail");
}

#[test]
fn parse_config_unset_missing_key() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config", "unset"]);
    assert!(cli.is_err(), "config unset missing key should fail");
}

#[test]
fn parse_config_no_subcommand() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "config"]);
    assert!(cli.is_err(), "config without subcommand should fail");
}

// =====================================================================
// EVM — gas limit boundary tests
// =====================================================================

#[test]
fn parse_evm_call_custom_gas() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--gas-limit",
        "100000",
    ]);
    assert!(cli.is_ok(), "evm call custom gas: {:?}", cli.err());
}

#[test]
fn parse_evm_call_max_gas() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--gas-limit",
        "18446744073709551615",
    ]);
    assert!(cli.is_ok(), "evm call max u64 gas: {:?}", cli.err());
}

#[test]
fn parse_evm_call_gas_overflow() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--gas-limit",
        "18446744073709551616",
    ]);
    assert!(cli.is_err(), "evm call gas overflow should fail");
}

#[test]
fn parse_evm_call_gas_zero() {
    // Zero gas parses but will fail at runtime validation
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "evm",
        "call",
        "--source",
        "0x0000000000000000000000000000000000000001",
        "--target",
        "0x0000000000000000000000000000000000000002",
        "--gas-limit",
        "0",
    ]);
    assert!(cli.is_ok(), "evm call gas zero parses: {:?}", cli.err());
}

// =====================================================================
// Subscribe — extended edge cases
// =====================================================================

#[test]
fn parse_subscribe_no_subcommand() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "subscribe"]);
    assert!(cli.is_err(), "subscribe without subcommand should fail");
}

// =====================================================================
// Contracts — extended edge cases
// =====================================================================

#[test]
fn parse_contracts_upload_with_global() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "--network",
        "test",
        "--wallet",
        "myw",
        "contracts",
        "upload",
        "--code",
        "/path/to/contract.wasm",
    ]);
    assert!(cli.is_ok(), "contracts upload global: {:?}", cli.err());
}

#[test]
fn parse_contracts_upload_with_deposit_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "contracts",
        "upload",
        "--code",
        "/path/to/contract.wasm",
        "--storage-deposit-limit",
        "1000000000",
    ]);
    assert!(cli.is_ok(), "contracts upload deposit: {:?}", cli.err());
}

// =====================================================================
// Multisig — extended edge cases
// =====================================================================

#[test]
fn parse_multisig_address_threshold_1() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "address",
        "--signatories", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold", "1",
    ]);
    assert!(cli.is_ok(), "multisig address t=1: {:?}", cli.err());
}

#[test]
fn parse_multisig_address_max_threshold() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "multisig", "address",
        "--signatories", "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY,5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty",
        "--threshold", "2",
    ]);
    assert!(cli.is_ok(), "multisig address t=2: {:?}", cli.err());
}

#[test]
fn parse_multisig_submit_with_json_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "submit",
        "--others",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--threshold",
        "2",
        "--pallet",
        "Balances",
        "--call",
        "transfer_allow_death",
        "--args",
        "[\"5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty\", 1000000000]",
    ]);
    assert!(cli.is_ok(), "multisig submit args: {:?}", cli.err());
}

#[test]
fn parse_multisig_submit_missing_others() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "multisig",
        "submit",
        "--threshold",
        "2",
        "--pallet",
        "Balances",
        "--call",
        "transfer",
    ]);
    assert!(cli.is_err(), "multisig submit missing others should fail");
}

// =====================================================================
// View commands — extra coverage (Step 14)
// =====================================================================

#[test]
fn parse_view_portfolio_default() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "portfolio"]);
    assert!(cli.is_ok(), "view portfolio default: {:?}", cli.err());
}

#[test]
fn parse_view_portfolio_both_args() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "portfolio",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--at-block",
        "500",
    ]);
    assert!(cli.is_ok(), "view portfolio both: {:?}", cli.err());
}

#[test]
fn parse_view_neuron_missing_netuid_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "neuron", "--uid", "0"]);
    assert!(cli.is_err(), "view neuron missing netuid should fail");
}

#[test]
fn parse_view_network_default_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "network"]);
    assert!(cli.is_ok(), "view network default: {:?}", cli.err());
}

#[test]
fn parse_view_dynamic_default_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "dynamic"]);
    assert!(cli.is_ok(), "view dynamic default: {:?}", cli.err());
}

#[test]
fn parse_view_validators_default_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "validators"]);
    assert!(cli.is_ok(), "view validators default: {:?}", cli.err());
}

#[test]
fn parse_view_validators_with_limit_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "validators", "--limit", "100"]);
    assert!(cli.is_ok(), "view validators limit: {:?}", cli.err());
}

#[test]
fn parse_view_validators_all_args_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "validators",
        "--netuid",
        "5",
        "--limit",
        "25",
        "--at-block",
        "2000000",
    ]);
    assert!(cli.is_ok(), "view validators all args: {:?}", cli.err());
}

#[test]
fn parse_view_history_default_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "history"]);
    assert!(cli.is_ok(), "view history default: {:?}", cli.err());
}

#[test]
fn parse_view_history_with_limit_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "history", "--limit", "100"]);
    assert!(cli.is_ok(), "view history limit: {:?}", cli.err());
}

#[test]
fn parse_view_history_with_address_and_limit() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "history",
        "--address",
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
        "--limit",
        "10",
    ]);
    assert!(cli.is_ok(), "view history addr+limit: {:?}", cli.err());
}

#[test]
fn parse_view_account_default_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "account"]);
    assert!(cli.is_ok(), "view account default: {:?}", cli.err());
}

#[test]
fn parse_view_staking_analytics_default_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "staking-analytics"]);
    assert!(cli.is_ok(), "view staking-analytics: {:?}", cli.err());
}

#[test]
fn parse_view_swap_sim_alpha_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli", "view", "swap-sim", "--netuid", "1", "--alpha", "100.0",
    ]);
    assert!(cli.is_ok(), "view swap-sim alpha: {:?}", cli.err());
}

#[test]
fn parse_view_metagraph_all_args_v2() {
    let cli = agcli::cli::Cli::try_parse_from([
        "agcli",
        "view",
        "metagraph",
        "--netuid",
        "1",
        "--since-block",
        "100000",
        "--limit",
        "10",
    ]);
    assert!(cli.is_ok(), "view metagraph all args: {:?}", cli.err());
}

#[test]
fn parse_view_metagraph_missing_netuid_v2() {
    let cli = agcli::cli::Cli::try_parse_from(["agcli", "view", "metagraph"]);
    assert!(cli.is_err(), "view metagraph missing netuid should fail");
}

