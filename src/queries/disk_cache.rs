//! General-purpose disk cache for expensive chain queries.
//!
//! Stores JSON-serialized results in `~/.agcli/cache/<key>.json` with a TTL.
//! Atomic writes (temp file + rename) prevent corruption under concurrent access.
//! Stale entries are served when fresh fetches fail (stale-while-error).

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Cached entry with metadata.
#[derive(Serialize, serde::Deserialize)]
struct CacheEntry<T> {
    /// Unix timestamp (seconds) when this entry was written.
    written_at: u64,
    /// The cached data.
    data: T,
}

/// Base directory for the general cache.
fn cache_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agcli")
        .join("cache")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Read a cached value if it exists and hasn't expired.
/// Returns `Some(data)` if cache hit, `None` if miss or expired.
pub fn get<T: DeserializeOwned>(key: &str, ttl: Duration) -> Option<T> {
    let path = cache_dir().join(format!("{}.json", key));
    let data = match std::fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return None,
    };
    let entry: CacheEntry<T> = match serde_json::from_str(&data) {
        Ok(e) => e,
        Err(e) => {
            tracing::debug!(key, error = %e, "disk cache: failed to parse entry, treating as miss");
            return None;
        }
    };
    let age = now_secs().saturating_sub(entry.written_at);
    if ttl.is_zero() || age > ttl.as_secs() {
        tracing::debug!(key, age_secs = age, ttl_secs = ttl.as_secs(), "disk cache: expired");
        return None;
    }
    tracing::debug!(key, age_secs = age, "disk cache: hit");
    Some(entry.data)
}

/// Write a value to the disk cache. Uses atomic temp-file + rename.
pub fn put<T: Serialize>(key: &str, data: &T) -> Result<()> {
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create cache dir: {}", dir.display()))?;

    let entry = CacheEntry {
        written_at: now_secs(),
        data,
    };
    let json = serde_json::to_string(&entry)
        .context("Failed to serialize cache entry")?;

    // Atomic write: temp file in same dir, then rename
    let tmp = dir.join(format!(".{}-{}.tmp", key, std::process::id()));
    std::fs::write(&tmp, json.as_bytes())
        .with_context(|| format!("Failed to write cache temp file: {}", tmp.display()))?;

    let target = dir.join(format!("{}.json", key));
    std::fs::rename(&tmp, &target)
        .with_context(|| format!("Failed to rename cache file: {} -> {}", tmp.display(), target.display()))?;

    tracing::debug!(key, "disk cache: written");
    Ok(())
}

/// Read a stale cached value (ignores TTL). Used for stale-while-error fallback.
pub fn get_stale<T: DeserializeOwned>(key: &str) -> Option<T> {
    let path = cache_dir().join(format!("{}.json", key));
    let data = std::fs::read_to_string(&path).ok()?;
    let entry: CacheEntry<T> = serde_json::from_str(&data).ok()?;
    let age = now_secs().saturating_sub(entry.written_at);
    tracing::debug!(key, age_secs = age, "disk cache: serving stale entry");
    Some(entry.data)
}

/// Remove a cached entry.
pub fn remove(key: &str) {
    let path = cache_dir().join(format!("{}.json", key));
    let _ = std::fs::remove_file(&path);
}

/// List all cache keys (filenames without .json extension).
pub fn list_keys() -> Vec<String> {
    let dir = cache_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return vec![];
    };
    entries
        .filter_map(|e| {
            let e = e.ok()?;
            let name = e.file_name().to_string_lossy().to_string();
            name.strip_suffix(".json").map(|k| k.to_string())
        })
        .filter(|k| !k.starts_with('.'))
        .collect()
}

/// Get the cache directory path (for display/diagnostics).
pub fn path() -> PathBuf {
    cache_dir()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn put_get_roundtrip() {
        let key = "test_roundtrip_disk_cache";
        let data = vec![1u32, 2, 3, 4, 5];
        put(key, &data).unwrap();
        let loaded: Vec<u32> = get(key, Duration::from_secs(60)).unwrap();
        assert_eq!(loaded, data);
        remove(key);
    }

    #[test]
    fn expired_returns_none() {
        let key = "test_expired_disk_cache";
        put(key, &"hello").unwrap();
        // TTL of 0 means immediately expired
        let result: Option<String> = get(key, Duration::from_secs(0));
        assert!(result.is_none());
        remove(key);
    }

    #[test]
    fn stale_ignores_ttl() {
        let key = "test_stale_disk_cache";
        put(key, &42u64).unwrap();
        // Even with 0 TTL, stale read succeeds
        let result: Option<u64> = get_stale(key);
        assert_eq!(result, Some(42));
        remove(key);
    }

    #[test]
    fn missing_key_returns_none() {
        let result: Option<String> = get("nonexistent_test_key_12345", Duration::from_secs(60));
        assert!(result.is_none());
    }

    #[test]
    fn list_keys_includes_entry() {
        let key = "test_list_keys_disk_cache";
        put(key, &true).unwrap();
        let keys = list_keys();
        assert!(keys.contains(&key.to_string()));
        remove(key);
    }

    #[test]
    fn remove_nonexistent_is_noop() {
        // Should not panic or error
        remove("this_key_definitely_does_not_exist_98765");
    }

    #[test]
    fn overwrite_updates_value() {
        let key = "test_overwrite_disk_cache";
        put(key, &"first").unwrap();
        put(key, &"second").unwrap();
        let loaded: String = get(key, Duration::from_secs(60)).unwrap();
        assert_eq!(loaded, "second");
        remove(key);
    }

    #[test]
    fn concurrent_writes_no_corruption() {
        // Stress test: 20 threads writing to the same key simultaneously.
        // Atomic temp+rename ensures no partial reads.
        let key = "test_concurrent_writes_disk_cache";
        // Ensure the cache directory exists before spawning threads
        let _ = put(key, &0u32);
        let mut handles = Vec::new();
        for i in 0..20u32 {
            let k = key.to_string();
            handles.push(std::thread::spawn(move || {
                // Ignore errors — concurrent renames may race
                let _ = put(&k, &i);
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        // Should read a valid u32, not corrupted JSON
        let result: Option<u32> = get(key, Duration::from_secs(60));
        assert!(result.is_some());
        assert!(result.unwrap() <= 20);
        remove(key);
    }

    #[test]
    fn concurrent_read_write_no_panic() {
        // Stress test: some threads read while others write — no crashes.
        let key = "test_concurrent_rw_disk_cache";
        put(key, &0u64).unwrap();
        let mut handles = Vec::new();
        for i in 0..10u64 {
            let k = key.to_string();
            if i % 2 == 0 {
                handles.push(std::thread::spawn(move || {
                    let _ = put(&k, &i); // ignore race errors
                }));
            } else {
                handles.push(std::thread::spawn(move || {
                    let _: Option<u64> = get(&k, Duration::from_secs(60));
                }));
            }
        }
        for h in handles {
            h.join().unwrap();
        }
        remove(key);
    }

    #[test]
    fn large_value_roundtrip() {
        // Test with a reasonably large serialized value (~64KB)
        let key = "test_large_value_disk_cache";
        let data: Vec<u32> = (0..16000).collect();
        put(key, &data).unwrap();
        let loaded: Vec<u32> = get(key, Duration::from_secs(60)).unwrap();
        assert_eq!(loaded.len(), 16000);
        assert_eq!(loaded[15999], 15999);
        remove(key);
    }
}
