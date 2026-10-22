//! Version 2 of the saved-document format: version 1 plus measurement colours and area
//! measurements.
//!
//! Like every version, these types never change once released. Parts that did not change
//! since version 1 reuse its types.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::area::{self, Area};
use crate::calibration::{Calibration, Line, ScalePoints};
use crate::document::{Document, Objects};
use crate::geometry::Point;
use crate::grid::{CellDimension, Grid};
use crate::measurement::{self, Measurement};
use crate::object::{Identified, ObjectId};

use super::InvalidDocument;
use super::v1::{CalibrationV1, ColorV1, DocumentV1, GridV1, PointV1, ScalePointsV1, UnitV1};

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

impl DocumentV2 {
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
            Objects {
                measurements,
                areas,
                grids,
            },
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
