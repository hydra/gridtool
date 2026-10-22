//! Interaction state of the area tool: the outline being placed, one corner per click.
//! Produces document [`Action`]s but never changes the document itself. Existing areas
//! are edited through the selection (points and bounding box), not this tool.

use eframe::egui::Pos2;
use gridtool::action::Action;
use gridtool::area::{Area, MIN_POINTS};
use gridtool::geometry::Point;
use gridtool::units::LengthUnit;

use super::hit_test::nearest_within;

/// A placed point that the pointer is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointHit {
    /// The first point: clicking it closes the outline once there are enough points.
    First,
    /// The most recently placed point: clicking it removes it.
    Last,
}

#[derive(Debug, Default)]
pub struct AreaTool {
    points: Vec<Point>,
}

impl AreaTool {
    /// The points placed so far, in order.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Whether clicking the first point would finish the outline.
    pub fn can_close(&self) -> bool {
        self.points.len() >= MIN_POINTS
    }

    /// The placed point within `radius` of `pointer`, given the placed points' screen
    /// positions. When the first and last point are both in range the nearer one wins;
    /// with a single point, it counts as the last.
    pub fn hit_at(on_screen: &[Pos2], pointer: Pos2, radius: f32) -> Option<PointHit> {
        let (&first, &last) = (on_screen.first()?, on_screen.last()?);
        let candidates = [last, first];
        nearest_within(&candidates, pointer, radius).map(|index| match index {
            0 => PointHit::Last,
            _ => PointHit::First,
        })
    }

    /// A click at image point `point`, landing on placed point `hit` if any:
    /// - on the first point, with at least three points: finishes the outline and returns
    ///   the action that adds the area (in `unit`)
    /// - on the last point: removes it
    /// - on the first point with too few points: ignored
    /// - elsewhere: places the next point
    pub fn click(
        &mut self,
        point: Point,
        hit: Option<PointHit>,
        unit: LengthUnit,
    ) -> Option<Action> {
        match hit {
            Some(PointHit::First) if self.can_close() => {
                let points = std::mem::take(&mut self.points);
                Area::new(points, unit).ok().map(Action::AddArea)
            }
            Some(PointHit::Last) => {
                self.remove_last_point();
                None
            }
            Some(PointHit::First) => None,
            None => {
                self.points.push(point);
                None
            }
        }
    }

    /// Removes the most recently placed point, if any.
    pub fn remove_last_point(&mut self) {
        self.points.pop();
    }

    /// Abandons the outline being placed.
    pub fn cancel(&mut self) {
        self.points.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNIT: LengthUnit = LengthUnit::Metre;

    fn place(tool: &mut AreaTool, coordinates: &[(f64, f64)]) {
        for &(x, y) in coordinates {
            assert_eq!(tool.click(Point::new(x, y), None, UNIT), None);
        }
    }

    #[test]
    fn clicking_the_first_point_finishes_the_outline() {
        let mut tool = AreaTool::default();
        place(&mut tool, &[(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)]);

        let action = tool.click(Point::new(0.5, 0.5), Some(PointHit::First), UNIT);

        let expected = Area::new(
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(5.0, 8.0),
            ],
            UNIT,
        )
        .unwrap();
        assert_eq!(action, Some(Action::AddArea(expected)));
        assert!(tool.points().is_empty());
    }

    #[test]
    fn the_first_point_cannot_finish_fewer_than_three_points() {
        let mut tool = AreaTool::default();
        place(&mut tool, &[(0.0, 0.0), (10.0, 0.0)]);

        assert!(!tool.can_close());
        assert_eq!(
            tool.click(Point::default(), Some(PointHit::First), UNIT),
            None
        );
        assert_eq!(tool.points().len(), 2);
    }

    #[test]
    fn clicking_the_last_point_or_backspace_removes_it() {
        let mut tool = AreaTool::default();
        place(&mut tool, &[(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)]);

        tool.click(Point::new(5.0, 8.0), Some(PointHit::Last), UNIT);
        assert_eq!(tool.points().len(), 2);

        tool.remove_last_point();
        assert_eq!(tool.points(), [Point::new(0.0, 0.0)]);

        tool.cancel();
        assert!(tool.points().is_empty());
        tool.remove_last_point();
        assert!(tool.points().is_empty());
    }

    #[test]
    fn hits_the_nearer_of_the_first_and_last_point() {
        let on_screen = [
            Pos2::new(0.0, 0.0),
            Pos2::new(100.0, 0.0),
            Pos2::new(12.0, 0.0),
        ];

        assert_eq!(
            AreaTool::hit_at(&on_screen, Pos2::new(3.0, 0.0), 10.0),
            Some(PointHit::First)
        );
        assert_eq!(
            AreaTool::hit_at(&on_screen, Pos2::new(9.0, 0.0), 10.0),
            Some(PointHit::Last)
        );
        assert_eq!(
            AreaTool::hit_at(&on_screen, Pos2::new(100.0, 0.0), 10.0),
            None
        );
        assert_eq!(AreaTool::hit_at(&[], Pos2::ZERO, 10.0), None);
    }

    #[test]
    fn a_single_point_counts_as_the_last() {
        assert_eq!(
            AreaTool::hit_at(&[Pos2::ZERO], Pos2::ZERO, 10.0),
            Some(PointHit::Last)
        );
    }
}
