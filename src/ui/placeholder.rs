use eframe::egui;

/// Fills the available space with a centered message, e.g. when there is nothing to show.
pub fn show(ui: &mut egui::Ui, text: impl Into<egui::WidgetText>) {
    ui.centered_and_justified(|ui| ui.label(text));
}
