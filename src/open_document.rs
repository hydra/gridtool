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
            // A newly added grid or a duplicate becomes the active object.
            self.view.selection.select(document.newest_object());
        }
    }

    /// Applies what the user picked in the object tree: an object becomes the active
    /// object and the view is centred on it.
    pub fn pick(&mut self, pick: TreePick) {
        let selection = &mut self.view.selection;
        match pick {
            TreePick::Object(id) => {
                selection.select(Some(id));
                let document = self.editor.document();
                if let Some(focus) = document.object_focus_point(id) {
                    let size = egui::Vec2::new(document.width() as f32, document.height() as f32);
                    self.view.viewport.center_on(focus, size);
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
