//! Icon buttons shared by the tool bars.

use eframe::egui;
use egui_material_icons::MaterialIcon;

const ICON_SIZE: f32 = 18.0;
/// Icons inside table rows, sized to match the text around them.
const INLINE_ICON_SIZE: f32 = 14.0;

/// An icon button with a tooltip (shown even while disabled); returns whether it was
/// clicked.
pub fn icon_button(ui: &mut egui::Ui, icon: MaterialIcon, tooltip: &str, enabled: bool) -> bool {
    ui.add_enabled(enabled, egui::Button::new(icon_text(icon)))
        .on_hover_text(tooltip)
        .on_disabled_hover_text(tooltip)
        .clicked()
}

/// `icon` at the tool bar icon size.
pub fn icon_text(icon: MaterialIcon) -> egui::RichText {
    icon.rich_text().size(ICON_SIZE)
}

/// `icon` at the size of the surrounding text.
pub fn inline_icon_text(icon: MaterialIcon) -> egui::RichText {
    icon.rich_text().size(INLINE_ICON_SIZE)
}
