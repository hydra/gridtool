//! Escape cancels the active tool's operation, unless it is meant for something else.

use eframe::egui::{self, Key};

/// Decides whether a press of Escape cancels the active tool's operation.
///
/// When a widget such as a text field has keyboard focus, Escape only takes the focus
/// away from it. egui clears the focus before the frame's widgets run, so whether a
/// widget had focus is remembered from the end of the previous frame.
#[derive(Debug, Default)]
pub struct EscapeKey {
    widget_had_focus: bool,
}

impl EscapeKey {
    /// Call once per frame, before any widgets are shown.
    pub fn cancels_tool(&self, ctx: &egui::Context) -> bool {
        !self.widget_had_focus && ctx.input(|input| input.key_pressed(Key::Escape))
    }

    /// Call once per frame, after every widget has been shown.
    pub fn end_frame(&mut self, ctx: &egui::Context) {
        self.widget_had_focus = ctx.memory(|memory| memory.focused().is_some());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Modifiers, RawInput};

    fn escape() -> Event {
        Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    /// Runs one frame with `events`, showing a text field (focused if `focus_field`), and
    /// returns whether Escape cancelled the tool.
    fn frame(
        ctx: &egui::Context,
        escape_key: &mut EscapeKey,
        events: Vec<Event>,
        focus_field: bool,
    ) -> bool {
        let mut cancels = false;
        let input = RawInput {
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            cancels = escape_key.cancels_tool(ui.ctx());
            let mut text = String::new();
            let field = ui.text_edit_singleline(&mut text);
            if focus_field {
                field.request_focus();
            }
        });
        escape_key.end_frame(ctx);
        // There is no renderer to upload textures to.
        output.textures_delta.clear();
        cancels
    }

    #[test]
    fn escape_cancels_the_tool_when_nothing_has_focus() {
        let ctx = egui::Context::default();
        let mut escape_key = EscapeKey::default();

        assert!(!frame(&ctx, &mut escape_key, Vec::new(), false));
        assert!(frame(&ctx, &mut escape_key, vec![escape()], false));
    }

    #[test]
    fn escape_in_a_focused_text_field_only_leaves_the_field() {
        let ctx = egui::Context::default();
        let mut escape_key = EscapeKey::default();
        frame(&ctx, &mut escape_key, Vec::new(), true);
        frame(&ctx, &mut escape_key, Vec::new(), true);

        assert!(!frame(&ctx, &mut escape_key, vec![escape()], false));
        // The field no longer has focus, so the next Escape cancels the tool.
        assert!(frame(&ctx, &mut escape_key, vec![escape()], false));
    }
}
