use eframe::egui;

use super::icon_button::{icon_button, icon_text};
use super::icons::{OPEN_IMAGE as OPEN_IMAGE_ICON, REDO as REDO_ICON, UNDO as UNDO_ICON};
use super::tool_mode::ToolMode;
/// Gap between groups of tool bar buttons.
const SECTION_SPACING: f32 = 16.0;

/// Something the user asked for via the top panel; the app decides how to carry it out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopPanelAction {
    OpenImage,
    Undo,
    Redo,
}

/// Whether the undo and redo buttons should be enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HistoryAvailability {
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Draws the top tool bar, updating `mode` if the user picks a different tool, and
/// returns the action the user triggered this frame, if any.
pub fn show(
    ui: &mut egui::Ui,
    mode: &mut ToolMode,
    history: HistoryAvailability,
) -> Option<TopPanelAction> {
    egui::Panel::top("top_panel")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut action = None;
                if icon_button(ui, OPEN_IMAGE_ICON, "Open image…", true) {
                    action = Some(TopPanelAction::OpenImage);
                }

                ui.add_space(SECTION_SPACING);
                for tool in ToolMode::ALL {
                    ui.selectable_value(mode, tool, icon_text(tool.icon()))
                        .on_hover_text(tool.name());
                }

                ui.add_space(SECTION_SPACING);
                if icon_button(ui, UNDO_ICON, "Undo", history.can_undo) {
                    action = Some(TopPanelAction::Undo);
                }
                if icon_button(ui, REDO_ICON, "Redo", history.can_redo) {
                    action = Some(TopPanelAction::Redo);
                }
                action
            })
            .inner
        })
        .inner
}
