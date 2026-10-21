/// Formats `value` with at most `decimals` decimal places, dropping trailing zeros
/// (e.g. `45`, `12.5`) and never showing `-0`.
pub fn format_trimmed(value: f64, decimals: usize) -> String {
    let formatted = format!("{value:.decimals$}");
    let trimmed = if formatted.contains('.') {
        formatted.trim_end_matches('0').trim_end_matches('.')
    } else {
        &formatted
    };
    match trimmed {
        "-0" => "0".to_owned(),
        other => other.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_trailing_zeros() {
        assert_eq!(format_trimmed(45.0, 2), "45");
        assert_eq!(format_trimmed(12.5, 2), "12.5");
        assert_eq!(format_trimmed(1.0 / 3.0, 2), "0.33");
    }

    #[test]
    fn keeps_zeros_in_whole_numbers() {
        assert_eq!(format_trimmed(100.0, 0), "100");
        assert_eq!(format_trimmed(100.0, 3), "100");
    }

    #[test]
    fn never_shows_negative_zero() {
        assert_eq!(format_trimmed(-0.0, 2), "0");
        assert_eq!(format_trimmed(-0.001, 2), "0");
    }
}
