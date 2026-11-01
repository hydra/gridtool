//! Drawn shapes: arbitrary lines through two or more points, optionally joined up to
//! enclose an area. Unlike measurements they carry only a drawing style, and nothing is
//! derived from the calibration.

use std::fmt;

use crate::color::Rgb;
use crate::geometry::{Bounds, BoundsHandle, Point};
use crate::polygon::{self, HatchDirection};

/// The fewest points that make a line.
pub const MIN_OPEN_POINTS: usize = 2;
/// The fewest points that enclose an area.
pub const MIN_CLOSED_POINTS: usize = 3;
/// The colour every shape is drawn in.
pub const COLOR: Rgb = Rgb::new(255, 128, 0);
pub const DEFAULT_LINE_OPACITY: f32 = 1.0;
pub const DEFAULT_FILL_OPACITY: f32 = 0.5;
/// Distance between fill lines of a new shape, in image pixels.
pub const DEFAULT_FILL_SPACING: f64 = 20.0;
/// The fill line distances offered to the user, in image pixels.
pub const FILL_SPACINGS: [f64; 6] = [5.0, 10.0, 20.0, 50.0, 100.0, 200.0];

/// How a shape's line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineStyle {
    #[default]
    Thin,
    Thick,
    Dashed,
}

impl LineStyle {
    pub const ALL: [LineStyle; 3] = [LineStyle::Thin, LineStyle::Thick, LineStyle::Dashed];

    pub fn name(self) -> &'static str {
        match self {
            LineStyle::Thin => "Thin",
            LineStyle::Thick => "Thick",
            LineStyle::Dashed => "Dashed",
        }
    }
}

/// How the area enclosed by a closed shape is filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillStyle {
    #[default]
    None,
    Solid,
    VerticalLines,
    HorizontalLines,
    Crosshatch,
}

impl FillStyle {
    pub const ALL: [FillStyle; 5] = [
        FillStyle::None,
        FillStyle::Solid,
        FillStyle::VerticalLines,
        FillStyle::HorizontalLines,
        FillStyle::Crosshatch,
    ];

    pub fn name(self) -> &'static str {
        match self {
            FillStyle::None => "None",
            FillStyle::Solid => "Solid",
            FillStyle::VerticalLines => "Vertical lines",
            FillStyle::HorizontalLines => "Horizontal lines",
            FillStyle::Crosshatch => "Crosshatch",
        }
    }

    /// The directions of the style's fill lines; empty if it does not use lines.
    pub fn hatch_directions(self) -> &'static [HatchDirection] {
        match self {
            FillStyle::None | FillStyle::Solid => &[],
            FillStyle::VerticalLines => &[HatchDirection::Vertical],
            FillStyle::HorizontalLines => &[HatchDirection::Horizontal],
            FillStyle::Crosshatch => &[HatchDirection::Vertical, HatchDirection::Horizontal],
        }
    }
}

/// A line through two or more points, or a closed outline through three or more.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnShape {
    points: Vec<Point>,
    closed: bool,
    line_style: LineStyle,
    line_opacity: f32,
    fill_style: FillStyle,
    /// Distance between fill lines, in image pixels.
    fill_spacing: f64,
    fill_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooFewPoints(pub usize);

impl fmt::Display for TooFewPoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a shape cannot have only {} points", self.0)
    }
}

impl std::error::Error for TooFewPoints {}

impl DrawnShape {
    /// A line through `points`, in order, in the default style.
    pub fn open(points: Vec<Point>) -> Result<Self, TooFewPoints> {
        Self::new(points, false, MIN_OPEN_POINTS)
    }

    /// A closed outline through `points`, in order, in the default style.
    pub fn closed(points: Vec<Point>) -> Result<Self, TooFewPoints> {
        Self::new(points, true, MIN_CLOSED_POINTS)
    }

    fn new(points: Vec<Point>, closed: bool, min_points: usize) -> Result<Self, TooFewPoints> {
        if points.len() < min_points {
            return Err(TooFewPoints(points.len()));
        }
        Ok(Self {
            points,
            closed,
            line_style: LineStyle::default(),
            line_opacity: DEFAULT_LINE_OPACITY,
            fill_style: FillStyle::default(),
            fill_spacing: DEFAULT_FILL_SPACING,
            fill_opacity: DEFAULT_FILL_OPACITY,
        })
    }

    /// The shape's points, in order.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Whether the last point joins the first, so that the shape encloses an area.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn line_style(&self) -> LineStyle {
        self.line_style
    }

    pub fn line_opacity(&self) -> f32 {
        self.line_opacity
    }

    pub fn fill_style(&self) -> FillStyle {
        self.fill_style
    }

    pub fn fill_spacing(&self) -> f64 {
        self.fill_spacing
    }

    pub fn fill_opacity(&self) -> f32 {
        self.fill_opacity
    }

    /// The smallest axis-aligned box containing every point.
    pub fn bounds(&self) -> Bounds {
        Bounds::enclosing(self.points.iter().copied()).expect("a shape has points")
    }

    /// The centre of the enclosed area of a closed shape; otherwise the centre of the
    /// bounds.
    pub fn center(&self) -> Point {
        self.closed
            .then(|| polygon::centroid(&self.points))
            .flatten()
            .unwrap_or_else(|| self.bounds().center())
    }

    /// The shape after dragging bounding-box `handle` by `(dx, dy)` image pixels: every
    /// point stretches with the box ([`Bounds::stretch`]).
    pub fn resized(&self, handle: BoundsHandle, dx: f64, dy: f64) -> DrawnShape {
        let bounds = self.bounds();
        self.map_points(|point| bounds.stretch(handle, dx, dy, point))
    }

    /// The shape with point `index` moved to `to`; unchanged if there is no such point.
    pub fn with_point(&self, index: usize, to: Point) -> DrawnShape {
        let mut shape = self.clone();
        if let Some(point) = shape.points.get_mut(index) {
            *point = to;
        }
        shape
    }

    /// The shape with every point passed through `map`.
    pub fn map_points(&self, map: impl Fn(Point) -> Point) -> DrawnShape {
        let mut shape = self.clone();
        shape.points = self.points.iter().map(|&point| map(point)).collect();
        shape
    }

    /// A copy with the given line opacity, e.g. to preview an edit in progress.
    pub fn with_line_opacity(&self, opacity: f32) -> DrawnShape {
        let mut shape = self.clone();
        shape.line_opacity = opacity;
        shape
    }

    /// A copy with the given fill opacity, e.g. to preview an edit in progress.
    pub fn with_fill_opacity(&self, opacity: f32) -> DrawnShape {
        let mut shape = self.clone();
        shape.fill_opacity = opacity;
        shape
    }

    /// The fill lines of a closed shape, in every direction of its fill style, that lie
    /// in `region` (see [`polygon::hatch_lines`]). `spacing` overrides the shape's fill
    /// spacing; it must be a positive multiple of it to keep the lines where they are.
    pub fn fill_lines(&self, spacing: f64, region: Bounds) -> Vec<(Point, Point)> {
        if !self.closed {
            return Vec::new();
        }
        self.fill_style
            .hatch_directions()
            .iter()
            .flat_map(|&direction| polygon::hatch_lines(&self.points, direction, spacing, region))
            .collect()
    }

    /// Replaces the points; the caller keeps the count unchanged.
    pub(crate) fn set_points(&mut self, points: Vec<Point>) {
        self.points = points;
    }

    pub(crate) fn set_line_style(&mut self, style: LineStyle) {
        self.line_style = style;
    }

    pub(crate) fn set_line_opacity(&mut self, opacity: f32) {
        self.line_opacity = opacity;
    }

    pub(crate) fn set_fill_style(&mut self, style: FillStyle) {
        self.fill_style = style;
    }

    pub(crate) fn set_fill_spacing(&mut self, spacing: f64) {
        self.fill_spacing = spacing;
    }

    pub(crate) fn set_fill_opacity(&mut self, opacity: f32) {
        self.fill_opacity = opacity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points(coordinates: &[(f64, f64)]) -> Vec<Point> {
        coordinates.iter().map(|&(x, y)| Point::new(x, y)).collect()
    }

    /// A 40 × 20 px rectangle from (10, 20).
    fn rectangle() -> DrawnShape {
        DrawnShape::closed(points(&[
            (10.0, 20.0),
            (50.0, 20.0),
            (50.0, 40.0),
            (10.0, 40.0),
        ]))
        .unwrap()
    }

    #[test]
    fn a_line_needs_two_points_and_a_closed_outline_three() {
        let two = points(&[(0.0, 0.0), (1.0, 1.0)]);

        assert_eq!(
            DrawnShape::open(vec![Point::default()]),
            Err(TooFewPoints(1))
        );
        assert!(!DrawnShape::open(two.clone()).unwrap().is_closed());
        assert_eq!(DrawnShape::closed(two), Err(TooFewPoints(2)));
        assert!(rectangle().is_closed());
    }

    #[test]
    fn a_new_shape_is_thin_and_unfilled() {
        let shape = rectangle();

        assert_eq!(shape.line_style(), LineStyle::Thin);
        assert_eq!(shape.line_opacity(), 1.0);
        assert_eq!(shape.fill_style(), FillStyle::None);
        assert_eq!(shape.fill_spacing(), 20.0);
        assert_eq!(shape.fill_opacity(), 0.5);
        assert!(FILL_SPACINGS.contains(&shape.fill_spacing()));
    }

    #[test]
    fn the_centre_of_a_closed_shape_is_its_centroid_and_of_a_line_its_bounds_centre() {
        let line = DrawnShape::open(points(&[(0.0, 0.0), (6.0, 0.0), (0.0, 3.0)])).unwrap();
        let triangle = DrawnShape::closed(points(&[(0.0, 0.0), (6.0, 0.0), (0.0, 3.0)])).unwrap();
        let flat = DrawnShape::closed(points(&[(0.0, 0.0), (2.0, 2.0), (4.0, 4.0)])).unwrap();

        assert_eq!(line.center(), Point::new(3.0, 1.5));
        assert_eq!(triangle.center(), Point::new(2.0, 1.0));
        assert_eq!(flat.center(), Point::new(2.0, 2.0));
    }

    #[test]
    fn resizing_stretches_every_point_and_keeps_the_style() {
        let mut shape = rectangle();
        shape.set_line_style(LineStyle::Dashed);

        let resized = shape.resized(BoundsHandle::Right, 40.0, 0.0);

        assert_eq!(
            resized.points(),
            points(&[(10.0, 20.0), (90.0, 20.0), (90.0, 40.0), (10.0, 40.0)])
        );
        assert_eq!(resized.line_style(), LineStyle::Dashed);
        assert!(resized.is_closed());
    }

    #[test]
    fn moving_one_point_keeps_the_others() {
        let moved = rectangle().with_point(2, Point::new(70.0, 90.0));

        assert_eq!(moved.points()[2], Point::new(70.0, 90.0));
        assert_eq!(moved.points()[1], Point::new(50.0, 20.0));
        assert_eq!(rectangle().with_point(9, Point::default()), rectangle());
    }

    #[test]
    fn fill_lines_follow_the_fill_style() {
        let region = Bounds::around(Point::new(0.0, 0.0), Point::new(100.0, 100.0));
        let count = |style| {
            let mut shape = rectangle();
            shape.set_fill_style(style);
            shape.fill_lines(10.0, region).len()
        };

        assert_eq!(count(FillStyle::None), 0);
        assert_eq!(count(FillStyle::Solid), 0);
        // x = 10, 20, 30 and 40, and y = 20 and 30: a line along the right or bottom edge
        // is not inside.
        assert_eq!(count(FillStyle::VerticalLines), 4);
        assert_eq!(count(FillStyle::HorizontalLines), 2);
        assert_eq!(count(FillStyle::Crosshatch), 6);
    }

    #[test]
    fn a_line_has_no_fill_lines() {
        let mut line = DrawnShape::open(points(&[(0.0, 0.0), (50.0, 50.0)])).unwrap();
        line.set_fill_style(FillStyle::Crosshatch);
        let region = Bounds::around(Point::new(0.0, 0.0), Point::new(100.0, 100.0));

        assert!(line.fill_lines(10.0, region).is_empty());
    }

    #[test]
    fn the_fill_style_names_are_distinct() {
        let names: std::collections::HashSet<_> =
            FillStyle::ALL.iter().map(|style| style.name()).collect();

        assert_eq!(names.len(), FillStyle::ALL.len());
    }
}
