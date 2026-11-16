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
use crate::shape::DrawnShape;
use crate::style::{InvalidStyle, StyleEdit, Styled};
use crate::tr;
use crate::units::LengthUnit;

/// The object a style edit applies to, by its place in its list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleTarget {
    Measurement(usize),
    Area(usize),
    Shape(usize),
}

impl fmt::Display for StyleTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            Self::Measurement(index) => tr!("target-measurement", { number: index + 1 }),
            Self::Area(index) => tr!("target-area", { number: index + 1 }),
            Self::Shape(index) => tr!("target-shape", { number: index + 1 }),
        })
    }
}

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
    /// Turns the view to a clockwise rotation in degrees, wrapped into [0, 360).
    SetRotation(f64),
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
    /// Replaces both ends of a measurement at once, after moving it as a whole.
    MoveMeasurement {
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
    /// Replaces every point of an area at once, after moving it as a whole. The number of
    /// points must stay the same.
    MoveArea { index: usize, points: Vec<Point> },
    /// Changes the unit an area is reported in.
    SetAreaUnit { index: usize, unit: LengthUnit },
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
    /// Appends a drawn shape to the end of the list.
    AddShape(DrawnShape),
    /// Moves one point of an existing shape.
    MoveShapePoint {
        index: usize,
        point: usize,
        to: Point,
    },
    /// Replaces every point of a shape at once (e.g. after resizing its bounding box). The
    /// number of points must stay the same.
    ResizeShape { index: usize, points: Vec<Point> },
    /// Replaces every point of a shape at once, after moving it as a whole. The number of
    /// points must stay the same.
    MoveShape { index: usize, points: Vec<Point> },
    /// Changes how a measurement, area or drawn shape is drawn.
    SetStyle {
        target: StyleTarget,
        edit: StyleEdit,
    },
    /// Removes a shape; later shapes move up one place.
    DeleteShape { index: usize },
    /// Appends an exact copy of a shape, with a new object id, to the end of the list.
    DuplicateShape { index: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionError {
    PointOutsideDocument(Point),
    NoScalePoints,
    InvalidDistance(f64),
    MeasurementIndexOutOfRange {
        index: usize,
        count: usize,
    },
    AreaIndexOutOfRange {
        index: usize,
        count: usize,
    },
    AreaPointIndexOutOfRange {
        point: usize,
        count: usize,
    },
    AreaPointCountChanged {
        expected: usize,
        actual: usize,
    },
    GridIndexOutOfRange {
        index: usize,
        count: usize,
    },
    GridSourceMissing,
    InvalidCellSize(f64),
    InvalidOpacity(f32),
    ShapeIndexOutOfRange {
        index: usize,
        count: usize,
    },
    ShapePointIndexOutOfRange {
        point: usize,
        count: usize,
    },
    ShapePointCountChanged {
        expected: usize,
        actual: usize,
    },
    /// Only an object that encloses an area can be filled.
    HasNoFill(StyleTarget),
    InvalidFillSpacing(f64),
    InvalidFillAngle(f64),
    InvalidRotation(f64),
    Calibration(CalibrationError),
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::PointOutsideDocument(point) => tr!("error-point-outside", { point: point }),
            Self::NoScalePoints => tr!("error-no-scale-points"),
            Self::InvalidDistance(distance) => {
                tr!("error-invalid-distance", { distance: distance })
            }
            Self::MeasurementIndexOutOfRange { index, count } => {
                tr!("error-measurement-missing", { number: index + 1, count: count })
            }
            Self::AreaIndexOutOfRange { index, count } => {
                tr!("error-area-missing", { number: index + 1, count: count })
            }
            Self::AreaPointIndexOutOfRange { point, count } => {
                tr!("error-area-point-missing", { number: point + 1, count: count })
            }
            Self::AreaPointCountChanged { expected, actual } => {
                tr!("error-area-point-count", { expected: expected, actual: actual })
            }
            Self::GridIndexOutOfRange { index, count } => {
                tr!("error-grid-missing", { number: index + 1, count: count })
            }
            Self::GridSourceMissing => tr!("error-grid-source-missing"),
            Self::InvalidCellSize(size) => tr!("error-invalid-cell-size", { size: size }),
            Self::InvalidOpacity(opacity) => tr!("error-invalid-opacity", { opacity: opacity }),
            Self::ShapeIndexOutOfRange { index, count } => {
                tr!("error-shape-missing", { number: index + 1, count: count })
            }
            Self::ShapePointIndexOutOfRange { point, count } => {
                tr!("error-shape-point-missing", { number: point + 1, count: count })
            }
            Self::ShapePointCountChanged { expected, actual } => {
                tr!("error-shape-point-count", { expected: expected, actual: actual })
            }
            Self::HasNoFill(target) => tr!("error-no-fill", { target: target }),
            Self::InvalidFillSpacing(spacing) => {
                tr!("error-invalid-fill-spacing", { spacing: spacing })
            }
            Self::InvalidFillAngle(degrees) => {
                tr!("error-invalid-fill-angle", { degrees: degrees })
            }
            Self::InvalidRotation(degrees) => {
                tr!("error-invalid-rotation", { degrees: degrees })
            }
            Self::Calibration(error) => return error.fmt(f),
        };
        f.write_str(&message)
    }
}

impl std::error::Error for ActionError {}

impl From<CalibrationError> for ActionError {
    fn from(error: CalibrationError) -> Self {
        Self::Calibration(error)
    }
}

impl From<InvalidStyle> for ActionError {
    fn from(invalid: InvalidStyle) -> Self {
        match invalid {
            InvalidStyle::Opacity(opacity) => Self::InvalidOpacity(opacity),
            InvalidStyle::FillSpacing(spacing) => Self::InvalidFillSpacing(spacing),
            InvalidStyle::FillAngle(degrees) => Self::InvalidFillAngle(degrees),
        }
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
            Action::SetRotation(degrees) => {
                if !degrees.is_finite() {
                    return Err(ActionError::InvalidRotation(*degrees));
                }
                document.set_rotation(crate::angle::normalize_degrees(*degrees));
            }
            Action::AddMeasurement(measurement) => {
                for endpoint in Endpoint::ALL {
                    ensure_inside(document, measurement.point(endpoint))?;
                }
                measurement.line().validate()?;
                let id = document.allocate_object_id();
                document
                    .measurements_mut()
                    .push(Identified::new(id, measurement.clone()));
            }
            Action::ResizeMeasurement { index, start, end }
            | Action::MoveMeasurement { index, start, end } => {
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
            Action::AddArea(area) => {
                for &point in area.points() {
                    ensure_inside(document, point)?;
                }
                area.line().validate()?;
                area.stored_fill().validate()?;
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
            Action::ResizeArea { index, points } | Action::MoveArea { index, points } => {
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
            Action::AddShape(shape) => {
                for &point in shape.points() {
                    ensure_inside(document, point)?;
                }
                shape.line().validate()?;
                shape.stored_fill().validate()?;
                let id = document.allocate_object_id();
                document
                    .shapes_mut()
                    .push(Identified::new(id, shape.clone()));
            }
            Action::MoveShapePoint { index, point, to } => {
                ensure_inside(document, *to)?;
                let shape = shape_mut(document, *index)?;
                let count = shape.points().len();
                if *point >= count {
                    return Err(ActionError::ShapePointIndexOutOfRange {
                        point: *point,
                        count,
                    });
                }
                *shape = shape.with_point(*point, *to);
            }
            Action::ResizeShape { index, points } | Action::MoveShape { index, points } => {
                for &point in points {
                    ensure_inside(document, point)?;
                }
                let shape = shape_mut(document, *index)?;
                let expected = shape.points().len();
                if points.len() != expected {
                    return Err(ActionError::ShapePointCountChanged {
                        expected,
                        actual: points.len(),
                    });
                }
                shape.set_points(points.clone());
            }
            Action::SetStyle { target, edit } => {
                edit.validate()?;
                match *target {
                    StyleTarget::Measurement(index) => {
                        restyle(measurement_mut(document, index)?, *edit, *target)?;
                    }
                    StyleTarget::Area(index) => {
                        restyle(area_mut(document, index)?, *edit, *target)?;
                    }
                    StyleTarget::Shape(index) => {
                        restyle(shape_mut(document, index)?, *edit, *target)?;
                    }
                }
            }
            Action::DeleteShape { index } => {
                shape_mut(document, *index)?;
                document.shapes_mut().remove(*index);
            }
            Action::DuplicateShape { index } => {
                let copy = shape_mut(document, *index)?.clone();
                let id = document.allocate_object_id();
                document.shapes_mut().push(Identified::new(id, copy));
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
        if let Some(index) = document.shape_index(id) {
            return Some(Action::DuplicateShape { index });
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
        if let Some(index) = document.shape_index(id) {
            return Some(Action::DeleteShape { index });
        }
        document
            .grid_index(id)
            .map(|index| Action::DeleteGrid { index })
    }

    /// Whether the object this action adds should become the active object.
    pub fn activates_new_object(&self) -> bool {
        matches!(
            self,
            Action::AddMeasurement(_)
                | Action::AddArea(_)
                | Action::AddGrid(_)
                | Action::AddShape(_)
                | Action::DuplicateMeasurement { .. }
                | Action::DuplicateArea { .. }
                | Action::DuplicateGrid { .. }
                | Action::DuplicateShape { .. }
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

fn shape_mut(document: &mut Document, index: usize) -> Result<&mut DrawnShape, ActionError> {
    let shapes = document.shapes_mut();
    let count = shapes.len();
    shapes
        .get_mut(index)
        .map(|shape| &mut **shape)
        .ok_or(ActionError::ShapeIndexOutOfRange { index, count })
}

/// Makes `edit` to `object`, which must have what it edits.
fn restyle<T: Styled>(
    object: &mut T,
    edit: StyleEdit,
    target: StyleTarget,
) -> Result<(), ActionError> {
    *object = object
        .with_edit(edit)
        .ok_or(ActionError::HasNoFill(target))?;
    Ok(())
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
        f.write_str(&match self {
            Action::SetScalePoints(points) => {
                tr!("action-set-scale-points", { method: points.method() })
            }
            Action::MoveScalePoint { index, to } => {
                tr!("action-move-scale-point", { number: index + 1, to: to })
            }
            Action::SetDistance {
                line,
                distance: Some(distance),
            } => tr!("action-set-distance", { line: line, distance: distance }),
            Action::SetDistance {
                line,
                distance: None,
            } => tr!("action-clear-distance", { line: line }),
            Action::SetUnit(unit) => tr!("action-set-unit", { unit: unit }),
            Action::SetOrigin(origin) => tr!("action-set-origin", { origin: origin }),
            Action::SetRotation(degrees) => tr!("action-set-rotation", {
                degrees: crate::angle::normalize_degrees(*degrees),
            }),
            Action::AddMeasurement(measurement) => tr!("action-add-measurement", {
                start: measurement.start(),
                end: measurement.end(),
            }),
            Action::MoveMeasurementPoint {
                index,
                endpoint,
                to,
            } => tr!("action-move-measurement-point", {
                number: index + 1,
                endpoint: endpoint,
                to: to,
            }),
            Action::ResizeMeasurement { index, start, end } => {
                tr!("action-resize-measurement", { number: index + 1, start: start, end: end })
            }
            Action::MoveMeasurement { index, start, end } => {
                tr!("action-move-measurement", { number: index + 1, start: start, end: end })
            }
            Action::SetMeasurementUnit { index, unit } => {
                tr!("action-set-measurement-unit", { number: index + 1, unit: unit })
            }
            Action::DeleteMeasurement { index } => {
                tr!("action-delete-measurement", { number: index + 1 })
            }
            Action::AddArea(area) => tr!("action-add-area", { count: area.points().len() }),
            Action::MoveAreaPoint { index, point, to } => {
                tr!("action-move-area-point", { number: index + 1, point: point + 1, to: to })
            }
            Action::ResizeArea { index, .. } => tr!("action-resize-area", { number: index + 1 }),
            Action::MoveArea { index, .. } => tr!("action-move-area", { number: index + 1 }),
            Action::SetAreaUnit { index, unit } => {
                tr!("action-set-area-unit", { number: index + 1, unit: unit })
            }
            Action::DeleteArea { index } => tr!("action-delete-area", { number: index + 1 }),
            Action::DuplicateArea { index } => {
                tr!("action-duplicate-area", { number: index + 1 })
            }
            Action::DuplicateMeasurement { index } => {
                tr!("action-duplicate-measurement", { number: index + 1 })
            }
            Action::DuplicateGrid { index } => {
                tr!("action-duplicate-grid", { number: index + 1 })
            }
            Action::DeleteGrid { index } => tr!("action-delete-grid", { number: index + 1 }),
            Action::AddGrid(grid) => match grid.source() {
                GridSource::ScaleLine(line) => {
                    tr!("action-add-grid-scale-line", { line: line })
                }
                GridSource::Measurement(_) => tr!("action-add-grid-measurement"),
            },
            Action::SetGridShape { index, shape } => {
                tr!("action-set-grid-shape", { number: index + 1, shape: shape.name() })
            }
            Action::SetGridSize {
                index,
                dimension,
                size,
            } => tr!("action-set-grid-size", {
                number: index + 1,
                dimension: dimension,
                size: size,
            }),
            Action::SetGridUnit { index, unit } => {
                tr!("action-set-grid-unit", { number: index + 1, unit: unit })
            }
            Action::SetGridColor { index, color } => {
                tr!("action-set-grid-color", { number: index + 1, color: color })
            }
            Action::SetGridOpacity { index, opacity } => tr!("action-set-grid-opacity", {
                number: index + 1,
                percent: (opacity * 100.0).round(),
            }),
            Action::AddShape(shape) if shape.is_closed() => {
                tr!("action-add-closed-shape", { count: shape.points().len() })
            }
            Action::AddShape(shape) => tr!("action-add-line", { count: shape.points().len() }),
            Action::MoveShapePoint { index, point, to } => {
                tr!("action-move-shape-point", { number: index + 1, point: point + 1, to: to })
            }
            Action::ResizeShape { index, .. } => {
                tr!("action-resize-shape", { number: index + 1 })
            }
            Action::MoveShape { index, .. } => tr!("action-move-shape", { number: index + 1 }),
            Action::SetStyle { target, edit } => {
                tr!("action-set-style", { target: target, edit: edit })
            }
            Action::DeleteShape { index } => tr!("action-delete-shape", { number: index + 1 }),
            Action::DuplicateShape { index } => {
                tr!("action-duplicate-shape", { number: index + 1 })
            }
        })
    }
}

#[cfg(test)]
mod tests;
