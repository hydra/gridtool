//! The mouse cursor over, or while dragging, a grip of the active object.

use std::f32::consts::FRAC_PI_4;

use eframe::egui::{CursorIcon, Vec2};
use gridtool::geometry::{BoundsHandle, Edge};

use super::object_drag::Grip;

/// The cursor for `grip`, on a view whose image x and y axes point along `axes` on screen.
pub fn grip_cursor(grip: Grip, is_dragging: bool, axes: [Vec2; 2]) -> CursorIcon {
    match grip {
        Grip::Whole => CursorIcon::Move,
        Grip::Bounds(handle) => resize_cursor(handle, axes),
        Grip::Point(_) if is_dragging => CursorIcon::Grabbing,
        Grip::Point(_) => CursorIcon::Grab,
    }
}

/// The two-way arrow nearest to the direction `handle` lies in from the centre of its box.
/// That depends on the view's rotation and not on the box's proportions, so a corner of a
/// long thin box still gets a diagonal arrow.
fn resize_cursor(handle: BoundsHandle, axes: [Vec2; 2]) -> CursorIcon {
    let side = |edge| match edge {
        Some(Edge::Min) => -1.0,
        Some(Edge::Max) => 1.0,
        None => 0.0,
    };
    let (x, y) = handle.edges();
    let direction = axes[0] * side(x) + axes[1] * side(y);
    // The arrows point both ways, so only the direction's line matters: four of them, an
    // eighth of a turn apart.
    match ((direction.angle() / FRAC_PI_4).round() as i32).rem_euclid(4) {
        0 => CursorIcon::ResizeHorizontal,
        1 => CursorIcon::ResizeNwSe,
        2 => CursorIcon::ResizeVertical,
        _ => CursorIcon::ResizeNeSw,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::emath::Rot2;

    const UNROTATED: [Vec2; 2] = [Vec2::X, Vec2::Y];

    fn rotated(degrees: f32) -> [Vec2; 2] {
        let rotation = Rot2::from_angle(degrees.to_radians());
        [rotation * Vec2::X, rotation * Vec2::Y]
    }

    #[test]
    fn each_resize_handle_has_the_arrow_of_its_direction() {
        for (handle, cursor) in [
            (BoundsHandle::TopLeft, CursorIcon::ResizeNwSe),
            (BoundsHandle::BottomRight, CursorIcon::ResizeNwSe),
            (BoundsHandle::TopRight, CursorIcon::ResizeNeSw),
            (BoundsHandle::BottomLeft, CursorIcon::ResizeNeSw),
            (BoundsHandle::Left, CursorIcon::ResizeHorizontal),
            (BoundsHandle::Right, CursorIcon::ResizeHorizontal),
            (BoundsHandle::Top, CursorIcon::ResizeVertical),
            (BoundsHandle::Bottom, CursorIcon::ResizeVertical),
        ] {
            assert_eq!(resize_cursor(handle, UNROTATED), cursor, "{handle:?}");
        }
    }

    #[test]
    fn resize_arrows_turn_with_the_view() {
        let quarter_turn = rotated(90.0);
        assert_eq!(
            resize_cursor(BoundsHandle::Right, quarter_turn),
            CursorIcon::ResizeVertical
        );
        assert_eq!(
            resize_cursor(BoundsHandle::BottomRight, quarter_turn),
            CursorIcon::ResizeNeSw
        );

        let eighth_turn = rotated(45.0);
        assert_eq!(
            resize_cursor(BoundsHandle::Right, eighth_turn),
            CursorIcon::ResizeNwSe
        );
        assert_eq!(
            resize_cursor(BoundsHandle::BottomRight, eighth_turn),
            CursorIcon::ResizeVertical
        );

        // A small rotation still reads as the nearest arrow.
        assert_eq!(
            resize_cursor(BoundsHandle::Right, rotated(350.0)),
            CursorIcon::ResizeHorizontal
        );
        assert_eq!(
            resize_cursor(BoundsHandle::TopRight, rotated(10.0)),
            CursorIcon::ResizeNeSw
        );
    }

    #[test]
    fn moving_has_four_arrows_and_points_a_hand_that_closes_when_dragged() {
        assert_eq!(grip_cursor(Grip::Whole, false, UNROTATED), CursorIcon::Move);
        assert_eq!(grip_cursor(Grip::Whole, true, UNROTATED), CursorIcon::Move);
        assert_eq!(
            grip_cursor(Grip::Point(0), false, UNROTATED),
            CursorIcon::Grab
        );
        assert_eq!(
            grip_cursor(Grip::Point(0), true, UNROTATED),
            CursorIcon::Grabbing
        );
        let handle = Grip::Bounds(BoundsHandle::Top);
        assert_eq!(
            grip_cursor(handle, false, UNROTATED),
            grip_cursor(handle, true, UNROTATED)
        );
    }
}
