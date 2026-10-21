//! Interaction state of the scale tool: which calibration method is selected, points
//! being placed, and the point being dragged. Produces document [`Action`]s but never
//! changes the document itself.

use gridtool::action::Action;
use gridtool::calibration::{Calibration, CalibrationMethod, ScalePoints};
use gridtool::geometry::Point;

#[derive(Debug, Clone, Copy, PartialEq)]
struct PointDrag {
    index: usize,
    from: Point,
    to: Point,
}

#[derive(Debug, Default)]
pub struct ScaleTool {
    method: CalibrationMethod,
    /// Points placed so far when the document has no points for `method` yet.
    pending: Vec<Point>,
    drag: Option<PointDrag>,
}

impl ScaleTool {
    pub fn method(&self) -> CalibrationMethod {
        self.method
    }

    /// Switches calibration method, abandoning any partially placed points.
    pub fn select_method(&mut self, method: CalibrationMethod) {
        if method != self.method {
            self.method = method;
            self.pending.clear();
            self.drag = None;
        }
    }

    /// Re-aligns with the document after it changed outside this tool (undo/redo).
    pub fn sync_to(&mut self, calibration: &Calibration) {
        if let Some(points) = calibration.points() {
            self.method = points.method();
        }
        self.pending.clear();
        self.drag = None;
    }

    /// Whether the user still needs to place points for the selected method.
    pub fn is_placing(&self, calibration: &Calibration) -> bool {
        self.placed(calibration).is_none()
    }

    /// The points to display: the document's points (with any drag in progress applied),
    /// or the points placed so far.
    pub fn points(&self, calibration: &Calibration) -> Vec<Point> {
        match self.placed(calibration) {
            Some(placed) => {
                let mut points = placed.points().to_vec();
                if let Some(drag) = self.drag {
                    points[drag.index] = drag.to;
                }
                points
            }
            None => self.pending.clone(),
        }
    }

    /// Places the next point. Once all of the method's points are placed, returns the
    /// action that sets them on the document.
    pub fn place(&mut self, calibration: &Calibration, point: Point) -> Option<Action> {
        if !self.is_placing(calibration) {
            return None;
        }
        self.pending.push(point);
        if self.pending.len() < self.method.point_count() {
            return None;
        }
        let points = std::mem::take(&mut self.pending);
        ScalePoints::new(self.method, points)
            .ok()
            .map(Action::SetScalePoints)
    }

    /// Starts dragging the document's point `index`. Returns `false` if there is no such point.
    pub fn begin_drag(&mut self, calibration: &Calibration, index: usize) -> bool {
        let Some(&from) = self
            .placed(calibration)
            .and_then(|placed| placed.points().get(index))
        else {
            return false;
        };
        self.drag = Some(PointDrag {
            index,
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

    /// Finishes the drag, returning the move action if the point actually moved.
    pub fn end_drag(&mut self) -> Option<Action> {
        let drag = self.drag.take()?;
        (drag.to != drag.from).then_some(Action::MoveScalePoint {
            index: drag.index,
            to: drag.to,
        })
    }

    pub fn cancel_drag(&mut self) {
        self.drag = None;
    }

    fn placed<'a>(&self, calibration: &'a Calibration) -> Option<&'a ScalePoints> {
        calibration
            .points()
            .filter(|points| points.method() == self.method)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::document::Document;

    fn point(n: f64) -> Point {
        Point::new(n, n)
    }

    /// Places all points for the tool's method and applies the resulting action.
    fn place_all(tool: &mut ScaleTool, document: &mut Document) {
        let mut action = None;
        for n in 0..tool.method().point_count() {
            action = tool.place(document.calibration(), point(n as f64 * 10.0));
        }
        action
            .expect("last point completes placement")
            .apply(document)
            .unwrap();
    }

    #[test]
    fn starts_placing_three_points() {
        let tool = ScaleTool::default();

        assert_eq!(tool.method(), CalibrationMethod::ThreePoint);
        assert!(tool.is_placing(Document::new(100, 100).calibration()));
    }

    #[test]
    fn placing_produces_one_action_after_the_last_point() {
        let mut tool = ScaleTool::default();
        let document = Document::new(100, 100);
        let calibration = document.calibration();

        assert_eq!(tool.place(calibration, point(1.0)), None);
        assert_eq!(tool.place(calibration, point(2.0)), None);
        assert_eq!(tool.points(calibration), [point(1.0), point(2.0)]);

        let action = tool.place(calibration, point(3.0));

        let expected = ScalePoints::new(
            CalibrationMethod::ThreePoint,
            vec![point(1.0), point(2.0), point(3.0)],
        )
        .unwrap();
        assert_eq!(action, Some(Action::SetScalePoints(expected)));
    }

    #[test]
    fn four_point_method_needs_four_points() {
        let mut tool = ScaleTool::default();
        tool.select_method(CalibrationMethod::FourPoint);
        let document = Document::new(100, 100);

        for n in 0..3 {
            assert_eq!(tool.place(document.calibration(), point(n as f64)), None);
        }
        assert!(tool.place(document.calibration(), point(3.0)).is_some());
    }

    #[test]
    fn switching_method_discards_pending_points() {
        let mut tool = ScaleTool::default();
        let document = Document::new(100, 100);
        tool.place(document.calibration(), point(1.0));

        tool.select_method(CalibrationMethod::FourPoint);

        assert!(tool.points(document.calibration()).is_empty());
    }

    #[test]
    fn shows_document_points_once_placed() {
        let mut tool = ScaleTool::default();
        let mut document = Document::new(100, 100);

        place_all(&mut tool, &mut document);

        assert!(!tool.is_placing(document.calibration()));
        assert_eq!(
            tool.points(document.calibration()),
            [point(0.0), point(10.0), point(20.0)]
        );
    }

    #[test]
    fn document_points_for_another_method_are_not_shown() {
        let mut tool = ScaleTool::default();
        let mut document = Document::new(100, 100);
        place_all(&mut tool, &mut document);

        tool.select_method(CalibrationMethod::FourPoint);

        assert!(tool.is_placing(document.calibration()));
        assert!(tool.points(document.calibration()).is_empty());
    }

    #[test]
    fn dragging_previews_the_move_and_ends_with_one_action() {
        let mut tool = ScaleTool::default();
        let mut document = Document::new(100, 100);
        place_all(&mut tool, &mut document);

        assert!(tool.begin_drag(document.calibration(), 1));
        tool.drag_to(point(40.0));
        tool.drag_to(point(50.0));

        assert_eq!(tool.points(document.calibration())[1], point(50.0));
        assert_eq!(
            tool.end_drag(),
            Some(Action::MoveScalePoint {
                index: 1,
                to: point(50.0)
            })
        );
        assert!(!tool.is_dragging());
    }

    #[test]
    fn a_drag_that_ends_where_it_started_produces_no_action() {
        let mut tool = ScaleTool::default();
        let mut document = Document::new(100, 100);
        place_all(&mut tool, &mut document);

        tool.begin_drag(document.calibration(), 0);

        assert_eq!(tool.end_drag(), None);
    }

    #[test]
    fn cannot_drag_points_that_are_not_placed() {
        let mut tool = ScaleTool::default();
        let document = Document::new(100, 100);

        assert!(!tool.begin_drag(document.calibration(), 0));
    }

    #[test]
    fn sync_adopts_the_document_method_and_drops_transient_state() {
        let mut document = Document::new(100, 100);
        let mut four_point_tool = ScaleTool::default();
        four_point_tool.select_method(CalibrationMethod::FourPoint);
        place_all(&mut four_point_tool, &mut document);

        let mut tool = ScaleTool::default();
        tool.place(document.calibration(), point(1.0));
        tool.sync_to(document.calibration());

        assert_eq!(tool.method(), CalibrationMethod::FourPoint);
        assert_eq!(tool.points(document.calibration()).len(), 4);
    }
}
