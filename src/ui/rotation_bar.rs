use eframe::egui;
use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::tr;

use super::angle::{format_degrees, parse_degrees};
use super::document_view::DocumentView;
use super::viewport::document_size;

const ANGLE_FIELD_WIDTH: f32 = 80.0;

/// Tool options row for the rotate tool: shows the view rotation and lets the user type
/// a new one, which the view shows as it is typed and which is committed as one action.
#[derive(Default)]
pub struct RotationBar {
    angle_text: String,
    /// The field had keyboard focus last frame.
    was_focused: bool,
}

impl RotationBar {
    /// Shows the controls. Typed rotations pivot about the document origin, and the one the
    /// view shows is committed when the field loses focus.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        view: &mut DocumentView,
        document: &Document,
    ) -> Option<Action> {
        let label = ui.label(tr!("rotation-label"));
        let field = ui
            .add(egui::TextEdit::singleline(&mut self.angle_text).desired_width(ANGLE_FIELD_WIDTH))
            .labelled_by(label.id);
        self.sync(&field, view, document)
    }

    /// While the user is typing, shows every valid angle they enter; otherwise keeps the field
    /// showing the current rotation.
    fn sync(
        &mut self,
        field: &egui::Response,
        view: &mut DocumentView,
        document: &Document,
    ) -> Option<Action> {
        // egui reports losing focus for two frames, so it is tracked here to commit just once.
        let left =
            std::mem::replace(&mut self.was_focused, field.has_focus()) && !field.has_focus();
        // Still set on the frame focus leaves, so the view keeps the typed rotation until the
        // action committing it has been applied.
        view.is_editing_rotation = field.has_focus() || left;
        if !field.has_focus() {
            self.angle_text = format_degrees(view.viewport.rotation_degrees());
        } else if field.changed()
            && let Some(degrees) = parse_degrees(&self.angle_text)
        {
            let size = document_size(document);
            view.viewport.rotate_about(degrees, document.origin(), size);
        }
        // Escape abandons the typed rotation, and the view turns back to the document's.
        let abandoned = field
            .ctx
            .input(|input| input.key_pressed(egui::Key::Escape));
        (left && !abandoned)
            .then(|| view.rotation_action(document))
            .flatten()
    }
}
