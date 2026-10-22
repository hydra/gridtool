//! Polygon maths for area measurements: area, centroid and triangulation of a closed
//! outline given by its corner points in order.

use crate::geometry::Point;

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

    #[test]
    fn collinear_points_are_handled() {
        // A square with an extra point halfway along its bottom edge.
        let outline = points(&[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)]);

        assert_eq!(triangulated_area(&outline), 4.0);
        assert!(triangulate(&points(&[(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)])).is_empty());
    }
}
