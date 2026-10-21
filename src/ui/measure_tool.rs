//! Interaction state of the measure tool: a placed start point awaiting its end point,
//! and the measurement handle being dragged. Produces document [`Action`]s but never
//! changes the document itself.

use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::geometry::Point;
use gridtool::measurement::{Endpoint, Measurement};
use gridtool::units::LengthUnit;

/// Identifies one end of one measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle {
    pub index: usize,
    pub endpoint: Endpoint,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct HandleDrag {
    handle: Handle,
    from: Point,
    to: Point,
}

#[derive(Debug, Default)]
pub struct MeasureTool {
    pending_start: Option<Point>,
    drag: Option<HandleDrag>,
}

impl MeasureTool {
    /// The start point of a measurement whose end has not been placed yet.
    pub fn pending_start(&self) -> Option<Point> {
        self.pending_start
    }

    /// Places the next point. The first click sets the start; the second sets the end
    /// and returns the action that adds the measurement (in `unit`). A second click on
    /// the same spot as the start is ignored.
    pub fn place(&mut self, point: Point, unit: LengthUnit) -> Option<Action> {
        match self.pending_start {
            None => {
                self.pending_start = Some(point);
                None
            }
            Some(start) if start == point => None,
            Some(start) => {
                self.pending_start = None;
                Some(Action::AddMeasurement(Measurement::new(start, point, unit)))
            }
        }
    }

    /// Where to display an end of `measurement`, taking any drag in progress into account.
    pub fn displayed_point(&self, handle: Handle, measurement: &Measurement) -> Point {
        match self.drag {
            Some(drag) if drag.handle == handle => drag.to,
            _ => measurement.point(handle.endpoint),
        }
    }

    /// Every measurement handle with its displayed position, in list order (start, end).
    pub fn handles(&self, document: &Document) -> Vec<(Handle, Point)> {
        document
            .measurements()
            .iter()
            .enumerate()
            .flat_map(|(index, measurement)| {
                Endpoint::ALL.map(|endpoint| {
                    let handle = Handle { index, endpoint };
                    (handle, self.displayed_point(handle, measurement))
                })
            })
            .collect()
    }

    /// Starts dragging `handle`. Returns `false` if that measurement does not exist.
    pub fn begin_drag(&mut self, document: &Document, handle: Handle) -> bool {
        let Some(measurement) = document.measurements().get(handle.index) else {
            return false;
        };
        let from = measurement.point(handle.endpoint);
        self.drag = Some(HandleDrag {
            handle,
            from,
            to: from,
        });
        true
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub fn drag_to(&mut self, point: Point) {
        if let Some(drag) = &mut self.drag {
            drag.to = point;
        }
    }

    /// Finishes the drag, returning the move action if the handle actually moved.
    pub fn end_drag(&mut self) -> Option<Action> {
        let drag = self.drag.take()?;
        (drag.to != drag.from).then_some(Action::MoveMeasurementPoint {
            index: drag.handle.index,
            endpoint: drag.handle.endpoint,
            to: drag.to,
        })
    }

    /// Abandons any placement or drag in progress.
    pub fn cancel(&mut self) {
        self.pending_start = None;
        self.drag = None;
    }

    /// Re-aligns with the document after it changed outside this tool (undo/redo): a
    /// drag may refer to a measurement that no longer exists, so it is dropped.
    pub fn sync_to_document(&mut self) {
        self.drag = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn document_with_one_measurement() -> Document {
        let mut document = Document::new(100, 100);
        Action::AddMeasurement(Measurement::new(
            Point::new(10.0, 10.0),
            Point::new(50.0, 10.0),
            LengthUnit::Centimetre,
        ))
        .apply(&mut document)
        .unwrap();
        document
    }

    const END_OF_FIRST: Handle = Handle {
        index: 0,
        endpoint: Endpoint::End,
    };

    #[test]
    fn two_clicks_add_one_measurement() {
        let mut tool = MeasureTool::default();

        assert_eq!(tool.place(Point::new(1.0, 2.0), LengthUnit::Metre), None);
        assert_eq!(tool.pending_start(), Some(Point::new(1.0, 2.0)));

        let action = tool.place(Point::new(3.0, 4.0), LengthUnit::Metre);

        assert_eq!(
            action,
            Some(Action::AddMeasurement(Measurement::new(
                Point::new(1.0, 2.0),
                Point::new(3.0, 4.0),
                LengthUnit::Metre
            )))
        );
        assert_eq!(tool.pending_start(), None);
    }

    #[test]
    fn clicking_the_start_again_is_ignored() {
        let mut tool = MeasureTool::default();
        tool.place(Point::new(1.0, 2.0), LengthUnit::Metre);

        assert_eq!(tool.place(Point::new(1.0, 2.0), LengthUnit::Metre), None);
        assert_eq!(tool.pending_start(), Some(Point::new(1.0, 2.0)));
    }

    #[test]
    fn lists_both_handles_of_every_measurement() {
        let tool = MeasureTool::default();
        let document = document_with_one_measurement();

        let handles = tool.handles(&document);

        assert_eq!(handles.len(), 2);
        assert_eq!(handles[1], (END_OF_FIRST, Point::new(50.0, 10.0)));
    }

    #[test]
    fn dragging_previews_and_ends_with_one_move_action() {
        let mut tool = MeasureTool::default();
        let document = document_with_one_measurement();

        assert!(tool.begin_drag(&document, END_OF_FIRST));
        tool.drag_to(Point::new(60.0, 20.0));

        assert_eq!(tool.handles(&document)[1].1, Point::new(60.0, 20.0));
        assert_eq!(
            tool.end_drag(),
            Some(Action::MoveMeasurementPoint {
                index: 0,
                endpoint: Endpoint::End,
                to: Point::new(60.0, 20.0)
            })
        );
        assert!(!tool.is_dragging());
    }

    #[test]
    fn a_drag_that_ends_where_it_started_produces_no_action() {
        let mut tool = MeasureTool::default();
        let document = document_with_one_measurement();
        tool.begin_drag(&document, END_OF_FIRST);

        assert_eq!(tool.end_drag(), None);
    }

    #[test]
    fn cannot_drag_a_missing_measurement() {
        let mut tool = MeasureTool::default();

        assert!(!tool.begin_drag(&Document::new(10, 10), END_OF_FIRST));
    }

    #[test]
    fn cancel_drops_pending_start_and_drag() {
        let mut tool = MeasureTool::default();
        let document = document_with_one_measurement();
        tool.place(Point::new(1.0, 1.0), LengthUnit::Metre);
        tool.begin_drag(&document, END_OF_FIRST);

        tool.cancel();

        assert_eq!(tool.pending_start(), None);
        assert!(!tool.is_dragging());
    }
}
