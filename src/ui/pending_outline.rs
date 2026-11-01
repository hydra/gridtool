//! The points of an outline being placed, one per click, shared by the tools that draw
//! outlines. Presentation state: the document only changes once the outline is finished
//! and its tool turns it into an action.

use eframe::egui::Pos2;
use gridtool::geometry::Point;

use super::hit_test::nearest_within;

/// The fewest points an outline must have to be closed by clicking its first point.
const MIN_POINTS_TO_CLOSE: usize = 3;

/// A placed point that the pointer is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointHit {
    /// The first point: clicking it closes the outline once there are enough points.
    First,
    /// The most recently placed point: clicking it removes it.
    Last,
}

#[derive(Debug, Default)]
pub struct PendingOutline {
    points: Vec<Point>,
}

impl PendingOutline {
    /// The points placed so far, in order.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Whether clicking the first point would finish the outline.
    pub fn can_close(&self) -> bool {
        self.points.len() >= MIN_POINTS_TO_CLOSE
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
    /// - on the first point, with at least three points: finishes the outline, returning
    ///   its points and leaving none placed
    /// - on the last point: removes it
    /// - on the first point with too few points: ignored
    /// - elsewhere: places the next point
    pub fn click(&mut self, point: Point, hit: Option<PointHit>) -> Option<Vec<Point>> {
        match hit {
            Some(PointHit::First) if self.can_close() => Some(self.take()),
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

    /// Takes every placed point, leaving none.
    pub fn take(&mut self) -> Vec<Point> {
        std::mem::take(&mut self.points)
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

    fn place(outline: &mut PendingOutline, coordinates: &[(f64, f64)]) {
        for &(x, y) in coordinates {
            assert_eq!(outline.click(Point::new(x, y), None), None);
        }
    }

    #[test]
    fn clicking_the_first_point_finishes_the_outline() {
        let mut outline = PendingOutline::default();
        place(&mut outline, &[(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)]);

        let finished = outline.click(Point::new(0.5, 0.5), Some(PointHit::First));

        assert_eq!(
            finished,
            Some(vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(5.0, 8.0),
            ])
        );
        assert!(outline.points().is_empty());
    }

    #[test]
    fn the_first_point_cannot_finish_fewer_than_three_points() {
        let mut outline = PendingOutline::default();
        place(&mut outline, &[(0.0, 0.0), (10.0, 0.0)]);

        assert!(!outline.can_close());
        assert_eq!(outline.click(Point::default(), Some(PointHit::First)), None);
        assert_eq!(outline.points().len(), 2);
    }

    #[test]
    fn clicking_the_last_point_or_backspace_removes_it() {
        let mut outline = PendingOutline::default();
        place(&mut outline, &[(0.0, 0.0), (10.0, 0.0), (5.0, 8.0)]);

        outline.click(Point::new(5.0, 8.0), Some(PointHit::Last));
        assert_eq!(outline.points().len(), 2);

        outline.remove_last_point();
        assert_eq!(outline.points(), [Point::new(0.0, 0.0)]);

        outline.cancel();
        assert!(outline.points().is_empty());
        outline.remove_last_point();
        assert!(outline.points().is_empty());
    }

    #[test]
    fn hits_the_nearer_of_the_first_and_last_point() {
        let on_screen = [
            Pos2::new(0.0, 0.0),
            Pos2::new(100.0, 0.0),
            Pos2::new(12.0, 0.0),
        ];

        assert_eq!(
            PendingOutline::hit_at(&on_screen, Pos2::new(3.0, 0.0), 10.0),
            Some(PointHit::First)
        );
        assert_eq!(
            PendingOutline::hit_at(&on_screen, Pos2::new(9.0, 0.0), 10.0),
            Some(PointHit::Last)
        );
        assert_eq!(
            PendingOutline::hit_at(&on_screen, Pos2::new(100.0, 0.0), 10.0),
            None
        );
        assert_eq!(PendingOutline::hit_at(&[], Pos2::ZERO, 10.0), None);
    }

    #[test]
    fn a_single_point_counts_as_the_last() {
        assert_eq!(
            PendingOutline::hit_at(&[Pos2::ZERO], Pos2::ZERO, 10.0),
            Some(PointHit::Last)
        );
    }
}
