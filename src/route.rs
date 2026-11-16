//! Route measurements: paths through two or more points on the document, made of straight
//! segments, whose real-world length is the sum of their segments' lengths, derived from the
//! document's scale calibration.

use std::fmt;

use crate::calibration::{Calibration, ScaleError};
use crate::color::Rgb;
use crate::geometry::{Bounds, BoundsHandle, Point};
use crate::style::{FillAppearance, LineAppearance, LineStyle, Styled};
use crate::tr;
use crate::units::LengthUnit;

/// The fewest points that make a route: one segment.
pub const MIN_POINTS: usize = 2;
/// The colour of a new route.
pub const DEFAULT_COLOR: Rgb = Rgb::new(255, 0, 255);
/// The line of a new route.
pub const DEFAULT_LINE: LineAppearance = LineAppearance::new(LineStyle::Thick, DEFAULT_COLOR);

/// A path through two or more points, in order, with its own display unit and line.
#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    points: Vec<Point>,
    unit: LengthUnit,
    line: LineAppearance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooFewPoints(pub usize);

impl fmt::Display for TooFewPoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&tr!("error-route-too-few-points", {
            minimum: MIN_POINTS,
            count: self.0,
        }))
    }
}

impl std::error::Error for TooFewPoints {}

impl Route {
    /// A route through `points`, in order, reported in `unit` and drawn with the default line.
    pub fn new(points: Vec<Point>, unit: LengthUnit) -> Result<Self, TooFewPoints> {
        if points.len() < MIN_POINTS {
            return Err(TooFewPoints(points.len()));
        }
        Ok(Self {
            points,
            unit,
            line: DEFAULT_LINE,
        })
    }

    /// The route's points, in order.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// The segments, in order: each runs from one point to the next.
    pub fn segments(&self) -> impl ExactSizeIterator<Item = (Point, Point)> + '_ {
        self.points.windows(2).map(|pair| (pair[0], pair[1]))
    }

    /// The unit lengths are reported in.
    pub fn unit(&self) -> LengthUnit {
        self.unit
    }

    pub(crate) fn set_unit(&mut self, unit: LengthUnit) {
        self.unit = unit;
    }

    /// The real-world length of each segment in [`Self::unit`], using the calibrated scale.
    pub fn segment_lengths(&self, calibration: &Calibration) -> Result<Vec<f64>, ScaleError> {
        let scale = calibration.scale()?;
        Ok(self
            .segments()
            .map(|(from, to)| scale.unit.convert(scale.length(from, to), self.unit))
            .collect())
    }

    /// The real-world length of the whole route in [`Self::unit`]: the sum of its segments'.
    pub fn length(&self, calibration: &Calibration) -> Result<f64, ScaleError> {
        Ok(self.segment_lengths(calibration)?.into_iter().sum())
    }

    /// The smallest axis-aligned box containing every point.
    pub fn bounds(&self) -> Bounds {
        Bounds::enclosing(self.points.iter().copied()).expect("a route has points")
    }

    /// The route after dragging bounding-box `handle` by `(dx, dy)` image pixels: every point
    /// stretches with the box ([`Bounds::stretch`]).
    pub fn resized(&self, handle: BoundsHandle, dx: f64, dy: f64) -> Route {
        let bounds = self.bounds();
        self.map_points(|point| bounds.stretch(handle, dx, dy, point))
    }

    /// The route with point `index` moved to `to`; unchanged if there is no such point.
    pub fn with_point(&self, index: usize, to: Point) -> Route {
        let mut route = self.clone();
        if let Some(point) = route.points.get_mut(index) {
            *point = to;
        }
        route
    }

    /// The route with every point passed through `map`.
    pub fn map_points(&self, map: impl Fn(Point) -> Point) -> Route {
        let mut route = self.clone();
        route.points = self.points.iter().map(|&point| map(point)).collect();
        route
    }

    /// Replaces the points; the caller keeps the count unchanged.
    pub(crate) fn set_points(&mut self, points: Vec<Point>) {
        self.points = points;
    }
}

/// A route is a line, so it has no fill.
impl Styled for Route {
    fn line(&self) -> LineAppearance {
        self.line
    }

    fn fill(&self) -> Option<FillAppearance> {
        None
    }

    fn line_mut(&mut self) -> &mut LineAppearance {
        &mut self.line
    }

    fn fill_mut(&mut self) -> Option<&mut FillAppearance> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::{CalibrationMethod, Line, ScalePoints};

    const CM: LengthUnit = LengthUnit::Centimetre;

    fn points(coordinates: &[(f64, f64)]) -> Vec<Point> {
        coordinates.iter().map(|&(x, y)| Point::new(x, y)).collect()
    }

    /// Three segments of 30, 40 and 50 px.
    fn route() -> Route {
        Route::new(
            points(&[(0.0, 0.0), (30.0, 0.0), (30.0, 40.0), (0.0, 0.0)]),
            CM,
        )
        .unwrap()
    }

    /// 2 px to the centimetre on both axes.
    fn calibration() -> Calibration {
        let mut calibration = Calibration::default();
        calibration.set_points(
            ScalePoints::new(
                CalibrationMethod::TwoPoint,
                points(&[(0.0, 0.0), (100.0, 0.0)]),
            )
            .unwrap(),
        );
        calibration.set_distance(Line::A, Some(50.0));
        calibration.set_unit(CM);
        calibration
    }

    #[test]
    fn a_route_needs_at_least_one_segment() {
        assert_eq!(Route::new(points(&[(1.0, 1.0)]), CM), Err(TooFewPoints(1)));
        assert_eq!(Route::new(Vec::new(), CM), Err(TooFewPoints(0)));
        assert!(Route::new(points(&[(1.0, 1.0), (2.0, 2.0)]), CM).is_ok());
    }

    #[test]
    fn a_new_route_has_a_thick_magenta_line_and_no_fill() {
        let route = route();

        assert_eq!(route.line(), DEFAULT_LINE);
        assert_eq!(route.line().style, LineStyle::Thick);
        assert_eq!(route.fill(), None);
        assert_eq!(route.unit(), CM);
    }

    #[test]
    fn segments_join_each_point_to_the_next() {
        let route = route();

        let segments: Vec<_> = route.segments().collect();

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[1], (Point::new(30.0, 0.0), Point::new(30.0, 40.0)));
    }

    #[test]
    fn the_length_is_the_sum_of_the_segment_lengths_in_the_routes_unit() {
        let mut route = route();
        let calibration = calibration();

        assert_eq!(
            route.segment_lengths(&calibration).unwrap(),
            vec![15.0, 20.0, 25.0]
        );
        assert_eq!(route.length(&calibration).unwrap(), 60.0);

        route.set_unit(LengthUnit::Metre);
        assert!((route.length(&calibration).unwrap() - 0.6).abs() < 1e-12);
    }

    #[test]
    fn without_a_scale_there_is_no_length() {
        assert!(route().length(&Calibration::default()).is_err());
        assert!(route().segment_lengths(&Calibration::default()).is_err());
    }

    #[test]
    fn editing_points_keeps_everything_else() {
        let route = route();

        let moved = route.with_point(1, Point::new(31.0, 2.0));
        assert_eq!(moved.points()[1], Point::new(31.0, 2.0));
        assert_eq!(moved.points()[2], route.points()[2]);
        assert_eq!(route.with_point(9, Point::new(5.0, 5.0)), route);

        let shifted = route.map_points(|p| Point::new(p.x + 1.0, p.y));
        assert_eq!(shifted.points()[3], Point::new(1.0, 0.0));
        assert_eq!(shifted.unit(), route.unit());
    }

    #[test]
    fn resizing_stretches_every_point_with_the_box() {
        let route = route();
        assert_eq!(
            route.bounds(),
            Bounds::around(Point::new(0.0, 0.0), Point::new(30.0, 40.0))
        );

        let wider = route.resized(BoundsHandle::Right, 30.0, 0.0);

        assert_eq!(wider.points()[1], Point::new(60.0, 0.0));
        assert_eq!(wider.points()[0], Point::new(0.0, 0.0));
    }
}
