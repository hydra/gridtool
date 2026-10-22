//! Dragging the active object's points or bounding-box handles: the live preview while
//! the pointer moves, and the document [`Action`] committed when it is released.

use gridtool::action::Action;
use gridtool::area::Area;
use gridtool::document::Document;
use gridtool::geometry::{Bounds, BoundsHandle, Point};
use gridtool::measurement::{Endpoint, Measurement};
use gridtool::object::ObjectId;

/// The part of the active object a drag holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grip {
    /// One of the selection box's resize handles: every point stretches with the box.
    Bounds(BoundsHandle),
    /// One of the object's own points (for a measurement, 0 is the start and 1 the end).
    Point(usize),
}

/// The shape of an object that can be dragged.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Measurement(Measurement),
    Area(Area),
}

impl Shape {
    /// The shape of the object with `id`, if it is a measurement or an area.
    pub fn of(document: &Document, id: ObjectId) -> Option<Shape> {
        if let Some(index) = document.measurement_index(id) {
            return Some(Shape::Measurement(
                (*document.measurements()[index]).clone(),
            ));
        }
        let index = document.area_index(id)?;
        Some(Shape::Area((*document.areas()[index]).clone()))
    }

    /// The draggable points, in order.
    pub fn points(&self) -> Vec<Point> {
        match self {
            Shape::Measurement(measurement) => Endpoint::ALL
                .map(|endpoint| measurement.point(endpoint))
                .to_vec(),
            Shape::Area(area) => area.points().to_vec(),
        }
    }

    pub fn bounds(&self) -> Bounds {
        match self {
            Shape::Measurement(measurement) => measurement.bounds(),
            Shape::Area(area) => area.bounds(),
        }
    }

    /// The shape after `grip` is moved by `(dx, dy)` image pixels, with every point kept
    /// inside `document`.
    fn dragged(&self, document: &Document, grip: Grip, dx: f64, dy: f64) -> Shape {
        let clamp = |point: Point| document.clamp(point);
        let moved = |point: Point| Point::new(point.x + dx, point.y + dy);
        match (self, grip) {
            (Shape::Measurement(measurement), Grip::Bounds(handle)) => {
                Shape::Measurement(measurement.resized(handle, dx, dy).map_points(clamp))
            }
            (Shape::Measurement(measurement), Grip::Point(index)) => {
                let endpoint = endpoint(index);
                let to = clamp(moved(measurement.point(endpoint)));
                Shape::Measurement(measurement.with_point(endpoint, to))
            }
            (Shape::Area(area), Grip::Bounds(handle)) => {
                Shape::Area(area.resized(handle, dx, dy).map_points(clamp))
            }
            (Shape::Area(area), Grip::Point(index)) => match area.points().get(index) {
                Some(&point) => Shape::Area(area.with_point(index, clamp(moved(point)))),
                None => self.clone(),
            },
        }
    }

    /// The action that turns the object at list position `index` from `self` into
    /// `edited` by way of `grip`.
    fn edit_action(&self, edited: &Shape, index: usize, grip: Grip) -> Option<Action> {
        if edited == self {
            return None;
        }
        Some(match (edited, grip) {
            (Shape::Measurement(measurement), Grip::Bounds(_)) => Action::ResizeMeasurement {
                index,
                start: measurement.start(),
                end: measurement.end(),
            },
            (Shape::Measurement(measurement), Grip::Point(point)) => {
                let endpoint = endpoint(point);
                Action::MoveMeasurementPoint {
                    index,
                    endpoint,
                    to: measurement.point(endpoint),
                }
            }
            (Shape::Area(area), Grip::Bounds(_)) => Action::ResizeArea {
                index,
                points: area.points().to_vec(),
            },
            (Shape::Area(area), Grip::Point(point)) => Action::MoveAreaPoint {
                index,
                point,
                to: *area.points().get(point)?,
            },
        })
    }

    /// The list position of `id` in the list this kind of shape lives in.
    fn index_in(&self, document: &Document, id: ObjectId) -> Option<usize> {
        match self {
            Shape::Measurement(_) => document.measurement_index(id),
            Shape::Area(_) => document.area_index(id),
        }
    }
}

fn endpoint(index: usize) -> Endpoint {
    if index == 0 {
        Endpoint::Start
    } else {
        Endpoint::End
    }
}

/// A drag of one grip of the active object, from the press to the pointer's current
/// position.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectDrag {
    grip: Grip,
    press: Point,
    original: Shape,
    preview: Shape,
}

impl ObjectDrag {
    /// Starts dragging `grip` of `shape`, with the pointer pressed at image point `press`.
    pub fn new(shape: Shape, grip: Grip, press: Point) -> Self {
        Self {
            grip,
            press,
            preview: shape.clone(),
            original: shape,
        }
    }

    /// The shape as it would be if the drag ended now.
    pub fn preview(&self) -> &Shape {
        &self.preview
    }

    /// Updates the preview for the pointer at image point `pointer`: the grip moves by
    /// the pointer's movement since the press.
    pub fn drag_to(&mut self, document: &Document, pointer: Point) {
        self.preview = self.original.dragged(
            document,
            self.grip,
            pointer.x - self.press.x,
            pointer.y - self.press.y,
        );
    }

    /// The action for the drag of object `id`, if it changed anything.
    pub fn finish(self, document: &Document, id: ObjectId) -> Option<Action> {
        let index = self.original.index_in(document, id)?;
        self.original.edit_action(&self.preview, index, self.grip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::units::LengthUnit;

    /// A 100 × 100 document with the measurement (10, 10)–(40, 30) and the triangle
    /// (50, 50), (90, 50), (70, 90).
    fn document() -> Document {
        let mut document = Document::new(100, 100);
        Action::AddMeasurement(Measurement::new(
            Point::new(10.0, 10.0),
            Point::new(40.0, 30.0),
            LengthUnit::Centimetre,
        ))
        .apply(&mut document)
        .unwrap();
        Action::AddArea(
            Area::new(
                vec![
                    Point::new(50.0, 50.0),
                    Point::new(90.0, 50.0),
                    Point::new(70.0, 90.0),
                ],
                LengthUnit::Centimetre,
            )
            .unwrap(),
        )
        .apply(&mut document)
        .unwrap();
        document
    }

    fn drag(
        document: &Document,
        id: ObjectId,
        grip: Grip,
        press: Point,
        to: Point,
    ) -> Option<Action> {
        let mut drag = ObjectDrag::new(Shape::of(document, id).unwrap(), grip, press);
        drag.drag_to(document, to);
        drag.finish(document, id)
    }

    #[test]
    fn dragging_a_measurement_point_moves_that_end_by_the_pointer_movement() {
        let document = document();
        let id = document.measurements()[0].id();

        // Grabbed 2 px off the end's centre; the end moves by the movement, not to the
        // pointer.
        let action = drag(
            &document,
            id,
            Grip::Point(1),
            Point::new(42.0, 30.0),
            Point::new(52.0, 35.0),
        );

        assert_eq!(
            action,
            Some(Action::MoveMeasurementPoint {
                index: 0,
                endpoint: Endpoint::End,
                to: Point::new(50.0, 35.0),
            })
        );
    }

    #[test]
    fn dragging_an_area_point_moves_only_that_point_and_stays_inside() {
        let document = document();
        let id = document.areas()[0].id();

        let action = drag(
            &document,
            id,
            Grip::Point(2),
            Point::new(70.0, 90.0),
            Point::new(75.0, 150.0),
        );

        assert_eq!(
            action,
            Some(Action::MoveAreaPoint {
                index: 0,
                point: 2,
                to: Point::new(75.0, 100.0),
            })
        );
    }

    #[test]
    fn dragging_an_area_handle_stretches_every_point() {
        let document = document();
        let id = document.areas()[0].id();

        // The bottom edge from y = 90 up to y = 70 halves the height.
        let action = drag(
            &document,
            id,
            Grip::Bounds(BoundsHandle::Bottom),
            Point::new(70.0, 90.0),
            Point::new(70.0, 70.0),
        );

        assert_eq!(
            action,
            Some(Action::ResizeArea {
                index: 0,
                points: vec![
                    Point::new(50.0, 50.0),
                    Point::new(90.0, 50.0),
                    Point::new(70.0, 70.0),
                ],
            })
        );
    }

    #[test]
    fn dragging_a_measurement_handle_resizes_it() {
        let document = document();
        let id = document.measurements()[0].id();

        let action = drag(
            &document,
            id,
            Grip::Bounds(BoundsHandle::Right),
            Point::new(40.0, 20.0),
            Point::new(60.0, 0.0),
        );

        assert_eq!(
            action,
            Some(Action::ResizeMeasurement {
                index: 0,
                start: Point::new(10.0, 10.0),
                end: Point::new(60.0, 30.0),
            })
        );
    }

    #[test]
    fn a_drag_that_changes_nothing_produces_no_action() {
        let document = document();
        let id = document.areas()[0].id();
        let press = Point::new(50.0, 50.0);

        assert_eq!(drag(&document, id, Grip::Point(0), press, press), None);
    }

    #[test]
    fn the_preview_follows_the_pointer() {
        let document = document();
        let id = document.areas()[0].id();
        let mut drag = ObjectDrag::new(
            Shape::of(&document, id).unwrap(),
            Grip::Point(0),
            Point::new(50.0, 50.0),
        );

        drag.drag_to(&document, Point::new(55.0, 45.0));

        assert_eq!(drag.preview().points()[0], Point::new(55.0, 45.0));
        assert_eq!(
            drag.preview().bounds(),
            Bounds::around(Point::new(55.0, 45.0), Point::new(90.0, 90.0))
        );
    }

    #[test]
    fn grids_have_no_shape() {
        let mut document = document();
        let source = gridtool::grid::GridSource::Measurement(document.measurements()[0].id());
        Action::AddGrid(gridtool::grid::Grid::new(source, LengthUnit::Centimetre))
            .apply(&mut document)
            .unwrap();

        assert_eq!(Shape::of(&document, document.grids()[0].id()), None);
    }
}
