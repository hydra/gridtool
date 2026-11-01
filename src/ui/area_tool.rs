//! Interaction state of the area tool: the outline being placed, one corner per click.
//! Produces document [`Action`]s but never changes the document itself. Existing areas
//! are edited through the selection (points and bounding box), not this tool.

use gridtool::action::Action;
use gridtool::area::Area;
use gridtool::geometry::Point;
use gridtool::style::Styled;
use gridtool::units::LengthUnit;

use super::new_object_settings::NewObjectSettings;
use super::pending_outline::{PendingOutline, PointHit};

#[derive(Debug)]
pub struct AreaTool {
    pub outline: PendingOutline,
    /// What the next area gets.
    pub settings: NewObjectSettings,
}

impl Default for AreaTool {
    fn default() -> Self {
        Self {
            outline: PendingOutline::default(),
            settings: NewObjectSettings::for_areas(),
        }
    }
}

impl AreaTool {
    /// A click at image point `point`, landing on placed point `hit` if any
    /// ([`PendingOutline::click`]). Returns the action that adds the area, with the tool's
    /// settings given the calibration's unit, when the click finishes the outline.
    pub fn click(
        &mut self,
        point: Point,
        hit: Option<PointHit>,
        calibration_unit: LengthUnit,
    ) -> Option<Action> {
        let points = self.outline.click(point, hit)?;
        let mut area = Area::new(points, self.settings.unit(calibration_unit)).ok()?;
        area.adopt(&self.settings.style(calibration_unit));
        Some(Action::AddArea(area))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::style::{FillStyle, LineStyle, StyleEdit};

    #[test]
    fn finishing_the_outline_adds_an_area_in_the_unit() {
        const UNIT: LengthUnit = LengthUnit::Metre;
        let mut tool = AreaTool::default();
        let corners = [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 8.0),
        ];
        for corner in corners {
            assert_eq!(tool.click(corner, None, UNIT), None);
        }

        let action = tool.click(Point::new(0.5, 0.5), Some(PointHit::First), UNIT);

        let expected = Area::new(corners.to_vec(), UNIT).unwrap();
        assert_eq!(action, Some(Action::AddArea(expected)));
        assert!(tool.outline.points().is_empty());
    }

    #[test]
    fn a_new_area_gets_the_tools_unit_and_style() {
        const UNIT: LengthUnit = LengthUnit::Centimetre;
        let mut tool = AreaTool::default();
        tool.settings.set_unit(LengthUnit::Metre);
        tool.settings.apply(StyleEdit::FillStyle(FillStyle::Lines));
        tool.settings.apply(StyleEdit::FillSpacing(25.0));
        tool.settings.apply(StyleEdit::LineStyle(LineStyle::Thin));
        for corner in [
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(5.0, 8.0),
        ] {
            tool.click(corner, None, UNIT);
        }

        let action = tool.click(Point::new(0.5, 0.5), Some(PointHit::First), UNIT);

        let Some(Action::AddArea(area)) = action else {
            panic!("expected an area, got {action:?}");
        };
        assert_eq!(area.unit(), LengthUnit::Metre);
        assert_eq!(area.line().style, LineStyle::Thin);
        let fill = area.stored_fill();
        assert_eq!(fill.style, FillStyle::Lines);
        assert_eq!(fill.spacing, 25.0);
        // The fill's unit is chosen separately, so it still follows the calibration.
        assert_eq!(fill.unit, UNIT);
    }
}
