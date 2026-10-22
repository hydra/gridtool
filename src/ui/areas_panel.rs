//! The Areas table: the document's area measurements, in document order, with their
//! total area, per-area units and colours, and delete buttons.

use eframe::egui;
use gridtool::action::Action;
use gridtool::area::Area;
use gridtool::document::Document;

use super::color_swatch;
use super::format::format_length;
use super::icon_button::inline_icon_text;
use super::icons;
use super::selection::Selection;
use super::unit_picker;

/// The panel's title.
pub const HEADING: &str = "Areas";
const COLUMN_HEADERS: [&str; 5] = ["#", "Area", "Units", "Color", ""];

/// Draws the table into `ui`, returning the action the user completed, if any.
pub fn show(ui: &mut egui::Ui, document: &Document, selection: &Selection) -> Option<Action> {
    ui.heading(HEADING);
    ui.separator();
    if document.areas().is_empty() {
        ui.weak("No areas yet. Use the Measure area tool to add one.");
        return None;
    }

    egui::ScrollArea::both()
        .id_salt("areas_scroll")
        .auto_shrink(false)
        .show(ui, |ui| {
            egui::Grid::new("areas_table")
                .striped(true)
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
                        let row_action = row(ui, document, index, &area);
                        action = action.or(row_action);
                        ui.end_row();
                    }
                    action
                })
                .inner
        })
        .inner
}

fn row(ui: &mut egui::Ui, document: &Document, index: usize, area: &Area) -> Option<Action> {
    // Every cell must be drawn each frame, so collect actions eagerly.
    let mut actions = Vec::new();

    ui.label((index + 1).to_string());

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

    actions.extend(
        color_swatch::show(
            ui,
            egui::Id::unique(("area_color", index)),
            area.color(),
            color_swatch::ROW_SWATCH_SIZE,
            "Choose the colour of this area",
        )
        .committed
        .map(|color| Action::SetAreaColor { index, color }),
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
    use eframe::egui::{self, FontId, RawInput};
    use gridtool::units::LengthUnit;

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
