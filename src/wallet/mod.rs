//! Wallet management — create, open, import, encrypt/decrypt keypairs.
//!
//! Bittensor wallets consist of:
//! - **Coldkey**: Signing key for high-value operations (staking, transfers).
//!   Stored encrypted on disk.
//! - **Hotkey**: Signing key for low-value operations (weights, serving).
//!   Stored unencrypted for automated use.
//!
//! Keys are SR25519 keypairs (Substrate default).

pub mod keyfile;
pub mod keypair;

use anyhow::{Context, Result};
use sp_core::{sr25519, Pair as _};
use std::path::{Path, PathBuf};
use zeroize::Zeroize;

/// Reject hotkey names that contain path traversal sequences or path separators.
/// (audit fix: prevent path traversal via crafted hotkey names like "../../../etc/passwd")
fn validate_hotkey_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Hotkey name cannot be empty.");
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('\0') {
        anyhow::bail!(
            "Invalid hotkey name '{}': must not contain '/', '\\', '..', or null bytes.",
            name
        );
    }
    Ok(())
}

/// Reject wallet names that contain path traversal sequences or path separators.
/// (audit fix: library-level validation — CLI already validates via helpers::validate_name,
/// but direct library callers bypass CLI and need this check)
fn validate_wallet_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Wallet name cannot be empty.");
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.contains('\0') {
        anyhow::bail!(
            "Invalid wallet name '{}': must not contain '/', '\\', '..', or null bytes.",
            name
        );
    }
    Ok(())
}

/// A Bittensor wallet containing coldkey and hotkey.
pub struct Wallet {
    /// Display name.
    pub name: String,
    /// Path to the wallet directory.
    pub path: PathBuf,
    /// Decrypted coldkey (loaded lazily).
    coldkey: Option<sr25519::Pair>,
    /// Hotkey pair.
    hotkey: Option<sr25519::Pair>,
    /// Coldkey SS58 address (always available if public key is known).
    coldkey_ss58: Option<String>,
    /// Hotkey SS58 address.
    hotkey_ss58: Option<String>,
}

impl std::fmt::Debug for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wallet")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("coldkey_ss58", &self.coldkey_ss58)
            .field("hotkey_ss58", &self.hotkey_ss58)
            .finish()
    }
}

impl Wallet {
    /// Open an existing wallet from disk.
    ///
    /// ```rust,no_run
    /// let w = agcli::Wallet::open("~/.bittensor/wallets/default").unwrap();
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = expand_tilde(path.as_ref());
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "default".to_string());

        let coldkey_ss58 = match keyfile::read_public_key(&path.join("coldkeypub.txt")) {
            Ok(pk) => Some(keypair::to_ss58(&pk, 42)),
            Err(e) => {
                tracing::debug!(wallet = %name, error = %e, "Could not read coldkeypub.txt (not yet created or corrupted)");
                None
            }
        };
        // Try the hotkey keypair file first, then the pub-key-only file (Python btcli: defaultpub.txt)
        let hotkey_ss58 = read_hotkey_ss58(&path, "default");

        Ok(Self {
            name,
            path,
            coldkey: None,
            hotkey: None,
            coldkey_ss58,
            hotkey_ss58,
        })
    }

    /// Create a new wallet with fresh keys.
    /// Returns `(wallet, coldkey_mnemonic, hotkey_mnemonic)`.
    ///
    /// Uses an exclusive lock on the wallet directory to prevent two concurrent
    /// processes from creating the same wallet and overwriting each other's keys.
    pub fn create(
        wallet_dir: impl AsRef<Path>,
        name: &str,
        password: &str,
        hotkey_name: &str,
    ) -> Result<(Self, String, String)> {
        validate_wallet_name(name)?;
        validate_hotkey_name(hotkey_name)?;
        let dir = expand_tilde(wallet_dir.as_ref()).join(name);
        // Create directory structure first (idempotent, needed so lock file can be placed)
        std::fs::create_dir_all(dir.join("hotkeys"))?;

        // Acquire directory-level lock to prevent concurrent wallet creation
        let _dir_lock = keyfile::lock_wallet_dir(&dir)?;

        // Check if wallet already exists (under the lock — prevents TOCTOU race)
        if dir.join("coldkey").exists() {
            anyhow::bail!(
                "Wallet '{}' already exists at {}.\n  Use a different name or remove it first.",
                name,
                dir.display()
            );
        }

        let coldkey = keypair::generate_mnemonic_keypair()?;
        let hotkey = keypair::generate_mnemonic_keypair()?;

        let coldkey_ss58 = keypair::to_ss58(&coldkey.0.public(), 42);
        let hotkey_ss58 = keypair::to_ss58(&hotkey.0.public(), 42);

        // Save encrypted coldkey
        keyfile::write_encrypted_keyfile(
            &dir.join("coldkey"),
            &coldkey.1, // mnemonic
            password,
        )?;

        // Save coldkey public key
        keyfile::write_public_key(&dir.join("coldkeypub.txt"), &coldkey.0.public())?;

        // Save hotkey (unencrypted)
        keyfile::write_keyfile(&dir.join("hotkeys").join(hotkey_name), &hotkey.1)?;

        // Move mnemonics out of tuples to avoid leaving unzeroized copies on drop
        let coldkey_mnemonic = coldkey.1;
        let hotkey_mnemonic = hotkey.1;

        Ok((
            Self {
                name: name.to_string(),
                path: dir,
                coldkey: Some(coldkey.0),
                hotkey: Some(hotkey.0),
                coldkey_ss58: Some(coldkey_ss58),
                hotkey_ss58: Some(hotkey_ss58),
            },
            coldkey_mnemonic,
            hotkey_mnemonic,
        ))
    }

    /// Create wallet from a Substrate dev URI (e.g. "//Alice").
    ///
    /// The wallet name defaults to the lowercase account name (e.g. "alice").
    /// Both coldkey and hotkey are derived from the same URI.
    /// The URI is stored encrypted as the coldkey (re-derivable on unlock).
    pub fn create_from_uri(
        wallet_dir: impl AsRef<Path>,
        uri: &str,
        password: &str,
    ) -> Result<Self> {
        let pair = keypair::pair_from_uri(uri)?;
        let name = uri.trim_start_matches('/').to_lowercase();
        validate_wallet_name(&name)?;
        let dir = expand_tilde(wallet_dir.as_ref()).join(&name);
        std::fs::create_dir_all(dir.join("hotkeys"))?;
        let _dir_lock = keyfile::lock_wallet_dir(&dir)?;

        // Check if wallet already exists (under the lock — prevents TOCTOU race)
        if dir.join("coldkey").exists() {
            anyhow::bail!(
                "Wallet '{}' already exists at {}.\n  Use a different name or remove it first.",
                name,
                dir.display()
            );
        }

        let coldkey_ss58 = keypair::to_ss58(&pair.public(), 42);
        let hotkey_ss58 = coldkey_ss58.clone();

        // Store URI as encrypted coldkey content
        keyfile::write_encrypted_keyfile(&dir.join("coldkey"), uri, password)?;
        keyfile::write_public_key(&dir.join("coldkeypub.txt"), &pair.public())?;
        // Store URI as hotkey (unencrypted, same key for dev convenience)
        keyfile::write_keyfile(&dir.join("hotkeys").join("default"), uri)?;

        Ok(Self {
            name,
            path: dir,
            coldkey: Some(pair.clone()),
            hotkey: Some(pair),
            coldkey_ss58: Some(coldkey_ss58),
            hotkey_ss58: Some(hotkey_ss58),
        })
    }

    /// Import wallet from mnemonic.
    ///
    /// Uses a directory-level lock to prevent concurrent imports from corrupting files.
    pub fn import_from_mnemonic(
        wallet_dir: impl AsRef<Path>,
        name: &str,
        mnemonic: &str,
        password: &str,
    ) -> Result<Self> {
        validate_wallet_name(name)?;
        let dir = expand_tilde(wallet_dir.as_ref()).join(name);
        std::fs::create_dir_all(dir.join("hotkeys"))?;

        // Acquire directory-level lock to prevent concurrent imports
        let _dir_lock = keyfile::lock_wallet_dir(&dir)?;

        // Check if wallet already exists (under the lock — prevents TOCTOU race)
        if dir.join("coldkey").exists() {
            anyhow::bail!(
                "Wallet '{}' already exists at {}.\n  Use `--force` or remove it first.",
                name,
                dir.display()
            );
        }

        let pair = keypair::pair_from_mnemonic(mnemonic)?;
        let ss58 = keypair::to_ss58(&pair.public(), 42);

        keyfile::write_encrypted_keyfile(&dir.join("coldkey"), mnemonic, password)?;
        keyfile::write_public_key(&dir.join("coldkeypub.txt"), &pair.public())?;

        Ok(Self {
            name: name.to_string(),
            path: dir,
            coldkey: Some(pair),
            hotkey: None,
            coldkey_ss58: Some(ss58),
            hotkey_ss58: None,
        })
    }

    /// Unlock the coldkey with a password.
    /// Auto-detects keyfile format (agcli AES-256-GCM or Python NaCl SecretBox).
    pub fn unlock_coldkey(&mut self, password: &str) -> Result<()> {
        let mut data = keyfile::read_any_encrypted_keyfile(&self.path.join("coldkey"), password)
            .context("Failed to decrypt coldkey")?;
        // The decrypted data may be a mnemonic or a JSON keypair (Python format)
        let result = (|| -> Result<sr25519::Pair> {
            if data.trim().starts_with('{') {
                // Python bittensor-wallet stores JSON: {"secretSeed": "0x...", ...} or
                // {"ss58Address": "...", "secretPhrase": "...", ...}
                let v: serde_json::Value =
                    serde_json::from_str(data.trim()).context("Failed to parse Python keyfile JSON")?;
                if let Some(seed) = v.get("secretSeed").and_then(|s| s.as_str()) {
                    keypair::pair_from_seed_hex(seed)
                } else if let Some(phrase) = v.get("secretPhrase").and_then(|s| s.as_str()) {
                    keypair::pair_from_mnemonic(phrase)
                } else {
                    anyhow::bail!("Python keyfile JSON has no secretSeed or secretPhrase");
                }
            } else if data.trim().starts_with("//") {
                // Dev URI (e.g. "//Alice") — stored by create_from_uri
                keypair::pair_from_uri(data.trim())
            } else {
                keypair::pair_from_mnemonic(data.trim())
            }
        })();
        data.zeroize(); // Wipe decrypted secret from memory
        let pair = result?;
        self.coldkey_ss58 = Some(keypair::to_ss58(&pair.public(), 42));
        self.coldkey = Some(pair);
        Ok(())
    }

    /// Load the hotkey (unencrypted).
    ///
    /// Resolution order:
    /// 1. `hotkeys/{name}` — full keypair (mnemonic, JSON with secretPhrase/secretSeed, or dev URI)
    /// 2. `hotkeys/{name}pub.txt` — public key only (Python btcli convention); sets SS58 but no pair
    pub fn load_hotkey(&mut self, hotkey_name: &str) -> Result<()> {
        validate_hotkey_name(hotkey_name)?;
        let keypair_path = self.path.join("hotkeys").join(hotkey_name);
        if keypair_path.exists() {
            let mut data = keyfile::read_keyfile(&keypair_path)?;
            let result = (|| -> Result<sr25519::Pair> {
                if data.trim().starts_with('{') {
                    let v: serde_json::Value =
                        serde_json::from_str(data.trim()).context("Failed to parse hotkey JSON")?;
                    if let Some(seed) = v.get("secretSeed").and_then(|s| s.as_str()) {
                        keypair::pair_from_seed_hex(seed)
                    } else if let Some(phrase) = v.get("secretPhrase").and_then(|s| s.as_str()) {
                        keypair::pair_from_mnemonic(phrase)
                    } else {
                        anyhow::bail!("Hotkey JSON has no secretSeed or secretPhrase");
                    }
                } else if data.trim().starts_with("//") {
                    keypair::pair_from_uri(data.trim())
                } else {
                    keypair::pair_from_mnemonic(data.trim())
                }
            })();
            data.zeroize(); // Wipe hotkey secret from memory
            let pair = result?;
            self.hotkey_ss58 = Some(keypair::to_ss58(&pair.public(), 42));
            self.hotkey = Some(pair);
            return Ok(());
        }

        // Fallback: pub-key-only file (Python btcli stores {name}pub.txt alongside hotkey)
        // Sufficient for operations that only need the hotkey address (staking, etc.)
        if let Some(ss58) = read_hotkey_ss58(&self.path, hotkey_name) {
            self.hotkey_ss58 = Some(ss58);
            return Ok(());
        }

        anyhow::bail!(
            "Hotkey '{}' not found in {}.\n  Available hotkeys: {}",
            hotkey_name,
            self.path.join("hotkeys").display(),
            {
                let keys = self.list_hotkeys().unwrap_or_default();
                if keys.is_empty() {
                    "(none)".to_string()
                } else {
                    keys.join(", ")
                }
            }
        )
    }

    /// Get the coldkey pair (must be unlocked).
    pub fn coldkey(&self) -> Result<&sr25519::Pair> {
        self.coldkey
            .as_ref()
            .context("Coldkey not unlocked. Call unlock_coldkey() first.")
    }

    /// Get the hotkey pair (must be loaded).
    pub fn hotkey(&self) -> Result<&sr25519::Pair> {
        self.hotkey
            .as_ref()
            .context("Hotkey not loaded. Call load_hotkey() first.")
    }

    /// Coldkey SS58 address.
    pub fn coldkey_ss58(&self) -> Option<&str> {
        self.coldkey_ss58.as_deref()
    }

    /// Hotkey SS58 address.
    pub fn hotkey_ss58(&self) -> Option<&str> {
        self.hotkey_ss58.as_deref()
    }

    /// Get the coldkey public key bytes.
    /// Returns an error if the coldkey has not been unlocked yet, preventing
    /// silent use of a zero/default public key.
    pub fn coldkey_public(&self) -> Result<sp_core::sr25519::Public> {
        self.coldkey
            .as_ref()
            .map(|p| p.public())
            .context("Coldkey not unlocked. Call unlock_coldkey() first.")
    }

    /// List all hotkeys in the wallet.
    ///
    /// Skips `.lock` and `*pub.txt` sidecar files. For Python btcli wallets that only have a
    /// `{name}pub.txt` file (no matching `{name}` keypair file), still includes the name so
    /// the hotkey address can be resolved via the pub file.
    pub fn list_hotkeys(&self) -> Result<Vec<String>> {
        let hotkey_dir = self.path.join("hotkeys");
        if !hotkey_dir.exists() {
            return Ok(vec![]);
        }
        let mut names = std::collections::HashSet::new();
        let mut pub_only: std::collections::HashSet<String> = std::collections::HashSet::new();
        for entry in std::fs::read_dir(&hotkey_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let fname = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            // Skip lock files and hidden files
            if fname.ends_with(".lock") || fname.starts_with('.') {
                continue;
            }
            // Pub sidecar: "{name}pub.txt" — record base name, don't add directly
            if fname.ends_with("pub.txt") {
                let base = fname.trim_end_matches("pub.txt").to_string();
                if !base.is_empty() {
                    pub_only.insert(base);
                }
                continue;
            }
            names.insert(fname);
        }
        // Add names from pub-only sidecars where no keypair file exists
        for base in pub_only {
            if !names.contains(&base) {
                names.insert(base);
            }
        }
        let mut names: Vec<String> = names.into_iter().collect();
        names.sort();
        Ok(names)
    }

    /// List all wallets in a directory.
    pub fn list_wallets(wallet_dir: impl AsRef<Path>) -> Result<Vec<String>> {
        let dir = expand_tilde(wallet_dir.as_ref());
        let mut names = Vec::new();
        if !dir.exists() {
            return Ok(names);
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    names.push(name.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }
}

/// Try to read a hotkey SS58 address from a wallet directory.
///
/// Checks in order:
/// 1. `hotkeys/{name}` — may be a full JSON keypair (extract publicKey/ss58Address) or mnemonic
/// 2. `hotkeys/{name}pub.txt` — Python btcli pub-key sidecar file
fn read_hotkey_ss58(wallet_path: &Path, hotkey_name: &str) -> Option<String> {
    if validate_hotkey_name(hotkey_name).is_err() {
        return None;
    }
    let hotkey_dir = wallet_path.join("hotkeys");

    // Try main keypair file — if it's a Python JSON, extract ss58Address directly
    let keypair_path = hotkey_dir.join(hotkey_name);
    if keypair_path.exists() {
        if let Ok(pk) = keyfile::read_public_key(&keypair_path) {
            return Some(keypair::to_ss58(&pk, 42));
        }
    }

    // Try Python btcli pub sidecar: hotkeys/{name}pub.txt
    let pub_path = hotkey_dir.join(format!("{}pub.txt", hotkey_name));
    if pub_path.exists() {
        if let Ok(pk) = keyfile::read_public_key(&pub_path) {
            return Some(keypair::to_ss58(&pk, 42));
        }
    }

    None
}

pub(crate) fn expand_tilde(path: &Path) -> PathBuf {
    if let Ok(stripped) = path.strip_prefix("~") {
        if let Some(home) = dirs::home_dir() {
            return home.join(stripped);
        }
    }
    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_uri_rejects_existing_wallet() {
        let dir = tempfile::tempdir().unwrap();
        // First creation should succeed
        let w = Wallet::create_from_uri(dir.path(), "//Alice", "pass123");
        assert!(w.is_ok(), "First create_from_uri should succeed: {:?}", w.err());

        // Second creation of the same wallet should fail
        let w2 = Wallet::create_from_uri(dir.path(), "//Alice", "pass456");
        assert!(w2.is_err(), "create_from_uri should reject existing wallet");
        let msg = w2.unwrap_err().to_string();
        assert!(
            msg.contains("already exists"),
            "Error should mention 'already exists', got: {}",
            msg
        );
    }

    #[test]
    fn import_from_mnemonic_rejects_existing_wallet() {
        let dir = tempfile::tempdir().unwrap();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        // First import should succeed
        let w = Wallet::import_from_mnemonic(dir.path(), "test_wallet", mnemonic, "pass123");
        assert!(w.is_ok(), "First import should succeed: {:?}", w.err());

        // Second import of the same wallet name should fail
        let w2 = Wallet::import_from_mnemonic(dir.path(), "test_wallet", mnemonic, "pass456");
        assert!(w2.is_err(), "import_from_mnemonic should reject existing wallet");
        let msg = w2.unwrap_err().to_string();
        assert!(
            msg.contains("already exists"),
            "Error should mention 'already exists', got: {}",
            msg
        );
    }

    #[test]
    fn create_rejects_existing_wallet() {
        // Verify the existing protection in create() still works
        let dir = tempfile::tempdir().unwrap();
        let w1 = Wallet::create(dir.path(), "mywallet", "pass123", "default");
        assert!(w1.is_ok(), "First create should succeed: {:?}", w1.err());

        let w2 = Wallet::create(dir.path(), "mywallet", "pass456", "default");
        assert!(w2.is_err(), "create should reject existing wallet");
    }

    #[test]
    fn create_from_uri_different_names_ok() {
        let dir = tempfile::tempdir().unwrap();
        let w1 = Wallet::create_from_uri(dir.path(), "//Alice", "pass");
        assert!(w1.is_ok());
        let w2 = Wallet::create_from_uri(dir.path(), "//Bob", "pass");
        assert!(w2.is_ok(), "Different URIs should create separate wallets");
    }

    // ── Issue 667: coldkey_public() must not silently return zero key ──

    #[test]
    fn coldkey_public_errors_when_not_unlocked() {
        let dir = tempfile::tempdir().unwrap();
        let (wallet, _, _) = Wallet::create(dir.path(), "test667", "pass", "default").unwrap();
        // Re-open without unlocking
        let w = Wallet::open(dir.path().join("test667")).unwrap();
        // coldkey_public() should error, not return a zero key
        assert!(
            w.coldkey_public().is_err(),
            "coldkey_public() should error when coldkey is not unlocked"
        );
    }

    #[test]
    fn coldkey_public_succeeds_when_unlocked() {
        let dir = tempfile::tempdir().unwrap();
        let (wallet, _, _) = Wallet::create(dir.path(), "test667b", "pass", "default").unwrap();
        // The wallet returned from create() has coldkey loaded
        let pk = wallet.coldkey_public();
        assert!(pk.is_ok(), "coldkey_public() should succeed when unlocked");
        // The public key should not be all zeros
        assert_ne!(
            pk.unwrap().0,
            [0u8; 32],
            "coldkey_public() should return a real key, not zeros"
        );
    }

    #[test]
    fn coldkey_public_succeeds_after_explicit_unlock() {
        let dir = tempfile::tempdir().unwrap();
        let (_wallet, _, _) = Wallet::create(dir.path(), "test667c", "mypass", "default").unwrap();
        let mut w = Wallet::open(dir.path().join("test667c")).unwrap();
        assert!(w.coldkey_public().is_err(), "should fail before unlock");
        w.unlock_coldkey("mypass").unwrap();
        assert!(w.coldkey_public().is_ok(), "should succeed after unlock");
    }

    // ── Issue 72-74: expand_tilde must resolve ~ in wallet paths ──

    #[test]
    fn expand_tilde_resolves_home_directory() {
        let expanded = expand_tilde(std::path::Path::new("~/.bittensor/wallets"));
        assert!(
            !expanded.starts_with("~"),
            "expand_tilde should resolve ~ to home dir, got: {:?}",
            expanded
        );
        assert!(
            expanded.to_string_lossy().contains(".bittensor/wallets"),
            "should preserve path after ~"
        );
    }

    #[test]
    fn expand_tilde_preserves_absolute_path() {
        let path = std::path::Path::new("/tmp/wallets");
        let expanded = expand_tilde(path);
        assert_eq!(expanded, path.to_path_buf(), "absolute paths should be unchanged");
    }

    #[test]
    fn expand_tilde_preserves_relative_path() {
        let path = std::path::Path::new("wallets/default");
        let expanded = expand_tilde(path);
        assert_eq!(expanded, path.to_path_buf(), "relative paths without ~ should be unchanged");
    }

    // ── Audit fix: hotkey name path traversal prevention ──

    #[test]
    fn validate_hotkey_name_rejects_path_traversal() {
        assert!(validate_hotkey_name("../../../etc/passwd").is_err());
        assert!(validate_hotkey_name("..").is_err());
        assert!(validate_hotkey_name("foo/../bar").is_err());
    }

    #[test]
    fn validate_hotkey_name_rejects_slashes() {
        assert!(validate_hotkey_name("foo/bar").is_err());
        assert!(validate_hotkey_name("foo\\bar").is_err());
    }

    #[test]
    fn validate_hotkey_name_rejects_empty() {
        assert!(validate_hotkey_name("").is_err());
    }

    #[test]
    fn validate_hotkey_name_rejects_null_bytes() {
        assert!(validate_hotkey_name("foo\0bar").is_err());
    }

    #[test]
    fn validate_hotkey_name_accepts_valid_names() {
        assert!(validate_hotkey_name("default").is_ok());
        assert!(validate_hotkey_name("my-hotkey").is_ok());
        assert!(validate_hotkey_name("hotkey_01").is_ok());
        assert!(validate_hotkey_name("HOTKEY").is_ok());
    }

    #[test]
    fn load_hotkey_rejects_traversal_name() {
        let dir = tempfile::tempdir().unwrap();
        let (mut wallet, _, _) = Wallet::create(dir.path(), "test", "pass", "default").unwrap();
        let result = wallet.load_hotkey("../../../etc/passwd");
        assert!(result.is_err(), "load_hotkey must reject path traversal");
        assert!(
            format!("{}", result.unwrap_err()).contains("must not contain"),
            "error should mention invalid characters"
        );
    }

    #[test]
    fn create_wallet_rejects_traversal_hotkey_name() {
        let dir = tempfile::tempdir().unwrap();
        let result = Wallet::create(dir.path(), "test", "pass", "../escape");
        assert!(result.is_err(), "create must reject path traversal in hotkey name");
    }

    // ── Issue 94: Wallet name path traversal at library level ──

    #[test]
    fn validate_wallet_name_rejects_traversal() {
        assert!(validate_wallet_name("../escape").is_err());
        assert!(validate_wallet_name("..").is_err());
        assert!(validate_wallet_name("foo/bar").is_err());
        assert!(validate_wallet_name("foo\\bar").is_err());
        assert!(validate_wallet_name("foo\0bar").is_err());
        assert!(validate_wallet_name("").is_err());
    }

    #[test]
    fn validate_wallet_name_accepts_valid() {
        assert!(validate_wallet_name("default").is_ok());
        assert!(validate_wallet_name("my-wallet").is_ok());
        assert!(validate_wallet_name("wallet_01").is_ok());
        assert!(validate_wallet_name("MyWallet").is_ok());
    }

    #[test]
    fn create_rejects_traversal_wallet_name() {
        let dir = tempfile::tempdir().unwrap();
        let result = Wallet::create(dir.path(), "../escape", "pass", "default");
        assert!(result.is_err(), "create must reject path traversal in wallet name");
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Invalid wallet name"), "error should mention invalid wallet name, got: {}", msg);
    }

    #[test]
    fn import_rejects_traversal_wallet_name() {
        let dir = tempfile::tempdir().unwrap();
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = Wallet::import_from_mnemonic(dir.path(), "../../etc/shadow", mnemonic, "pass");
        assert!(result.is_err(), "import must reject path traversal in wallet name");
    }

    // ── Issue 95/96: Decrypted key data zeroized after use ──

    #[test]
    fn unlock_coldkey_succeeds_and_key_is_usable() {
        // Verifies that the zeroize refactor doesn't break unlock functionality
        let dir = tempfile::tempdir().unwrap();
        let (_, _, _) = Wallet::create(dir.path(), "ztest", "mypass", "default").unwrap();
        let mut w = Wallet::open(dir.path().join("ztest")).unwrap();
        assert!(w.unlock_coldkey("mypass").is_ok(), "unlock_coldkey should still work after zeroize refactor");
        assert!(w.coldkey().is_ok(), "coldkey pair should be available after unlock");
    }

    #[test]
    fn load_hotkey_succeeds_after_zeroize_refactor() {
        // Verifies that the zeroize refactor doesn't break hotkey loading
        let dir = tempfile::tempdir().unwrap();
        let (_, _, _) = Wallet::create(dir.path(), "ztest2", "mypass", "default").unwrap();
        let mut w = Wallet::open(dir.path().join("ztest2")).unwrap();
        assert!(w.load_hotkey("default").is_ok(), "load_hotkey should still work after zeroize refactor");
        assert!(w.hotkey().is_ok(), "hotkey pair should be available after load");
    }
}
