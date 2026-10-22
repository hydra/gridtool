//! Bringing an object picked in the object tree into view: the view is left alone while
//! any part of the object is on screen; otherwise it pans to the object, and zooms out if
//! the object still does not fit.

use eframe::egui::{Pos2, Rect, Vec2};
use gridtool::document::Outline;
use gridtool::geometry::Point;

use super::viewport::Viewport;

/// The space kept clear between a revealed object and the edge of the canvas, in screen
/// points, when the view has to zoom out to fit it.
pub const FIT_MARGIN: f32 = 16.0;

/// Makes `outline` visible in the canvas `view` (a screen rect) if none of it is: pans so
/// that `focus` is at the centre, then zooms out about the centre until the outline fits.
/// Rotation is never changed.
pub fn reveal(
    viewport: &mut Viewport,
    outline: &Outline,
    focus: Point,
    document_size: Vec2,
    view: Rect,
) {
    let on_screen = |viewport: &Viewport| -> Vec<Pos2> {
        let transform = viewport.transform(view.center(), document_size);
        outline
            .points
            .iter()
            .map(|point| transform.image_to_screen(*point))
            .collect()
    };
    if is_visible(&on_screen(viewport), outline.closed, view) {
        return;
    }
    viewport.center_on(focus, document_size);
    let room = view.shrink(FIT_MARGIN);
    let room = if room.is_positive() { room } else { view };
    let factor = fit_factor(&on_screen(viewport), room.center(), room.size() * 0.5);
    if factor < 1.0 {
        viewport.zoom_about(view.center(), view.center(), factor);
    }
}

/// Whether any part of the shape through `points` (screen positions) is inside `rect`:
/// a point, part of an edge, or, for a `closed` shape, its inside.
pub fn is_visible(points: &[Pos2], closed: bool, rect: Rect) -> bool {
    if points.iter().any(|point| rect.contains(*point)) {
        return true;
    }
    if edges(points, closed).any(|(a, b)| segment_crosses_rect(a, b, rect)) {
        return true;
    }
    // With no point inside and no edge crossing, the rect is wholly inside or wholly
    // outside the shape, so its centre decides.
    closed && polygon_contains(points, rect.center())
}

/// The largest factor (at most 1) by which to scale `points` about `center` so that they
/// all lie within `half_size` of it on each axis. 1 if they already do.
fn fit_factor(points: &[Pos2], center: Pos2, half_size: Vec2) -> f32 {
    points
        .iter()
        .flat_map(|point| {
            let offset = *point - center;
            [(offset.x, half_size.x), (offset.y, half_size.y)]
        })
        .filter(|(offset, _)| offset.abs() > f32::EPSILON)
        .map(|(offset, half)| half / offset.abs())
        .fold(1.0, f32::min)
}

fn edges(points: &[Pos2], closed: bool) -> impl Iterator<Item = (Pos2, Pos2)> + '_ {
    let closing = (closed && points.len() > 2)
        .then(|| (points[points.len() - 1], points[0]))
        .into_iter();
    points
        .windows(2)
        .map(|pair| (pair[0], pair[1]))
        .chain(closing)
}

/// Whether segment `a`–`b`, with both ends outside `rect`, crosses it.
fn segment_crosses_rect(a: Pos2, b: Pos2, rect: Rect) -> bool {
    let corners = [
        rect.left_top(),
        rect.right_top(),
        rect.right_bottom(),
        rect.left_bottom(),
    ];
    (0..4).any(|i| segments_intersect(a, b, corners[i], corners[(i + 1) % 4]))
}

fn segments_intersect(a: Pos2, b: Pos2, c: Pos2, d: Pos2) -> bool {
    let cross = |o: Pos2, p: Pos2, q: Pos2| (p - o).x * (q - o).y - (p - o).y * (q - o).x;
    let (d1, d2) = (cross(c, d, a), cross(c, d, b));
    let (d3, d4) = (cross(a, b, c), cross(a, b, d));
    let straddles = |p: f32, q: f32| (p <= 0.0 && q >= 0.0) || (p >= 0.0 && q <= 0.0);
    let collinear = d1 == 0.0 && d2 == 0.0;
    if collinear {
        return Rect::from_two_pos(a, b).intersects(Rect::from_two_pos(c, d));
    }
    straddles(d1, d2) && straddles(d3, d4)
}

/// Even-odd test of whether `point` is inside the polygon through `points`.
fn polygon_contains(points: &[Pos2], point: Pos2) -> bool {
    edges(points, true)
        .filter(|(a, b)| (a.y > point.y) != (b.y > point.y))
        .filter(|(a, b)| point.x < a.x + (point.y - a.y) / (b.y - a.y) * (b.x - a.x))
        .count()
        % 2
        == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: Rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(800.0, 600.0));
    const DOCUMENT_SIZE: Vec2 = Vec2::new(4000.0, 3000.0);

    fn line(from: (f64, f64), to: (f64, f64)) -> Outline {
        Outline {
            points: vec![Point::new(from.0, from.1), Point::new(to.0, to.1)],
            closed: false,
        }
    }

    fn rectangle(min: (f64, f64), max: (f64, f64)) -> Outline {
        Outline {
            points: [
                (min.0, min.1),
                (max.0, min.1),
                (max.0, max.1),
                (min.0, max.1),
            ]
            .map(|(x, y)| Point::new(x, y))
            .to_vec(),
            closed: true,
        }
    }

    fn screen(outline: &Outline, viewport: &Viewport) -> Vec<Pos2> {
        let transform = viewport.transform(VIEW.center(), DOCUMENT_SIZE);
        outline
            .points
            .iter()
            .map(|point| transform.image_to_screen(*point))
            .collect()
    }

    /// At the default view, image (2000, 1500) is at the view centre (400, 300), so the
    /// visible image region is (1600, 1200)–(2400, 1800).
    fn reveal_in(viewport: &mut Viewport, outline: &Outline) {
        let focus = gridtool::geometry::Bounds::enclosing(outline.points.iter().copied())
            .unwrap()
            .center();
        reveal(viewport, outline, focus, DOCUMENT_SIZE, VIEW);
    }

    #[test]
    fn an_object_partly_on_screen_does_not_move_the_view() {
        for outline in [
            // One end on screen.
            line((2300.0, 1500.0), (3500.0, 1500.0)),
            // Both ends off screen, crossing it.
            line((1000.0, 1500.0), (3000.0, 1500.0)),
            // An area with no point on screen, whose edges cross it.
            rectangle((1000.0, 1400.0), (3000.0, 1700.0)),
            // An area surrounding the whole screen.
            rectangle((1000.0, 1000.0), (3000.0, 2000.0)),
        ] {
            let mut viewport = Viewport::default();

            reveal_in(&mut viewport, &outline);

            assert_eq!(viewport, Viewport::default(), "{outline:?}");
        }
    }

    #[test]
    fn an_object_off_screen_is_centred_without_zooming_when_it_fits() {
        let mut viewport = Viewport::default();
        let outline = line((3000.0, 2500.0), (3200.0, 2500.0));

        reveal_in(&mut viewport, &outline);

        let points = screen(&outline, &viewport);
        assert!((points[0] - Pos2::new(300.0, 300.0)).length() < 1e-2);
        assert!((points[1] - Pos2::new(500.0, 300.0)).length() < 1e-2);
        assert_eq!(viewport.zoom(), 1.0);
    }

    #[test]
    fn an_object_too_big_for_the_view_is_zoomed_out_to_fit_within_the_margin() {
        let mut viewport = Viewport::default();
        // 2000 px wide, centred at (1500, 2600): entirely below the visible region.
        let outline = line((500.0, 2600.0), (2500.0, 2600.0));

        reveal_in(&mut viewport, &outline);

        let expected_zoom = (800.0 - 2.0 * FIT_MARGIN) / 2000.0;
        assert!((viewport.zoom() - expected_zoom).abs() < 1e-5);
        let points = screen(&outline, &viewport);
        assert!((points[0].x - FIT_MARGIN).abs() < 1e-2);
        assert!((points[1].x - (800.0 - FIT_MARGIN)).abs() < 1e-2);
        assert!((points[0].y - 300.0).abs() < 1e-2);
    }

    #[test]
    fn the_fit_follows_the_view_rotation() {
        let mut viewport = Viewport::default();
        viewport.set_rotation_degrees(90.0);
        // Rotated a quarter turn, this 1000 px horizontal line is drawn vertically, and
        // the view is only 600 tall.
        let outline = line((100.0, 100.0), (1100.0, 100.0));

        reveal_in(&mut viewport, &outline);

        let expected_zoom = (600.0 - 2.0 * FIT_MARGIN) / 1000.0;
        assert!((viewport.zoom() - expected_zoom).abs() < 1e-5);
        assert_eq!(viewport.rotation_degrees(), 90.0);
        let points = screen(&outline, &viewport);
        assert!((points[0].x - 400.0).abs() < 1e-2 && (points[1].x - 400.0).abs() < 1e-2);
    }

    #[test]
    fn visibility_covers_points_edges_and_insides() {
        let rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(10.0, 10.0));
        let ring = [(-5.0, -5.0), (15.0, -5.0), (15.0, 15.0), (-5.0, 15.0)].map(Pos2::from);

        assert!(is_visible(&[Pos2::new(5.0, 5.0)], false, rect));
        assert!(is_visible(&ring, true, rect));
        assert!(
            !is_visible(&ring, false, rect),
            "an open path around the rect"
        );
        assert!(is_visible(
            &[Pos2::new(-5.0, 5.0), Pos2::new(15.0, 5.0)],
            false,
            rect
        ));
        assert!(!is_visible(
            &[Pos2::new(-5.0, 20.0), Pos2::new(15.0, 20.0)],
            false,
            rect
        ));
        // A triangle beside the rect, whose bounding box overlaps it.
        let beside = [(12.0, -5.0), (20.0, 20.0), (12.0, 20.0)].map(Pos2::from);
        assert!(!is_visible(&beside, true, rect));
    }
}
