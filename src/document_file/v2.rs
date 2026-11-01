//! Version 2 of the saved-document format: version 1 plus measurement colours and area
//! measurements.
//!
//! Like every version, these types never change once released. Parts that did not change
//! since version 1 reuse its types. Version 2 files are loaded by migrating them to
//! version 3 ([`super::v3`]).

use serde::{Deserialize, Serialize};

use crate::measurement;

use super::v1::{CalibrationV1, ColorV1, DocumentV1, GridV1, PointV1, UnitV1};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentV2 {
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
    /// The id the next added object will get.
    pub next_object_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementV2 {
    pub id: u64,
    pub start: PointV1,
    pub end: PointV1,
    pub unit: UnitV1,
    pub color: ColorV1,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AreaV2 {
    pub id: u64,
    /// The outline's corners, in order; at least three.
    pub points: Vec<PointV1>,
    pub unit: UnitV1,
    pub color: ColorV1,
}

/// Version 1 had no colours on measurements (they were all the default) and no areas.
impl From<DocumentV1> for DocumentV2 {
    fn from(file: DocumentV1) -> Self {
        Self {
            image: file.image,
            width: file.width,
            height: file.height,
            origin: file.origin,
            calibration: file.calibration,
            measurements: file
                .measurements
                .into_iter()
                .map(|stored| MeasurementV2 {
                    id: stored.id,
                    start: stored.start,
                    end: stored.end,
                    unit: stored.unit,
                    color: measurement::DEFAULT_COLOR.into(),
                })
                .collect(),
            areas: Vec::new(),
            grids: file.grids,
            next_object_id: file.next_object_id,
        }
    }
}
