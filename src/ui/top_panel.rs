use eframe::egui;

use super::icon_button::{icon_button, selectable_icon_button};
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

impl ToolBarState {
    /// The tool shown as active, given the `mode` last chosen. Every tool works on a document,
    /// so without one no tool is active.
    fn active_tool(self, mode: ToolMode) -> Option<ToolMode> {
        self.has_document.then_some(mode)
    }
}

/// What came of drawing the tool bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopPanel {
    /// The action the user triggered this frame, if any.
    pub action: Option<TopPanelAction>,
    /// How tall the tool bar's contents are, without the panel's margins.
    pub content_height: f32,
}

/// Draws the top tool bar, updating `mode` if the user picks a different tool.
pub fn show(ui: &mut egui::Ui, mode: &mut ToolMode, state: ToolBarState) -> TopPanel {
    egui::Panel::top("top_panel")
        .show(ui, |ui| {
            let row = ui.horizontal(|ui| {
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
                    if tool.starts_group() {
                        ui.add_space(SECTION_SPACING);
                    }
                    let selected = state.active_tool(*mode) == Some(tool);
                    let enabled = state.has_document;
                    if selectable_icon_button(ui, tool.icon(), selected, tool.name(), enabled) {
                        *mode = tool;
                    }
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
            });
            TopPanel {
                action: row.inner,
                content_height: row.response.rect.height(),
            }
        })
        .inner
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Rect};

    const WIDTH: f32 = 900.0;
    /// Inside the row of buttons.
    const ROW_Y: f32 = 16.0;

    /// Clicks along the tool bar, a few points apart, returning the modes it chose along the
    /// way and the actions it triggered, each once however many clicks landed on its button.
    fn click_along_the_tool_bar(state: ToolBarState) -> (Vec<ToolMode>, Vec<TopPanelAction>) {
        let ctx = egui::Context::default();
        egui_material_icons::initialize(&ctx);
        let (mut mode, mut modes, mut actions) = (ToolMode::Origin, Vec::new(), Vec::new());
        for x in (0..WIDTH as u32).step_by(6) {
            let pos = Pos2::new(x as f32, ROW_Y);
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
                Vec::new(),
            ] {
                let input = RawInput {
                    screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(WIDTH, 600.0))),
                    events,
                    ..Default::default()
                };
                let mut output = ctx.run_ui(input, |ui| {
                    if let Some(action) = show(ui, &mut mode, state).action
                        && actions.last() != Some(&action)
                    {
                        actions.push(action);
                    }
                });
                output.textures_delta.clear();
                if modes.last() != Some(&mode) {
                    modes.push(mode);
                }
            }
        }
        (modes, actions)
    }

    #[test]
    fn with_no_document_no_tool_is_active_and_only_opening_buttons_work() {
        let state = ToolBarState::default();
        assert_eq!(state.active_tool(ToolMode::Pan), None);

        let (modes, actions) = click_along_the_tool_bar(state);

        assert_eq!(modes, [ToolMode::Origin], "the mode cannot change");
        assert_eq!(
            actions,
            [
                TopPanelAction::New,
                TopPanelAction::Load,
                TopPanelAction::OpenImage
            ]
        );
    }

    #[test]
    fn undo_and_redo_work_when_the_state_says_they_can() {
        let state = ToolBarState {
            has_document: true,
            can_undo: true,
            can_redo: true,
            ..Default::default()
        };

        let (_, actions) = click_along_the_tool_bar(state);

        assert!(actions.contains(&TopPanelAction::Undo), "{actions:?}");
        assert!(actions.contains(&TopPanelAction::Redo), "{actions:?}");
    }

    #[test]
    fn with_a_document_the_chosen_tool_is_active_and_every_tool_can_be_chosen() {
        let state = ToolBarState {
            has_document: true,
            ..Default::default()
        };
        assert_eq!(state.active_tool(ToolMode::Grid), Some(ToolMode::Grid));

        let (modes, actions) = click_along_the_tool_bar(state);

        for tool in ToolMode::ALL {
            assert!(modes.contains(&tool), "{tool:?} in {modes:?}");
        }
        assert!(!actions.contains(&TopPanelAction::Undo));
        assert!(!actions.contains(&TopPanelAction::Redo));
    }
}
