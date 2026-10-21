//! Distance measurements: lines on the document whose real-world length is derived from
//! the document's scale calibration.

use std::fmt;

use crate::calibration::{Calibration, ScaleError};
use crate::geometry::{Bounds, BoundsHandle, Edge, Point};
use crate::units::LengthUnit;

/// One end of a measurement line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Endpoint {
    Start,
    End,
}

impl Endpoint {
    pub const ALL: [Endpoint; 2] = [Endpoint::Start, Endpoint::End];
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Endpoint::Start => "start",
            Endpoint::End => "end",
        })
    }
}

/// A measured line with its own display unit, independent of the calibration unit.
#[derive(Debug, Clone, PartialEq)]
pub struct Measurement {
    start: Point,
    end: Point,
    unit: LengthUnit,
}

impl Measurement {
    pub fn new(start: Point, end: Point, unit: LengthUnit) -> Self {
        Self { start, end, unit }
    }

    pub fn point(&self, endpoint: Endpoint) -> Point {
        match endpoint {
            Endpoint::Start => self.start,
            Endpoint::End => self.end,
        }
    }

    pub fn start(&self) -> Point {
        self.start
    }

    pub fn end(&self) -> Point {
        self.end
    }

    /// The unit the distance is reported in.
    pub fn unit(&self) -> LengthUnit {
        self.unit
    }

    /// Real-world length of the line in [`Self::unit`], using the calibrated scale.
    pub fn distance(&self, calibration: &Calibration) -> Result<f64, ScaleError> {
        let scale = calibration.scale()?;
        Ok(scale
            .unit
            .convert(scale.length(self.start, self.end), self.unit))
    }

    /// The smallest axis-aligned box containing the line.
    pub fn bounds(&self) -> Bounds {
        Bounds::around(self.start, self.end)
    }

    /// The measurement after dragging bounding-box `handle` by `(dx, dy)` image pixels.
    ///
    /// Along each axis, one endpoint lies on the bounding box's min edge and the other on
    /// its max edge (the start takes the min edge when they are level). Each endpoint
    /// follows the edge it lies on, so dragging a corner moves the endpoint at that corner
    /// and dragging an edge moves only that coordinate. Dragging an edge past the opposite
    /// one simply flips the line.
    pub fn resized(&self, handle: BoundsHandle, dx: f64, dy: f64) -> Measurement {
        let (moved_x, moved_y) = handle.edges();
        let start_x_edge = if self.start.x <= self.end.x {
            Edge::Min
        } else {
            Edge::Max
        };
        let start_y_edge = if self.start.y <= self.end.y {
            Edge::Min
        } else {
            Edge::Max
        };
        let follow = |value: f64, edge: Edge, moved: Option<Edge>, delta: f64| {
            if moved == Some(edge) {
                value + delta
            } else {
                value
            }
        };
        Measurement {
            start: Point::new(
                follow(self.start.x, start_x_edge, moved_x, dx),
                follow(self.start.y, start_y_edge, moved_y, dy),
            ),
            end: Point::new(
                follow(self.end.x, start_x_edge.opposite(), moved_x, dx),
                follow(self.end.y, start_y_edge.opposite(), moved_y, dy),
            ),
            unit: self.unit,
        }
    }

    pub(crate) fn set_points(&mut self, start: Point, end: Point) {
        self.start = start;
        self.end = end;
    }

    pub(crate) fn set_point(&mut self, endpoint: Endpoint, point: Point) {
        match endpoint {
            Endpoint::Start => self.start = point,
            Endpoint::End => self.end = point,
        }
    }

    pub(crate) fn set_unit(&mut self, unit: LengthUnit) {
        self.unit = unit;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::calibration::{CalibrationMethod, Line, ScalePoints};
    use crate::document::Document;

    /// A calibration of 0.5 cm/px horizontally and 0.25 cm/px vertically.
    fn calibrated() -> Calibration {
        let mut document = Document::new(500, 500);
        let points = ScalePoints::new(
            CalibrationMethod::ThreePoint,
            vec![
                Point::new(10.0, 10.0),
                Point::new(110.0, 10.0),
                Point::new(110.0, 210.0),
            ],
        )
        .unwrap();
        for action in [
            Action::SetScalePoints(points),
            Action::SetDistance {
                line: Line::A,
                distance: Some(50.0),
            },
            Action::SetDistance {
                line: Line::B,
                distance: Some(50.0),
            },
        ] {
            action.apply(&mut document).unwrap();
        }
        document.calibration().clone()
    }

    fn assert_near(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn accesses_points_by_endpoint() {
        let measurement = Measurement::new(
            Point::new(1.0, 2.0),
            Point::new(3.0, 4.0),
            LengthUnit::Metre,
        );

        assert_eq!(measurement.point(Endpoint::Start), Point::new(1.0, 2.0));
        assert_eq!(measurement.point(Endpoint::End), Point::new(3.0, 4.0));
    }

    #[test]
    fn distance_applies_the_per_axis_scale() {
        // 60 px across (30 cm) and 160 px down (40 cm) → 50 cm.
        let measurement = Measurement::new(
            Point::new(0.0, 0.0),
            Point::new(60.0, 160.0),
            LengthUnit::Centimetre,
        );

        assert_near(measurement.distance(&calibrated()).unwrap(), 50.0);
    }

    #[test]
    fn distance_is_converted_to_the_measurement_unit() {
        let measurement = Measurement::new(
            Point::new(0.0, 0.0),
            Point::new(60.0, 160.0),
            LengthUnit::Metre,
        );

        assert_near(measurement.distance(&calibrated()).unwrap(), 0.5);
    }

    #[test]
    fn distance_is_unavailable_without_a_scale() {
        let measurement = Measurement::new(
            Point::new(0.0, 0.0),
            Point::new(1.0, 1.0),
            LengthUnit::Centimetre,
        );

        assert_eq!(
            measurement.distance(&Calibration::default()),
            Err(ScaleError::MissingPoints)
        );
    }

    /// A line from bottom-left (10, 50) to top-right (40, 20).
    fn rising_line() -> Measurement {
        Measurement::new(
            Point::new(10.0, 50.0),
            Point::new(40.0, 20.0),
            LengthUnit::Centimetre,
        )
    }

    #[test]
    fn bounds_enclose_the_line() {
        let bounds = rising_line().bounds();

        assert_eq!(bounds.min, Point::new(10.0, 20.0));
        assert_eq!(bounds.max, Point::new(40.0, 50.0));
    }

    #[test]
    fn dragging_the_corner_at_an_endpoint_moves_only_that_endpoint() {
        let resized = rising_line().resized(BoundsHandle::TopRight, 5.0, -3.0);

        assert_eq!(resized.start(), Point::new(10.0, 50.0));
        assert_eq!(resized.end(), Point::new(45.0, 17.0));
    }

    #[test]
    fn dragging_an_empty_corner_moves_one_coordinate_of_each_endpoint() {
        // Top-left: the left edge holds the start's x, the top edge holds the end's y.
        let resized = rising_line().resized(BoundsHandle::TopLeft, -4.0, -6.0);

        assert_eq!(resized.start(), Point::new(6.0, 50.0));
        assert_eq!(resized.end(), Point::new(40.0, 14.0));
    }

    #[test]
    fn dragging_an_edge_moves_only_that_axis() {
        let resized = rising_line().resized(BoundsHandle::Bottom, 99.0, 7.0);

        assert_eq!(resized.start(), Point::new(10.0, 57.0));
        assert_eq!(resized.end(), Point::new(40.0, 20.0));
    }

    #[test]
    fn dragging_past_the_opposite_edge_flips_the_line() {
        let resized = rising_line().resized(BoundsHandle::Right, -50.0, 0.0);

        assert_eq!(resized.end(), Point::new(-10.0, 20.0));
        assert_eq!(resized.bounds().min.x, -10.0);
    }

    #[test]
    fn a_level_line_can_be_given_height_from_either_edge() {
        let level = Measurement::new(
            Point::new(0.0, 10.0),
            Point::new(30.0, 10.0),
            LengthUnit::Metre,
        );

        let lower_end = level.resized(BoundsHandle::Bottom, 0.0, 5.0);
        let raised_start = level.resized(BoundsHandle::Top, 0.0, -5.0);

        assert_eq!(lower_end.end(), Point::new(30.0, 15.0));
        assert_eq!(raised_start.start(), Point::new(0.0, 5.0));
        assert_eq!(lower_end.unit(), LengthUnit::Metre);
    }
}
