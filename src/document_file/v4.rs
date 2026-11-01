//! Version 4 of the saved-document format: version 3 with a line style for measurements,
//! and a line and fill style for areas.
//!
//! Like every version, these types never change once released. Parts that did not change
//! since earlier versions reuse their types. Version 4 files are loaded by migrating them
//! to version 5 ([`super::v5`]).

use serde::{Deserialize, Serialize};

use crate::style::DEFAULT_FILL_ANGLE;

use super::v1::{CalibrationV1, ColorV1, GridV1, PointV1, UnitV1};
use super::v2::{AreaV2, MeasurementV2};
use super::v3::{DocumentV3, FillStyleV3, LineStyleV3, ShapeV3};

/// How opaque an area's fill was before areas had a fill style.
const V3_AREA_FILL_OPACITY: f32 = 0.25;
/// The fill line distance given to areas from version 3 files, which had no fill lines.
const V3_AREA_FILL_SPACING: f64 = 10.0;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentV4 {
    /// The image file, relative to the document file where possible.
    pub image: String,
    /// Image size in pixels; checked against the image when loading.
    pub width: u32,
    pub height: u32,
    pub origin: PointV1,
    pub calibration: CalibrationV1,
    pub measurements: Vec<MeasurementV4>,
    pub areas: Vec<AreaV4>,
    pub grids: Vec<GridV1>,
    pub shapes: Vec<ShapeV3>,
    /// The id the next added object will get.
    pub next_object_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementV4 {
    pub id: u64,
    pub start: PointV1,
    pub end: PointV1,
    pub unit: UnitV1,
    pub line_style: LineStyleV3,
    pub line_color: ColorV1,
    pub line_opacity: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AreaV4 {
    pub id: u64,
    /// The outline's points, in order; at least three.
    pub points: Vec<PointV1>,
    pub unit: UnitV1,
    pub line_style: LineStyleV3,
    pub line_color: ColorV1,
    pub line_opacity: f32,
    pub fill_style: FillStyleV3,
    pub fill_color: ColorV1,
    /// The real-world distance between fill lines, in `fill_unit`.
    pub fill_spacing: f64,
    pub fill_unit: UnitV1,
    /// Degrees clockwise from the image's x axis, in [0, 180).
    pub fill_angle: f64,
    pub fill_opacity: f32,
}

/// Version 3 drew measurements with a thick opaque line in their colour.
impl From<MeasurementV2> for MeasurementV4 {
    fn from(measurement: MeasurementV2) -> Self {
        Self {
            id: measurement.id,
            start: measurement.start,
            end: measurement.end,
            unit: measurement.unit,
            line_style: LineStyleV3::Thick,
            line_color: measurement.color,
            line_opacity: 1.0,
        }
    }
}

/// Version 3 drew areas with a thick opaque line and a translucent solid fill, both in
/// their colour.
impl From<AreaV2> for AreaV4 {
    fn from(area: AreaV2) -> Self {
        Self {
            id: area.id,
            points: area.points,
            unit: area.unit,
            line_style: LineStyleV3::Thick,
            line_color: area.color,
            line_opacity: 1.0,
            fill_style: FillStyleV3::Solid,
            fill_color: area.color,
            fill_spacing: V3_AREA_FILL_SPACING,
            fill_unit: area.unit,
            fill_angle: DEFAULT_FILL_ANGLE,
            fill_opacity: V3_AREA_FILL_OPACITY,
        }
    }
}

impl From<DocumentV3> for DocumentV4 {
    fn from(file: DocumentV3) -> Self {
        Self {
            image: file.image,
            width: file.width,
            height: file.height,
            origin: file.origin,
            calibration: file.calibration,
            measurements: file.measurements.into_iter().map(Into::into).collect(),
            areas: file.areas.into_iter().map(Into::into).collect(),
            grids: file.grids,
            shapes: file.shapes,
            next_object_id: file.next_object_id,
        }
    }
}
