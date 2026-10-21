//! Every change that can be made to a [`Document`]. Each action is a self-contained
//! value, so it can be recorded for undo/redo, shown in a history list, or created by a
//! non-GUI front end.

use std::fmt;

use crate::calibration::{CalibrationError, Line, ScalePoints};
use crate::document::Document;
use crate::geometry::Point;
use crate::measurement::{Endpoint, Measurement};
use crate::units::LengthUnit;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Places a complete set of calibration points, replacing any existing ones.
    SetScalePoints(ScalePoints),
    /// Moves one existing calibration point.
    MoveScalePoint { index: usize, to: Point },
    /// Sets (or clears, with `None`) the real-world length of a reference line.
    SetDistance { line: Line, distance: Option<f64> },
    /// Changes the unit the reference distances are measured in.
    SetUnit(LengthUnit),
    /// Appends a measurement to the end of the list.
    AddMeasurement(Measurement),
    /// Moves one end of an existing measurement.
    MoveMeasurementPoint {
        index: usize,
        endpoint: Endpoint,
        to: Point,
    },
    /// Changes the unit a measurement's distance is reported in.
    SetMeasurementUnit { index: usize, unit: LengthUnit },
    /// Removes a measurement; later measurements move up one place.
    DeleteMeasurement { index: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionError {
    PointOutsideDocument(Point),
    NoScalePoints,
    InvalidDistance(f64),
    MeasurementIndexOutOfRange { index: usize, count: usize },
    Calibration(CalibrationError),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PointOutsideDocument(point) => write!(f, "point {point} is outside the document"),
            Self::NoScalePoints => f.write_str("there are no scale points to move"),
            Self::InvalidDistance(distance) => {
                write!(f, "distance must be a positive number, got {distance}")
            }
            Self::MeasurementIndexOutOfRange { index, count } => write!(
                f,
                "measurement {} does not exist (there are {count})",
                index + 1
            ),
            Self::Calibration(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ActionError {}

impl From<CalibrationError> for ActionError {
    fn from(error: CalibrationError) -> Self {
        Self::Calibration(error)
    }
}

impl Action {
    /// Applies the action to `document`. On error the document is left unchanged.
    pub fn apply(&self, document: &mut Document) -> Result<(), ActionError> {
        match self {
            Action::SetScalePoints(points) => {
                for &point in points.points() {
                    ensure_inside(document, point)?;
                }
                document.calibration_mut().set_points(points.clone());
            }
            Action::MoveScalePoint { index, to } => {
                ensure_inside(document, *to)?;
                document
                    .calibration_mut()
                    .points_mut()
                    .ok_or(ActionError::NoScalePoints)?
                    .set_point(*index, *to)?;
            }
            Action::SetDistance { line, distance } => {
                if let Some(distance) = *distance
                    && !(distance.is_finite() && distance > 0.0)
                {
                    return Err(ActionError::InvalidDistance(distance));
                }
                document.calibration_mut().set_distance(*line, *distance);
            }
            Action::SetUnit(unit) => document.calibration_mut().set_unit(*unit),
            Action::AddMeasurement(measurement) => {
                for endpoint in Endpoint::ALL {
                    ensure_inside(document, measurement.point(endpoint))?;
                }
                document.measurements_mut().push(measurement.clone());
            }
            Action::MoveMeasurementPoint {
                index,
                endpoint,
                to,
            } => {
                ensure_inside(document, *to)?;
                measurement_mut(document, *index)?.set_point(*endpoint, *to);
            }
            Action::SetMeasurementUnit { index, unit } => {
                measurement_mut(document, *index)?.set_unit(*unit);
            }
            Action::DeleteMeasurement { index } => {
                measurement_mut(document, *index)?;
                document.measurements_mut().remove(*index);
            }
        }
        Ok(())
    }
}

fn ensure_inside(document: &Document, point: Point) -> Result<(), ActionError> {
    if document.contains(point) {
        Ok(())
    } else {
        Err(ActionError::PointOutsideDocument(point))
    }
}

fn measurement_mut(document: &mut Document, index: usize) -> Result<&mut Measurement, ActionError> {
    let measurements = document.measurements_mut();
    let count = measurements.len();
    measurements
        .get_mut(index)
        .ok_or(ActionError::MeasurementIndexOutOfRange { index, count })
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::SetScalePoints(points) => write!(f, "Set {} scale points", points.method()),
            Action::MoveScalePoint { index, to } => {
                write!(f, "Move scale point {} to {to}", index + 1)
            }
            Action::SetDistance {
                line,
                distance: Some(distance),
            } => write!(f, "Set distance {line} to {distance}"),
            Action::SetDistance {
                line,
                distance: None,
            } => write!(f, "Clear distance {line}"),
            Action::SetUnit(unit) => write!(f, "Set units to {unit}"),
            Action::AddMeasurement(measurement) => write!(
                f,
                "Add measurement from {} to {}",
                measurement.start(),
                measurement.end()
            ),
            Action::MoveMeasurementPoint {
                index,
                endpoint,
                to,
            } => write!(f, "Move measurement {} {endpoint} to {to}", index + 1),
            Action::SetMeasurementUnit { index, unit } => {
                write!(f, "Set measurement {} units to {unit}", index + 1)
            }
            Action::DeleteMeasurement { index } => {
                write!(f, "Delete measurement {}", index + 1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calibration::CalibrationMethod;

    fn three_points() -> ScalePoints {
        ScalePoints::new(
            CalibrationMethod::ThreePoint,
            vec![
                Point::new(10.0, 10.0),
                Point::new(90.0, 10.0),
                Point::new(90.0, 40.0),
            ],
        )
        .unwrap()
    }

    fn document_with_points() -> Document {
        let mut document = Document::new(100, 50);
        Action::SetScalePoints(three_points())
            .apply(&mut document)
            .unwrap();
        document
    }

    #[test]
    fn sets_scale_points() {
        let document = document_with_points();

        assert_eq!(document.calibration().points(), Some(&three_points()));
    }

    #[test]
    fn rejects_scale_points_outside_the_document() {
        let mut document = Document::new(10, 10);
        let before = document.clone();

        let result = Action::SetScalePoints(three_points()).apply(&mut document);

        assert!(matches!(result, Err(ActionError::PointOutsideDocument(_))));
        assert_eq!(document, before);
    }

    #[test]
    fn moves_a_scale_point() {
        let mut document = document_with_points();

        Action::MoveScalePoint {
            index: 2,
            to: Point::new(50.0, 45.0),
        }
        .apply(&mut document)
        .unwrap();

        let points = document.calibration().points().unwrap().points();
        assert_eq!(points[2], Point::new(50.0, 45.0));
    }

    #[test]
    fn moving_requires_existing_points_and_a_valid_index() {
        let mut empty = Document::new(100, 50);
        let move_to_origin = |index| Action::MoveScalePoint {
            index,
            to: Point::default(),
        };

        assert_eq!(
            move_to_origin(0).apply(&mut empty),
            Err(ActionError::NoScalePoints)
        );

        let mut document = document_with_points();
        assert!(matches!(
            move_to_origin(3).apply(&mut document),
            Err(ActionError::Calibration(
                CalibrationError::PointIndexOutOfRange { .. }
            ))
        ));
    }

    #[test]
    fn sets_and_clears_distances() {
        let mut document = Document::new(100, 50);

        Action::SetDistance {
            line: Line::B,
            distance: Some(12.5),
        }
        .apply(&mut document)
        .unwrap();
        assert_eq!(document.calibration().distance(Line::B), Some(12.5));

        Action::SetDistance {
            line: Line::B,
            distance: None,
        }
        .apply(&mut document)
        .unwrap();
        assert_eq!(document.calibration().distance(Line::B), None);
    }

    #[test]
    fn rejects_non_positive_distances() {
        let mut document = Document::new(100, 50);

        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            let action = Action::SetDistance {
                line: Line::A,
                distance: Some(invalid),
            };
            assert!(
                action.apply(&mut document).is_err(),
                "{invalid} should be rejected"
            );
        }
    }

    #[test]
    fn sets_the_unit() {
        let mut document = Document::new(100, 50);

        Action::SetUnit(LengthUnit::Metre)
            .apply(&mut document)
            .unwrap();

        assert_eq!(document.calibration().unit(), LengthUnit::Metre);
    }

    #[test]
    fn describes_itself_for_the_history_list() {
        assert_eq!(
            Action::SetScalePoints(three_points()).to_string(),
            "Set 3-point scale points"
        );
        assert_eq!(
            Action::MoveScalePoint {
                index: 0,
                to: Point::new(1.0, 2.0)
            }
            .to_string(),
            "Move scale point 1 to (1.0, 2.0)"
        );
        assert_eq!(
            Action::SetDistance {
                line: Line::A,
                distance: Some(50.0)
            }
            .to_string(),
            "Set distance A to 50"
        );
        assert_eq!(
            Action::SetUnit(LengthUnit::Metre).to_string(),
            "Set units to m"
        );
        assert_eq!(
            Action::AddMeasurement(measurement(1.0, 9.0)).to_string(),
            "Add measurement from (1.0, 1.0) to (9.0, 9.0)"
        );
        assert_eq!(
            Action::MoveMeasurementPoint {
                index: 1,
                endpoint: Endpoint::End,
                to: Point::new(3.0, 4.0)
            }
            .to_string(),
            "Move measurement 2 end to (3.0, 4.0)"
        );
        assert_eq!(
            Action::SetMeasurementUnit {
                index: 0,
                unit: LengthUnit::Metre
            }
            .to_string(),
            "Set measurement 1 units to m"
        );
        assert_eq!(
            Action::DeleteMeasurement { index: 2 }.to_string(),
            "Delete measurement 3"
        );
    }

    fn measurement(from: f64, to: f64) -> Measurement {
        Measurement::new(
            Point::new(from, from),
            Point::new(to, to),
            LengthUnit::Centimetre,
        )
    }

    fn document_with_measurements(count: usize) -> Document {
        let mut document = Document::new(100, 50);
        for n in 0..count {
            Action::AddMeasurement(measurement(n as f64, n as f64 + 10.0))
                .apply(&mut document)
                .unwrap();
        }
        document
    }

    #[test]
    fn adds_measurements_in_order() {
        let document = document_with_measurements(3);

        let starts: Vec<_> = document
            .measurements()
            .iter()
            .map(|m| m.start().x)
            .collect();
        assert_eq!(starts, [0.0, 1.0, 2.0]);
    }

    #[test]
    fn rejects_measurements_outside_the_document() {
        let mut document = Document::new(100, 50);

        let result = Action::AddMeasurement(measurement(0.0, 60.0)).apply(&mut document);

        assert!(matches!(result, Err(ActionError::PointOutsideDocument(_))));
        assert!(document.measurements().is_empty());
    }

    #[test]
    fn moves_a_measurement_endpoint() {
        let mut document = document_with_measurements(2);

        Action::MoveMeasurementPoint {
            index: 1,
            endpoint: Endpoint::Start,
            to: Point::new(40.0, 20.0),
        }
        .apply(&mut document)
        .unwrap();

        let moved = &document.measurements()[1];
        assert_eq!(moved.start(), Point::new(40.0, 20.0));
        assert_eq!(moved.end(), Point::new(11.0, 11.0));
    }

    #[test]
    fn rejects_moving_a_measurement_endpoint_outside_the_document() {
        let mut document = document_with_measurements(1);
        let before = document.clone();

        let result = Action::MoveMeasurementPoint {
            index: 0,
            endpoint: Endpoint::End,
            to: Point::new(101.0, 0.0),
        }
        .apply(&mut document);

        assert!(result.is_err());
        assert_eq!(document, before);
    }

    #[test]
    fn sets_a_measurement_unit() {
        let mut document = document_with_measurements(2);

        Action::SetMeasurementUnit {
            index: 0,
            unit: LengthUnit::Metre,
        }
        .apply(&mut document)
        .unwrap();

        assert_eq!(document.measurements()[0].unit(), LengthUnit::Metre);
        assert_eq!(document.measurements()[1].unit(), LengthUnit::Centimetre);
        assert_eq!(document.calibration().unit(), LengthUnit::Centimetre);
    }

    #[test]
    fn deletes_a_measurement_keeping_the_order_of_the_rest() {
        let mut document = document_with_measurements(3);

        Action::DeleteMeasurement { index: 1 }
            .apply(&mut document)
            .unwrap();

        let starts: Vec<_> = document
            .measurements()
            .iter()
            .map(|m| m.start().x)
            .collect();
        assert_eq!(starts, [0.0, 2.0]);
    }

    #[test]
    fn measurement_actions_reject_missing_indices() {
        let mut document = document_with_measurements(2);
        let before = document.clone();
        let out_of_range = ActionError::MeasurementIndexOutOfRange { index: 2, count: 2 };

        for action in [
            Action::MoveMeasurementPoint {
                index: 2,
                endpoint: Endpoint::Start,
                to: Point::default(),
            },
            Action::SetMeasurementUnit {
                index: 2,
                unit: LengthUnit::Metre,
            },
            Action::DeleteMeasurement { index: 2 },
        ] {
            assert_eq!(action.apply(&mut document), Err(out_of_range.clone()));
        }
        assert_eq!(document, before);
    }
}
