//! Version 3 of the saved-document format: version 2 plus drawn shapes.
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
use crate::shape::{self, DrawnShape, FillStyle, LineStyle};

use super::InvalidDocument;
use super::v1::{CalibrationV1, GridV1, PointV1, ScalePointsV1};
use super::v2::{AreaV2, DocumentV2, MeasurementV2};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentV3 {
    /// The image file, relative to the document file where possible.
    pub image: String,
    /// Image size in pixels; checked against the image when loading.
    pub width: u32,
    pub height: u32,
    pub origin: PointV1,
    pub calibration: CalibrationV1,
    pub measurements: Vec<MeasurementV2>,
    pub areas: Vec<AreaV2>,
    pub grids: Vec<GridV1>,
    pub shapes: Vec<ShapeV3>,
    /// The id the next added object will get.
    pub next_object_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeV3 {
    pub id: u64,
    /// The shape's points, in order; at least two, or three if closed.
    pub points: Vec<PointV1>,
    pub closed: bool,
    pub line_style: LineStyleV3,
    pub line_opacity: f32,
    /// `None` unless closed.
    pub fill_style: FillStyleV3,
    pub fill_spacing: f64,
    pub fill_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineStyleV3 {
    Thin,
    Thick,
    Dashed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillStyleV3 {
    None,
    Solid,
    VerticalLines,
    HorizontalLines,
    Crosshatch,
}

impl From<LineStyle> for LineStyleV3 {
    fn from(style: LineStyle) -> Self {
        match style {
            LineStyle::Thin => Self::Thin,
            LineStyle::Thick => Self::Thick,
            LineStyle::Dashed => Self::Dashed,
        }
    }
}

impl From<LineStyleV3> for LineStyle {
    fn from(style: LineStyleV3) -> Self {
        match style {
            LineStyleV3::Thin => Self::Thin,
            LineStyleV3::Thick => Self::Thick,
            LineStyleV3::Dashed => Self::Dashed,
        }
    }
}

impl From<FillStyle> for FillStyleV3 {
    fn from(style: FillStyle) -> Self {
        match style {
            FillStyle::None => Self::None,
            FillStyle::Solid => Self::Solid,
            FillStyle::VerticalLines => Self::VerticalLines,
            FillStyle::HorizontalLines => Self::HorizontalLines,
            FillStyle::Crosshatch => Self::Crosshatch,
        }
    }
}

impl From<FillStyleV3> for FillStyle {
    fn from(style: FillStyleV3) -> Self {
        match style {
            FillStyleV3::None => Self::None,
            FillStyleV3::Solid => Self::Solid,
            FillStyleV3::VerticalLines => Self::VerticalLines,
            FillStyleV3::HorizontalLines => Self::HorizontalLines,
            FillStyleV3::Crosshatch => Self::Crosshatch,
        }
    }
}

/// Version 2 had no drawn shapes.
impl From<DocumentV2> for DocumentV3 {
    fn from(file: DocumentV2) -> Self {
        Self {
            image: file.image,
            width: file.width,
            height: file.height,
            origin: file.origin,
            calibration: file.calibration,
            measurements: file.measurements,
            areas: file.areas,
            grids: file.grids,
            shapes: Vec::new(),
            next_object_id: file.next_object_id,
        }
    }
}

impl DocumentV3 {
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
                .map(|measurement| MeasurementV2 {
                    id: measurement.id().raw(),
                    start: measurement.start().into(),
                    end: measurement.end().into(),
                    unit: measurement.unit().into(),
                    color: measurement.color().into(),
                })
                .collect(),
            areas: document
                .areas()
                .iter()
                .map(|area| AreaV2 {
                    id: area.id().raw(),
                    points: area.points().iter().map(|&p| p.into()).collect(),
                    unit: area.unit().into(),
                    color: area.color().into(),
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
                .map(|shape| ShapeV3 {
                    id: shape.id().raw(),
                    points: shape.points().iter().map(|&p| p.into()).collect(),
                    closed: shape.is_closed(),
                    line_style: shape.line_style().into(),
                    line_opacity: shape.line_opacity(),
                    fill_style: shape.fill_style().into(),
                    fill_spacing: shape.fill_spacing(),
                    fill_opacity: shape.fill_opacity(),
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
            measurement.set_color(stored.color.into());
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
            area.set_color(stored.color.into());
            areas.push(Identified::new(claim(stored.id)?, area));
        }

        let mut grids = Vec::with_capacity(self.grids.len());
        for stored in &self.grids {
            ensure_positive(stored.size_a)?;
            ensure_positive(stored.size_b)?;
            ensure_opacity(stored.opacity)?;
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
                DrawnShape::closed(points)
            } else {
                DrawnShape::open(points)
            };
            let mut shape = shape
                .map_err(|shape::TooFewPoints(count)| InvalidDocument::TooFewShapePoints(count))?;
            if !stored.closed && stored.fill_style != FillStyleV3::None {
                return Err(InvalidDocument::FilledOpenShape(stored.id));
            }
            ensure_opacity(stored.line_opacity)?;
            ensure_opacity(stored.fill_opacity)?;
            ensure_positive(stored.fill_spacing)?;
            shape.set_line_style(stored.line_style.into());
            shape.set_line_opacity(stored.line_opacity);
            shape.set_fill_style(stored.fill_style.into());
            shape.set_fill_spacing(stored.fill_spacing);
            shape.set_fill_opacity(stored.fill_opacity);
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

fn ensure_opacity(value: f32) -> Result<(), InvalidDocument> {
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(InvalidDocument::BadOpacity(value))
    }
}

fn ensure_positive(value: f64) -> Result<(), InvalidDocument> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(InvalidDocument::NotPositive(value))
    }
}
