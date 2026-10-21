//! Interaction state of the measure tool: a placed start point awaiting its end point.
//! Produces document [`Action`]s but never changes the document itself. Existing
//! measurements are edited through the selection's bounding box, not this tool.

use gridtool::action::Action;
use gridtool::geometry::Point;
use gridtool::measurement::Measurement;
use gridtool::units::LengthUnit;

#[derive(Debug, Default)]
pub struct MeasureTool {
    pending_start: Option<Point>,
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

    /// Abandons a placement in progress.
    pub fn cancel(&mut self) {
        self.pending_start = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn cancel_drops_the_pending_start() {
        let mut tool = MeasureTool::default();
        tool.place(Point::new(1.0, 1.0), LengthUnit::Metre);

        tool.cancel();

        assert_eq!(tool.pending_start(), None);
    }
}
