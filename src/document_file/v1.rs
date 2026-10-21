//! Version 1 of the saved-document format.
//!
//! These types describe a document as stored on disk. They are deliberately separate
//! from the runtime types: once released, a version's types never change, so files
//! written by older versions of the application can always be read. A new format gets
//! a new module and a new [`super::DocumentFile`] variant.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::calibration::{Calibration, CalibrationMethod, Line, ScalePoints};
use crate::document::Document;
use crate::geometry::Point;
use crate::grid::{CellDimension, CellShape, Grid, GridSource, Rgb};
use crate::measurement::Measurement;
use crate::object::{Identified, ObjectId};
use crate::units::LengthUnit;

use super::InvalidDocument;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentV1 {
    /// The image file, relative to the document file where possible.
    pub image: String,
    /// Image size in pixels; checked against the image when loading.
    pub width: u32,
    pub height: u32,
    pub origin: PointV1,
    pub calibration: CalibrationV1,
    pub measurements: Vec<MeasurementV1>,
    pub grids: Vec<GridV1>,
    /// The id the next added object will get.
    pub next_object_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PointV1 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalibrationV1 {
    pub points: Option<ScalePointsV1>,
    pub distance_a: Option<f64>,
    pub distance_b: Option<f64>,
    pub unit: UnitV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScalePointsV1 {
    pub method: MethodV1,
    pub points: Vec<PointV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodV1 {
    ThreePoint,
    FourPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitV1 {
    Centimetre,
    Metre,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementV1 {
    pub id: u64,
    pub start: PointV1,
    pub end: PointV1,
    pub unit: UnitV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GridV1 {
    pub id: u64,
    pub source: GridSourceV1,
    pub shape: CellShapeV1,
    pub size_a: f64,
    pub size_b: f64,
    pub unit: UnitV1,
    pub color: ColorV1,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridSourceV1 {
    ScaleLineA,
    ScaleLineB,
    /// A measurement, by object id.
    Measurement(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellShapeV1 {
    Square,
    Rectangle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorV1 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl DocumentV1 {
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
                .map(|measurement| MeasurementV1 {
                    id: measurement.id().raw(),
                    start: measurement.start().into(),
                    end: measurement.end().into(),
                    unit: measurement.unit().into(),
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
            let measurement = Measurement::new(
                inside(stored.start)?,
                inside(stored.end)?,
                stored.unit.into(),
            );
            measurements.push(Identified::new(claim(stored.id)?, measurement));
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

        Ok(Document::from_parts(
            self.width,
            self.height,
            calibration,
            measurements,
            grids,
            inside(self.origin)?,
            self.next_object_id,
        ))
    }
}

fn ensure_positive(value: f64) -> Result<(), InvalidDocument> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(InvalidDocument::NotPositive(value))
    }
}

// Conversions between runtime values and their stored form.

impl From<Point> for PointV1 {
    fn from(point: Point) -> Self {
        Self {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<PointV1> for Point {
    fn from(point: PointV1) -> Self {
        Point::new(point.x, point.y)
    }
}

impl From<CalibrationMethod> for MethodV1 {
    fn from(method: CalibrationMethod) -> Self {
        match method {
            CalibrationMethod::ThreePoint => Self::ThreePoint,
            CalibrationMethod::FourPoint => Self::FourPoint,
        }
    }
}

impl From<MethodV1> for CalibrationMethod {
    fn from(method: MethodV1) -> Self {
        match method {
            MethodV1::ThreePoint => Self::ThreePoint,
            MethodV1::FourPoint => Self::FourPoint,
        }
    }
}

impl From<LengthUnit> for UnitV1 {
    fn from(unit: LengthUnit) -> Self {
        match unit {
            LengthUnit::Centimetre => Self::Centimetre,
            LengthUnit::Metre => Self::Metre,
        }
    }
}

impl From<UnitV1> for LengthUnit {
    fn from(unit: UnitV1) -> Self {
        match unit {
            UnitV1::Centimetre => Self::Centimetre,
            UnitV1::Metre => Self::Metre,
        }
    }
}

impl From<GridSource> for GridSourceV1 {
    fn from(source: GridSource) -> Self {
        match source {
            GridSource::ScaleLine(Line::A) => Self::ScaleLineA,
            GridSource::ScaleLine(Line::B) => Self::ScaleLineB,
            GridSource::Measurement(id) => Self::Measurement(id.raw()),
        }
    }
}

impl From<GridSourceV1> for GridSource {
    fn from(source: GridSourceV1) -> Self {
        match source {
            GridSourceV1::ScaleLineA => Self::ScaleLine(Line::A),
            GridSourceV1::ScaleLineB => Self::ScaleLine(Line::B),
            // A grid may outlive its measurement, so a missing id is allowed.
            GridSourceV1::Measurement(id) => Self::Measurement(ObjectId::new(id)),
        }
    }
}

impl From<CellShape> for CellShapeV1 {
    fn from(shape: CellShape) -> Self {
        match shape {
            CellShape::Square => Self::Square,
            CellShape::Rectangle => Self::Rectangle,
        }
    }
}

impl From<CellShapeV1> for CellShape {
    fn from(shape: CellShapeV1) -> Self {
        match shape {
            CellShapeV1::Square => Self::Square,
            CellShapeV1::Rectangle => Self::Rectangle,
        }
    }
}

impl From<Rgb> for ColorV1 {
    fn from(color: Rgb) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

impl From<ColorV1> for Rgb {
    fn from(color: ColorV1) -> Self {
        Rgb::new(color.r, color.g, color.b)
    }
}
