//! Distance measurements: lines on the document whose real-world length is derived from
//! the document's scale calibration.

use std::fmt;

use crate::calibration::{Calibration, ScaleError};
use crate::geometry::Point;
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
}
