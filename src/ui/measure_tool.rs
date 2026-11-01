//! Interaction state of the measure tool: a placed start point awaiting its end point.
//! Produces document [`Action`]s but never changes the document itself. Existing
//! measurements are edited through the selection's bounding box, not this tool.

use gridtool::action::Action;
use gridtool::geometry::Point;
use gridtool::measurement::Measurement;
use gridtool::style::Styled;
use gridtool::units::LengthUnit;

use super::new_object_settings::NewObjectSettings;

#[derive(Debug)]
pub struct MeasureTool {
    pending_start: Option<Point>,
    /// What the next measurement gets.
    pub settings: NewObjectSettings,
}

impl Default for MeasureTool {
    fn default() -> Self {
        Self {
            pending_start: None,
            settings: NewObjectSettings::for_measurements(),
        }
    }
}

impl MeasureTool {
    /// The start point of a measurement whose end has not been placed yet.
    pub fn pending_start(&self) -> Option<Point> {
        self.pending_start
    }

    /// Places the next point. The first click sets the start; the second sets the end
    /// and returns the action that adds the measurement, with the tool's settings given the
    /// calibration's unit. A second click on the same spot as the start is ignored.
    pub fn place(&mut self, point: Point, calibration_unit: LengthUnit) -> Option<Action> {
        match self.pending_start {
            None => {
                self.pending_start = Some(point);
                None
            }
            Some(start) if start == point => None,
            Some(start) => {
                self.pending_start = None;
                let unit = self.settings.unit(calibration_unit);
                let mut measurement = Measurement::new(start, point, unit);
                measurement.adopt(&self.settings.style(calibration_unit));
                Some(Action::AddMeasurement(measurement))
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
    use gridtool::style::{LineStyle, StyleEdit};

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
    fn a_new_measurement_gets_the_tools_unit_and_style() {
        let mut tool = MeasureTool::default();
        tool.settings.set_unit(LengthUnit::Metre);
        tool.settings.apply(StyleEdit::LineStyle(LineStyle::Dashed));
        tool.place(Point::new(1.0, 2.0), LengthUnit::Centimetre);

        let action = tool.place(Point::new(3.0, 4.0), LengthUnit::Centimetre);

        let Some(Action::AddMeasurement(measurement)) = action else {
            panic!("expected a measurement, got {action:?}");
        };
        assert_eq!(measurement.unit(), LengthUnit::Metre);
        assert_eq!(measurement.line().style, LineStyle::Dashed);
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
