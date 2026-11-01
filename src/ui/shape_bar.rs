//! Properties row for the active drawn shape: line style, colour and opacity and, for a
//! closed shape, fill style, colour, opacity and, for fill lines, their distance and angle.

use eframe::egui;
use gridtool::action::Action;
use gridtool::color::Rgb;
use gridtool::document::Document;
use gridtool::object::ObjectId;
use gridtool::shape::{DrawnShape, FillStyle, LineStyle, normalized_fill_angle};

use super::color_swatch;
use super::commit_field::CommitField;
use super::format::{format_length, parse_number};
use super::icon_button::icon_button;
use super::icons;
use super::opacity_slider;
use super::selection::Selection;
use super::unit_picker;

const SPACING_FIELD_WIDTH: f32 = 60.0;
const ANGLE_FIELD_WIDTH: f32 = 45.0;
const QUARTER_TURN: f64 = 90.0;

#[derive(Default)]
pub struct ShapeBar {
    fields: CommitField,
}

impl ShapeBar {
    /// Shows the row if the active object is a drawn shape, returning the action the user
    /// completed, if any.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        selection: &mut Selection,
    ) -> Option<Action> {
        let id = selection.active()?;
        let index = document.shape_index(id)?;
        let stored: &DrawnShape = &document.shapes()[index];

        egui::Panel::top("shape_bar")
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    // Every widget must be drawn each frame, so collect actions eagerly.
                    let mut actions = Vec::new();
                    ui.strong(label(index, stored));
                    ui.separator();
                    actions.extend(line_style_buttons(ui, index, stored));
                    actions.extend(line_color(ui, id, index, stored, selection));
                    actions.extend(line_opacity(ui, id, index, stored, selection));
                    if stored.is_closed() {
                        ui.separator();
                        actions.extend(fill_style_buttons(ui, index, stored));
                        if stored.fill_style().has_lines() {
                            actions.extend(self.fill_spacing(ui, id, index, stored));
                            actions.extend(self.fill_angle(ui, id, index, stored));
                        }
                        if stored.fill_style() != FillStyle::None {
                            actions.extend(fill_color(ui, id, index, stored, selection));
                            actions.extend(fill_opacity(ui, id, index, stored, selection));
                        }
                        if stored.fill_style().has_lines()
                            && let Err(reason) = document.calibration().scale()
                        {
                            ui.separator();
                            ui.weak(format!(
                                "Fill lines not shown: they need a scale ({reason})"
                            ));
                        }
                    }
                    actions.into_iter().next()
                })
                .inner
            })
            .inner
    }

    /// The distance between the fill lines: a field and a drop-down of units, as for a
    /// grid's cell size.
    fn fill_spacing(
        &mut self,
        ui: &mut egui::Ui,
        id: ObjectId,
        index: usize,
        shape: &DrawnShape,
    ) -> Option<Action> {
        let tooltip = "Distance between the fill lines";
        let label = ui.label("Distance:").on_hover_text(tooltip);
        let field_id = egui::Id::unique(("shape_fill_spacing", id));
        let current = shape.fill_spacing();
        let (field, committed) =
            self.fields
                .show(ui, field_id, format_length(current), SPACING_FIELD_WIDTH);
        field.labelled_by(label.id).on_hover_text(tooltip);
        let mut actions = Vec::new();
        actions.extend(
            typed_spacing(committed.as_deref(), current)
                .map(|spacing| Action::SetShapeFillSpacing { index, spacing }),
        );
        actions.extend(
            unit_picker::show(
                ui,
                ("shape_fill_unit", id),
                shape.fill_unit(),
                "Units of the distance between the fill lines",
            )
            .map(|unit| Action::SetShapeFillUnit { index, unit }),
        );
        actions.into_iter().next()
    }

    /// The direction of the fill lines: buttons that turn them a quarter turn, and a field
    /// for any other angle.
    fn fill_angle(
        &mut self,
        ui: &mut egui::Ui,
        id: ObjectId,
        index: usize,
        shape: &DrawnShape,
    ) -> Option<Action> {
        let mut actions = Vec::new();
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
            actions.push(rotate_fill(index, shape, -QUARTER_TURN));
        }
        if right {
            actions.push(rotate_fill(index, shape, QUARTER_TURN));
        }

        let tooltip = "Angle of the fill lines, in degrees clockwise from horizontal";
        let label = ui.label("Angle:").on_hover_text(tooltip);
        let field_id = egui::Id::unique(("shape_fill_angle", id));
        let current = shape.fill_angle();
        let (field, committed) =
            self.fields
                .show(ui, field_id, format_length(current), ANGLE_FIELD_WIDTH);
        field.labelled_by(label.id).on_hover_text(tooltip);
        ui.label("°");
        actions.extend(
            typed_angle(committed.as_deref(), current)
                .map(|degrees| Action::SetShapeFillAngle { index, degrees }),
        );
        actions.into_iter().next()
    }
}

/// "Shape <n> (line)" or "Shape <n> (area)".
pub fn label(index: usize, shape: &DrawnShape) -> String {
    let kind = if shape.is_closed() { "area" } else { "line" };
    format!("Shape {} ({kind})", index + 1)
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

fn rotate_fill(index: usize, shape: &DrawnShape, by: f64) -> Action {
    Action::SetShapeFillAngle {
        index,
        degrees: normalized_fill_angle(shape.fill_angle() + by),
    }
}

fn line_style_buttons(ui: &mut egui::Ui, index: usize, shape: &DrawnShape) -> Option<Action> {
    ui.label("Line:");
    let current = shape.line_style();
    style_buttons(ui, &LineStyle::ALL, current, LineStyle::name)
        .map(|style| Action::SetShapeLineStyle { index, style })
}

fn fill_style_buttons(ui: &mut egui::Ui, index: usize, shape: &DrawnShape) -> Option<Action> {
    ui.label("Fill:");
    let current = shape.fill_style();
    style_buttons(ui, &FillStyle::ALL, current, FillStyle::name)
        .map(|style| Action::SetShapeFillStyle { index, style })
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

fn line_color(
    ui: &mut egui::Ui,
    id: ObjectId,
    index: usize,
    stored: &DrawnShape,
    selection: &mut Selection,
) -> Option<Action> {
    let edit = ColorEditor {
        label: "Line color:",
        salt: "shape_line_color",
        tooltip: "Choose the colour of this shape's line",
        current: stored.line_color(),
        preview: DrawnShape::with_line_color,
    };
    edit.show(ui, id, stored, selection)
        .map(|color| Action::SetShapeLineColor { index, color })
}

fn fill_color(
    ui: &mut egui::Ui,
    id: ObjectId,
    index: usize,
    stored: &DrawnShape,
    selection: &mut Selection,
) -> Option<Action> {
    let edit = ColorEditor {
        label: "Fill color:",
        salt: "shape_fill_color",
        tooltip: "Choose the colour of this shape's fill",
        current: stored.fill_color(),
        preview: DrawnShape::with_fill_color,
    };
    edit.show(ui, id, stored, selection)
        .map(|color| Action::SetShapeFillColor { index, color })
}

/// A labelled colour swatch for one of the shape's colours. The colour previews live on the
/// canvas while the picker is open and is committed once when it closes.
struct ColorEditor {
    label: &'static str,
    salt: &'static str,
    tooltip: &'static str,
    current: Rgb,
    preview: fn(&DrawnShape, Rgb) -> DrawnShape,
}

impl ColorEditor {
    fn show(
        self,
        ui: &mut egui::Ui,
        id: ObjectId,
        stored: &DrawnShape,
        selection: &mut Selection,
    ) -> Option<Rgb> {
        ui.add(egui::Label::new(self.label).extend());
        let edit = color_swatch::show(
            ui,
            egui::Id::unique((self.salt, id)),
            self.current,
            color_swatch::BAR_SWATCH_SIZE,
            self.tooltip,
        );
        if let Some(color) = edit.preview {
            let displayed = selection.displayed_drawn(id, stored);
            selection.preview_drawn((self.preview)(&displayed, color));
        }
        if edit.closed {
            selection.clear_drawn_preview();
        }
        edit.committed
    }
}

fn line_opacity(
    ui: &mut egui::Ui,
    id: ObjectId,
    index: usize,
    stored: &DrawnShape,
    selection: &mut Selection,
) -> Option<Action> {
    ui.add(egui::Label::new("Line opacity:").extend());
    let displayed = selection.displayed_drawn(id, stored);
    let edit = opacity_slider::show(ui, displayed.line_opacity());
    if let Some(opacity) = edit.preview {
        selection.preview_drawn(displayed.with_line_opacity(opacity));
    }
    if !edit.finished {
        return None;
    }
    let chosen = selection.displayed_drawn(id, stored).line_opacity();
    selection.clear_drawn_preview();
    (chosen != stored.line_opacity()).then_some(Action::SetShapeLineOpacity {
        index,
        opacity: chosen,
    })
}

fn fill_opacity(
    ui: &mut egui::Ui,
    id: ObjectId,
    index: usize,
    stored: &DrawnShape,
    selection: &mut Selection,
) -> Option<Action> {
    ui.add(egui::Label::new("Fill opacity:").extend());
    let displayed = selection.displayed_drawn(id, stored);
    let edit = opacity_slider::show(ui, displayed.fill_opacity());
    if let Some(opacity) = edit.preview {
        selection.preview_drawn(displayed.with_fill_opacity(opacity));
    }
    if !edit.finished {
        return None;
    }
    let chosen = selection.displayed_drawn(id, stored).fill_opacity();
    selection.clear_drawn_preview();
    (chosen != stored.fill_opacity()).then_some(Action::SetShapeFillOpacity {
        index,
        opacity: chosen,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect};
    use gridtool::geometry::Point;
    use gridtool::units::LengthUnit;

    const WIDE_SCREEN: Rect = Rect::from_min_max(Pos2::ZERO, Pos2::new(1800.0, 800.0));

    fn triangle() -> DrawnShape {
        DrawnShape::closed(
            vec![
                Point::new(1.0, 1.0),
                Point::new(20.0, 1.0),
                Point::new(10.0, 20.0),
            ],
            LengthUnit::Centimetre,
        )
        .unwrap()
    }

    #[test]
    fn labels_say_whether_the_shape_is_a_line_or_an_area() {
        let line = DrawnShape::open(
            vec![Point::default(), Point::new(1.0, 1.0)],
            LengthUnit::Centimetre,
        )
        .unwrap();

        assert_eq!(label(0, &line), "Shape 1 (line)");
        assert_eq!(label(2, &triangle()), "Shape 3 (area)");
    }

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

    fn document_with_filled_triangle(style: FillStyle) -> (Document, Selection) {
        let mut document = Document::new(100, 100);
        Action::AddShape(triangle()).apply(&mut document).unwrap();
        Action::SetShapeFillStyle { index: 0, style }
            .apply(&mut document)
            .unwrap();
        let mut selection = Selection::default();
        selection.select(Some(document.shapes()[0].id()));
        (document, selection)
    }

    /// Runs one frame of the row with `events`, returning the action it completed.
    fn run_frame(
        ctx: &egui::Context,
        bar: &mut ShapeBar,
        document: &Document,
        selection: &mut Selection,
        events: Vec<Event>,
    ) -> Option<Action> {
        egui_material_icons::initialize(ctx);
        let input = RawInput {
            screen_rect: Some(WIDE_SCREEN),
            events,
            ..Default::default()
        };
        let mut action = None;
        let mut output = ctx.run_ui(input, |ui| {
            action = bar.show(ui, document, selection);
        });
        // There is no renderer to upload textures to.
        output.textures_delta.clear();
        action
    }

    #[test]
    fn showing_the_row_for_any_fill_style_commits_nothing() {
        for style in FillStyle::ALL {
            let (document, mut selection) = document_with_filled_triangle(style);
            let ctx = egui::Context::default();

            let action = run_frame(
                &ctx,
                &mut ShapeBar::default(),
                &document,
                &mut selection,
                Vec::new(),
            );

            assert_eq!(action, None);
        }
    }

    /// Clicks along the row, a few points apart, collecting what each click completes.
    fn actions_from_clicking_along_the_row(style: FillStyle) -> Vec<Action> {
        let (document, mut selection) = document_with_filled_triangle(style);
        let ctx = egui::Context::default();
        let mut bar = ShapeBar::default();
        let mut actions = Vec::new();
        for x in (0..WIDE_SCREEN.max.x as u32).step_by(6) {
            let pos = Pos2::new(x as f32, 14.0);
            let button = |pressed| Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            };
            for events in [
                vec![Event::PointerMoved(pos)],
                vec![button(true)],
                vec![button(false)],
                Vec::new(),
            ] {
                actions.extend(run_frame(&ctx, &mut bar, &document, &mut selection, events));
            }
        }
        actions
    }

    #[test]
    fn the_rotate_buttons_turn_the_fill_lines_a_quarter_turn() {
        let actions = actions_from_clicking_along_the_row(FillStyle::Lines);

        let rotate = Action::SetShapeFillAngle {
            index: 0,
            degrees: 90.0,
        };
        let rotations = actions.iter().filter(|&action| *action == rotate);
        assert!(rotations.count() >= 2, "{actions:?}");
    }

    #[test]
    fn the_rotate_buttons_are_also_there_for_crosshatch_but_not_a_solid_fill() {
        let rotates = |style| {
            actions_from_clicking_along_the_row(style)
                .iter()
                .any(|action| matches!(action, Action::SetShapeFillAngle { .. }))
        };

        assert!(rotates(FillStyle::Crosshatch));
        assert!(!rotates(FillStyle::Solid));
    }

    #[test]
    fn rotating_fill_lines_stays_within_a_half_turn() {
        let mut shape = triangle();
        let mut document = Document::new(100, 100);
        Action::AddShape(shape.clone())
            .apply(&mut document)
            .unwrap();
        Action::SetShapeFillAngle {
            index: 0,
            degrees: 120.0,
        }
        .apply(&mut document)
        .unwrap();
        shape = (*document.shapes()[0]).clone();

        for by in [QUARTER_TURN, -QUARTER_TURN] {
            assert_eq!(
                rotate_fill(0, &shape, by),
                Action::SetShapeFillAngle {
                    index: 0,
                    degrees: 30.0,
                }
            );
        }
    }
}
