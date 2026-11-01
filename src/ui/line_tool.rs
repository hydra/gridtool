//! Interaction state of the line drawing tool: the line being placed, one point per click.
//! It is finished by joining it up into a closed shape, or left open with a double-click
//! or Enter. Produces document [`Action`]s but never changes the document itself. Existing
//! shapes are edited through the selection (points and bounding box), not this tool.

use gridtool::action::Action;
use gridtool::geometry::Point;
use gridtool::shape::{DrawnShape, MIN_OPEN_POINTS};
use gridtool::style::Styled;
use gridtool::units::LengthUnit;

use super::new_object_settings::NewObjectSettings;
use super::pending_outline::{PendingOutline, PointHit};

#[derive(Debug)]
pub struct LineTool {
    pub outline: PendingOutline,
    /// What the next shape gets. Its fill applies only if it ends up closed.
    pub settings: NewObjectSettings,
}

impl Default for LineTool {
    fn default() -> Self {
        Self {
            outline: PendingOutline::default(),
            settings: NewObjectSettings::for_shapes(),
        }
    }
}

impl LineTool {
    /// A click at image point `point`, landing on placed point `hit` if any. A double-click
    /// on the last point leaves the line open and finishes it; anything else works as
    /// [`PendingOutline::click`] says. Returns the action that adds the shape, with the
    /// tool's settings given the calibration's unit, if the click finished it.
    pub fn click(
        &mut self,
        point: Point,
        hit: Option<PointHit>,
        double_click: bool,
        calibration_unit: LengthUnit,
    ) -> Option<Action> {
        if double_click && hit == Some(PointHit::Last) && self.can_finish_open() {
            return self.finish_open(calibration_unit);
        }
        let points = self.outline.click(point, hit)?;
        let fill_unit = self.settings.fill_unit(calibration_unit);
        let shape = DrawnShape::closed(points, fill_unit).ok()?;
        Some(self.add_with_settings(shape, calibration_unit))
    }

    /// Finishes the line as placed so far, leaving it open: the action that adds it, or
    /// `None`, keeping the points, if there are too few.
    pub fn finish_open(&mut self, calibration_unit: LengthUnit) -> Option<Action> {
        if !self.can_finish_open() {
            return None;
        }
        let fill_unit = self.settings.fill_unit(calibration_unit);
        let shape = DrawnShape::open(self.outline.take(), fill_unit).ok()?;
        Some(self.add_with_settings(shape, calibration_unit))
    }

    fn add_with_settings(&self, mut shape: DrawnShape, calibration_unit: LengthUnit) -> Action {
        shape.adopt(&self.settings.style(calibration_unit));
        Action::AddShape(shape)
    }

    fn can_finish_open(&self) -> bool {
        self.outline.points().len() >= MIN_OPEN_POINTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::style::{FillStyle, LineStyle, StyleEdit};

    const UNIT: LengthUnit = LengthUnit::Metre;
    const POINTS: [Point; 3] = [
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(5.0, 8.0),
    ];

    fn tool_with(points: &[Point]) -> LineTool {
        let mut tool = LineTool::default();
        for &point in points {
            assert_eq!(tool.click(point, None, false, UNIT), None);
        }
        tool
    }

    #[test]
    fn clicking_the_first_point_adds_a_closed_shape_with_the_given_fill_unit() {
        let mut tool = tool_with(&POINTS);

        let action = tool.click(Point::new(0.5, 0.5), Some(PointHit::First), false, UNIT);

        let expected = DrawnShape::closed(POINTS.to_vec(), UNIT).unwrap();
        assert_eq!(action, Some(Action::AddShape(expected)));
        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn finishing_adds_an_open_line() {
        let mut tool = tool_with(&POINTS);

        let expected = DrawnShape::open(POINTS.to_vec(), UNIT).unwrap();
        assert_eq!(tool.finish_open(UNIT), Some(Action::AddShape(expected)));
        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn a_new_shape_gets_the_tools_style_and_only_a_closed_one_its_fill() {
        let settings = |tool: &mut LineTool| {
            tool.settings.apply(StyleEdit::LineStyle(LineStyle::Dashed));
            tool.settings.apply(StyleEdit::FillStyle(FillStyle::Solid));
        };

        let mut open = tool_with(&POINTS);
        settings(&mut open);
        let Some(Action::AddShape(open)) = open.finish_open(UNIT) else {
            panic!("expected a shape");
        };
        assert_eq!(open.line().style, LineStyle::Dashed);
        assert_eq!(open.stored_fill().style, FillStyle::None);

        let mut closed = tool_with(&POINTS);
        settings(&mut closed);
        let action = closed.click(POINTS[0], Some(PointHit::First), false, UNIT);
        let Some(Action::AddShape(closed)) = action else {
            panic!("expected a shape, got {action:?}");
        };
        assert_eq!(closed.line().style, LineStyle::Dashed);
        assert_eq!(closed.stored_fill().style, FillStyle::Solid);
    }

    #[test]
    fn a_line_needs_two_points_to_finish() {
        let mut tool = tool_with(&POINTS[..1]);

        assert_eq!(tool.finish_open(UNIT), None);
        assert_eq!(tool.outline.points().len(), 1);
    }

    #[test]
    fn double_clicking_the_last_point_finishes_the_line_keeping_it() {
        let mut tool = tool_with(&POINTS[..2]);

        // The first click of the double-click placed the point; the second lands on it.
        let action = tool.click(POINTS[1], Some(PointHit::Last), true, UNIT);

        let expected = DrawnShape::open(POINTS[..2].to_vec(), UNIT).unwrap();
        assert_eq!(action, Some(Action::AddShape(expected)));
    }

    #[test]
    fn a_double_click_on_a_lone_point_removes_it_as_a_single_click_would() {
        let mut tool = tool_with(&POINTS[..1]);

        assert_eq!(
            tool.click(POINTS[0], Some(PointHit::Last), true, UNIT),
            None
        );
        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn a_single_click_on_the_last_point_removes_it() {
        let mut tool = tool_with(&POINTS);

        assert_eq!(
            tool.click(POINTS[2], Some(PointHit::Last), false, UNIT),
            None
        );
        assert_eq!(tool.outline.points(), &POINTS[..2]);
    }

    #[test]
    fn a_double_click_elsewhere_places_a_point() {
        let mut tool = tool_with(&POINTS[..2]);

        assert_eq!(tool.click(POINTS[2], None, true, UNIT), None);
        assert_eq!(tool.outline.points(), POINTS);
    }
}
