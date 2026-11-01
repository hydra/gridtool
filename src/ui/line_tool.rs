//! Interaction state of the line drawing tool: the line being placed, one point per click.
//! It is finished by joining it up into a closed shape, or left open with a double-click
//! or Enter. Produces document [`Action`]s but never changes the document itself. Existing
//! shapes are edited through the selection (points and bounding box), not this tool.

use gridtool::action::Action;
use gridtool::geometry::Point;
use gridtool::shape::{DrawnShape, MIN_OPEN_POINTS};

use super::pending_outline::{PendingOutline, PointHit};

#[derive(Debug, Default)]
pub struct LineTool {
    pub outline: PendingOutline,
}

impl LineTool {
    /// A click at image point `point`, landing on placed point `hit` if any. A double-click
    /// on the last point leaves the line open and finishes it; anything else works as
    /// [`PendingOutline::click`] says. Returns the action that adds the shape, if the click
    /// finished it.
    pub fn click(
        &mut self,
        point: Point,
        hit: Option<PointHit>,
        double_click: bool,
    ) -> Option<Action> {
        if double_click && hit == Some(PointHit::Last) && self.can_finish_open() {
            return self.finish_open();
        }
        let points = self.outline.click(point, hit)?;
        DrawnShape::closed(points).ok().map(Action::AddShape)
    }

    /// Finishes the line as placed so far, leaving it open: the action that adds it, or
    /// `None`, keeping the points, if there are too few.
    pub fn finish_open(&mut self) -> Option<Action> {
        if !self.can_finish_open() {
            return None;
        }
        DrawnShape::open(self.outline.take())
            .ok()
            .map(Action::AddShape)
    }

    fn can_finish_open(&self) -> bool {
        self.outline.points().len() >= MIN_OPEN_POINTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POINTS: [Point; 3] = [
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(5.0, 8.0),
    ];

    fn tool_with(points: &[Point]) -> LineTool {
        let mut tool = LineTool::default();
        for &point in points {
            assert_eq!(tool.click(point, None, false), None);
        }
        tool
    }

    #[test]
    fn clicking_the_first_point_adds_a_closed_shape() {
        let mut tool = tool_with(&POINTS);

        let action = tool.click(Point::new(0.5, 0.5), Some(PointHit::First), false);

        let expected = DrawnShape::closed(POINTS.to_vec()).unwrap();
        assert_eq!(action, Some(Action::AddShape(expected)));
        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn finishing_adds_an_open_line() {
        let mut tool = tool_with(&POINTS);

        let expected = DrawnShape::open(POINTS.to_vec()).unwrap();
        assert_eq!(tool.finish_open(), Some(Action::AddShape(expected)));
        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn a_line_needs_two_points_to_finish() {
        let mut tool = tool_with(&POINTS[..1]);

        assert_eq!(tool.finish_open(), None);
        assert_eq!(tool.outline.points().len(), 1);
    }

    #[test]
    fn double_clicking_the_last_point_finishes_the_line_keeping_it() {
        let mut tool = tool_with(&POINTS[..2]);

        // The first click of the double-click placed the point; the second lands on it.
        let action = tool.click(POINTS[1], Some(PointHit::Last), true);

        let expected = DrawnShape::open(POINTS[..2].to_vec()).unwrap();
        assert_eq!(action, Some(Action::AddShape(expected)));
    }

    #[test]
    fn a_double_click_on_a_lone_point_removes_it_as_a_single_click_would() {
        let mut tool = tool_with(&POINTS[..1]);

        assert_eq!(tool.click(POINTS[0], Some(PointHit::Last), true), None);
        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn a_single_click_on_the_last_point_removes_it() {
        let mut tool = tool_with(&POINTS);

        assert_eq!(tool.click(POINTS[2], Some(PointHit::Last), false), None);
        assert_eq!(tool.outline.points(), &POINTS[..2]);
    }

    #[test]
    fn a_double_click_elsewhere_places_a_point() {
        let mut tool = tool_with(&POINTS[..2]);

        assert_eq!(tool.click(POINTS[2], None, true), None);
        assert_eq!(tool.outline.points(), POINTS);
    }
}
