//! Grids: regularly spaced lines over the whole document, aligned to a reference line (a
//! scale calibration line or a measurement), with cells sized in real-world units.

use std::fmt;

use crate::calibration::{Line, ScaleError};
use crate::color::Rgb;
use crate::document::Document;
use crate::geometry::{Bounds, Point};
use crate::object::ObjectId;
use crate::units::LengthUnit;

/// The initial cell size, in the grid's unit.
pub const DEFAULT_CELL_SIZE: f64 = 10.0;
pub const DEFAULT_COLOR: Rgb = Rgb::GREEN;
pub const DEFAULT_OPACITY: f32 = 0.5;
/// A safety limit: a request for more lines than this within one region is refused.
pub const MAX_LINES: usize = 10_000;
/// Fraction of a cell by which a line may fall outside a region and still be included.
const INDEX_TOLERANCE: f64 = 1e-9;

/// The line a grid is aligned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridSource {
    /// One of the scale calibration lines.
    ScaleLine(Line),
    /// A measurement line, by id so it survives other measurements being deleted.
    Measurement(ObjectId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CellShape {
    #[default]
    Square,
    Rectangle,
}

impl CellShape {
    pub const ALL: [CellShape; 2] = [CellShape::Square, CellShape::Rectangle];

    pub fn name(self) -> &'static str {
        match self {
            CellShape::Square => "Square",
            CellShape::Rectangle => "Rectangle",
        }
    }
}

/// Which cell size an edit sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellDimension {
    /// The side of a square cell; sets both A and B.
    Side,
    /// Along the alignment source.
    A,
    /// Perpendicular to the alignment source.
    B,
}

impl fmt::Display for CellDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CellDimension::Side => "L",
            CellDimension::A => "A",
            CellDimension::B => "B",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridError {
    /// The scale line or measurement the grid is aligned to no longer exists.
    SourceMissing,
    /// The alignment source has zero length, so it has no direction.
    ZeroLengthSource,
    /// Cell sizes are real-world lengths, so a scale is needed.
    NoScale(ScaleError),
    /// More than [`MAX_LINES`] lines would be needed within the requested region.
    TooDense,
}

impl fmt::Display for GridError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceMissing => f.write_str("its alignment line no longer exists"),
            Self::ZeroLengthSource => f.write_str("its alignment line has zero length"),
            Self::NoScale(reason) => write!(f, "it needs a scale ({reason})"),
            Self::TooDense => f.write_str("the cells are too small to draw"),
        }
    }
}

impl std::error::Error for GridError {}

/// A grid and its display properties.
#[derive(Debug, Clone, PartialEq)]
pub struct Grid {
    source: GridSource,
    shape: CellShape,
    size_a: f64,
    size_b: f64,
    unit: LengthUnit,
    color: Rgb,
    opacity: f32,
}

impl Grid {
    /// A square-celled, 50 % green grid with 10-unit cells.
    pub fn new(source: GridSource, unit: LengthUnit) -> Self {
        Self {
            source,
            shape: CellShape::default(),
            size_a: DEFAULT_CELL_SIZE,
            size_b: DEFAULT_CELL_SIZE,
            unit,
            color: DEFAULT_COLOR,
            opacity: DEFAULT_OPACITY,
        }
    }

    pub fn source(&self) -> GridSource {
        self.source
    }

    pub fn shape(&self) -> CellShape {
        self.shape
    }

    /// The stored size for `dimension` (`Side` reads A).
    pub fn size(&self, dimension: CellDimension) -> f64 {
        match dimension {
            CellDimension::Side | CellDimension::A => self.size_a,
            CellDimension::B => self.size_b,
        }
    }

    /// Cell size along and across the alignment source, honouring the shape.
    pub fn cell_size(&self) -> (f64, f64) {
        match self.shape {
            CellShape::Square => (self.size_a, self.size_a),
            CellShape::Rectangle => (self.size_a, self.size_b),
        }
    }

    /// The unit the cell sizes are in (the source's unit when the grid was added).
    pub fn unit(&self) -> LengthUnit {
        self.unit
    }

    pub fn color(&self) -> Rgb {
        self.color
    }

    /// Opacity from 0 (invisible) to 1 (opaque).
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// A copy with a different colour (for previews).
    pub fn with_color(&self, color: Rgb) -> Self {
        Self {
            color,
            ..self.clone()
        }
    }

    /// A copy with a different opacity (for previews).
    pub fn with_opacity(&self, opacity: f32) -> Self {
        Self {
            opacity,
            ..self.clone()
        }
    }

    pub(crate) fn set_shape(&mut self, shape: CellShape) {
        self.shape = shape;
    }

    pub(crate) fn set_size(&mut self, dimension: CellDimension, size: f64) {
        match dimension {
            CellDimension::Side => {
                self.size_a = size;
                self.size_b = size;
            }
            CellDimension::A => self.size_a = size,
            CellDimension::B => self.size_b = size,
        }
    }

    /// Changes the unit the cell sizes are in. The size values are kept as they are.
    pub(crate) fn set_unit(&mut self, unit: LengthUnit) {
        self.unit = unit;
    }

    pub(crate) fn set_color(&mut self, color: Rgb) {
        self.color = color;
    }

    pub(crate) fn set_opacity(&mut self, opacity: f32) {
        self.opacity = opacity;
    }

    /// The image-space start and end of the line the grid is aligned to.
    pub fn source_line(&self, document: &Document) -> Result<(Point, Point), GridError> {
        match self.source {
            GridSource::ScaleLine(line) => document
                .calibration()
                .points()
                .map(|points| points.line(line))
                .ok_or(GridError::SourceMissing),
            GridSource::Measurement(id) => document
                .measurement_index(id)
                .map(|index| {
                    let measurement = &document.measurements()[index];
                    (measurement.start(), measurement.end())
                })
                .ok_or(GridError::SourceMissing),
        }
    }

    /// Lays the grid out in real-world space (image pixels scaled by the calibrated X and
    /// Y scale), so cells are true squares/rectangles in the real world even when the two
    /// axes have different scales.
    fn layout(&self, document: &Document) -> Result<Layout, GridError> {
        let (start, end) = self.source_line(document)?;
        let scale = document.calibration().scale().map_err(GridError::NoScale)?;
        let to_world = Vector::new(scale.x_per_pixel, scale.y_per_pixel);
        let world = |p: Point| Vector::new(p.x * to_world.x, p.y * to_world.y);

        let origin = world(start);
        let along = (world(end) - origin)
            .normalized()
            .ok_or(GridError::ZeroLengthSource)?;
        let across = along.perpendicular();
        let (size_a, size_b) = self.cell_size();
        let (spacing_a, spacing_b) = (
            self.unit.convert(size_a, scale.unit),
            self.unit.convert(size_b, scale.unit),
        );
        Ok(Layout {
            to_world,
            origin,
            // Lines parallel to the source are offset across it, and vice versa.
            families: [(along, across, spacing_b), (across, along, spacing_a)],
        })
    }

    /// The smallest distance, in image pixels, between neighbouring parallel grid lines.
    /// Fails for the same reasons as [`Grid::lines`] (except [`GridError::TooDense`]).
    pub fn line_spacing(&self, document: &Document) -> Result<f64, GridError> {
        let layout = self.layout(document)?;
        Ok(layout
            .families
            .iter()
            .map(|&(_, offset_axis, spacing)| {
                // Offsets along `offset_axis` in world space correspond to this normal in
                // image space, so lines `spacing` apart are `spacing / |normal|` pixels apart.
                let normal = Vector::new(
                    offset_axis.x * layout.to_world.x,
                    offset_axis.y * layout.to_world.y,
                );
                spacing / normal.x.hypot(normal.y)
            })
            .fold(f64::INFINITY, f64::min))
    }

    /// The grid's lines as image-space segments, clipped to the part of the document
    /// inside `region` (image coordinates), e.g. the part currently visible.
    ///
    /// One family of lines runs parallel to the alignment source and is spaced by B; the
    /// other runs perpendicular, spaced by A. Both families pass through the source's
    /// start point, so the lines are in the same place whatever `region` is.
    pub fn lines(
        &self,
        document: &Document,
        region: Bounds,
    ) -> Result<Vec<(Point, Point)>, GridError> {
        let Layout {
            to_world,
            origin,
            families,
        } = self.layout(document)?;
        let to_image = |v: Vector| Point::new(v.x / to_world.x, v.y / to_world.y);
        let document_bounds = document.bounds();
        let min = Point::new(
            region.min.x.max(document_bounds.min.x),
            region.min.y.max(document_bounds.min.y),
        );
        let max = Point::new(
            region.max.x.min(document_bounds.max.x),
            region.max.y.min(document_bounds.max.y),
        );
        if min.x >= max.x || min.y >= max.y {
            return Ok(Vec::new());
        }
        let world = WorldRect {
            min: Vector::new(min.x * to_world.x, min.y * to_world.y),
            max: Vector::new(max.x * to_world.x, max.y * to_world.y),
        };

        let mut ranges = Vec::with_capacity(families.len());
        let mut count = 0usize;
        for &(_, offset_axis, spacing) in &families {
            let (low, high) = world.projection_range(origin, offset_axis);
            // The tolerance keeps lines lying exactly on the region's edge.
            let first = (low / spacing - INDEX_TOLERANCE).ceil();
            let last = (high / spacing + INDEX_TOLERANCE).floor();
            let lines = if last >= first {
                (last - first) as usize + 1
            } else {
                0
            };
            count = count.saturating_add(lines);
            if count > MAX_LINES {
                return Err(GridError::TooDense);
            }
            ranges.push((first as i64, last as i64));
        }

        let mut segments = Vec::with_capacity(count);
        for (&(direction, offset_axis, spacing), &(first, last)) in families.iter().zip(&ranges) {
            for k in first..=last {
                let through = origin + offset_axis * (k as f64 * spacing);
                if let Some((a, b)) = world.clip(through, direction) {
                    segments.push((to_image(a), to_image(b)));
                }
            }
        }
        Ok(segments)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vector {
    x: f64,
    y: f64,
}

impl Vector {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn dot(self, other: Vector) -> f64 {
        self.x * other.x + self.y * other.y
    }

    fn normalized(self) -> Option<Vector> {
        let length = self.x.hypot(self.y);
        (length > 0.0).then(|| Vector::new(self.x / length, self.y / length))
    }

    fn perpendicular(self) -> Vector {
        Vector::new(-self.y, self.x)
    }
}

impl std::ops::Add for Vector {
    type Output = Vector;
    fn add(self, other: Vector) -> Vector {
        Vector::new(self.x + other.x, self.y + other.y)
    }
}

impl std::ops::Sub for Vector {
    type Output = Vector;
    fn sub(self, other: Vector) -> Vector {
        Vector::new(self.x - other.x, self.y - other.y)
    }
}

impl std::ops::Mul<f64> for Vector {
    type Output = Vector;
    fn mul(self, factor: f64) -> Vector {
        Vector::new(self.x * factor, self.y * factor)
    }
}

/// A grid laid out in real-world space.
struct Layout {
    /// Real-world units per image pixel along X and Y.
    to_world: Vector,
    /// The alignment source's start point.
    origin: Vector,
    /// For each family of parallel lines: the line direction, the axis the lines are
    /// offset along, and the spacing between them.
    families: [(Vector, Vector, f64); 2],
}

/// An axis-aligned rectangle in real-world coordinates.
struct WorldRect {
    min: Vector,
    max: Vector,
}

impl WorldRect {
    fn corners(&self) -> [Vector; 4] {
        [
            self.min,
            Vector::new(self.max.x, self.min.y),
            self.max,
            Vector::new(self.min.x, self.max.y),
        ]
    }

    /// Smallest and largest offset of the rectangle from `origin` along unit `axis`.
    fn projection_range(&self, origin: Vector, axis: Vector) -> (f64, f64) {
        self.corners()
            .iter()
            .map(|&corner| (corner - origin).dot(axis))
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                (low.min(value), high.max(value))
            })
    }

    /// The part of the infinite line through `point` along `direction` that lies inside
    /// the rectangle (Liang–Barsky), or `None` if it misses or only touches a corner.
    fn clip(&self, point: Vector, direction: Vector) -> Option<(Vector, Vector)> {
        let mut t_min = f64::NEG_INFINITY;
        let mut t_max = f64::INFINITY;
        for (position, delta, low, high) in [
            (point.x, direction.x, self.min.x, self.max.x),
            (point.y, direction.y, self.min.y, self.max.y),
        ] {
            if delta.abs() < 1e-12 {
                if position < low - 1e-9 || position > high + 1e-9 {
                    return None;
                }
            } else {
                let (a, b) = ((low - position) / delta, (high - position) / delta);
                t_min = t_min.max(a.min(b));
                t_max = t_max.min(a.max(b));
            }
        }
        (t_max - t_min > 1e-9).then(|| (point + direction * t_min, point + direction * t_max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::calibration::{CalibrationMethod, ScalePoints};
    use crate::measurement::Measurement;

    /// A 100×50 document calibrated at `sx` × `sy` cm per pixel, with one measurement
    /// along `measurement` and scale line A running right from the origin.
    fn document(sx: f64, sy: f64, measurement: (Point, Point)) -> Document {
        let mut document = Document::new(100, 50);
        let points = ScalePoints::new(
            CalibrationMethod::ThreePoint,
            vec![
                Point::new(0.0, 0.0),
                Point::new(20.0, 0.0),
                Point::new(20.0, 40.0),
            ],
        )
        .unwrap();
        for action in [
            Action::SetScalePoints(points),
            Action::SetDistance {
                line: Line::A,
                distance: Some(20.0 * sx),
            },
            Action::SetDistance {
                line: Line::B,
                distance: Some(40.0 * sy),
            },
            Action::AddMeasurement(Measurement::new(
                measurement.0,
                measurement.1,
                LengthUnit::Centimetre,
            )),
        ] {
            action.apply(&mut document).unwrap();
        }
        document
    }

    fn unit_scale_document() -> Document {
        document(1.0, 1.0, (Point::new(10.0, 10.0), Point::new(10.0, 40.0)))
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn is_vertical(segment: &(Point, Point)) -> bool {
        near(segment.0.x, segment.1.x)
    }

    #[test]
    fn new_grids_have_the_default_properties() {
        let grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Metre);

        assert_eq!(grid.shape(), CellShape::Square);
        assert_eq!(grid.cell_size(), (10.0, 10.0));
        assert_eq!(grid.unit(), LengthUnit::Metre);
        assert_eq!(grid.color(), Rgb::new(0, 255, 0));
        assert_eq!(grid.opacity(), 0.5);
    }

    #[test]
    fn square_cells_use_size_a_for_both_dimensions() {
        let mut grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Metre);
        grid.set_size(CellDimension::A, 4.0);
        grid.set_size(CellDimension::B, 7.0);

        assert_eq!(grid.cell_size(), (4.0, 4.0));
        grid.set_shape(CellShape::Rectangle);
        assert_eq!(grid.cell_size(), (4.0, 7.0));
    }

    #[test]
    fn setting_the_side_sets_both_sizes() {
        let mut grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Metre);

        grid.set_size(CellDimension::Side, 3.0);
        grid.set_shape(CellShape::Rectangle);

        assert_eq!(grid.cell_size(), (3.0, 3.0));
    }

    #[test]
    fn axis_aligned_grid_covers_the_document() {
        // Source: scale line A, horizontal from the origin; 1 cm per px; 10 cm cells.
        let grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);

        let lines = grid
            .lines(&unit_scale_document(), unit_scale_document().bounds())
            .unwrap();

        let mut xs: Vec<f64> = lines
            .iter()
            .filter(|s| is_vertical(s))
            .map(|s| s.0.x)
            .collect();
        xs.sort_by(f64::total_cmp);
        let horizontal = lines.iter().filter(|s| !is_vertical(s)).count();
        assert_eq!(xs, (0..=10).map(|k| k as f64 * 10.0).collect::<Vec<_>>());
        assert_eq!(horizontal, 6); // y = 0, 10, …, 50
        // Vertical lines span the full document height.
        for (a, b) in lines.iter().filter(|s| is_vertical(s)) {
            assert!(near(a.y.min(b.y), 0.0) && near(a.y.max(b.y), 50.0));
        }
    }

    #[test]
    fn rectangle_cells_space_lines_by_a_along_and_b_across_the_source() {
        let mut grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);
        grid.set_shape(CellShape::Rectangle);
        grid.set_size(CellDimension::A, 25.0);
        grid.set_size(CellDimension::B, 50.0);

        let lines = grid
            .lines(&unit_scale_document(), unit_scale_document().bounds())
            .unwrap();

        // Across lines (vertical, spaced by A = 25): x = 0, 25, 50, 75, 100.
        assert_eq!(lines.iter().filter(|s| is_vertical(s)).count(), 5);
        // Along lines (horizontal, spaced by B = 50): y = 0, 50.
        assert_eq!(lines.iter().filter(|s| !is_vertical(s)).count(), 2);
    }

    #[test]
    fn grid_is_aligned_to_and_passes_through_the_measurement() {
        // Vertical measurement from (10, 10): lines pass through x = 10 and y = 10.
        let grid = Grid::new(
            GridSource::Measurement(unit_scale_document().measurements()[0].id()),
            LengthUnit::Centimetre,
        );

        let lines = grid
            .lines(&unit_scale_document(), unit_scale_document().bounds())
            .unwrap();

        assert!(lines.iter().any(|s| is_vertical(s) && near(s.0.x, 10.0)));
        assert!(lines.iter().any(|s| !is_vertical(s) && near(s.0.y, 10.0)));
        assert!(lines.iter().all(|s| is_vertical(s) || near(s.0.y, s.1.y)));
    }

    #[test]
    fn diagonal_grid_lines_are_one_cell_apart() {
        let document = document(1.0, 1.0, (Point::new(0.0, 0.0), Point::new(30.0, 30.0)));
        let id = document.measurements()[0].id();
        let grid = Grid::new(GridSource::Measurement(id), LengthUnit::Centimetre);

        let lines = grid.lines(&document, document.bounds()).unwrap();

        // Every line is at 45° and lies a whole number of 10 cm cells from the origin,
        // measured perpendicular to it: along-lines have (x - y) / √2 constant, across-
        // lines have (x + y) / √2 constant.
        let whole_cells = |offset: f64| {
            let remainder = (offset / 2f64.sqrt()).abs() % 10.0;
            near(remainder, 0.0) || near(remainder, 10.0)
        };
        for (a, b) in &lines {
            let along = near(a.x - a.y, b.x - b.y) && whole_cells(a.x - a.y);
            let across = near(a.x + a.y, b.x + b.y) && whole_cells(a.x + a.y);
            assert!(along || across, "{a:?} - {b:?}");
        }
        assert!(!lines.is_empty());
    }

    #[test]
    fn cells_are_square_in_the_real_world_when_axes_have_different_scales() {
        // 0.5 cm/px across, 2 cm/px down: a 10 cm cell is 20 px wide and 5 px tall.
        let document = document(0.5, 2.0, (Point::new(0.0, 0.0), Point::new(1.0, 0.0)));
        let grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);

        let lines = grid.lines(&document, document.bounds()).unwrap();

        let mut xs: Vec<f64> = lines
            .iter()
            .filter(|s| is_vertical(s))
            .map(|s| s.0.x)
            .collect();
        let mut ys: Vec<f64> = lines
            .iter()
            .filter(|s| !is_vertical(s))
            .map(|s| s.0.y)
            .collect();
        xs.sort_by(f64::total_cmp);
        ys.sort_by(f64::total_cmp);
        assert!(near(xs[1] - xs[0], 20.0));
        assert!(near(ys[1] - ys[0], 5.0));
    }

    #[test]
    fn cell_size_is_converted_from_the_grid_unit() {
        // 1 cm per px; a 0.1 m cell is 10 px.
        let grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Metre);
        let mut tiny = grid.clone();
        tiny.set_size(CellDimension::Side, 0.1);

        let lines = tiny
            .lines(&unit_scale_document(), unit_scale_document().bounds())
            .unwrap();

        assert_eq!(lines.iter().filter(|s| is_vertical(s)).count(), 11);
    }

    #[test]
    fn grids_need_a_scale_and_an_existing_source() {
        let grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);
        let mut document = unit_scale_document();
        let id = document.measurements()[0].id();

        Action::SetDistance {
            line: Line::B,
            distance: None,
        }
        .apply(&mut document)
        .unwrap();
        assert!(matches!(
            grid.lines(&document, document.bounds()),
            Err(GridError::NoScale(_))
        ));

        Action::DeleteMeasurement { index: 0 }
            .apply(&mut document)
            .unwrap();
        let orphan = Grid::new(GridSource::Measurement(id), LengthUnit::Centimetre);
        assert_eq!(
            orphan.lines(&document, document.bounds()),
            Err(GridError::SourceMissing)
        );
        assert_eq!(
            Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre)
                .lines(&Document::new(10, 10), Document::new(10, 10).bounds()),
            Err(GridError::SourceMissing)
        );
    }

    #[test]
    fn too_many_lines_are_refused() {
        let mut grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);
        grid.set_size(CellDimension::Side, 0.001);

        assert_eq!(
            grid.lines(&unit_scale_document(), unit_scale_document().bounds()),
            Err(GridError::TooDense)
        );
    }

    fn region(min: (f64, f64), max: (f64, f64)) -> Bounds {
        Bounds::around(Point::new(min.0, min.1), Point::new(max.0, max.1))
    }

    #[test]
    fn tiny_cells_are_drawn_within_a_small_region() {
        // 1 cm per px; 0.01 cm cells are 0.01 px apart.
        let mut grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);
        grid.set_size(CellDimension::Side, 0.01);
        let document = unit_scale_document();

        let lines = grid
            .lines(&document, region((30.0, 20.0), (30.5, 20.5)))
            .unwrap();

        let mut xs: Vec<f64> = lines
            .iter()
            .filter(|s| is_vertical(s))
            .map(|s| s.0.x)
            .collect();
        xs.sort_by(f64::total_cmp);
        assert_eq!(xs.len(), 51);
        assert!(near(xs[0], 30.0) && near(xs[50], 30.5));
        for segment in lines.iter().filter(|s| is_vertical(s)) {
            assert!(near(segment.0.y.min(segment.1.y), 20.0));
            assert!(near(segment.0.y.max(segment.1.y), 20.5));
        }
    }

    #[test]
    fn a_region_does_not_move_the_lines() {
        let grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);
        let document = unit_scale_document();

        let lines = grid
            .lines(&document, region((15.0, -100.0), (45.0, 100.0)))
            .unwrap();

        let mut xs: Vec<f64> = lines
            .iter()
            .filter(|s| is_vertical(s))
            .map(|s| s.0.x)
            .collect();
        xs.sort_by(f64::total_cmp);
        assert_eq!(xs, [20.0, 30.0, 40.0]);
        // The region is also clipped to the document.
        assert!(lines.iter().all(|s| s.0.y >= 0.0 && s.1.y <= 50.0));
    }

    #[test]
    fn a_region_outside_the_document_has_no_lines() {
        let grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);
        let document = unit_scale_document();

        assert_eq!(
            grid.lines(&document, region((200.0, 0.0), (300.0, 50.0))),
            Ok(Vec::new())
        );
    }

    #[test]
    fn line_spacing_is_the_smallest_gap_in_image_pixels() {
        let grid = Grid::new(GridSource::ScaleLine(Line::A), LengthUnit::Centimetre);
        assert!(near(
            grid.line_spacing(&unit_scale_document()).unwrap(),
            10.0
        ));

        // 2 cm/px along X and 0.5 cm/px along Y: 10 cm is 5 px across and 20 px down.
        let stretched = document(2.0, 0.5, (Point::new(0.0, 0.0), Point::new(1.0, 1.0)));
        assert!(near(grid.line_spacing(&stretched).unwrap(), 5.0));

        let mut no_scale = unit_scale_document();
        Action::SetDistance {
            line: Line::A,
            distance: None,
        }
        .apply(&mut no_scale)
        .unwrap();
        assert!(matches!(
            grid.line_spacing(&no_scale),
            Err(GridError::NoScale(_))
        ));
    }
}
