//! How objects are drawn: the appearance of a line and of a fill, shared by measurements,
//! areas and drawn shapes, and the single edits that can be made to them.

use std::fmt;

use crate::calibration::Scale;
use crate::color::Rgb;
use crate::geometry::{Bounds, Point};
use crate::polygon::{self, HatchDirection};
use crate::units::LengthUnit;

/// Real-world distance between fill lines by default, in the fill's unit.
pub const DEFAULT_FILL_SPACING: f64 = 10.0;
/// Fill lines are horizontal by default.
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

/// How a line is drawn.
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

/// How an enclosed area is filled.
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

    /// The style's sets of fill lines, as angles from the fill angle; empty if it does not
    /// use lines.
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

/// Why a style value is not allowed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InvalidStyle {
    Opacity(f32),
    FillSpacing(f64),
    FillAngle(f64),
}

fn ensure_opacity(opacity: f32) -> Result<(), InvalidStyle> {
    if (0.0..=1.0).contains(&opacity) {
        Ok(())
    } else {
        Err(InvalidStyle::Opacity(opacity))
    }
}

fn ensure_spacing(spacing: f64) -> Result<(), InvalidStyle> {
    if spacing.is_finite() && spacing > 0.0 {
        Ok(())
    } else {
        Err(InvalidStyle::FillSpacing(spacing))
    }
}

fn ensure_angle(degrees: f64) -> Result<(), InvalidStyle> {
    if is_valid_fill_angle(degrees) {
        Ok(())
    } else {
        Err(InvalidStyle::FillAngle(degrees))
    }
}

/// How an object's line is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineAppearance {
    pub style: LineStyle,
    pub color: Rgb,
    /// From 0 (invisible) to 1 (opaque).
    pub opacity: f32,
}

impl LineAppearance {
    /// An opaque line.
    pub const fn new(style: LineStyle, color: Rgb) -> Self {
        Self {
            style,
            color,
            opacity: 1.0,
        }
    }

    pub fn validate(&self) -> Result<(), InvalidStyle> {
        ensure_opacity(self.opacity)
    }
}

/// How an object's enclosed area is filled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FillAppearance {
    pub style: FillStyle,
    pub color: Rgb,
    /// Real-world distance between fill lines, in `unit`.
    pub spacing: f64,
    pub unit: LengthUnit,
    /// Real-world direction of the fill lines, in degrees clockwise from the image's x axis
    /// (see [`normalized_fill_angle`]).
    pub angle: f64,
    /// From 0 (invisible) to 1 (opaque).
    pub opacity: f32,
}

impl FillAppearance {
    /// A fill with horizontal fill lines the default distance apart, in `unit`.
    pub const fn new(style: FillStyle, color: Rgb, opacity: f32, unit: LengthUnit) -> Self {
        Self {
            style,
            color,
            spacing: DEFAULT_FILL_SPACING,
            unit,
            angle: DEFAULT_FILL_ANGLE,
            opacity,
        }
    }

    pub fn validate(&self) -> Result<(), InvalidStyle> {
        ensure_opacity(self.opacity)?;
        ensure_spacing(self.spacing)?;
        ensure_angle(self.angle)
    }

    /// The fill lines of the closed outline through `points` that lie in `region`, as
    /// image-space segments.
    ///
    /// They are laid out in real-world space (image pixels scaled by the calibrated X and Y
    /// scale), so their distance and angle are true in the real world even when the two
    /// axes have different scales. Enough lines are skipped to keep the ones returned at
    /// least `min_gap` image pixels apart.
    pub fn lines(
        &self,
        points: &[Point],
        scale: Scale,
        min_gap: f64,
        region: Bounds,
    ) -> Vec<(Point, Point)> {
        let (x_scale, y_scale) = (scale.x_per_pixel, scale.y_per_pixel);
        let to_world = |point: Point| Point::new(point.x * x_scale, point.y * y_scale);
        let outline: Vec<Point> = points.iter().map(|&point| to_world(point)).collect();
        let world_region = Bounds {
            min: to_world(region.min),
            max: to_world(region.max),
        };
        let spacing = self.unit.convert(self.spacing, scale.unit);

        self.style
            .line_offsets()
            .iter()
            .flat_map(|offset| {
                let direction = HatchDirection::at_angle(self.angle + offset);
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
}

/// One change to how an object, or the next object a tool adds, is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StyleEdit {
    LineStyle(LineStyle),
    LineColor(Rgb),
    LineOpacity(f32),
    FillStyle(FillStyle),
    FillColor(Rgb),
    /// The distance between fill lines, in the fill's unit.
    FillSpacing(f64),
    /// The unit of the distance between fill lines, keeping its value.
    FillUnit(LengthUnit),
    /// The direction of the fill lines, in degrees in [0, 180).
    FillAngle(f64),
    FillOpacity(f32),
}

impl StyleEdit {
    pub fn validate(&self) -> Result<(), InvalidStyle> {
        match *self {
            Self::LineOpacity(opacity) | Self::FillOpacity(opacity) => ensure_opacity(opacity),
            Self::FillSpacing(spacing) => ensure_spacing(spacing),
            Self::FillAngle(degrees) => ensure_angle(degrees),
            Self::LineStyle(_)
            | Self::LineColor(_)
            | Self::FillStyle(_)
            | Self::FillColor(_)
            | Self::FillUnit(_) => Ok(()),
        }
    }

    /// `line` with this edit made, or `None` if this edit is not of a line.
    pub fn applied_to_line(&self, line: LineAppearance) -> Option<LineAppearance> {
        let mut line = line;
        match *self {
            Self::LineStyle(style) => line.style = style,
            Self::LineColor(color) => line.color = color,
            Self::LineOpacity(opacity) => line.opacity = opacity,
            _ => return None,
        }
        Some(line)
    }

    /// `fill` with this edit made, or `None` if this edit is not of a fill.
    pub fn applied_to_fill(&self, fill: FillAppearance) -> Option<FillAppearance> {
        let mut fill = fill;
        match *self {
            Self::FillStyle(style) => fill.style = style,
            Self::FillColor(color) => fill.color = color,
            Self::FillSpacing(spacing) => fill.spacing = spacing,
            Self::FillUnit(unit) => fill.unit = unit,
            Self::FillAngle(degrees) => fill.angle = degrees,
            Self::FillOpacity(opacity) => fill.opacity = opacity,
            _ => return None,
        }
        Some(fill)
    }
}

/// What the edit changes and to what, as written in action labels.
impl fmt::Display for StyleEdit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let percent = |opacity: f32| (opacity * 100.0).round();
        match *self {
            Self::LineStyle(style) => write!(f, "line style to {}", style.name()),
            Self::LineColor(color) => write!(f, "line color to {color}"),
            Self::LineOpacity(opacity) => write!(f, "line opacity to {}%", percent(opacity)),
            Self::FillStyle(style) => write!(f, "fill style to {}", style.name()),
            Self::FillColor(color) => write!(f, "fill color to {color}"),
            Self::FillSpacing(spacing) => write!(f, "fill spacing to {spacing}"),
            Self::FillUnit(unit) => write!(f, "fill units to {unit}"),
            Self::FillAngle(degrees) => write!(f, "fill angle to {degrees}°"),
            Self::FillOpacity(opacity) => write!(f, "fill opacity to {}%", percent(opacity)),
        }
    }
}

/// Something drawn with a line and, if it encloses an area, a fill.
pub trait Styled: Clone {
    fn line(&self) -> LineAppearance;
    /// `None` if it encloses no area.
    fn fill(&self) -> Option<FillAppearance>;
    fn line_mut(&mut self) -> &mut LineAppearance;
    /// `None` if it encloses no area.
    fn fill_mut(&mut self) -> Option<&mut FillAppearance>;

    /// A copy with `edit` made, or `None` if it edits a fill and there is none.
    fn with_edit(&self, edit: StyleEdit) -> Option<Self> {
        let mut edited = self.clone();
        if let Some(line) = edit.applied_to_line(self.line()) {
            *edited.line_mut() = line;
        } else {
            let fill = edit.applied_to_fill(self.fill()?)?;
            *edited.fill_mut()? = fill;
        }
        Some(edited)
    }

    /// Takes on the line and any fill of `style`.
    fn adopt(&mut self, style: &impl Styled) {
        *self.line_mut() = style.line();
        if let (Some(mine), Some(theirs)) = (self.fill_mut(), style.fill()) {
            *mine = theirs;
        }
    }
}

/// The style a tool gives the next object it adds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObjectStyle {
    pub line: LineAppearance,
    pub fill: Option<FillAppearance>,
}

impl Styled for ObjectStyle {
    fn line(&self) -> LineAppearance {
        self.line
    }

    fn fill(&self) -> Option<FillAppearance> {
        self.fill
    }

    fn line_mut(&mut self) -> &mut LineAppearance {
        &mut self.line
    }

    fn fill_mut(&mut self) -> Option<&mut FillAppearance> {
        self.fill.as_mut()
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
    const ORANGE: Rgb = Rgb::new(255, 128, 0);

    /// A 40 × 20 px rectangle from (10, 20).
    fn rectangle() -> [Point; 4] {
        [
            Point::new(10.0, 20.0),
            Point::new(50.0, 20.0),
            Point::new(50.0, 40.0),
            Point::new(10.0, 40.0),
        ]
    }

    fn fill(style: FillStyle) -> FillAppearance {
        FillAppearance::new(style, ORANGE, 0.5, CM)
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
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
    fn invalid_values_are_reported() {
        assert_eq!(
            StyleEdit::LineOpacity(1.5).validate(),
            Err(InvalidStyle::Opacity(1.5))
        );
        assert_eq!(
            StyleEdit::FillSpacing(0.0).validate(),
            Err(InvalidStyle::FillSpacing(0.0))
        );
        assert_eq!(
            StyleEdit::FillAngle(180.0).validate(),
            Err(InvalidStyle::FillAngle(180.0))
        );
        assert_eq!(StyleEdit::FillOpacity(0.5).validate(), Ok(()));

        let mut bad = fill(FillStyle::Solid);
        bad.spacing = f64::INFINITY;
        assert_eq!(
            bad.validate(),
            Err(InvalidStyle::FillSpacing(f64::INFINITY))
        );
        let mut bad = LineAppearance::new(LineStyle::Thin, ORANGE);
        bad.opacity = -0.5;
        assert_eq!(bad.validate(), Err(InvalidStyle::Opacity(-0.5)));
    }

    #[test]
    fn an_edit_changes_only_its_own_value() {
        let line = LineAppearance::new(LineStyle::Thin, ORANGE);

        assert_eq!(
            StyleEdit::LineOpacity(0.25).applied_to_line(line),
            Some(LineAppearance {
                opacity: 0.25,
                ..line
            })
        );
        assert_eq!(StyleEdit::FillOpacity(0.25).applied_to_line(line), None);

        let solid = fill(FillStyle::Solid);
        assert_eq!(
            StyleEdit::FillUnit(LengthUnit::Metre).applied_to_fill(solid),
            Some(FillAppearance {
                unit: LengthUnit::Metre,
                ..solid
            })
        );
        assert_eq!(StyleEdit::LineOpacity(0.25).applied_to_fill(solid), None);
    }

    #[test]
    fn fill_lines_follow_the_fill_style() {
        let count = |style| {
            fill(style)
                .lines(&rectangle(), UNIT_SCALE, 0.0, EVERYWHERE)
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
        let mut lines = fill(FillStyle::Lines);
        for (start, end) in lines.lines(&rectangle(), UNIT_SCALE, 0.0, EVERYWHERE) {
            assert_eq!(start.y, end.y);
        }

        lines.angle = 90.0;
        let vertical = lines.lines(&rectangle(), UNIT_SCALE, 0.0, EVERYWHERE);
        assert_eq!(vertical.len(), 4);
        for (start, end) in vertical {
            assert_eq!(start.x, end.x);
        }
    }

    #[test]
    fn crosshatch_lines_are_perpendicular_at_any_angle() {
        let mut crosshatch = fill(FillStyle::Crosshatch);
        crosshatch.angle = 30.0;

        let lines = crosshatch.lines(&rectangle(), UNIT_SCALE, 0.0, EVERYWHERE);

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
        let mut lines = fill(FillStyle::Lines);
        let heights = |fill: &FillAppearance| -> Vec<f64> {
            let found = fill.lines(&rectangle(), scale, 0.0, EVERYWHERE);
            found.iter().map(|(start, _)| start.y).collect()
        };

        assert_eq!(heights(&lines), [20.0, 25.0, 30.0, 35.0]);

        lines.unit = LengthUnit::Metre;
        lines.spacing = 0.2;
        assert_eq!(heights(&lines), [20.0, 30.0]);
    }

    #[test]
    fn fill_lines_are_true_in_the_real_world_when_the_axes_have_different_scales() {
        // 0.5 cm/px across and 2 cm/px down: 10 cm is 20 px across and 5 px down.
        let scale = Scale {
            x_per_pixel: 0.5,
            y_per_pixel: 2.0,
            unit: CM,
        };
        let mut style = fill(FillStyle::Lines);
        let found = style.lines(&rectangle(), scale, 0.0, EVERYWHERE);
        assert!(near(found[1].0.y - found[0].0.y, 5.0));

        style.angle = 90.0;
        let found = style.lines(&rectangle(), scale, 0.0, EVERYWHERE);
        assert!(near((found[1].0.x - found[0].0.x).abs(), 20.0));

        // At 45° in the real world, the lines are 10 cm apart measured across them.
        style.angle = 45.0;
        let found = style.lines(&rectangle(), scale, 0.0, EVERYWHERE);
        let across = |point: Point| (point.y * 2.0 - point.x * 0.5) / 2f64.sqrt();
        for (start, end) in &found {
            assert!(near(across(*start), across(*end)), "not at 45°");
            let remainder = across(*start).rem_euclid(10.0);
            assert!(near(remainder, 0.0) || near(remainder, 10.0), "{remainder}");
        }
        assert!(!found.is_empty());
    }

    #[test]
    fn fill_lines_are_skipped_when_they_would_be_too_close() {
        let lines = fill(FillStyle::Lines);
        let heights = |min_gap| -> Vec<f64> {
            let found = lines.lines(&rectangle(), UNIT_SCALE, min_gap, EVERYWHERE);
            found.iter().map(|(start, _)| start.y).collect()
        };

        assert_eq!(heights(10.0), [20.0, 30.0]);
        // Still at multiples of the spacing, so a subset of the real lines.
        assert_eq!(heights(15.0), [20.0]);
    }

    #[test]
    fn the_fill_style_names_are_distinct() {
        let names: std::collections::HashSet<_> =
            FillStyle::ALL.iter().map(|style| style.name()).collect();

        assert_eq!(names.len(), FillStyle::ALL.len());
    }

    #[test]
    fn an_edit_of_something_without_a_fill_is_refused_only_for_the_fill() {
        let line = LineAppearance::new(LineStyle::Thin, ORANGE);
        let unfilled = ObjectStyle { line, fill: None };

        assert_eq!(unfilled.with_edit(StyleEdit::FillOpacity(0.5)), None);
        assert_eq!(
            unfilled.with_edit(StyleEdit::LineStyle(LineStyle::Thick)),
            Some(ObjectStyle {
                line: LineAppearance {
                    style: LineStyle::Thick,
                    ..line
                },
                fill: None
            })
        );
    }

    #[test]
    fn adopting_a_style_takes_its_line_and_any_fill_it_has() {
        let line = LineAppearance::new(LineStyle::Dashed, Rgb::GREEN);
        let filled = ObjectStyle {
            line,
            fill: Some(fill(FillStyle::Solid)),
        };
        let mut unfilled = ObjectStyle {
            line: LineAppearance::new(LineStyle::Thin, ORANGE),
            fill: None,
        };
        let mut other = ObjectStyle {
            line: LineAppearance::new(LineStyle::Thin, ORANGE),
            fill: Some(fill(FillStyle::Lines)),
        };

        unfilled.adopt(&filled);
        other.adopt(&filled);

        assert_eq!(unfilled.line, line);
        assert_eq!(unfilled.fill, None);
        assert_eq!(other, filled);
    }

    #[test]
    fn edits_describe_themselves_for_action_labels() {
        let labels = [
            (
                StyleEdit::LineStyle(LineStyle::Dashed),
                "line style to Dashed",
            ),
            (StyleEdit::LineColor(ORANGE), "line color to #FF8000"),
            (StyleEdit::LineOpacity(0.25), "line opacity to 25%"),
            (
                StyleEdit::FillStyle(FillStyle::Lines),
                "fill style to Lines",
            ),
            (StyleEdit::FillColor(ORANGE), "fill color to #FF8000"),
            (StyleEdit::FillSpacing(50.0), "fill spacing to 50"),
            (StyleEdit::FillUnit(LengthUnit::Metre), "fill units to m"),
            (StyleEdit::FillAngle(22.5), "fill angle to 22.5°"),
            (StyleEdit::FillOpacity(0.5), "fill opacity to 50%"),
        ];
        for (edit, label) in labels {
            assert_eq!(edit.to_string(), label);
        }
    }
}
