//! Controls for how something is drawn: line style, colour and opacity and, if it encloses
//! an area, fill style, colour, opacity and, for fill lines, their distance and angle. They
//! report edits rather than make them, so the same controls can edit an object in the
//! document or what a tool gives the next object it adds.

use eframe::egui;
use gridtool::color::Rgb;
use gridtool::style::{
    FillAngles, FillAppearance, FillStyle, LineAppearance, LineStyle, StyleEdit, Styled,
    normalized_fill_angle,
};

use super::color_swatch;
use super::commit_field::CommitField;
use super::format::{format_length, parse_number};
use super::icon_button::{icon_button, selectable_icon_button};
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

/// The colour of the line of what is being edited.
#[derive(Clone, Copy)]
struct LineColor {
    stored: Rgb,
    /// With any edit being previewed.
    displayed: Rgb,
}

/// What the fill's controls need to know about other things than the fill.
#[derive(Clone, Copy)]
struct FillContext {
    line_color: LineColor,
    /// The angle that each fill style changes to.
    angles: FillAngles,
}

#[derive(Default)]
pub struct StyleControls {
    fields: CommitField,
}

impl StyleControls {
    /// Shows the controls for `stored`, each at its value in `displayed` (`stored` with any
    /// edit being previewed). `id` tells these controls apart from others. `scale_problem`
    /// is why there is no scale, if there is none, and `angles` the fill angle that each fill
    /// style changes to.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        stored: &impl Styled,
        displayed: &impl Styled,
        scale_problem: Option<String>,
        angles: FillAngles,
    ) -> StyleEvents {
        let mut events = StyleEvents::default();
        self.line(ui, id, stored.line(), displayed.line(), &mut events);
        if let (Some(fill), Some(shown)) = (stored.fill(), displayed.fill()) {
            ui.separator();
            let context = FillContext {
                line_color: LineColor {
                    stored: stored.line().color,
                    displayed: displayed.line().color,
                },
                angles,
            };
            self.fill(ui, id, fill, shown, context, &mut events);
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
        context: FillContext,
        events: &mut StyleEvents,
    ) {
        ui.label("Fill:");
        events.commit(
            style_buttons(ui, &FillStyle::ALL, stored.style, FillStyle::name)
                .map(|style| fill_style_edit(style, stored.angle, context.angles)),
        );
        if stored.style.has_lines() {
            self.fill_spacing(ui, id, stored, events);
            self.fill_angle(ui, id, stored, events);
        }
        if stored.style != FillStyle::None {
            fill_color(
                ui,
                id.with("fill_color"),
                stored.color,
                context.line_color,
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

        let tooltip = "Angle of the fill lines, in degrees clockwise from vertical";
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

/// Changes to fill style `style`: its lines take the angle last set for it. A style with no
/// lines has no angle of its own, so the fill keeps the one it has, `current`.
fn fill_style_edit(style: FillStyle, current: f64, angles: FillAngles) -> StyleEdit {
    StyleEdit::FillStyle(style, angles.for_style(style).unwrap_or(current))
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

/// A labelled colour swatch.
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
    swatch(ui, id, tooltip, stored, edit, events);
}

/// The fill's colour: a padlock, selected while the fill has no colour of its own, and a
/// swatch that can only be used while it does.
fn fill_color(
    ui: &mut egui::Ui,
    id: egui::Id,
    stored: Option<Rgb>,
    line_color: LineColor,
    events: &mut StyleEvents,
) {
    ui.add(egui::Label::new("Fill color:").extend());
    let (icon, tooltip, swatch_tooltip) = match stored {
        None => (
            icons::LOCKED,
            "The fill uses the line's colour. Click to choose its own",
            "Click the padlock to choose the colour of the fill",
        ),
        Some(_) => (
            icons::UNLOCKED,
            "The fill has its own colour. Click to use the line's colour, forgetting this one",
            "Choose the colour of the fill",
        ),
    };
    if selectable_icon_button(ui, icon, stored.is_none(), tooltip) {
        events.commit(Some(padlock_edit(stored, line_color.stored)));
    }
    ui.add_enabled_ui(stored.is_some(), |ui| {
        swatch(
            ui,
            id,
            swatch_tooltip,
            stored.unwrap_or(line_color.displayed),
            |color| StyleEdit::FillColor(Some(color)),
            events,
        );
    });
}

/// What clicking the padlock does to a fill whose own colour is `stored`, on an object whose
/// line is `line_color`: it takes on the line's colour, or forgets its own.
fn padlock_edit(stored: Option<Rgb>, line_color: Rgb) -> StyleEdit {
    match stored {
        None => StyleEdit::FillColor(Some(line_color)),
        Some(_) => StyleEdit::FillColor(None),
    }
}

/// A colour swatch. The colour previews while the picker is open and is committed once when
/// it closes.
fn swatch(
    ui: &mut egui::Ui,
    id: egui::Id,
    tooltip: &str,
    stored: Rgb,
    edit: fn(Rgb) -> StyleEdit,
    events: &mut StyleEvents,
) {
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
    fn the_padlock_takes_on_the_line_colour_or_forgets_the_fills_own() {
        assert_eq!(
            padlock_edit(None, Rgb::CYAN),
            StyleEdit::FillColor(Some(Rgb::CYAN))
        );
        assert_eq!(
            padlock_edit(Some(Rgb::GREEN), Rgb::CYAN),
            StyleEdit::FillColor(None)
        );
    }

    /// Clicks along a row showing the fill colour controls, returning the edits they complete
    /// and whether a colour picker ever opened.
    fn click_along_fill_color(stored: Option<Rgb>) -> (Vec<StyleEdit>, bool) {
        const ROW_Y: f32 = 10.0;
        let line_color = LineColor {
            stored: Rgb::CYAN,
            displayed: Rgb::CYAN,
        };
        let ctx = egui::Context::default();
        egui_material_icons::initialize(&ctx);
        let mut edits = Vec::new();
        let mut picker_opened = false;
        for x in (0..200).step_by(4) {
            let pos = egui::Pos2::new(x as f32, ROW_Y);
            let button = |pressed| egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            };
            for events in [
                vec![egui::Event::PointerMoved(pos)],
                vec![button(true)],
                vec![button(false)],
                Vec::new(),
            ] {
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::Vec2::new(800.0, 600.0),
                    )),
                    events,
                    ..Default::default()
                };
                let mut output = ctx.run_ui(input, |ui| {
                    ui.horizontal(|ui| {
                        let mut events = StyleEvents::default();
                        fill_color(ui, ui.scope_id(), stored, line_color, &mut events);
                        edits.extend(events.commit);
                    });
                });
                output.textures_delta.clear();
                picker_opened |= egui::Popup::is_any_open(&ctx);
            }
        }
        (edits, picker_opened)
    }

    #[test]
    fn with_no_fill_colour_the_padlock_sets_the_line_colour_and_the_swatch_is_disabled() {
        let (edits, picker_opened) = click_along_fill_color(None);

        assert!(!edits.is_empty());
        assert!(
            edits
                .iter()
                .all(|&edit| edit == StyleEdit::FillColor(Some(Rgb::CYAN))),
            "{edits:?}"
        );
        assert!(!picker_opened);
    }

    #[test]
    fn with_a_fill_colour_the_padlock_forgets_it_and_the_swatch_opens_the_picker() {
        let (edits, picker_opened) = click_along_fill_color(Some(Rgb::GREEN));

        assert!(edits.contains(&StyleEdit::FillColor(None)), "{edits:?}");
        assert!(picker_opened);
    }

    #[test]
    fn a_new_fill_style_takes_the_angle_last_set_for_it_or_else_its_default() {
        let mut angles = FillAngles::default();
        assert_eq!(
            fill_style_edit(FillStyle::Crosshatch, 10.0, angles),
            StyleEdit::FillStyle(FillStyle::Crosshatch, 45.0)
        );
        assert_eq!(
            fill_style_edit(FillStyle::Lines, 10.0, angles),
            StyleEdit::FillStyle(FillStyle::Lines, 90.0)
        );

        angles.remember(FillStyle::Crosshatch, 30.0);
        assert_eq!(
            fill_style_edit(FillStyle::Crosshatch, 10.0, angles),
            StyleEdit::FillStyle(FillStyle::Crosshatch, 30.0)
        );
    }

    #[test]
    fn a_fill_style_without_lines_keeps_the_angle() {
        let angles = FillAngles::default();

        for style in [FillStyle::Solid, FillStyle::None] {
            assert_eq!(
                fill_style_edit(style, 10.0, angles),
                StyleEdit::FillStyle(style, 10.0)
            );
        }
    }

    #[test]
    fn rotating_fill_lines_stays_within_a_half_turn() {
        for by in [QUARTER_TURN, -QUARTER_TURN] {
            assert_eq!(rotated_fill(120.0, by), StyleEdit::FillAngle(30.0));
        }
    }
}
