//! Typed error codes for scripting and automation.
//!
//! Each error category maps to a distinct exit code so callers can
//! distinguish between "retry later" (network) and "fix your input"
//! (validation) without parsing stderr.

/// Exit codes used by agcli.
///
/// Standard convention: 0 = success, 1 = generic, 2+ = categorized.
pub mod exit_code {
    /// Generic / uncategorized error.
    pub const GENERIC: i32 = 1;
    /// Network / connection error (server unreachable, timeout, DNS failure).
    pub const NETWORK: i32 = 10;
    /// Authentication / wallet error (wrong password, missing key, locked wallet).
    pub const AUTH: i32 = 11;
    /// Validation error (bad input, invalid address, parse failure).
    pub const VALIDATION: i32 = 12;
    /// Chain/runtime error (extrinsic rejected, insufficient balance, rate-limited).
    pub const CHAIN: i32 = 13;
    /// File I/O error (permission denied, missing file, disk full).
    pub const IO: i32 = 14;
    /// Timeout (operation exceeded deadline).
    pub const TIMEOUT: i32 = 15;
}

/// Classify an anyhow error chain into an exit code.
pub fn classify(err: &anyhow::Error) -> i32 {
    let msg = format!("{:#}", err).to_lowercase();

    // Walk the chain for typed errors
    for cause in err.chain() {
        // Network errors
        if let Some(re) = cause.downcast_ref::<reqwest::Error>() {
            if re.is_timeout() {
                return exit_code::TIMEOUT;
            }
            return exit_code::NETWORK;
        }
        if cause.downcast_ref::<serde_json::Error>().is_some() {
            return exit_code::VALIDATION;
        }
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            match io.kind() {
                std::io::ErrorKind::NotFound
                | std::io::ErrorKind::PermissionDenied
                | std::io::ErrorKind::AlreadyExists => return exit_code::IO,
                std::io::ErrorKind::TimedOut => return exit_code::TIMEOUT,
                std::io::ErrorKind::ConnectionRefused
                | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted => return exit_code::NETWORK,
                _ => {}
            }
        }
    }

    // Heuristic classification from error messages
    if msg.contains("wrong password")
        || msg.contains("decryption failed")
        || msg.contains("no hotkey loaded")
        || msg.contains("unlock")
        || msg.contains("no coldkey")
        || msg.contains("keyfile")
    {
        return exit_code::AUTH;
    }

    if msg.contains("invalid ss58")
        || msg.contains("failed to parse")
        || msg.contains("invalid address")
        || msg.contains("not a valid")
        || msg.contains("must be ")
        || msg.contains("expected format")
        // `agcli subscribe events --filter …` typo / unknown category (`validate_event_filter`)
        || msg.contains("invalid event filter")
        // `agcli explain --topic …` typo / unknown built-in topic
        || msg.contains("unknown topic")
        // Query path: unknown / inactive netuid (e.g. `subnet show`)
        || (msg.contains("subnet") && msg.contains("not found"))
        // `agcli balance --threshold` (`validate_threshold` in helpers.rs)
        || msg.contains("balance --threshold")
    {
        return exit_code::VALIDATION;
    }

    // Chain errors checked BEFORE network — "insufficient" or "extrinsic" in a
    // message like "failed to connect extrinsic …" must classify as CHAIN, not NETWORK.
    if msg.contains("insufficient")
        || msg.contains("rate limit")
        || msg.contains("extrinsic")
        || msg.contains("dispatch")
        || msg.contains("nonce")
        // Subtensor-specific dispatch errors
        || msg.contains("notenoughstake")
        || msg.contains("not enough stake")
        || msg.contains("hotkey not registered")
        || msg.contains("hotkeynotregistered")
        || msg.contains("slippagetoo")
        || msg.contains("slippage too")
        || msg.contains("subnet not exist")
        || msg.contains("subnetnotexist")
        || msg.contains("not subnet owner")
        || msg.contains("notsubnetowner")
        || msg.contains("registration disabled")
        || msg.contains("registrationdisabled")
        || msg.contains("calldisabled")
        || msg.contains("call disabled")
        || msg.contains("insufficientliquidity")
        || msg.contains("liquidity")
        || msg.contains("delegatetaketoo")
        || msg.contains("invalidchild")
    {
        return exit_code::CHAIN;
    }

    if msg.contains("timeout") || msg.contains("timed out") {
        return exit_code::TIMEOUT;
    }

    // Use more specific patterns to avoid false positives on words like "endpoint"
    // appearing in unrelated chain error messages.
    if msg.contains("connection refused")
        || msg.contains("failed to connect")
        || msg.contains("dns")
        || msg.contains("websocket")
        || msg.contains("unreachable")
    {
        return exit_code::NETWORK;
    }

    if msg.contains("permission denied")
        || msg.contains("no such file")
        || msg.contains("cannot read")
        || msg.contains("cannot write")
        || msg.contains("cannot create")
    {
        return exit_code::IO;
    }

    exit_code::GENERIC
}

/// Provide an actionable hint based on the error code and message.
pub fn hint(code: i32, msg: &str) -> Option<&'static str> {
    let lower = msg.to_lowercase();
    match code {
        exit_code::NETWORK => {
            if lower.contains("dns") {
                Some("Tip: Check your DNS settings or try a different endpoint with --endpoint <url>")
            } else if lower.contains("refused") || lower.contains("unreachable") {
                Some("Tip: The chain endpoint may be down. Try --endpoint wss://entrypoint-finney.opentensor.ai:443 or check your network connection")
            } else {
                Some("Tip: Check your internet connection, or try a different endpoint with --endpoint <url>")
            }
        }
        exit_code::AUTH => {
            if lower.contains("password") {
                Some("Tip: Verify your password. Use AGCLI_PASSWORD env var for non-interactive mode")
            } else if lower.contains("hotkey") {
                Some("Tip: Create a hotkey with `agcli wallet create-hotkey` or pass --hotkey-address <ss58>")
            } else {
                Some("Tip: Check wallet path with --wallet-dir and --wallet flags. List wallets: `agcli wallet list`")
            }
        }
        exit_code::TIMEOUT => {
            Some("Tip: Increase timeout with --timeout <seconds> (default: none). The chain may be congested")
        }
        exit_code::CHAIN => {
            if lower.contains("insufficient") {
                Some("Tip: Check your balance with `agcli balance`. Transaction fees require a small reserve")
            } else if lower.contains("rate limit") {
                Some("Tip: Wait a few blocks before retrying. Use `agcli block latest` to check block progress")
            } else if lower.contains("nonce") {
                Some("Tip: Another transaction may be pending. Wait for it to finalize before retrying")
            } else if lower.contains("not enough stake") || lower.contains("notenoughstake") {
                Some("Tip: Check your stake with `agcli stake list`. You may need more stake to perform this operation")
            } else if lower.contains("hotkey not registered") || lower.contains("hotkeynotregistered") {
                Some("Tip: Register your hotkey on the subnet first: `agcli subnet register-neuron --netuid <N>`")
            } else if lower.contains("slippage") {
                Some("Tip: The price moved too much. Use a higher --max-slippage or --price to allow more slippage")
            } else if lower.contains("subnet not exist") || lower.contains("subnetnotexist") {
                Some("Tip: Check available subnets with `agcli subnet list`")
            } else if lower.contains("not subnet owner") || lower.contains("notsubnetowner") {
                Some("Tip: Only the subnet owner can perform this operation. Check ownership with `agcli subnet show --netuid <N>`")
            } else if lower.contains("registration disabled") || lower.contains("registrationdisabled") {
                Some("Tip: Registration is currently disabled on this subnet. Check `agcli subnet hyperparams --netuid <N>`")
            } else if lower.contains("call disabled") || lower.contains("calldisabled") {
                Some("Tip: This call is currently disabled on this subnet")
            } else {
                Some("Tip: The chain rejected this operation. Check `agcli doctor` for diagnostic info")
            }
        }
        exit_code::IO => {
            Some("Tip: Check file permissions and paths. Use --wallet-dir to specify wallet location")
        }
        exit_code::VALIDATION => {
            if lower.contains("unknown topic") {
                Some("Tip: Run `agcli explain` to list topics. For weight flows: `agcli explain --topic weights` or `agcli weights --help`")
            } else if lower.contains("invalid event filter") {
                Some("Tip: Run `agcli subscribe events --help` and `docs/commands/subscribe.md` for `--filter` values; start with `--filter all`")
            } else if lower.contains("subnet") && lower.contains("not found") {
                Some("Tip: List subnets with `agcli subnet list`, then `agcli subnet show --netuid <N>` (alias: `subnet info`), `agcli subnet hyperparams --netuid <N>`, `agcli subnet metagraph --netuid <N>`, `agcli diff subnet --netuid <N>`, `agcli diff metagraph --netuid <N>`, `agcli subnet cost --netuid <N>`, `agcli subnet emissions --netuid <N>`, `agcli subnet health --netuid <N>`, `agcli subnet probe --netuid <N>`, `agcli subnet commits --netuid <N>`, `agcli subnet watch --netuid <N>`, `agcli subnet monitor --netuid <N>`, `agcli subnet liquidity --netuid <N>`, `agcli subnet cache-load --netuid <N>`, `agcli subnet cache-list --netuid <N>`, `agcli subnet cache-diff --netuid <N>`, `agcli subnet cache-prune --netuid <N>`, `agcli subnet emission-split --netuid <N>`, `agcli subnet mechanism-count --netuid <N>`, `agcli subnet set-mechanism-count --netuid <N>`, `agcli subnet set-emission-split --netuid <N>`, `agcli subnet check-start --netuid <N>`, `agcli subnet start --netuid <N>`, `agcli subnet snipe --netuid <N>`, `agcli subnet set-param --netuid <N>`, `agcli subnet set-symbol --netuid <N>`, `agcli subnet trim --netuid <N>`, `agcli subnet register-neuron --netuid <N>`, `agcli subnet pow --netuid <N>`, `agcli subnet dissolve --netuid <N>`, `agcli subnet root-dissolve --netuid <N>`, `agcli subnet terminate-lease --netuid <N>`, `agcli weights set --netuid <N>`, `agcli weights commit --netuid <N>`, `agcli weights reveal --netuid <N>`, `agcli weights commit-reveal --netuid <N>`, `agcli weights status --netuid <N>`, `agcli weights commit-timelocked --netuid <N>`, `agcli weights set-mechanism --netuid <N>`, `agcli weights commit-mechanism --netuid <N>`, `agcli weights reveal-mechanism --netuid <N>`, or `agcli weights show --netuid <N>`")
            } else if lower.contains("balance --threshold") {
                Some("Tip: Use a non-negative finite TAO amount, e.g. `agcli balance --watch --threshold 1.0` (see `docs/commands/balance.md`)")
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_wrong_password() {
        let err = anyhow::anyhow!("Decryption failed — wrong password");
        assert_eq!(classify(&err), exit_code::AUTH);
    }

    #[test]
    fn classify_invalid_address() {
        let err = anyhow::anyhow!("Invalid SS58 address: bad checksum");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn classify_subscribe_invalid_event_filter() {
        let err = anyhow::anyhow!(
            "Invalid event filter: 'nope'. Valid filters are: all, staking, registration, transfer, weights, subnet, delegation, keys, swap, governance, crowdloan (plus short aliases like stake, reg, dex).\n  Tip: use --filter all to see every decoded event."
        );
        assert_eq!(classify(&err), exit_code::VALIDATION);
        let msg = format!("{err:#}");
        let h = hint(exit_code::VALIDATION, &msg);
        assert!(h.is_some_and(|s| s.contains("subscribe events")));
    }

    #[test]
    fn classify_connection_error() {
        let err = anyhow::anyhow!("Failed to connect to endpoint wss://...");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_insufficient_balance() {
        let err = anyhow::anyhow!("Extrinsic failed: insufficient balance for transfer");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_timeout() {
        let err = anyhow::anyhow!("Operation timed out after 30s");
        assert_eq!(classify(&err), exit_code::TIMEOUT);
    }

    #[test]
    fn classify_io_error() {
        let err = anyhow::anyhow!("Permission denied writing to /etc/foo");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn classify_generic() {
        let err = anyhow::anyhow!("Something unexpected happened");
        assert_eq!(classify(&err), exit_code::GENERIC);
    }

    #[test]
    fn classify_subnet_show_not_found() {
        let err =
            anyhow::anyhow!("Subnet 99999 not found.\n  List available subnets: agcli subnet list");
        assert_eq!(classify(&err), exit_code::VALIDATION);
        let msg = format!("{err:#}");
        let h = hint(exit_code::VALIDATION, &msg);
        assert!(h.is_some_and(|s| {
            s.contains("subnet list")
                && s.contains("hyperparams")
                && s.contains("metagraph")
                && s.contains("cost")
                && s.contains("emissions")
                && s.contains("health")
                && s.contains("probe")
                && s.contains("commits")
                && s.contains("watch")
                && s.contains("monitor")
                && s.contains("liquidity")
                && s.contains("cache-load")
                && s.contains("cache-list")
                && s.contains("cache-diff")
                && s.contains("cache-prune")
                && s.contains("emission-split")
                && s.contains("mechanism-count")
                && s.contains("check-start")
                && s.contains("subnet start")
                && s.contains("snipe")
                && s.contains("set-param")
                && s.contains("set-symbol")
                && s.contains("trim")
                && s.contains("register-neuron")
                && s.contains("subnet pow")
                && s.contains("subnet dissolve")
                && s.contains("root-dissolve")
                && s.contains("terminate-lease")
                && s.contains("set-mechanism-count")
                && s.contains("set-emission-split")
                && s.contains("weights set")
                && s.contains("weights commit")
                && s.contains("weights reveal")
                && s.contains("weights commit-reveal")
                && s.contains("weights status")
                && s.contains("weights commit-timelocked")
                && s.contains("weights set-mechanism")
                && s.contains("weights commit-mechanism")
                && s.contains("weights reveal-mechanism")
                && s.contains("weights show")
                && s.contains("diff subnet")
                && s.contains("diff metagraph")
        }));
    }

    #[test]
    fn classify_diff_subnet_not_found_at_block_validation_12() {
        let err = anyhow::anyhow!("Subnet 8 not found at block 100");
        assert_eq!(classify(&err), exit_code::VALIDATION);
        let msg = format!("{err:#}");
        let h = hint(exit_code::VALIDATION, &msg);
        assert!(h.is_some_and(|s| s.contains("diff subnet") && s.contains("diff metagraph")));
    }

    #[test]
    fn classify_subnet_hyperparams_not_found_same_as_show() {
        // `subnet hyperparams` uses the same bail text as `subnet show` when SN is missing.
        let err =
            anyhow::anyhow!("Subnet 42 not found.\n  List available subnets: agcli subnet list");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn classify_weights_set_subnet_not_found_same_as_show() {
        // Weight commands bail with this text when hyperparams are absent (set, CR paths, timelocked, mechanism, status, `weights show`).
        let err =
            anyhow::anyhow!("Subnet 7 not found.\n  List available subnets: agcli subnet list");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn classify_explain_unknown_topic() {
        let err = anyhow::anyhow!("Unknown topic 'not-a-real-topic'");
        assert_eq!(classify(&err), exit_code::VALIDATION);
        let msg = format!("{err:#}");
        let h = hint(exit_code::VALIDATION, &msg);
        assert!(h.is_some_and(|s| s.contains("explain") && s.contains("weights")));
    }

    #[test]
    fn classify_chained_io_error() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "cannot write keyfile");
        let err = anyhow::Error::new(io_err).context("Writing wallet coldkey");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn classify_chained_connection_error() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let err = anyhow::Error::new(io_err).context("Connecting to finney");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_nonce_error() {
        let err = anyhow::anyhow!("Nonce already used for this account");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_dns_error() {
        let err = anyhow::anyhow!("DNS resolution failed for entrypoint-finney.opentensor.ai");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_rate_limit() {
        let err = anyhow::anyhow!("Rate limit exceeded: too many staking operations");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_no_such_file() {
        let err = anyhow::anyhow!("No such file or directory: ~/.bittensor/wallets/default");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn classify_websocket() {
        let err = anyhow::anyhow!("WebSocket connection dropped unexpectedly");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_empty_error() {
        let err = anyhow::anyhow!("");
        assert_eq!(classify(&err), exit_code::GENERIC);
    }

    #[test]
    fn classify_case_insensitive() {
        let err = anyhow::anyhow!("TIMEOUT waiting for block finalization");
        assert_eq!(classify(&err), exit_code::TIMEOUT);
    }

    // ──── hint() tests ────

    #[test]
    fn classify_serde_json_error() {
        let json_err: serde_json::Error =
            serde_json::from_str::<String>("not valid json").unwrap_err();
        let err = anyhow::Error::new(json_err).context("Failed to deserialize chain data");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn hint_network_dns() {
        let h = hint(exit_code::NETWORK, "DNS resolution failed");
        assert!(h.is_some());
        assert!(h.unwrap().contains("DNS"));
    }

    #[test]
    fn hint_network_refused() {
        let h = hint(exit_code::NETWORK, "Connection refused");
        assert!(h.is_some());
        assert!(h.unwrap().contains("endpoint"));
    }

    #[test]
    fn hint_auth_password() {
        let h = hint(exit_code::AUTH, "wrong password");
        assert!(h.is_some());
        assert!(h.unwrap().contains("password"));
    }

    #[test]
    fn hint_auth_hotkey() {
        let h = hint(exit_code::AUTH, "No hotkey loaded");
        assert!(h.is_some());
        assert!(h.unwrap().contains("hotkey"));
    }

    #[test]
    fn hint_timeout() {
        let h = hint(exit_code::TIMEOUT, "timed out");
        assert!(h.is_some());
        assert!(h.unwrap().contains("--timeout"));
    }

    #[test]
    fn hint_chain_insufficient() {
        let h = hint(exit_code::CHAIN, "insufficient balance");
        assert!(h.is_some());
        assert!(h.unwrap().contains("balance"));
    }

    #[test]
    fn hint_chain_nonce() {
        let h = hint(exit_code::CHAIN, "Nonce already used");
        assert!(h.is_some());
        assert!(h.unwrap().contains("pending"));
    }

    #[test]
    fn hint_io() {
        let h = hint(exit_code::IO, "permission denied");
        assert!(h.is_some());
    }

    #[test]
    fn hint_validation_none() {
        // Validation errors don't get hints (they already contain specific messages)
        let h = hint(exit_code::VALIDATION, "invalid input");
        assert!(h.is_none());
    }

    #[test]
    fn hint_generic_none() {
        let h = hint(exit_code::GENERIC, "unknown error");
        assert!(h.is_none());
    }

    // ──── Additional classify() coverage ────

    #[test]
    fn classify_dispatch_error() {
        let err = anyhow::anyhow!("Dispatch error: extrinsic failed on chain");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_cannot_create() {
        let err = anyhow::anyhow!("Cannot create directory: /foo/bar");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn classify_not_valid() {
        let err = anyhow::anyhow!("Input is not a valid netuid");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn classify_expected_format() {
        let err = anyhow::anyhow!("Expected format: ss58 address");
        assert_eq!(classify(&err), exit_code::VALIDATION);
    }

    #[test]
    fn classify_unreachable() {
        let err = anyhow::anyhow!("Host unreachable: entrypoint-finney.opentensor.ai");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_cannot_read() {
        // "Cannot read" matches IO, but "keyfile" matches AUTH first — AUTH takes priority
        let err = anyhow::anyhow!("Cannot read wallet keyfile");
        assert_eq!(classify(&err), exit_code::AUTH);
    }

    #[test]
    fn classify_cannot_read_generic_file() {
        let err = anyhow::anyhow!("Cannot read config file /etc/foo");
        assert_eq!(classify(&err), exit_code::IO);
    }

    #[test]
    fn hint_chain_rate_limit() {
        let h = hint(exit_code::CHAIN, "Rate limit exceeded");
        assert!(h.is_some());
        assert!(h.unwrap().contains("Wait"));
    }

    #[test]
    fn hint_network_generic() {
        let h = hint(exit_code::NETWORK, "some network error");
        assert!(h.is_some());
        assert!(h.unwrap().contains("internet connection"));
    }

    #[test]
    fn hint_auth_generic() {
        let h = hint(exit_code::AUTH, "wallet locked");
        assert!(h.is_some());
        assert!(h.unwrap().contains("wallet"));
    }

    // ──── Issue 122 (v24): Error classify heuristic ordering ────

    #[test]
    fn classify_chain_before_network_insufficient() {
        // "insufficient" should classify as CHAIN even when "connect" also appears
        let err = anyhow::anyhow!("Failed to connect extrinsic to mempool: insufficient fee");
        assert_eq!(
            classify(&err),
            exit_code::CHAIN,
            "Chain errors must take priority over network heuristics"
        );
    }

    #[test]
    fn classify_chain_before_network_extrinsic() {
        // "extrinsic" alone should be CHAIN, not matched by "connect"
        let err = anyhow::anyhow!("Extrinsic dispatch error from endpoint");
        assert_eq!(
            classify(&err),
            exit_code::CHAIN,
            "'extrinsic' should match CHAIN before 'endpoint' matches NETWORK"
        );
    }

    #[test]
    fn classify_connection_refused_still_network() {
        // Specific network patterns should still work
        let err = anyhow::anyhow!("connection refused on wss://entrypoint.finney");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    #[test]
    fn classify_failed_to_connect_still_network() {
        let err = anyhow::anyhow!("failed to connect to chain endpoint");
        assert_eq!(classify(&err), exit_code::NETWORK);
    }

    // ──── Subtensor-specific dispatch error classification ────

    #[test]
    fn classify_not_enough_stake() {
        let err = anyhow::anyhow!("Dispatch error: NotEnoughStake");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_hotkey_not_registered() {
        let err = anyhow::anyhow!("Dispatch error: HotKeyNotRegisteredInSubNet");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_slippage_too_high() {
        let err = anyhow::anyhow!("Dispatch error: SlippageTooHigh");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_subnet_not_exists() {
        let err = anyhow::anyhow!("Dispatch error: SubnetNotExists");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_not_subnet_owner() {
        let err = anyhow::anyhow!("Dispatch error: NotSubnetOwner");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_registration_disabled() {
        let err = anyhow::anyhow!("Dispatch error: SubNetRegistrationDisabled");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_call_disabled() {
        let err = anyhow::anyhow!("Dispatch error: CallDisabled");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_insufficient_liquidity() {
        let err = anyhow::anyhow!("Dispatch error: InsufficientLiquidity");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_delegate_take_too_high() {
        let err = anyhow::anyhow!("Dispatch error: DelegateTakeTooHigh");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    #[test]
    fn classify_invalid_child() {
        let err = anyhow::anyhow!("Dispatch error: InvalidChild");
        assert_eq!(classify(&err), exit_code::CHAIN);
    }

    // ──── Subtensor-specific hint tests ────

    #[test]
    fn hint_chain_not_enough_stake() {
        let h = hint(exit_code::CHAIN, "NotEnoughStake: need more stake");
        assert!(h.is_some());
        assert!(h.unwrap().contains("stake"));
    }

    #[test]
    fn hint_chain_hotkey_not_registered() {
        let h = hint(exit_code::CHAIN, "HotKeyNotRegisteredInSubNet");
        assert!(h.is_some());
        assert!(h.unwrap().contains("register"));
    }

    #[test]
    fn hint_chain_slippage() {
        let h = hint(exit_code::CHAIN, "SlippageTooHigh on swap");
        assert!(h.is_some());
        assert!(h.unwrap().contains("slippage"));
    }

    #[test]
    fn hint_chain_subnet_not_exists() {
        let h = hint(exit_code::CHAIN, "Subnet not exist error");
        assert!(h.is_some());
        assert!(h.unwrap().contains("subnet list"));
    }

    #[test]
    fn hint_chain_not_owner() {
        let h = hint(exit_code::CHAIN, "NotSubnetOwner: not authorized");
        assert!(h.is_some());
        assert!(h.unwrap().contains("owner"));
    }

    #[test]
    fn hint_chain_registration_disabled() {
        let h = hint(exit_code::CHAIN, "Registration disabled on subnet");
        assert!(h.is_some());
        assert!(h.unwrap().contains("disabled"));
    }
}
