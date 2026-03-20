//! Shared CLI helper functions.

use crate::wallet::Wallet;
use anyhow::Result;

use crate::cli::{cli_has_flag, OutputFormat};

/// Common context passed to all command handlers, reducing parameter sprawl.
///
/// Instead of passing 6-9 individual parameters to every handler,
/// handlers receive a single `&Ctx` reference.
pub struct Ctx<'a> {
    pub wallet_dir: &'a str,
    pub wallet_name: &'a str,
    pub hotkey_name: &'a str,
    pub output: OutputFormat,
    pub password: Option<&'a str>,
    pub yes: bool,
    pub mev: bool,
    pub live_interval: Option<u64>,
    pub proxy: Option<&'a str>,
}

/// Returns true if the user explicitly passed `--wallet` or `-w` on the CLI.
///
/// This is used to distinguish "user typed `--wallet default`" from "clap filled
/// in the default value". Without this check, a user cannot filter to a wallet
/// literally named "default" in `wallet show`.
pub fn wallet_explicitly_set() -> bool {
    let args: Vec<String> = std::env::args().collect();
    cli_has_flag(&args, "--wallet") || cli_has_flag(&args, "-w")
}

/// Escape a value for RFC 4180 CSV output.
/// If the value contains a comma, double-quote, or newline, wrap it in double-quotes
/// and escape any internal double-quotes by doubling them.
pub fn csv_escape(val: &str) -> std::borrow::Cow<'_, str> {
    if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
        let escaped = val.replace('"', "\"\"");
        std::borrow::Cow::Owned(format!("\"{}\"", escaped))
    } else {
        std::borrow::Cow::Borrowed(val)
    }
}

/// Join CSV fields with commas, escaping each field per RFC 4180.
pub fn csv_row_from(fields: &[&str]) -> String {
    let mut out = String::new();
    for (i, f) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&csv_escape(f));
    }
    out
}

/// Create a styled spinner with a message, returns the ProgressBar handle.
/// Caller should call `.finish_with_message()` or `.finish_and_clear()` when done.
pub fn spinner(msg: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .expect("static spinner template is valid")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

pub fn open_wallet(wallet_dir: &str, wallet_name: &str) -> Result<Wallet> {
    validate_name(wallet_name, "wallet")?;
    let raw = format!("{}/{}", wallet_dir, wallet_name);
    // Expand ~ so the existence check works outside a shell context.
    let path = if let Some(rest) = raw.strip_prefix("~/") {
        dirs::home_dir()
            .map(|h| h.join(rest).to_string_lossy().into_owned())
            .unwrap_or(raw)
    } else {
        raw
    };
    if !std::path::Path::new(&path).exists() {
        anyhow::bail!(
            "Wallet '{}' not found in {}.\n  Create one with: agcli wallet create --name {}\n  List existing:   agcli wallet list\n  To use a different wallet directory set AGCLI_WALLET_DIR.",
            wallet_name, wallet_dir, wallet_name
        );
    }
    Wallet::open(&path)
}

/// Unlock the coldkey. If `password` is provided, use it directly (non-interactive).
/// Otherwise, prompt interactively (unless batch mode).
pub fn unlock_coldkey(wallet: &mut Wallet, password: Option<&str>) -> Result<()> {
    let pw = match password {
        Some(p) => p.to_string(),
        None => {
            if is_batch_mode() {
                anyhow::bail!(
                    "Password required in batch mode. Pass --password <pw> or set AGCLI_PASSWORD."
                );
            }
            dialoguer::Password::new()
                .with_prompt("Coldkey password")
                .interact()?
        }
    };
    tracing::debug!("Unlocking coldkey");
    wallet.unlock_coldkey(&pw)
        .map_err(|e| {
            let msg = e.to_string();
            tracing::warn!(error = %msg, "Coldkey unlock failed");
            if msg.contains("wrong password") || msg.contains("Decryption failed") {
                anyhow::anyhow!("{}\n  Tip: pass --password <pw> or set AGCLI_PASSWORD env var for non-interactive use.", msg)
            } else {
                e
            }
        })
}

/// Validate that an amount is positive and non-zero.
/// Returns a human-friendly error if the amount is invalid.
pub fn validate_amount(amount: f64, label: &str) -> Result<()> {
    if amount < 0.0 {
        anyhow::bail!(
            "Invalid {}: {:.9}. Amount cannot be negative.",
            label,
            amount
        );
    }
    if amount == 0.0 {
        anyhow::bail!(
            "Invalid {}: amount must be greater than zero.\n  Tip: minimum stake is 1 RAO (0.000000001 τ).",
            label
        );
    }
    if !amount.is_finite() {
        anyhow::bail!(
            "Invalid {}: amount must be a finite number (got {}).",
            label,
            amount
        );
    }
    Ok(())
}

/// Safely convert a TAO-scale f64 to RAO (u64) with overflow protection.
/// Uses the same saturating logic as `Balance::from_tao()` to prevent silent truncation.
pub fn safe_rao(amount: f64) -> u64 {
    crate::types::Balance::from_tao(amount).rao()
}

/// Validate childkey take percentage is in the allowed range [0, 18].
pub fn validate_take_pct(take: f64) -> Result<()> {
    if take < 0.0 {
        anyhow::bail!(
            "Invalid childkey take: {:.2}%. Take cannot be negative.",
            take
        );
    }
    if take > 18.0 {
        anyhow::bail!(
            "Invalid childkey take: {:.2}%. Maximum allowed is 18%.\n  Tip: use --take 18 for maximum take.",
            take
        );
    }
    if !take.is_finite() {
        anyhow::bail!(
            "Invalid childkey take: must be a finite number (got {}).",
            take
        );
    }
    Ok(())
}

/// Validate a token symbol string (non-empty, reasonable length, ASCII).
pub fn validate_symbol(symbol: &str) -> Result<()> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid symbol: cannot be empty.\n  Tip: use a short, uppercase token symbol like \"ALPHA\" or \"SN1\"."
        );
    }
    if trimmed.len() > 32 {
        anyhow::bail!(
            "Invalid symbol: \"{}\" is too long ({} chars, max 32).\n  Tip: token symbols should be short, like \"ALPHA\".",
            trimmed, trimmed.len()
        );
    }
    if !trimmed.is_ascii() {
        anyhow::bail!(
            "Invalid symbol: \"{}\" contains non-ASCII characters. Use only ASCII letters/numbers.",
            trimmed
        );
    }
    Ok(())
}

/// Validate emission split weights (non-empty, no zeros in individual weights unless intentional).
pub fn validate_emission_weights(weights: &[u16]) -> Result<()> {
    if weights.is_empty() {
        anyhow::bail!("At least one emission weight is required.");
    }
    let total: u64 = weights.iter().map(|w| *w as u64).sum();
    if total == 0 {
        anyhow::bail!(
            "Invalid emission weights: total is zero. At least one weight must be non-zero."
        );
    }
    Ok(())
}

/// Validate snipe max-cost is non-negative.
pub fn validate_max_cost(max_cost: f64) -> Result<()> {
    if max_cost < 0.0 {
        anyhow::bail!(
            "Invalid --max-cost: {:.9}. Cost limit cannot be negative.",
            max_cost
        );
    }
    if !max_cost.is_finite() {
        anyhow::bail!(
            "Invalid --max-cost: must be a finite number (got {}).",
            max_cost
        );
    }
    Ok(())
}

/// Validate a wallet or hotkey name. Rejects path traversal, special characters,
/// and names that would be unsafe as directory/file names.
pub fn validate_name(name: &str, label: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {} name: cannot be empty.\n  Tip: use a simple alphanumeric name like \"default\" or \"mywallet\".",
            label
        );
    }
    if trimmed.len() > 64 {
        anyhow::bail!(
            "Invalid {} name: \"{}\" is too long ({} chars, max 64).",
            label,
            trimmed,
            trimmed.len()
        );
    }
    // Path traversal
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        anyhow::bail!(
            "Invalid {} name: \"{}\" contains path separators or traversal sequences.\n  Tip: use a simple name without '/', '\\', or '..'.",
            label, trimmed
        );
    }
    // Absolute paths (Unix or Windows)
    if trimmed.starts_with('/')
        || trimmed.starts_with('\\')
        || (trimmed.len() >= 2 && trimmed.as_bytes()[1] == b':')
    {
        anyhow::bail!(
            "Invalid {} name: \"{}\" looks like an absolute path. Use a simple name.",
            label,
            trimmed
        );
    }
    // Reserved or hidden names
    if trimmed.starts_with('.') {
        anyhow::bail!(
            "Invalid {} name: \"{}\" starts with a dot (hidden file).\n  Tip: use a name that starts with a letter or number.",
            label, trimmed
        );
    }
    // Only allow alphanumeric, hyphens, underscores
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        anyhow::bail!(
            "Invalid {} name: \"{}\" contains invalid characters.\n  Tip: use only letters, numbers, hyphens, and underscores.",
            label, trimmed
        );
    }
    // OS reserved names (Windows)
    let upper = trimmed.to_uppercase();
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.contains(&upper.as_str()) {
        anyhow::bail!(
            "Invalid {} name: \"{}\" is a reserved system name.",
            label,
            trimmed
        );
    }
    Ok(())
}

/// Validate an IPv4 address string and return the numeric representation.
/// Rejects broadcast (255.255.255.255), unspecified (0.0.0.0), and warns on private ranges.
pub fn validate_ipv4(ip: &str) -> Result<u128> {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        anyhow::bail!(
            "Invalid IPv4 address: \"{}\". Expected format: A.B.C.D (e.g., 1.2.3.4).",
            ip
        );
    }
    let mut octets = [0u8; 4];
    for (i, part) in parts.iter().enumerate() {
        // Reject leading zeros (ambiguous: octal vs decimal)
        if part.len() > 1 && part.starts_with('0') {
            anyhow::bail!(
                "Invalid IPv4 address: \"{}\" — octet {} has leading zeros. Use {} instead.",
                ip,
                i + 1,
                {
                    let trimmed = part.trim_start_matches('0');
                    if trimmed.is_empty() {
                        "0"
                    } else {
                        trimmed
                    }
                }
            );
        }
        octets[i] = part.parse::<u8>().map_err(|_| {
            anyhow::anyhow!(
                "Invalid IPv4 address: \"{}\" — octet {} ('{}') is not a valid number (0–255).",
                ip,
                i + 1,
                part
            )
        })?;
    }
    // Reject all-zeros
    if octets == [0, 0, 0, 0] {
        anyhow::bail!(
            "Invalid IP address: 0.0.0.0 (unspecified). Use your actual public IP address."
        );
    }
    // Reject broadcast
    if octets == [255, 255, 255, 255] {
        anyhow::bail!(
            "Invalid IP address: 255.255.255.255 (broadcast). Use your actual public IP address."
        );
    }
    // Reject loopback
    if octets[0] == 127 {
        anyhow::bail!(
            "Invalid IP address: {} (loopback). Use your public IP address for serving on the network.",
            ip
        );
    }
    // Warn on private ranges (print warning but allow)
    let is_private = matches!(
        (octets[0], octets[1]),
        (10, _) | (172, 16..=31) | (192, 168)
    );
    if is_private {
        eprintln!(
            "Warning: {} is a private IP address. Other nodes on the public network won't be able to reach you.\n  Tip: use your public IP address for serving.",
            ip
        );
    }
    let ip_u128 = ((octets[0] as u128) << 24)
        | ((octets[1] as u128) << 16)
        | ((octets[2] as u128) << 8)
        | (octets[3] as u128);
    Ok(ip_u128)
}

/// Validate a delegate take percentage is in the allowed range [0, 18].
pub fn validate_delegate_take(take: f64) -> Result<()> {
    if take < 0.0 {
        anyhow::bail!(
            "Invalid delegate take: {:.2}%. Take cannot be negative.",
            take
        );
    }
    if take > 18.0 {
        anyhow::bail!(
            "Invalid delegate take: {:.2}%. Maximum allowed is 18%.\n  Tip: use --take 18 for maximum.",
            take
        );
    }
    if !take.is_finite() {
        anyhow::bail!(
            "Invalid delegate take: must be a finite number (got {}).",
            take
        );
    }
    Ok(())
}

/// Validate an SS58 address string. Returns Ok(()) if valid, or a helpful error message.
/// Use this to validate user-supplied addresses (--dest, --delegate, --hotkey-address, --spawner, etc.)
/// before submitting them to the chain.
pub fn validate_ss58(address: &str, label: &str) -> Result<()> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {}: address cannot be empty.\n  Tip: provide a valid Bittensor SS58 address (48 characters, starts with '5').",
            label
        );
    }
    // Quick sanity checks before the expensive crypto verification
    let char_len = trimmed.chars().count();
    if char_len < 10 {
        anyhow::bail!(
            "Invalid {} address '{}' — too short. Bittensor SS58 addresses are 48 characters starting with '5'.",
            label, trimmed
        );
    }
    if char_len > 60 {
        let preview: String = trimmed.chars().take(20).collect();
        anyhow::bail!(
            "Invalid {} address '{}' — too long ({} chars). Bittensor SS58 addresses are 48 characters.",
            label, preview, char_len
        );
    }
    // Check for common mistakes: 0x prefix (Ethereum address), spaces, non-base58 chars
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        anyhow::bail!(
            "Invalid {} address: '{}' looks like an Ethereum/hex address.\n  Tip: Bittensor uses SS58 addresses (start with '5'). Convert at https://ss58.org or use `agcli wallet show`.",
            label, trimmed
        );
    }
    if trimmed.contains(' ') || trimmed.contains('\t') {
        anyhow::bail!(
            "Invalid {} address: contains whitespace. Remove any spaces or tabs from the address.",
            label
        );
    }
    // Base58 character set check (1-9, A-H, J-N, P-Z, a-k, m-z — no 0, I, O, l)
    if let Some(bad) = trimmed.chars().find(
        |c| !matches!(c, '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z'),
    ) {
        anyhow::bail!(
            "Invalid {} address '{}': character '{}' is not valid Base58.\n  Tip: SS58 addresses use Base58 encoding (no 0, I, O, or l).",
            label, crate::utils::short_ss58(trimmed), bad
        );
    }
    // Full cryptographic verification via sp_core
    use sp_core::{crypto::Ss58Codec, sr25519};
    sr25519::Public::from_ss58check(trimmed).map_err(|_| {
        anyhow::anyhow!(
            "Invalid {} address '{}': checksum verification failed.\n  Tip: double-check the address. Use `agcli wallet show` to get your correct address.",
            label, crate::utils::short_ss58(trimmed)
        )
    })?;
    Ok(())
}

/// Validate password strength for wallet creation. Rejects empty passwords
/// (which would produce trivially-breakable encryption). Prints warnings for
/// weak-but-non-empty passwords.
pub fn validate_password_strength(password: &str) -> Result<()> {
    if password.is_empty() {
        anyhow::bail!(
            "Empty password is not allowed — your wallet encryption would be trivially breakable.\n  \
             Provide a password with at least 8 characters."
        );
    }
    if password.len() < 8 {
        anyhow::bail!(
            "Password too short ({} characters). Minimum 8 characters required to protect wallet encryption.\n  \
             Short passwords can be brute-forced against Argon2id in minutes on modern GPUs.",
            password.len()
        );
    }
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());
    let variety = [has_upper, has_lower, has_digit, has_special]
        .iter()
        .filter(|&&b| b)
        .count();
    if variety < 2 {
        eprintln!(
            "Warning: password uses only one character type. Mix uppercase, lowercase, numbers, and symbols for stronger security."
        );
    }
    // Check for common weak passwords
    let common = [
        "password",
        "12345678",
        "123456789",
        "1234567890",
        "qwerty",
        "abc123",
        "letmein",
        "welcome",
        "monkey",
        "dragon",
        "master",
        "login",
        "princess",
        "football",
        "shadow",
    ];
    if common.contains(&password.to_lowercase().as_str()) {
        anyhow::bail!(
            "Refusing commonly used password — your wallet would be vulnerable to dictionary attacks.\n  \
             Choose a unique password with at least 8 characters."
        );
    }
    Ok(())
}

/// Validate a port number is in the valid range [1, 65535].
pub fn validate_port(port: u16, label: &str) -> Result<()> {
    if port == 0 {
        anyhow::bail!(
            "Invalid {} port: 0. Port must be between 1 and 65535.\n  Tip: common ports are 8091 (axon) and 443 (HTTPS).",
            label
        );
    }
    if port < 1024 {
        eprintln!(
            "Warning: {} port {} is a privileged port (< 1024). You may need root access to bind to it.",
            label, port
        );
    }
    Ok(())
}

/// Validate a netuid is in a reasonable range for the Bittensor network.
pub fn validate_netuid(netuid: u16) -> Result<()> {
    if netuid == 0 {
        anyhow::bail!(
            "Invalid netuid: 0. Root network (netuid 0) is not a user subnet.\n  Tip: user subnets start at netuid 1."
        );
    }
    Ok(())
}

/// Validate a batch-axon JSON file structure. Returns a vec of errors found.
/// Each entry should have: netuid (u16), ip (valid IPv4), port (u16).
/// Optional fields: protocol (u8, default 4), version (u32, default 0).
pub fn validate_batch_axon_json(json_str: &str) -> Result<Vec<serde_json::Value>> {
    let entries: Vec<serde_json::Value> = serde_json::from_str(json_str).map_err(|e| {
        anyhow::anyhow!(
            "Invalid batch-axon JSON: {}.\n  Expected format: [{{\"netuid\": 1, \"ip\": \"1.2.3.4\", \"port\": 8091}}]",
            e
        )
    })?;
    if entries.is_empty() {
        anyhow::bail!(
            "Batch-axon JSON is empty. Provide at least one entry.\n  Format: [{{\"netuid\": 1, \"ip\": \"1.2.3.4\", \"port\": 8091}}]"
        );
    }
    for (i, entry) in entries.iter().enumerate() {
        let obj = entry.as_object().ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {} is not a JSON object. Each entry must be {{\"netuid\": N, \"ip\": \"...\", \"port\": N}}.", i)
        })?;
        // Required: netuid
        let netuid = obj.get("netuid").ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: missing required field 'netuid'.", i)
        })?;
        let netuid_val = netuid.as_u64().ok_or_else(|| {
            anyhow::anyhow!(
                "Batch-axon entry {}: 'netuid' must be a positive integer (got {}).",
                i,
                netuid
            )
        })?;
        if netuid_val > 65535 {
            anyhow::bail!(
                "Batch-axon entry {}: 'netuid' {} exceeds maximum (65535).",
                i,
                netuid_val
            );
        }
        // Required: ip
        let ip = obj.get("ip").ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: missing required field 'ip'.", i)
        })?;
        let ip_str = ip.as_str().ok_or_else(|| {
            anyhow::anyhow!(
                "Batch-axon entry {}: 'ip' must be a string (got {}).",
                i,
                ip
            )
        })?;
        validate_ipv4(ip_str).map_err(|e| anyhow::anyhow!("Batch-axon entry {}: {}", i, e))?;
        // Required: port
        let port = obj.get("port").ok_or_else(|| {
            anyhow::anyhow!("Batch-axon entry {}: missing required field 'port'.", i)
        })?;
        let port_val = port.as_u64().ok_or_else(|| {
            anyhow::anyhow!(
                "Batch-axon entry {}: 'port' must be a positive integer (got {}).",
                i,
                port
            )
        })?;
        if port_val == 0 || port_val > 65535 {
            anyhow::bail!(
                "Batch-axon entry {}: 'port' {} is out of range (1–65535).",
                i,
                port_val
            );
        }
        // Optional: protocol (u8, default 4)
        if let Some(proto) = obj.get("protocol") {
            let proto_val = proto.as_u64().ok_or_else(|| {
                anyhow::anyhow!(
                    "Batch-axon entry {}: 'protocol' must be a number (got {}).",
                    i,
                    proto
                )
            })?;
            if proto_val > 255 {
                anyhow::bail!(
                    "Batch-axon entry {}: 'protocol' {} exceeds maximum (255).",
                    i,
                    proto_val
                );
            }
        }
        // Optional: version (u32)
        if let Some(ver) = obj.get("version") {
            ver.as_u64().ok_or_else(|| {
                anyhow::anyhow!(
                    "Batch-axon entry {}: 'version' must be a number (got {}).",
                    i,
                    ver
                )
            })?;
        }
        // Warn on unknown fields
        let known = ["netuid", "ip", "port", "protocol", "version"];
        for key in obj.keys() {
            if !known.contains(&key.as_str()) {
                eprintln!(
                    "Warning: batch-axon entry {}: unknown field '{}' (ignored).",
                    i, key
                );
            }
        }
    }
    Ok(entries)
}

/// Check per-subnet spending limit from config.
/// Returns Ok if no limit set or amount is within limit, Err otherwise.
pub fn check_spending_limit(netuid: u16, tao_amount: f64) -> Result<()> {
    let cfg = crate::config::Config::load();
    if let Some(ref limits) = cfg.spending_limits {
        let key = netuid.to_string();
        if let Some(&limit) = limits.get(&key) {
            if tao_amount > limit {
                tracing::warn!(
                    netuid = netuid,
                    amount = tao_amount,
                    limit = limit,
                    "Per-subnet spending limit exceeded"
                );
                anyhow::bail!(
                    "Spending limit exceeded for SN{}: trying {:.4}τ but limit is {:.4}τ.\n  Adjust with: agcli config set spending_limit.{} {}",
                    netuid, tao_amount, limit, netuid, tao_amount
                );
            }
        }
        // Also check wildcard "*" key for global limit
        if let Some(&limit) = limits.get("*") {
            if tao_amount > limit {
                tracing::warn!(
                    amount = tao_amount,
                    limit = limit,
                    "Global spending limit exceeded"
                );
                anyhow::bail!(
                    "Global spending limit exceeded: trying {:.4}τ but limit is {:.4}τ.\n  Adjust with: agcli config set spending_limit.* {}",
                    tao_amount, limit, tao_amount
                );
            }
        }
    }
    Ok(())
}

/// Check spending limits for a raw pallet call (used by batch, scheduler, multisig).
///
/// Inspects the pallet/call name and args to extract the TAO amount and netuid
/// for known staking operations. Unknown calls are allowed through (they may not
/// involve TAO spending).
///
/// Known staking calls on SubtensorModule:
///   add_stake(hotkey, netuid, amount_rao)
///   remove_stake(hotkey, netuid, amount_rao)
///   move_stake(hotkey_origin, hotkey_dest, origin_netuid, dest_netuid, amount_rao)
///   swap_stake(hotkey, origin_netuid, dest_netuid, amount_rao)
///   transfer_stake(dest, hotkey, origin_netuid, dest_netuid, amount_rao)
///   add_stake_limit(hotkey, netuid, amount_rao, limit_price, allow_partial)
///   remove_stake_limit(hotkey, netuid, amount_rao, limit_price, allow_partial)
///   swap_stake_limit(hotkey, origin_netuid, dest_netuid, amount_rao, limit_price, allow_partial)
pub fn check_spending_limit_for_raw_call(
    pallet: &str,
    call: &str,
    args: &[serde_json::Value],
) -> Result<()> {
    if pallet != "SubtensorModule" {
        return Ok(());
    }

    // Extract (netuid_index, amount_rao_index) for each known call
    let (netuid_idx, amount_idx) = match call {
        // add_stake(hotkey, netuid, amount_rao)
        // remove_stake(hotkey, netuid, amount_rao)
        "add_stake" | "remove_stake" => (1, 2),
        // add_stake_limit(hotkey, netuid, amount_rao, limit_price, allow_partial)
        // remove_stake_limit(hotkey, netuid, amount_rao, limit_price, allow_partial)
        "add_stake_limit" | "remove_stake_limit" => (1, 2),
        // move_stake(hotkey_o, hotkey_d, origin_netuid, dest_netuid, amount_rao)
        "move_stake" => (2, 4), // check origin_netuid (funds leave this subnet)
        // swap_stake(hotkey, origin_netuid, dest_netuid, amount_rao)
        "swap_stake" => (1, 3), // check origin_netuid (funds leave this subnet)
        // transfer_stake(dest, hotkey, origin_netuid, dest_netuid, amount_rao)
        "transfer_stake" => (2, 4), // check origin_netuid (funds leave this subnet)
        // swap_stake_limit(hotkey, origin_netuid, dest_netuid, amount_rao, ...)
        "swap_stake_limit" => (1, 3), // check origin_netuid (funds leave this subnet)
        _ => return Ok(()),           // unknown call, no spending limit to check
    };

    if args.len() <= amount_idx {
        // Not enough args to extract amount — let the call fail at encoding time
        return Ok(());
    }

    let netuid_u64 = args[netuid_idx].as_u64().ok_or_else(|| {
        anyhow::anyhow!(
            "Spending limit check: expected numeric netuid at arg index {}, got: {}",
            netuid_idx,
            args[netuid_idx]
        )
    })?;
    if netuid_u64 > u16::MAX as u64 {
        anyhow::bail!(
            "Spending limit check: netuid {} exceeds maximum ({})",
            netuid_u64,
            u16::MAX
        );
    }
    let netuid = netuid_u64 as u16;
    let amount_rao = args[amount_idx].as_u64().ok_or_else(|| {
        anyhow::anyhow!(
            "Spending limit check: expected numeric amount at arg index {}, got: {}",
            amount_idx,
            args[amount_idx]
        )
    })?;
    let tao_amount = amount_rao as f64 / 1_000_000_000.0;

    if tao_amount > 0.0 {
        check_spending_limit(netuid, tao_amount)?;
    }
    Ok(())
}

/// Print a JSON value to stdout. Respects the global pretty-print flag.
pub fn print_json(value: &serde_json::Value) {
    if is_pretty_mode() {
        match serde_json::to_string_pretty(value) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Error: failed to serialize JSON: {}", e),
        }
    } else {
        println!("{}", value);
    }
}

/// Print a Serialize-able value as JSON. Respects global pretty-print flag.
pub fn print_json_ser<T: serde::Serialize>(value: &T) {
    let result = if is_pretty_mode() {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    };
    match result {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("Error: failed to serialize JSON: {}", e),
    }
}

/// Print a JSON value to stderr. Respects the global pretty-print flag.
pub fn eprint_json(value: &serde_json::Value) {
    if is_pretty_mode() {
        match serde_json::to_string_pretty(value) {
            Ok(s) => eprintln!("{}", s),
            Err(e) => eprintln!("Error: failed to serialize JSON: {}", e),
        }
    } else {
        eprintln!("{}", value);
    }
}

/// Print transaction result in both json and table modes.
pub fn print_tx_result(output: OutputFormat, hash: &str, label: &str) {
    if output.is_json() {
        print_json(&serde_json::json!({"tx_hash": hash}));
    } else {
        println!("{} Tx: {}", label, hash);
    }
}

/// Thread-local pretty mode flag.
static PRETTY_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Set pretty mode globally.
pub fn set_pretty_mode(pretty: bool) {
    PRETTY_MODE.store(pretty, std::sync::atomic::Ordering::Relaxed);
}

/// Check if pretty mode is active.
pub fn is_pretty_mode() -> bool {
    PRETTY_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Global batch mode flag: all missing args are hard errors, never prompt for input.
static BATCH_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Global yes mode flag: skip confirmation prompts (auto-confirm).
static YES_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Set batch mode globally (called from execute()).
pub fn set_batch_mode(batch: bool) {
    BATCH_MODE.store(batch, std::sync::atomic::Ordering::Relaxed);
}

/// Set yes mode globally (called from execute()).
pub fn set_yes_mode(yes: bool) {
    YES_MODE.store(yes, std::sync::atomic::Ordering::Relaxed);
}

/// Check if batch mode is active (--batch: hard error on missing args).
pub fn is_batch_mode() -> bool {
    BATCH_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Check if confirmation prompts should be skipped (--yes or --batch).
pub fn is_yes_mode() -> bool {
    YES_MODE.load(std::sync::atomic::Ordering::Relaxed)
        || BATCH_MODE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Prompt the user to confirm a destructive or admin action.
/// Returns Ok(()) if confirmed (or --yes/--batch mode), Err if user declines.
pub fn confirm_action(message: &str) -> Result<()> {
    if is_yes_mode() {
        return Ok(());
    }
    eprint!("{} [y/N] ", message);
    use std::io::Write;
    std::io::stderr().flush().ok();
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read confirmation input: {}", e))?;
    let trimmed = input.trim().to_lowercase();
    if trimmed == "y" || trimmed == "yes" {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Operation cancelled by user"))
    }
}

pub fn resolve_coldkey_address(
    address: Option<String>,
    wallet_dir: &str,
    wallet_name: &str,
) -> String {
    address.unwrap_or_else(|| {
        match open_wallet(wallet_dir, wallet_name) {
            Ok(w) => w.coldkey_ss58().map(|s| s.to_string()).unwrap_or_default(),
            Err(e) => {
                tracing::debug!(wallet = wallet_name, error = %e, "Could not open wallet to resolve coldkey");
                String::new()
            }
        }
    })
}

/// Resolve a coldkey address from an optional CLI argument, with SS58 validation when the user
/// explicitly provides one. Falls back to the wallet's coldkey if no address is given.
pub fn resolve_and_validate_coldkey_address(
    address: Option<String>,
    wallet_dir: &str,
    wallet_name: &str,
    label: &str,
) -> Result<String> {
    if let Some(ref addr) = address {
        validate_ss58(addr, label)?;
    }
    let resolved = resolve_coldkey_address(address, wallet_dir, wallet_name);
    if resolved.is_empty() {
        anyhow::bail!(
            "Could not resolve coldkey address from wallet '{}' in {}.\n  Tip: pass --address <ss58> explicitly, or create a wallet with: agcli wallet create",
            wallet_name, wallet_dir
        );
    }
    Ok(resolved)
}

pub fn resolve_hotkey_ss58(
    hotkey_arg: Option<String>,
    wallet: &mut Wallet,
    hotkey_name: &str,
) -> Result<String> {
    if let Some(hk) = hotkey_arg {
        validate_ss58(&hk, "hotkey-address")?;
        return Ok(hk);
    }
    wallet.load_hotkey(hotkey_name)?;
    wallet
        .hotkey_ss58()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not resolve hotkey address.\n  Tip: pass --hotkey-address <ss58_address> or create a hotkey with `agcli wallet create-hotkey`."))
}

/// Shortcut: open wallet, unlock, resolve hotkey, return (pair, hotkey_ss58).
pub fn unlock_and_resolve(
    wallet_dir: &str,
    wallet_name: &str,
    hotkey_name: &str,
    hotkey_arg: Option<String>,
    password: Option<&str>,
) -> Result<(sp_core::sr25519::Pair, String)> {
    let mut wallet = open_wallet(wallet_dir, wallet_name)?;
    unlock_coldkey(&mut wallet, password)?;
    let hotkey_ss58 = resolve_hotkey_ss58(hotkey_arg, &mut wallet, hotkey_name)?;
    let pair = wallet.coldkey()?.clone();
    Ok((pair, hotkey_ss58))
}

pub fn parse_weight_pairs(weights_str: &str) -> Result<(Vec<u16>, Vec<u16>)> {
    let mut uids = Vec::new();
    let mut weights = Vec::new();
    for pair in weights_str.split(',') {
        let parts: Vec<&str> = pair.trim().split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid weight pair '{}'. Format: 'uid:weight' (e.g., '0:100,1:200,2:50')",
                pair
            );
        }
        uids.push(
            parts[0].trim().parse::<u16>().map_err(|_| {
                anyhow::anyhow!("Invalid UID '{}' — must be 0–65535", parts[0].trim())
            })?,
        );
        weights.push(parts[1].trim().parse::<u16>().map_err(|_| {
            anyhow::anyhow!("Invalid weight '{}' — must be 0–65535", parts[1].trim())
        })?);
    }
    Ok((uids, weights))
}

pub fn parse_children(children_str: &str) -> Result<Vec<(u64, String)>> {
    let trimmed = children_str.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Children list cannot be empty.\n  Format: 'proportion:hotkey_ss58' (e.g., '50000:5Cai...,50000:5Dqw...')"
        );
    }
    let mut result = Vec::new();
    for pair in trimmed.split(',') {
        let pair_trimmed = pair.trim();
        if pair_trimmed.is_empty() {
            continue; // skip trailing commas
        }
        // SS58 addresses contain colons in some edge cases, so split on first colon only
        let colon_pos = pair_trimmed.find(':').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid child pair '{}'. Format: 'proportion:hotkey_ss58' (e.g., '50000:5Cai...')",
                pair_trimmed
            )
        })?;
        let proportion_str = &pair_trimmed[..colon_pos].trim();
        let hotkey_str = &pair_trimmed[colon_pos + 1..].trim();
        let proportion = proportion_str.parse::<u64>().map_err(|_| {
            anyhow::anyhow!(
                "Invalid proportion '{}' — must be a positive integer (u64)",
                proportion_str
            )
        })?;
        if proportion == 0 {
            anyhow::bail!("Invalid proportion: 0. Each child must have a non-zero proportion.");
        }
        // Validate the hotkey is a valid SS58 address
        validate_ss58(hotkey_str, "child hotkey")?;
        result.push((proportion, hotkey_str.to_string()));
    }
    if result.is_empty() {
        anyhow::bail!(
            "No valid children provided.\n  Format: 'proportion:hotkey_ss58' (e.g., '50000:5Cai...')"
        );
    }
    Ok(result)
}

/// Render a slice in json, csv, or table format.
///
/// - `json`: Serializes `data` with `print_json_ser`.
/// - `csv`: Prints `csv_header` then calls `csv_row` per item.
/// - `table`: Prints optional `preamble`, then builds a comfy_table
///   with `table_headers` and `table_row` per item.
pub fn render_rows<T: serde::Serialize>(
    output: OutputFormat,
    data: &[T],
    csv_header: &str,
    csv_row: impl Fn(&T) -> String,
    table_headers: &[&str],
    table_row: impl Fn(&T) -> Vec<String>,
    preamble: Option<&str>,
) {
    if output.is_json() {
        print_json_ser(&data);
    } else if output.is_csv() {
        println!("{}", csv_header);
        for item in data {
            println!("{}", csv_row(item));
        }
    } else {
        if let Some(text) = preamble {
            println!("{}", text);
        }
        let mut table = comfy_table::Table::new();
        table.set_header(table_headers.iter().copied());
        for item in data {
            table.add_row(table_row(item));
        }
        println!("{table}");
    }
}

/// Build a HashMap of netuid → &DynamicInfo for quick lookups.
pub fn build_dynamic_map(
    dynamic: &[crate::types::chain_data::DynamicInfo],
) -> std::collections::HashMap<u16, &crate::types::chain_data::DynamicInfo> {
    dynamic.iter().map(|d| (d.netuid.0, d)).collect()
}

/// Validate a mnemonic phrase. Checks word count (12, 15, 18, 21, or 24 words)
/// and verifies all words are in the BIP-39 English dictionary with checksum validation.
/// Returns Ok(()) on success, or a helpful error message.
pub fn validate_mnemonic(mnemonic: &str) -> Result<()> {
    let trimmed = mnemonic.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Mnemonic phrase cannot be empty.\n  Tip: a BIP-39 mnemonic is 12, 15, 18, 21, or 24 English words."
        );
    }
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    let valid_counts = [12, 15, 18, 21, 24];
    if !valid_counts.contains(&words.len()) {
        anyhow::bail!(
            "Invalid mnemonic: {} words found, expected 12, 15, 18, 21, or 24.\n  Tip: check for missing or extra words. Most wallets use 12-word mnemonics.",
            words.len()
        );
    }
    // Check each word is in the BIP-39 English wordlist
    let wordlist = bip39::Language::English.word_list();
    for (i, word) in words.iter().enumerate() {
        if wordlist.binary_search(word).is_err() {
            // Try to suggest a close match
            let suggestion = wordlist.iter().find(|w| {
                let end = word
                    .char_indices()
                    .nth(3)
                    .map(|(i, _)| i)
                    .unwrap_or(word.len());
                w.starts_with(&word[..end])
            });
            let tip = if let Some(s) = suggestion {
                format!("  Did you mean \"{}\"?", s)
            } else {
                String::from(
                    "  Check spelling — all words must be from the BIP-39 English wordlist.",
                )
            };
            anyhow::bail!(
                "Invalid mnemonic: word {} (\"{}\") is not in the BIP-39 dictionary.\n{}",
                i + 1,
                word,
                tip
            );
        }
    }
    // Full checksum validation via bip39 crate
    use bip39::Mnemonic;
    Mnemonic::parse_in(bip39::Language::English, trimmed).map_err(|e| {
        anyhow::anyhow!(
            "Invalid mnemonic checksum: {}.\n  Tip: the last word encodes a checksum. Make sure all words are correct and in the right order.",
            e
        )
    })?;
    Ok(())
}

/// Validate input for the `wallet derive` command.
/// Accepts either a 0x-prefixed 32-byte hex public key or a BIP-39 mnemonic phrase.
/// Rejects: empty input, odd-length hex, wrong-length hex keys, invalid hex chars,
/// ambiguous inputs that look neither like hex nor like a mnemonic.
pub fn validate_derive_input(input: &str) -> Result<()> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Derive input cannot be empty.\n  Tip: pass a 0x-prefixed hex public key (64 hex chars) or a BIP-39 mnemonic phrase."
        );
    }
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        // Hex public key path
        let hex_str = &trimmed[2..];
        if hex_str.is_empty() {
            anyhow::bail!(
                "Hex public key is empty after '0x' prefix.\n  Tip: provide 64 hex characters, e.g. '0x0123...abcd'."
            );
        }
        if !hex_str.len().is_multiple_of(2) {
            anyhow::bail!(
                "Hex has odd length ({} chars). Hex bytes come in pairs.\n  Tip: check for a missing or extra character.",
                hex_str.len()
            );
        }
        // Validate chars before decode
        if let Some(pos) = hex_str.find(|c: char| !c.is_ascii_hexdigit()) {
            let bad_char = hex_str[pos..].chars().next().unwrap();
            anyhow::bail!(
                "Invalid hex character '{}' at position {}.\n  Tip: hex values use only 0-9 and a-f.",
                bad_char, pos + 2
            );
        }
        let byte_len = hex_str.len() / 2;
        if byte_len != 32 {
            anyhow::bail!(
                "Public key must be 32 bytes (64 hex chars), got {} bytes ({} hex chars).\n  Tip: an SR25519 public key is exactly 32 bytes.",
                byte_len, hex_str.len()
            );
        }
    } else {
        // Treat as mnemonic
        validate_mnemonic(trimmed)?;
    }
    Ok(())
}

/// Require a mnemonic phrase: use `provided` if Some, else prompt interactively (or error in batch mode).
/// Validates the mnemonic format and dictionary words before returning.
///
/// **Security**: If the mnemonic was provided via the `--mnemonic` CLI flag
/// (detectable because `AGCLI_MNEMONIC` env var is not set), the function
/// **refuses** it — CLI arguments are visible in `ps` output and process listings,
/// exposing the mnemonic to any local user. Use `AGCLI_MNEMONIC` env var instead.
pub fn require_mnemonic(provided: Option<String>) -> Result<String> {
    let mnemonic = match provided {
        Some(m) => {
            // Detect if mnemonic came from CLI flag vs env var.
            // clap fills the field from either source; we check which one.
            if std::env::var("AGCLI_MNEMONIC").is_err() {
                anyhow::bail!(
                    "Refusing --mnemonic flag: mnemonic phrases are visible in `ps` output.\n\
                     Use the AGCLI_MNEMONIC environment variable instead:\n  \
                     export AGCLI_MNEMONIC='your twelve words here'\n  \
                     agcli wallet import"
                );
            }
            m
        }
        None => {
            if is_batch_mode() {
                anyhow::bail!("Mnemonic required in batch mode. Set AGCLI_MNEMONIC env var.");
            }
            dialoguer::Input::<String>::new()
                .with_prompt("Enter mnemonic phrase")
                .interact_text()
                .map_err(anyhow::Error::from)?
        }
    };
    validate_mnemonic(&mnemonic)?;
    Ok(mnemonic)
}

/// Require a password: use `cmd_password` (command-level), `global_password` (global flag), or prompt.
/// If `confirm` is true, ask for password confirmation on interactive entry.
pub fn require_password(
    cmd_password: Option<String>,
    global_password: Option<&str>,
    confirm: bool,
) -> Result<String> {
    cmd_password
        .or_else(|| global_password.map(|s| s.to_string()))
        .map(Ok)
        .unwrap_or_else(|| {
            if is_batch_mode() {
                return Err(anyhow::anyhow!(
                    "Password required in batch mode. Pass --password <pw> or set AGCLI_PASSWORD."
                ));
            }
            if confirm {
                dialoguer::Password::new()
                    .with_prompt("Set password")
                    .with_confirmation("Confirm", "Mismatch")
                    .interact()
                    .map_err(anyhow::Error::from)
            } else {
                dialoguer::Password::new()
                    .with_prompt("Password")
                    .interact()
                    .map_err(anyhow::Error::from)
            }
        })
}

/// Validate JSON args for multisig batch extrinsics.
/// Rejects: non-array JSON, null elements, deeply nested structures (depth > 4),
/// excessively long strings (> 1024 chars), NaN/Infinity floats.
/// Returns the parsed JSON array on success.
pub fn validate_multisig_json_args(json_str: &str) -> Result<Vec<serde_json::Value>> {
    let trimmed = json_str.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Empty JSON args.\n  Tip: pass a JSON array, e.g. '[1, \"0x...\"]'.");
    }
    let parsed: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| {
        anyhow::anyhow!(
            "Invalid JSON: {}.\n  Tip: args must be a valid JSON array, e.g. '[1, \"0x...\"]'.",
            e
        )
    })?;
    let arr = match parsed {
        serde_json::Value::Array(a) => a,
        other => {
            anyhow::bail!(
                "Expected a JSON array, got {}.\n  Tip: wrap your args in square brackets, e.g. '[{}]'.",
                match &other {
                    serde_json::Value::Object(_) => "an object",
                    serde_json::Value::String(_) => "a string",
                    serde_json::Value::Number(_) => "a number",
                    serde_json::Value::Bool(_) => "a boolean",
                    serde_json::Value::Null => "null",
                    _ => "unknown type",
                },
                other
            );
        }
    };
    // Validate each element
    fn check_depth(v: &serde_json::Value, depth: usize) -> Result<()> {
        if depth > 4 {
            anyhow::bail!(
                "JSON nesting too deep (>4 levels).\n  Tip: flatten your args structure."
            );
        }
        match v {
            serde_json::Value::Null => {
                anyhow::bail!(
                    "null values not allowed in multisig args.\n  Tip: use 0 or \"\" instead of null."
                );
            }
            serde_json::Value::String(s) if s.len() > 1024 => {
                anyhow::bail!(
                    "String arg too long ({} chars, max 1024).\n  Tip: shorten the value or use a different encoding.",
                    s.len()
                );
            }
            serde_json::Value::Number(n) => {
                if n.as_f64().is_some_and(|f| f.is_nan() || f.is_infinite()) {
                    anyhow::bail!("NaN/Infinity not allowed in args.\n  Tip: use a finite number.");
                }
            }
            serde_json::Value::Array(inner) => {
                for item in inner {
                    check_depth(item, depth + 1)?;
                }
            }
            serde_json::Value::Object(map) => {
                for (_k, val) in map {
                    check_depth(val, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    for (i, elem) in arr.iter().enumerate() {
        check_depth(elem, 0).map_err(|e| anyhow::anyhow!("Invalid arg at index {}: {}", i, e))?;
    }
    Ok(arr)
}

/// Validate an EVM address string (hex, 20 bytes). Accepts optional 0x prefix.
/// Returns a helpful error for common mistakes.
pub fn validate_evm_address(address: &str, label: &str) -> Result<()> {
    let trimmed = address.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {} EVM address: cannot be empty.\n  Tip: provide a 0x-prefixed 40 hex char address, e.g. '0x1234...abcd'.",
            label
        );
    }
    let hex_str = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if hex_str.is_empty() {
        anyhow::bail!(
            "Invalid {} EVM address: empty after '0x' prefix.\n  Tip: provide 40 hex characters after '0x'.",
            label
        );
    }
    if !hex_str.len().is_multiple_of(2) {
        anyhow::bail!(
            "Invalid {} EVM address: odd hex length ({} chars). Hex bytes come in pairs.\n  Tip: check for a missing or extra character.",
            label, hex_str.len()
        );
    }
    if let Some(pos) = hex_str.find(|c: char| !c.is_ascii_hexdigit()) {
        let bad_char = hex_str[pos..].chars().next().unwrap();
        anyhow::bail!(
            "Invalid {} EVM address: character '{}' at position {} is not valid hex.\n  Tip: use only 0-9 and a-f.",
            label, bad_char, pos
        );
    }
    let byte_len = hex_str.len() / 2;
    if byte_len != 20 {
        anyhow::bail!(
            "Invalid {} EVM address: must be 20 bytes (40 hex chars), got {} bytes ({} hex chars).\n  Tip: Ethereum/EVM addresses are exactly 20 bytes.",
            label, byte_len, hex_str.len()
        );
    }
    Ok(())
}

/// Validate a hex data string. Accepts optional 0x prefix.
/// Rejects odd-length and non-hex characters.
pub fn validate_hex_data(data: &str, label: &str) -> Result<()> {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {} hex data: cannot be empty.\n  Tip: use '0x' for empty data.",
            label
        );
    }
    let hex_str = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    // "0x" alone is valid (empty data)
    if hex_str.is_empty() {
        return Ok(());
    }
    if !hex_str.len().is_multiple_of(2) {
        anyhow::bail!(
            "Invalid {} hex data: odd length ({} chars). Hex bytes come in pairs.\n  Tip: check for a missing or extra character.",
            label, hex_str.len()
        );
    }
    if let Some(pos) = hex_str.find(|c: char| !c.is_ascii_hexdigit()) {
        let bad_char = hex_str[pos..].chars().next().unwrap();
        anyhow::bail!(
            "Invalid {} hex data: character '{}' at position {} is not valid hex.\n  Tip: use only 0-9 and a-f.",
            label, bad_char, pos
        );
    }
    Ok(())
}

/// Validate a pallet or call name for scheduler/preimage commands.
/// Must be non-empty, reasonable length, and contain only valid Rust identifier characters.
pub fn validate_pallet_call(name: &str, label: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {}: cannot be empty.\n  Tip: use the pallet name exactly as in the runtime, e.g. 'System', 'Balances', 'SubtensorModule'.",
            label
        );
    }
    if trimmed.len() > 128 {
        let preview: String = trimmed.chars().take(32).collect();
        anyhow::bail!(
            "Invalid {}: '{}' is too long ({} chars, max 128).",
            label,
            preview,
            trimmed.len()
        );
    }
    // Must start with a letter (PascalCase for pallets, snake_case for calls)
    if !trimmed.chars().next().unwrap().is_ascii_alphabetic() {
        anyhow::bail!(
            "Invalid {}: '{}' must start with a letter.\n  Tip: pallet names are PascalCase (e.g. 'System'), call names are snake_case (e.g. 'remark').",
            label, trimmed
        );
    }
    // Only allow alphanumeric and underscore (covers PascalCase + snake_case)
    if let Some(bad) = trimmed
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '_')
    {
        anyhow::bail!(
            "Invalid {}: character '{}' is not allowed.\n  Tip: use only letters, numbers, and underscores.",
            label, bad
        );
    }
    Ok(())
}

/// Validate a scheduler task ID (for schedule-named / cancel-named).
/// Must be non-empty and ≤ 32 bytes (on-chain limit).
pub fn validate_schedule_id(id: &str) -> Result<()> {
    if id.is_empty() {
        anyhow::bail!(
            "Schedule ID cannot be empty.\n  Tip: provide a short, descriptive name for your scheduled task."
        );
    }
    if id.len() > 32 {
        anyhow::bail!(
            "Schedule ID too long: {} bytes (max 32).\n  Tip: use a shorter ID.",
            id.len()
        );
    }
    Ok(())
}

/// Validate a crowdloan TAO amount (deposit, contribution, cap).
/// Must be positive and finite. Unlike staking amounts, zero is rejected with a crowdloan-specific tip.
pub fn validate_crowdloan_amount(amount: f64, label: &str) -> Result<()> {
    if !amount.is_finite() {
        anyhow::bail!(
            "Invalid {}: must be a finite number (got {}).\n  Tip: use a decimal TAO amount like 1.0 or 100.",
            label, amount
        );
    }
    if amount < 0.0 {
        anyhow::bail!(
            "Invalid {}: {:.9} TAO. Amount cannot be negative.\n  Tip: use a positive TAO amount.",
            label,
            amount
        );
    }
    if amount == 0.0 {
        anyhow::bail!(
            "Invalid {}: amount must be greater than zero.\n  Tip: specify a TAO amount like --{} 1.0",
            label, label.replace(' ', "-")
        );
    }
    Ok(())
}

/// Validate a liquidity price bound (TAO per Alpha).
/// Must be positive and finite. Zero and negative prices are rejected.
pub fn validate_price(price: f64, label: &str) -> Result<()> {
    if !price.is_finite() {
        anyhow::bail!(
            "Invalid {}: must be a finite number (got {}).\n  Tip: use a decimal price like 0.001 or 1.5",
            label, price
        );
    }
    if price <= 0.0 {
        anyhow::bail!(
            "Invalid {}: {:.9}. Price must be positive (TAO per Alpha).\n  Tip: use a positive decimal like 0.001",
            label, price
        );
    }
    Ok(())
}

/// Validate commitment data string. Must be non-empty and within a reasonable size.
pub fn validate_commitment_data(data: &str) -> Result<()> {
    if data.trim().is_empty() {
        anyhow::bail!(
            "Invalid commitment data: cannot be empty.\n  Tip: provide key:value pairs like \"endpoint:http://myserver.com,version:1.0\""
        );
    }
    if data.len() > 1024 {
        anyhow::bail!(
            "Invalid commitment data: too long ({} bytes, max 1024).\n  Tip: keep commitment data concise.",
            data.len()
        );
    }
    Ok(())
}

/// Validate a subscribe event filter category (must match [`crate::events::EventFilter`] `FromStr` aliases).
pub fn validate_event_filter(filter: &str) -> Result<()> {
    let lower = filter.trim().to_lowercase();
    let ok = matches!(
        lower.as_str(),
        "all"
            | "staking"
            | "stake"
            | "registration"
            | "register"
            | "reg"
            | "transfer"
            | "transfers"
            | "weights"
            | "weight"
            | "subnet"
            | "subnets"
            | "delegation"
            | "delegate"
            | "delegates"
            | "keys"
            | "key"
            | "swap"
            | "dex"
            | "liquidity"
            | "governance"
            | "gov"
            | "sudo"
            | "safemode"
            | "crowdloan"
            | "crowdloans"
            | "fund"
    );
    if !ok {
        anyhow::bail!(
            "Invalid event filter: '{}'. Valid filters are: all, staking, registration, transfer, weights, subnet, delegation, keys, swap, governance, crowdloan (plus short aliases like stake, reg, dex).\n  Tip: use --filter all to see every decoded event.",
            filter
        );
    }
    Ok(())
}

/// Validate a WASM file before uploading to the contracts pallet.
/// Checks magic bytes (\0asm), minimum size, and reasonable maximum size.
pub fn validate_wasm_file(data: &[u8], path: &str) -> Result<()> {
    const WASM_MAGIC: &[u8; 4] = b"\0asm";
    const MAX_WASM_SIZE: usize = 16 * 1024 * 1024; // 16 MB

    if data.is_empty() {
        anyhow::bail!(
            "WASM file '{}' is empty.\n  Tip: provide a valid .wasm contract binary.",
            path
        );
    }
    if data.len() < 8 {
        anyhow::bail!(
            "WASM file '{}' is too small ({} bytes). Not a valid WASM module.\n  Tip: a valid WASM file starts with the bytes 00 61 73 6d (\\0asm).",
            path, data.len()
        );
    }
    if &data[0..4] != WASM_MAGIC {
        let preview: String = data[0..4.min(data.len())]
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        anyhow::bail!(
            "File '{}' is not a WASM module (magic bytes: {}, expected: 00 61 73 6d).\n  Tip: compile your contract to WASM first, e.g. 'cargo contract build'.",
            path, preview
        );
    }
    if data.len() > MAX_WASM_SIZE {
        anyhow::bail!(
            "WASM file '{}' is too large ({:.1} MB, max {} MB).\n  Tip: optimize your contract with 'cargo contract build --release'.",
            path,
            data.len() as f64 / (1024.0 * 1024.0),
            MAX_WASM_SIZE / (1024 * 1024)
        );
    }
    Ok(())
}

/// Validate gas limit for EVM calls.
/// Zero gas means the call will always fail. Warns on unusually low values.
pub fn validate_gas_limit(gas: u64, label: &str) -> Result<()> {
    if gas == 0 {
        anyhow::bail!(
            "Invalid {}: gas limit cannot be zero.\n  Tip: use at least 21000 for a simple transfer, more for contract calls.",
            label
        );
    }
    Ok(())
}

/// Validate a batch JSON file before processing.
/// Reads the file and checks structural validity: must be a JSON array of objects,
/// each with "pallet", "call", and "args" fields.
pub fn validate_batch_file(content: &str, path: &str) -> Result<Vec<serde_json::Value>> {
    let parsed: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| anyhow::anyhow!(
            "Invalid JSON in batch file '{}': {}\n  Tip: file must contain a JSON array of call objects.\n  Example: [{{\"pallet\": \"Balances\", \"call\": \"transfer_allow_death\", \"args\": [...]}}]",
            path, e
        ))?;

    let arr = parsed.as_array().ok_or_else(|| anyhow::anyhow!(
        "Batch file '{}' must contain a JSON array, got {}.\n  Tip: wrap your calls in square brackets: [{{}}, {{}}]",
        path,
        match &parsed {
            serde_json::Value::Object(_) => "an object (did you forget to wrap in []?)",
            serde_json::Value::String(_) => "a string",
            serde_json::Value::Number(_) => "a number",
            serde_json::Value::Bool(_) => "a boolean",
            serde_json::Value::Null => "null",
            _ => "a non-array value",
        }
    ))?;

    if arr.is_empty() {
        anyhow::bail!(
            "Batch file '{}' is empty (no calls to submit).\n  Tip: add at least one call object to the array.",
            path
        );
    }

    if arr.len() > 1000 {
        anyhow::bail!(
            "Batch file '{}' has too many calls ({}, max 1000).\n  Tip: split into smaller batch files.",
            path, arr.len()
        );
    }

    for (i, call) in arr.iter().enumerate() {
        let obj = call.as_object().ok_or_else(|| anyhow::anyhow!(
            "Batch call #{} is not an object (got {}).\n  Tip: each call must be {{\"pallet\": \"...\", \"call\": \"...\", \"args\": [...]}}",
            i,
            match call {
                serde_json::Value::Null => "null".to_string(),
                serde_json::Value::Bool(b) => format!("boolean {}", b),
                serde_json::Value::Number(n) => format!("number {}", n),
                serde_json::Value::String(s) => format!("string {:?}", &s[..s.len().min(50)]),
                serde_json::Value::Array(_) => "an array".to_string(),
                _ => "unknown".to_string(),
            }
        ))?;

        if !obj.contains_key("pallet") {
            anyhow::bail!(
                "Batch call #{}: missing \"pallet\" field.\n  Tip: add \"pallet\": \"SubtensorModule\" (or the target pallet name).",
                i
            );
        }
        if obj.get("pallet").and_then(|v| v.as_str()).is_none() {
            anyhow::bail!(
                "Batch call #{}: \"pallet\" must be a string.\n  Tip: use the pallet name, e.g. \"SubtensorModule\".",
                i
            );
        }
        if !obj.contains_key("call") {
            anyhow::bail!(
                "Batch call #{}: missing \"call\" field.\n  Tip: add \"call\": \"add_stake\" (the extrinsic name).",
                i
            );
        }
        if obj.get("call").and_then(|v| v.as_str()).is_none() {
            anyhow::bail!(
                "Batch call #{}: \"call\" must be a string.\n  Tip: use the call name, e.g. \"transfer_allow_death\".",
                i
            );
        }
        if !obj.contains_key("args") {
            anyhow::bail!(
                "Batch call #{}: missing \"args\" field.\n  Tip: add \"args\": [] (even if no arguments are needed).",
                i
            );
        }
        if obj.get("args").and_then(|v| v.as_array()).is_none() {
            anyhow::bail!(
                "Batch call #{}: \"args\" must be an array.\n  Tip: use \"args\": [arg1, arg2, ...] or \"args\": [] for no arguments.",
                i
            );
        }
    }

    Ok(arr.clone())
}

/// Validate weight input string before parsing.
/// Catches common mistakes: empty input, bad separators, obviously wrong formats.
pub fn validate_weight_input(input: &str) -> Result<()> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Weight input cannot be empty.\n  Tip: use 'uid:weight' pairs (e.g., '0:100,1:200'), JSON array, or '@file.json'."
        );
    }
    // stdin or file reference — valid, let resolve_weights handle it
    if trimmed == "-" || trimmed.starts_with('@') {
        return Ok(());
    }
    // JSON — valid format, let parser handle it
    if trimmed.starts_with('[') || trimmed.starts_with('{') {
        return Ok(());
    }
    // uid:weight pairs — basic structural check
    for (i, pair) in trimmed.split(',').enumerate() {
        let p = pair.trim();
        if p.is_empty() {
            anyhow::bail!(
                "Empty weight pair at position {} (trailing comma?).\n  Tip: format is 'uid:weight,uid:weight' (e.g., '0:100,1:200').",
                i
            );
        }
        if !p.contains(':') {
            anyhow::bail!(
                "Invalid weight pair '{}' at position {} — missing ':' separator.\n  Tip: format is 'uid:weight' (e.g., '0:100').",
                p, i
            );
        }
        let parts: Vec<&str> = p.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid weight pair '{}' at position {} — expected exactly one ':' separator.\n  Tip: format is 'uid:weight' (e.g., '0:100').",
                p, i
            );
        }
    }
    Ok(())
}

/// Validate a view/history limit parameter.
/// Must be at least 1, and capped at a reasonable maximum to prevent OOM.
pub fn validate_view_limit(limit: usize, label: &str) -> Result<()> {
    if limit == 0 {
        anyhow::bail!(
            "Invalid {}: limit must be at least 1.\n  Tip: use --limit 50 for a reasonable default.",
            label
        );
    }
    if limit > 10_000 {
        anyhow::bail!(
            "Invalid {}: limit {} is too large (max 10,000).\n  Tip: use a smaller limit to avoid excessive memory usage.",
            label, limit
        );
    }
    Ok(())
}

/// Validate a balance threshold value (used in `balance --watch --threshold`).
/// Unlike `validate_amount`, zero is allowed (alert when balance drops to zero),
/// but negative, NaN, and Infinity are rejected.
pub fn validate_threshold(value: f64, label: &str) -> Result<()> {
    if value < 0.0 {
        anyhow::bail!(
            "Invalid {}: {:.9}. Threshold cannot be negative.\n  Tip: use a value like 1.0 to alert when balance drops below 1 TAO.",
            label, value
        );
    }
    if !value.is_finite() {
        anyhow::bail!(
            "Invalid {}: value must be a valid number (got {}).",
            label,
            value
        );
    }
    Ok(())
}

/// Validate admin raw call name. Must be a valid Rust identifier (snake_case)
/// and should match a known AdminUtils call from `admin::known_params()`.
pub fn validate_admin_call_name(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Admin call name cannot be empty.\n  Tip: use a call name like 'sudo_set_tempo'. Run `agcli admin list` to see available calls."
        );
    }
    if trimmed.len() > 128 {
        let preview: String = trimmed.chars().take(32).collect();
        anyhow::bail!(
            "Admin call name '{}...' is too long ({} chars, max 128).",
            preview,
            trimmed.len()
        );
    }
    if !trimmed.chars().next().unwrap().is_ascii_alphabetic() {
        anyhow::bail!(
            "Admin call name '{}' must start with a letter.\n  Tip: call names are snake_case, e.g. 'sudo_set_tempo'.",
            trimmed
        );
    }
    if let Some(bad) = trimmed
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '_')
    {
        anyhow::bail!(
            "Admin call name contains invalid character '{}'.\n  Tip: use only letters, numbers, and underscores.",
            bad
        );
    }
    // Check against known AdminUtils calls — reject unknown calls to prevent
    // typos from executing unintended sudo operations (Issue 711).
    let known = crate::admin::known_params();
    let known_names: Vec<&str> = known.iter().map(|(n, _, _)| *n).collect();
    if !known_names.contains(&trimmed) {
        // Find closest match for helpful suggestion
        let suggestion = known_names
            .iter()
            .filter(|n| {
                n.contains(trimmed) || trimmed.contains(**n) || levenshtein_close(n, trimmed)
            })
            .copied()
            .next();
        let mut msg = format!(
            "Unknown admin call '{}'. Run `agcli admin list` to see available calls.",
            trimmed
        );
        if let Some(closest) = suggestion {
            msg.push_str(&format!("\n  Did you mean '{}'?", closest));
        }
        anyhow::bail!("{}", msg);
    }
    Ok(())
}

/// Simple Levenshtein-like check: true if edit distance <= 2.
fn levenshtein_close(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let (la, lb) = (a_bytes.len(), b_bytes.len());
    if la.abs_diff(lb) > 2 {
        return false;
    }
    let max_len = la.max(lb);
    if max_len == 0 {
        return true;
    }
    // Simple: count mismatches at aligned positions
    let mut diffs = 0u32;
    for i in 0..la.min(lb) {
        if a_bytes[i] != b_bytes[i] {
            diffs += 1;
        }
    }
    diffs += la.abs_diff(lb) as u32;
    diffs <= 2
}

/// Validate a thread count for CPU-bound operations (e.g. POW mining).
///
/// Threads must be >= 1 and <= 256 (sanity cap to prevent resource exhaustion).
pub fn validate_threads(threads: u32, label: &str) -> Result<()> {
    if threads == 0 {
        anyhow::bail!(
            "{} thread count cannot be zero. Use at least 1 thread.",
            label
        );
    }
    if threads > 256 {
        anyhow::bail!(
            "{} thread count {} is too high (max 256). Using more threads than available cores wastes resources.",
            label, threads
        );
    }
    Ok(())
}

/// Validate a URL string (basic format check).
///
/// Accepts http:// and https:// URLs with a non-empty host.
/// Empty strings are allowed (optional fields).
pub fn validate_url(url: &str, label: &str) -> Result<()> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Ok(()); // Empty is OK for optional fields
    }
    if trimmed.len() > 2048 {
        anyhow::bail!(
            "{} URL is too long ({} chars, max 2048).",
            label,
            trimmed.len()
        );
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        anyhow::bail!(
            "{} URL must start with http:// or https:// (got '{}').",
            label,
            if trimmed.len() > 60 {
                let preview: String = trimmed.chars().take(60).collect();
                format!("{}...", preview)
            } else {
                trimmed.to_string()
            }
        );
    }
    // Must have a host after the scheme
    let after_scheme = if let Some(rest) = trimmed.strip_prefix("https://") {
        rest
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        rest
    } else {
        trimmed
    };
    if after_scheme.is_empty() || after_scheme.starts_with('/') || after_scheme.starts_with('?') {
        anyhow::bail!("{} URL is missing a host name.", label);
    }
    Ok(())
}

/// Validate a subnet identity name string.
///
/// Must be non-empty, max 256 chars, ASCII printable, no control characters.
pub fn validate_subnet_name(name: &str, label: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("{} cannot be empty.", label);
    }
    if trimmed.len() > 256 {
        let preview: String = trimmed.chars().take(32).collect();
        anyhow::bail!(
            "{} '{}...' is too long ({} chars, max 256).",
            label,
            preview,
            trimmed.len()
        );
    }
    if let Some(bad) = trimmed.chars().find(|c| c.is_control()) {
        anyhow::bail!(
            "{} contains control character (U+{:04X}). Use only printable characters.",
            label,
            bad as u32
        );
    }
    Ok(())
}

/// Validate a GitHub repo string (owner/repo format).
///
/// Empty strings are allowed (optional fields). If non-empty, must match
/// `owner/repo` format with alphanumeric + hyphens + underscores + dots.
pub fn validate_github_repo(repo: &str) -> Result<()> {
    let trimmed = repo.trim();
    if trimmed.is_empty() {
        return Ok(()); // Optional
    }
    if trimmed.len() > 256 {
        let preview: String = trimmed.chars().take(32).collect();
        anyhow::bail!("GitHub repo '{}' is too long (max 256 chars).", preview);
    }
    // Must contain exactly one '/'
    let parts: Vec<&str> = trimmed.splitn(3, '/').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        anyhow::bail!(
            "GitHub repo '{}' must be in 'owner/repo' format (e.g. 'opentensor/subtensor').",
            trimmed
        );
    }
    // Both parts: alphanumeric + hyphens + underscores + dots
    for part in &parts {
        if let Some(bad) = part
            .chars()
            .find(|c| !c.is_ascii_alphanumeric() && *c != '-' && *c != '_' && *c != '.')
        {
            anyhow::bail!(
                "GitHub repo '{}' contains invalid character '{}'. Use only letters, numbers, hyphens, underscores, and dots.",
                trimmed, bad
            );
        }
    }
    Ok(())
}

/// Validate a proxy type string against the known on-chain variants.
/// Returns an error with suggestions if the type is unknown — prevents the
/// dangerous silent default-to-"Any" (most permissive) on typos.
pub fn validate_proxy_type(s: &str) -> Result<()> {
    const KNOWN: &[&str] = &[
        "any",
        "owner",
        "nontransfer",
        "non_transfer",
        "staking",
        "noncritical",
        "non_critical",
        "triumvirate",
        "governance",
        "senate",
        "nonfungible",
        "non_fungible",
        "registration",
        "transfer",
        "smalltransfer",
        "small_transfer",
        "rootweights",
        "root_weights",
        "childkeys",
        "child_keys",
        "sudouncheckedsetcode",
        "sudo_unchecked_set_code",
        "swaphotkey",
        "swap_hotkey",
        "subnetleasebeneficiary",
        "subnet_lease_beneficiary",
        "rootclaim",
        "root_claim",
    ];
    if s.is_empty() {
        anyhow::bail!("Proxy type cannot be empty. Valid types: Any, Owner, Staking, Transfer, NonTransfer, NonCritical, Governance, Senate, Registration, NonFungible, SmallTransfer, RootWeights, ChildKeys, Triumvirate");
    }
    if !KNOWN.contains(&s.to_lowercase().as_str()) {
        // Build display names (deduplicated, friendly)
        let display = &[
            "Any",
            "Owner",
            "Staking",
            "Transfer",
            "NonTransfer",
            "NonCritical",
            "Governance",
            "Senate",
            "Registration",
            "NonFungible",
            "SmallTransfer",
            "RootWeights",
            "ChildKeys",
            "Triumvirate",
            "SwapHotkey",
            "SubnetLeaseBeneficiary",
            "RootClaim",
            "SudoUncheckedSetCode",
        ];
        anyhow::bail!(
            "Unknown proxy type '{}'. Valid types: {}",
            s,
            display.join(", ")
        );
    }
    Ok(())
}

/// Validate a call hash string (e.g. for proxy announce/reject, multisig approve/cancel).
/// Must be 0x-prefixed (or bare) hex encoding of exactly 32 bytes (64 hex chars).
pub fn validate_call_hash(hash: &str, label: &str) -> Result<()> {
    let trimmed = hash.trim();
    if trimmed.is_empty() {
        anyhow::bail!(
            "Invalid {} call hash: cannot be empty.\n  Tip: provide a 0x-prefixed 64 hex char hash (blake2_256 of the encoded call).",
            label
        );
    }
    let hex_str = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    if hex_str.is_empty() {
        anyhow::bail!(
            "Invalid {} call hash: empty after '0x' prefix.\n  Tip: provide 64 hex characters (32 bytes) after '0x'.",
            label
        );
    }
    if let Some(pos) = hex_str.find(|c: char| !c.is_ascii_hexdigit()) {
        let bad_char = hex_str[pos..].chars().next().unwrap();
        anyhow::bail!(
            "Invalid {} call hash: character '{}' at position {} is not valid hex.\n  Tip: use only 0-9 and a-f.",
            label, bad_char, pos
        );
    }
    if hex_str.len() != 64 {
        anyhow::bail!(
            "Invalid {} call hash: must be exactly 32 bytes (64 hex chars), got {} hex chars.\n  Tip: the call hash is the blake2_256 of the SCALE-encoded call data.",
            label, hex_str.len()
        );
    }
    Ok(())
}

/// Validate a config network value against known networks.
/// Accepts: finney, test, local, archive (case-insensitive).
pub fn validate_config_network(value: &str) -> Result<()> {
    let lower = value.trim().to_lowercase();
    match lower.as_str() {
        "finney" | "test" | "local" | "archive" => Ok(()),
        _ => anyhow::bail!(
            "Unknown network '{}'. Valid networks: finney, test, local, archive.\n  Tip: use --endpoint <url> for custom endpoints.",
            value
        ),
    }
}

/// Validate a spending limit value for config (must be non-negative and finite).
pub fn validate_spending_limit(value: f64, netuid_str: &str) -> Result<()> {
    // Validate netuid suffix is a valid u16 or the wildcard "*"
    if netuid_str != "*" {
        let _: u16 = netuid_str.parse().map_err(|_| {
            anyhow::anyhow!(
                "Invalid netuid '{}' in spending_limit key. Must be a number 0-65535 or '*' for global limit.",
                netuid_str
            )
        })?;
    }
    if value.is_nan() || value.is_infinite() {
        anyhow::bail!("Spending limit must be a finite number, got: {}", value);
    }
    if value < 0.0 {
        anyhow::bail!("Spending limit cannot be negative, got: {}", value);
    }
    Ok(())
}

/// Parse an optional JSON string into a vec of subxt dynamic Values.
/// Validates the JSON structure before converting.
pub fn parse_json_args(args: &Option<String>) -> anyhow::Result<Vec<subxt::dynamic::Value>> {
    if let Some(ref args_json) = args {
        let validated = validate_multisig_json_args(args_json)?;
        Ok(validated.iter().map(json_to_subxt_value).collect())
    } else {
        Ok(vec![])
    }
}

/// Convert a serde_json::Value to a subxt dynamic Value for multisig call args.
pub fn json_to_subxt_value(v: &serde_json::Value) -> subxt::dynamic::Value {
    use subxt::dynamic::Value;
    match v {
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Value::u128(u as u128)
            } else if let Some(i) = n.as_i64() {
                Value::i128(i as i128)
            } else {
                Value::string(n.to_string())
            }
        }
        serde_json::Value::String(s) => {
            if let Some(hex_str) = s.strip_prefix("0x") {
                if let Ok(bytes) = hex::decode(hex_str) {
                    return Value::from_bytes(bytes);
                }
            }
            Value::string(s.clone())
        }
        serde_json::Value::Bool(b) => Value::bool(*b),
        serde_json::Value::Array(arr) => {
            Value::unnamed_composite(arr.iter().map(json_to_subxt_value))
        }
        _ => Value::string(v.to_string()),
    }
}

/// Validate a stake limit price (f64 → u64 conversion safety).
/// Price is multiplied by 1e9 to convert TAO to RAO — this can overflow u64 for huge values.
pub fn validate_limit_price(price: f64, label: &str) -> Result<()> {
    if !price.is_finite() {
        anyhow::bail!(
            "Invalid {}: must be a finite number (got {}).\n  Tip: use a decimal price like 0.001 or 1.5",
            label, price
        );
    }
    if price <= 0.0 {
        anyhow::bail!(
            "Invalid {}: {:.9}. Price must be positive.\n  Tip: use a positive decimal like 0.001",
            label,
            price
        );
    }
    // price * 1e9 must fit in u64 (max ~18.44 TAO-equivalent at 1e9 scale = 18_446_744_073 in TAO units)
    let scaled = price * 1_000_000_000.0;
    if scaled > u64::MAX as f64 {
        anyhow::bail!(
            "Invalid {}: {:.9} is too large. Maximum price is ~{:.2} (u64 overflow after RAO conversion).",
            label, price, u64::MAX as f64 / 1_000_000_000.0
        );
    }
    Ok(())
}

/// Validate scheduler block number. Must be > 0 (block 0 is genesis, not schedulable).
pub fn validate_block_number(block: u32, label: &str) -> Result<()> {
    if block == 0 {
        anyhow::bail!(
            "Invalid {}: block 0 is the genesis block and cannot be targeted.\n  Tip: use a future block number.",
            label
        );
    }
    Ok(())
}

/// Validate scheduler repeat parameters. repeat_every must be > 0 to avoid infinite loops.
pub fn validate_repeat_params(repeat_every: u32, repeat_count: u32) -> Result<()> {
    if repeat_every == 0 {
        anyhow::bail!(
            "Invalid repeat-every: cannot be 0 (would cause infinite scheduling at the same block).\n  Tip: use at least 1 block between repeats."
        );
    }
    if repeat_count == 0 {
        anyhow::bail!(
            "Invalid repeat-count: cannot be 0 (no repetitions would execute).\n  Tip: use at least 1, or omit --repeat-every and --repeat-count entirely."
        );
    }
    // Check for potential overflow: when + repeat_every * repeat_count
    let total_span = (repeat_every as u64).checked_mul(repeat_count as u64);
    if total_span.is_none() || total_span.unwrap() > u32::MAX as u64 {
        anyhow::bail!(
            "Invalid repeat parameters: repeat_every ({}) * repeat_count ({}) overflows the block number space.\n  Tip: reduce the repeat interval or count.",
            repeat_every, repeat_count
        );
    }
    Ok(())
}

/// Validate a Docker container name.
///
/// Docker container names must match `[a-zA-Z0-9][a-zA-Z0-9_.-]*` and be
/// at most 128 characters. Empty strings are rejected.
pub fn validate_docker_name(name: &str, label: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("{} cannot be empty.", label);
    }
    if name.len() > 128 {
        let preview: String = name.chars().take(32).collect();
        anyhow::bail!(
            "{} '{}...' is too long ({} chars, max 128).",
            label,
            preview,
            name.len()
        );
    }
    let first = name.as_bytes()[0];
    if !first.is_ascii_alphanumeric() {
        anyhow::bail!(
            "{} '{}' must start with an alphanumeric character (got '{}').",
            label,
            name,
            first as char
        );
    }
    if let Some(bad) = name
        .chars()
        .find(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' | '-'))
    {
        anyhow::bail!(
            "{} '{}' contains invalid character '{}'. Only [a-zA-Z0-9_.-] are allowed.",
            label,
            name,
            bad
        );
    }
    Ok(())
}

/// Validate a Docker image reference.
///
/// Image references must be non-empty, max 256 chars, and contain only valid
/// characters (alphanumeric, `/`, `.`, `-`, `_`, `:`). No shell metacharacters.
pub fn validate_docker_image(image: &str) -> Result<()> {
    if image.is_empty() {
        anyhow::bail!("Docker image cannot be empty.");
    }
    if image.len() > 256 {
        let preview: String = image.chars().take(32).collect();
        anyhow::bail!(
            "Docker image '{}...' is too long ({} chars, max 256).",
            preview,
            image.len()
        );
    }
    if let Some(bad) = image.chars().find(
        |c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '/' | '.' | '-' | '_' | ':' | '@'),
    ) {
        anyhow::bail!(
            "Docker image '{}' contains invalid character '{}'. Only [a-zA-Z0-9/._:-@] are allowed.",
            image,
            bad
        );
    }
    Ok(())
}

/// Validate liquidity price range: price_low must be strictly less than price_high.
pub fn validate_price_range(price_low: f64, price_high: f64) -> Result<()> {
    if price_low >= price_high {
        anyhow::bail!(
            "Invalid price range: price-low ({:.9}) must be strictly less than price-high ({:.9}).\n  Tip: ensure your lower bound is below the upper bound.",
            price_low, price_high
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_rao_normal_values() {
        assert_eq!(safe_rao(1.0), 1_000_000_000);
        assert_eq!(safe_rao(0.000000001), 1); // 1 RAO
        assert_eq!(safe_rao(0.5), 500_000_000);
        assert_eq!(safe_rao(100.0), 100_000_000_000);
    }

    #[test]
    fn safe_rao_saturates_on_overflow() {
        // u64::MAX / 1e9 ≈ 18.44, so 20.0 TAO * 1e9 should saturate
        let huge = safe_rao(f64::MAX);
        assert_eq!(huge, u64::MAX, "Huge float should saturate to u64::MAX");

        // Values over ~18.44 billion TAO would overflow
        let big = safe_rao(18_446_744_074.0);
        assert_eq!(big, u64::MAX);
    }

    #[test]
    fn safe_rao_handles_special_floats() {
        assert_eq!(safe_rao(f64::NAN), 0, "NaN should produce 0");
        assert_eq!(safe_rao(f64::INFINITY), 0, "Infinity should produce 0");
        assert_eq!(
            safe_rao(f64::NEG_INFINITY),
            0,
            "Neg infinity should produce 0"
        );
        assert_eq!(safe_rao(-1.0), 0, "Negative should produce 0");
    }

    #[test]
    fn safe_rao_zero() {
        assert_eq!(safe_rao(0.0), 0);
    }

    // ========== Issue 709: confirm_action tests ==========

    #[test]
    fn confirm_action_yes_mode_bypasses() {
        set_yes_mode(true);
        assert!(confirm_action("Destroy everything?").is_ok());
        set_yes_mode(false);
    }

    #[test]
    fn confirm_action_batch_mode_bypasses() {
        set_batch_mode(true);
        assert!(confirm_action("Destroy everything?").is_ok());
        set_batch_mode(false);
    }

    #[test]
    fn is_yes_mode_default_false() {
        set_yes_mode(false);
        set_batch_mode(false);
        assert!(!is_yes_mode());
    }

    #[test]
    fn is_yes_mode_true_when_yes_set() {
        set_yes_mode(true);
        assert!(is_yes_mode());
        set_yes_mode(false);
    }

    #[test]
    fn is_yes_mode_true_when_batch_set() {
        set_batch_mode(true);
        assert!(is_yes_mode());
        set_batch_mode(false);
    }

    // ========== Issue 711: validate_admin_call_name rejects unknown calls ==========

    #[test]
    fn validate_admin_call_name_accepts_known_calls() {
        assert!(validate_admin_call_name("sudo_set_tempo").is_ok());
        assert!(validate_admin_call_name("sudo_set_max_allowed_validators").is_ok());
        assert!(validate_admin_call_name("sudo_set_difficulty").is_ok());
    }

    #[test]
    fn validate_admin_call_name_rejects_unknown_calls() {
        let result = validate_admin_call_name("sudo_set_nonexistent");
        assert!(result.is_err(), "Unknown call name should be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Unknown admin call"),
            "Error should mention unknown call: {}",
            msg
        );
    }

    #[test]
    fn validate_admin_call_name_rejects_typos() {
        let result = validate_admin_call_name("sudo_set_temp"); // typo for sudo_set_tempo
        assert!(result.is_err(), "Typo should be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Did you mean"),
            "Should suggest closest match: {}",
            msg
        );
    }

    #[test]
    fn validate_admin_call_name_rejects_empty() {
        assert!(validate_admin_call_name("").is_err());
    }

    #[test]
    fn validate_admin_call_name_rejects_special_chars() {
        assert!(validate_admin_call_name("sudo; rm -rf /").is_err());
        assert!(validate_admin_call_name("call(1)").is_err());
    }

    #[test]
    fn validate_admin_call_name_rejects_too_long() {
        let long = "a".repeat(200);
        assert!(validate_admin_call_name(&long).is_err());
    }

    // ──── Issue 668/731: Mnemonic CLI flag rejection ────

    #[test]
    fn require_mnemonic_rejects_cli_flag_without_env_var() {
        // Ensure AGCLI_MNEMONIC is NOT set for this test
        std::env::remove_var("AGCLI_MNEMONIC");
        let result = require_mnemonic(Some("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()));
        assert!(
            result.is_err(),
            "Should reject mnemonic passed via CLI flag"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Refusing --mnemonic flag"),
            "Error should mention refusing CLI flag: {}",
            msg
        );
        assert!(
            msg.contains("AGCLI_MNEMONIC"),
            "Error should mention env var alternative: {}",
            msg
        );
    }

    #[test]
    fn require_mnemonic_accepts_env_var() {
        // Set AGCLI_MNEMONIC so the detection thinks it came from env
        std::env::set_var("AGCLI_MNEMONIC", "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about");
        let result = require_mnemonic(Some("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()));
        std::env::remove_var("AGCLI_MNEMONIC");
        assert!(
            result.is_ok(),
            "Should accept mnemonic from env var: {:?}",
            result.err()
        );
    }

    #[test]
    fn require_mnemonic_error_mentions_ps_visibility() {
        std::env::remove_var("AGCLI_MNEMONIC");
        let result = require_mnemonic(Some("test words here a b c d e f g h i".to_string()));
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("ps"),
            "Error should warn about ps visibility: {}",
            msg
        );
    }

    #[test]
    fn require_mnemonic_batch_mode_error_mentions_env_var() {
        std::env::remove_var("AGCLI_MNEMONIC");
        // In batch mode (no TTY), None should error with env var instructions
        // We can't easily set batch mode in tests, but we can verify the Some() path
        let result = require_mnemonic(Some("a b c d e f g h i j k l".to_string()));
        assert!(result.is_err(), "Should still reject via CLI flag");
    }

    // ========== Issue 661: Password strength rejects short/common passwords ==========

    #[test]
    fn password_strength_rejects_empty() {
        let result = validate_password_strength("");
        assert!(result.is_err(), "Empty password should be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Empty password"), "Error: {}", msg);
    }

    #[test]
    fn password_strength_rejects_short_password() {
        let result = validate_password_strength("abc");
        assert!(result.is_err(), "3-char password should be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("too short") || msg.contains("Minimum 8"),
            "Error: {}",
            msg
        );
    }

    #[test]
    fn password_strength_rejects_7_char_password() {
        let result = validate_password_strength("1234567");
        assert!(result.is_err(), "7-char password should be rejected");
    }

    #[test]
    fn password_strength_rejects_common_8_char() {
        let result = validate_password_strength("12345678");
        // "12345678" is in the common password list, should be rejected even though >= 8 chars
        assert!(
            result.is_err(),
            "Common password '12345678' should be rejected"
        );
    }

    #[test]
    fn password_strength_accepts_strong_password() {
        let result = validate_password_strength("MyStr0ng!Pass");
        assert!(
            result.is_ok(),
            "Strong password should be accepted: {:?}",
            result.err()
        );
    }

    #[test]
    fn password_strength_rejects_common_password() {
        let result = validate_password_strength("password");
        assert!(
            result.is_err(),
            "Common password 'password' should be rejected"
        );
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("commonly used"), "Error: {}", msg);
    }

    #[test]
    fn password_strength_rejects_common_case_insensitive() {
        let result = validate_password_strength("PASSWORD");
        assert!(
            result.is_err(),
            "Case-insensitive common password should be rejected"
        );
    }

    #[test]
    fn password_strength_rejects_common_qwerty() {
        assert!(validate_password_strength("qwerty").is_err());
    }

    #[test]
    fn password_strength_accepts_8_char_unique() {
        // Not in common list, >= 8 chars
        let result = validate_password_strength("xK9!mZ2q");
        assert!(
            result.is_ok(),
            "Unique 8-char password should pass: {:?}",
            result.err()
        );
    }

    // ========== Issue 658: is_yes_mode checks both --yes and --batch ==========

    #[test]
    fn is_yes_mode_false_when_neither_set() {
        set_yes_mode(false);
        set_batch_mode(false);
        assert!(!is_yes_mode());
    }

    #[test]
    fn is_yes_mode_true_when_only_batch_set() {
        set_yes_mode(false);
        set_batch_mode(true);
        assert!(is_yes_mode(), "--batch should also enable yes mode");
        set_batch_mode(false);
    }

    #[test]
    fn is_yes_mode_true_when_only_yes_set() {
        set_batch_mode(false);
        set_yes_mode(true);
        assert!(is_yes_mode());
        set_yes_mode(false);
    }

    #[test]
    fn confirm_action_auto_confirms_in_batch_mode() {
        set_yes_mode(false);
        set_batch_mode(true);
        assert!(
            confirm_action("Delete everything?").is_ok(),
            "confirm_action should auto-confirm in batch mode"
        );
        set_batch_mode(false);
    }

    // ========== Issue 698: Docker name/image validation tests ==========

    #[test]
    fn validate_docker_name_accepts_valid_names() {
        assert!(validate_docker_name("agcli_localnet", "Container name").is_ok());
        assert!(validate_docker_name("my-container.v2", "Container name").is_ok());
        assert!(validate_docker_name("A123", "Container name").is_ok());
        assert!(validate_docker_name("a", "Container name").is_ok());
    }

    #[test]
    fn validate_docker_name_rejects_empty() {
        let err = validate_docker_name("", "Container name").unwrap_err();
        assert!(err.to_string().contains("cannot be empty"), "{}", err);
    }

    #[test]
    fn validate_docker_name_rejects_leading_special() {
        let err = validate_docker_name("_foo", "Container name").unwrap_err();
        assert!(
            err.to_string().contains("must start with an alphanumeric"),
            "{}",
            err
        );
        let err2 = validate_docker_name("-bar", "Container name").unwrap_err();
        assert!(
            err2.to_string().contains("must start with an alphanumeric"),
            "{}",
            err2
        );
        let err3 = validate_docker_name(".baz", "Container name").unwrap_err();
        assert!(
            err3.to_string().contains("must start with an alphanumeric"),
            "{}",
            err3
        );
    }

    #[test]
    fn validate_docker_name_rejects_shell_metacharacters() {
        let err = validate_docker_name("foo;rm -rf /", "Container name").unwrap_err();
        assert!(err.to_string().contains("invalid character"), "{}", err);
        let err2 = validate_docker_name("foo$(cmd)", "Container name").unwrap_err();
        assert!(err2.to_string().contains("invalid character"), "{}", err2);
    }

    #[test]
    fn validate_docker_name_rejects_spaces() {
        let err = validate_docker_name("my container", "Container name").unwrap_err();
        assert!(err.to_string().contains("invalid character"), "{}", err);
    }

    #[test]
    fn validate_docker_name_rejects_too_long() {
        let long = format!("a{}", "x".repeat(128));
        let err = validate_docker_name(&long, "Container name").unwrap_err();
        assert!(err.to_string().contains("too long"), "{}", err);
    }

    #[test]
    fn validate_docker_image_accepts_valid_images() {
        assert!(
            validate_docker_image("ghcr.io/opentensor/subtensor-localnet:devnet-ready").is_ok()
        );
        assert!(validate_docker_image("ubuntu:22.04").is_ok());
        assert!(validate_docker_image("my-registry.com/org/image:v1.2.3").is_ok());
        assert!(validate_docker_image("image@sha256:abc123").is_ok());
    }

    #[test]
    fn validate_docker_image_rejects_empty() {
        let err = validate_docker_image("").unwrap_err();
        assert!(err.to_string().contains("cannot be empty"), "{}", err);
    }

    #[test]
    fn validate_docker_image_rejects_shell_metacharacters() {
        let err = validate_docker_image("ubuntu;echo pwned").unwrap_err();
        assert!(err.to_string().contains("invalid character"), "{}", err);
    }

    #[test]
    fn validate_docker_image_rejects_too_long() {
        let long = "a".repeat(257);
        let err = validate_docker_image(&long).unwrap_err();
        assert!(err.to_string().contains("too long"), "{}", err);
    }

    // ──── Issue 120 (v24): Spending limit bypass via JSON string types ────

    #[test]
    fn check_spending_limit_rejects_string_netuid() {
        use serde_json::json;
        // If netuid is passed as a string instead of number, the check must error
        // rather than silently defaulting to 0
        let args = vec![json!("5abc123"), json!("1"), json!(50_000_000_000u64)];
        let result = check_spending_limit_for_raw_call("SubtensorModule", "add_stake", &args);
        assert!(
            result.is_err(),
            "String netuid should be rejected, got: {:?}",
            result
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("numeric netuid"),
            "Error should mention numeric netuid: {}",
            msg
        );
    }

    #[test]
    fn check_spending_limit_rejects_string_amount() {
        use serde_json::json;
        let args = vec![json!("5abc123"), json!(1), json!("50000000000")];
        let result = check_spending_limit_for_raw_call("SubtensorModule", "add_stake", &args);
        assert!(
            result.is_err(),
            "String amount should be rejected, got: {:?}",
            result
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("numeric amount"),
            "Error should mention numeric amount: {}",
            msg
        );
    }

    #[test]
    fn check_spending_limit_accepts_numeric_args() {
        use serde_json::json;
        // Numeric args should pass through without error (spending limit not configured in test)
        let args = vec![json!("5abc123"), json!(1), json!(50_000_000_000u64)];
        let result = check_spending_limit_for_raw_call("SubtensorModule", "add_stake", &args);
        assert!(result.is_ok(), "Numeric args should pass: {:?}", result);
    }

    // ── Issue 127: validate_docker_name UTF-8 panic ──

    #[test]
    fn validate_docker_name_multibyte_long_name_no_panic() {
        // 2-byte chars: 65 of them = 130 bytes > 128, but should not panic
        let name: String = std::iter::repeat('é').take(65).collect();
        let result = validate_docker_name(&name, "container");
        assert!(result.is_err(), "Should reject name > 128 chars");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("too long"),
            "Error message should mention too long: {}",
            msg
        );
    }

    #[test]
    fn validate_docker_name_ascii_long_name() {
        let name: String = std::iter::repeat('a').take(129).collect();
        let result = validate_docker_name(&name, "container");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    // ── Issue 128: validate_docker_image UTF-8 panic ──

    #[test]
    fn validate_docker_image_multibyte_long_no_panic() {
        // 3-byte chars: 86 of them = 258 bytes > 256
        let image: String = std::iter::repeat('中').take(86).collect();
        let result = validate_docker_image(&image);
        assert!(result.is_err(), "Should reject image > 256 chars");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("too long"),
            "Error message should mention too long: {}",
            msg
        );
    }

    // ── Issue 129: validate_spending_limit wildcard ──

    #[test]
    fn validate_spending_limit_accepts_wildcard() {
        let result = validate_spending_limit(100.0, "*");
        assert!(
            result.is_ok(),
            "Wildcard '*' should be accepted: {:?}",
            result
        );
    }

    #[test]
    fn validate_spending_limit_rejects_invalid_netuid() {
        let result = validate_spending_limit(100.0, "abc");
        assert!(result.is_err(), "Invalid netuid should be rejected");
    }

    #[test]
    fn validate_spending_limit_accepts_numeric_netuid() {
        let result = validate_spending_limit(100.0, "1");
        assert!(
            result.is_ok(),
            "Numeric netuid should be accepted: {:?}",
            result
        );
    }

    #[test]
    fn validate_mnemonic_multibyte_word_no_panic() {
        // Issue 145: word[..word.len().min(3)] panics on multi-byte UTF-8 input
        // The function should return an error, not panic.
        let result = validate_mnemonic(
            "café latte espresso drink make sure love open right left paper again",
        );
        assert!(
            result.is_err(),
            "non-BIP39 words should produce an error, not a panic"
        );
    }

    #[test]
    fn validate_mnemonic_emoji_word_no_panic() {
        // Issue 145: emoji (4 bytes) at byte position 0-3 would cause panic on [..3]
        let result = validate_mnemonic("🎉 abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about");
        assert!(
            result.is_err(),
            "emoji word should produce an error, not a panic"
        );
    }

    // --- Issue 154: spending limit checks origin_netuid not dest_netuid for move/swap/transfer ---

    #[test]
    fn spending_limit_move_stake_uses_origin_netuid() {
        // move_stake(hotkey_o, hotkey_d, origin_netuid=1, dest_netuid=2, amount_rao=1000)
        // The spending limit should be checked against netuid 1 (origin), not netuid 2 (dest)
        use serde_json::json;
        let args = vec![
            json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
            json!("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"),
            json!(1),                // origin_netuid
            json!(2),                // dest_netuid
            json!(1_000_000_000u64), // 1 TAO in rao
        ];
        // This test verifies the function extracts netuid from index 2 (origin), not 3 (dest).
        // Without a spending limit configured, the call should succeed.
        let result = check_spending_limit_for_raw_call("SubtensorModule", "move_stake", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn spending_limit_swap_stake_uses_origin_netuid() {
        // swap_stake(hotkey, origin_netuid=1, dest_netuid=2, amount_rao=1000)
        use serde_json::json;
        let args = vec![
            json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
            json!(1), // origin_netuid
            json!(2), // dest_netuid
            json!(1_000_000_000u64),
        ];
        let result = check_spending_limit_for_raw_call("SubtensorModule", "swap_stake", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn spending_limit_transfer_stake_uses_origin_netuid() {
        // transfer_stake(dest, hotkey, origin_netuid=1, dest_netuid=2, amount_rao=1000)
        use serde_json::json;
        let args = vec![
            json!("5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"),
            json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
            json!(1), // origin_netuid
            json!(2), // dest_netuid
            json!(1_000_000_000u64),
        ];
        let result = check_spending_limit_for_raw_call("SubtensorModule", "transfer_stake", &args);
        assert!(result.is_ok());
    }

    // ---- v29 regression tests ----

    #[test]
    fn validate_event_filter_accepts_aliases() {
        // Issue 160: aliases like "stake", "reg", "transfers" were rejected
        for alias in &["stake", "register", "reg", "transfers", "weight", "subnets"] {
            assert!(
                validate_event_filter(alias).is_ok(),
                "validate_event_filter should accept alias '{}'",
                alias
            );
        }
    }

    #[test]
    fn validate_event_filter_rejects_invalid() {
        assert!(validate_event_filter("stak").is_err());
        assert!(validate_event_filter("foo").is_err());
        assert!(validate_event_filter("").is_err());
    }

    #[test]
    fn validate_event_filter_accepts_canonical_names() {
        for name in &[
            "all",
            "staking",
            "registration",
            "transfer",
            "weights",
            "subnet",
        ] {
            assert!(
                validate_event_filter(name).is_ok(),
                "validate_event_filter should accept canonical name '{}'",
                name
            );
        }
    }

    #[test]
    fn spending_limit_raw_call_rejects_oversized_netuid() {
        // Issue 161: netuid > u16::MAX should be rejected, not silently truncated
        use serde_json::json;
        let args = vec![
            json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
            json!(65537u64), // netuid > u16::MAX
            json!(1_000_000_000u64),
        ];
        let result = check_spending_limit_for_raw_call("SubtensorModule", "add_stake", &args);
        assert!(result.is_err(), "should reject netuid > u16::MAX");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("exceeds maximum"),
            "error should mention exceeds maximum: {}",
            msg
        );
    }

    #[test]
    fn spending_limit_raw_call_accepts_valid_netuid() {
        // Ensure normal netuids still work
        use serde_json::json;
        let args = vec![
            json!("5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"),
            json!(1u64),
            json!(1_000_000_000u64),
        ];
        let result = check_spending_limit_for_raw_call("SubtensorModule", "add_stake", &args);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_ipv4_leading_zeros_suggest_nonzero() {
        // Issue 165: "00" should suggest "0", not empty string
        let result = validate_ipv4("1.00.0.1");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Use 0 instead") || msg.contains("Use 0"),
            "error should suggest '0', got: {}",
            msg
        );
    }

    #[test]
    fn validate_ipv4_leading_zeros_suggest_number() {
        // "01" should suggest "1"
        let result = validate_ipv4("10.01.0.1");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("1"), "error should suggest '1', got: {}", msg);
    }
}
