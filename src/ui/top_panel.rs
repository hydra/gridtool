use eframe::egui;

use super::icon_button::{icon_button, icon_text, selectable_icon_button};
use super::icons;
use super::tool_mode::ToolMode;
use super::workspace::Panel;

/// Gap between groups of tool bar buttons.
const SECTION_SPACING: f32 = 16.0;

/// Something the user asked for via the top panel; the app decides how to carry it out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopPanelAction {
    New,
    Load,
    Save,
    SaveAs,
    Undo,
    Redo,
    /// Close the panel if it is open, otherwise reopen it.
    TogglePanel(Panel),
    ResetLayout,
}

/// Which tool bar buttons should be enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToolBarState {
    /// A document is active.
    pub has_document: bool,
    /// The active document has unsaved changes.
    pub has_unsaved_changes: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    /// Which panels are open, by [`Panel::index`].
    pub open_panels: [bool; Panel::ALL.len()],
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
                button(ui, icons::NEW, "New…", true, TopPanelAction::New);
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

                ui.add_space(SECTION_SPACING);
                action.or(panels_menu(ui, state))
            });
            TopPanel {
                action: row.inner,
                content_height: row.response.rect.height(),
            }
        })
        .inner
}

/// The menu that closes and reopens panels, and puts them back where they started.
fn panels_menu(ui: &mut egui::Ui, state: ToolBarState) -> Option<TopPanelAction> {
    let menu = ui.menu_button(icon_text(icons::PANELS), |ui| {
        let mut action = None;
        for panel in Panel::ALL {
            let mut open = state.open_panels[panel.index()];
            if ui.checkbox(&mut open, panel.title()).clicked() {
                action = Some(TopPanelAction::TogglePanel(panel));
            }
        }
        ui.separator();
        if ui.button("Reset layout").clicked() {
            action = Some(TopPanelAction::ResetLayout);
        }
        action
    });
    menu.response.on_hover_text("Panels");
    menu.inner.flatten()
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
        assert_eq!(actions, [TopPanelAction::New, TopPanelAction::Load]);
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
    fn the_panels_menu_toggles_panels_and_resets_the_layout() {
        let ctx = egui::Context::default();
        egui_material_icons::initialize(&ctx);
        let state = ToolBarState {
            open_panels: [true, false, true, true],
            ..Default::default()
        };
        let mut chosen = None;
        let mut frame = |events: Vec<Event>| {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(WIDTH, 600.0))),
                events,
                ..Default::default()
            };
            let mut mode = ToolMode::Pan;
            let mut output = ctx.run_ui(input, |ui| {
                if let Some(action) = show(ui, &mut mode, state).action {
                    chosen = Some(action);
                }
            });
            output.textures_delta.clear();
            output
                .shapes
                .iter()
                .filter_map(|clipped| match &clipped.shape {
                    egui::Shape::Text(text) => {
                        Some((text.galley.job.text.clone(), text.visual_bounding_rect()))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        let click = |pos| {
            [
                vec![Event::PointerMoved(pos)],
                vec![Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                }],
                vec![Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }],
            ]
        };

        // The menu button is the last thing on the bar; find it by what it opens.
        let mut opened = false;
        'search: for x in (0..WIDTH as u32).rev().step_by(6) {
            for events in click(Pos2::new(x as f32, ROW_Y)) {
                frame(events);
            }
            for _ in 0..2 {
                if frame(Vec::new())
                    .iter()
                    .any(|(text, _)| text == "Reset layout")
                {
                    opened = true;
                    break 'search;
                }
            }
        }
        assert!(opened, "the menu opens");
        let shown = frame(Vec::new());
        let position_of = |wanted: &str| {
            shown
                .iter()
                .find(|(text, _)| text == wanted)
                .unwrap_or_else(|| panic!("{wanted} in {shown:?}"))
                .1
                .center()
        };
        for title in Panel::ALL.map(Panel::title) {
            position_of(title);
        }
        let point_to_point = position_of(Panel::PointToPoint.title());

        for events in click(point_to_point) {
            frame(events);
        }
        assert_eq!(
            chosen,
            Some(TopPanelAction::TogglePanel(Panel::PointToPoint))
        );
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
