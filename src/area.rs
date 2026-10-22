//! Area measurements: closed outlines on the document whose real-world area is derived
//! from the document's scale calibration.

use std::fmt;

use crate::calibration::{Calibration, ScaleError};
use crate::color::Rgb;
use crate::geometry::{Bounds, BoundsHandle, Point};
use crate::polygon;
use crate::units::LengthUnit;

/// The fewest points that enclose an area.
pub const MIN_POINTS: usize = 3;
/// The colour of a new area.
pub const DEFAULT_COLOR: Rgb = Rgb::CYAN;

/// A closed outline through three or more points (convex or concave), with its own
/// display unit and colour.
#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    points: Vec<Point>,
    unit: LengthUnit,
    color: Rgb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooFewPoints(pub usize);

impl fmt::Display for TooFewPoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "an area needs at least {MIN_POINTS} points, got {}",
            self.0
        )
    }
}

impl std::error::Error for TooFewPoints {}

impl Area {
    /// An area through `points`, in order, reported in `unit` and drawn in the default
    /// colour.
    pub fn new(points: Vec<Point>, unit: LengthUnit) -> Result<Self, TooFewPoints> {
        if points.len() < MIN_POINTS {
            return Err(TooFewPoints(points.len()));
        }
        Ok(Self {
            points,
            unit,
            color: DEFAULT_COLOR,
        })
    }

    /// The outline's corners, in order; the last connects back to the first.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// The unit the area is reported in (squared).
    pub fn unit(&self) -> LengthUnit {
        self.unit
    }

    pub fn color(&self) -> Rgb {
        self.color
    }

    /// Real-world area enclosed by the outline, in [`Self::unit`] squared, using the
    /// calibrated X and Y scales.
    pub fn area(&self, calibration: &Calibration) -> Result<f64, ScaleError> {
        let scale = calibration.scale()?;
        let area = polygon::area(&self.points) * scale.x_per_pixel * scale.y_per_pixel;
        Ok(scale.unit.convert_area(area, self.unit))
    }

    /// The smallest axis-aligned box containing every point.
    pub fn bounds(&self) -> Bounds {
        Bounds::enclosing(self.points.iter().copied()).expect("an area has points")
    }

    /// The centre of the enclosed area, or the centre of the bounds if it encloses none.
    pub fn center(&self) -> Point {
        polygon::centroid(&self.points).unwrap_or_else(|| self.bounds().center())
    }

    /// The area after dragging bounding-box `handle` by `(dx, dy)` image pixels: every
    /// point stretches with the box ([`Bounds::stretch`]).
    pub fn resized(&self, handle: BoundsHandle, dx: f64, dy: f64) -> Area {
        let bounds = self.bounds();
        self.with_points(
            self.points
                .iter()
                .map(|&point| bounds.stretch(handle, dx, dy, point))
                .collect(),
        )
    }

    /// The area with point `index` moved to `to`; unchanged if there is no such point.
    pub fn with_point(&self, index: usize, to: Point) -> Area {
        let mut points = self.points.clone();
        if let Some(point) = points.get_mut(index) {
            *point = to;
        }
        self.with_points(points)
    }

    /// The area with every point passed through `map`.
    pub fn map_points(&self, map: impl Fn(Point) -> Point) -> Area {
        self.with_points(self.points.iter().map(|&point| map(point)).collect())
    }

    fn with_points(&self, points: Vec<Point>) -> Area {
        Area {
            points,
            ..self.clone()
        }
    }

    /// Replaces the points; the caller keeps the count at or above [`MIN_POINTS`].
    pub(crate) fn set_points(&mut self, points: Vec<Point>) {
        self.points = points;
    }

    pub(crate) fn set_unit(&mut self, unit: LengthUnit) {
        self.unit = unit;
    }

    pub(crate) fn set_color(&mut self, color: Rgb) {
        self.color = color;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::calibration::{CalibrationMethod, Line, ScalePoints};
    use crate::document::Document;

    /// A calibration of 0.5 cm/px horizontally and 0.25 cm/px vertically.
    fn calibrated() -> Calibration {
        let mut document = Document::new(500, 500);
        let points = ScalePoints::new(
            CalibrationMethod::ThreePoint,
            vec![
                Point::new(10.0, 10.0),
                Point::new(110.0, 10.0),
                Point::new(110.0, 210.0),
            ],
        )
        .unwrap();
        for action in [
            Action::SetScalePoints(points),
            Action::SetDistance {
                line: Line::A,
                distance: Some(50.0),
            },
            Action::SetDistance {
                line: Line::B,
                distance: Some(50.0),
            },
        ] {
            action.apply(&mut document).unwrap();
        }
        document.calibration().clone()
    }

    /// A 40 × 80 px rectangle from (10, 20).
    fn rectangle(unit: LengthUnit) -> Area {
        Area::new(
            vec![
                Point::new(10.0, 20.0),
                Point::new(50.0, 20.0),
                Point::new(50.0, 100.0),
                Point::new(10.0, 100.0),
            ],
            unit,
        )
        .unwrap()
    }

    #[test]
    fn needs_at_least_three_points() {
        let two = vec![Point::new(0.0, 0.0), Point::new(1.0, 1.0)];

        assert_eq!(Area::new(two, LengthUnit::Metre), Err(TooFewPoints(2)));
        assert_eq!(rectangle(LengthUnit::Metre).color(), Rgb::CYAN);
    }

    #[test]
    fn area_applies_both_axis_scales_and_the_unit() {
        // 40 px = 20 cm across, 80 px = 20 cm down: 400 cm².
        let centimetres = rectangle(LengthUnit::Centimetre)
            .area(&calibrated())
            .unwrap();
        assert!((centimetres - 400.0).abs() < 1e-9, "{centimetres}");
        let metres = rectangle(LengthUnit::Metre).area(&calibrated()).unwrap();
        assert!((metres - 0.04).abs() < 1e-12, "{metres}");
    }

    #[test]
    fn area_is_unavailable_without_a_scale() {
        assert_eq!(
            rectangle(LengthUnit::Metre).area(&Calibration::default()),
            Err(ScaleError::MissingPoints)
        );
    }

    #[test]
    fn bounds_and_centre() {
        let area = rectangle(LengthUnit::Metre);

        assert_eq!(
            area.bounds(),
            Bounds::around(Point::new(10.0, 20.0), Point::new(50.0, 100.0))
        );
        let center = area.center();
        assert!((center.x - 30.0).abs() < 1e-9 && (center.y - 60.0).abs() < 1e-9);
    }

    #[test]
    fn resizing_stretches_every_point() {
        let resized = rectangle(LengthUnit::Metre).resized(BoundsHandle::Right, 40.0, 0.0);

        assert_eq!(
            resized.points(),
            [
                Point::new(10.0, 20.0),
                Point::new(90.0, 20.0),
                Point::new(90.0, 100.0),
                Point::new(10.0, 100.0),
            ]
        );
        assert_eq!(resized.unit(), LengthUnit::Metre);
    }

    #[test]
    fn moving_one_point_keeps_the_others() {
        let moved = rectangle(LengthUnit::Metre).with_point(2, Point::new(70.0, 90.0));

        assert_eq!(moved.points()[2], Point::new(70.0, 90.0));
        assert_eq!(moved.points()[1], Point::new(50.0, 20.0));
        assert_eq!(moved.points().len(), 4);
    }
}
