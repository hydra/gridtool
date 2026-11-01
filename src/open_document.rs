//! A document being edited, with its presentation state and where it is saved.

use std::path::{Path, PathBuf};

use eframe::egui;
use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::document_file::{self, LoadedDocument, SaveError};
use gridtool::editor::Editor;
use gridtool::imaging::DecodedImage;

use crate::open_image::OpenedImage;
use crate::ui::canvas;
use crate::ui::document_view::DocumentView;
use crate::ui::object_tree::TreePick;
use crate::ui::reveal;

pub struct OpenDocument {
    pub editor: Editor,
    pub view: DocumentView,
    image_path: PathBuf,
    /// The document file it was loaded from or last saved to.
    file: Option<PathBuf>,
    /// The document as it was when last loaded or saved; `None` if it never was.
    saved: Option<Document>,
}

impl OpenDocument {
    /// A new, never-saved document for a freshly opened image.
    pub fn from_image(ctx: &egui::Context, opened: OpenedImage) -> Self {
        Self::new(
            ctx,
            Document::sized_to(&opened.image),
            &opened.image,
            opened.path,
            None,
        )
    }

    /// A document loaded from `file`; it starts with no unsaved changes.
    pub fn from_loaded(ctx: &egui::Context, loaded: LoadedDocument, file: PathBuf) -> Self {
        Self::new(
            ctx,
            loaded.document,
            &loaded.image,
            loaded.image_path,
            Some(file),
        )
    }

    fn new(
        ctx: &egui::Context,
        document: Document,
        image: &DecodedImage,
        image_path: PathBuf,
        file: Option<PathBuf>,
    ) -> Self {
        let name = image_path.display().to_string();
        let saved = file.is_some().then(|| document.clone());
        Self {
            editor: Editor::new(document),
            view: DocumentView::new(canvas::upload_texture(ctx, &name, image)),
            image_path,
            file,
            saved,
        }
    }

    /// Whether the document differs from the file it was loaded from or saved to. A
    /// document that was never saved always has unsaved changes.
    pub fn has_unsaved_changes(&self) -> bool {
        self.saved.as_ref() != Some(self.editor.document())
    }

    /// The document file it was loaded from or last saved to.
    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// Where to suggest saving: the current file, or a `.ron` file named after the image
    /// next to it.
    pub fn save_suggestion(&self) -> PathBuf {
        self.file
            .clone()
            .unwrap_or_else(|| document_file::with_extension(&self.image_path.with_extension("")))
    }

    /// Writes the document to `path`, which becomes its file.
    pub fn save_to(&mut self, path: &Path) -> Result<(), SaveError> {
        let document = self.editor.document();
        document_file::save(path, document, &self.image_path)?;
        self.saved = Some(document.clone());
        self.file = Some(path.to_path_buf());
        Ok(())
    }

    pub fn perform(&mut self, action: Action) {
        let activates_new_object = action.activates_new_object();
        let changed = match self.editor.perform(action) {
            Ok(changed) => changed,
            Err(error) => {
                eprintln!("action rejected: {error}");
                false
            }
        };
        let document = self.editor.document();
        self.view.selection.sync_to(document);
        if activates_new_object && changed {
            // A newly added object or a duplicate becomes the active object.
            self.view.selection.select(document.newest_object());
        }
    }

    /// Applies what the user picked in the object tree: an object becomes the active
    /// object and, if none of it is on the canvas, is brought into view.
    pub fn pick(&mut self, pick: TreePick) {
        let selection = &mut self.view.selection;
        match pick {
            TreePick::Object(id) => {
                selection.select(Some(id));
                let document = self.editor.document();
                if let (Some(outline), Some(focus), Some(canvas)) = (
                    document.object_outline(id),
                    document.object_focus_point(id),
                    self.view.canvas_rect,
                ) {
                    let size = egui::Vec2::new(document.width() as f32, document.height() as f32);
                    reveal::reveal(&mut self.view.viewport, &outline, focus, size, canvas);
                }
            }
            TreePick::NoObject => selection.select(None),
        }
    }

    pub fn undo(&mut self) {
        if self.editor.undo() {
            self.on_history_changed();
        }
    }

    pub fn redo(&mut self) {
        if self.editor.redo() {
            self.on_history_changed();
        }
    }

    fn on_history_changed(&mut self) {
        self.view
            .scale_tool
            .sync_to(self.editor.document().calibration());
        self.view.selection.sync_to(self.editor.document());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::area::Area;
    use gridtool::geometry::Point;
    use gridtool::measurement::Measurement;
    use gridtool::shape::DrawnShape;
    use gridtool::units::LengthUnit;

    use crate::ui::tool_mode::ToolMode;

    const UNIT: LengthUnit = LengthUnit::Centimetre;

    fn open_document() -> OpenDocument {
        let image = DecodedImage {
            width: 100,
            height: 100,
            rgba: vec![255; 4 * 100 * 100],
        };
        let opened = OpenedImage {
            path: PathBuf::from("plan.png"),
            image,
        };
        OpenDocument::from_image(&egui::Context::default(), opened)
    }

    fn triangle() -> Vec<Point> {
        vec![
            Point::new(1.0, 1.0),
            Point::new(20.0, 1.0),
            Point::new(10.0, 20.0),
        ]
    }

    #[test]
    fn a_new_measurement_area_or_shape_becomes_active_and_keeps_its_tool() {
        let cases = [
            (
                Action::AddMeasurement(Measurement::new(
                    Point::new(1.0, 1.0),
                    Point::new(9.0, 9.0),
                    UNIT,
                )),
                ToolMode::Measure,
            ),
            (
                Action::AddArea(Area::new(triangle(), UNIT).unwrap()),
                ToolMode::Area,
            ),
            (
                Action::AddShape(DrawnShape::closed(triangle(), UNIT).unwrap()),
                ToolMode::DrawLine,
            ),
        ];
        for (action, tool) in cases {
            let mut open = open_document();

            open.perform(action);

            let document = open.editor.document();
            assert!(document.newest_object().is_some(), "{tool:?}");
            assert_eq!(open.view.selection.active(), document.newest_object());
            assert_eq!(
                open.view.tool_of_new_active_object(open.editor.document()),
                Some(tool)
            );
        }
    }

    #[test]
    fn undoing_the_addition_of_the_active_object_clears_it() {
        let mut open = open_document();
        open.perform(Action::AddArea(Area::new(triangle(), UNIT).unwrap()));

        open.undo();

        assert_eq!(open.view.selection.active(), None);
    }
}
