//! Core types used across the SDK, mirroring subtensor chain types.

pub mod balance;
pub mod chain_data;
pub mod network;

pub use balance::Balance;
pub use network::{NetUid, Network};

#[cfg(test)]
mod tests {
    // Verify that re-exports are accessible from the types module root.

    #[test]
    fn balance_reexported() {
        let b = super::Balance::from_rao(1_000_000_000);
        assert_eq!(b.rao(), 1_000_000_000);
    }

    #[test]
    fn balance_reexport_matches_canonical() {
        // Ensure the re-export and the canonical path resolve to the same type.
        let via_reexport = super::Balance::from_tao(2.0);
        let via_canonical = super::balance::Balance::from_tao(2.0);
        assert_eq!(via_reexport, via_canonical);
    }

    #[test]
    fn netuid_reexported() {
        let uid = super::NetUid(42);
        assert_eq!(uid.as_u16(), 42);
    }

    #[test]
    fn netuid_reexport_root_constant() {
        assert_eq!(super::NetUid::ROOT.as_u16(), 0);
    }

    #[test]
    fn network_reexported() {
        let net = super::Network::default();
        assert_eq!(format!("{}", net), "finney");
    }

    #[test]
    fn network_reexport_ws_url() {
        let net = super::Network::Finney;
        assert!(!net.ws_url().is_empty());
    }

    #[test]
    fn submodules_accessible() {
        // chain_data is pub, so types from it should be reachable.
        let uid = super::chain_data::SubnetInfo {
            netuid: super::NetUid(1),
            name: "test".to_string(),
            symbol: "T".to_string(),
            n: 10,
            max_n: 100,
            tempo: 360,
            emission_value: 0,
            burn: super::Balance::ZERO,
            difficulty: 0,
            immunity_period: 0,
            owner: "5Own".to_string(),
            registration_allowed: true,
        };
        assert_eq!(uid.netuid, super::NetUid(1));
    }
}
