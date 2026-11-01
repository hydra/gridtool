//! Drawn shapes: arbitrary lines through two or more points, optionally joined up to
//! enclose an area. Unlike measurements nothing is measured from them, but the distance
//! between their fill lines is a real-world length, as a grid's cell size is.

use std::fmt;

use crate::color::Rgb;
use crate::geometry::{Bounds, BoundsHandle, Point};
use crate::polygon;
use crate::style::{FillAppearance, FillStyle, LineAppearance, LineStyle, Styled};
use crate::units::LengthUnit;

/// The fewest points that make a line.
pub const MIN_OPEN_POINTS: usize = 2;
/// The fewest points that enclose an area.
pub const MIN_CLOSED_POINTS: usize = 3;
pub const DEFAULT_LINE_COLOR: Rgb = Rgb::new(255, 128, 0);
pub const DEFAULT_FILL_OPACITY: f32 = 0.5;
/// The line of a new shape.
pub const DEFAULT_LINE: LineAppearance = LineAppearance::new(LineStyle::Thin, DEFAULT_LINE_COLOR);

/// The fill of a new shape, whose fill line distance is in `unit`.
pub const fn default_fill(unit: LengthUnit) -> FillAppearance {
    FillAppearance::new(FillStyle::None, DEFAULT_FILL_OPACITY, unit)
}

/// A line through two or more points, or a closed outline through three or more.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnShape {
    points: Vec<Point>,
    closed: bool,
    line: LineAppearance,
    /// Has no effect unless the shape is closed.
    fill: FillAppearance,
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
    pub fn open(points: Vec<Point>, fill_unit: LengthUnit) -> Result<Self, TooFewPoints> {
        Self::new(points, false, MIN_OPEN_POINTS, fill_unit)
    }

    /// A closed outline through `points`, in order, in the default style.
    pub fn closed(points: Vec<Point>, fill_unit: LengthUnit) -> Result<Self, TooFewPoints> {
        Self::new(points, true, MIN_CLOSED_POINTS, fill_unit)
    }

    fn new(
        points: Vec<Point>,
        closed: bool,
        min_points: usize,
        fill_unit: LengthUnit,
    ) -> Result<Self, TooFewPoints> {
        if points.len() < min_points {
            return Err(TooFewPoints(points.len()));
        }
        Ok(Self {
            points,
            closed,
            line: DEFAULT_LINE,
            fill: default_fill(fill_unit),
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

    /// The fill as stored, which an open shape has too; see [`Styled::fill`].
    pub fn stored_fill(&self) -> FillAppearance {
        self.fill
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

    /// Replaces the points; the caller keeps the count unchanged.
    pub(crate) fn set_points(&mut self, points: Vec<Point>) {
        self.points = points;
    }

    /// Replaces the fill, which an open shape keeps too (for loading a saved shape).
    pub(crate) fn set_stored_fill(&mut self, fill: FillAppearance) {
        self.fill = fill;
    }
}

/// Only a closed shape encloses an area to fill.
impl Styled for DrawnShape {
    fn line(&self) -> LineAppearance {
        self.line
    }

    fn fill(&self) -> Option<FillAppearance> {
        self.closed.then_some(self.fill)
    }

    fn line_mut(&mut self) -> &mut LineAppearance {
        &mut self.line
    }

    fn fill_mut(&mut self) -> Option<&mut FillAppearance> {
        self.closed.then_some(&mut self.fill)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::StyleEdit;

    const CM: LengthUnit = LengthUnit::Centimetre;

    fn points(coordinates: &[(f64, f64)]) -> Vec<Point> {
        coordinates.iter().map(|&(x, y)| Point::new(x, y)).collect()
    }

    /// A 40 × 20 px rectangle from (10, 20).
    fn rectangle() -> DrawnShape {
        DrawnShape::closed(
            points(&[(10.0, 20.0), (50.0, 20.0), (50.0, 40.0), (10.0, 40.0)]),
            CM,
        )
        .unwrap()
    }

    #[test]
    fn a_line_needs_two_points_and_a_closed_outline_three() {
        let two = points(&[(0.0, 0.0), (1.0, 1.0)]);

        assert_eq!(
            DrawnShape::open(vec![Point::default()], CM),
            Err(TooFewPoints(1))
        );
        assert!(!DrawnShape::open(two.clone(), CM).unwrap().is_closed());
        assert_eq!(DrawnShape::closed(two, CM), Err(TooFewPoints(2)));
        assert!(rectangle().is_closed());
    }

    #[test]
    fn a_new_shape_is_thin_and_unfilled_with_horizontal_fill_lines_in_the_given_unit() {
        let shape = DrawnShape::closed(
            points(&[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]),
            LengthUnit::Metre,
        )
        .unwrap();

        assert_eq!(shape.line().style, LineStyle::Thin);
        assert_eq!(shape.line().color, Rgb::new(255, 128, 0));
        assert_eq!(shape.line().opacity, 1.0);
        let fill = shape.fill().unwrap();
        assert_eq!(fill.style, FillStyle::None);
        assert_eq!(fill.color, None);
        assert_eq!(fill.spacing, 1.0);
        assert_eq!(fill.unit, LengthUnit::Metre);
        assert_eq!(fill.angle, 90.0);
        assert_eq!(fill.opacity, 0.5);
    }

    #[test]
    fn the_centre_of_a_closed_shape_is_its_centroid_and_of_a_line_its_bounds_centre() {
        let line = DrawnShape::open(points(&[(0.0, 0.0), (6.0, 0.0), (0.0, 3.0)]), CM).unwrap();
        let triangle =
            DrawnShape::closed(points(&[(0.0, 0.0), (6.0, 0.0), (0.0, 3.0)]), CM).unwrap();
        let flat = DrawnShape::closed(points(&[(0.0, 0.0), (2.0, 2.0), (4.0, 4.0)]), CM).unwrap();

        assert_eq!(line.center(), Point::new(3.0, 1.5));
        assert_eq!(triangle.center(), Point::new(2.0, 1.0));
        assert_eq!(flat.center(), Point::new(2.0, 2.0));
    }

    #[test]
    fn resizing_stretches_every_point_and_keeps_the_style() {
        let shape = rectangle()
            .with_edit(StyleEdit::LineStyle(LineStyle::Dashed))
            .unwrap();

        let resized = shape.resized(BoundsHandle::Right, 40.0, 0.0);

        assert_eq!(
            resized.points(),
            points(&[(10.0, 20.0), (90.0, 20.0), (90.0, 40.0), (10.0, 40.0)])
        );
        assert_eq!(resized.line().style, LineStyle::Dashed);
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
    fn a_line_has_no_fill_to_edit_but_can_change_its_line() {
        let line = DrawnShape::open(points(&[(0.0, 0.0), (50.0, 50.0)]), CM).unwrap();

        assert_eq!(line.fill(), None);
        assert_eq!(line.with_edit(StyleEdit::FillOpacity(0.25)), None);
        let edited = line.with_edit(StyleEdit::LineOpacity(0.25)).unwrap();
        assert_eq!(edited.line().opacity, 0.25);
    }

    #[test]
    fn a_closed_shape_can_change_its_fill() {
        let edited = rectangle().with_edit(StyleEdit::FillOpacity(0.25)).unwrap();

        assert_eq!(edited.fill().unwrap().opacity, 0.25);
    }
}
