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

/// Formats a measured real-world length (up to 3 decimals).
pub fn format_length(value: f64) -> String {
    format_trimmed(value, 3)
}

/// Formats an image coordinate for editing (up to 2 decimals).
pub fn format_coordinate(value: f64) -> String {
    format_trimmed(value, 2)
}

/// Parses a decimal number typed by the user, accepting only finite values.
pub fn parse_number(text: &str) -> Option<f64> {
    text.trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
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
    fn lengths_and_coordinates_use_their_own_precision() {
        assert_eq!(format_length(12.34567), "12.346");
        assert_eq!(format_coordinate(12.34567), "12.35");
        assert_eq!(format_coordinate(40.0), "40");
    }

    #[test]
    fn parses_finite_numbers_only() {
        assert_eq!(parse_number(" -12.5 "), Some(-12.5));
        assert_eq!(parse_number("0"), Some(0.0));
        for invalid in ["", "abc", "inf", "NaN", "1,5"] {
            assert_eq!(
                parse_number(invalid),
                None,
                "{invalid:?} should be rejected"
            );
        }
    }

    #[test]
    fn never_shows_negative_zero() {
        assert_eq!(format_trimmed(-0.0, 2), "0");
        assert_eq!(format_trimmed(-0.001, 2), "0");
    }
}
