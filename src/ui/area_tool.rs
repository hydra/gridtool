//! Interaction state of the area tool: the outline being placed, one corner per click.
//! Produces document [`Action`]s but never changes the document itself. Existing areas
//! are edited through the selection (points and bounding box), not this tool.

use gridtool::action::Action;
use gridtool::area::Area;
use gridtool::geometry::Point;
use gridtool::units::LengthUnit;

use super::pending_outline::{PendingOutline, PointHit};

#[derive(Debug, Default)]
pub struct AreaTool {
    pub outline: PendingOutline,
}

impl AreaTool {
    /// A click at image point `point`, landing on placed point `hit` if any
    /// ([`PendingOutline::click`]). Returns the action that adds the area, in `unit`, when
    /// the click finishes the outline.
    pub fn click(
        &mut self,
        point: Point,
        hit: Option<PointHit>,
        unit: LengthUnit,
    ) -> Option<Action> {
        let points = self.outline.click(point, hit)?;
        Area::new(points, unit).ok().map(Action::AddArea)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
