//! Angles in degrees, kept within one turn.

/// Angles are kept below this. 360 itself is the same as 0.
pub const FULL_TURN: f64 = 360.0;

/// `degrees` wrapped around a full turn, into [0, 360). A non-finite angle becomes 0.
pub fn normalize_degrees(degrees: f64) -> f64 {
    // The remainder of a negative multiple of a full turn is negative zero.
    let angle = degrees.rem_euclid(FULL_TURN).abs();
    // The remainder of a tiny negative angle rounds up to the full turn itself, and that of a
    // non-finite one is NaN, which is not below it either.
    if angle < FULL_TURN { angle } else { 0.0 }
}

/// Whether `degrees` is an angle as stored: in [0, 360).
pub fn is_normalized_degrees(degrees: f64) -> bool {
    (0.0..FULL_TURN).contains(&degrees)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angles_in_range_are_kept() {
        assert_eq!(normalize_degrees(0.0), 0.0);
        assert_eq!(normalize_degrees(123.5), 123.5);
    }

    #[test]
    fn full_turns_and_negative_angles_are_wrapped() {
        assert_eq!(normalize_degrees(360.0), 0.0);
        assert_eq!(normalize_degrees(450.0), 90.0);
        assert_eq!(normalize_degrees(-90.0), 270.0);
        assert!(normalize_degrees(-720.0).is_sign_positive());
        assert_eq!(normalize_degrees(-720.0), 0.0);
    }

    #[test]
    fn a_tiny_negative_angle_is_never_a_full_turn() {
        assert!(normalize_degrees(-1e-14) < FULL_TURN);
    }

    #[test]
    fn non_finite_angles_become_zero() {
        assert_eq!(normalize_degrees(f64::NAN), 0.0);
        assert_eq!(normalize_degrees(f64::INFINITY), 0.0);
        assert_eq!(normalize_degrees(f64::NEG_INFINITY), 0.0);
    }

    #[test]
    fn only_angles_within_one_turn_are_normalized() {
        assert!(is_normalized_degrees(0.0));
        assert!(is_normalized_degrees(359.9));
        assert!(!is_normalized_degrees(360.0));
        assert!(!is_normalized_degrees(-0.1));
        assert!(!is_normalized_degrees(f64::NAN));
    }
}
