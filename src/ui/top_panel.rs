use eframe::egui::{self, ThemePreference};
use gridtool::i18n::Language;
use gridtool::tr;

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
    About,
    SetTheme(ThemePreference),
    SetLanguage(Language),
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
    pub theme: ThemePreference,
    pub language: Language,
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
                let mut button = |ui: &mut egui::Ui, icon, tooltip: &str, enabled, clicked| {
                    if icon_button(ui, icon, tooltip, enabled) {
                        action = Some(clicked);
                    }
                };
                button(
                    ui,
                    icons::NEW,
                    &tr!("toolbar-new"),
                    true,
                    TopPanelAction::New,
                );
                button(
                    ui,
                    icons::LOAD,
                    &tr!("toolbar-load"),
                    true,
                    TopPanelAction::Load,
                );
                button(
                    ui,
                    icons::SAVE,
                    &tr!("toolbar-save"),
                    state.has_document && state.has_unsaved_changes,
                    TopPanelAction::Save,
                );
                button(
                    ui,
                    icons::SAVE_AS,
                    &tr!("toolbar-save-as"),
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
                    if selectable_icon_button(ui, tool.icon(), selected, &tool.name(), enabled) {
                        *mode = tool;
                    }
                }

                ui.add_space(SECTION_SPACING);
                button(
                    ui,
                    icons::UNDO,
                    &tr!("toolbar-undo"),
                    state.can_undo,
                    TopPanelAction::Undo,
                );
                button(
                    ui,
                    icons::REDO,
                    &tr!("toolbar-redo"),
                    state.can_redo,
                    TopPanelAction::Redo,
                );

                ui.add_space(SECTION_SPACING);
                let panels = panels_menu(ui, state);
                let preference = ui
                    .with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        button(
                            ui,
                            icons::ABOUT,
                            &tr!("toolbar-about"),
                            true,
                            TopPanelAction::About,
                        );
                        settings_menu(ui, state)
                    })
                    .inner;
                action.or(panels).or(preference)
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
        if ui.button(tr!("toolbar-reset-layout")).clicked() {
            action = Some(TopPanelAction::ResetLayout);
        }
        action
    });
    menu.response.on_hover_text(tr!("toolbar-panels"));
    menu.inner.flatten()
}

const THEMES: [ThemePreference; 3] = [
    ThemePreference::System,
    ThemePreference::Light,
    ThemePreference::Dark,
];

fn theme_name(theme: ThemePreference) -> String {
    match theme {
        ThemePreference::System => tr!("theme-button-system"),
        ThemePreference::Light => tr!("theme-button-light"),
        ThemePreference::Dark => tr!("theme-button-dark"),
    }
}

/// The menu that chooses the theme and language of the whole application.
fn settings_menu(ui: &mut egui::Ui, state: ToolBarState) -> Option<TopPanelAction> {
    let menu = ui.menu_button(icon_text(icons::SETTINGS), |ui| {
        ui.strong(tr!("settings-theme"));
        let theme = options(ui, state.theme, &THEMES, theme_name);
        ui.separator();
        ui.strong(tr!("settings-language"));
        let language = options(ui, state.language, &Language::ALL, Language::name);
        theme
            .map(TopPanelAction::SetTheme)
            .or(language.map(TopPanelAction::SetLanguage))
    });
    menu.response.on_hover_text(tr!("toolbar-settings"));
    menu.inner.flatten()
}

/// One row for each of `options`, returning the one the user picked this frame, if any.
fn options<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    current: T,
    options: &[T],
    name: fn(T) -> String,
) -> Option<T> {
    let mut picked = None;
    for &option in options {
        if ui
            .selectable_label(option == current, name(option))
            .clicked()
        {
            picked = Some(option);
        }
    }
    picked
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
    fn with_no_document_no_tool_is_active_and_only_opening_and_about_buttons_work() {
        let state = ToolBarState::default();
        assert_eq!(state.active_tool(ToolMode::Pan), None);

        let (modes, actions) = click_along_the_tool_bar(state);

        assert_eq!(modes, [ToolMode::Origin], "the mode cannot change");
        assert_eq!(
            actions,
            [
                TopPanelAction::New,
                TopPanelAction::Load,
                TopPanelAction::About
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
    fn the_about_button_is_at_the_right_hand_end_of_the_bar() {
        let ctx = egui::Context::default();
        egui_material_icons::initialize(&ctx);
        let frame = |events: Vec<Event>| {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(WIDTH, 600.0))),
                events,
                ..Default::default()
            };
            let (mut mode, mut action) = (ToolMode::Pan, None);
            let mut output = ctx.run_ui(input, |ui| {
                action = show(ui, &mut mode, ToolBarState::default()).action;
            });
            output.textures_delta.clear();
            action
        };
        let pos = Pos2::new(WIDTH - 20.0, ROW_Y);
        let button = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };

        frame(vec![Event::PointerMoved(pos)]);
        frame(vec![button(true)]);

        assert_eq!(frame(vec![button(false)]), Some(TopPanelAction::About));
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

        // Find the menu button, near the end of the bar, by what it opens.
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
        for panel in Panel::ALL {
            position_of(&panel.title());
        }
        let point_to_point = position_of(&Panel::PointToPoint.title());

        for events in click(point_to_point) {
            frame(events);
        }
        assert_eq!(
            chosen,
            Some(TopPanelAction::TogglePanel(Panel::PointToPoint))
        );
    }

    /// Opens the settings menu, and picks the row showing `option` from it.
    fn pick_from_the_settings_menu(option: &str) -> Option<TopPanelAction> {
        let ctx = egui::Context::default();
        egui_material_icons::initialize(&ctx);
        let mut chosen = None;
        let mut frame = |events: Vec<Event>| {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(WIDTH, 600.0))),
                events,
                ..Default::default()
            };
            let mut mode = ToolMode::Pan;
            let mut output = ctx.run_ui(input, |ui| {
                if let Some(action) = show(ui, &mut mode, ToolBarState::default()).action {
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
        let position_of = |texts: &[(String, Rect)], wanted: &str| {
            texts
                .iter()
                .find(|(text, _)| text == wanted)
                .unwrap_or_else(|| panic!("{wanted} in {texts:?}"))
                .1
                .center()
        };

        let texts = frame(Vec::new());
        for events in click(position_of(&texts, icons::SETTINGS.codepoint)) {
            frame(events);
        }
        frame(Vec::new());
        let texts = frame(Vec::new());
        for events in click(position_of(&texts, option)) {
            frame(events);
        }
        chosen
    }

    #[test]
    fn the_settings_menu_picks_any_theme() {
        for theme in THEMES {
            assert_eq!(
                pick_from_the_settings_menu(&theme_name(theme)),
                Some(TopPanelAction::SetTheme(theme))
            );
        }
    }

    #[test]
    fn the_settings_menu_picks_any_language() {
        // The app carries out the choice, so the language of this test program stays as it is.
        for language in Language::ALL {
            assert_eq!(
                pick_from_the_settings_menu(&language.name()),
                Some(TopPanelAction::SetLanguage(language))
            );
        }
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
