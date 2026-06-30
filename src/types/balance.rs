//! TAO and Alpha balance types.
//!
//! TAO has 9 decimal places (1 TAO = 1_000_000_000 RAO).

use serde::{Deserialize, Serialize};
use std::fmt;

/// One TAO in RAO (smallest unit).
pub const RAO_PER_TAO: u64 = 1_000_000_000;

/// Represents a TAO balance stored as RAO (u64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Balance {
    rao: u64,
}

impl Balance {
    pub const ZERO: Self = Self { rao: 0 };

    /// Create from RAO (smallest unit).
    pub fn from_rao(rao: u64) -> Self {
        Self { rao }
    }

    /// Create from TAO (floating point, rounded to nearest RAO).
    ///
    /// Returns `Balance::ZERO` for negative, NaN, or infinite inputs.
    pub fn from_tao(tao: f64) -> Self {
        if !tao.is_finite() || tao < 0.0 {
            return Self::ZERO;
        }
        let rao_f = (tao * RAO_PER_TAO as f64).round();
        // After rounding, clamp to u64 range (saturating cast)
        let rao = if rao_f >= u64::MAX as f64 {
            u64::MAX
        } else {
            rao_f as u64
        };
        Self { rao }
    }

    /// Return value in RAO.
    pub fn rao(&self) -> u64 {
        self.rao
    }

    /// Return value in TAO.
    pub fn tao(&self) -> f64 {
        self.rao as f64 / RAO_PER_TAO as f64
    }

    /// Human-readable string like "1.234567890 τ".
    pub fn display_tao(&self) -> String {
        format!("{:.9} τ", self.tao())
    }
}

impl fmt::Display for Balance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} τ", self.tao())
    }
}

impl std::ops::Add for Balance {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            rao: self.rao.saturating_add(rhs.rao),
        }
    }
}

impl std::ops::Sub for Balance {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            rao: self.rao.saturating_sub(rhs.rao),
        }
    }
}

/// Represents an Alpha token balance for a subnet's native token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AlphaBalance {
    raw: u64,
}

impl AlphaBalance {
    pub const ZERO: Self = Self { raw: 0 };

    pub fn from_raw(raw: u64) -> Self {
        Self { raw }
    }

    /// Create from human-readable alpha units (9 decimals, same scale as TAO/RAO).
    ///
    /// On-chain `remove_stake`, `move_stake`, and `swap_stake` take `AlphaBalance`, not TAO.
    /// Invalid input maps to zero; prefer [`Self::try_from_units`] for SDK callers.
    pub fn from_units(units: f64) -> Self {
        Self::try_from_units(units).unwrap_or(Self::ZERO)
    }

    /// Like [`Self::from_units`] but errors on non-finite or negative input.
    pub fn try_from_units(units: f64) -> anyhow::Result<Self> {
        if !units.is_finite() || units < 0.0 {
            anyhow::bail!("invalid alpha amount: must be finite and non-negative, got {units}");
        }
        let raw_f = (units * RAO_PER_TAO as f64).round();
        let raw = if raw_f >= u64::MAX as f64 {
            u64::MAX
        } else {
            raw_f as u64
        };
        Ok(Self { raw })
    }

    pub fn raw(&self) -> u64 {
        self.raw
    }

    /// Human-readable alpha units (9 decimals).
    pub fn units(&self) -> f64 {
        self.raw as f64 / RAO_PER_TAO as f64
    }

    /// TAO-equivalent of this alpha at a subnet `price` (**TAO per alpha**)
    pub fn to_tao(&self, price: f64) -> Balance {
        Balance::from_tao(self.units() * price)
    }

    /// Human-readable string like "1.234567890 α" (9 decimals, mirrors [`Balance::display_tao`]).
    pub fn display_units(&self) -> String {
        format!("{:.9} α", self.units())
    }
}

impl fmt::Display for AlphaBalance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4} α", self.raw as f64 / RAO_PER_TAO as f64)
    }
}

/// AMM limit-order price: **TAO per alpha**, on-chain as RAO/α (`limit_price` u64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LimitPriceRao(u64);

impl LimitPriceRao {
    pub fn from_rao(rao: u64) -> Self {
        Self(rao)
    }

    pub fn rao(&self) -> u64 {
        self.0
    }

    /// Human TAO/α for display.
    pub fn tao_per_alpha(&self) -> f64 {
        self.0 as f64 / RAO_PER_TAO as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tao_conversion() {
        let b = Balance::from_tao(1.5);
        assert_eq!(b.rao(), 1_500_000_000);
        assert!((b.tao() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn saturating_ops() {
        let a = Balance::from_rao(10);
        let b = Balance::from_rao(20);
        assert_eq!((a - b).rao(), 0); // saturating
    }

    #[test]
    fn zero_balance() {
        let b = Balance::ZERO;
        assert_eq!(b.rao(), 0);
        assert_eq!(b.tao(), 0.0);
    }

    #[test]
    fn add_balances() {
        let a = Balance::from_rao(500_000_000);
        let b = Balance::from_rao(700_000_000);
        let c = a + b;
        assert_eq!(c.rao(), 1_200_000_000);
        assert!((c.tao() - 1.2).abs() < 1e-9);
    }

    #[test]
    fn saturating_add_at_max() {
        let a = Balance::from_rao(u64::MAX);
        let b = Balance::from_rao(1);
        let c = a + b;
        assert_eq!(c.rao(), u64::MAX);
    }

    #[test]
    fn display_tao_format() {
        let b = Balance::from_tao(3.15);
        let s = b.display_tao();
        assert!(s.contains("τ"));
        assert!(s.starts_with("3."));
    }

    #[test]
    fn from_tao_fractional() {
        let b = Balance::from_tao(0.000000001);
        assert_eq!(b.rao(), 1);
    }

    #[test]
    fn ordering() {
        let a = Balance::from_tao(1.0);
        let b = Balance::from_tao(2.0);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn alpha_balance_basics() {
        let a = AlphaBalance::ZERO;
        assert_eq!(a.raw(), 0);
        let b = AlphaBalance::from_raw(1_000_000_000);
        assert_eq!(b.raw(), 1_000_000_000);
        let display = format!("{}", b);
        assert!(display.contains("α"));
    }

    #[test]
    fn alpha_from_units_matches_tao_scale() {
        let a = AlphaBalance::from_units(1.5);
        assert_eq!(a.raw(), 1_500_000_000);
        assert!((a.units() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn alpha_to_tao_applies_price() {
        // 2753.93 α at 0.009219 τ/α ≈ 25.39 τ — the per-subnet conversion the CLI must do,
        // not the raw alpha reinterpreted as τ (which would read ~2753 τ, ~108× too high).
        let alpha = AlphaBalance::from_units(2753.9299566);
        let tao = alpha.to_tao(0.009219);
        assert!((tao.tao() - 25.3905).abs() < 0.01, "got {} τ", tao.tao());
    }

    #[test]
    fn alpha_to_tao_zero_price_is_zero() {
        assert_eq!(AlphaBalance::from_units(1000.0).to_tao(0.0), Balance::ZERO);
    }

    #[test]
    fn try_from_units_rejects_invalid() {
        assert!(AlphaBalance::try_from_units(-1.0).is_err());
        assert!(AlphaBalance::try_from_units(f64::NAN).is_err());
        assert!(AlphaBalance::try_from_units(f64::INFINITY).is_err());
    }

    #[test]
    fn balance_equality() {
        let a = Balance::from_tao(1.0);
        let b = Balance::from_rao(1_000_000_000);
        assert_eq!(a, b);
    }

    #[test]
    fn balance_serialization() {
        let b = Balance::from_tao(2.5);
        let json = serde_json::to_string(&b).unwrap();
        let deserialized: Balance = serde_json::from_str(&json).unwrap();
        assert_eq!(b, deserialized);
    }

    // ── Fix: from_tao input validation (Issues 652, 657) ──

    #[test]
    fn from_tao_negative_returns_zero() {
        assert_eq!(Balance::from_tao(-1.0), Balance::ZERO);
        assert_eq!(Balance::from_tao(-0.001), Balance::ZERO);
        assert_eq!(Balance::from_tao(f64::MIN), Balance::ZERO);
    }

    #[test]
    fn from_tao_nan_returns_zero() {
        assert_eq!(Balance::from_tao(f64::NAN), Balance::ZERO);
    }

    #[test]
    fn from_tao_infinity_returns_zero() {
        assert_eq!(Balance::from_tao(f64::INFINITY), Balance::ZERO);
        assert_eq!(Balance::from_tao(f64::NEG_INFINITY), Balance::ZERO);
    }

    #[test]
    fn from_tao_rounds_correctly() {
        // 0.1 TAO should round to exactly 100_000_000 RAO, not 99_999_999
        let b = Balance::from_tao(0.1);
        assert_eq!(b.rao(), 100_000_000);
    }

    #[test]
    fn from_tao_point_three() {
        // 0.3 TAO — another f64 representation edge case
        let b = Balance::from_tao(0.3);
        assert_eq!(b.rao(), 300_000_000);
    }

    #[test]
    fn from_tao_large_value_saturates() {
        // Value larger than u64::MAX RAO should saturate
        let b = Balance::from_tao(20_000_000_000.0); // 20B TAO > u64::MAX RAO
        assert_eq!(b.rao(), u64::MAX);
    }

    #[test]
    fn from_tao_negative_zero_is_zero() {
        // -0.0 is non-negative per IEEE 754
        let b = Balance::from_tao(-0.0);
        assert_eq!(b, Balance::ZERO);
    }
}
