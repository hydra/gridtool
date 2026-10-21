use eframe::egui;

use crate::tool_mode::ToolMode;

const OPEN_IMAGE_ICON: &str = "🗁";
const ICON_SIZE: f32 = 18.0;
/// Gap between the file actions and the tool mode buttons.
const SECTION_SPACING: f32 = 16.0;

/// Something the user asked for via the top panel; the app decides how to carry it out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopPanelAction {
    OpenImage,
}

/// Draws the top tool bar, updating `mode` if the user picks a different tool, and
/// returns the action the user triggered this frame, if any.
pub fn show(ui: &mut egui::Ui, mode: &mut ToolMode) -> Option<TopPanelAction> {
    egui::Panel::top("top_panel")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let open = ui
                    .button(icon_text(OPEN_IMAGE_ICON))
                    .on_hover_text("Open image…");

                ui.add_space(SECTION_SPACING);

                for tool in ToolMode::ALL {
                    ui.selectable_value(mode, tool, icon_text(tool.icon()))
                        .on_hover_text(tool.name());
                }

                open.clicked().then_some(TopPanelAction::OpenImage)
            })
            .inner
        })
        .inner
}

fn icon_text(icon: &str) -> egui::RichText {
    egui::RichText::new(icon).size(ICON_SIZE)
}
