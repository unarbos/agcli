//! SR25519 keypair utilities — generation, derivation, SS58 encoding.

use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use rand::Rng;
use sp_core::{crypto::Ss58Codec, sr25519, Pair};
use zeroize::Zeroize;

/// Generate a new mnemonic and derive the SR25519 keypair.
/// Returns (pair, mnemonic_phrase).
pub fn generate_mnemonic_keypair() -> Result<(sr25519::Pair, String)> {
    let mut entropy = [0u8; 16]; // 128 bits = 12 words
    rand::thread_rng().fill(&mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| anyhow::anyhow!("mnemonic generation failed: {:?}", e))?;
    entropy.zeroize(); // Wipe entropy — recovering it means recovering the mnemonic
    let phrase = mnemonic.to_string();
    let pair = pair_from_mnemonic(&phrase)?;
    Ok((pair, phrase))
}

/// Derive SR25519 keypair from a BIP-39 mnemonic phrase.
pub fn pair_from_mnemonic(mnemonic: &str) -> Result<sr25519::Pair> {
    sr25519::Pair::from_phrase(mnemonic, None)
        .map(|(pair, _seed)| pair)
        .map_err(|e| anyhow::anyhow!("Invalid mnemonic: {:?}", e))
}

/// Derive SR25519 keypair from a seed hex string (0x-prefixed or plain).
pub fn pair_from_seed_hex(seed: &str) -> Result<sr25519::Pair> {
    let seed = seed.strip_prefix("0x").unwrap_or(seed);
    let mut bytes = hex::decode(seed).context("Invalid hex seed")?;
    let mut seed_arr: [u8; 32] = bytes
        .clone()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Seed must be 32 bytes"))?;
    bytes.zeroize(); // Wipe decoded seed bytes
    let pair = sr25519::Pair::from_seed(&seed_arr);
    seed_arr.zeroize(); // Wipe seed array
    Ok(pair)
}

/// Encode an SR25519 public key to SS58 format.
pub fn to_ss58(public: &sr25519::Public, prefix: u16) -> String {
    public.to_ss58check_with_version(sp_core::crypto::Ss58AddressFormat::custom(prefix))
}

/// Derive SR25519 keypair from a Substrate dev URI (e.g. "//Alice", "//Bob").
/// Supports: Alice, Bob, Charlie, Dave, Eve, Ferdie, and arbitrary `//Name` URIs.
pub fn pair_from_uri(uri: &str) -> Result<sr25519::Pair> {
    sr25519::Pair::from_string(uri, None)
        .map_err(|e| anyhow::anyhow!("Invalid key URI '{}': {:?}", uri, e))
}

/// Well-known Substrate dev account names.
pub const DEV_ACCOUNTS: &[&str] = &["Alice", "Bob", "Charlie", "Dave", "Eve", "Ferdie"];

/// Decode an SS58 address to an SR25519 public key.
pub fn from_ss58(address: &str) -> Result<sr25519::Public> {
    sr25519::Public::from_ss58check(address)
        .map_err(|_| {
            let trimmed = address.trim();
            if trimmed.is_empty() {
                anyhow::anyhow!("Empty address. Provide a valid Bittensor SS58 address (starts with '5').")
            } else if trimmed.len() < 10 {
                anyhow::anyhow!("Invalid SS58 address '{}' — too short. Bittensor addresses are 48 characters starting with '5'.", trimmed)
            } else {
                anyhow::anyhow!("Invalid SS58 address '{}'. Expected a 48-character Bittensor address starting with '5'.\n  Tip: verify the address on taostats.io or use `agcli wallet show` to get your address.", crate::utils::short_ss58(trimmed))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_roundtrip() {
        let (pair, mnemonic) = generate_mnemonic_keypair().unwrap();
        let pair2 = pair_from_mnemonic(&mnemonic).unwrap();
        assert_eq!(pair.public(), pair2.public());
    }

    #[test]
    fn ss58_roundtrip() {
        let (pair, _) = generate_mnemonic_keypair().unwrap();
        let addr = to_ss58(&pair.public(), 42);
        let pub2 = from_ss58(&addr).unwrap();
        assert_eq!(pair.public(), pub2);
    }

    #[test]
    fn dev_accounts_derive() {
        // Alice's well-known SS58 address
        let alice = pair_from_uri("//Alice").unwrap();
        assert_eq!(
            to_ss58(&alice.public(), 42),
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
        );
        // Bob's well-known SS58 address
        let bob = pair_from_uri("//Bob").unwrap();
        assert_eq!(
            to_ss58(&bob.public(), 42),
            "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty"
        );
        // All dev accounts should derive without error
        for name in DEV_ACCOUNTS {
            let uri = format!("//{}", name);
            pair_from_uri(&uri).unwrap();
        }
    }

    // ──── Issues 663-664: Zeroization of entropy/seed ────

    #[test]
    fn generate_mnemonic_keypair_works_after_zeroize() {
        // Verify mnemonic generation still produces valid keypairs
        // after adding entropy zeroization
        let (pair1, mnemonic) = generate_mnemonic_keypair().unwrap();
        assert_eq!(mnemonic.split_whitespace().count(), 12, "Should produce 12-word mnemonic");
        // Re-derive from mnemonic should produce same key
        let pair2 = pair_from_mnemonic(&mnemonic).unwrap();
        assert_eq!(pair1.public(), pair2.public(), "Re-derived key should match");
    }

    #[test]
    fn pair_from_seed_hex_works_after_zeroize() {
        // Verify seed hex derivation still works after adding zeroization
        // Well-known: Alice's secret seed
        let alice_seed = "0xe5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";
        let pair = pair_from_seed_hex(alice_seed).unwrap();
        // Just verify it produces a valid key
        let ss58 = to_ss58(&pair.public(), 42);
        assert!(ss58.starts_with('5'), "SS58 should start with 5, got {}", ss58);
    }

    #[test]
    fn pair_from_seed_hex_without_prefix() {
        let seed = "e5be9a5092b81bca64be81d212e7f2f9eba183bb7a90954f7b76361f6edb5c0a";
        let pair = pair_from_seed_hex(seed).unwrap();
        let ss58 = to_ss58(&pair.public(), 42);
        assert!(ss58.starts_with('5'));
    }

    #[test]
    fn pair_from_seed_hex_invalid_length() {
        let result = pair_from_seed_hex("0x1234");
        assert!(result.is_err(), "Short seed should fail");
    }
}
