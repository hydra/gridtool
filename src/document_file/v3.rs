//! Version 3 of the saved-document format: version 2 plus drawn shapes.
//!
//! Like every version, these types never change once released. Parts that did not change
//! since earlier versions reuse their types. Version 3 files are loaded by migrating them
//! to version 4 ([`super::v4`]).

use serde::{Deserialize, Serialize};

use crate::style::{FillStyle, LineStyle};

use super::v1::{CalibrationV1, ColorV1, GridV1, PointV1, UnitV1};
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
    pub line_color: ColorV1,
    pub line_opacity: f32,
    /// `None` unless closed.
    pub fill_style: FillStyleV3,
    pub fill_color: ColorV1,
    /// The real-world distance between fill lines, in `fill_unit`.
    pub fill_spacing: f64,
    pub fill_unit: UnitV1,
    /// Degrees clockwise from the image's x axis, in [0, 180).
    pub fill_angle: f64,
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
    Lines,
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
            FillStyle::Lines => Self::Lines,
            FillStyle::Crosshatch => Self::Crosshatch,
        }
    }
}

impl From<FillStyleV3> for FillStyle {
    fn from(style: FillStyleV3) -> Self {
        match style {
            FillStyleV3::None => Self::None,
            FillStyleV3::Solid => Self::Solid,
            FillStyleV3::Lines => Self::Lines,
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
