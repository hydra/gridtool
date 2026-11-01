//! Drawn shapes: arbitrary lines through two or more points, optionally joined up to
//! enclose an area. Unlike measurements nothing is measured from them, but the distance
//! between their fill lines is a real-world length, as a grid's cell size is.

use std::fmt;

use crate::calibration::Scale;
use crate::color::Rgb;
use crate::geometry::{Bounds, BoundsHandle, Point};
use crate::polygon::{self, HatchDirection};
use crate::units::LengthUnit;

/// The fewest points that make a line.
pub const MIN_OPEN_POINTS: usize = 2;
/// The fewest points that enclose an area.
pub const MIN_CLOSED_POINTS: usize = 3;
pub const DEFAULT_LINE_COLOR: Rgb = Rgb::new(255, 128, 0);
pub const DEFAULT_LINE_OPACITY: f32 = 1.0;
pub const DEFAULT_FILL_COLOR: Rgb = DEFAULT_LINE_COLOR;
pub const DEFAULT_FILL_OPACITY: f32 = 0.5;
/// Distance between fill lines of a new shape, in the shape's fill unit.
pub const DEFAULT_FILL_SPACING: f64 = 10.0;
/// New shapes have horizontal fill lines.
pub const DEFAULT_FILL_ANGLE: f64 = 0.0;
/// Fill lines repeat every half turn, so angles are kept below this.
const HALF_TURN: f64 = 180.0;

/// `degrees` as the equivalent angle of parallel lines, in [0, 180).
pub fn normalized_fill_angle(degrees: f64) -> f64 {
    let angle = degrees.rem_euclid(HALF_TURN);
    // The remainder of a tiny negative angle rounds up to the half turn itself.
    if angle < HALF_TURN { angle } else { 0.0 }
}

/// Whether `degrees` is a fill angle as stored: in [0, 180).
pub fn is_valid_fill_angle(degrees: f64) -> bool {
    (0.0..HALF_TURN).contains(&degrees)
}

/// How a shape's line is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineStyle {
    #[default]
    Thin,
    Thick,
    Dashed,
}

impl LineStyle {
    pub const ALL: [LineStyle; 3] = [LineStyle::Thin, LineStyle::Thick, LineStyle::Dashed];

    pub fn name(self) -> &'static str {
        match self {
            LineStyle::Thin => "Thin",
            LineStyle::Thick => "Thick",
            LineStyle::Dashed => "Dashed",
        }
    }
}

/// How the area enclosed by a closed shape is filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillStyle {
    #[default]
    None,
    Solid,
    Lines,
    Crosshatch,
}

impl FillStyle {
    pub const ALL: [FillStyle; 4] = [
        FillStyle::None,
        FillStyle::Solid,
        FillStyle::Lines,
        FillStyle::Crosshatch,
    ];

    pub fn name(self) -> &'static str {
        match self {
            FillStyle::None => "None",
            FillStyle::Solid => "Solid",
            FillStyle::Lines => "Lines",
            FillStyle::Crosshatch => "Crosshatch",
        }
    }

    /// The style's sets of fill lines, as angles from the shape's fill angle; empty if it
    /// does not use lines.
    fn line_offsets(self) -> &'static [f64] {
        match self {
            FillStyle::None | FillStyle::Solid => &[],
            FillStyle::Lines => &[0.0],
            FillStyle::Crosshatch => &[0.0, 90.0],
        }
    }

    /// Whether the style draws fill lines.
    pub fn has_lines(self) -> bool {
        !self.line_offsets().is_empty()
    }
}

/// A line through two or more points, or a closed outline through three or more.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawnShape {
    points: Vec<Point>,
    closed: bool,
    line_style: LineStyle,
    line_color: Rgb,
    line_opacity: f32,
    fill_style: FillStyle,
    fill_color: Rgb,
    /// Real-world distance between fill lines, in `fill_unit`.
    fill_spacing: f64,
    fill_unit: LengthUnit,
    /// Real-world direction of the fill lines, in degrees clockwise from the image's x axis
    /// (see [`normalized_fill_angle`]).
    fill_angle: f64,
    fill_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooFewPoints(pub usize);

impl fmt::Display for TooFewPoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a shape cannot have only {} points", self.0)
    }
}

impl std::error::Error for TooFewPoints {}

impl DrawnShape {
    /// A line through `points`, in order, in the default style.
    pub fn open(points: Vec<Point>, fill_unit: LengthUnit) -> Result<Self, TooFewPoints> {
        Self::new(points, false, MIN_OPEN_POINTS, fill_unit)
    }

    /// A closed outline through `points`, in order, in the default style.
    pub fn closed(points: Vec<Point>, fill_unit: LengthUnit) -> Result<Self, TooFewPoints> {
        Self::new(points, true, MIN_CLOSED_POINTS, fill_unit)
    }

    fn new(
        points: Vec<Point>,
        closed: bool,
        min_points: usize,
        fill_unit: LengthUnit,
    ) -> Result<Self, TooFewPoints> {
        if points.len() < min_points {
            return Err(TooFewPoints(points.len()));
        }
        Ok(Self {
            points,
            closed,
            line_style: LineStyle::default(),
            line_color: DEFAULT_LINE_COLOR,
            line_opacity: DEFAULT_LINE_OPACITY,
            fill_style: FillStyle::default(),
            fill_color: DEFAULT_FILL_COLOR,
            fill_spacing: DEFAULT_FILL_SPACING,
            fill_unit,
            fill_angle: DEFAULT_FILL_ANGLE,
            fill_opacity: DEFAULT_FILL_OPACITY,
        })
    }

    /// The shape's points, in order.
    pub fn points(&self) -> &[Point] {
        &self.points
    }

    /// Whether the last point joins the first, so that the shape encloses an area.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn line_style(&self) -> LineStyle {
        self.line_style
    }

    pub fn line_color(&self) -> Rgb {
        self.line_color
    }

    pub fn line_opacity(&self) -> f32 {
        self.line_opacity
    }

    pub fn fill_style(&self) -> FillStyle {
        self.fill_style
    }

    pub fn fill_color(&self) -> Rgb {
        self.fill_color
    }

    /// The real-world distance between fill lines, in [`Self::fill_unit`].
    pub fn fill_spacing(&self) -> f64 {
        self.fill_spacing
    }

    pub fn fill_unit(&self) -> LengthUnit {
        self.fill_unit
    }

    /// The direction of the fill lines, in degrees clockwise from the image's x axis.
    pub fn fill_angle(&self) -> f64 {
        self.fill_angle
    }

    pub fn fill_opacity(&self) -> f32 {
        self.fill_opacity
    }

    /// The smallest axis-aligned box containing every point.
    pub fn bounds(&self) -> Bounds {
        Bounds::enclosing(self.points.iter().copied()).expect("a shape has points")
    }

    /// The centre of the enclosed area of a closed shape; otherwise the centre of the
    /// bounds.
    pub fn center(&self) -> Point {
        self.closed
            .then(|| polygon::centroid(&self.points))
            .flatten()
            .unwrap_or_else(|| self.bounds().center())
    }

    /// The shape after dragging bounding-box `handle` by `(dx, dy)` image pixels: every
    /// point stretches with the box ([`Bounds::stretch`]).
    pub fn resized(&self, handle: BoundsHandle, dx: f64, dy: f64) -> DrawnShape {
        let bounds = self.bounds();
        self.map_points(|point| bounds.stretch(handle, dx, dy, point))
    }

    /// The shape with point `index` moved to `to`; unchanged if there is no such point.
    pub fn with_point(&self, index: usize, to: Point) -> DrawnShape {
        let mut shape = self.clone();
        if let Some(point) = shape.points.get_mut(index) {
            *point = to;
        }
        shape
    }

    /// The shape with every point passed through `map`.
    pub fn map_points(&self, map: impl Fn(Point) -> Point) -> DrawnShape {
        let mut shape = self.clone();
        shape.points = self.points.iter().map(|&point| map(point)).collect();
        shape
    }

    /// A copy with the given line colour, e.g. to preview an edit in progress.
    pub fn with_line_color(&self, color: Rgb) -> DrawnShape {
        let mut shape = self.clone();
        shape.line_color = color;
        shape
    }

    /// A copy with the given line opacity, e.g. to preview an edit in progress.
    pub fn with_line_opacity(&self, opacity: f32) -> DrawnShape {
        let mut shape = self.clone();
        shape.line_opacity = opacity;
        shape
    }

    /// A copy with the given fill colour, e.g. to preview an edit in progress.
    pub fn with_fill_color(&self, color: Rgb) -> DrawnShape {
        let mut shape = self.clone();
        shape.fill_color = color;
        shape
    }

    /// A copy with the given fill opacity, e.g. to preview an edit in progress.
    pub fn with_fill_opacity(&self, opacity: f32) -> DrawnShape {
        let mut shape = self.clone();
        shape.fill_opacity = opacity;
        shape
    }

    /// The fill lines of a closed shape that lie in `region`, as image-space segments.
    ///
    /// They are laid out in real-world space (image pixels scaled by the calibrated X and Y
    /// scale), so their distance and angle are true in the real world even when the two
    /// axes have different scales. Enough lines are skipped to keep the ones returned at
    /// least `min_gap` image pixels apart.
    pub fn fill_lines(&self, scale: Scale, min_gap: f64, region: Bounds) -> Vec<(Point, Point)> {
        if !self.closed {
            return Vec::new();
        }
        let (x_scale, y_scale) = (scale.x_per_pixel, scale.y_per_pixel);
        let to_world = |point: Point| Point::new(point.x * x_scale, point.y * y_scale);
        let outline: Vec<Point> = self.points.iter().map(|&point| to_world(point)).collect();
        let world_region = Bounds {
            min: to_world(region.min),
            max: to_world(region.max),
        };
        let spacing = self.fill_unit.convert(self.fill_spacing, scale.unit);

        self.fill_style
            .line_offsets()
            .iter()
            .flat_map(|offset| {
                let direction = HatchDirection::at_angle(self.fill_angle + offset);
                // Offsets along the world normal correspond to this normal in the image, so
                // lines `spacing` apart are `spacing / |normal|` pixels apart.
                let (normal_x, normal_y) = direction.normal();
                let gap = spacing / (normal_x * x_scale).hypot(normal_y * y_scale);
                let step = (min_gap / gap).ceil().max(1.0);
                polygon::hatch_lines(&outline, direction, spacing * step, world_region)
            })
            .map(|(start, end)| {
                let to_image = |point: Point| Point::new(point.x / x_scale, point.y / y_scale);
                (to_image(start), to_image(end))
            })
            .collect()
    }

    /// Replaces the points; the caller keeps the count unchanged.
    pub(crate) fn set_points(&mut self, points: Vec<Point>) {
        self.points = points;
    }

    pub(crate) fn set_line_style(&mut self, style: LineStyle) {
        self.line_style = style;
    }

    pub(crate) fn set_line_color(&mut self, color: Rgb) {
        self.line_color = color;
    }

    pub(crate) fn set_line_opacity(&mut self, opacity: f32) {
        self.line_opacity = opacity;
    }

    pub(crate) fn set_fill_style(&mut self, style: FillStyle) {
        self.fill_style = style;
    }

    pub(crate) fn set_fill_color(&mut self, color: Rgb) {
        self.fill_color = color;
    }

    pub(crate) fn set_fill_spacing(&mut self, spacing: f64) {
        self.fill_spacing = spacing;
    }

    /// Changes the unit the fill spacing is in. The spacing value is kept as it is.
    pub(crate) fn set_fill_unit(&mut self, unit: LengthUnit) {
        self.fill_unit = unit;
    }

    pub(crate) fn set_fill_angle(&mut self, degrees: f64) {
        self.fill_angle = degrees;
    }

    pub(crate) fn set_fill_opacity(&mut self, opacity: f32) {
        self.fill_opacity = opacity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CM: LengthUnit = LengthUnit::Centimetre;
    /// 1 cm per image pixel along both axes.
    const UNIT_SCALE: Scale = Scale {
        x_per_pixel: 1.0,
        y_per_pixel: 1.0,
        unit: CM,
    };
    const EVERYWHERE: Bounds = Bounds {
        min: Point::new(-1000.0, -1000.0),
        max: Point::new(1000.0, 1000.0),
    };

    fn points(coordinates: &[(f64, f64)]) -> Vec<Point> {
        coordinates.iter().map(|&(x, y)| Point::new(x, y)).collect()
    }

    /// A 40 × 20 px rectangle from (10, 20).
    fn rectangle() -> DrawnShape {
        DrawnShape::closed(
            points(&[(10.0, 20.0), (50.0, 20.0), (50.0, 40.0), (10.0, 40.0)]),
            CM,
        )
        .unwrap()
    }

    fn rectangle_filled_with(style: FillStyle) -> DrawnShape {
        let mut shape = rectangle();
        shape.set_fill_style(style);
        shape
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn a_line_needs_two_points_and_a_closed_outline_three() {
        let two = points(&[(0.0, 0.0), (1.0, 1.0)]);

        assert_eq!(
            DrawnShape::open(vec![Point::default()], CM),
            Err(TooFewPoints(1))
        );
        assert!(!DrawnShape::open(two.clone(), CM).unwrap().is_closed());
        assert_eq!(DrawnShape::closed(two, CM), Err(TooFewPoints(2)));
        assert!(rectangle().is_closed());
    }

    #[test]
    fn a_new_shape_is_thin_and_unfilled_with_horizontal_fill_lines_in_the_given_unit() {
        let shape = DrawnShape::closed(
            points(&[(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)]),
            LengthUnit::Metre,
        )
        .unwrap();

        assert_eq!(shape.line_style(), LineStyle::Thin);
        assert_eq!(shape.line_color(), Rgb::new(255, 128, 0));
        assert_eq!(shape.line_opacity(), 1.0);
        assert_eq!(shape.fill_style(), FillStyle::None);
        assert_eq!(shape.fill_color(), Rgb::new(255, 128, 0));
        assert_eq!(shape.fill_spacing(), 10.0);
        assert_eq!(shape.fill_unit(), LengthUnit::Metre);
        assert_eq!(shape.fill_angle(), 0.0);
        assert_eq!(shape.fill_opacity(), 0.5);
    }

    #[test]
    fn fill_angles_are_kept_within_a_half_turn() {
        assert_eq!(normalized_fill_angle(0.0), 0.0);
        assert_eq!(normalized_fill_angle(45.0), 45.0);
        assert_eq!(normalized_fill_angle(180.0), 0.0);
        assert_eq!(normalized_fill_angle(270.0), 90.0);
        assert_eq!(normalized_fill_angle(-90.0), 90.0);
        assert_eq!(normalized_fill_angle(-30.0), 150.0);
        // Rounds up to exactly 180 if taken naively.
        assert_eq!(normalized_fill_angle(-1e-20), 0.0);

        assert!(is_valid_fill_angle(0.0));
        assert!(is_valid_fill_angle(179.5));
        assert!(!is_valid_fill_angle(180.0));
        assert!(!is_valid_fill_angle(-1.0));
        assert!(!is_valid_fill_angle(f64::NAN));
    }

    #[test]
    fn the_centre_of_a_closed_shape_is_its_centroid_and_of_a_line_its_bounds_centre() {
        let line = DrawnShape::open(points(&[(0.0, 0.0), (6.0, 0.0), (0.0, 3.0)]), CM).unwrap();
        let triangle =
            DrawnShape::closed(points(&[(0.0, 0.0), (6.0, 0.0), (0.0, 3.0)]), CM).unwrap();
        let flat = DrawnShape::closed(points(&[(0.0, 0.0), (2.0, 2.0), (4.0, 4.0)]), CM).unwrap();

        assert_eq!(line.center(), Point::new(3.0, 1.5));
        assert_eq!(triangle.center(), Point::new(2.0, 1.0));
        assert_eq!(flat.center(), Point::new(2.0, 2.0));
    }

    #[test]
    fn resizing_stretches_every_point_and_keeps_the_style() {
        let mut shape = rectangle();
        shape.set_line_style(LineStyle::Dashed);

        let resized = shape.resized(BoundsHandle::Right, 40.0, 0.0);

        assert_eq!(
            resized.points(),
            points(&[(10.0, 20.0), (90.0, 20.0), (90.0, 40.0), (10.0, 40.0)])
        );
        assert_eq!(resized.line_style(), LineStyle::Dashed);
        assert!(resized.is_closed());
    }

    #[test]
    fn moving_one_point_keeps_the_others() {
        let moved = rectangle().with_point(2, Point::new(70.0, 90.0));

        assert_eq!(moved.points()[2], Point::new(70.0, 90.0));
        assert_eq!(moved.points()[1], Point::new(50.0, 20.0));
        assert_eq!(rectangle().with_point(9, Point::default()), rectangle());
    }

    #[test]
    fn fill_lines_follow_the_fill_style() {
        let count = |style| {
            rectangle_filled_with(style)
                .fill_lines(UNIT_SCALE, 0.0, EVERYWHERE)
                .len()
        };

        assert_eq!(count(FillStyle::None), 0);
        assert_eq!(count(FillStyle::Solid), 0);
        // y = 20 and 30: a line along the bottom edge is not inside.
        assert_eq!(count(FillStyle::Lines), 2);
        // Those, and four crossing them.
        assert_eq!(count(FillStyle::Crosshatch), 6);
        assert!(FillStyle::Lines.has_lines() && FillStyle::Crosshatch.has_lines());
        assert!(!FillStyle::None.has_lines() && !FillStyle::Solid.has_lines());
    }

    #[test]
    fn fill_lines_are_horizontal_by_default_and_turn_with_the_fill_angle() {
        let mut shape = rectangle_filled_with(FillStyle::Lines);
        for (start, end) in shape.fill_lines(UNIT_SCALE, 0.0, EVERYWHERE) {
            assert_eq!(start.y, end.y);
        }

        shape.set_fill_angle(90.0);
        let vertical = shape.fill_lines(UNIT_SCALE, 0.0, EVERYWHERE);
        assert_eq!(vertical.len(), 4);
        for (start, end) in vertical {
            assert_eq!(start.x, end.x);
        }
    }

    #[test]
    fn crosshatch_lines_are_perpendicular_at_any_angle() {
        let mut shape = rectangle_filled_with(FillStyle::Crosshatch);
        shape.set_fill_angle(30.0);

        let lines = shape.fill_lines(UNIT_SCALE, 0.0, EVERYWHERE);

        let slope_of = |(a, b): &(Point, Point)| (b.y - a.y).atan2(b.x - a.x).to_degrees();
        let mut angles: Vec<f64> = lines
            .iter()
            .map(slope_of)
            .map(normalized_fill_angle)
            .collect();
        angles.sort_by(f64::total_cmp);
        assert!(near(angles[0], 30.0), "{angles:?}");
        assert!(near(*angles.last().unwrap(), 120.0), "{angles:?}");
    }

    #[test]
    fn fill_spacing_is_a_real_world_distance_in_its_own_unit() {
        // 2 cm per pixel, so 10 cm is 5 px; 0.2 m is 10 px.
        let scale = Scale {
            x_per_pixel: 2.0,
            y_per_pixel: 2.0,
            unit: CM,
        };
        let mut shape = rectangle_filled_with(FillStyle::Lines);
        let heights = |shape: &DrawnShape| -> Vec<f64> {
            let lines = shape.fill_lines(scale, 0.0, EVERYWHERE);
            lines.iter().map(|(start, _)| start.y).collect()
        };

        assert_eq!(heights(&shape), [20.0, 25.0, 30.0, 35.0]);

        shape.set_fill_unit(LengthUnit::Metre);
        shape.set_fill_spacing(0.2);
        assert_eq!(heights(&shape), [20.0, 30.0]);
    }

    #[test]
    fn fill_lines_are_true_in_the_real_world_when_the_axes_have_different_scales() {
        // 0.5 cm/px across and 2 cm/px down: 10 cm is 20 px across and 5 px down.
        let scale = Scale {
            x_per_pixel: 0.5,
            y_per_pixel: 2.0,
            unit: CM,
        };
        let mut shape = rectangle_filled_with(FillStyle::Lines);
        let lines = shape.fill_lines(scale, 0.0, EVERYWHERE);
        assert!(near(lines[1].0.y - lines[0].0.y, 5.0));

        shape.set_fill_angle(90.0);
        let lines = shape.fill_lines(scale, 0.0, EVERYWHERE);
        assert!(near((lines[1].0.x - lines[0].0.x).abs(), 20.0));

        // At 45° in the real world, the lines are 10 cm apart measured across them.
        shape.set_fill_angle(45.0);
        let lines = shape.fill_lines(scale, 0.0, EVERYWHERE);
        let world = |point: Point| (point.x * 0.5, point.y * 2.0);
        let across = |point: Point| {
            let (x, y) = world(point);
            (y - x) / 2f64.sqrt()
        };
        for (start, end) in &lines {
            assert!(near(across(*start), across(*end)), "not at 45°");
            let remainder = across(*start).rem_euclid(10.0);
            assert!(near(remainder, 0.0) || near(remainder, 10.0), "{remainder}");
        }
        assert!(!lines.is_empty());
    }

    #[test]
    fn fill_lines_are_skipped_when_they_would_be_too_close() {
        let shape = rectangle_filled_with(FillStyle::Lines);
        let heights = |min_gap| -> Vec<f64> {
            let lines = shape.fill_lines(UNIT_SCALE, min_gap, EVERYWHERE);
            lines.iter().map(|(start, _)| start.y).collect()
        };

        assert_eq!(heights(10.0), [20.0, 30.0]);
        // Still at multiples of the spacing, so a subset of the real lines.
        assert_eq!(heights(15.0), [20.0]);
    }

    #[test]
    fn a_line_has_no_fill_lines() {
        let mut line = DrawnShape::open(points(&[(0.0, 0.0), (50.0, 50.0)]), CM).unwrap();
        line.set_fill_style(FillStyle::Crosshatch);

        assert!(line.fill_lines(UNIT_SCALE, 0.0, EVERYWHERE).is_empty());
    }

    #[test]
    fn the_fill_style_names_are_distinct() {
        let names: std::collections::HashSet<_> =
            FillStyle::ALL.iter().map(|style| style.name()).collect();

        assert_eq!(names.len(), FillStyle::ALL.len());
    }
}
