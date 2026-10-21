use eframe::egui;
use gridtool::editor::Editor;

const DEFAULT_WIDTH: f32 = 240.0;

/// Right-hand panel listing every recorded action. The row for the current position in
/// the undo/redo stack is highlighted; undone actions are dimmed.
pub fn show(ui: &mut egui::Ui, editor: Option<&Editor>) {
    egui::Panel::right("history_panel")
        .resizable(true)
        .default_size(DEFAULT_WIDTH)
        .show(ui, |ui| {
            ui.heading("History");
            ui.separator();
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                match editor {
                    Some(editor) => history_rows(ui, editor),
                    None => {
                        ui.weak("No document open");
                    }
                }
            });
        });
}

fn history_rows(ui: &mut egui::Ui, editor: &Editor) {
    let applied = editor.applied_count();
    history_row(ui, "Opened image".to_owned(), applied == 0, false);
    for (index, action) in editor.actions().enumerate() {
        let position = index + 1;
        history_row(
            ui,
            action.to_string(),
            applied == position,
            position > applied,
        );
    }
}

fn history_row(ui: &mut egui::Ui, text: String, is_current: bool, is_undone: bool) {
    let text = egui::RichText::new(text);
    let text = if is_undone {
        text.weak().italics()
    } else {
        text
    };
    // Rows are display-only; clicking does not jump in history.
    let _ = ui.selectable_label(is_current, text);
}
