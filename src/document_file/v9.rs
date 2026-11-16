//! Version 9 of the saved-document format: version 8 with route measurements.
//!
//! Like every version, these types never change once released. Parts that did not change
//! since earlier versions reuse their types.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::document::Document;
use crate::geometry::Point;
use crate::object::{Identified, ObjectId};
use crate::route::{self, Route};
use crate::style::Styled;

use super::InvalidDocument;
use super::v1::{CalibrationV1, ColorV1, GridV1, PointV1, UnitV1};
use super::v3::LineStyleV3;
use super::v4::MeasurementV4;
use super::v5::{AreaV5, ShapeV5, stored_line};
use super::v8::DocumentV8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentV9 {
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
    pub routes: Vec<RouteV9>,
    /// The id the next added object will get.
    pub next_object_id: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteV9 {
    pub id: u64,
    /// The route's points, in order; at least two.
    pub points: Vec<PointV1>,
    pub unit: UnitV1,
    pub line_style: LineStyleV3,
    pub line_color: ColorV1,
    pub line_opacity: f32,
}

impl DocumentV9 {
    fn with_routes(file: DocumentV8, routes: Vec<RouteV9>) -> Self {
        let DocumentV8 {
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
            routes,
            next_object_id,
        }
    }

    /// Everything but the routes, which is all that version 8 has.
    fn without_routes(&self) -> DocumentV8 {
        DocumentV8 {
            image: self.image.clone(),
            width: self.width,
            height: self.height,
            origin: self.origin,
            rotation: self.rotation,
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
        let routes = document
            .routes()
            .iter()
            .map(|route| {
                let line = route.line();
                RouteV9 {
                    id: route.id().raw(),
                    points: route.points().iter().map(|&p| p.into()).collect(),
                    unit: route.unit().into(),
                    line_style: line.style.into(),
                    line_color: line.color.into(),
                    line_opacity: line.opacity,
                }
            })
            .collect();
        Self::with_routes(DocumentV8::describe(document, image), routes)
    }

    /// Re-creates the runtime document, checking everything an action would have checked.
    pub fn to_document(&self) -> Result<Document, InvalidDocument> {
        let mut document = self.without_routes().to_document()?;
        let mut route_ids = HashSet::new();
        for stored in &self.routes {
            let points = stored
                .points
                .iter()
                .map(|&point| {
                    let point = Point::from(point);
                    if document.contains(point) {
                        Ok(point)
                    } else {
                        Err(InvalidDocument::PointOutside(point))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut route = Route::new(points, stored.unit.into())
                .map_err(|route::TooFewPoints(count)| InvalidDocument::TooFewRoutePoints(count))?;
            *route.line_mut() =
                stored_line(stored.line_style, stored.line_color, stored.line_opacity)?;
            let id = stored.id;
            let taken = document.contains_object(ObjectId::new(id));
            if id == 0 || id >= self.next_object_id || taken || !route_ids.insert(id) {
                return Err(InvalidDocument::BadObjectId(id));
            }
            document.add_loaded_route(Identified::new(ObjectId::new(id), route));
        }
        Ok(document)
    }
}
