//! Modal message dialogs with a fixed set of buttons, e.g. "Unsaved changes".

use eframe::egui;

const MIN_WIDTH: f32 = 280.0;

/// A dialog button. Each has a fixed label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogButton {
    Yes,
    No,
    Cancel,
    Ok,
}

impl DialogButton {
    pub fn label(self) -> &'static str {
        match self {
            DialogButton::Yes => "Yes",
            DialogButton::No => "No",
            DialogButton::Cancel => "Cancel",
            DialogButton::Ok => "OK",
        }
    }
}

/// What a dialog shows.
#[derive(Debug, Clone, PartialEq)]
pub struct Dialog {
    pub title: String,
    pub message: String,
    /// Left to right.
    pub buttons: &'static [DialogButton],
    /// Has keyboard focus when the dialog opens (so Enter picks it), and is the answer
    /// when the dialog is dismissed with Escape or a click outside it.
    pub default: DialogButton,
}

impl Dialog {
    /// "Unsaved changes" with Yes / No / Cancel; Cancel is the default.
    pub fn unsaved_changes() -> Self {
        Self {
            title: "Unsaved changes".to_owned(),
            message: "Do you want to save now?".to_owned(),
            buttons: &[DialogButton::Yes, DialogButton::No, DialogButton::Cancel],
            default: DialogButton::Cancel,
        }
    }

    /// Asks whether to replace an existing file; No is the default.
    pub fn overwrite(file_name: &str) -> Self {
        Self {
            title: "File already exists".to_owned(),
            message: format!("{file_name} already exists. Do you want to replace it?"),
            buttons: &[DialogButton::Yes, DialogButton::No],
            default: DialogButton::No,
        }
    }

    /// Reports a failure, with a single OK button.
    pub fn error(title: &str, message: String) -> Self {
        Self {
            title: title.to_owned(),
            message,
            buttons: &[DialogButton::Ok],
            default: DialogButton::Ok,
        }
    }

    /// Shows the dialog over everything else, returning the button the user chose this
    /// frame, if any.
    pub fn show(&self, ctx: &egui::Context) -> Option<DialogButton> {
        let id = egui::Id::unique("modal_dialog");
        // Focus the default button once, when the dialog first appears.
        let first_frame = ctx.data_mut(|data| {
            let seen = data.get_temp::<bool>(id).unwrap_or(false);
            data.insert_temp(id, true);
            !seen
        });

        let modal = egui::Modal::new(id).show(ctx, |ui| {
            ui.set_min_width(MIN_WIDTH);
            ui.heading(&self.title);
            ui.add_space(8.0);
            ui.label(&self.message);
            ui.add_space(12.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut chosen = None;
                // Right-to-left layout, so add the buttons in reverse.
                for &button in self.buttons.iter().rev() {
                    let response = ui.button(button.label());
                    if first_frame && button == self.default {
                        response.request_focus();
                    }
                    if response.clicked() {
                        chosen = Some(button);
                    }
                }
                chosen
            })
            .inner
        });

        let answer = modal
            .inner
            .or_else(|| modal.should_close().then_some(self.default));
        if answer.is_some() {
            ctx.data_mut(|data| data.remove::<bool>(id));
        }
        answer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Key, Modifiers, RawInput};

    /// Shows `dialog` for one frame with `events`, returning its answer.
    fn frame(ctx: &egui::Context, dialog: &Dialog, events: Vec<Event>) -> Option<DialogButton> {
        let mut answer = None;
        let input = RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| answer = dialog.show(ui.ctx()));
        // There is no renderer to upload textures to.
        output.textures_delta.clear();
        answer
    }

    fn key(key: Key, pressed: bool) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        }
    }

    #[test]
    fn enter_picks_the_default_button() {
        let ctx = egui::Context::default();
        let dialog = Dialog::unsaved_changes();

        assert_eq!(frame(&ctx, &dialog, Vec::new()), None);
        let answer = frame(
            &ctx,
            &dialog,
            vec![key(Key::Enter, true), key(Key::Enter, false)],
        );

        assert_eq!(answer, Some(DialogButton::Cancel));
    }

    #[test]
    fn escape_answers_with_the_default_button() {
        let ctx = egui::Context::default();
        let dialog = Dialog::overwrite("plan.ron");

        assert_eq!(frame(&ctx, &dialog, Vec::new()), None);
        let answer = frame(&ctx, &dialog, vec![key(Key::Escape, true)]);

        assert_eq!(answer, Some(DialogButton::No));
    }
}
