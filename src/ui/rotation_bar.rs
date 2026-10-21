use eframe::egui;

use crate::angle::{format_degrees, parse_degrees};
use crate::document::Document;

const ANGLE_FIELD_WIDTH: f32 = 80.0;

/// Tool options row for the rotate tool: shows the document's rotation and lets the
/// user type a new one.
#[derive(Default)]
pub struct RotationBar {
    angle_text: String,
}

impl RotationBar {
    pub fn show(&mut self, ui: &mut egui::Ui, document: Option<&mut Document>) {
        egui::Panel::top("rotation_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                let label = ui.label("Rotation (°):");
                let has_document = document.is_some();
                let field = ui
                    .add_enabled(
                        has_document,
                        egui::TextEdit::singleline(&mut self.angle_text)
                            .desired_width(ANGLE_FIELD_WIDTH),
                    )
                    .labelled_by(label.id);

                if let Some(document) = document {
                    self.sync(&field, document);
                } else {
                    self.angle_text.clear();
                }
            });
        });
    }

    /// While the user is typing, applies every valid angle they enter; otherwise keeps
    /// the field showing the document's current rotation.
    fn sync(&mut self, field: &egui::Response, document: &mut Document) {
        if !field.has_focus() {
            self.angle_text = format_degrees(document.rotation_degrees());
        } else if field.changed()
            && let Some(degrees) = parse_degrees(&self.angle_text)
        {
            document.set_rotation_degrees(degrees);
        }
    }
}
