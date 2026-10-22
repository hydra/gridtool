//! Keys that act on the active tool: Escape cancels what it is part-way through, and
//! Backspace removes the last point placed. Neither applies while a widget such as a
//! text field has keyboard focus, since the key is meant for that widget.

use eframe::egui::{self, Key};

/// The tool keys pressed this frame.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ToolKeyPresses {
    /// Escape: cancel the tool's operation.
    pub cancel: bool,
    /// Backspace: remove the last point placed.
    pub remove_last_point: bool,
}

/// Reads the tool keys, ignoring them while a widget has keyboard focus.
///
/// Escape also takes the focus away from a focused widget, and egui does that before the
/// frame's widgets run, so whether a widget had focus is remembered from the end of the
/// previous frame.
#[derive(Debug, Default)]
pub struct ToolKeys {
    widget_had_focus: bool,
}

impl ToolKeys {
    /// Call once per frame, before any widgets are shown.
    pub fn read(&self, ctx: &egui::Context) -> ToolKeyPresses {
        if self.widget_had_focus {
            return ToolKeyPresses::default();
        }
        ctx.input(|input| ToolKeyPresses {
            cancel: input.key_pressed(Key::Escape),
            remove_last_point: input.key_pressed(Key::Backspace),
        })
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

    fn press(key: Key) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    /// Runs one frame with `events`, showing a text field (focused if `focus_field`), and
    /// returns the tool keys read.
    fn frame(
        ctx: &egui::Context,
        keys: &mut ToolKeys,
        events: Vec<Event>,
        focus_field: bool,
    ) -> ToolKeyPresses {
        let mut presses = ToolKeyPresses::default();
        let input = RawInput {
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            presses = keys.read(ui.ctx());
            let mut text = String::new();
            let field = ui.text_edit_singleline(&mut text);
            if focus_field {
                field.request_focus();
            }
        });
        keys.end_frame(ctx);
        // There is no renderer to upload textures to.
        output.textures_delta.clear();
        presses
    }

    #[test]
    fn escape_and_backspace_act_on_the_tool_when_nothing_has_focus() {
        let ctx = egui::Context::default();
        let mut keys = ToolKeys::default();

        assert_eq!(
            frame(&ctx, &mut keys, Vec::new(), false),
            ToolKeyPresses::default()
        );
        assert!(frame(&ctx, &mut keys, vec![press(Key::Escape)], false).cancel);
        assert!(frame(&ctx, &mut keys, vec![press(Key::Backspace)], false).remove_last_point);
    }

    #[test]
    fn escape_in_a_focused_text_field_only_leaves_the_field() {
        let ctx = egui::Context::default();
        let mut keys = ToolKeys::default();
        frame(&ctx, &mut keys, Vec::new(), true);
        frame(&ctx, &mut keys, Vec::new(), true);

        assert!(!frame(&ctx, &mut keys, vec![press(Key::Escape)], false).cancel);
        // The field no longer has focus, so the next Escape cancels the tool.
        assert!(frame(&ctx, &mut keys, vec![press(Key::Escape)], false).cancel);
    }

    #[test]
    fn backspace_in_a_focused_text_field_edits_the_text() {
        let ctx = egui::Context::default();
        let mut keys = ToolKeys::default();
        frame(&ctx, &mut keys, Vec::new(), true);
        frame(&ctx, &mut keys, Vec::new(), true);

        let presses = frame(&ctx, &mut keys, vec![press(Key::Backspace)], true);

        assert!(!presses.remove_last_point);
    }
}
