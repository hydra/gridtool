//! The Areas table: the document's area measurements, in document order, with their
//! total area, per-area units and colours, and delete buttons. The active area's row is
//! selected, and clicking a row makes its area active.

use eframe::egui;
use gridtool::action::{Action, StyleTarget};
use gridtool::area::Area;
use gridtool::document::Document;
use gridtool::style::{StyleEdit, Styled};

use super::color_swatch;
use super::format::format_length;
use super::icon_button::inline_icon_text;
use super::icons;
use super::selection::Selection;
use super::table_rows::{TableOutput, TableRows, row_color};
use super::unit_picker;

/// The panel's title.
pub const HEADING: &str = "Areas";
const COLUMN_HEADERS: [&str; 5] = ["#", "Color", "Area", "Units", ""];

/// Draws the table into `ui`, returning what the user did. The row of the active object is
/// selected and, if `reveal`, scrolled into view.
pub fn show(
    ui: &mut egui::Ui,
    document: &Document,
    selection: &Selection,
    reveal: bool,
) -> TableOutput {
    ui.heading(HEADING);
    ui.separator();
    if document.areas().is_empty() {
        ui.weak("No areas yet. Use the Measure area tool to add one.");
        return TableOutput::default();
    }

    let active = selection.active();
    let selected_row = active.and_then(|id| document.area_index(id));
    egui::ScrollArea::both()
        .id_salt("areas_scroll")
        .auto_shrink(false)
        .show(ui, |ui| {
            let mut rows = TableRows::default();
            let table = egui::Grid::new("areas_table")
                .with_row_color(row_color(selected_row))
                .num_columns(COLUMN_HEADERS.len())
                .show(ui, |ui| {
                    for header in COLUMN_HEADERS {
                        ui.strong(header);
                    }
                    ui.end_row();

                    let mut action = None;
                    for (index, stored) in document.areas().iter().enumerate() {
                        // Show a drag in progress live.
                        let area = selection.displayed_area(stored.id(), stored);
                        let row_action =
                            rows.row(ui, stored.id(), |ui| row(ui, document, index, &area));
                        action = action.or(row_action);
                    }
                    action
                });
            let pick = rows.finish(ui, table.response.rect, active.filter(|_| reveal));
            TableOutput::new(table.inner, pick)
        })
        .inner
}

fn row(ui: &mut egui::Ui, document: &Document, index: usize, area: &Area) -> Option<Action> {
    // Every cell must be drawn each frame, so collect actions eagerly.
    let mut actions = Vec::new();

    ui.label((index + 1).to_string());

    actions.extend(
        color_swatch::show(
            ui,
            egui::Id::unique(("area_color", index)),
            area.line().color,
            color_swatch::ROW_SWATCH_SIZE,
            "Choose the line colour of this area",
        )
        .committed
        .map(|color| Action::SetStyle {
            target: StyleTarget::Area(index),
            edit: StyleEdit::LineColor(color),
        }),
    );

    match area.area(document.calibration()) {
        Ok(value) => {
            ui.label(format_length(value));
        }
        Err(reason) => {
            ui.weak("—").on_hover_text(format!("Scale: {reason}"));
        }
    }

    actions.extend(
        unit_picker::show_area(ui, ("area_unit", index), area.unit(), "Units of this area")
            .map(|unit| Action::SetAreaUnit { index, unit }),
    );

    if ui
        .button(inline_icon_text(icons::DELETE))
        .on_hover_text("Delete area")
        .clicked()
    {
        actions.push(Action::DeleteArea { index });
    }

    actions.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, FontId, Modifiers, PointerButton, Pos2, RawInput, Rect};
    use gridtool::geometry::Point;
    use gridtool::units::LengthUnit;

    fn document_with_areas(count: usize) -> Document {
        let mut document = Document::new(100, 100);
        for _ in 0..count {
            let points = [(1.0, 1.0), (50.0, 1.0), (20.0, 40.0)].map(|(x, y)| Point::new(x, y));
            Action::AddArea(Area::new(points.to_vec(), LengthUnit::Metre).unwrap())
                .apply(&mut document)
                .unwrap();
        }
        document
    }

    /// Clicks down the first column, returning the objects whose rows were picked, in order.
    fn picks_from_clicking_down_the_table(document: &Document) -> Vec<gridtool::object::ObjectId> {
        let ctx = egui::Context::default();
        let selection = Selection::default();
        let mut picks = Vec::new();
        let mut frame = |events: Vec<Event>| {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(500.0, 400.0))),
                events,
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                picks.extend(show(ui, document, &selection, false).pick);
            });
            output.textures_delta.clear();
        };
        frame(Vec::new());
        for y in (0..200).step_by(4) {
            let pos = Pos2::new(12.0, y as f32);
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
            ] {
                frame(events);
            }
        }
        picks
    }

    #[test]
    fn clicking_a_row_picks_its_area_and_the_header_picks_nothing() {
        let document = document_with_areas(2);
        let ids: Vec<_> = document.areas().iter().map(|area| area.id()).collect();

        let mut picks = picks_from_clicking_down_the_table(&document);
        picks.dedup();

        assert_eq!(picks, ids);
    }

    #[test]
    fn the_colour_is_in_the_second_column() {
        assert_eq!(COLUMN_HEADERS[1], "Color");
    }

    #[test]
    fn area_unit_symbols_are_in_the_default_font() {
        let ctx = egui::Context::default();
        // Fonts are built at the start of a frame.
        let mut output = ctx.run_ui(RawInput::default(), |_| {});
        output.textures_delta.clear();

        for unit in LengthUnit::ALL {
            let symbol = unit.area_symbol();
            assert!(
                ctx.fonts_mut(|fonts| fonts.has_glyphs(&FontId::proportional(14.0), symbol)),
                "{symbol} has no glyph"
            );
        }
    }
}
