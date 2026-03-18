//! Formatting helpers for CLI display.

use crate::types::Balance;

/// Truncate an SS58 address for display: "5Gx...abc"
pub fn short_ss58(addr: &str) -> String {
    // SS58 addresses are ASCII, so byte indexing is safe and avoids Vec<char> allocation
    if addr.len() <= 10 {
        return addr.to_string();
    }
    let mut s = String::with_capacity(11); // 4 + 3 + 4
    s.push_str(&addr[..4]);
    s.push_str("...");
    s.push_str(&addr[addr.len() - 4..]);
    s
}

/// Format TAO balance with commas: "1,234.5678"
pub fn format_tao(balance: Balance) -> String {
    let rao = balance.rao();
    let whole_tao = rao / crate::types::balance::RAO_PER_TAO;
    // 4 decimal places: divide remaining rao by 100_000 (1e9 / 1e4)
    let frac_part = (rao % crate::types::balance::RAO_PER_TAO) / 100_000;
    if whole_tao >= 1_000 {
        // Build whole part with proper comma separators for any magnitude
        let whole_str = whole_tao.to_string();
        let mut comma_str = String::with_capacity(whole_str.len() + whole_str.len() / 3);
        for (i, c) in whole_str.chars().enumerate() {
            if i > 0 && (whole_str.len() - i) % 3 == 0 {
                comma_str.push(',');
            }
            comma_str.push(c);
        }
        format!("{}.{:04}", comma_str, frac_part)
    } else {
        format!("{}.{:04}", whole_tao, frac_part)
    }
}

/// Truncate a string to `max` chars, appending ellipsis if needed.
pub fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        return s.to_string();
    }
    // Take max-1 chars and append ellipsis, avoiding intermediate Vec<char>
    let end = s.char_indices()
        .nth(max - 1)
        .map(|(i, _)| i)
        .unwrap_or(s.len());
    let mut result = String::with_capacity(end + 3); // +3 for '…' (UTF-8)
    result.push_str(&s[..end]);
    result.push('…');
    result
}

/// Normalize u16 weight to f64 in [0, 1].
pub fn u16_to_float(val: u16) -> f64 {
    val as f64 / 65535.0
}

/// Convert f64 in [0, 1] to u16 weight (rounded to nearest).
pub fn float_to_u16(val: f64) -> u16 {
    (val.clamp(0.0, 1.0) * 65535.0).round() as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_address() {
        let addr = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
        assert_eq!(short_ss58(addr), "5Grw...utQY");
    }

    #[test]
    fn short_address_short_input() {
        assert_eq!(short_ss58("abcde"), "abcde");
        assert_eq!(short_ss58(""), "");
        assert_eq!(short_ss58("1234567890"), "1234567890");
    }

    #[test]
    fn weight_roundtrip() {
        let f = 0.5;
        let u = float_to_u16(f);
        let back = u16_to_float(u);
        assert!((back - f).abs() < 0.001);
    }

    #[test]
    fn weight_boundaries() {
        assert_eq!(float_to_u16(0.0), 0);
        assert_eq!(float_to_u16(1.0), 65535);
        assert_eq!(u16_to_float(0), 0.0);
        assert!((u16_to_float(65535) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weight_clamp() {
        assert_eq!(float_to_u16(-1.0), 0);
        assert_eq!(float_to_u16(2.0), 65535);
    }

    #[test]
    fn format_tao_small() {
        let b = Balance::from_tao(0.5);
        let s = format_tao(b);
        assert!(s.starts_with("0."));
    }

    #[test]
    fn format_tao_large() {
        let b = Balance::from_tao(12345.6789);
        let s = format_tao(b);
        assert!(s.contains(","));
    }

    // ── Fix: format_tao for values >= 1M TAO (Issue 653) ──

    #[test]
    fn format_tao_million() {
        let b = Balance::from_rao(1_234_567_890_000_000);
        let s = format_tao(b);
        assert!(s.starts_with("1,234,567"), "got: {}", s);
    }

    #[test]
    fn format_tao_billion() {
        // 9,876,543 TAO — verifies multi-comma formatting
        let b = Balance::from_rao(9_876_543_000_000_000);
        let s = format_tao(b);
        assert!(s.starts_with("9,876,543"), "got: {}", s);
    }

    #[test]
    fn format_tao_thousands() {
        // Verify existing thousands formatting still works
        let b = Balance::from_rao(1_500_000_000_000); // 1,500 TAO
        let s = format_tao(b);
        assert!(s.starts_with("1,500"), "got: {}", s);
    }

    #[test]
    fn format_tao_exact_1000() {
        let b = Balance::from_rao(1_000_000_000_000); // exactly 1,000 TAO
        let s = format_tao(b);
        assert!(s.starts_with("1,000"), "got: {}", s);
    }

    // ── Fix: float_to_u16 rounding (Issue 659) ──

    #[test]
    fn float_to_u16_rounds_not_truncates() {
        // 0.5 should map to 32768 (rounded), not 32767 (truncated)
        assert_eq!(float_to_u16(0.5), 32768);
    }

    #[test]
    fn float_to_u16_roundtrip_half() {
        // Roundtrip: 0.5 → u16 → float should be close to 0.5
        let u = float_to_u16(0.5);
        let back = u16_to_float(u);
        assert!((back - 0.5).abs() < 0.0001, "got: {}", back);
    }

    // ──── Issue 115: format_tao uses integer arithmetic (no float precision loss) ────

    #[test]
    fn format_tao_integer_precision_exact() {
        // Exactly 1234.5678 TAO = 1_234_567_800_000 RAO
        let b = Balance::from_rao(1_234_567_800_000);
        let s = format_tao(b);
        assert_eq!(s, "1,234.5678", "got: {}", s);
    }

    #[test]
    fn format_tao_zero() {
        let b = Balance::from_rao(0);
        let s = format_tao(b);
        assert_eq!(s, "0.0000", "got: {}", s);
    }

    #[test]
    fn format_tao_sub_tao() {
        // 0.1234 TAO = 123_400_000 RAO
        let b = Balance::from_rao(123_400_000);
        let s = format_tao(b);
        assert_eq!(s, "0.1234", "got: {}", s);
    }

    #[test]
    fn format_tao_large_value_no_float_error() {
        // 18,446,744,073 TAO (near u64::MAX / 1e9) using integer arithmetic
        let rao = 18_446_744_073_000_000_000u64;
        let b = Balance::from_rao(rao);
        let s = format_tao(b);
        // Should start with "18,446,744,073" and not produce negative frac
        assert!(s.starts_with("18,446,744,073"), "got: {}", s);
        assert!(!s.contains('-'), "Should not contain negative sign: {}", s);
    }

    #[test]
    fn format_tao_exactly_999() {
        // 999 TAO — should not have commas
        let b = Balance::from_rao(999_000_000_000);
        let s = format_tao(b);
        assert_eq!(s, "999.0000", "got: {}", s);
    }

    // ──── Issue 126 (v24): truncate(s, 0) must not panic ────

    #[test]
    fn truncate_zero_max_returns_empty() {
        assert_eq!(truncate("hello world", 0), "");
        assert_eq!(truncate("", 0), "");
    }

    #[test]
    fn truncate_max_one() {
        // max=1: Should take 0 chars and append ellipsis? No — take max-1=0 chars + ellipsis
        // Actually the function takes max-1 chars boundary + ellipsis for strings > max
        let result = truncate("hello", 1);
        // "hello" has 5 chars > 1, so truncate to 0 chars + ellipsis
        assert_eq!(result, "…");
    }
}
