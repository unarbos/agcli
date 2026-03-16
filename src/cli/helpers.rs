//! Shared CLI helper functions.

use crate::wallet::Wallet;
use anyhow::Result;

use crate::cli::OutputFormat;

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
}

/// Escape a value for RFC 4180 CSV output.
/// If the value contains a comma, double-quote, or newline, wrap it in double-quotes
/// and escape any internal double-quotes by doubling them.
pub fn csv_escape(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
        let escaped = val.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        val.to_string()
    }
}

/// Join CSV fields with commas, escaping each field per RFC 4180.
pub fn csv_row_from(fields: &[&str]) -> String {
    fields
        .iter()
        .map(|f| csv_escape(f))
        .collect::<Vec<_>>()
        .join(",")
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
            "Wallet '{}' not found in {}.\n  Create one with: agcli wallet create --name {}\n  List existing:   agcli wallet list",
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
            label, amount
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
            label, amount
        );
    }
    Ok(())
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
            label, trimmed, trimmed.len()
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
    if trimmed.starts_with('/') || trimmed.starts_with('\\') || (trimmed.len() >= 2 && trimmed.as_bytes()[1] == b':') {
        anyhow::bail!(
            "Invalid {} name: \"{}\" looks like an absolute path. Use a simple name.",
            label, trimmed
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
    if !trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        anyhow::bail!(
            "Invalid {} name: \"{}\" contains invalid characters.\n  Tip: use only letters, numbers, hyphens, and underscores.",
            label, trimmed
        );
    }
    // OS reserved names (Windows)
    let upper = trimmed.to_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    if reserved.contains(&upper.as_str()) {
        anyhow::bail!(
            "Invalid {} name: \"{}\" is a reserved system name.",
            label, trimmed
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
                ip, i + 1, part.trim_start_matches('0')
            );
        }
        octets[i] = part.parse::<u8>().map_err(|_| {
            anyhow::anyhow!(
                "Invalid IPv4 address: \"{}\" — octet {} ('{}') is not a valid number (0–255).",
                ip, i + 1, part
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

/// Thread-local batch mode flag (set by main before dispatch).
static BATCH_MODE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Set batch mode globally (called from execute()).
pub fn set_batch_mode(batch: bool) {
    BATCH_MODE.store(batch, std::sync::atomic::Ordering::Relaxed);
}

/// Check if batch mode is active.
pub fn is_batch_mode() -> bool {
    BATCH_MODE.load(std::sync::atomic::Ordering::Relaxed)
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

pub fn resolve_hotkey_ss58(
    hotkey_arg: Option<String>,
    wallet: &mut Wallet,
    hotkey_name: &str,
) -> Result<String> {
    if let Some(hk) = hotkey_arg {
        return Ok(hk);
    }
    wallet.load_hotkey(hotkey_name)?;
    wallet
        .hotkey_ss58()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Could not resolve hotkey address.\n  Tip: pass --hotkey <ss58_address> or create a hotkey with `agcli wallet create-hotkey`."))
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
    let mut result = Vec::new();
    for pair in children_str.split(',') {
        let parts: Vec<&str> = pair.trim().split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid child pair '{}'. Format: 'proportion:hotkey_ss58' (e.g., '50000:5Cai...')",
                pair
            );
        }
        let proportion = parts[0].trim().parse::<u64>().map_err(|_| {
            anyhow::anyhow!(
                "Invalid proportion '{}' — must be a positive integer (u64)",
                parts[0].trim()
            )
        })?;
        let hotkey = parts[1].trim().to_string();
        result.push((proportion, hotkey));
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
        table.set_header(
            table_headers
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );
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

/// Require a mnemonic phrase: use `provided` if Some, else prompt interactively (or error in batch mode).
pub fn require_mnemonic(provided: Option<String>) -> Result<String> {
    match provided {
        Some(m) => Ok(m),
        None => {
            if is_batch_mode() {
                anyhow::bail!("Mnemonic required in batch mode. Pass --mnemonic <phrase>.");
            }
            dialoguer::Input::<String>::new()
                .with_prompt("Enter mnemonic phrase")
                .interact_text()
                .map_err(anyhow::Error::from)
        }
    }
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

/// Parse an optional JSON string into a vec of subxt dynamic Values.
pub fn parse_json_args(args: &Option<String>) -> anyhow::Result<Vec<subxt::dynamic::Value>> {
    if let Some(ref args_json) = args {
        let parsed: Vec<serde_json::Value> = serde_json::from_str(args_json).map_err(|e| {
            anyhow::anyhow!(
                "Invalid JSON args '{}'. Expected a JSON array, e.g. '[1, \"0x...\"]'",
                e
            )
        })?;
        Ok(parsed.iter().map(json_to_subxt_value).collect())
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
