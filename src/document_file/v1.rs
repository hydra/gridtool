//! Version 1 of the saved-document format.
//!
//! These types describe a document as stored on disk. They are deliberately separate
//! from the runtime types: once released, a version's types never change, so files
//! written by older versions of the application can always be read. A new format gets
//! a new module and a new [`super::DocumentFile`] variant.
//!
//! Version 1 files are loaded by migrating them to version 2 ([`super::v2`]), which also
//! reuses the version 1 types that did not change.

use serde::{Deserialize, Serialize};

use crate::calibration::{CalibrationMethod, Line};
use crate::color::Rgb;
use crate::geometry::Point;
use crate::grid::{CellShape, GridSource};
use crate::object::ObjectId;
use crate::units::LengthUnit;

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
