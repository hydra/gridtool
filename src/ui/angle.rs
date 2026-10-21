//! Helpers for working with angles expressed in degrees.

use super::format::format_trimmed;

const FULL_TURN_DEGREES: f32 = 360.0;
const DISPLAY_DECIMALS: usize = 2;

/// Wraps an angle into the range `[0, 360)`. Non-finite angles become `0`.
pub fn normalize_degrees(degrees: f32) -> f32 {
    if !degrees.is_finite() {
        return 0.0;
    }
    let wrapped = degrees.rem_euclid(FULL_TURN_DEGREES);
    // `rem_euclid` can round up to exactly 360 for tiny negative inputs.
    if wrapped >= FULL_TURN_DEGREES {
        0.0
    } else {
        wrapped
    }
}

/// Parses user input such as `"45"`, `" -12.5 "` or `"90°"` into degrees.
pub fn parse_degrees(text: &str) -> Option<f32> {
    let number = text.trim().trim_end_matches('°').trim_end();
    number
        .parse::<f32>()
        .ok()
        .filter(|degrees| degrees.is_finite())
}

/// Formats degrees for display, without trailing zeros (e.g. `45`, `12.5`).
pub fn format_degrees(degrees: f32) -> String {
    format_trimmed(f64::from(degrees), DISPLAY_DECIMALS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_keeps_angles_already_in_range() {
        assert_eq!(normalize_degrees(0.0), 0.0);
        assert_eq!(normalize_degrees(123.5), 123.5);
    }

    #[test]
    fn normalize_wraps_full_turns_and_negative_angles() {
        assert_eq!(normalize_degrees(360.0), 0.0);
        assert_eq!(normalize_degrees(450.0), 90.0);
        assert_eq!(normalize_degrees(-90.0), 270.0);
        assert_eq!(normalize_degrees(-720.0), 0.0);
    }

    #[test]
    fn normalize_never_returns_a_full_turn() {
        assert!(normalize_degrees(-1e-7) < FULL_TURN_DEGREES);
    }

    #[test]
    fn normalize_maps_non_finite_angles_to_zero() {
        assert_eq!(normalize_degrees(f32::NAN), 0.0);
        assert_eq!(normalize_degrees(f32::INFINITY), 0.0);
    }

    #[test]
    fn parse_accepts_numbers_with_whitespace_and_degree_sign() {
        assert_eq!(parse_degrees("45"), Some(45.0));
        assert_eq!(parse_degrees("  -12.5 "), Some(-12.5));
        assert_eq!(parse_degrees("90°"), Some(90.0));
        assert_eq!(parse_degrees("90 °"), Some(90.0));
    }

    #[test]
    fn parse_rejects_invalid_input() {
        assert_eq!(parse_degrees(""), None);
        assert_eq!(parse_degrees("abc"), None);
        assert_eq!(parse_degrees("inf"), None);
        assert_eq!(parse_degrees("NaN"), None);
    }

    #[test]
    fn format_uses_two_decimals_at_most() {
        assert_eq!(format_degrees(45.0), "45");
        assert_eq!(format_degrees(12.345), "12.35");
    }
}
