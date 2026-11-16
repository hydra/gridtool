//! Version 8 of the saved-document format: version 7 with the view rotation, so a document
//! reopens turned the way it was saved. Earlier versions load unrotated.
//!
//! Like every version, these types never change once released. Parts that did not change
//! since earlier versions reuse their types.

use serde::{Deserialize, Serialize};

use crate::angle;
use crate::document::Document;

use super::InvalidDocument;
use super::v1::{CalibrationV1, GridV1, PointV1};
use super::v4::MeasurementV4;
use super::v5::{AreaV5, DocumentV5, ShapeV5};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentV8 {
    /// The image file, relative to the document file where possible.
    pub image: String,
    /// Image size in pixels; checked against the image when loading.
    pub width: u32,
    pub height: u32,
    pub origin: PointV1,
    /// Clockwise view rotation in degrees, in [0, 360).
    pub rotation: f64,
    pub calibration: CalibrationV1,
    pub measurements: Vec<MeasurementV4>,
    pub areas: Vec<AreaV5>,
    pub grids: Vec<GridV1>,
    pub shapes: Vec<ShapeV5>,
    /// The id the next added object will get.
    pub next_object_id: u64,
}

impl DocumentV8 {
    fn with_rotation(file: DocumentV5, rotation: f64) -> Self {
        let DocumentV5 {
            image,
            width,
            height,
            origin,
            calibration,
            measurements,
            areas,
            grids,
            shapes,
            next_object_id,
        } = file;
        Self {
            image,
            width,
            height,
            origin,
            rotation,
            calibration,
            measurements,
            areas,
            grids,
            shapes,
            next_object_id,
        }
    }

    /// Everything but the rotation, which is all that version 7 has.
    fn without_rotation(&self) -> DocumentV5 {
        DocumentV5 {
            image: self.image.clone(),
            width: self.width,
            height: self.height,
            origin: self.origin,
            calibration: self.calibration.clone(),
            measurements: self.measurements.clone(),
            areas: self.areas.clone(),
            grids: self.grids.clone(),
            shapes: self.shapes.clone(),
            next_object_id: self.next_object_id,
        }
    }

    /// Describes `document`, whose image is referred to as `image`.
    pub fn describe(document: &Document, image: String) -> Self {
        Self::with_rotation(DocumentV5::describe(document, image), document.rotation())
    }

    /// Re-creates the runtime document, checking everything an action would have checked.
    pub fn to_document(&self) -> Result<Document, InvalidDocument> {
        if !angle::is_normalized_degrees(self.rotation) {
            return Err(InvalidDocument::BadRotation(self.rotation));
        }
        let mut document = self.without_rotation().to_document()?;
        document.set_rotation(self.rotation);
        Ok(document)
    }
}
