//! Drawing lines and fills in their styles, for every kind of object that has them, and
//! the document's drawn shapes: their fill, then their line.

use eframe::egui::{self, Pos2, Rect, Shape, Stroke, Vec2};
use gridtool::calibration::Scale;
use gridtool::color::Rgb;
use gridtool::document::Document;
use gridtool::geometry::Point;
use gridtool::style::{FillAppearance, FillStyle, LineAppearance, LineStyle, Styled};

use super::canvas::{DASH_GAP, DASH_LENGTH, fill_outline, paint_point_handle, to_screen};
use super::color::{to_color32, with_opacity};
use super::grid_display::MIN_SCREEN_SPACING;
use super::selection::Selection;
use super::viewport::DocumentTransform;

const THIN_LINE_WIDTH: f32 = 1.0;
const THICK_LINE_WIDTH: f32 = 3.0;
const DASHED_LINE_WIDTH: f32 = 2.0;
const FILL_LINE_WIDTH: f32 = 1.0;

/// Draws every shape, as currently displayed (including any live preview). The active
/// shape also shows a marker at each point; the others show only what was drawn.
pub fn paint_shapes(
    painter: &egui::Painter,
    transform: DocumentTransform,
    document: &Document,
    selection: &Selection,
) {
    // Fill lines are a real-world distance apart, so they need a scale.
    let scale = document.calibration().scale().ok();
    for stored in document.shapes() {
        let drawn = selection.displayed_drawn(stored.id(), stored);
        let on_screen = to_screen(transform, drawn.points());
        if let Some(fill) = drawn.fill() {
            let color = fill.drawn_color(drawn.line().color);
            paint_fill(
                painter,
                transform,
                scale,
                fill,
                color,
                drawn.points(),
                &on_screen,
            );
        }
        paint_line(painter, drawn.line(), &on_screen, drawn.is_closed());
        if selection.active() == Some(stored.id()) {
            for &position in &on_screen {
                paint_point_handle(painter, position, to_color32(drawn.line().color));
            }
        }
    }
}

/// Fills the closed outline through `points` (image coordinates, drawn at `on_screen`) in
/// `color`. Fill lines are a real-world distance apart, so they are drawn only if there is a
/// `scale`.
pub fn paint_fill(
    painter: &egui::Painter,
    transform: DocumentTransform,
    scale: Option<Scale>,
    fill: FillAppearance,
    color: Rgb,
    points: &[Point],
    on_screen: &[Pos2],
) {
    let color = with_opacity(color, fill.opacity);
    if fill.style == FillStyle::Solid {
        fill_outline(painter, points, on_screen, color);
        return;
    }
    let Some(scale) = scale.filter(|_| fill.style.has_lines()) else {
        return;
    };
    // When zoomed out, enough lines are skipped to keep them apart on screen, rather than
    // filling the area solid.
    let min_gap = f64::from(MIN_SCREEN_SPACING) / f64::from(transform.zoom());
    let region = transform.image_bounds_of(painter.clip_rect());
    let stroke = Stroke::new(FILL_LINE_WIDTH, color);
    for (start, end) in fill.lines(points, scale, min_gap, region) {
        painter.line_segment(
            [
                transform.image_to_screen(start),
                transform.image_to_screen(end),
            ],
            stroke,
        );
    }
}

/// Draws the line through `on_screen`, back to its start if `closed`.
pub fn paint_line(painter: &egui::Painter, line: LineAppearance, on_screen: &[Pos2], closed: bool) {
    let color = with_opacity(line.color, line.opacity);
    match line.style {
        LineStyle::Thin => paint_solid(painter, on_screen, closed, THIN_LINE_WIDTH, color),
        LineStyle::Thick => paint_solid(painter, on_screen, closed, THICK_LINE_WIDTH, color),
        LineStyle::Dashed => {
            let stroke = Stroke::new(DASHED_LINE_WIDTH, color);
            for (a, b) in edges(on_screen, closed) {
                paint_dashed_edge(painter, a, b, stroke);
            }
        }
    }
}

fn paint_solid(
    painter: &egui::Painter,
    on_screen: &[Pos2],
    closed: bool,
    width: f32,
    color: egui::Color32,
) {
    let stroke = Stroke::new(width, color);
    painter.add(if closed {
        Shape::closed_line(on_screen.to_vec(), stroke)
    } else {
        Shape::line(on_screen.to_vec(), stroke)
    });
}

/// Each edge through `points`, including the one from the last point to the first if
/// `closed`.
fn edges(points: &[Pos2], closed: bool) -> impl Iterator<Item = (Pos2, Pos2)> + '_ {
    let closing = closed
        .then(|| points.last().copied().zip(points.first().copied()))
        .flatten();
    points
        .windows(2)
        .map(|pair| (pair[0], pair[1]))
        .chain(closing)
}

/// Dashes only the part of the edge on the canvas: when zoomed far in, one edge can be
/// millions of points long, which is far too many dashes to generate every frame.
fn paint_dashed_edge(painter: &egui::Painter, a: Pos2, b: Pos2, stroke: Stroke) {
    if let Some((start, end)) = visible_dash_run(a, b, painter.clip_rect()) {
        painter.extend(Shape::dashed_line(
            &[start, end],
            stroke,
            DASH_LENGTH,
            DASH_GAP,
        ));
    }
}

/// The part of edge `a`–`b` to dash so as to cover what is inside `rect`, or `None` if
/// none of it is. It starts a whole number of dash periods from `a`, so the dashes stay in
/// the same place on the edge as the view pans.
fn visible_dash_run(a: Pos2, b: Pos2, rect: Rect) -> Option<(Pos2, Pos2)> {
    let (enter, exit) = clip_to_rect(a, b, rect)?;
    let edge = b - a;
    let length = edge.length();
    let period = DASH_LENGTH + DASH_GAP;
    let start = (enter * length / period).floor() * period / length.max(f32::EPSILON);
    Some((a + edge * start, a + edge * exit))
}

/// Where segment `a`–`b` enters and leaves `rect`, as fractions along the segment
/// (Liang–Barsky), or `None` if it misses it.
fn clip_to_rect(a: Pos2, b: Pos2, rect: Rect) -> Option<(f32, f32)> {
    let delta: Vec2 = b - a;
    let mut enter: f32 = 0.0;
    let mut exit: f32 = 1.0;
    for (direction, distance) in [
        (-delta.x, a.x - rect.left()),
        (delta.x, rect.right() - a.x),
        (-delta.y, a.y - rect.top()),
        (delta.y, rect.bottom() - a.y),
    ] {
        if direction == 0.0 {
            if distance < 0.0 {
                return None;
            }
            continue;
        }
        let at = distance / direction;
        if direction < 0.0 {
            enter = enter.max(at);
        } else {
            exit = exit.min(at);
        }
    }
    (enter <= exit).then_some((enter, exit))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: Rect = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0));

    #[test]
    fn clipping_keeps_only_the_part_of_a_segment_inside() {
        let (enter, exit) =
            clip_to_rect(Pos2::new(-50.0, 50.0), Pos2::new(150.0, 50.0), RECT).unwrap();
        assert_eq!((enter, exit), (0.25, 0.75));

        assert_eq!(
            clip_to_rect(Pos2::new(10.0, 10.0), Pos2::new(20.0, 20.0), RECT),
            Some((0.0, 1.0))
        );
    }

    #[test]
    fn clipping_rejects_segments_that_miss() {
        assert_eq!(
            clip_to_rect(Pos2::new(-50.0, 150.0), Pos2::new(150.0, 150.0), RECT),
            None
        );
        // The bounding boxes overlap, but the segment passes the corner.
        assert_eq!(
            clip_to_rect(Pos2::new(90.0, 130.0), Pos2::new(130.0, 90.0), RECT),
            None
        );
    }

    #[test]
    fn a_dash_run_is_bounded_by_the_canvas_however_long_the_edge_is() {
        let (start, end) =
            visible_dash_run(Pos2::new(-1.0e6, 50.0), Pos2::new(1.0e6, 50.0), RECT).unwrap();

        // Single precision limits how exact this can be, far from the edge's start.
        assert!(start.x <= 0.0 && start.x > -100.0, "{start:?}");
        assert!(end.x >= 100.0 && end.x < 200.0, "{end:?}");
    }

    #[test]
    fn a_dash_run_starts_a_whole_number_of_periods_along_the_edge() {
        let a = Pos2::new(-25.0, 50.0);

        let (start, _) = visible_dash_run(a, Pos2::new(75.0, 50.0), RECT).unwrap();

        // 25 is 2.5 periods of 10: the run starts at the second.
        assert!((start.x - -5.0).abs() < 1e-3, "{start:?}");
    }

    #[test]
    fn a_closed_outline_has_an_edge_back_to_the_start() {
        let points = [Pos2::ZERO, Pos2::new(1.0, 0.0), Pos2::new(1.0, 1.0)];

        assert_eq!(edges(&points, false).count(), 2);
        let closed: Vec<_> = edges(&points, true).collect();
        assert_eq!(closed.len(), 3);
        assert_eq!(closed[2], (points[2], points[0]));
    }
}
