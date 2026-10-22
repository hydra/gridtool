//! Every change that can be made to a [`Document`]. Each action is a self-contained
//! value, so it can be recorded for undo/redo, shown in a history list, or created by a
//! non-GUI front end.

use std::fmt;

use crate::area::Area;
use crate::calibration::{CalibrationError, Line, ScalePoints};
use crate::color::Rgb;
use crate::document::Document;
use crate::geometry::Point;
use crate::grid::{CellDimension, CellShape, Grid, GridSource};
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
    /// Changes a measurement's line colour.
    SetMeasurementColor { index: usize, color: Rgb },
    /// Removes a measurement; later measurements move up one place.
    DeleteMeasurement { index: usize },
    /// Appends an exact copy of a measurement, with a new object id, to the end of the list.
    DuplicateMeasurement { index: usize },
    /// Appends an area measurement to the end of the list.
    AddArea(Area),
    /// Moves one point of an existing area.
    MoveAreaPoint {
        index: usize,
        point: usize,
        to: Point,
    },
    /// Replaces every point of an area at once (e.g. after resizing its bounding box). The
    /// number of points must stay the same.
    ResizeArea { index: usize, points: Vec<Point> },
    /// Changes the unit an area is reported in.
    SetAreaUnit { index: usize, unit: LengthUnit },
    /// Changes an area's colour.
    SetAreaColor { index: usize, color: Rgb },
    /// Removes an area; later areas move up one place.
    DeleteArea { index: usize },
    /// Appends an exact copy of an area, with a new object id, to the end of the list.
    DuplicateArea { index: usize },
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
    AreaIndexOutOfRange { index: usize, count: usize },
    AreaPointIndexOutOfRange { point: usize, count: usize },
    AreaPointCountChanged { expected: usize, actual: usize },
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
            Self::AreaIndexOutOfRange { index, count } => {
                write!(f, "area {} does not exist (there are {count})", index + 1)
            }
            Self::AreaPointIndexOutOfRange { point, count } => write!(
                f,
                "area point {} does not exist (there are {count})",
                point + 1
            ),
            Self::AreaPointCountChanged { expected, actual } => write!(
                f,
                "an area with {expected} points cannot be given {actual} points"
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
            Action::SetMeasurementColor { index, color } => {
                measurement_mut(document, *index)?.set_color(*color);
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
            Action::AddArea(area) => {
                for &point in area.points() {
                    ensure_inside(document, point)?;
                }
                let id = document.allocate_object_id();
                document.areas_mut().push(Identified::new(id, area.clone()));
            }
            Action::MoveAreaPoint { index, point, to } => {
                ensure_inside(document, *to)?;
                let area = area_mut(document, *index)?;
                let count = area.points().len();
                if *point >= count {
                    return Err(ActionError::AreaPointIndexOutOfRange {
                        point: *point,
                        count,
                    });
                }
                *area = area.with_point(*point, *to);
            }
            Action::ResizeArea { index, points } => {
                for &point in points {
                    ensure_inside(document, point)?;
                }
                let area = area_mut(document, *index)?;
                let expected = area.points().len();
                if points.len() != expected {
                    return Err(ActionError::AreaPointCountChanged {
                        expected,
                        actual: points.len(),
                    });
                }
                area.set_points(points.clone());
            }
            Action::SetAreaUnit { index, unit } => area_mut(document, *index)?.set_unit(*unit),
            Action::SetAreaColor { index, color } => area_mut(document, *index)?.set_color(*color),
            Action::DeleteArea { index } => {
                area_mut(document, *index)?;
                document.areas_mut().remove(*index);
            }
            Action::DuplicateArea { index } => {
                let copy = area_mut(document, *index)?.clone();
                let id = document.allocate_object_id();
                document.areas_mut().push(Identified::new(id, copy));
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
        if let Some(index) = document.area_index(id) {
            return Some(Action::DuplicateArea { index });
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
        if let Some(index) = document.area_index(id) {
            return Some(Action::DeleteArea { index });
        }
        document
            .grid_index(id)
            .map(|index| Action::DeleteGrid { index })
    }

    /// Whether the object this action adds should become the active object.
    pub fn activates_new_object(&self) -> bool {
        matches!(
            self,
            Action::AddGrid(_)
                | Action::DuplicateMeasurement { .. }
                | Action::DuplicateArea { .. }
                | Action::DuplicateGrid { .. }
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

fn area_mut(document: &mut Document, index: usize) -> Result<&mut Area, ActionError> {
    let areas = document.areas_mut();
    let count = areas.len();
    areas
        .get_mut(index)
        .map(|area| &mut **area)
        .ok_or(ActionError::AreaIndexOutOfRange { index, count })
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
            Action::SetMeasurementColor { index, color } => {
                write!(f, "Set measurement {} color to {color}", index + 1)
            }
            Action::DeleteMeasurement { index } => {
                write!(f, "Delete measurement {}", index + 1)
            }
            Action::AddArea(area) => write!(f, "Add area with {} points", area.points().len()),
            Action::MoveAreaPoint { index, point, to } => {
                write!(f, "Move area {} point {} to {to}", index + 1, point + 1)
            }
            Action::ResizeArea { index, .. } => write!(f, "Resize area {}", index + 1),
            Action::SetAreaUnit { index, unit } => {
                write!(f, "Set area {} units to {unit}", index + 1)
            }
            Action::SetAreaColor { index, color } => {
                write!(f, "Set area {} color to {color}", index + 1)
            }
            Action::DeleteArea { index } => write!(f, "Delete area {}", index + 1),
            Action::DuplicateArea { index } => write!(f, "Duplicate area {}", index + 1),
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
mod tests;
