use std::collections::HashMap;

use eframe::egui;

struct Edit {
    original: String,
    text: String,
}

/// Single-line text fields whose value is committed once, when editing ends (focus is
/// lost, including via Enter), rather than on every keystroke.
///
/// While a field is unfocused it shows the current value; while focused it keeps the
/// user's text. One instance can serve any number of fields, each with its own id.
#[derive(Default)]
pub struct CommitField {
    edits: HashMap<egui::Id, Edit>,
}

impl CommitField {
    /// Shows the field. Returns its response, plus the edited text if editing ended this
    /// frame and the text differs from what was shown when editing began.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        id: egui::Id,
        current: String,
        width: f32,
    ) -> (egui::Response, Option<String>) {
        let was_editing = self.edits.contains_key(&id);
        let mut scratch = current.clone();
        let text = match self.edits.get_mut(&id) {
            Some(edit) => &mut edit.text,
            None => &mut scratch,
        };
        let response = ui.add(egui::TextEdit::singleline(text).id(id).desired_width(width));

        let mut committed = None;
        if response.lost_focus() {
            let (original, text) = match self.edits.remove(&id) {
                Some(edit) => (edit.original, edit.text),
                None => (current, scratch),
            };
            committed = (text != original).then_some(text);
        } else if response.has_focus() && !was_editing {
            self.edits.insert(
                id,
                Edit {
                    original: current,
                    text: scratch,
                },
            );
        } else if !response.has_focus() {
            self.edits.remove(&id);
        }
        (response, committed)
    }
}
