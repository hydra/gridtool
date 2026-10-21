//! Real-world scale calibration from two measured reference lines.

use std::fmt;

use crate::geometry::Point;
use crate::units::LengthUnit;

/// How the two reference lines are defined by the calibration points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CalibrationMethod {
    /// Three points: line A is 1→2, line B is 2→3 (sharing point 2).
    #[default]
    ThreePoint,
    /// Four points: line A is 1→2, line B is 3→4 (independent lines).
    FourPoint,
}

impl CalibrationMethod {
    pub const ALL: [CalibrationMethod; 2] =
        [CalibrationMethod::ThreePoint, CalibrationMethod::FourPoint];

    pub fn point_count(self) -> usize {
        match self {
            CalibrationMethod::ThreePoint => 3,
            CalibrationMethod::FourPoint => 4,
        }
    }

    /// Indices of the start and end points of `line`.
    pub fn line_endpoints(self, line: Line) -> (usize, usize) {
        match (self, line) {
            (_, Line::A) => (0, 1),
            (CalibrationMethod::ThreePoint, Line::B) => (1, 2),
            (CalibrationMethod::FourPoint, Line::B) => (2, 3),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CalibrationMethod::ThreePoint => "3-point",
            CalibrationMethod::FourPoint => "4-point",
        }
    }
}

impl fmt::Display for CalibrationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One of the two reference lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Line {
    A,
    B,
}

impl Line {
    pub const ALL: [Line; 2] = [Line::A, Line::B];

    fn index(self) -> usize {
        match self {
            Line::A => 0,
            Line::B => 1,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Line::A => "A",
            Line::B => "B",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalibrationError {
    WrongPointCount {
        method: CalibrationMethod,
        actual: usize,
    },
    PointIndexOutOfRange {
        index: usize,
        count: usize,
    },
}

impl fmt::Display for CalibrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongPointCount { method, actual } => write!(
                f,
                "{method} calibration needs {} points, got {actual}",
                method.point_count()
            ),
            Self::PointIndexOutOfRange { index, count } => {
                write!(f, "point {} does not exist (there are {count})", index + 1)
            }
        }
    }
}

impl std::error::Error for CalibrationError {}

/// A complete set of calibration points for a method.
#[derive(Debug, Clone, PartialEq)]
pub struct ScalePoints {
    method: CalibrationMethod,
    points: Vec<Point>,
}

impl ScalePoints {
    pub fn new(method: CalibrationMethod, points: Vec<Point>) -> Result<Self, CalibrationError> {
        if points.len() != method.point_count() {
            return Err(CalibrationError::WrongPointCount {
                method,
                actual: points.len(),
            });
        }
        Ok(Self { method, points })
    }

    pub fn method(&self) -> CalibrationMethod {
        self.method
    }

    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Start and end of `line`.
    pub fn line(&self, line: Line) -> (Point, Point) {
        let (start, end) = self.method.line_endpoints(line);
        (self.points[start], self.points[end])
    }

    pub fn set_point(&mut self, index: usize, point: Point) -> Result<(), CalibrationError> {
        let count = self.points.len();
        let slot = self
            .points
            .get_mut(index)
            .ok_or(CalibrationError::PointIndexOutOfRange { index, count })?;
        *slot = point;
        Ok(())
    }
}

/// The calculated real-world size of one image pixel along each image axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scale {
    pub x_per_pixel: f64,
    pub y_per_pixel: f64,
    pub unit: LengthUnit,
}

impl Scale {
    /// Real-world length, in [`Self::unit`], of the line between two image points.
    pub fn length(&self, from: Point, to: Point) -> f64 {
        let dx = (to.x - from.x) * self.x_per_pixel;
        let dy = (to.y - from.y) * self.y_per_pixel;
        dx.hypot(dy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleError {
    MissingPoints,
    MissingDistance(Line),
    ZeroLengthLine(Line),
    /// The lines' orientations do not allow X and Y to be separated (e.g. parallel lines).
    Indeterminate,
    /// The distances cannot be satisfied by any positive X and Y scale.
    Inconsistent,
}

impl fmt::Display for ScaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPoints => f.write_str("place the calibration points"),
            Self::MissingDistance(line) => write!(f, "enter the distance of line {line}"),
            Self::ZeroLengthLine(line) => write!(f, "line {line} has zero length"),
            Self::Indeterminate => {
                f.write_str("lines A and B need different orientations to separate X and Y")
            }
            Self::Inconsistent => f.write_str("the distances do not fit the lines"),
        }
    }
}

impl std::error::Error for ScaleError {}

/// Everything the user has entered to calibrate the document's scale.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Calibration {
    points: Option<ScalePoints>,
    distances: [Option<f64>; 2],
    unit: LengthUnit,
}

impl Calibration {
    pub fn points(&self) -> Option<&ScalePoints> {
        self.points.as_ref()
    }

    /// Real-world length of `line`, in [`Self::unit`].
    pub fn distance(&self, line: Line) -> Option<f64> {
        self.distances[line.index()]
    }

    pub fn unit(&self) -> LengthUnit {
        self.unit
    }

    pub(crate) fn points_mut(&mut self) -> Option<&mut ScalePoints> {
        self.points.as_mut()
    }

    pub(crate) fn set_points(&mut self, points: ScalePoints) {
        self.points = Some(points);
    }

    pub(crate) fn set_distance(&mut self, line: Line, distance: Option<f64>) {
        self.distances[line.index()] = distance;
    }

    pub(crate) fn set_unit(&mut self, unit: LengthUnit) {
        self.unit = unit;
    }

    /// Calculates the X and Y scale.
    ///
    /// Each line, with pixel extent (dx, dy) and real length L, gives
    /// `(dx·sx)² + (dy·sy)² = L²`. The two lines form a linear system in `sx²` and `sy²`,
    /// so the axes can have different scales and the lines need not be axis-aligned.
    pub fn scale(&self) -> Result<Scale, ScaleError> {
        let points = self.points.as_ref().ok_or(ScaleError::MissingPoints)?;
        let [a, b] = Line::ALL.map(|line| self.line_equation(points, line));
        let ([ax, ay, al], [bx, by, bl]) = (a?, b?);

        let determinant = ax * by - bx * ay;
        let magnitude = (ax * by).abs().max((bx * ay).abs());
        if determinant.abs() <= magnitude * 1e-9 {
            return Err(ScaleError::Indeterminate);
        }
        let x_squared = (al * by - bl * ay) / determinant;
        let y_squared = (ax * bl - bx * al) / determinant;
        if x_squared <= 0.0 || y_squared <= 0.0 {
            return Err(ScaleError::Inconsistent);
        }

        Ok(Scale {
            x_per_pixel: x_squared.sqrt(),
            y_per_pixel: y_squared.sqrt(),
            unit: self.unit,
        })
    }

    /// Coefficients `[dx², dy², L²]` of the scale equation for `line`.
    fn line_equation(&self, points: &ScalePoints, line: Line) -> Result<[f64; 3], ScaleError> {
        let length = self
            .distance(line)
            .ok_or(ScaleError::MissingDistance(line))?;
        let (start, end) = points.line(line);
        let (dx, dy) = (end.x - start.x, end.y - start.y);
        if dx == 0.0 && dy == 0.0 {
            return Err(ScaleError::ZeroLengthLine(line));
        }
        Ok([dx * dx, dy * dy, length * length])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calibration(
        method: CalibrationMethod,
        points: &[(f64, f64)],
        a: f64,
        b: f64,
    ) -> Calibration {
        let points = points.iter().map(|&(x, y)| Point::new(x, y)).collect();
        let mut calibration = Calibration::default();
        calibration.set_points(ScalePoints::new(method, points).unwrap());
        calibration.set_distance(Line::A, Some(a));
        calibration.set_distance(Line::B, Some(b));
        calibration
    }

    fn assert_near(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn three_point_lines_share_the_middle_point() {
        let method = CalibrationMethod::ThreePoint;
        assert_eq!(method.line_endpoints(Line::A), (0, 1));
        assert_eq!(method.line_endpoints(Line::B), (1, 2));
    }

    #[test]
    fn four_point_lines_are_independent() {
        let method = CalibrationMethod::FourPoint;
        assert_eq!(method.line_endpoints(Line::A), (0, 1));
        assert_eq!(method.line_endpoints(Line::B), (2, 3));
    }

    #[test]
    fn scale_points_require_the_method_point_count() {
        let result = ScalePoints::new(CalibrationMethod::FourPoint, vec![Point::default(); 3]);

        assert_eq!(
            result,
            Err(CalibrationError::WrongPointCount {
                method: CalibrationMethod::FourPoint,
                actual: 3
            })
        );
    }

    #[test]
    fn setting_a_missing_point_fails() {
        let mut points =
            ScalePoints::new(CalibrationMethod::ThreePoint, vec![Point::default(); 3]).unwrap();

        assert!(points.set_point(3, Point::default()).is_err());
    }

    #[test]
    fn three_point_axis_aligned_scale() {
        // A: 100 px horizontal = 50 cm, B: 200 px vertical = 50 cm.
        let calibration = calibration(
            CalibrationMethod::ThreePoint,
            &[(10.0, 10.0), (110.0, 10.0), (110.0, 210.0)],
            50.0,
            50.0,
        );

        let scale = calibration.scale().unwrap();

        assert_near(scale.x_per_pixel, 0.5);
        assert_near(scale.y_per_pixel, 0.25);
        assert_eq!(scale.unit, LengthUnit::Centimetre);
    }

    #[test]
    fn four_point_diagonal_lines_recover_known_scale() {
        let (sx, sy) = (0.2_f64, 0.5_f64);
        let length = |dx: f64, dy: f64| ((dx * sx).powi(2) + (dy * sy).powi(2)).sqrt();
        let calibration = calibration(
            CalibrationMethod::FourPoint,
            &[(0.0, 0.0), (30.0, 40.0), (100.0, 100.0), (180.0, 90.0)],
            length(30.0, 40.0),
            length(80.0, -10.0),
        );

        let scale = calibration.scale().unwrap();

        assert_near(scale.x_per_pixel, sx);
        assert_near(scale.y_per_pixel, sy);
    }

    #[test]
    fn scale_needs_points_and_distances() {
        let mut calibration = Calibration::default();
        assert_eq!(calibration.scale(), Err(ScaleError::MissingPoints));

        calibration.set_points(
            ScalePoints::new(
                CalibrationMethod::ThreePoint,
                vec![
                    Point::new(0.0, 0.0),
                    Point::new(1.0, 0.0),
                    Point::new(1.0, 1.0),
                ],
            )
            .unwrap(),
        );
        assert_eq!(
            calibration.scale(),
            Err(ScaleError::MissingDistance(Line::A))
        );

        calibration.set_distance(Line::A, Some(1.0));
        assert_eq!(
            calibration.scale(),
            Err(ScaleError::MissingDistance(Line::B))
        );
    }

    #[test]
    fn zero_length_line_is_reported() {
        let calibration = calibration(
            CalibrationMethod::ThreePoint,
            &[(5.0, 5.0), (5.0, 5.0), (5.0, 50.0)],
            1.0,
            1.0,
        );

        assert_eq!(
            calibration.scale(),
            Err(ScaleError::ZeroLengthLine(Line::A))
        );
    }

    #[test]
    fn parallel_lines_are_indeterminate() {
        let calibration = calibration(
            CalibrationMethod::FourPoint,
            &[(0.0, 0.0), (100.0, 0.0), (0.0, 50.0), (60.0, 50.0)],
            10.0,
            6.0,
        );

        assert_eq!(calibration.scale(), Err(ScaleError::Indeterminate));
    }

    #[test]
    fn impossible_distances_are_inconsistent() {
        // Mostly-horizontal A is long but mostly-vertical B is tiny, forcing a negative sx².
        let calibration = calibration(
            CalibrationMethod::FourPoint,
            &[(0.0, 0.0), (100.0, 10.0), (0.0, 0.0), (10.0, 100.0)],
            1.0,
            100.0,
        );

        assert_eq!(calibration.scale(), Err(ScaleError::Inconsistent));
    }
}
