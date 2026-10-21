//! Icon buttons shared by the tool bars.

use eframe::egui;

const ICON_SIZE: f32 = 18.0;

/// An icon button with a tooltip (shown even while disabled); returns whether it was
/// clicked.
pub fn icon_button(ui: &mut egui::Ui, icon: &str, tooltip: &str, enabled: bool) -> bool {
    ui.add_enabled(enabled, egui::Button::new(icon_text(icon)))
        .on_hover_text(tooltip)
        .on_disabled_hover_text(tooltip)
        .clicked()
}

/// `icon` at the tool bar icon size.
pub fn icon_text(icon: &str) -> egui::RichText {
    egui::RichText::new(icon).size(ICON_SIZE)
}
