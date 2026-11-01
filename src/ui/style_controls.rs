//! Controls for how something is drawn: line style, colour and opacity and, if it encloses
//! an area, fill style, colour, opacity and, for fill lines, their distance and angle. They
//! report edits rather than make them, so the same controls can edit an object in the
//! document or what a tool gives the next object it adds.

use eframe::egui;
use gridtool::color::Rgb;
use gridtool::style::{
    FillAppearance, FillStyle, LineAppearance, LineStyle, StyleEdit, Styled, normalized_fill_angle,
};

use super::color_swatch;
use super::commit_field::CommitField;
use super::format::{format_length, parse_number};
use super::icon_button::icon_button;
use super::icons;
use super::opacity_slider;
use super::unit_picker;

const SPACING_FIELD_WIDTH: f32 = 60.0;
const ANGLE_FIELD_WIDTH: f32 = 45.0;
const QUARTER_TURN: f64 = 90.0;

/// What the user did with the controls this frame.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct StyleEvents {
    /// An edit still being chosen, to show live.
    pub preview: Option<StyleEdit>,
    /// The user stopped choosing, so any edit being shown live should go.
    pub clear_preview: bool,
    /// An edit the user completed.
    pub commit: Option<StyleEdit>,
}

impl StyleEvents {
    fn commit(&mut self, edit: Option<StyleEdit>) {
        self.commit = self.commit.or(edit);
    }
}

#[derive(Default)]
pub struct StyleControls {
    fields: CommitField,
}

impl StyleControls {
    /// Shows the controls for `stored`, each at its value in `displayed` (`stored` with any
    /// edit being previewed). `id` tells these controls apart from others. `scale_problem`
    /// is why there is no scale, if there is none.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        stored: &impl Styled,
        displayed: &impl Styled,
        scale_problem: Option<String>,
    ) -> StyleEvents {
        let mut events = StyleEvents::default();
        self.line(ui, id, stored.line(), displayed.line(), &mut events);
        if let (Some(fill), Some(shown)) = (stored.fill(), displayed.fill()) {
            ui.separator();
            self.fill(ui, id, fill, shown, &mut events);
            if fill.style.has_lines()
                && let Some(reason) = scale_problem
            {
                ui.separator();
                ui.weak(format!(
                    "Fill lines not shown: they need a scale ({reason})"
                ));
            }
        }
        events
    }

    fn line(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        stored: LineAppearance,
        displayed: LineAppearance,
        events: &mut StyleEvents,
    ) {
        ui.label("Line:");
        events.commit(
            style_buttons(ui, &LineStyle::ALL, stored.style, LineStyle::name)
                .map(StyleEdit::LineStyle),
        );
        color(
            ui,
            id.with("line_color"),
            "Line color:",
            "Choose the colour of the line",
            stored.color,
            StyleEdit::LineColor,
            events,
        );
        opacity(
            ui,
            "Line opacity:",
            stored.opacity,
            displayed.opacity,
            StyleEdit::LineOpacity,
            events,
        );
    }

    fn fill(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        stored: FillAppearance,
        displayed: FillAppearance,
        events: &mut StyleEvents,
    ) {
        ui.label("Fill:");
        events.commit(
            style_buttons(ui, &FillStyle::ALL, stored.style, FillStyle::name)
                .map(StyleEdit::FillStyle),
        );
        if stored.style.has_lines() {
            self.fill_spacing(ui, id, stored, events);
            self.fill_angle(ui, id, stored, events);
        }
        if stored.style != FillStyle::None {
            color(
                ui,
                id.with("fill_color"),
                "Fill color:",
                "Choose the colour of the fill",
                stored.color,
                StyleEdit::FillColor,
                events,
            );
            opacity(
                ui,
                "Fill opacity:",
                stored.opacity,
                displayed.opacity,
                StyleEdit::FillOpacity,
                events,
            );
        }
    }

    /// The distance between the fill lines: a field and a drop-down of units, as for a
    /// grid's cell size.
    fn fill_spacing(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        fill: FillAppearance,
        events: &mut StyleEvents,
    ) {
        let tooltip = "Distance between the fill lines";
        let label = ui.label("Distance:").on_hover_text(tooltip);
        let (field, committed) = self.fields.show(
            ui,
            id.with("fill_spacing"),
            format_length(fill.spacing),
            SPACING_FIELD_WIDTH,
        );
        field.labelled_by(label.id).on_hover_text(tooltip);
        events
            .commit(typed_spacing(committed.as_deref(), fill.spacing).map(StyleEdit::FillSpacing));
        events.commit(
            unit_picker::show(
                ui,
                id.with("fill_unit"),
                fill.unit,
                "Units of the distance between the fill lines",
            )
            .map(StyleEdit::FillUnit),
        );
    }

    /// The direction of the fill lines: buttons that turn them a quarter turn, and a field
    /// for any other angle.
    fn fill_angle(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        fill: FillAppearance,
        events: &mut StyleEvents,
    ) {
        let left = icon_button(
            ui,
            icons::ROTATE_90_LEFT,
            "Rotate the fill lines 90° left",
            true,
        );
        let right = icon_button(
            ui,
            icons::ROTATE_90_RIGHT,
            "Rotate the fill lines 90° right",
            true,
        );
        if left {
            events.commit(Some(rotated_fill(fill.angle, -QUARTER_TURN)));
        }
        if right {
            events.commit(Some(rotated_fill(fill.angle, QUARTER_TURN)));
        }

        let tooltip = "Angle of the fill lines, in degrees clockwise from horizontal";
        let label = ui.label("Angle:").on_hover_text(tooltip);
        let (field, committed) = self.fields.show(
            ui,
            id.with("fill_angle"),
            format_length(fill.angle),
            ANGLE_FIELD_WIDTH,
        );
        field.labelled_by(label.id).on_hover_text(tooltip);
        ui.label("°");
        events.commit(typed_angle(committed.as_deref(), fill.angle).map(StyleEdit::FillAngle));
    }
}

/// The spacing to set from `text` typed in the field: only a positive number that differs
/// from `current`.
fn typed_spacing(text: Option<&str>, current: f64) -> Option<f64> {
    text.and_then(parse_number)
        .filter(|&spacing| spacing > 0.0 && spacing != current)
}

/// The angle to set from `text` typed in the field, as the equivalent angle within a half
/// turn: only if that differs from `current`.
fn typed_angle(text: Option<&str>, current: f64) -> Option<f64> {
    text.and_then(parse_number)
        .map(normalized_fill_angle)
        .filter(|&degrees| degrees != current)
}

fn rotated_fill(current: f64, by: f64) -> StyleEdit {
    StyleEdit::FillAngle(normalized_fill_angle(current + by))
}

/// One button per style, with `current` shown selected. Returns the style the user picked,
/// if different.
fn style_buttons<S: Copy + PartialEq>(
    ui: &mut egui::Ui,
    styles: &[S],
    current: S,
    name: fn(S) -> &'static str,
) -> Option<S> {
    let mut picked = None;
    for &style in styles {
        if ui.selectable_label(style == current, name(style)).clicked() && style != current {
            picked = Some(style);
        }
    }
    picked
}

/// A labelled colour swatch. The colour previews while the picker is open and is committed
/// once when it closes.
fn color(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    tooltip: &str,
    stored: Rgb,
    edit: fn(Rgb) -> StyleEdit,
    events: &mut StyleEvents,
) {
    ui.add(egui::Label::new(label).extend());
    let swatch = color_swatch::show(ui, id, stored, color_swatch::BAR_SWATCH_SIZE, tooltip);
    if let Some(color) = swatch.preview {
        events.preview = Some(edit(color));
    }
    events.clear_preview |= swatch.closed;
    events.commit(swatch.committed.map(edit));
}

/// A labelled opacity slider, at the opacity as `displayed`. Changes preview, and the
/// opacity is committed once the user has finished choosing.
fn opacity(
    ui: &mut egui::Ui,
    label: &str,
    stored: f32,
    displayed: f32,
    edit: fn(f32) -> StyleEdit,
    events: &mut StyleEvents,
) {
    ui.add(egui::Label::new(label).extend());
    let slider = opacity_slider::show(ui, displayed);
    if let Some(opacity) = slider.preview {
        events.preview = Some(edit(opacity));
    }
    if slider.finished {
        events.clear_preview = true;
        let chosen = slider.preview.unwrap_or(displayed);
        events.commit((chosen != stored).then_some(edit(chosen)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_typed_spacing_must_be_a_new_positive_number() {
        assert_eq!(typed_spacing(Some(" 2.5 "), 10.0), Some(2.5));
        for text in ["0", "-1", "abc", "", "10"] {
            assert_eq!(typed_spacing(Some(text), 10.0), None, "{text:?}");
        }
        assert_eq!(typed_spacing(None, 10.0), None);
    }

    #[test]
    fn a_typed_angle_is_kept_within_a_half_turn() {
        assert_eq!(typed_angle(Some("45"), 0.0), Some(45.0));
        assert_eq!(typed_angle(Some("225"), 0.0), Some(45.0));
        assert_eq!(typed_angle(Some("-45"), 0.0), Some(135.0));
        // The same angle as the current one, or not a number.
        assert_eq!(typed_angle(Some("180"), 0.0), None);
        assert_eq!(typed_angle(Some("abc"), 0.0), None);
        assert_eq!(typed_angle(None, 0.0), None);
    }

    #[test]
    fn rotating_fill_lines_stays_within_a_half_turn() {
        for by in [QUARTER_TURN, -QUARTER_TURN] {
            assert_eq!(rotated_fill(120.0, by), StyleEdit::FillAngle(30.0));
        }
    }
}
