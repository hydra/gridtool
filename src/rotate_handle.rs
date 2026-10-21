//! Geometry of the rotate tool's arrow handle, which points from the document's
//! center in the direction of its rotation.

use eframe::egui::emath::Rot2;
use eframe::egui::{Pos2, Vec2};

/// Arrow length in screen points.
pub const ARROW_LENGTH: f32 = 200.0;

/// How close (in screen points) the pointer must be to the arrow tip to grab it.
pub const GRAB_RADIUS: f32 = 12.0;

/// Screen position of the arrow tip. At zero rotation the arrow points right.
pub fn tip(center: Pos2, rotation_radians: f32) -> Pos2 {
    center + Rot2::from_angle(rotation_radians) * Vec2::new(ARROW_LENGTH, 0.0)
}

/// Whether `pointer` is close enough to the arrow tip to start dragging it.
pub fn is_over_tip(center: Pos2, rotation_radians: f32, pointer: Pos2) -> bool {
    pointer.distance(tip(center, rotation_radians)) <= GRAB_RADIUS
}

/// The rotation that makes the arrow point at `pointer`, or `None` if the pointer is
/// on the center (where the direction is undefined).
pub fn rotation_towards(center: Pos2, pointer: Pos2) -> Option<f32> {
    let direction = pointer - center;
    (direction.length_sq() > f32::EPSILON).then(|| direction.angle())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, PI};

    const CENTER: Pos2 = Pos2::new(100.0, 100.0);

    fn assert_near(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-4, "expected {expected}, got {actual}");
    }

    #[test]
    fn tip_points_right_when_unrotated() {
        assert_eq!(tip(CENTER, 0.0), Pos2::new(100.0 + ARROW_LENGTH, 100.0));
    }

    #[test]
    fn tip_points_down_after_a_clockwise_quarter_turn() {
        let tip = tip(CENTER, FRAC_PI_2);

        assert_near(tip.x, 100.0);
        assert_near(tip.y, 100.0 + ARROW_LENGTH);
    }

    #[test]
    fn grabs_only_near_the_tip() {
        let tip = tip(CENTER, 0.0);

        assert!(is_over_tip(CENTER, 0.0, tip));
        assert!(is_over_tip(CENTER, 0.0, tip + Vec2::new(0.0, GRAB_RADIUS)));
        assert!(!is_over_tip(CENTER, 0.0, tip + Vec2::new(0.0, GRAB_RADIUS + 1.0)));
        assert!(!is_over_tip(CENTER, 0.0, CENTER));
    }

    #[test]
    fn rotation_follows_the_pointer_direction() {
        assert_near(rotation_towards(CENTER, CENTER + Vec2::new(5.0, 0.0)).unwrap(), 0.0);
        assert_near(rotation_towards(CENTER, CENTER + Vec2::new(0.0, 5.0)).unwrap(), FRAC_PI_2);
        assert_near(rotation_towards(CENTER, CENTER + Vec2::new(-5.0, 0.0)).unwrap(), PI);
    }

    #[test]
    fn rotation_is_undefined_at_the_center() {
        assert_eq!(rotation_towards(CENTER, CENTER), None);
    }
}
