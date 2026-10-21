use eframe::egui;
use gridtool::document::Document;

use super::angle::{format_degrees, parse_degrees};
use super::viewport::Viewport;

const ANGLE_FIELD_WIDTH: f32 = 80.0;

/// Tool options row for the rotate tool: shows the view rotation and lets the user type
/// a new one. View rotation is not a document action, so it is applied directly.
#[derive(Default)]
pub struct RotationBar {
    angle_text: String,
}

impl RotationBar {
    /// Shows the row. Typed rotations pivot about the document origin.
    pub fn show(&mut self, ui: &mut egui::Ui, viewport: &mut Viewport, document: &Document) {
        egui::Panel::top("rotation_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                let label = ui.label("View rotation (°):");
                let field = ui
                    .add(
                        egui::TextEdit::singleline(&mut self.angle_text)
                            .desired_width(ANGLE_FIELD_WIDTH),
                    )
                    .labelled_by(label.id);
                self.sync(&field, viewport, document);
            });
        });
    }

    /// While the user is typing, applies every valid angle they enter; otherwise keeps
    /// the field showing the current rotation.
    fn sync(&mut self, field: &egui::Response, viewport: &mut Viewport, document: &Document) {
        if !field.has_focus() {
            self.angle_text = format_degrees(viewport.rotation_degrees());
        } else if field.changed()
            && let Some(degrees) = parse_degrees(&self.angle_text)
        {
            let size = egui::Vec2::new(document.width() as f32, document.height() as f32);
            viewport.rotate_about(degrees, document.origin(), size);
        }
    }
}
