//! Chain data structures decoded from subtensor storage.

use crate::types::balance::{AlphaBalance, Balance};
use crate::types::network::NetUid;
use serde::{Deserialize, Serialize};

/// Neuron (miner/validator) information on a subnet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfo {
    pub hotkey: String,
    pub coldkey: String,
    pub uid: u16,
    pub netuid: NetUid,
    pub active: bool,
    pub stake: Balance,
    pub rank: f64,
    pub emission: f64,
    pub incentive: f64,
    pub consensus: f64,
    pub trust: f64,
    pub validator_trust: f64,
    pub dividends: f64,
    pub last_update: u64,
    pub validator_permit: bool,
    pub pruning_score: f64,
    pub axon_info: Option<AxonInfo>,
    pub prometheus_info: Option<PrometheusInfo>,
}

/// Lightweight neuron info (no axon/prometheus).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronInfoLite {
    pub hotkey: String,
    pub coldkey: String,
    pub uid: u16,
    pub netuid: NetUid,
    pub active: bool,
    pub stake: Balance,
    pub rank: f64,
    pub emission: f64,
    pub incentive: f64,
    pub consensus: f64,
    pub trust: f64,
    pub validator_trust: f64,
    pub dividends: f64,
    pub last_update: u64,
    pub validator_permit: bool,
    pub pruning_score: f64,
}

/// Axon (miner endpoint) metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxonInfo {
    pub block: u64,
    pub version: u32,
    pub ip: String,
    pub port: u16,
    pub ip_type: u8,
    pub protocol: u8,
}

/// Prometheus endpoint metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusInfo {
    pub block: u64,
    pub version: u32,
    pub ip: String,
    pub port: u16,
    pub ip_type: u8,
}

/// Subnet information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetInfo {
    pub netuid: NetUid,
    pub name: String,
    pub symbol: String,
    pub n: u16,
    pub max_n: u16,
    pub tempo: u16,
    pub emission_value: u64,
    pub burn: Balance,
    pub difficulty: u64,
    pub immunity_period: u16,
    pub owner: String,
    pub registration_allowed: bool,
}

/// Dynamic subnet information (Dynamic TAO).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicInfo {
    pub netuid: NetUid,
    pub name: String,
    pub symbol: String,
    pub tempo: u16,
    pub emission: u64,
    pub tao_in: Balance,
    pub alpha_in: AlphaBalance,
    pub alpha_out: AlphaBalance,
    pub price: f64,
    pub owner_hotkey: String,
    pub owner_coldkey: String,
    pub last_step: u64,
    pub blocks_since_last_step: u64,
    pub alpha_out_emission: u64,
    pub alpha_in_emission: u64,
    pub tao_in_emission: u64,
    pub pending_alpha_emission: u64,
    pub pending_root_emission: u64,
    pub subnet_volume: u128,
    pub network_registered_at: u64,
}

impl DynamicInfo {
    /// Compute total emission per tempo from component fields.
    /// The `emission` field on-chain is deprecated/zero; use the sum of
    /// alpha_out_emission + alpha_in_emission + tao_in_emission instead.
    pub fn total_emission(&self) -> u64 {
        self.alpha_out_emission
            .saturating_add(self.alpha_in_emission)
            .saturating_add(self.tao_in_emission)
    }
}

/// Subnet hyperparameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetHyperparameters {
    pub netuid: NetUid,
    pub rho: u16,
    pub kappa: u16,
    pub immunity_period: u16,
    pub min_allowed_weights: u16,
    pub max_weights_limit: u16,
    pub tempo: u16,
    pub min_difficulty: u64,
    pub max_difficulty: u64,
    pub weights_version: u64,
    pub weights_rate_limit: u64,
    pub adjustment_interval: u16,
    pub activity_cutoff: u16,
    pub registration_allowed: bool,
    pub target_regs_per_interval: u16,
    pub min_burn: Balance,
    pub max_burn: Balance,
    pub bonds_moving_avg: u64,
    pub max_regs_per_block: u16,
    pub serving_rate_limit: u64,
    pub max_validators: u16,
    pub adjustment_alpha: u64,
    pub difficulty: u64,
    pub commit_reveal_weights_enabled: bool,
    pub commit_reveal_weights_interval: u64,
    pub liquid_alpha_enabled: bool,
}

/// Delegate information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateInfo {
    pub hotkey: String,
    pub owner: String,
    pub take: f64,
    pub total_stake: Balance,
    pub nominators: Vec<(String, Balance)>,
    pub registrations: Vec<NetUid>,
    pub validator_permits: Vec<NetUid>,
    pub return_per_1000: Balance,
}

/// Stake information for a coldkey-hotkey-subnet triple.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub hotkey: String,
    pub coldkey: String,
    pub netuid: NetUid,
    pub stake: Balance,
    pub alpha_stake: AlphaBalance,
}

/// On-chain identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainIdentity {
    pub name: String,
    pub url: String,
    pub github: String,
    pub image: String,
    pub discord: String,
    pub description: String,
    pub additional: String,
}

/// Subnet identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetIdentity {
    pub subnet_name: String,
    pub github_repo: String,
    pub subnet_contact: String,
    pub subnet_url: String,
    pub discord: String,
    pub description: String,
    pub additional: String,
}

/// Metagraph: full snapshot of a subnet's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metagraph {
    pub netuid: NetUid,
    pub n: u16,
    pub block: u64,
    pub neurons: Vec<NeuronInfoLite>,
    pub stake: Vec<Balance>,
    pub ranks: Vec<f64>,
    pub trust: Vec<f64>,
    pub consensus: Vec<f64>,
    pub incentive: Vec<f64>,
    pub dividends: Vec<f64>,
    pub emission: Vec<f64>,
    pub validator_trust: Vec<f64>,
    pub validator_permit: Vec<bool>,
    pub uids: Vec<u16>,
    pub active: Vec<bool>,
    pub last_update: Vec<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helper constructors ────────────────────────────────────────

    fn make_axon_info() -> AxonInfo {
        AxonInfo {
            block: 100,
            version: 1,
            ip: "192.168.1.1".to_string(),
            port: 8091,
            ip_type: 4,
            protocol: 1,
        }
    }

    fn make_prometheus_info() -> PrometheusInfo {
        PrometheusInfo {
            block: 100,
            version: 1,
            ip: "192.168.1.1".to_string(),
            port: 9090,
            ip_type: 4,
        }
    }

    fn make_neuron_info() -> NeuronInfo {
        NeuronInfo {
            hotkey: "5Hot".to_string(),
            coldkey: "5Cold".to_string(),
            uid: 0,
            netuid: NetUid(1),
            active: true,
            stake: Balance::from_tao(100.0),
            rank: 0.5,
            emission: 0.01,
            incentive: 0.3,
            consensus: 0.4,
            trust: 0.8,
            validator_trust: 0.9,
            dividends: 0.05,
            last_update: 1000,
            validator_permit: true,
            pruning_score: 0.1,
            axon_info: Some(make_axon_info()),
            prometheus_info: Some(make_prometheus_info()),
        }
    }

    fn make_neuron_info_lite() -> NeuronInfoLite {
        NeuronInfoLite {
            hotkey: "5Hot".to_string(),
            coldkey: "5Cold".to_string(),
            uid: 0,
            netuid: NetUid(1),
            active: true,
            stake: Balance::from_tao(100.0),
            rank: 0.5,
            emission: 0.01,
            incentive: 0.3,
            consensus: 0.4,
            trust: 0.8,
            validator_trust: 0.9,
            dividends: 0.05,
            last_update: 1000,
            validator_permit: true,
            pruning_score: 0.1,
        }
    }

    fn make_subnet_info() -> SubnetInfo {
        SubnetInfo {
            netuid: NetUid(1),
            name: "alpha".to_string(),
            symbol: "A".to_string(),
            n: 256,
            max_n: 4096,
            tempo: 360,
            emission_value: 1_000_000,
            burn: Balance::from_tao(1.0),
            difficulty: 10_000_000,
            immunity_period: 7200,
            owner: "5Owner".to_string(),
            registration_allowed: true,
        }
    }

    fn make_dynamic_info() -> DynamicInfo {
        DynamicInfo {
            netuid: NetUid(1),
            name: "alpha".to_string(),
            symbol: "A".to_string(),
            tempo: 360,
            emission: 0,
            tao_in: Balance::from_tao(1000.0),
            alpha_in: AlphaBalance::from_raw(500_000_000_000),
            alpha_out: AlphaBalance::from_raw(300_000_000_000),
            price: 1.5,
            owner_hotkey: "5HotOwner".to_string(),
            owner_coldkey: "5ColdOwner".to_string(),
            last_step: 500,
            blocks_since_last_step: 10,
            alpha_out_emission: 100_000,
            alpha_in_emission: 200_000,
            tao_in_emission: 50_000,
            pending_alpha_emission: 10_000,
            pending_root_emission: 5_000,
            subnet_volume: 999_999,
            network_registered_at: 42,
        }
    }

    fn make_metagraph() -> Metagraph {
        let neuron = make_neuron_info_lite();
        Metagraph {
            netuid: NetUid(1),
            n: 1,
            block: 5000,
            neurons: vec![neuron],
            stake: vec![Balance::from_tao(100.0)],
            ranks: vec![0.5],
            trust: vec![0.8],
            consensus: vec![0.4],
            incentive: vec![0.3],
            dividends: vec![0.05],
            emission: vec![0.01],
            validator_trust: vec![0.9],
            validator_permit: vec![true],
            uids: vec![0],
            active: vec![true],
            last_update: vec![1000],
        }
    }

    // ── DynamicInfo::total_emission() ──────────────────────────────

    #[test]
    fn total_emission_sums_components() {
        let info = make_dynamic_info();
        // 100_000 + 200_000 + 50_000 = 350_000
        assert_eq!(info.total_emission(), 350_000);
    }

    #[test]
    fn total_emission_all_zero() {
        let mut info = make_dynamic_info();
        info.alpha_out_emission = 0;
        info.alpha_in_emission = 0;
        info.tao_in_emission = 0;
        assert_eq!(info.total_emission(), 0);
    }

    #[test]
    fn total_emission_one_component_only() {
        let mut info = make_dynamic_info();
        info.alpha_out_emission = 0;
        info.alpha_in_emission = 0;
        info.tao_in_emission = 777;
        assert_eq!(info.total_emission(), 777);
    }

    #[test]
    fn total_emission_large_values() {
        let mut info = make_dynamic_info();
        info.alpha_out_emission = u64::MAX / 3;
        info.alpha_in_emission = u64::MAX / 3;
        info.tao_in_emission = u64::MAX / 3;
        // Should not overflow; result ~ u64::MAX - 2 (integer division rounding)
        let result = info.total_emission();
        assert!(result > u64::MAX / 2);
        assert!(result <= u64::MAX);
    }

    #[test]
    fn total_emission_saturates_on_overflow() {
        let mut info = make_dynamic_info();
        info.alpha_out_emission = u64::MAX;
        info.alpha_in_emission = 1;
        info.tao_in_emission = 1;
        // saturating_add should cap at u64::MAX
        assert_eq!(info.total_emission(), u64::MAX);
    }

    #[test]
    fn total_emission_saturates_two_max() {
        let mut info = make_dynamic_info();
        info.alpha_out_emission = u64::MAX;
        info.alpha_in_emission = u64::MAX;
        info.tao_in_emission = u64::MAX;
        assert_eq!(info.total_emission(), u64::MAX);
    }

    #[test]
    fn total_emission_saturates_at_boundary() {
        let mut info = make_dynamic_info();
        info.alpha_out_emission = u64::MAX - 1;
        info.alpha_in_emission = 1;
        info.tao_in_emission = 0;
        // Exactly u64::MAX, no saturation needed
        assert_eq!(info.total_emission(), u64::MAX);

        // Now push it past
        info.tao_in_emission = 1;
        assert_eq!(info.total_emission(), u64::MAX); // saturated
    }

    // ── NeuronInfo construction & serialization ────────────────────

    #[test]
    fn neuron_info_construction() {
        let neuron = make_neuron_info();
        assert_eq!(neuron.hotkey, "5Hot");
        assert_eq!(neuron.coldkey, "5Cold");
        assert_eq!(neuron.uid, 0);
        assert_eq!(neuron.netuid, NetUid(1));
        assert!(neuron.active);
        assert!(neuron.validator_permit);
        assert!(neuron.axon_info.is_some());
        assert!(neuron.prometheus_info.is_some());
    }

    #[test]
    fn neuron_info_without_optional_fields() {
        let mut neuron = make_neuron_info();
        neuron.axon_info = None;
        neuron.prometheus_info = None;
        assert!(neuron.axon_info.is_none());
        assert!(neuron.prometheus_info.is_none());
    }

    #[test]
    fn neuron_info_serialization_roundtrip() {
        let neuron = make_neuron_info();
        let json = serde_json::to_string(&neuron).unwrap();
        let deserialized: NeuronInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.hotkey, neuron.hotkey);
        assert_eq!(deserialized.uid, neuron.uid);
        assert_eq!(deserialized.netuid, neuron.netuid);
        assert_eq!(deserialized.stake, neuron.stake);
        assert_eq!(deserialized.active, neuron.active);
        assert!((deserialized.rank - neuron.rank).abs() < 1e-12);
    }

    #[test]
    fn neuron_info_serialization_null_optionals() {
        let mut neuron = make_neuron_info();
        neuron.axon_info = None;
        neuron.prometheus_info = None;
        let json = serde_json::to_string(&neuron).unwrap();
        let deserialized: NeuronInfo = serde_json::from_str(&json).unwrap();
        assert!(deserialized.axon_info.is_none());
        assert!(deserialized.prometheus_info.is_none());
    }

    // ── NeuronInfoLite construction & serialization ────────────────

    #[test]
    fn neuron_info_lite_construction() {
        let neuron = make_neuron_info_lite();
        assert_eq!(neuron.hotkey, "5Hot");
        assert_eq!(neuron.uid, 0);
        assert_eq!(neuron.netuid, NetUid(1));
        assert!(neuron.active);
    }

    #[test]
    fn neuron_info_lite_serialization_roundtrip() {
        let neuron = make_neuron_info_lite();
        let json = serde_json::to_string(&neuron).unwrap();
        let deserialized: NeuronInfoLite = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.hotkey, neuron.hotkey);
        assert_eq!(deserialized.coldkey, neuron.coldkey);
        assert_eq!(deserialized.uid, neuron.uid);
        assert_eq!(deserialized.netuid, neuron.netuid);
        assert_eq!(deserialized.stake, neuron.stake);
        assert!((deserialized.rank - neuron.rank).abs() < 1e-12);
        assert!((deserialized.emission - neuron.emission).abs() < 1e-12);
        assert_eq!(deserialized.validator_permit, neuron.validator_permit);
    }

    // ── SubnetInfo construction & serialization ────────────────────

    #[test]
    fn subnet_info_construction() {
        let info = make_subnet_info();
        assert_eq!(info.netuid, NetUid(1));
        assert_eq!(info.name, "alpha");
        assert_eq!(info.n, 256);
        assert_eq!(info.max_n, 4096);
        assert!(info.registration_allowed);
    }

    #[test]
    fn subnet_info_serialization_roundtrip() {
        let info = make_subnet_info();
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: SubnetInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.netuid, info.netuid);
        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.symbol, info.symbol);
        assert_eq!(deserialized.n, info.n);
        assert_eq!(deserialized.max_n, info.max_n);
        assert_eq!(deserialized.tempo, info.tempo);
        assert_eq!(deserialized.emission_value, info.emission_value);
        assert_eq!(deserialized.burn, info.burn);
        assert_eq!(deserialized.difficulty, info.difficulty);
        assert_eq!(deserialized.immunity_period, info.immunity_period);
        assert_eq!(deserialized.owner, info.owner);
        assert_eq!(deserialized.registration_allowed, info.registration_allowed);
    }

    // ── DynamicInfo construction & serialization ───────────────────

    #[test]
    fn dynamic_info_construction() {
        let info = make_dynamic_info();
        assert_eq!(info.netuid, NetUid(1));
        assert_eq!(info.name, "alpha");
        assert_eq!(info.tempo, 360);
        assert!((info.price - 1.5).abs() < 1e-12);
        assert_eq!(info.network_registered_at, 42);
    }

    #[test]
    fn dynamic_info_serialization_roundtrip() {
        let info = make_dynamic_info();
        let json = serde_json::to_string(&info).unwrap();
        let deserialized: DynamicInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.netuid, info.netuid);
        assert_eq!(deserialized.name, info.name);
        assert_eq!(deserialized.symbol, info.symbol);
        assert_eq!(deserialized.tempo, info.tempo);
        assert_eq!(deserialized.tao_in, info.tao_in);
        assert_eq!(deserialized.alpha_in, info.alpha_in);
        assert_eq!(deserialized.alpha_out, info.alpha_out);
        assert!((deserialized.price - info.price).abs() < 1e-12);
        assert_eq!(deserialized.alpha_out_emission, info.alpha_out_emission);
        assert_eq!(deserialized.alpha_in_emission, info.alpha_in_emission);
        assert_eq!(deserialized.tao_in_emission, info.tao_in_emission);
        assert_eq!(deserialized.subnet_volume, info.subnet_volume);
        assert_eq!(deserialized.total_emission(), info.total_emission());
    }

    // ── AxonInfo & PrometheusInfo ──────────────────────────────────

    #[test]
    fn axon_info_serialization_roundtrip() {
        let axon = make_axon_info();
        let json = serde_json::to_string(&axon).unwrap();
        let deserialized: AxonInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.block, axon.block);
        assert_eq!(deserialized.ip, axon.ip);
        assert_eq!(deserialized.port, axon.port);
    }

    #[test]
    fn prometheus_info_serialization_roundtrip() {
        let prom = make_prometheus_info();
        let json = serde_json::to_string(&prom).unwrap();
        let deserialized: PrometheusInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.block, prom.block);
        assert_eq!(deserialized.ip, prom.ip);
        assert_eq!(deserialized.port, prom.port);
    }

    // ── Metagraph construction ─────────────────────────────────────

    #[test]
    fn metagraph_construction() {
        let mg = make_metagraph();
        assert_eq!(mg.netuid, NetUid(1));
        assert_eq!(mg.n, 1);
        assert_eq!(mg.block, 5000);
        assert_eq!(mg.neurons.len(), 1);
        assert_eq!(mg.stake.len(), 1);
        assert_eq!(mg.ranks.len(), 1);
        assert_eq!(mg.uids.len(), 1);
        assert_eq!(mg.active.len(), 1);
    }

    #[test]
    fn metagraph_empty() {
        let mg = Metagraph {
            netuid: NetUid(0),
            n: 0,
            block: 0,
            neurons: vec![],
            stake: vec![],
            ranks: vec![],
            trust: vec![],
            consensus: vec![],
            incentive: vec![],
            dividends: vec![],
            emission: vec![],
            validator_trust: vec![],
            validator_permit: vec![],
            uids: vec![],
            active: vec![],
            last_update: vec![],
        };
        assert_eq!(mg.n, 0);
        assert!(mg.neurons.is_empty());
    }

    #[test]
    fn metagraph_serialization_roundtrip() {
        let mg = make_metagraph();
        let json = serde_json::to_string(&mg).unwrap();
        let deserialized: Metagraph = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.netuid, mg.netuid);
        assert_eq!(deserialized.n, mg.n);
        assert_eq!(deserialized.block, mg.block);
        assert_eq!(deserialized.neurons.len(), mg.neurons.len());
        assert_eq!(deserialized.stake.len(), mg.stake.len());
        assert_eq!(deserialized.uids, mg.uids);
        assert_eq!(deserialized.active, mg.active);
        assert_eq!(deserialized.validator_permit, mg.validator_permit);
    }

    #[test]
    fn metagraph_neuron_data_preserved() {
        let mg = make_metagraph();
        let neuron = &mg.neurons[0];
        assert_eq!(neuron.hotkey, "5Hot");
        assert_eq!(neuron.netuid, NetUid(1));
        assert_eq!(mg.stake[0], Balance::from_tao(100.0));
        assert!((mg.ranks[0] - 0.5).abs() < 1e-12);
    }

    // ── DelegateInfo, StakeInfo, ChainIdentity, SubnetIdentity ────

    #[test]
    fn delegate_info_serialization_roundtrip() {
        let info = DelegateInfo {
            hotkey: "5Del".to_string(),
            owner: "5Own".to_string(),
            take: 0.18,
            total_stake: Balance::from_tao(50_000.0),
            nominators: vec![
                ("5Nom1".to_string(), Balance::from_tao(10_000.0)),
                ("5Nom2".to_string(), Balance::from_tao(5_000.0)),
            ],
            registrations: vec![NetUid(1), NetUid(3)],
            validator_permits: vec![NetUid(1)],
            return_per_1000: Balance::from_tao(0.5),
        };
        let json = serde_json::to_string(&info).unwrap();
        let d: DelegateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(d.hotkey, "5Del");
        assert_eq!(d.nominators.len(), 2);
        assert_eq!(d.registrations.len(), 2);
    }

    #[test]
    fn stake_info_serialization_roundtrip() {
        let info = StakeInfo {
            hotkey: "5Hot".to_string(),
            coldkey: "5Cold".to_string(),
            netuid: NetUid(7),
            stake: Balance::from_tao(200.0),
            alpha_stake: AlphaBalance::from_raw(100_000_000_000),
        };
        let json = serde_json::to_string(&info).unwrap();
        let d: StakeInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(d.netuid, NetUid(7));
        assert_eq!(d.stake, Balance::from_tao(200.0));
        assert_eq!(d.alpha_stake, AlphaBalance::from_raw(100_000_000_000));
    }

    #[test]
    fn chain_identity_serialization_roundtrip() {
        let id = ChainIdentity {
            name: "MyValidator".to_string(),
            url: "https://example.com".to_string(),
            github: "user/repo".to_string(),
            image: "ipfs://Qm...".to_string(),
            discord: "user#1234".to_string(),
            description: "A validator node".to_string(),
            additional: "".to_string(),
        };
        let json = serde_json::to_string(&id).unwrap();
        let d: ChainIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(d.name, "MyValidator");
        assert_eq!(d.url, "https://example.com");
    }

    #[test]
    fn subnet_identity_serialization_roundtrip() {
        let id = SubnetIdentity {
            subnet_name: "TestSubnet".to_string(),
            github_repo: "org/subnet-repo".to_string(),
            subnet_contact: "contact@example.com".to_string(),
            subnet_url: "https://subnet.example.com".to_string(),
            discord: "discord.gg/test".to_string(),
            description: "A test subnet".to_string(),
            additional: "extra info".to_string(),
        };
        let json = serde_json::to_string(&id).unwrap();
        let d: SubnetIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(d.subnet_name, "TestSubnet");
        assert_eq!(d.github_repo, "org/subnet-repo");
    }

    // ── SubnetHyperparameters ──────────────────────────────────────

    #[test]
    fn subnet_hyperparameters_serialization_roundtrip() {
        let hp = SubnetHyperparameters {
            netuid: NetUid(1),
            rho: 10,
            kappa: 32767,
            immunity_period: 7200,
            min_allowed_weights: 1024,
            max_weights_limit: 455,
            tempo: 360,
            min_difficulty: 10_000_000,
            max_difficulty: 100_000_000,
            weights_version: 1,
            weights_rate_limit: 100,
            adjustment_interval: 112,
            activity_cutoff: 5000,
            registration_allowed: true,
            target_regs_per_interval: 2,
            min_burn: Balance::from_tao(0.1),
            max_burn: Balance::from_tao(100.0),
            bonds_moving_avg: 900_000,
            max_regs_per_block: 1,
            serving_rate_limit: 10,
            max_validators: 64,
            adjustment_alpha: 58000,
            difficulty: 10_000_000,
            commit_reveal_weights_enabled: false,
            commit_reveal_weights_interval: 1000,
            liquid_alpha_enabled: true,
        };
        let json = serde_json::to_string(&hp).unwrap();
        let d: SubnetHyperparameters = serde_json::from_str(&json).unwrap();
        assert_eq!(d.netuid, NetUid(1));
        assert_eq!(d.tempo, 360);
        assert_eq!(d.max_validators, 64);
        assert!(d.liquid_alpha_enabled);
        assert!(!d.commit_reveal_weights_enabled);
        assert_eq!(d.min_burn, Balance::from_tao(0.1));
    }

    // ── Clone/Debug trait smoke tests ──────────────────────────────

    #[test]
    fn all_types_are_cloneable() {
        let neuron = make_neuron_info();
        let _ = neuron.clone();

        let lite = make_neuron_info_lite();
        let _ = lite.clone();

        let subnet = make_subnet_info();
        let _ = subnet.clone();

        let dynamic = make_dynamic_info();
        let _ = dynamic.clone();

        let mg = make_metagraph();
        let _ = mg.clone();
    }

    #[test]
    fn all_types_have_debug() {
        let neuron = make_neuron_info();
        let debug = format!("{:?}", neuron);
        assert!(debug.contains("NeuronInfo"));

        let lite = make_neuron_info_lite();
        let debug = format!("{:?}", lite);
        assert!(debug.contains("NeuronInfoLite"));

        let dynamic = make_dynamic_info();
        let debug = format!("{:?}", dynamic);
        assert!(debug.contains("DynamicInfo"));

        let mg = make_metagraph();
        let debug = format!("{:?}", mg);
        assert!(debug.contains("Metagraph"));
    }
}
