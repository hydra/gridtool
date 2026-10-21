use eframe::egui;

use super::icon_button::{icon_button, icon_text};
use super::icons;
use super::tool_mode::ToolMode;

/// Gap between groups of tool bar buttons.
const SECTION_SPACING: f32 = 16.0;

/// Something the user asked for via the top panel; the app decides how to carry it out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopPanelAction {
    New,
    Load,
    Save,
    SaveAs,
    OpenImage,
    Undo,
    Redo,
}

/// Which tool bar buttons should be enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToolBarState {
    /// A document is open.
    pub has_document: bool,
    /// The open document has unsaved changes.
    pub has_unsaved_changes: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

/// Draws the top tool bar, updating `mode` if the user picks a different tool, and
/// returns the action the user triggered this frame, if any.
pub fn show(ui: &mut egui::Ui, mode: &mut ToolMode, state: ToolBarState) -> Option<TopPanelAction> {
    egui::Panel::top("top_panel")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut action = None;
                let mut button = |ui: &mut egui::Ui, icon, tooltip, enabled, clicked| {
                    if icon_button(ui, icon, tooltip, enabled) {
                        action = Some(clicked);
                    }
                };
                button(ui, icons::NEW, "New", true, TopPanelAction::New);
                button(ui, icons::LOAD, "Load…", true, TopPanelAction::Load);
                button(
                    ui,
                    icons::SAVE,
                    "Save",
                    state.has_document && state.has_unsaved_changes,
                    TopPanelAction::Save,
                );
                button(
                    ui,
                    icons::SAVE_AS,
                    "Save as…",
                    state.has_document,
                    TopPanelAction::SaveAs,
                );

                ui.add_space(SECTION_SPACING);
                button(
                    ui,
                    icons::OPEN_IMAGE,
                    "Open image…",
                    true,
                    TopPanelAction::OpenImage,
                );

                ui.add_space(SECTION_SPACING);
                for tool in ToolMode::ALL {
                    ui.selectable_value(mode, tool, icon_text(tool.icon()))
                        .on_hover_text(tool.name());
                }

                ui.add_space(SECTION_SPACING);
                button(
                    ui,
                    icons::UNDO,
                    "Undo",
                    state.can_undo,
                    TopPanelAction::Undo,
                );
                button(
                    ui,
                    icons::REDO,
                    "Redo",
                    state.can_redo,
                    TopPanelAction::Redo,
                );
                action
            })
            .inner
        })
        .inner
}
