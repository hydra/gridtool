//! Interaction state of the Measure route tool: the route being placed, one point per click.
//! A route is never closed. It is finished with a double-click on its last point, or with
//! Enter. Produces document [`Action`]s but never changes the document itself. Existing
//! routes are edited through the selection (points and bounding box) and the Routes panel.

use gridtool::action::Action;
use gridtool::geometry::Point;
use gridtool::route::{MIN_POINTS, Route};
use gridtool::style::Styled;
use gridtool::units::LengthUnit;

use super::new_object_settings::NewObjectSettings;
use super::pending_outline::{PendingOutline, PointHit};

#[derive(Debug)]
pub struct RouteTool {
    pub outline: PendingOutline,
    /// What the next route gets.
    pub settings: NewObjectSettings,
}

impl Default for RouteTool {
    fn default() -> Self {
        Self {
            outline: PendingOutline::default(),
            settings: NewObjectSettings::for_routes(),
        }
    }
}

impl RouteTool {
    /// A click at image point `point`, landing on placed point `hit` if any:
    /// - a double-click on the last point, with at least two points placed, finishes the route
    /// - a single click on the last point removes it
    /// - anywhere else, including on the first point, places the next point, so a route can
    ///   come back to where it started
    ///
    /// Returns the action that adds the route, with the tool's settings given the
    /// calibration's unit, if the click finished it.
    pub fn click(
        &mut self,
        point: Point,
        hit: Option<PointHit>,
        double_click: bool,
        calibration_unit: LengthUnit,
    ) -> Option<Action> {
        let on_last = hit == Some(PointHit::Last);
        if double_click && on_last && self.can_finish() {
            return self.finish(calibration_unit);
        }
        // A route is never closed, so its first point is just somewhere to go.
        let hit = hit.filter(|_| on_last);
        self.outline.click(point, hit);
        None
    }

    /// Finishes the route as placed so far: the action that adds it, or `None`, keeping the
    /// points, if there are too few.
    pub fn finish(&mut self, calibration_unit: LengthUnit) -> Option<Action> {
        if !self.can_finish() {
            return None;
        }
        let unit = self.settings.unit(calibration_unit);
        let mut route = Route::new(self.outline.take(), unit).ok()?;
        route.adopt(&self.settings.style(calibration_unit));
        Some(Action::AddRoute(route))
    }

    fn can_finish(&self) -> bool {
        self.outline.points().len() >= MIN_POINTS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::style::{LineStyle, StyleEdit};

    const UNIT: LengthUnit = LengthUnit::Metre;
    const POINTS: [Point; 3] = [
        Point::new(0.0, 0.0),
        Point::new(10.0, 0.0),
        Point::new(10.0, 8.0),
    ];

    fn tool_with(points: &[Point]) -> RouteTool {
        let mut tool = RouteTool::default();
        for &point in points {
            assert_eq!(tool.click(point, None, false, UNIT), None);
        }
        tool
    }

    #[test]
    fn double_clicking_the_last_point_adds_the_route_in_the_tools_unit() {
        let mut tool = tool_with(&POINTS);

        let action = tool.click(POINTS[2], Some(PointHit::Last), true, UNIT);

        let expected = Route::new(POINTS.to_vec(), UNIT).unwrap();
        assert_eq!(action, Some(Action::AddRoute(expected)));
        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn clicking_the_last_point_once_removes_it() {
        let mut tool = tool_with(&POINTS);

        assert_eq!(
            tool.click(POINTS[2], Some(PointHit::Last), false, UNIT),
            None
        );

        assert_eq!(tool.outline.points(), &POINTS[..2]);
    }

    #[test]
    fn a_double_click_on_a_lone_point_removes_it() {
        let mut tool = tool_with(&POINTS[..1]);

        assert_eq!(
            tool.click(POINTS[0], Some(PointHit::Last), true, UNIT),
            None
        );

        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn clicking_the_first_point_places_another_point_instead_of_closing() {
        let mut tool = tool_with(&POINTS);

        let back = Point::new(0.5, 0.5);
        assert_eq!(tool.click(back, Some(PointHit::First), false, UNIT), None);

        assert_eq!(tool.outline.points().len(), 4);
        assert_eq!(tool.outline.points()[3], back);
    }

    #[test]
    fn finishing_needs_at_least_one_segment() {
        let mut lone = tool_with(&POINTS[..1]);
        assert_eq!(lone.finish(UNIT), None);
        assert_eq!(lone.outline.points().len(), 1);

        let mut tool = tool_with(&POINTS[..2]);
        let expected = Route::new(POINTS[..2].to_vec(), UNIT).unwrap();
        assert_eq!(tool.finish(UNIT), Some(Action::AddRoute(expected)));
    }

    #[test]
    fn a_new_route_gets_the_tools_unit_and_line() {
        let mut tool = tool_with(&POINTS);
        tool.settings.set_unit(LengthUnit::Centimetre);
        tool.settings.apply(StyleEdit::LineStyle(LineStyle::Dashed));

        let Some(Action::AddRoute(route)) = tool.finish(UNIT) else {
            panic!("expected a route");
        };
        assert_eq!(route.unit(), LengthUnit::Centimetre);
        assert_eq!(route.line().style, LineStyle::Dashed);
        assert_eq!(route.line().color, gridtool::route::DEFAULT_COLOR);
    }
}
