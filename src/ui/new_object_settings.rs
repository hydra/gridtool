//! What a tool gives the next object it adds: its unit and style, chosen in the tool
//! controls row. This is presentation state: it is not part of the document and is not
//! undoable. Units follow the calibration's unit until the user chooses one.

use gridtool::area;
use gridtool::measurement;
use gridtool::shape;
use gridtool::style::{ObjectStyle, StyleEdit, Styled};
use gridtool::units::LengthUnit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewObjectSettings {
    unit: Option<LengthUnit>,
    fill_unit: Option<LengthUnit>,
    style: ObjectStyle,
}

impl NewObjectSettings {
    fn following_calibration(style: ObjectStyle) -> Self {
        Self {
            unit: None,
            fill_unit: None,
            style,
        }
    }

    /// The settings a new measurement starts from.
    pub fn for_measurements() -> Self {
        Self::following_calibration(ObjectStyle {
            line: measurement::DEFAULT_LINE,
            fill: None,
        })
    }

    /// The settings a new area starts from.
    pub fn for_areas() -> Self {
        Self::following_calibration(ObjectStyle {
            line: area::DEFAULT_LINE,
            fill: Some(area::default_fill(LengthUnit::default())),
        })
    }

    /// The settings a new drawn shape starts from.
    pub fn for_shapes() -> Self {
        Self::following_calibration(ObjectStyle {
            line: shape::DEFAULT_LINE,
            fill: Some(shape::default_fill(LengthUnit::default())),
        })
    }

    /// The unit new objects are reported in, given the calibration's unit.
    pub fn unit(&self, calibration_unit: LengthUnit) -> LengthUnit {
        self.unit.unwrap_or(calibration_unit)
    }

    pub fn set_unit(&mut self, unit: LengthUnit) {
        self.unit = Some(unit);
    }

    /// The unit of the distance between new objects' fill lines, given the calibration's
    /// unit.
    pub fn fill_unit(&self, calibration_unit: LengthUnit) -> LengthUnit {
        self.fill_unit.unwrap_or(calibration_unit)
    }

    /// The style new objects get, given the calibration's unit.
    pub fn style(&self, calibration_unit: LengthUnit) -> ObjectStyle {
        let mut style = self.style;
        if let Some(fill) = style.fill_mut() {
            fill.unit = self.fill_unit(calibration_unit);
        }
        style
    }

    /// Makes `edit` to the style new objects get. An edit of a fill is ignored if new
    /// objects have none.
    pub fn apply(&mut self, edit: StyleEdit) {
        if let StyleEdit::FillUnit(unit) = edit {
            self.fill_unit = Some(unit);
        }
        if edit.validate().is_ok()
            && let Some(style) = self.style.with_edit(edit)
        {
            self.style = style;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::color::Rgb;
    use gridtool::style::{FillStyle, LineStyle};

    #[test]
    fn units_follow_the_calibration_until_one_is_chosen() {
        let mut settings = NewObjectSettings::for_areas();
        assert_eq!(settings.unit(LengthUnit::Metre), LengthUnit::Metre);
        assert_eq!(settings.fill_unit(LengthUnit::Metre), LengthUnit::Metre);
        assert_eq!(
            settings.style(LengthUnit::Metre).fill.unwrap().unit,
            LengthUnit::Metre
        );

        settings.set_unit(LengthUnit::Centimetre);
        settings.apply(StyleEdit::FillUnit(LengthUnit::Centimetre));

        assert_eq!(settings.unit(LengthUnit::Metre), LengthUnit::Centimetre);
        assert_eq!(
            settings.style(LengthUnit::Metre).fill.unwrap().unit,
            LengthUnit::Centimetre
        );
    }

    #[test]
    fn each_tool_starts_from_its_objects_default_style() {
        let unit = LengthUnit::Metre;

        let measurements = NewObjectSettings::for_measurements().style(unit);
        assert_eq!(measurements.line.style, LineStyle::Thick);
        assert_eq!(measurements.line.color, Rgb::BLUE);
        assert_eq!(measurements.fill, None);

        let areas = NewObjectSettings::for_areas().style(unit);
        assert_eq!(areas.line.style, LineStyle::Thick);
        assert_eq!(areas.fill.unwrap().style, FillStyle::Solid);

        let shapes = NewObjectSettings::for_shapes().style(unit);
        assert_eq!(shapes.line.style, LineStyle::Thin);
        assert_eq!(shapes.fill.unwrap().style, FillStyle::None);
    }

    #[test]
    fn edits_change_the_style_and_invalid_or_impossible_ones_do_not() {
        let mut settings = NewObjectSettings::for_measurements();
        settings.apply(StyleEdit::LineColor(Rgb::GREEN));
        settings.apply(StyleEdit::LineOpacity(2.0));
        settings.apply(StyleEdit::FillStyle(FillStyle::Solid));

        let style = settings.style(LengthUnit::Metre);
        assert_eq!(style.line.color, Rgb::GREEN);
        assert_eq!(style.line.opacity, 1.0);
        assert_eq!(style.fill, None);
    }
}
