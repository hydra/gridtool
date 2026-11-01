//! Version 5 of the saved-document format: version 4 with an optional fill colour for areas
//! and shapes, whose fill angles are now measured from vertical. A fill with no colour of its
//! own is drawn in the line's colour.
//!
//! Like every version, these types never change once released. Parts that did not change
//! since earlier versions reuse their types.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::area::{self, Area};
use crate::calibration::{Calibration, Line, ScalePoints};
use crate::document::{Document, Objects};
use crate::geometry::Point;
use crate::grid::{CellDimension, Grid};
use crate::measurement::Measurement;
use crate::object::{Identified, ObjectId};
use crate::shape::{self, DrawnShape};
use crate::style::{FillAppearance, InvalidStyle, LineAppearance, ObjectStyle, Styled};

use super::InvalidDocument;
use super::v1::{CalibrationV1, ColorV1, GridV1, MethodV1, PointV1, ScalePointsV1, UnitV1};
use super::v3::{FillStyleV3, LineStyleV3, ShapeV3};
use super::v4::{AreaV4, DocumentV4, MeasurementV4};

/// Versions up to 5 kept fill angles below this; version 6 does not (see [`DocumentV5::to_document`]).
const HALF_TURN: f64 = 180.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentV5 {
    /// The image file, relative to the document file where possible.
    pub image: String,
    /// Image size in pixels; checked against the image when loading.
    pub width: u32,
    pub height: u32,
    pub origin: PointV1,
    pub calibration: CalibrationV1,
    pub measurements: Vec<MeasurementV4>,
    pub areas: Vec<AreaV5>,
    pub grids: Vec<GridV1>,
    pub shapes: Vec<ShapeV5>,
    /// The id the next added object will get.
    pub next_object_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AreaV5 {
    pub id: u64,
    /// The outline's points, in order; at least three.
    pub points: Vec<PointV1>,
    pub unit: UnitV1,
    pub line_style: LineStyleV3,
    pub line_color: ColorV1,
    pub line_opacity: f32,
    pub fill_style: FillStyleV3,
    /// `None` if the fill is drawn in the line's colour.
    pub fill_color: Option<ColorV1>,
    /// The real-world distance between fill lines, in `fill_unit`.
    pub fill_spacing: f64,
    pub fill_unit: UnitV1,
    /// Degrees clockwise from vertical, in [0, 180), so 90 is horizontal.
    pub fill_angle: f64,
    pub fill_opacity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeV5 {
    pub id: u64,
    /// The shape's points, in order; at least two, or three if closed.
    pub points: Vec<PointV1>,
    pub closed: bool,
    pub line_style: LineStyleV3,
    pub line_color: ColorV1,
    pub line_opacity: f32,
    /// `None` unless closed.
    pub fill_style: FillStyleV3,
    /// `None` if the fill is drawn in the line's colour.
    pub fill_color: Option<ColorV1>,
    /// The real-world distance between fill lines, in `fill_unit`.
    pub fill_spacing: f64,
    pub fill_unit: UnitV1,
    /// Degrees clockwise from vertical, in [0, 180), so 90 is horizontal.
    pub fill_angle: f64,
    pub fill_opacity: f32,
}

fn is_within_half_turn(degrees: f64) -> bool {
    (0.0..HALF_TURN).contains(&degrees)
}

/// The angle of fill lines `degrees` clockwise from the image's x axis, as version 4 stored it.
/// An invalid angle stays invalid, so that loading rejects it.
fn angle_from_vertical(degrees: f64) -> f64 {
    if is_within_half_turn(degrees) {
        (degrees + 90.0) % HALF_TURN
    } else {
        degrees
    }
}

/// Every fill in version 4 had its own colour.
impl From<AreaV4> for AreaV5 {
    fn from(area: AreaV4) -> Self {
        Self {
            id: area.id,
            points: area.points,
            unit: area.unit,
            line_style: area.line_style,
            line_color: area.line_color,
            line_opacity: area.line_opacity,
            fill_style: area.fill_style,
            fill_color: Some(area.fill_color),
            fill_spacing: area.fill_spacing,
            fill_unit: area.fill_unit,
            fill_angle: angle_from_vertical(area.fill_angle),
            fill_opacity: area.fill_opacity,
        }
    }
}

/// Every fill in version 4 had its own colour.
impl From<ShapeV3> for ShapeV5 {
    fn from(shape: ShapeV3) -> Self {
        Self {
            id: shape.id,
            points: shape.points,
            closed: shape.closed,
            line_style: shape.line_style,
            line_color: shape.line_color,
            line_opacity: shape.line_opacity,
            fill_style: shape.fill_style,
            fill_color: Some(shape.fill_color),
            fill_spacing: shape.fill_spacing,
            fill_unit: shape.fill_unit,
            fill_angle: angle_from_vertical(shape.fill_angle),
            fill_opacity: shape.fill_opacity,
        }
    }
}

impl From<DocumentV4> for DocumentV5 {
    fn from(file: DocumentV4) -> Self {
        Self {
            image: file.image,
            width: file.width,
            height: file.height,
            origin: file.origin,
            calibration: file.calibration,
            measurements: file.measurements,
            areas: file.areas.into_iter().map(Into::into).collect(),
            grids: file.grids,
            shapes: file.shapes.into_iter().map(Into::into).collect(),
            next_object_id: file.next_object_id,
        }
    }
}

impl DocumentV5 {
    /// Re-creates the runtime document from a file of version 5 or earlier, which is also not
    /// valid with a fill angle of a half turn or more.
    pub fn to_document_within_half_turn(&self) -> Result<Document, InvalidDocument> {
        let areas = self.areas.iter().map(|area| area.fill_angle);
        let shapes = self.shapes.iter().map(|shape| shape.fill_angle);
        match areas.chain(shapes).find(|a| !is_within_half_turn(*a)) {
            Some(degrees) => Err(InvalidDocument::BadFillAngle(degrees)),
            None => self.to_document_without_two_point(),
        }
    }

    /// Re-creates the runtime document from a file of version 6 or earlier, which is also not
    /// valid with a 2-point scale.
    pub fn to_document_without_two_point(&self) -> Result<Document, InvalidDocument> {
        let method = self.calibration.points.as_ref().map(|points| points.method);
        if method == Some(MethodV1::TwoPoint) {
            return Err(InvalidDocument::TwoPointNotInVersion);
        }
        self.to_document()
    }

    /// Describes `document`, whose image is referred to as `image`.
    pub fn describe(document: &Document, image: String) -> Self {
        let calibration = document.calibration();
        Self {
            image,
            width: document.width(),
            height: document.height(),
            origin: document.origin().into(),
            calibration: CalibrationV1 {
                points: calibration.points().map(|points| ScalePointsV1 {
                    method: points.method().into(),
                    points: points.points().iter().map(|&p| p.into()).collect(),
                }),
                distance_a: calibration.distance(Line::A),
                distance_b: calibration.distance(Line::B),
                unit: calibration.unit().into(),
            },
            measurements: document
                .measurements()
                .iter()
                .map(|measurement| {
                    let line = measurement.line();
                    MeasurementV4 {
                        id: measurement.id().raw(),
                        start: measurement.start().into(),
                        end: measurement.end().into(),
                        unit: measurement.unit().into(),
                        line_style: line.style.into(),
                        line_color: line.color.into(),
                        line_opacity: line.opacity,
                    }
                })
                .collect(),
            areas: document
                .areas()
                .iter()
                .map(|area| {
                    let (line, fill) = (area.line(), area.stored_fill());
                    AreaV5 {
                        id: area.id().raw(),
                        points: area.points().iter().map(|&p| p.into()).collect(),
                        unit: area.unit().into(),
                        line_style: line.style.into(),
                        line_color: line.color.into(),
                        line_opacity: line.opacity,
                        fill_style: fill.style.into(),
                        fill_color: fill.color.map(Into::into),
                        fill_spacing: fill.spacing,
                        fill_unit: fill.unit.into(),
                        fill_angle: fill.angle,
                        fill_opacity: fill.opacity,
                    }
                })
                .collect(),
            grids: document
                .grids()
                .iter()
                .map(|grid| GridV1 {
                    id: grid.id().raw(),
                    source: grid.source().into(),
                    shape: grid.shape().into(),
                    size_a: grid.size(CellDimension::A),
                    size_b: grid.size(CellDimension::B),
                    unit: grid.unit().into(),
                    color: grid.color().into(),
                    opacity: grid.opacity(),
                })
                .collect(),
            shapes: document
                .shapes()
                .iter()
                .map(|shape| {
                    let (line, fill) = (shape.line(), shape.stored_fill());
                    ShapeV5 {
                        id: shape.id().raw(),
                        points: shape.points().iter().map(|&p| p.into()).collect(),
                        closed: shape.is_closed(),
                        line_style: line.style.into(),
                        line_color: line.color.into(),
                        line_opacity: line.opacity,
                        fill_style: fill.style.into(),
                        fill_color: fill.color.map(Into::into),
                        fill_spacing: fill.spacing,
                        fill_unit: fill.unit.into(),
                        fill_angle: fill.angle,
                        fill_opacity: fill.opacity,
                    }
                })
                .collect(),
            next_object_id: document.next_object_id(),
        }
    }

    /// Re-creates the runtime document, checking everything an action would have
    /// checked, so a hand-edited or corrupt file cannot produce an invalid document.
    pub fn to_document(&self) -> Result<Document, InvalidDocument> {
        if self.width == 0 || self.height == 0 {
            return Err(InvalidDocument::EmptySize);
        }
        let blank = Document::new(self.width, self.height);
        let inside = |point: PointV1| -> Result<Point, InvalidDocument> {
            let point = Point::from(point);
            if blank.contains(point) {
                Ok(point)
            } else {
                Err(InvalidDocument::PointOutside(point))
            }
        };

        let mut calibration = Calibration::default();
        if let Some(stored) = &self.calibration.points {
            let points = stored
                .points
                .iter()
                .map(|&point| inside(point))
                .collect::<Result<Vec<_>, _>>()?;
            let points = ScalePoints::new(stored.method.into(), points)
                .map_err(|_| InvalidDocument::WrongScalePointCount)?;
            calibration.set_points(points);
        }
        for (line, distance) in [
            (Line::A, self.calibration.distance_a),
            (Line::B, self.calibration.distance_b),
        ] {
            if let Some(value) = distance {
                ensure_positive(value)?;
            }
            calibration.set_distance(line, distance);
        }
        calibration.set_unit(self.calibration.unit.into());

        let mut ids = HashSet::new();
        let mut claim = |id: u64| {
            if id == 0 || id >= self.next_object_id || !ids.insert(id) {
                Err(InvalidDocument::BadObjectId(id))
            } else {
                Ok(ObjectId::new(id))
            }
        };

        let mut measurements = Vec::with_capacity(self.measurements.len());
        for stored in &self.measurements {
            let mut measurement = Measurement::new(
                inside(stored.start)?,
                inside(stored.end)?,
                stored.unit.into(),
            );
            *measurement.line_mut() =
                stored_line(stored.line_style, stored.line_color, stored.line_opacity)?;
            measurements.push(Identified::new(claim(stored.id)?, measurement));
        }

        let mut areas = Vec::with_capacity(self.areas.len());
        for stored in &self.areas {
            let points = stored
                .points
                .iter()
                .map(|&point| inside(point))
                .collect::<Result<Vec<_>, _>>()?;
            let mut area = Area::new(points, stored.unit.into())
                .map_err(|area::TooFewPoints(count)| InvalidDocument::TooFewAreaPoints(count))?;
            area.adopt(&ObjectStyle {
                line: stored_line(stored.line_style, stored.line_color, stored.line_opacity)?,
                fill: Some(stored_fill(
                    stored.fill_style,
                    stored.fill_color,
                    stored.fill_spacing,
                    stored.fill_unit,
                    stored.fill_angle,
                    stored.fill_opacity,
                )?),
            });
            areas.push(Identified::new(claim(stored.id)?, area));
        }

        let mut grids = Vec::with_capacity(self.grids.len());
        for stored in &self.grids {
            ensure_positive(stored.size_a)?;
            ensure_positive(stored.size_b)?;
            if !(0.0..=1.0).contains(&stored.opacity) {
                return Err(InvalidDocument::BadOpacity(stored.opacity));
            }
            let mut grid = Grid::new(stored.source.into(), stored.unit.into());
            grid.set_shape(stored.shape.into());
            grid.set_size(CellDimension::A, stored.size_a);
            grid.set_size(CellDimension::B, stored.size_b);
            grid.set_color(stored.color.into());
            grid.set_opacity(stored.opacity);
            grids.push(Identified::new(claim(stored.id)?, grid));
        }

        let mut shapes = Vec::with_capacity(self.shapes.len());
        for stored in &self.shapes {
            let points = stored
                .points
                .iter()
                .map(|&point| inside(point))
                .collect::<Result<Vec<_>, _>>()?;
            let shape = if stored.closed {
                DrawnShape::closed(points, stored.fill_unit.into())
            } else {
                DrawnShape::open(points, stored.fill_unit.into())
            };
            let mut shape = shape
                .map_err(|shape::TooFewPoints(count)| InvalidDocument::TooFewShapePoints(count))?;
            if !stored.closed && stored.fill_style != FillStyleV3::None {
                return Err(InvalidDocument::FilledOpenShape(stored.id));
            }
            *shape.line_mut() =
                stored_line(stored.line_style, stored.line_color, stored.line_opacity)?;
            shape.set_stored_fill(stored_fill(
                stored.fill_style,
                stored.fill_color,
                stored.fill_spacing,
                stored.fill_unit,
                stored.fill_angle,
                stored.fill_opacity,
            )?);
            shapes.push(Identified::new(claim(stored.id)?, shape));
        }

        Ok(Document::from_parts(
            self.width,
            self.height,
            calibration,
            Objects {
                measurements,
                areas,
                grids,
                shapes,
            },
            inside(self.origin)?,
            self.next_object_id,
        ))
    }
}

fn stored_line(
    style: LineStyleV3,
    color: ColorV1,
    opacity: f32,
) -> Result<LineAppearance, InvalidDocument> {
    let line = LineAppearance {
        style: style.into(),
        color: color.into(),
        opacity,
    };
    line.validate()?;
    Ok(line)
}

fn stored_fill(
    style: FillStyleV3,
    color: Option<ColorV1>,
    spacing: f64,
    unit: UnitV1,
    angle: f64,
    opacity: f32,
) -> Result<FillAppearance, InvalidDocument> {
    let fill = FillAppearance {
        style: style.into(),
        color: color.map(Into::into),
        spacing,
        unit: unit.into(),
        angle,
        opacity,
    };
    fill.validate()?;
    Ok(fill)
}

impl From<InvalidStyle> for InvalidDocument {
    fn from(invalid: InvalidStyle) -> Self {
        match invalid {
            InvalidStyle::Opacity(opacity) => Self::BadOpacity(opacity),
            InvalidStyle::FillSpacing(spacing) => Self::NotPositive(spacing),
            InvalidStyle::FillAngle(degrees) => Self::BadFillAngle(degrees),
        }
    }
}

fn ensure_positive(value: f64) -> Result<(), InvalidDocument> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(InvalidDocument::NotPositive(value))
    }
}
