use std::fmt;

/// A position in document (image) pixel coordinates: origin at the top-left corner of
/// the un-rotated image, x to the right, y down.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

/// An axis-aligned rectangle in image coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}

impl Bounds {
    /// The smallest bounds containing both points.
    pub fn around(a: Point, b: Point) -> Self {
        Self {
            min: Point::new(a.x.min(b.x), a.y.min(b.y)),
            max: Point::new(a.x.max(b.x), a.y.max(b.y)),
        }
    }

    /// The smallest bounds containing every point, or `None` if there are none.
    pub fn enclosing(points: impl IntoIterator<Item = Point>) -> Option<Self> {
        let mut points = points.into_iter();
        let first = points.next()?;
        Some(
            points.fold(Self::around(first, first), |bounds, point| Self {
                min: Point::new(bounds.min.x.min(point.x), bounds.min.y.min(point.y)),
                max: Point::new(bounds.max.x.max(point.x), bounds.max.y.max(point.y)),
            }),
        )
    }

    pub fn center(&self) -> Point {
        Point::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }

    /// These bounds grown by `margin` on every side.
    pub fn expanded(&self, margin: f64) -> Self {
        Self {
            min: Point::new(self.min.x - margin, self.min.y - margin),
            max: Point::new(self.max.x + margin, self.max.y + margin),
        }
    }

    /// Whether `point` lies inside (edges included).
    pub fn contains(&self, point: Point) -> bool {
        (self.min.x..=self.max.x).contains(&point.x) && (self.min.y..=self.max.y).contains(&point.y)
    }

    /// Where `point` moves to when `handle` is dragged by `(dx, dy)` and everything inside
    /// the bounds stretches with it. The edges opposite the grabbed ones stay put, and
    /// dragging an edge past its opposite edge mirrors the points. Along an axis where the
    /// bounds have no size, points move with the grabbed edge instead.
    pub fn stretch(&self, handle: BoundsHandle, dx: f64, dy: f64, point: Point) -> Point {
        let (x_edge, y_edge) = handle.edges();
        Point::new(
            stretch_axis(point.x, self.min.x, self.max.x, x_edge, dx),
            stretch_axis(point.y, self.min.y, self.max.y, y_edge, dy),
        )
    }

    /// Image position of a resize handle.
    pub fn handle_point(&self, handle: BoundsHandle) -> Point {
        let center = self.center();
        let (x_edge, y_edge) = handle.edges();
        let x = match x_edge {
            Some(Edge::Min) => self.min.x,
            Some(Edge::Max) => self.max.x,
            None => center.x,
        };
        let y = match y_edge {
            Some(Edge::Min) => self.min.y,
            Some(Edge::Max) => self.max.y,
            None => center.y,
        };
        Point::new(x, y)
    }
}

/// One axis of [`Bounds::stretch`]: `value` lies between `min` and `max`, and the `moved`
/// edge (if any) moves by `delta`.
fn stretch_axis(value: f64, min: f64, max: f64, moved: Option<Edge>, delta: f64) -> f64 {
    let size = max - min;
    match moved {
        None => value,
        Some(_) if size == 0.0 => value + delta,
        Some(Edge::Min) => max + (value - max) * (size - delta) / size,
        Some(Edge::Max) => min + (value - min) * (size + delta) / size,
    }
}

/// Which edge of the bounds along one axis: the low-coordinate side (left / top) or the
/// high-coordinate side (right / bottom).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Min,
    Max,
}

impl Edge {
    pub fn opposite(self) -> Edge {
        match self {
            Edge::Min => Edge::Max,
            Edge::Max => Edge::Min,
        }
    }
}

/// One of the eight resize handles of a bounding box: four corners and four edge midpoints.
/// Directions are in image space (top = low y).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoundsHandle {
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
}

impl BoundsHandle {
    pub const ALL: [BoundsHandle; 8] = [
        BoundsHandle::TopLeft,
        BoundsHandle::Top,
        BoundsHandle::TopRight,
        BoundsHandle::Right,
        BoundsHandle::BottomRight,
        BoundsHandle::Bottom,
        BoundsHandle::BottomLeft,
        BoundsHandle::Left,
    ];

    /// The x and y edges this handle moves; `None` means it does not move that axis.
    pub fn edges(self) -> (Option<Edge>, Option<Edge>) {
        use BoundsHandle::*;
        let x = match self {
            TopLeft | Left | BottomLeft => Some(Edge::Min),
            TopRight | Right | BottomRight => Some(Edge::Max),
            Top | Bottom => None,
        };
        let y = match self {
            TopLeft | Top | TopRight => Some(Edge::Min),
            BottomLeft | Bottom | BottomRight => Some(Edge::Max),
            Left | Right => None,
        };
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn displays_with_one_decimal_place() {
        assert_eq!(Point::new(12.0, 3.456).to_string(), "(12.0, 3.5)");
    }

    #[test]
    fn bounds_around_points_in_any_order() {
        let bounds = Bounds::around(Point::new(10.0, 2.0), Point::new(4.0, 8.0));

        assert_eq!(bounds.min, Point::new(4.0, 2.0));
        assert_eq!(bounds.max, Point::new(10.0, 8.0));
        assert_eq!(bounds.center(), Point::new(7.0, 5.0));
    }

    #[test]
    fn expanded_bounds_contain_nearby_points() {
        let bounds = Bounds::around(Point::new(0.0, 0.0), Point::new(10.0, 0.0));

        assert!(!bounds.contains(Point::new(5.0, 2.0)));
        assert!(bounds.expanded(3.0).contains(Point::new(5.0, 2.0)));
        assert!(!bounds.expanded(3.0).contains(Point::new(5.0, 4.0)));
    }

    #[test]
    fn handle_points_are_corners_and_edge_midpoints() {
        let bounds = Bounds::around(Point::new(0.0, 0.0), Point::new(10.0, 20.0));

        assert_eq!(
            bounds.handle_point(BoundsHandle::TopLeft),
            Point::new(0.0, 0.0)
        );
        assert_eq!(bounds.handle_point(BoundsHandle::Top), Point::new(5.0, 0.0));
        assert_eq!(
            bounds.handle_point(BoundsHandle::Right),
            Point::new(10.0, 10.0)
        );
        assert_eq!(
            bounds.handle_point(BoundsHandle::BottomLeft),
            Point::new(0.0, 20.0)
        );
    }

    #[test]
    fn enclosing_bounds_cover_every_point() {
        let bounds = Bounds::enclosing([
            Point::new(3.0, 9.0),
            Point::new(-1.0, 4.0),
            Point::new(5.0, 6.0),
        ]);

        assert_eq!(
            bounds,
            Some(Bounds::around(Point::new(-1.0, 4.0), Point::new(5.0, 9.0)))
        );
        assert_eq!(Bounds::enclosing([]), None);
    }

    #[test]
    fn stretching_scales_points_away_from_the_fixed_edges() {
        let bounds = Bounds::around(Point::new(10.0, 10.0), Point::new(20.0, 30.0));
        let middle = Point::new(15.0, 20.0);

        // Right edge 20 -> 30 doubles the width from the left edge.
        assert_eq!(
            bounds.stretch(BoundsHandle::Right, 10.0, 99.0, middle),
            Point::new(20.0, 20.0)
        );
        // Top-left corner by (5, 10) halves both sizes towards the bottom-right.
        assert_eq!(
            bounds.stretch(BoundsHandle::TopLeft, 5.0, 10.0, middle),
            Point::new(17.5, 25.0)
        );
        assert_eq!(
            bounds.stretch(BoundsHandle::TopLeft, 5.0, 10.0, bounds.max),
            bounds.max
        );
        // Past the opposite edge mirrors.
        assert_eq!(
            bounds.stretch(BoundsHandle::Left, 20.0, 0.0, bounds.min),
            Point::new(30.0, 10.0)
        );
    }

    #[test]
    fn stretching_a_flat_axis_moves_points_with_the_edge() {
        let flat = Bounds::around(Point::new(0.0, 5.0), Point::new(10.0, 5.0));

        assert_eq!(
            flat.stretch(BoundsHandle::Bottom, 0.0, 3.0, Point::new(4.0, 5.0)),
            Point::new(4.0, 8.0)
        );
    }

    #[test]
    fn every_handle_moves_at_least_one_edge() {
        for handle in BoundsHandle::ALL {
            assert_ne!(handle.edges(), (None, None), "{handle:?}");
        }
    }
}
