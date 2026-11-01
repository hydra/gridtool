//! What the application has open, and the file commands that change it: New, Load, Save,
//! Save as and closing, including the questions they ask (unsaved changes, overwriting a file)
//! and the errors they report.
//!
//! Native file dialogs block, so each command runs to completion within a frame unless
//! it needs to ask a question; then it waits in [`Session::dialog`] until
//! [`Session::answer`] is called.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use eframe::egui;
use gridtool::document_file;

use crate::file_picker::{DocumentFilePicker, ImageFilePicker};
use crate::open_document::{OpenDocument, file_name};
use crate::open_image::pick_and_decode;
use crate::ui::dialog::{Dialog, DialogButton};

/// Identifies an open document for as long as the session lasts; never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(u64);

/// A command that closes something, once any unsaved changes are dealt with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Closing {
    /// Close one document.
    Document(DocumentId),
    /// Close the window, ending the application.
    Window,
}

/// A question or report waiting for the user.
#[derive(Debug, Clone, PartialEq)]
enum Prompt {
    /// Save `document` before `then`?
    UnsavedChanges {
        document: DocumentId,
        then: Closing,
    },
    /// Replace the existing file at `path` with `document`, then continue with `then`?
    Overwrite {
        document: DocumentId,
        path: PathBuf,
        then: Option<Closing>,
    },
    Error(Dialog),
}

pub struct Session {
    /// In the order they were opened.
    documents: Vec<(DocumentId, OpenDocument)>,
    /// The document the tool bar, panels and commands act on.
    active: Option<DocumentId>,
    next_id: u64,
    image_picker: Box<dyn ImageFilePicker>,
    document_picker: Box<dyn DocumentFilePicker>,
    prompt: Option<Prompt>,
    /// While closing the window, the documents the user has agreed to lose.
    discarded: HashSet<DocumentId>,
    /// The user has agreed to close the window.
    exiting: bool,
}

impl Session {
    pub fn new(
        image_picker: Box<dyn ImageFilePicker>,
        document_picker: Box<dyn DocumentFilePicker>,
    ) -> Self {
        Self {
            documents: Vec::new(),
            active: None,
            next_id: 0,
            image_picker,
            document_picker,
            prompt: None,
            discarded: HashSet::new(),
            exiting: false,
        }
    }

    pub fn document_ids(&self) -> impl Iterator<Item = DocumentId> + '_ {
        self.documents.iter().map(|(id, _)| *id)
    }

    pub fn document_by_id(&self, id: DocumentId) -> Option<&OpenDocument> {
        self.documents
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, open)| open)
    }

    pub fn document_by_id_mut(&mut self, id: DocumentId) -> Option<&mut OpenDocument> {
        self.documents
            .iter_mut()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, open)| open)
    }

    pub fn active_id(&self) -> Option<DocumentId> {
        self.active
    }

    /// Makes `id` the active document, if it is open.
    pub fn activate(&mut self, id: DocumentId) {
        if self.document_by_id(id).is_some() {
            self.active = Some(id);
        }
    }

    /// The active document.
    pub fn document(&self) -> Option<&OpenDocument> {
        self.document_by_id(self.active?)
    }

    /// The active document.
    pub fn document_mut(&mut self) -> Option<&mut OpenDocument> {
        self.document_by_id_mut(self.active?)
    }

    /// The dialog waiting for an answer, if any.
    pub fn dialog(&self) -> Option<Dialog> {
        self.prompt.as_ref().map(|prompt| match prompt {
            Prompt::UnsavedChanges { document, .. } => {
                let name = self
                    .document_by_id(*document)
                    .map_or_else(String::new, OpenDocument::name);
                Dialog::unsaved_changes(&name)
            }
            Prompt::Overwrite { path, .. } => Dialog::overwrite(&file_name(path)),
            Prompt::Error(dialog) => dialog.clone(),
        })
    }

    /// New: starts a new document from an image the user picks, which becomes active.
    pub fn open_image(&mut self, ctx: &egui::Context) {
        if self.prompt.is_some() {
            return;
        }
        match pick_and_decode(self.image_picker.as_ref()) {
            None => {}
            Some(Ok(opened)) => self.add(OpenDocument::from_image(ctx, opened)),
            Some(Err(error)) => {
                self.prompt = Some(Prompt::Error(Dialog::error(
                    "Could not open image",
                    error.to_string(),
                )));
            }
        }
    }

    /// Load: opens a document file the user picks, which becomes active. A file that is
    /// already open is not opened twice: its document becomes active instead.
    pub fn load(&mut self, ctx: &egui::Context) {
        if self.prompt.is_some() {
            return;
        }
        let Some(path) = self.document_picker.pick_document() else {
            return;
        };
        let already_open = self.documents.iter().find(|(_, open)| {
            open.file()
                .is_some_and(|file| is_same_file(file, path.as_path()))
        });
        if let Some((id, _)) = already_open {
            self.active = Some(*id);
            return;
        }
        match document_file::load(&path) {
            Ok(loaded) => self.add(OpenDocument::from_loaded(ctx, loaded, path)),
            // The open documents are kept.
            Err(error) => {
                self.prompt = Some(Prompt::Error(Dialog::error(
                    "Could not load document",
                    format!("{}: {error}", path.display()),
                )));
            }
        }
    }

    fn add(&mut self, open: OpenDocument) {
        let id = DocumentId(self.next_id);
        self.next_id += 1;
        self.documents.push((id, open));
        self.active = Some(id);
    }

    /// Closes a document, first asking about unsaved changes if it has any.
    pub fn close(&mut self, ctx: &egui::Context, id: DocumentId) {
        if self.prompt.is_some() {
            return;
        }
        let closing = Closing::Document(id);
        if self
            .document_by_id(id)
            .is_some_and(OpenDocument::has_unsaved_changes)
        {
            self.active = Some(id);
            self.prompt = Some(Prompt::UnsavedChanges {
                document: id,
                then: closing,
            });
        } else {
            self.carry_out(ctx, closing);
        }
    }

    /// Called when the user asks to close the window; returns whether it may close now.
    ///
    /// With unsaved changes it may not: the Unsaved changes dialog is shown for each such
    /// document in turn, and answering the last closes the window by itself.
    pub fn request_close(&mut self, ctx: &egui::Context) -> bool {
        if !self.exiting && self.prompt.is_none() {
            self.discarded.clear();
            self.carry_out(ctx, Closing::Window);
        }
        self.exiting
    }

    /// The window title: the active document's file name, or "`unsaved document`" for one
    /// that was never saved, then the application name.
    pub fn window_title(&self) -> String {
        match self.document() {
            None => crate::APP_NAME.to_owned(),
            Some(open) => {
                let name = open
                    .file()
                    .map_or_else(|| UNSAVED_NAME.to_owned(), file_name);
                format!("{name} - {}", crate::APP_NAME)
            }
        }
    }

    /// Saves the active document to its file, asking for one if it has none yet.
    pub fn save(&mut self, ctx: &egui::Context) {
        if let Some(id) = self.active {
            self.save_then(ctx, id, None);
        }
    }

    /// Saves the active document, always asking where.
    pub fn save_as(&mut self, ctx: &egui::Context) {
        if let Some(id) = self.active {
            self.save_as_then(ctx, id, None);
        }
    }

    /// Acts on the user's answer to [`Session::dialog`].
    pub fn answer(&mut self, ctx: &egui::Context, button: DialogButton) {
        let Some(prompt) = self.prompt.take() else {
            return;
        };
        match (prompt, button) {
            (Prompt::UnsavedChanges { document, then }, DialogButton::Yes) => {
                self.save_then(ctx, document, Some(then));
            }
            (Prompt::UnsavedChanges { document, then }, DialogButton::No) => {
                self.discarded.insert(document);
                self.carry_out(ctx, then);
            }
            (
                Prompt::Overwrite {
                    document,
                    path,
                    then,
                },
                DialogButton::Yes,
            ) => self.write(ctx, document, &path, then),
            // Cancel, No to overwriting, or OK on an error: nothing more happens.
            _ => {}
        }
    }

    fn save_then(&mut self, ctx: &egui::Context, id: DocumentId, then: Option<Closing>) {
        match self
            .document_by_id(id)
            .and_then(OpenDocument::file)
            .map(Path::to_path_buf)
        {
            Some(file) => self.write(ctx, id, &file, then),
            None => self.save_as_then(ctx, id, then),
        }
    }

    fn save_as_then(&mut self, ctx: &egui::Context, id: DocumentId, then: Option<Closing>) {
        let Some(suggestion) = self.document_by_id(id).map(OpenDocument::save_suggestion) else {
            return;
        };
        // Cancelling the file dialog also abandons whatever was to follow the save.
        let Some(chosen) = self.document_picker.pick_save_path(&suggestion) else {
            return;
        };
        let path = document_file::with_extension(&chosen);
        // The native dialog confirmed overwriting `chosen`, but not a file we only found
        // by adding the extension.
        if path != chosen && path.exists() {
            self.prompt = Some(Prompt::Overwrite {
                document: id,
                path,
                then,
            });
        } else {
            self.write(ctx, id, &path, then);
        }
    }

    fn write(&mut self, ctx: &egui::Context, id: DocumentId, path: &Path, then: Option<Closing>) {
        let Some(open) = self.document_by_id_mut(id) else {
            return;
        };
        match open.save_to(path) {
            Ok(()) => {
                if let Some(closing) = then {
                    self.carry_out(ctx, closing);
                }
            }
            Err(error) => {
                self.prompt = Some(Prompt::Error(Dialog::error(
                    "Could not save document",
                    format!("{}: {error}", path.display()),
                )));
            }
        }
    }

    /// Closes what `closing` names without asking about the document just dealt with. No
    /// document becomes active in place of a closed one: which should depends on what is
    /// visible, so the document area decides.
    fn carry_out(&mut self, ctx: &egui::Context, closing: Closing) {
        match closing {
            Closing::Document(id) => {
                self.documents.retain(|(candidate, _)| *candidate != id);
                self.discarded.remove(&id);
                if self.active == Some(id) {
                    self.active = None;
                }
            }
            Closing::Window => match self.next_document_to_ask_about() {
                Some(document) => {
                    self.active = Some(document);
                    self.prompt = Some(Prompt::UnsavedChanges {
                        document,
                        then: Closing::Window,
                    });
                }
                None => {
                    self.exiting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            },
        }
    }

    fn next_document_to_ask_about(&self) -> Option<DocumentId> {
        self.documents
            .iter()
            .find(|(id, open)| open.has_unsaved_changes() && !self.discarded.contains(id))
            .map(|(id, _)| *id)
    }
}

/// The name shown for a document that has no file yet.
const UNSAVED_NAME: &str = "unsaved document";

/// Whether two paths lead to the same file, however they are written.
fn is_same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

#[cfg(test)]
mod tests;
