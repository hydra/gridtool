use eframe::egui;

use crate::document::Document;
use crate::file_picker::{ImageFilePicker, RfdImagePicker};
use crate::imaging::ImageLoadError;
use crate::open_image::{OpenedImage, pick_and_decode};
use crate::tool_mode::ToolMode;
use crate::ui::canvas::{self, Canvas};
use crate::ui::placeholder;
use crate::ui::rotation_bar::RotationBar;
use crate::ui::top_panel::{self, TopPanelAction};

/// A document together with the GPU texture of its image.
struct OpenDocument {
    document: Document,
    texture: egui::TextureHandle,
}

/// What the central area currently holds.
enum Content {
    Empty,
    Error(String),
    Document(OpenDocument),
}

impl Content {
    fn document_mut(&mut self) -> Option<&mut Document> {
        match self {
            Content::Document(open) => Some(&mut open.document),
            Content::Empty | Content::Error(_) => None,
        }
    }
}

pub struct GridToolApp {
    picker: Box<dyn ImageFilePicker>,
    mode: ToolMode,
    content: Content,
    canvas: Canvas,
    rotation_bar: RotationBar,
}

impl GridToolApp {
    pub fn new(picker: Box<dyn ImageFilePicker>) -> Self {
        Self {
            picker,
            mode: ToolMode::default(),
            content: Content::Empty,
            canvas: Canvas::default(),
            rotation_bar: RotationBar::default(),
        }
    }

    fn handle(&mut self, ctx: &egui::Context, action: TopPanelAction) {
        match action {
            TopPanelAction::OpenImage => {
                if let Some(result) = pick_and_decode(self.picker.as_ref()) {
                    self.open(ctx, result);
                }
            }
        }
    }

    fn open(&mut self, ctx: &egui::Context, result: Result<OpenedImage, ImageLoadError>) {
        self.content = match result {
            Ok(opened) => {
                let name = opened.path.display().to_string();
                Content::Document(OpenDocument {
                    document: Document::sized_to(&opened.image),
                    texture: canvas::upload_texture(ctx, &name, &opened.image),
                })
            }
            Err(error) => Content::Error(error.to_string()),
        };
        self.canvas.reset_view();
    }
}

impl Default for GridToolApp {
    fn default() -> Self {
        Self::new(Box::new(RfdImagePicker))
    }
}

impl eframe::App for GridToolApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(action) = top_panel::show(ui, &mut self.mode) {
            let ctx = ui.ctx().clone();
            self.handle(&ctx, action);
        }

        if self.mode == ToolMode::Rotate {
            self.rotation_bar.show(ui, self.content.document_mut());
        }

        egui::CentralPanel::default().show(ui, |ui| match &mut self.content {
            Content::Empty => placeholder::show(ui, "Open an image to get started."),
            Content::Error(message) => {
                let color = ui.visuals().error_fg_color;
                placeholder::show(ui, egui::RichText::new(message.as_str()).color(color));
            }
            Content::Document(open) => {
                self.canvas
                    .show(ui, self.mode, &mut open.document, &open.texture);
            }
        });
    }
}
