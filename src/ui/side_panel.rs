//! The right-hand panel: measurements on top, history below, with a draggable divider.

use eframe::egui;
use gridtool::action::Action;
use gridtool::editor::Editor;

use super::history_panel;
use super::measure_tool::MeasureTool;
use super::measurements_panel::MeasurementsPanel;

const DEFAULT_WIDTH: f32 = 460.0;
const DEFAULT_MEASUREMENTS_HEIGHT: f32 = 280.0;

/// Shows the side panel, returning the document action the user completed, if any.
/// `open` is the open document's editor and measure tool, if a document is open.
pub fn show(
    ui: &mut egui::Ui,
    measurements: &mut MeasurementsPanel,
    open: Option<(&Editor, &MeasureTool)>,
) -> Option<Action> {
    egui::Panel::right("side_panel")
        .resizable(true)
        .default_size(DEFAULT_WIDTH)
        .show(ui, |ui| {
            let action = egui::Panel::top("measurements_panel")
                .resizable(true)
                .default_size(DEFAULT_MEASUREMENTS_HEIGHT)
                .show(ui, |ui| match open {
                    Some((editor, tool)) => measurements.show(ui, editor.document(), tool),
                    None => {
                        ui.heading("Measurements");
                        ui.separator();
                        ui.weak("No document open");
                        None
                    }
                })
                .inner;
            history_panel::show(ui, open.map(|(editor, _)| editor));
            action
        })
        .inner
}
