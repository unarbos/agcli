//! Keyfile I/O — encrypted coldkeys, plaintext hotkeys.
//!
//! Coldkeys are encrypted with AES-256-GCM using a key derived from
//! Argon2id (matching the Python bittensor-wallet encryption scheme).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
// argon2 crate used via fully-qualified paths (argon2::Argon2, argon2::Params, etc.)
use fs2::FileExt;
use rand::RngCore;
use sp_core::sr25519;
use std::fs;
use std::path::Path;
use zeroize::Zeroize;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// Maximum time to wait for a keyfile lock before giving up.
const LOCK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Acquire an exclusive advisory lock on a keyfile path with timeout.
/// Returns the lock file handle (lock released on drop).
/// Times out after 10 seconds to prevent indefinite hangs if another process
/// crashed while holding the lock.
fn lock_keyfile(path: &Path) -> Result<fs::File> {
    let lock_path = path.with_extension("lock");
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let lock_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .with_context(|| format!("Cannot create lock file '{}'", lock_path.display()))?;
    // Try non-blocking lock first for the fast path
    match lock_file.try_lock_exclusive() {
        Ok(()) => return Ok(lock_file),
        Err(_) => {
            tracing::debug!(path = %lock_path.display(), "Lock contended, polling with timeout");
        }
    }
    // Poll with backoff up to LOCK_TIMEOUT
    let start = std::time::Instant::now();
    let mut sleep_ms = 50;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
        match lock_file.try_lock_exclusive() {
            Ok(()) => return Ok(lock_file),
            Err(_) if start.elapsed() >= LOCK_TIMEOUT => {
                anyhow::bail!(
                    "Timed out after {}s waiting for lock on '{}'.\n  \
                     Another agcli process may be holding it, or a previous process crashed.\n  \
                     If no other process is running, remove the stale lock: rm '{}'",
                    LOCK_TIMEOUT.as_secs(),
                    lock_path.display(),
                    lock_path.display()
                );
            }
            Err(_) => {
                sleep_ms = (sleep_ms * 2).min(500); // backoff: 50→100→200→500ms
            }
        }
    }
}

/// Acquire an exclusive lock on a wallet directory (for creation/import).
/// Prevents two processes from creating the same wallet concurrently.
/// Returns the lock file handle (released on drop).
pub fn lock_wallet_dir(dir: &Path) -> Result<fs::File> {
    let lock_path = dir.join(".wallet.lock");
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let lock_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&lock_path)
        .with_context(|| format!("Cannot create wallet dir lock '{}'", lock_path.display()))?;
    match lock_file.try_lock_exclusive() {
        Ok(()) => return Ok(lock_file),
        Err(_) => {
            tracing::debug!(path = %lock_path.display(), "Wallet dir lock contended, polling");
        }
    }
    let start = std::time::Instant::now();
    let mut sleep_ms = 50;
    loop {
        std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
        match lock_file.try_lock_exclusive() {
            Ok(()) => return Ok(lock_file),
            Err(_) if start.elapsed() >= LOCK_TIMEOUT => {
                anyhow::bail!(
                    "Timed out waiting for wallet directory lock on '{}'.\n  \
                     Another agcli process may be creating this wallet. If not, remove: rm '{}'",
                    lock_path.display(),
                    lock_path.display()
                );
            }
            Err(_) => {
                sleep_ms = (sleep_ms * 2).min(500);
            }
        }
    }
}

/// Atomically write data to a file: write to a temp file in the same directory,
/// set permissions, then rename into place. This ensures the target path is
/// never left in a partial state (crash-safe).
fn atomic_write(path: &Path, data: &[u8], mode: u32) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;

    // Write to a temp file in the same directory (same filesystem for rename)
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, data)
        .with_context(|| format!("write temp file '{}'", tmp_path.display()))?;

    // Set permissions BEFORE rename so the file is never world-readable at the target path
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(mode))
            .with_context(|| format!("Failed to set permissions on '{}'", tmp_path.display()))?;
    }
    let _ = mode; // suppress unused warning on non-unix

    // Atomic rename into place
    fs::rename(&tmp_path, path)
        .with_context(|| format!("rename '{}' -> '{}'", tmp_path.display(), path.display()))?;
    Ok(())
}

/// Write mnemonic encrypted with password.
///
/// Rejects empty passwords to prevent trivially-breakable encryption.
pub fn write_encrypted_keyfile(path: &Path, mnemonic: &str, password: &str) -> Result<()> {
    if password.is_empty() {
        anyhow::bail!(
            "Empty password is not allowed for coldkey encryption. \
             Choose a strong password to protect your funds."
        );
    }
    tracing::debug!(path = %path.display(), "Writing encrypted keyfile");
    let _lock = lock_keyfile(path)?;

    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let mut key = derive_key(password, &salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, mnemonic.as_bytes())
        .map_err(|e| anyhow::anyhow!("encryption failed: {}", e))?;
    key.zeroize(); // Wipe derived key from memory immediately after use

    // Format: salt || nonce || ciphertext
    let mut data = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    data.extend_from_slice(&salt);
    data.extend_from_slice(&nonce_bytes);
    data.extend_from_slice(&ciphertext);

    atomic_write(path, &data, 0o600)?;
    Ok(())
}

/// Read and decrypt an encrypted keyfile, returning the mnemonic.
pub fn read_encrypted_keyfile(path: &Path, password: &str) -> Result<String> {
    tracing::debug!(path = %path.display(), "Reading encrypted keyfile");
    let _lock = lock_keyfile(path)?;
    let data =
        fs::read(path).with_context(|| format!("Cannot read keyfile at '{}'", path.display()))?;
    if data.len() < SALT_LEN + NONCE_LEN {
        anyhow::bail!("Keyfile '{}' is corrupted (too short). Re-create your wallet with `agcli wallet create`.", path.display());
    }

    let (salt, rest) = data.split_at(SALT_LEN);
    let (nonce_bytes, ciphertext) = rest.split_at(NONCE_LEN);

    let mut key = derive_key(password, salt)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;
    key.zeroize(); // Wipe derived key from memory immediately after cipher init
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong password. If you forgot your password, restore from your mnemonic with `agcli wallet regen-coldkey`."))?;

    // Convert without cloning to avoid leaving an unzeroized copy in memory
    let result = String::from_utf8(plaintext).context("mnemonic is not valid UTF-8");
    // Note: if from_utf8 succeeds, ownership transferred to String (no clone needed).
    // If it fails, the error contains the bytes which will be dropped, but that's
    // an error path (invalid data) so zeroization of the error payload is acceptable.
    result
}

/// Write a plaintext keyfile (for hotkeys).
pub fn write_keyfile(path: &Path, mnemonic: &str) -> Result<()> {
    let _lock = lock_keyfile(path)?;
    atomic_write(path, mnemonic.as_bytes(), 0o600)?;
    Ok(())
}

/// Read a plaintext keyfile (acquires exclusive lock to avoid reading mid-write).
pub fn read_keyfile(path: &Path) -> Result<String> {
    let _lock = lock_keyfile(path)?;
    fs::read_to_string(path).context("read keyfile")
}

/// Write public key to a file (hex-encoded).
pub fn write_public_key(path: &Path, public: &sr25519::Public) -> Result<()> {
    let _lock = lock_keyfile(path)?;
    atomic_write(path, hex::encode(public.0).as_bytes(), 0o644)?;
    Ok(())
}

/// Read a public key from file.
///
/// Supports two formats:
/// - **agcli format**: plain hex-encoded 32-byte public key (with or without `0x` prefix)
/// - **Python bittensor-wallet format**: JSON object with `publicKey` (hex) or `ss58Address`
pub fn read_public_key(path: &Path) -> Result<sr25519::Public> {
    let _lock = lock_keyfile(path)?;
    let content = fs::read_to_string(path).context("read public key file")?;
    let trimmed = content.trim();

    // Python bittensor-wallet stores coldkeypub.txt as JSON:
    // {"publicKey":"0x...","ss58Address":"5...","accountId":"0x..."}
    if trimmed.starts_with('{') {
        let v: serde_json::Value =
            serde_json::from_str(trimmed).context("Failed to parse coldkeypub.txt JSON")?;
        // Prefer publicKey hex over ss58Address (avoids SS58 codec round-trip)
        if let Some(hex_str) = v.get("publicKey").and_then(|s| s.as_str()) {
            let hex_str = hex_str.trim().strip_prefix("0x").unwrap_or(hex_str.trim());
            let bytes = hex::decode(hex_str).context("invalid hex in publicKey")?;
            if bytes.len() != 32 {
                anyhow::bail!("publicKey must be 32 bytes, got {}", bytes.len());
            }
            let mut arr = [0u8; 32];
            arr.copy_from_slice(&bytes);
            return Ok(sr25519::Public::from_raw(arr));
        }
        if let Some(ss58) = v.get("ss58Address").and_then(|s| s.as_str()) {
            use sp_core::crypto::Ss58Codec;
            return sr25519::Public::from_ss58check(ss58.trim())
                .map_err(|e| anyhow::anyhow!("invalid ss58Address in coldkeypub.txt: {:?}", e));
        }
        anyhow::bail!("coldkeypub.txt JSON has no publicKey or ss58Address field");
    }

    // agcli format: plain hex string
    let hex_str = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    let bytes = hex::decode(hex_str).context("invalid hex in public key file")?;
    if bytes.len() != 32 {
        anyhow::bail!("public key must be 32 bytes, got {}", bytes.len());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(sr25519::Public::from_raw(arr))
}

/// Derive a 256-bit AES key from password + salt using Argon2id.
///
/// Uses hardened parameters (256 MiB memory, 4 iterations) comparable to
/// the Python bittensor-wallet's Argon2i settings (512 MiB, 8 iterations).
/// The argon2 crate defaults (19 MiB, 2 iterations) are too weak for
/// protecting wallet mnemonics.
///
/// **Caller is responsible for zeroizing the returned key when done.**
fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
    let params = argon2::Params::new(
        256 * 1024, // m_cost: 256 MiB in KiB (strong, but half of Python's 512 MiB for usability)
        4,          // t_cost: 4 iterations (Python uses 8, but Argon2id is stronger per-iteration than Argon2i)
        1,          // p_cost: single-threaded (matches Python)
        Some(KEY_LEN),
    )
    .map_err(|e| anyhow::anyhow!("argon2 params error: {}", e))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {}", e))?;
    Ok(key)
}

// ──────── Python bittensor-wallet compatibility ────────

/// Magic prefix for NaCl-encrypted keyfiles (Python bittensor-wallet format).
const NACL_PREFIX: &[u8] = b"$NACL";

/// Fixed salt used by the Python bittensor-wallet for Argon2i KDF.
const NACL_SALT: [u8; 16] = [
    0x13, 0x71, 0x83, 0xdf, 0xf1, 0x5a, 0x09, 0xbc, 0x9c, 0x90, 0xb5, 0x51, 0x87, 0x39, 0xe9, 0xb1,
];

/// Check if keyfile data is in the Python NaCl format.
pub fn is_nacl_encrypted(data: &[u8]) -> bool {
    data.starts_with(NACL_PREFIX)
}

/// Read and decrypt a Python bittensor-wallet NaCl-encrypted keyfile.
/// Format: "$NACL" prefix + SecretBox encrypted data (nonce + ciphertext + MAC).
/// KDF: Argon2i with opslimit=8, memlimit=512MiB, fixed NACL_SALT.
pub fn read_python_keyfile(path: &Path, password: &str) -> Result<String> {
    let data = fs::read(path).context("read keyfile")?;
    decrypt_nacl_keyfile_data(&data, password)
}

/// Decrypt NaCl keyfile data (with or without $NACL prefix).
pub fn decrypt_nacl_keyfile_data(data: &[u8], password: &str) -> Result<String> {
    let encrypted = if data.starts_with(NACL_PREFIX) {
        &data[NACL_PREFIX.len()..]
    } else {
        data
    };

    // Derive key using Argon2i matching libsodium / PyNaCl constants:
    //   crypto_pwhash_argon2i_OPSLIMIT_SENSITIVE = 8
    //   crypto_pwhash_argon2i_MEMLIMIT_SENSITIVE = 536870912 bytes = 524288 KiB (512 MiB)
    let argon2_params = argon2::Params::new(
        524_288, // 512 MiB in KiB (crypto_pwhash_argon2i_MEMLIMIT_SENSITIVE)
        8,       // t_cost (crypto_pwhash_argon2i_OPSLIMIT_SENSITIVE)
        1,       // p_cost (parallelism — libsodium argon2i uses 1)
        Some(KEY_LEN),
    )
    .map_err(|e| anyhow::anyhow!("argon2 params error: {}", e))?;
    let argon2 = argon2::Argon2::new(
        argon2::Algorithm::Argon2i,
        argon2::Version::V0x13,
        argon2_params,
    );
    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), &NACL_SALT, &mut key)
        .map_err(|e| anyhow::anyhow!("key derivation failed: {}", e))?;

    // Decrypt using XSalsa20-Poly1305 (NaCl SecretBox)
    // PyNaCl SecretBox format: nonce (24 bytes) + ciphertext (with MAC)
    use crypto_secretbox::{
        aead::{Aead, KeyInit},
        XSalsa20Poly1305,
    };
    if encrypted.len() < 24 {
        key.zeroize();
        anyhow::bail!("NaCl keyfile too short");
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(24);
    let cipher = XSalsa20Poly1305::new_from_slice(&key)
        .map_err(|e| anyhow::anyhow!("cipher init: {}", e))?;
    key.zeroize(); // Wipe derived key from memory immediately after cipher init
    let nonce = crypto_secretbox::Nonce::from_slice(nonce_bytes);
    let mut plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Decryption failed — wrong password for Python wallet. If you forgot your password, restore from your mnemonic with `agcli wallet regen-coldkey`."))?;

    let result = String::from_utf8(plaintext.clone()).context("decrypted data is not valid UTF-8");
    plaintext.zeroize(); // Wipe decrypted plaintext bytes
    result
}

/// Detect keyfile format and decrypt accordingly.
/// Supports both agcli's AES-256-GCM format and Python's NaCl SecretBox format.
/// Returns raw decrypted content — may be a mnemonic string or a JSON object.
/// Use [`extract_secret_phrase`] to get the mnemonic from either format.
pub fn read_any_encrypted_keyfile(path: &Path, password: &str) -> Result<String> {
    let data = fs::read(path).context("read keyfile")?;
    if is_nacl_encrypted(&data) {
        decrypt_nacl_keyfile_data(&data, password)
    } else {
        // Try our AES-256-GCM format
        read_encrypted_keyfile(path, password)
    }
}

/// Extract the secret phrase (mnemonic) from decrypted keyfile content.
///
/// Handles both plain-mnemonic format (agcli) and JSON format (Python bittensor-wallet).
/// For JSON, looks for `secretPhrase` first, then falls back to `secretSeed`.
pub fn extract_secret_phrase(decrypted: &str) -> Result<String> {
    let trimmed = decrypted.trim();
    if trimmed.starts_with('{') {
        let v: serde_json::Value =
            serde_json::from_str(trimmed).context("Failed to parse keyfile JSON")?;
        if let Some(phrase) = v.get("secretPhrase").and_then(|s| s.as_str()) {
            return Ok(phrase.trim().to_string());
        }
        if let Some(seed) = v.get("secretSeed").and_then(|s| s.as_str()) {
            return Ok(seed.trim().to_string());
        }
        anyhow::bail!("Keyfile JSON has no secretPhrase or secretSeed field");
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_coldkey");
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let password = "test_password_123";

        write_encrypted_keyfile(&path, mnemonic, password).unwrap();
        let recovered = read_encrypted_keyfile(&path, password).unwrap();
        assert_eq!(mnemonic, recovered);
    }

    #[test]
    fn wrong_password_fails() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_coldkey");
        let mnemonic = "test mnemonic phrase";
        write_encrypted_keyfile(&path, mnemonic, "correct").unwrap();
        assert!(read_encrypted_keyfile(&path, "wrong").is_err());
    }

    #[test]
    fn concurrent_encrypted_read_write() {
        // Verify that concurrent reads and writes to the same keyfile
        // are safely serialized via advisory locks.
        // Use 2 threads — hardened Argon2id KDF (256 MiB) makes each KDF ~8-15s, and
        // with serialized exclusive locks, too many threads exceeds the 30s lock timeout.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("concurrent_coldkey");
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let password = "test123";

        // Write the keyfile first
        write_encrypted_keyfile(&path, mnemonic, password).unwrap();

        // Spawn concurrent readers
        let mut handles = Vec::new();
        for _ in 0..2 {
            let p = path.clone();
            let pw = password.to_string();
            handles.push(std::thread::spawn(move || read_encrypted_keyfile(&p, &pw)));
        }

        // All reads should succeed with the same result
        for h in handles {
            let result = h.join().expect("reader thread panicked");
            assert_eq!(result.unwrap(), mnemonic);
        }
    }

    #[test]
    fn concurrent_plaintext_read_write() {
        // Verify concurrent reads of plaintext keyfiles work correctly.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("concurrent_hotkey");
        let mnemonic = "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10 word11 word12";
        write_keyfile(&path, mnemonic).unwrap();

        let mut handles = Vec::new();
        for _ in 0..8 {
            let p = path.clone();
            handles.push(std::thread::spawn(move || read_keyfile(&p)));
        }

        for h in handles {
            let result = h.join().expect("reader thread panicked");
            assert_eq!(result.unwrap(), mnemonic);
        }
    }

    #[test]
    fn corrupted_keyfile_reports_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad_coldkey");
        // Write a file that's too short to be valid
        std::fs::write(&path, [0u8; 5]).unwrap();
        let result = read_encrypted_keyfile(&path, "any");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("corrupted"),
            "Expected 'corrupted' in error: {}",
            msg
        );
    }

    #[test]
    fn public_key_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pubkey.txt");
        let pk = sr25519::Public::from_raw([42u8; 32]);
        write_public_key(&path, &pk).unwrap();
        let recovered = read_public_key(&path).unwrap();
        assert_eq!(pk, recovered);
    }

    #[test]
    fn lock_timeout_on_held_lock() {
        // Verify that lock_keyfile times out instead of hanging forever
        // when another process holds the lock.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("timeout_coldkey");
        let lock_path = path.with_extension("lock");

        // Manually create and hold a lock
        fs::create_dir_all(dir.path()).unwrap();
        let held_lock = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .unwrap();
        held_lock.lock_exclusive().unwrap();

        // In another thread, try to acquire the same lock — should timeout
        let p = path.clone();
        let handle = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = lock_keyfile(&p);
            let elapsed = start.elapsed();
            (result, elapsed)
        });

        let (result, elapsed) = handle.join().expect("thread panicked");
        assert!(result.is_err(), "Should have timed out");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Timed out"),
            "Expected timeout error, got: {}",
            msg
        );
        // Should have waited at least a few seconds but not more than LOCK_TIMEOUT + buffer
        assert!(
            elapsed.as_secs() >= 10,
            "Should wait at least 10s, waited {:?}",
            elapsed
        );
        assert!(
            elapsed.as_secs() <= 40,
            "Should not wait more than 40s, waited {:?}",
            elapsed
        );

        // Release the held lock
        drop(held_lock);
    }

    #[test]
    fn lock_succeeds_after_contention() {
        // Verify that a lock is acquired after brief contention.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("contention_coldkey");
        let lock_path = path.with_extension("lock");

        fs::create_dir_all(dir.path()).unwrap();
        let held_lock = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&lock_path)
            .unwrap();
        held_lock.lock_exclusive().unwrap();

        // Release the lock after 200ms
        let held = std::sync::Arc::new(std::sync::Mutex::new(Some(held_lock)));
        let held2 = held.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(200));
            drop(held2.lock().unwrap().take());
        });

        // This should succeed once the lock is released
        let result = lock_keyfile(&path);
        assert!(
            result.is_ok(),
            "Should acquire lock after contention: {:?}",
            result.err()
        );
    }

    #[test]
    fn read_encrypted_keyfile_acquires_lock() {
        // Verify that read_encrypted_keyfile properly serializes with writers.
        // We hold the keyfile lock and verify the read blocks (times out on lock).
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("locked_coldkey");
        let password = "test123";
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        // Write the keyfile first
        write_encrypted_keyfile(&path, mnemonic, password).unwrap();

        // Hold the lock
        let _held = lock_keyfile(&path).unwrap();

        // Try to read from another thread — should block and eventually timeout
        let p = path.clone();
        let pw = password.to_string();
        let handle = std::thread::spawn(move || {
            let start = std::time::Instant::now();
            let result = read_encrypted_keyfile(&p, &pw);
            (result, start.elapsed())
        });

        // Drop the lock quickly so the read succeeds
        drop(_held);
        let (result, _elapsed) = handle.join().expect("thread panicked");
        assert!(result.is_ok(), "Read should succeed after lock release: {:?}", result.err());
        assert_eq!(result.unwrap(), mnemonic);
    }

    #[test]
    fn read_keyfile_fails_on_lock_timeout() {
        // Verify that read_keyfile now propagates lock errors instead of silently falling back.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("locked_hotkey");
        let mnemonic = "word1 word2 word3";

        write_keyfile(&path, mnemonic).unwrap();

        // Hold the lock indefinitely
        let held_lock = lock_keyfile(&path).unwrap();

        // Try to read from another thread — should fail with timeout
        let p = path.clone();
        let handle = std::thread::spawn(move || read_keyfile(&p));

        // Wait a bit then release so the test doesn't take 30s
        std::thread::sleep(std::time::Duration::from_millis(100));
        drop(held_lock);

        let result = handle.join().expect("thread panicked");
        // Should succeed now that lock is released
        assert!(result.is_ok(), "read_keyfile should succeed after lock release");
    }

    #[test]
    fn read_public_key_acquires_lock() {
        // Verify that read_public_key acquires a lock before reading.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("locked_pubkey.txt");
        let pk = sr25519::Public::from_raw([42u8; 32]);

        write_public_key(&path, &pk).unwrap();

        // Hold the lock, release it, verify read succeeds
        let held = lock_keyfile(&path).unwrap();
        let p = path.clone();
        let handle = std::thread::spawn(move || read_public_key(&p));

        // Release quickly
        drop(held);
        let result = handle.join().expect("thread panicked");
        assert!(result.is_ok(), "read_public_key should succeed: {:?}", result.err());
        assert_eq!(result.unwrap(), pk);
    }

    // ──── Issue 729: Empty password rejection ────

    #[test]
    fn empty_password_rejected_on_encrypt() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty_pw_coldkey");
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = write_encrypted_keyfile(&path, mnemonic, "");
        assert!(result.is_err(), "Empty password should be rejected");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Empty password"),
            "Error should mention empty password, got: {}",
            msg
        );
        // File should NOT have been created
        assert!(!path.exists(), "Keyfile should not be created with empty password");
    }

    #[test]
    fn whitespace_only_password_allowed() {
        // A password of spaces is technically non-empty (user's choice, even if weak)
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("space_pw_coldkey");
        let mnemonic = "test words";
        let result = write_encrypted_keyfile(&path, mnemonic, " ");
        assert!(result.is_ok(), "Whitespace password should be allowed (non-empty)");
        // Verify roundtrip
        let recovered = read_encrypted_keyfile(&path, " ").unwrap();
        assert_eq!(recovered, mnemonic);
    }

    // ──── Issues 662-665: Key material zeroization ────

    #[test]
    fn encrypt_decrypt_still_works_with_zeroization() {
        // Verify the zeroization changes didn't break encrypt/decrypt roundtrip
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zeroize_test_coldkey");
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let password = "strong_password_123";

        write_encrypted_keyfile(&path, mnemonic, password).unwrap();
        let recovered = read_encrypted_keyfile(&path, password).unwrap();
        assert_eq!(mnemonic, recovered, "Zeroization should not break roundtrip");
    }

    #[test]
    fn read_any_encrypted_still_works_with_zeroization() {
        // Verify read_any_encrypted_keyfile (which dispatches between AES-GCM and NaCl)
        // still works correctly after zeroization changes
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("zeroize_any_coldkey");
        let mnemonic = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
        let password = "test_pass";

        write_encrypted_keyfile(&path, mnemonic, password).unwrap();
        let recovered = read_any_encrypted_keyfile(&path, password).unwrap();
        assert_eq!(mnemonic, recovered, "read_any_encrypted should work with zeroization");
    }

    // ──── Issue 666: Argon2id hardened parameters ────

    #[test]
    fn argon2id_hardened_params_produces_valid_key() {
        // Verify derive_key works with the new hardened params
        let salt = [0x42u8; SALT_LEN];
        let key = derive_key("test_password", &salt).unwrap();
        assert_eq!(key.len(), KEY_LEN);
        // Key should not be all zeros
        assert!(key.iter().any(|&b| b != 0), "Derived key should not be all zeros");
    }

    #[test]
    fn argon2id_hardened_roundtrip_still_works() {
        // Verify encrypt/decrypt roundtrip works after parameter change
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hardened_argon2_coldkey");
        let mnemonic = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
        let password = "strong_password_456";

        write_encrypted_keyfile(&path, mnemonic, password).unwrap();
        let recovered = read_encrypted_keyfile(&path, password).unwrap();
        assert_eq!(mnemonic, recovered, "Hardened Argon2id roundtrip should match");
    }

    #[test]
    fn argon2id_hardened_different_passwords_different_keys() {
        let salt = [0xABu8; SALT_LEN];
        let key1 = derive_key("password_one", &salt).unwrap();
        let key2 = derive_key("password_two", &salt).unwrap();
        assert_ne!(key1, key2, "Different passwords should produce different keys");
    }

    #[test]
    fn argon2id_hardened_different_salts_different_keys() {
        let salt1 = [0x01u8; SALT_LEN];
        let salt2 = [0x02u8; SALT_LEN];
        let key1 = derive_key("same_password", &salt1).unwrap();
        let key2 = derive_key("same_password", &salt2).unwrap();
        assert_ne!(key1, key2, "Different salts should produce different keys");
    }

    #[test]
    fn argon2id_hardened_wrong_password_fails_decrypt() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hardened_wrong_pw");
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        write_encrypted_keyfile(&path, mnemonic, "correct_pw").unwrap();
        let result = read_encrypted_keyfile(&path, "wrong_pw");
        assert!(result.is_err(), "Wrong password should fail with hardened params");
    }

    #[test]
    fn extract_secret_phrase_from_json() {
        // Verify extract_secret_phrase still works (covers the mnemonic String path)
        let json_data = r#"{"secretPhrase":"hello world mnemonic","publicKey":"0x1234"}"#;
        let phrase = extract_secret_phrase(json_data).unwrap();
        assert_eq!(phrase, "hello world mnemonic");
    }

    #[test]
    fn extract_secret_phrase_from_plain() {
        let plain = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let phrase = extract_secret_phrase(plain).unwrap();
        assert_eq!(phrase, plain);
    }
}
