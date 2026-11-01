//! Polygon maths for closed outlines given by their corner points in order: area,
//! centroid, triangulation and hatching.

use crate::geometry::{Bounds, Point};

/// The direction of a set of parallel fill lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HatchDirection {
    cos: f64,
    sin: f64,
}

impl HatchDirection {
    /// Lines `degrees` clockwise from the x axis (y increases downwards).
    pub fn at_angle(degrees: f64) -> Self {
        let angle = degrees.rem_euclid(180.0);
        // Exact, so that lines along an axis have exactly equal coordinates.
        let (sin, cos) = if angle == 0.0 {
            (0.0, 1.0)
        } else if angle == 90.0 {
            (1.0, 0.0)
        } else {
            angle.to_radians().sin_cos()
        };
        Self { cos, sin }
    }

    /// The unit vector the lines are offset along, across them.
    pub fn normal(self) -> (f64, f64) {
        (-self.sin, self.cos)
    }

    /// `point` as (position along the lines, position across them).
    fn along_across(self, point: Point) -> (f64, f64) {
        (
            point.x * self.cos + point.y * self.sin,
            point.y * self.cos - point.x * self.sin,
        )
    }

    /// The inverse of [`Self::along_across`].
    fn point_at(self, along: f64, across: f64) -> Point {
        Point::new(
            along * self.cos - across * self.sin,
            along * self.sin + across * self.cos,
        )
    }

    /// The smallest and largest position across the lines of any of `points`.
    fn across_range(self, points: impl IntoIterator<Item = Point>) -> (f64, f64) {
        points
            .into_iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), point| {
                let across = self.along_across(point).1;
                (low.min(across), high.max(across))
            })
    }
}

/// The parts inside the outline of parallel lines at every whole multiple of `spacing`
/// from the coordinate origin, for the lines that pass through `region`. A line crossing the
/// outline more than once gives several segments (inside is decided by the even-odd rule).
/// Because the lines sit at multiples of `spacing`, they stay put when the outline changes.
pub fn hatch_lines(
    points: &[Point],
    direction: HatchDirection,
    spacing: f64,
    region: Bounds,
) -> Vec<(Point, Point)> {
    if points.is_empty() {
        return Vec::new();
    }
    let (outline_low, outline_high) = direction.across_range(points.iter().copied());
    let (region_low, region_high) = direction.across_range([
        region.min,
        Point::new(region.max.x, region.min.y),
        region.max,
        Point::new(region.min.x, region.max.y),
    ]);
    let low = outline_low.max(region_low);
    let high = outline_high.min(region_high);
    let (first, last) = (
        (low / spacing).ceil() as i64,
        (high / spacing).floor() as i64,
    );

    let mut segments = Vec::new();
    for line in first..=last {
        let position = line as f64 * spacing;
        let mut crossings: Vec<f64> = edges(points)
            .filter_map(|(a, b)| {
                let (a_along, a_across) = direction.along_across(a);
                let (b_along, b_across) = direction.along_across(b);
                // Half-open, so that a line through a corner counts it once.
                ((a_across > position) != (b_across > position)).then(|| {
                    a_along + (position - a_across) / (b_across - a_across) * (b_along - a_along)
                })
            })
            .collect();
        crossings.sort_by(f64::total_cmp);
        // A line that only touches a corner has nothing to draw.
        segments.extend(
            crossings
                .chunks_exact(2)
                .filter(|pair| pair[0] < pair[1])
                .map(|pair| {
                    (
                        direction.point_at(pair[0], position),
                        direction.point_at(pair[1], position),
                    )
                }),
        );
    }
    segments
}

/// The shoelace sum: twice the polygon's area, positive for one winding direction and
/// negative for the other. For a self-intersecting outline, oppositely wound parts
/// cancel out.
fn twice_signed_area(points: &[Point]) -> f64 {
    edges(points).map(|(a, b)| a.x * b.y - b.x * a.y).sum()
}

/// Each edge of the closed outline, including the one from the last point to the first.
fn edges(points: &[Point]) -> impl Iterator<Item = (Point, Point)> + '_ {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(&a, &b)| (a, b))
}

/// The area enclosed by the outline, in square image pixels (the shoelace formula).
pub fn area(points: &[Point]) -> f64 {
    (twice_signed_area(points) / 2.0).abs()
}

/// The centre of mass of the enclosed area, or `None` if it encloses no area.
pub fn centroid(points: &[Point]) -> Option<Point> {
    let twice_area = twice_signed_area(points);
    if twice_area.abs() < f64::EPSILON {
        return None;
    }
    let (x, y) = edges(points).fold((0.0, 0.0), |(x, y), (a, b)| {
        let cross = a.x * b.y - b.x * a.y;
        (x + (a.x + b.x) * cross, y + (a.y + b.y) * cross)
    });
    let scale = 1.0 / (3.0 * twice_area);
    Some(Point::new(x * scale, y * scale))
}

/// Splits the outline into triangles (as indices into `points`) by ear clipping, so that
/// convex and concave outlines can both be filled. An outline that crosses itself may
/// be only partly covered.
pub fn triangulate(points: &[Point]) -> Vec<[usize; 3]> {
    let mut remaining: Vec<usize> = (0..points.len()).collect();
    if twice_signed_area(points) < 0.0 {
        remaining.reverse();
    }
    let mut triangles = Vec::with_capacity(points.len().saturating_sub(2));
    while remaining.len() > 3 {
        let count = remaining.len();
        let ear = (0..count).find(|&i| {
            let corner = [
                remaining[(i + count - 1) % count],
                remaining[i],
                remaining[(i + 1) % count],
            ];
            is_ear(points, &remaining, corner)
        });
        let Some(i) = ear else {
            // Degenerate or self-intersecting: no ear left to clip.
            return triangles;
        };
        triangles.push([
            remaining[(i + count - 1) % count],
            remaining[i],
            remaining[(i + 1) % count],
        ]);
        remaining.remove(i);
    }
    if let [a, b, c] = remaining[..]
        && cross(points[a], points[b], points[c]) != 0.0
    {
        triangles.push([a, b, c]);
    }
    triangles
}

/// Whether the corner `[previous, current, next]` of a positively wound outline can be
/// cut off: it turns the right way and no other remaining point lies inside it.
fn is_ear(points: &[Point], remaining: &[usize], corner: [usize; 3]) -> bool {
    let [a, b, c] = corner.map(|index| points[index]);
    if cross(a, b, c) <= 0.0 {
        return false;
    }
    remaining
        .iter()
        .filter(|index| !corner.contains(index))
        .map(|&index| points[index])
        .filter(|&point| point != a && point != b && point != c)
        .all(|point| !in_triangle(point, a, b, c))
}

/// The z component of `(b - a) × (c - b)`: positive when `a → b → c` turns the same way
/// as a positively wound outline.
fn cross(a: Point, b: Point, c: Point) -> f64 {
    (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x)
}

/// Whether `point` lies inside or on the positively wound triangle `a, b, c`.
fn in_triangle(point: Point, a: Point, b: Point, c: Point) -> bool {
    cross(a, b, point) >= 0.0 && cross(b, c, point) >= 0.0 && cross(c, a, point) >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points(coordinates: &[(f64, f64)]) -> Vec<Point> {
        coordinates.iter().map(|&(x, y)| Point::new(x, y)).collect()
    }

    /// An L shape: a 20 × 20 square with its top-right 10 × 10 quarter removed.
    fn l_shape() -> Vec<Point> {
        points(&[
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (20.0, 10.0),
            (20.0, 20.0),
            (0.0, 20.0),
        ])
    }

    fn triangulated_area(outline: &[Point]) -> f64 {
        triangulate(outline)
            .iter()
            .map(|triangle| area(&triangle.map(|index| outline[index])))
            .sum()
    }

    #[test]
    fn area_is_the_same_for_either_winding() {
        let square = points(&[(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)]);
        let mut reversed = square.clone();
        reversed.reverse();

        assert_eq!(area(&square), 12.0);
        assert_eq!(area(&reversed), 12.0);
        assert_eq!(area(&l_shape()), 300.0);
    }

    #[test]
    fn the_centroid_is_the_centre_of_mass() {
        let square = points(&[(0.0, 0.0), (4.0, 0.0), (4.0, 2.0), (0.0, 2.0)]);
        let triangle = points(&[(0.0, 0.0), (6.0, 0.0), (0.0, 3.0)]);

        assert_eq!(centroid(&square), Some(Point::new(2.0, 1.0)));
        assert_eq!(centroid(&triangle), Some(Point::new(2.0, 1.0)));
        assert_eq!(
            centroid(&points(&[(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)])),
            None
        );
    }

    #[test]
    fn convex_and_concave_outlines_are_fully_triangulated() {
        let square = points(&[(0.0, 0.0), (4.0, 0.0), (4.0, 3.0), (0.0, 3.0)]);
        assert_eq!(triangulate(&square).len(), 2);
        assert_eq!(triangulated_area(&square), 12.0);

        let mut l_shape = l_shape();
        assert_eq!(triangulate(&l_shape).len(), 4);
        assert_eq!(triangulated_area(&l_shape), 300.0);
        l_shape.reverse();
        assert_eq!(triangulated_area(&l_shape), 300.0);
    }

    const EVERYWHERE: Bounds = Bounds {
        min: Point::new(-1000.0, -1000.0),
        max: Point::new(1000.0, 1000.0),
    };

    fn horizontal() -> HatchDirection {
        HatchDirection::at_angle(0.0)
    }

    fn vertical() -> HatchDirection {
        HatchDirection::at_angle(90.0)
    }

    #[test]
    fn hatch_lines_sit_at_multiples_of_the_spacing_inside_the_outline() {
        let square = points(&[(5.0, 5.0), (25.0, 5.0), (25.0, 25.0), (5.0, 25.0)]);

        assert_eq!(
            hatch_lines(&square, vertical(), 10.0, EVERYWHERE),
            [
                (Point::new(20.0, 5.0), Point::new(20.0, 25.0)),
                (Point::new(10.0, 5.0), Point::new(10.0, 25.0)),
            ]
        );
        assert_eq!(
            hatch_lines(&square, horizontal(), 20.0, EVERYWHERE),
            [(Point::new(5.0, 20.0), Point::new(25.0, 20.0))]
        );
    }

    #[test]
    fn a_hatch_direction_is_the_same_every_half_turn() {
        assert_eq!(HatchDirection::at_angle(180.0), horizontal());
        assert_eq!(HatchDirection::at_angle(270.0), vertical());
        assert_eq!(HatchDirection::at_angle(-90.0), vertical());
    }

    #[test]
    fn diagonal_hatch_lines_are_the_spacing_apart_measured_across_them() {
        let square = points(&[(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)]);

        let lines = hatch_lines(&square, HatchDirection::at_angle(45.0), 10.0, EVERYWHERE);

        assert!(!lines.is_empty());
        for (start, end) in lines {
            // Clockwise from the x axis: x and y grow together along the line.
            let (across_start, across_end) = (
                (start.y - start.x) / 2f64.sqrt(),
                (end.y - end.x) / 2f64.sqrt(),
            );
            assert!((across_start - across_end).abs() < 1e-9);
            let remainder = across_start.rem_euclid(10.0);
            assert!(!(1e-9..=10.0 - 1e-9).contains(&remainder), "{remainder}");
        }
    }

    #[test]
    fn hatch_lines_split_where_a_concave_outline_is_crossed_more_than_once() {
        // A U shape: 30 wide and 20 tall, with a notch from x = 10 to 20 down to y = 10.
        let u_shape = points(&[
            (0.0, 0.0),
            (10.0, 0.0),
            (10.0, 10.0),
            (20.0, 10.0),
            (20.0, 0.0),
            (30.0, 0.0),
            (30.0, 20.0),
            (0.0, 20.0),
        ]);

        let lines = hatch_lines(&u_shape, horizontal(), 5.0, EVERYWHERE);

        let at_5: Vec<_> = lines.iter().filter(|(a, _)| a.y == 5.0).collect();
        assert_eq!(
            at_5,
            [
                &(Point::new(0.0, 5.0), Point::new(10.0, 5.0)),
                &(Point::new(20.0, 5.0), Point::new(30.0, 5.0)),
            ]
        );
        let at_15: Vec<_> = lines.iter().filter(|(a, _)| a.y == 15.0).collect();
        assert_eq!(at_15, [&(Point::new(0.0, 15.0), Point::new(30.0, 15.0))]);
    }

    #[test]
    fn hatch_lines_are_only_generated_for_the_region() {
        let square = points(&[(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)]);
        let region = Bounds::around(Point::new(25.0, -500.0), Point::new(45.0, 500.0));

        let lines = hatch_lines(&square, vertical(), 10.0, region);

        let positions: Vec<f64> = lines.iter().map(|(a, _)| a.x).collect();
        assert_eq!(positions, [40.0, 30.0]);
    }

    #[test]
    fn a_hatch_line_through_a_corner_stays_paired() {
        let diamond = points(&[(10.0, 0.0), (20.0, 10.0), (10.0, 20.0), (0.0, 10.0)]);

        let lines = hatch_lines(&diamond, horizontal(), 10.0, EVERYWHERE);

        // y = 0 touches only the top corner and y = 20 only the bottom one.
        assert_eq!(lines, [(Point::new(0.0, 10.0), Point::new(20.0, 10.0))]);
    }

    #[test]
    fn collinear_points_are_handled() {
        // A square with an extra point halfway along its bottom edge.
        let outline = points(&[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)]);

        assert_eq!(triangulated_area(&outline), 4.0);
        assert!(triangulate(&points(&[(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)])).is_empty());
    }
}
