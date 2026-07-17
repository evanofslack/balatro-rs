/// Formats a number for display. Displays comma-separated decimal format
/// and human readable format. Shows e scale eventually (1.5e16).
pub fn format_number(n: i64) -> String {
    let commas = format_commas(n);
    match short_scale(n.unsigned_abs()) {
        Some(short) if n < 0 => format!("{commas} (-{short})"),
        Some(short) => format!("{commas} ({short})"),
        None => commas,
    }
}

fn format_commas(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    let digits = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    format!("{sign}{out}")
}

fn short_scale(n: u64) -> Option<String> {
    const MILLION: f64 = 1_000_000.0;
    const BILLION: f64 = 1_000_000_000.0;
    const TRILLION: f64 = 1_000_000_000_000.0;

    let f = n as f64;
    if f < MILLION {
        None
    } else if f < BILLION {
        Some(format!("{:.1}M", f / MILLION))
    } else if f < TRILLION {
        Some(format!("{:.1}B", f / BILLION))
    } else {
        Some(scientific(f))
    }
}

/// `1.5e16`-style scientific notation.
fn scientific(f: f64) -> String {
    let mut exponent = f.log10().floor() as i32;
    let mut mantissa = f / 10f64.powi(exponent);
    // Rounding can push mantissa to 10.0; renormalize.
    if (mantissa * 10.0).round() / 10.0 >= 10.0 {
        mantissa = 1.0;
        exponent += 1;
    }
    format!("{mantissa:.1}e{exponent}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_commas() {
        assert_eq!(format_number(307), "307");
        assert_eq!(format_number(1272), "1,272");
        assert_eq!(format_number(-1272), "-1,272");
    }

    #[test]
    fn test_format_number_millions() {
        assert_eq!(format_number(11_300_000), "11,300,000 (11.3M)");
        assert_eq!(format_number(999_999), "999,999");
    }

    #[test]
    fn test_format_number_billions() {
        assert_eq!(format_number(128_198_110_833), "128,198,110,833 (128.2B)");
    }

    #[test]
    fn test_format_number_scientific() {
        assert_eq!(format_number(1_500_000_000_000_000), "1,500,000,000,000,000 (1.5e15)");
        assert_eq!(format_number(15_000_000_000_000_000), "15,000,000,000,000,000 (1.5e16)");
    }

    #[test]
    fn test_format_number_scientific_rounding_carries_exponent() {
        // 9.996e12 rounds to 10.0e12 at 1dp, must renormalize to 1.0e13.
        assert_eq!(format_number(9_996_000_000_000), "9,996,000,000,000 (1.0e13)");
    }
}
