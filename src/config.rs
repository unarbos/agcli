//! Persistent configuration file (~/.agcli/config.toml).
//!
//! Stores user preferences: default network, wallet, hotkey, endpoint, output format.
//! CLI flags override config file values.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Configuration loaded from ~/.agcli/config.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Default network (finney, test, local, or custom URL).
    pub network: Option<String>,
    /// Custom chain endpoint (overrides network).
    pub endpoint: Option<String>,
    /// Wallet directory.
    pub wallet_dir: Option<String>,
    /// Default wallet name.
    pub wallet: Option<String>,
    /// Default hotkey name.
    pub hotkey: Option<String>,
    /// Default output format (table, json, csv).
    pub output: Option<String>,
    /// Proxy account SS58 (if set, wraps all extrinsics in Proxy.proxy).
    pub proxy: Option<String>,
    /// Default live polling interval in seconds.
    pub live_interval: Option<u64>,
    /// Batch mode default (never prompt for input).
    pub batch: Option<bool>,
    /// Per-subnet spending limits in TAO (key = netuid as string).
    pub spending_limits: Option<std::collections::HashMap<String, f64>>,
    /// Finalization timeout in seconds (default: 30).
    pub finalization_timeout: Option<u64>,
    /// Extrinsic mortality in blocks (0 = default, ~64 blocks).
    pub mortality_blocks: Option<u64>,
}

impl Config {
    /// Default config file path.
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".agcli")
            .join("config.toml")
    }

    /// Load config from the default path. Returns default if file doesn't exist.
    pub fn load() -> Self {
        Self::load_from(&Self::default_path()).unwrap_or_default()
    }

    /// Load config from a specific path.
    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to the default path.
    pub fn save(&self) -> Result<()> {
        self.save_to(&Self::default_path())
    }

    /// Save config to a specific path.
    /// Uses atomic write (temp file + rename) to prevent corruption from concurrent writers.
    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        // Write to a uniquely-named temp file, then atomic rename.
        // Unique name prevents races when multiple processes save concurrently.
        let pid = std::process::id();
        let tid = std::thread::current().id();
        let tmp_name = format!(".config.{}.{:?}.tmp", pid, tid,);
        let tmp_path = path.with_file_name(tmp_name);
        std::fs::write(&tmp_path, &content)?;
        // Restrict permissions before atomic rename (config may contain wallet paths/names)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600)) {
                let _ = std::fs::remove_file(&tmp_path);
                return Err(e.into());
            }
        }
        std::fs::rename(&tmp_path, path).map_err(|e| {
            // Clean up temp file on rename failure
            let _ = std::fs::remove_file(&tmp_path);
            anyhow::anyhow!("Failed to atomically save config: {}", e)
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let cfg = Config {
            network: Some("finney".to_string()),
            wallet: Some("mywallet".to_string()),
            hotkey: Some("default".to_string()),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed.network.as_deref(), Some("finney"));
        assert_eq!(parsed.wallet.as_deref(), Some("mywallet"));
    }

    #[test]
    fn missing_file_returns_default() {
        let cfg = Config::load_from(Path::new("/nonexistent/path/config.toml")).unwrap();
        assert!(cfg.network.is_none());
    }

    /// Multiple concurrent writers to the same config file should all succeed.
    #[test]
    fn concurrent_config_writes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut handles = Vec::new();
        for i in 0..10u32 {
            let p = path.clone();
            handles.push(std::thread::spawn(move || {
                let cfg = Config {
                    network: Some(format!("net-{}", i)),
                    wallet: Some(format!("wallet-{}", i)),
                    ..Default::default()
                };
                cfg.save_to(&p)
            }));
        }

        let mut errors = 0;
        for h in handles {
            if h.join().unwrap().is_err() {
                errors += 1;
            }
        }
        // All writes should succeed (even if they overwrite each other)
        assert_eq!(errors, 0);

        // File should be parseable TOML afterward
        let final_cfg = Config::load_from(&path).unwrap();
        assert!(final_cfg.network.is_some());
    }

    /// Config with all fields populated roundtrips correctly.
    #[test]
    fn full_config_roundtrip() {
        let mut limits = std::collections::HashMap::new();
        limits.insert("1".to_string(), 100.0);
        limits.insert("18".to_string(), 50.0);

        let cfg = Config {
            network: Some("finney".into()),
            endpoint: Some("wss://custom:443".into()),
            wallet_dir: Some("/home/user/.bt".into()),
            wallet: Some("mywallet".into()),
            hotkey: Some("hk1".into()),
            output: Some("json".into()),
            proxy: Some("5GrwvaEF...".into()),
            live_interval: Some(30),
            batch: Some(true),
            spending_limits: Some(limits),
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed.endpoint.as_deref(), Some("wss://custom:443"));
        assert_eq!(parsed.live_interval, Some(30));
        assert_eq!(parsed.batch, Some(true));
        assert!(parsed.spending_limits.as_ref().unwrap().contains_key("18"));
    }

    /// Atomic write: no temp file left behind after successful save.
    #[test]
    fn atomic_write_no_temp_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let tmp_path = dir.path().join("config.toml.tmp");

        let cfg = Config {
            network: Some("finney".into()),
            ..Default::default()
        };
        cfg.save_to(&path).unwrap();

        assert!(path.exists(), "Config file should exist");
        assert!(
            !tmp_path.exists(),
            "Temp file should be cleaned up after rename"
        );

        // Verify the saved file is valid TOML
        let loaded = Config::load_from(&path).unwrap();
        assert_eq!(loaded.network.as_deref(), Some("finney"));
    }

    // ──── Issue 112: Config file permissions should be 0o600 ────

    #[cfg(unix)]
    #[test]
    fn config_file_has_restricted_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let cfg = Config {
            network: Some("finney".into()),
            wallet: Some("mywallet".into()),
            ..Default::default()
        };
        cfg.save_to(&path).unwrap();

        let meta = std::fs::metadata(&path).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "Config file should be owner-only (0o600), got: {:o}",
            mode
        );
    }

    /// Atomic write: concurrent writers produce a valid config (not corrupted).
    #[test]
    fn atomic_concurrent_writes_no_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let mut handles = Vec::new();
        for i in 0..20u32 {
            let p = path.clone();
            handles.push(std::thread::spawn(move || {
                let cfg = Config {
                    network: Some(format!("net-{}", i)),
                    wallet: Some(format!("wallet-{}", i)),
                    ..Default::default()
                };
                cfg.save_to(&p)
            }));
        }

        for h in handles {
            h.join().unwrap().unwrap();
        }

        // The final file must be valid parseable TOML (no partial writes)
        let final_cfg = Config::load_from(&path).unwrap();
        assert!(final_cfg.network.is_some());
        assert!(final_cfg.wallet.is_some());
    }

    // --- Issue 152: config save_to temp file cleanup on permission failure ---

    #[test]
    fn save_to_no_temp_file_left_on_success() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_config.toml");
        let mut cfg = Config::default();
        cfg.network = Some("finney".to_string());
        cfg.save_to(&path).unwrap();
        // Check no temp files remain
        let entries: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_str().map_or(false, |n| n.contains(".tmp")))
            .collect();
        assert!(entries.is_empty(), "temp file should not remain after successful save");
    }
}
