//! Properties row for the active drawn shape: line style and opacity and, for a closed
//! shape, fill style, fill line distance and fill opacity.

use eframe::egui;
use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::object::ObjectId;
use gridtool::shape::{DrawnShape, FILL_SPACINGS, FillStyle, LineStyle};

use super::format::format_length;
use super::opacity_slider;
use super::selection::Selection;

const SPACING_WIDTH: f32 = 70.0;

/// Shows the row if the active object is a drawn shape, returning the action the user
/// completed, if any.
pub fn show(ui: &mut egui::Ui, document: &Document, selection: &mut Selection) -> Option<Action> {
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
                actions.extend(line_opacity(ui, id, index, stored, selection));
                if stored.is_closed() {
                    ui.separator();
                    actions.extend(fill_style_buttons(ui, index, stored));
                    if !stored.fill_style().hatch_directions().is_empty() {
                        actions.extend(fill_spacing_picker(ui, id, index, stored));
                    }
                    if stored.fill_style() != FillStyle::None {
                        actions.extend(fill_opacity(ui, id, index, stored, selection));
                    }
                }
                actions.into_iter().next()
            })
            .inner
        })
        .inner
}

/// "Shape <n> (line)" or "Shape <n> (area)".
pub fn label(index: usize, shape: &DrawnShape) -> String {
    let kind = if shape.is_closed() { "area" } else { "line" };
    format!("Shape {} ({kind})", index + 1)
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

/// A drop-down of the distances between fill lines. A distance from elsewhere (a
/// hand-edited file, say) is shown as it is, and stays until another is picked.
fn fill_spacing_picker(
    ui: &mut egui::Ui,
    id: ObjectId,
    index: usize,
    shape: &DrawnShape,
) -> Option<Action> {
    let current = shape.fill_spacing();
    let text = |spacing: f64| format!("{} px", format_length(spacing));
    let mut selected = current;
    egui::ComboBox::from_id_salt(("shape_fill_spacing", id))
        .selected_text(text(current))
        .width(SPACING_WIDTH)
        .show_ui(ui, |ui| {
            for spacing in FILL_SPACINGS {
                ui.selectable_value(&mut selected, spacing, text(spacing));
            }
        })
        .response
        .on_hover_text("Distance between the fill lines, in image pixels");
    (selected != current).then_some(Action::SetShapeFillSpacing {
        index,
        spacing: selected,
    })
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
    use eframe::egui::{Pos2, RawInput, Rect};
    use gridtool::geometry::Point;

    fn triangle() -> DrawnShape {
        DrawnShape::closed(vec![
            Point::new(1.0, 1.0),
            Point::new(20.0, 1.0),
            Point::new(10.0, 20.0),
        ])
        .unwrap()
    }

    #[test]
    fn labels_say_whether_the_shape_is_a_line_or_an_area() {
        let line = DrawnShape::open(vec![Point::default(), Point::new(1.0, 1.0)]).unwrap();

        assert_eq!(label(0, &line), "Shape 1 (line)");
        assert_eq!(label(2, &triangle()), "Shape 3 (area)");
    }

    /// Runs one frame of the row and reports the action and whether anything was shown.
    fn show_row(document: &Document, selection: &mut Selection) -> Option<Action> {
        let ctx = egui::Context::default();
        let input = RawInput {
            screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(1200.0, 800.0))),
            ..Default::default()
        };
        let mut action = None;
        let mut output = ctx.run_ui(input, |ui| {
            action = show(ui, document, selection);
        });
        // There is no renderer to upload textures to.
        output.textures_delta.clear();
        action
    }

    #[test]
    fn showing_the_row_for_any_fill_style_commits_nothing() {
        let mut document = Document::new(100, 100);
        Action::AddShape(triangle()).apply(&mut document).unwrap();
        let mut selection = Selection::default();
        selection.select(Some(document.shapes()[0].id()));

        for style in FillStyle::ALL {
            Action::SetShapeFillStyle { index: 0, style }
                .apply(&mut document)
                .unwrap();
            assert_eq!(show_row(&document, &mut selection), None);
        }
    }
}
