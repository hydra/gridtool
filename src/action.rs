//! Every change that can be made to a [`Document`]. Each action is a self-contained
//! value, so it can be recorded for undo/redo, shown in a history list, or created by a
//! non-GUI front end.

use std::fmt;

use crate::calibration::{CalibrationError, Line, ScalePoints};
use crate::document::Document;
use crate::geometry::Point;
use crate::grid::{CellDimension, CellShape, Grid, GridSource, Rgb};
use crate::measurement::{Endpoint, Measurement};
use crate::object::{Identified, ObjectId};
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
    /// Moves the image's origin (the point the view rotates about).
    SetOrigin(Point),
    /// Appends a measurement to the end of the list.
    AddMeasurement(Measurement),
    /// Moves one end of an existing measurement.
    MoveMeasurementPoint {
        index: usize,
        endpoint: Endpoint,
        to: Point,
    },
    /// Replaces both ends of a measurement at once (e.g. after resizing its bounding box).
    ResizeMeasurement {
        index: usize,
        start: Point,
        end: Point,
    },
    /// Changes the unit a measurement's distance is reported in.
    SetMeasurementUnit { index: usize, unit: LengthUnit },
    /// Removes a measurement; later measurements move up one place.
    DeleteMeasurement { index: usize },
    /// Appends an exact copy of a measurement, with a new object id, to the end of the list.
    DuplicateMeasurement { index: usize },
    /// Appends a grid to the end of the list.
    AddGrid(Grid),
    /// Appends an exact copy of a grid, with a new object id, to the end of the list.
    DuplicateGrid { index: usize },
    /// Removes a grid; later grids move up one place.
    DeleteGrid { index: usize },
    /// Changes a grid's cell shape.
    SetGridShape { index: usize, shape: CellShape },
    /// Changes one of a grid's cell sizes (in the grid's unit).
    SetGridSize {
        index: usize,
        dimension: CellDimension,
        size: f64,
    },
    /// Changes the unit a grid's cell sizes are in, keeping the size values.
    SetGridUnit { index: usize, unit: LengthUnit },
    /// Changes a grid's line colour.
    SetGridColor { index: usize, color: Rgb },
    /// Changes a grid's opacity (0 to 1).
    SetGridOpacity { index: usize, opacity: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionError {
    PointOutsideDocument(Point),
    NoScalePoints,
    InvalidDistance(f64),
    MeasurementIndexOutOfRange { index: usize, count: usize },
    GridIndexOutOfRange { index: usize, count: usize },
    GridSourceMissing,
    InvalidCellSize(f64),
    InvalidOpacity(f32),
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
            Self::GridIndexOutOfRange { index, count } => {
                write!(f, "grid {} does not exist (there are {count})", index + 1)
            }
            Self::GridSourceMissing => f.write_str("the grid's alignment line does not exist"),
            Self::InvalidCellSize(size) => {
                write!(f, "cell size must be a positive number, got {size}")
            }
            Self::InvalidOpacity(opacity) => {
                write!(f, "opacity must be between 0 and 1, got {opacity}")
            }
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
            Action::SetOrigin(origin) => {
                ensure_inside(document, *origin)?;
                document.set_origin(*origin);
            }
            Action::AddMeasurement(measurement) => {
                for endpoint in Endpoint::ALL {
                    ensure_inside(document, measurement.point(endpoint))?;
                }
                let id = document.allocate_object_id();
                document
                    .measurements_mut()
                    .push(Identified::new(id, measurement.clone()));
            }
            Action::ResizeMeasurement { index, start, end } => {
                ensure_inside(document, *start)?;
                ensure_inside(document, *end)?;
                measurement_mut(document, *index)?.set_points(*start, *end);
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
            Action::DuplicateMeasurement { index } => {
                let copy = measurement_mut(document, *index)?.clone();
                let id = document.allocate_object_id();
                document.measurements_mut().push(Identified::new(id, copy));
            }
            Action::AddGrid(grid) => {
                if grid.source_line(document).is_err() {
                    return Err(ActionError::GridSourceMissing);
                }
                for dimension in [CellDimension::A, CellDimension::B] {
                    ensure_valid_cell_size(grid.size(dimension))?;
                }
                ensure_valid_opacity(grid.opacity())?;
                let id = document.allocate_object_id();
                document.grids_mut().push(Identified::new(id, grid.clone()));
            }
            Action::DuplicateGrid { index } => {
                let copy = grid_mut(document, *index)?.clone();
                let id = document.allocate_object_id();
                document.grids_mut().push(Identified::new(id, copy));
            }
            Action::DeleteGrid { index } => {
                grid_mut(document, *index)?;
                document.grids_mut().remove(*index);
            }
            Action::SetGridShape { index, shape } => grid_mut(document, *index)?.set_shape(*shape),
            Action::SetGridSize {
                index,
                dimension,
                size,
            } => {
                ensure_valid_cell_size(*size)?;
                grid_mut(document, *index)?.set_size(*dimension, *size);
            }
            Action::SetGridUnit { index, unit } => grid_mut(document, *index)?.set_unit(*unit),
            Action::SetGridColor { index, color } => grid_mut(document, *index)?.set_color(*color),
            Action::SetGridOpacity { index, opacity } => {
                ensure_valid_opacity(*opacity)?;
                grid_mut(document, *index)?.set_opacity(*opacity);
            }
        }
        Ok(())
    }

    /// The action that duplicates the object with `id`, or `None` if there is no such
    /// object or it cannot be duplicated.
    pub fn duplicate_object(document: &Document, id: ObjectId) -> Option<Action> {
        if let Some(index) = document.measurement_index(id) {
            return Some(Action::DuplicateMeasurement { index });
        }
        document
            .grid_index(id)
            .map(|index| Action::DuplicateGrid { index })
    }

    /// The action that deletes the object with `id`, or `None` if there is no such object
    /// or it cannot be deleted.
    pub fn delete_object(document: &Document, id: ObjectId) -> Option<Action> {
        if let Some(index) = document.measurement_index(id) {
            return Some(Action::DeleteMeasurement { index });
        }
        document
            .grid_index(id)
            .map(|index| Action::DeleteGrid { index })
    }

    /// Whether the object this action adds should become the active object.
    pub fn activates_new_object(&self) -> bool {
        matches!(
            self,
            Action::AddGrid(_) | Action::DuplicateMeasurement { .. } | Action::DuplicateGrid { .. }
        )
    }
}

fn ensure_valid_cell_size(size: f64) -> Result<(), ActionError> {
    if size.is_finite() && size > 0.0 {
        Ok(())
    } else {
        Err(ActionError::InvalidCellSize(size))
    }
}

fn ensure_valid_opacity(opacity: f32) -> Result<(), ActionError> {
    if (0.0..=1.0).contains(&opacity) {
        Ok(())
    } else {
        Err(ActionError::InvalidOpacity(opacity))
    }
}

fn grid_mut(document: &mut Document, index: usize) -> Result<&mut Grid, ActionError> {
    let grids = document.grids_mut();
    let count = grids.len();
    grids
        .get_mut(index)
        .map(|grid| &mut **grid)
        .ok_or(ActionError::GridIndexOutOfRange { index, count })
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
        .map(|measurement| &mut **measurement)
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
            Action::SetOrigin(origin) => write!(f, "Set origin to {origin}"),
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
            Action::ResizeMeasurement { index, start, end } => {
                write!(f, "Resize measurement {} to {start} - {end}", index + 1)
            }
            Action::SetMeasurementUnit { index, unit } => {
                write!(f, "Set measurement {} units to {unit}", index + 1)
            }
            Action::DeleteMeasurement { index } => {
                write!(f, "Delete measurement {}", index + 1)
            }
            Action::DuplicateMeasurement { index } => {
                write!(f, "Duplicate measurement {}", index + 1)
            }
            Action::DuplicateGrid { index } => write!(f, "Duplicate grid {}", index + 1),
            Action::DeleteGrid { index } => write!(f, "Delete grid {}", index + 1),
            Action::AddGrid(grid) => match grid.source() {
                GridSource::ScaleLine(line) => write!(f, "Add grid aligned to scale line {line}"),
                GridSource::Measurement(_) => f.write_str("Add grid aligned to a measurement"),
            },
            Action::SetGridShape { index, shape } => {
                write!(f, "Set grid {} cell shape to {}", index + 1, shape.name())
            }
            Action::SetGridSize {
                index,
                dimension,
                size,
            } => write!(f, "Set grid {} size {dimension} to {size}", index + 1),
            Action::SetGridUnit { index, unit } => {
                write!(f, "Set grid {} units to {unit}", index + 1)
            }
            Action::SetGridColor { index, color } => {
                write!(f, "Set grid {} color to {color}", index + 1)
            }
            Action::SetGridOpacity { index, opacity } => write!(
                f,
                "Set grid {} opacity to {}%",
                index + 1,
                (opacity * 100.0).round()
            ),
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
    fn sets_the_origin_inside_the_document_only() {
        let mut document = Document::new(100, 50);

        Action::SetOrigin(Point::new(10.0, 40.0))
            .apply(&mut document)
            .unwrap();
        assert_eq!(document.origin(), Point::new(10.0, 40.0));

        let before = document.clone();
        assert!(matches!(
            Action::SetOrigin(Point::new(100.5, 10.0)).apply(&mut document),
            Err(ActionError::PointOutsideDocument(_))
        ));
        assert_eq!(document, before);
        assert_eq!(
            Action::SetOrigin(Point::new(12.0, 3.5)).to_string(),
            "Set origin to (12.0, 3.5)"
        );
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
    fn resizes_a_measurement_keeping_its_id_and_unit() {
        let mut document = document_with_measurements(2);
        let id = document.measurements()[1].id();

        Action::ResizeMeasurement {
            index: 1,
            start: Point::new(5.0, 6.0),
            end: Point::new(70.0, 40.0),
        }
        .apply(&mut document)
        .unwrap();

        let resized = &document.measurements()[1];
        assert_eq!(resized.id(), id);
        assert_eq!(resized.start(), Point::new(5.0, 6.0));
        assert_eq!(resized.end(), Point::new(70.0, 40.0));
        assert_eq!(resized.unit(), LengthUnit::Centimetre);
    }

    #[test]
    fn rejects_resizing_outside_the_document_or_a_missing_measurement() {
        let mut document = document_with_measurements(1);
        let before = document.clone();
        let resize = |index, end| Action::ResizeMeasurement {
            index,
            start: Point::new(1.0, 1.0),
            end,
        };

        assert!(
            resize(0, Point::new(1.0, 51.0))
                .apply(&mut document)
                .is_err()
        );
        assert!(
            resize(1, Point::new(2.0, 2.0))
                .apply(&mut document)
                .is_err()
        );
        assert_eq!(document, before);
    }

    #[test]
    fn describes_a_resize() {
        let action = Action::ResizeMeasurement {
            index: 0,
            start: Point::new(1.0, 2.0),
            end: Point::new(3.0, 4.0),
        };

        assert_eq!(
            action.to_string(),
            "Resize measurement 1 to (1.0, 2.0) - (3.0, 4.0)"
        );
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

    fn document_with_grid() -> Document {
        let mut document = document_with_points();
        Action::AddGrid(Grid::new(
            GridSource::ScaleLine(Line::A),
            LengthUnit::Centimetre,
        ))
        .apply(&mut document)
        .unwrap();
        document
    }

    #[test]
    fn adds_a_grid_with_its_own_object_id() {
        let mut document = document_with_grid();
        Action::AddMeasurement(measurement(1.0, 2.0))
            .apply(&mut document)
            .unwrap();

        let grid_id = document.grids()[0].id();
        assert_ne!(grid_id, document.measurements()[0].id());
        assert!(document.contains_object(grid_id));
        assert_eq!(document.grid_index(grid_id), Some(0));
    }

    #[test]
    fn rejects_a_grid_whose_source_does_not_exist() {
        let mut document = Document::new(100, 50);

        let result = Action::AddGrid(Grid::new(
            GridSource::ScaleLine(Line::B),
            LengthUnit::Centimetre,
        ))
        .apply(&mut document);

        assert_eq!(result, Err(ActionError::GridSourceMissing));
        assert!(document.grids().is_empty());
    }

    #[test]
    fn edits_grid_properties() {
        let mut document = document_with_grid();

        for action in [
            Action::SetGridShape {
                index: 0,
                shape: CellShape::Rectangle,
            },
            Action::SetGridSize {
                index: 0,
                dimension: CellDimension::B,
                size: 4.5,
            },
            Action::SetGridColor {
                index: 0,
                color: Rgb::new(1, 2, 3),
            },
            Action::SetGridOpacity {
                index: 0,
                opacity: 0.25,
            },
        ] {
            action.apply(&mut document).unwrap();
        }

        let grid = &document.grids()[0];
        assert_eq!(grid.shape(), CellShape::Rectangle);
        assert_eq!(grid.cell_size(), (10.0, 4.5));
        assert_eq!(grid.color(), Rgb::new(1, 2, 3));
        assert_eq!(grid.opacity(), 0.25);
    }

    #[test]
    fn rejects_invalid_grid_edits() {
        let mut document = document_with_grid();
        let before = document.clone();

        for action in [
            Action::SetGridSize {
                index: 0,
                dimension: CellDimension::Side,
                size: 0.0,
            },
            Action::SetGridSize {
                index: 0,
                dimension: CellDimension::A,
                size: f64::NAN,
            },
            Action::SetGridOpacity {
                index: 0,
                opacity: 1.5,
            },
            Action::SetGridShape {
                index: 1,
                shape: CellShape::Square,
            },
        ] {
            assert!(action.apply(&mut document).is_err(), "{action:?}");
        }
        assert_eq!(document, before);
    }

    #[test]
    fn sets_a_grid_unit_keeping_the_size_values() {
        let mut document = document_with_grid();

        Action::SetGridUnit {
            index: 0,
            unit: LengthUnit::Metre,
        }
        .apply(&mut document)
        .unwrap();

        let grid = &document.grids()[0];
        assert_eq!(grid.unit(), LengthUnit::Metre);
        assert_eq!(grid.cell_size(), (10.0, 10.0));
        assert_eq!(document.calibration().unit(), LengthUnit::Centimetre);
    }

    #[test]
    fn rejects_setting_the_unit_of_a_missing_grid() {
        let mut document = document_with_grid();
        let before = document.clone();

        let result = Action::SetGridUnit {
            index: 1,
            unit: LengthUnit::Metre,
        }
        .apply(&mut document);

        assert_eq!(
            result,
            Err(ActionError::GridIndexOutOfRange { index: 1, count: 1 })
        );
        assert_eq!(document, before);
    }

    #[test]
    fn duplicates_a_measurement_with_a_new_id_and_the_same_properties() {
        let mut document = document_with_measurements(2);
        Action::SetMeasurementUnit {
            index: 0,
            unit: LengthUnit::Metre,
        }
        .apply(&mut document)
        .unwrap();

        Action::DuplicateMeasurement { index: 0 }
            .apply(&mut document)
            .unwrap();

        let [original, _, copy] = document.measurements() else {
            panic!("expected three measurements");
        };
        assert_eq!(**copy, **original);
        assert_ne!(copy.id(), original.id());
        assert_eq!(document.newest_object(), Some(copy.id()));
    }

    #[test]
    fn duplicates_a_grid_with_a_new_id_and_the_same_properties() {
        let mut document = document_with_grid();
        for action in [
            Action::SetGridShape {
                index: 0,
                shape: CellShape::Rectangle,
            },
            Action::SetGridSize {
                index: 0,
                dimension: CellDimension::B,
                size: 4.5,
            },
            Action::SetGridColor {
                index: 0,
                color: Rgb::new(1, 2, 3),
            },
            Action::SetGridOpacity {
                index: 0,
                opacity: 0.25,
            },
            Action::DuplicateGrid { index: 0 },
        ] {
            action.apply(&mut document).unwrap();
        }

        let [original, copy] = document.grids() else {
            panic!("expected two grids");
        };
        assert_eq!(**copy, **original);
        assert_ne!(copy.id(), original.id());
        assert_eq!(document.newest_object(), Some(copy.id()));
    }

    #[test]
    fn deletes_a_grid_keeping_the_order_of_the_rest() {
        let mut document = document_with_grid();
        for _ in 0..2 {
            Action::DuplicateGrid { index: 0 }
                .apply(&mut document)
                .unwrap();
        }
        let ids: Vec<_> = document.grids().iter().map(|grid| grid.id()).collect();

        Action::DeleteGrid { index: 1 }
            .apply(&mut document)
            .unwrap();

        let remaining: Vec<_> = document.grids().iter().map(|grid| grid.id()).collect();
        assert_eq!(remaining, [ids[0], ids[2]]);
    }

    #[test]
    fn duplicate_and_delete_reject_missing_indices() {
        let mut document = document_with_grid();
        Action::AddMeasurement(measurement(1.0, 2.0))
            .apply(&mut document)
            .unwrap();
        let before = document.clone();

        assert_eq!(
            Action::DuplicateMeasurement { index: 1 }.apply(&mut document),
            Err(ActionError::MeasurementIndexOutOfRange { index: 1, count: 1 })
        );
        for action in [
            Action::DuplicateGrid { index: 1 },
            Action::DeleteGrid { index: 1 },
        ] {
            assert_eq!(
                action.apply(&mut document),
                Err(ActionError::GridIndexOutOfRange { index: 1, count: 1 })
            );
        }
        assert_eq!(document, before);
    }

    #[test]
    fn finds_the_duplicate_and_delete_actions_for_an_object() {
        let mut document = document_with_grid();
        Action::AddMeasurement(measurement(1.0, 2.0))
            .apply(&mut document)
            .unwrap();
        let grid = document.grids()[0].id();
        let measurement = document.measurements()[0].id();

        assert_eq!(
            Action::duplicate_object(&document, measurement),
            Some(Action::DuplicateMeasurement { index: 0 })
        );
        assert_eq!(
            Action::delete_object(&document, measurement),
            Some(Action::DeleteMeasurement { index: 0 })
        );
        assert_eq!(
            Action::duplicate_object(&document, grid),
            Some(Action::DuplicateGrid { index: 0 })
        );
        assert_eq!(
            Action::delete_object(&document, grid),
            Some(Action::DeleteGrid { index: 0 })
        );

        Action::DeleteGrid { index: 0 }
            .apply(&mut document)
            .unwrap();
        assert_eq!(Action::duplicate_object(&document, grid), None);
        assert_eq!(Action::delete_object(&document, grid), None);
    }

    #[test]
    fn only_adding_grids_and_duplicating_activate_the_new_object() {
        assert!(Action::DuplicateMeasurement { index: 0 }.activates_new_object());
        assert!(Action::DuplicateGrid { index: 0 }.activates_new_object());
        assert!(
            Action::AddGrid(Grid::new(
                GridSource::ScaleLine(Line::A),
                LengthUnit::Centimetre
            ))
            .activates_new_object()
        );
        assert!(!Action::AddMeasurement(measurement(1.0, 2.0)).activates_new_object());
        assert!(!Action::DeleteGrid { index: 0 }.activates_new_object());
    }

    #[test]
    fn describes_duplicate_and_delete_actions() {
        assert_eq!(
            Action::DuplicateMeasurement { index: 0 }.to_string(),
            "Duplicate measurement 1"
        );
        assert_eq!(
            Action::DuplicateGrid { index: 1 }.to_string(),
            "Duplicate grid 2"
        );
        assert_eq!(Action::DeleteGrid { index: 2 }.to_string(), "Delete grid 3");
    }

    #[test]
    fn describes_grid_actions() {
        let labels = [
            (
                Action::AddGrid(Grid::new(
                    GridSource::ScaleLine(Line::A),
                    LengthUnit::Centimetre,
                )),
                "Add grid aligned to scale line A",
            ),
            (
                Action::SetGridShape {
                    index: 0,
                    shape: CellShape::Rectangle,
                },
                "Set grid 1 cell shape to Rectangle",
            ),
            (
                Action::SetGridSize {
                    index: 1,
                    dimension: CellDimension::Side,
                    size: 12.0,
                },
                "Set grid 2 size L to 12",
            ),
            (
                Action::SetGridUnit {
                    index: 0,
                    unit: LengthUnit::Metre,
                },
                "Set grid 1 units to m",
            ),
            (
                Action::SetGridColor {
                    index: 0,
                    color: Rgb::new(255, 0, 0),
                },
                "Set grid 1 color to #FF0000",
            ),
            (
                Action::SetGridOpacity {
                    index: 0,
                    opacity: 0.25,
                },
                "Set grid 1 opacity to 25%",
            ),
        ];
        for (action, label) in labels {
            assert_eq!(action.to_string(), label);
        }
    }
}
