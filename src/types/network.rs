//! Network identifiers and connection presets.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Subnet UID (u16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NetUid(pub u16);

impl NetUid {
    /// Root network (netuid 0).
    pub const ROOT: Self = Self(0);

    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for NetUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u16> for NetUid {
    fn from(v: u16) -> Self {
        Self(v)
    }
}

/// Well-known Bittensor networks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Network {
    #[default]
    Finney,
    Test,
    Local,
    Archive,
    Custom(String),
}

impl Network {
    /// WebSocket endpoint URL (primary).
    pub fn ws_url(&self) -> &str {
        match self {
            Network::Finney => "wss://entrypoint-finney.opentensor.ai:443",
            Network::Test => "wss://test.finney.opentensor.ai:443",
            Network::Local => "ws://127.0.0.1:9944",
            Network::Archive => "wss://bittensor-finney.api.onfinality.io/public-ws",
            Network::Custom(url) => url,
        }
    }

    /// All endpoint URLs (primary + fallbacks) for connection retry.
    pub fn ws_urls(&self) -> Vec<&str> {
        match self {
            Network::Finney => vec![
                "wss://entrypoint-finney.opentensor.ai:443",
                "wss://bittensor-finney.api.onfinality.io/public-ws",
            ],
            Network::Test => vec!["wss://test.finney.opentensor.ai:443"],
            Network::Local => vec!["ws://127.0.0.1:9944"],
            Network::Archive => vec![
                "wss://bittensor-finney.api.onfinality.io/public-ws",
                "wss://entrypoint-finney.opentensor.ai:443",
            ],
            Network::Custom(url) => vec![url.as_str()],
        }
    }
}

impl fmt::Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Network::Finney => write!(f, "finney"),
            Network::Test => write!(f, "test"),
            Network::Local => write!(f, "local"),
            Network::Archive => write!(f, "archive"),
            Network::Custom(url) => write!(f, "custom({})", url),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── NetUid ──────────────────────────────────────────────────────

    #[test]
    fn netuid_construction() {
        let uid = NetUid(42);
        assert_eq!(uid.0, 42);
    }

    #[test]
    fn netuid_root_constant() {
        assert_eq!(NetUid::ROOT.as_u16(), 0);
        assert_eq!(NetUid::ROOT, NetUid(0));
    }

    #[test]
    fn netuid_as_u16() {
        let uid = NetUid(7);
        assert_eq!(uid.as_u16(), 7u16);
    }

    #[test]
    fn netuid_display() {
        let uid = NetUid(18);
        assert_eq!(format!("{}", uid), "18");
    }

    #[test]
    fn netuid_display_zero() {
        assert_eq!(format!("{}", NetUid(0)), "0");
    }

    #[test]
    fn netuid_display_max() {
        assert_eq!(format!("{}", NetUid(u16::MAX)), "65535");
    }

    #[test]
    fn netuid_from_u16() {
        let uid: NetUid = 99u16.into();
        assert_eq!(uid.as_u16(), 99);

        let uid2 = NetUid::from(0u16);
        assert_eq!(uid2, NetUid::ROOT);
    }

    #[test]
    fn netuid_eq() {
        assert_eq!(NetUid(1), NetUid(1));
        assert_ne!(NetUid(1), NetUid(2));
    }

    #[test]
    fn netuid_ord() {
        assert!(NetUid(0) < NetUid(1));
        assert!(NetUid(100) > NetUid(50));

        let mut uids = vec![NetUid(3), NetUid(1), NetUid(2)];
        uids.sort();
        assert_eq!(uids, vec![NetUid(1), NetUid(2), NetUid(3)]);
    }

    #[test]
    fn netuid_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(NetUid(1));
        set.insert(NetUid(1));
        set.insert(NetUid(2));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn netuid_clone_copy() {
        let a = NetUid(5);
        let b = a; // Copy
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn netuid_serialization_roundtrip() {
        let uid = NetUid(42);
        let json = serde_json::to_string(&uid).unwrap();
        let deserialized: NetUid = serde_json::from_str(&json).unwrap();
        assert_eq!(uid, deserialized);
    }

    #[test]
    fn netuid_serialization_zero() {
        let uid = NetUid::ROOT;
        let json = serde_json::to_string(&uid).unwrap();
        let deserialized: NetUid = serde_json::from_str(&json).unwrap();
        assert_eq!(uid, deserialized);
    }

    #[test]
    fn netuid_serialization_max() {
        let uid = NetUid(u16::MAX);
        let json = serde_json::to_string(&uid).unwrap();
        let deserialized: NetUid = serde_json::from_str(&json).unwrap();
        assert_eq!(uid, deserialized);
    }

    #[test]
    fn netuid_debug_format() {
        let uid = NetUid(10);
        let debug = format!("{:?}", uid);
        assert!(debug.contains("NetUid"));
        assert!(debug.contains("10"));
    }

    // ── Network ─────────────────────────────────────────────────────

    #[test]
    fn network_default_is_finney() {
        let net = Network::default();
        assert_eq!(format!("{}", net), "finney");
    }

    #[test]
    fn network_display_finney() {
        assert_eq!(format!("{}", Network::Finney), "finney");
    }

    #[test]
    fn network_display_test() {
        assert_eq!(format!("{}", Network::Test), "test");
    }

    #[test]
    fn network_display_local() {
        assert_eq!(format!("{}", Network::Local), "local");
    }

    #[test]
    fn network_display_archive() {
        assert_eq!(format!("{}", Network::Archive), "archive");
    }

    #[test]
    fn network_display_custom() {
        let net = Network::Custom("wss://my-node.example.com:9944".to_string());
        assert_eq!(format!("{}", net), "custom(wss://my-node.example.com:9944)");
    }

    #[test]
    fn network_ws_url_finney() {
        assert_eq!(
            Network::Finney.ws_url(),
            "wss://entrypoint-finney.opentensor.ai:443"
        );
    }

    #[test]
    fn network_ws_url_test() {
        assert_eq!(
            Network::Test.ws_url(),
            "wss://test.finney.opentensor.ai:443"
        );
    }

    #[test]
    fn network_ws_url_local() {
        assert_eq!(Network::Local.ws_url(), "ws://127.0.0.1:9944");
    }

    #[test]
    fn network_ws_url_archive() {
        assert_eq!(
            Network::Archive.ws_url(),
            "wss://bittensor-finney.api.onfinality.io/public-ws"
        );
    }

    #[test]
    fn network_ws_url_custom() {
        let url = "wss://custom-node.example.com:443";
        let net = Network::Custom(url.to_string());
        assert_eq!(net.ws_url(), url);
    }

    #[test]
    fn network_ws_urls_finney_has_fallback() {
        let urls = Network::Finney.ws_urls();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "wss://entrypoint-finney.opentensor.ai:443");
        assert_eq!(
            urls[1],
            "wss://bittensor-finney.api.onfinality.io/public-ws"
        );
    }

    #[test]
    fn network_ws_urls_test_single() {
        let urls = Network::Test.ws_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "wss://test.finney.opentensor.ai:443");
    }

    #[test]
    fn network_ws_urls_local_single() {
        let urls = Network::Local.ws_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], "ws://127.0.0.1:9944");
    }

    #[test]
    fn network_ws_urls_archive_has_fallback() {
        let urls = Network::Archive.ws_urls();
        assert_eq!(urls.len(), 2);
        // Archive primary is onfinality, fallback is entrypoint
        assert_eq!(
            urls[0],
            "wss://bittensor-finney.api.onfinality.io/public-ws"
        );
        assert_eq!(urls[1], "wss://entrypoint-finney.opentensor.ai:443");
    }

    #[test]
    fn network_ws_urls_custom_single() {
        let url = "wss://my-endpoint.io:9944";
        let net = Network::Custom(url.to_string());
        let urls = net.ws_urls();
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0], url);
    }

    #[test]
    fn network_ws_urls_first_matches_ws_url() {
        // For every variant, ws_urls()[0] should equal ws_url()
        let variants: Vec<Network> = vec![
            Network::Finney,
            Network::Test,
            Network::Local,
            Network::Archive,
            Network::Custom("wss://foo.bar".to_string()),
        ];
        for net in variants {
            assert_eq!(
                net.ws_urls()[0],
                net.ws_url(),
                "ws_urls()[0] != ws_url() for {}",
                net
            );
        }
    }

    #[test]
    fn network_serialization_roundtrip_finney() {
        let net = Network::Finney;
        let json = serde_json::to_string(&net).unwrap();
        let deserialized: Network = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{}", deserialized), "finney");
    }

    #[test]
    fn network_serialization_roundtrip_custom() {
        let net = Network::Custom("wss://special.node:443".to_string());
        let json = serde_json::to_string(&net).unwrap();
        let deserialized: Network = serde_json::from_str(&json).unwrap();
        assert_eq!(
            format!("{}", deserialized),
            "custom(wss://special.node:443)"
        );
    }

    #[test]
    fn network_custom_empty_url() {
        let net = Network::Custom(String::new());
        assert_eq!(net.ws_url(), "");
        assert_eq!(net.ws_urls(), vec![""]);
        assert_eq!(format!("{}", net), "custom()");
    }

    #[test]
    fn network_debug_format() {
        let net = Network::Finney;
        let debug = format!("{:?}", net);
        assert!(debug.contains("Finney"));
    }
}
