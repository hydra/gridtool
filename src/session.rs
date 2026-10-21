//! What the application has open, and the file commands that change it: New, Load,
//! Save, Save as and Open image, including the questions they ask (unsaved changes,
//! overwriting a file) and the errors they report.
//!
//! Native file dialogs block, so each command runs to completion within a frame unless
//! it needs to ask a question; then it waits in [`Session::dialog`] until
//! [`Session::answer`] is called.

use std::path::{Path, PathBuf};

use eframe::egui;
use gridtool::document_file;

use crate::file_picker::{DocumentFilePicker, ImageFilePicker};
use crate::open_document::OpenDocument;
use crate::open_image::pick_and_decode;
use crate::ui::dialog::{Dialog, DialogButton};

/// What the central area currently holds.
pub enum Content {
    Empty,
    Error(String),
    Document(Box<OpenDocument>),
}

/// A command that replaces the open document, once any unsaved changes are dealt with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replacement {
    /// Close the document, forgetting its file.
    New,
    /// Load a document file.
    Load,
    /// Start a new document from an image file.
    OpenImage,
}

/// A question or report waiting for the user.
#[derive(Debug, Clone, PartialEq)]
enum Prompt {
    /// Save before `then`?
    UnsavedChanges {
        then: Replacement,
    },
    /// Replace the existing file at `path`, then continue with `then`?
    Overwrite {
        path: PathBuf,
        then: Option<Replacement>,
    },
    Error(Dialog),
}

pub struct Session {
    content: Content,
    image_picker: Box<dyn ImageFilePicker>,
    document_picker: Box<dyn DocumentFilePicker>,
    prompt: Option<Prompt>,
}

impl Session {
    pub fn new(
        image_picker: Box<dyn ImageFilePicker>,
        document_picker: Box<dyn DocumentFilePicker>,
    ) -> Self {
        Self {
            content: Content::Empty,
            image_picker,
            document_picker,
            prompt: None,
        }
    }

    pub fn content_mut(&mut self) -> &mut Content {
        &mut self.content
    }

    pub fn document(&self) -> Option<&OpenDocument> {
        match &self.content {
            Content::Document(open) => Some(open),
            Content::Empty | Content::Error(_) => None,
        }
    }

    pub fn document_mut(&mut self) -> Option<&mut OpenDocument> {
        match &mut self.content {
            Content::Document(open) => Some(open),
            Content::Empty | Content::Error(_) => None,
        }
    }

    fn has_unsaved_changes(&self) -> bool {
        self.document()
            .is_some_and(OpenDocument::has_unsaved_changes)
    }

    /// The dialog waiting for an answer, if any.
    pub fn dialog(&self) -> Option<Dialog> {
        self.prompt.as_ref().map(|prompt| match prompt {
            Prompt::UnsavedChanges { .. } => Dialog::unsaved_changes(),
            Prompt::Overwrite { path, .. } => Dialog::overwrite(&file_name(path)),
            Prompt::Error(dialog) => dialog.clone(),
        })
    }

    /// Replaces the open document, first asking about unsaved changes if there are any.
    pub fn replace(&mut self, ctx: &egui::Context, replacement: Replacement) {
        if self.prompt.is_some() {
            return;
        }
        if self.has_unsaved_changes() {
            self.prompt = Some(Prompt::UnsavedChanges { then: replacement });
        } else {
            self.carry_out(ctx, replacement);
        }
    }

    /// Saves to the document's file, asking for one if it has none yet.
    pub fn save(&mut self, ctx: &egui::Context) {
        self.save_then(ctx, None);
    }

    /// Always asks where to save.
    pub fn save_as(&mut self, ctx: &egui::Context) {
        self.save_as_then(ctx, None);
    }

    /// Acts on the user's answer to [`Session::dialog`].
    pub fn answer(&mut self, ctx: &egui::Context, button: DialogButton) {
        let Some(prompt) = self.prompt.take() else {
            return;
        };
        match (prompt, button) {
            (Prompt::UnsavedChanges { then }, DialogButton::Yes) => self.save_then(ctx, Some(then)),
            (Prompt::UnsavedChanges { then }, DialogButton::No) => self.carry_out(ctx, then),
            (Prompt::Overwrite { path, then }, DialogButton::Yes) => self.write(ctx, &path, then),
            // Cancel, No to overwriting, or OK on an error: nothing more happens.
            _ => {}
        }
    }

    fn save_then(&mut self, ctx: &egui::Context, then: Option<Replacement>) {
        match self
            .document()
            .and_then(|open| open.file())
            .map(Path::to_path_buf)
        {
            Some(file) => self.write(ctx, &file, then),
            None => self.save_as_then(ctx, then),
        }
    }

    fn save_as_then(&mut self, ctx: &egui::Context, then: Option<Replacement>) {
        let Some(suggestion) = self.document().map(OpenDocument::save_suggestion) else {
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
            self.prompt = Some(Prompt::Overwrite { path, then });
        } else {
            self.write(ctx, &path, then);
        }
    }

    fn write(&mut self, ctx: &egui::Context, path: &Path, then: Option<Replacement>) {
        let Some(open) = self.document_mut() else {
            return;
        };
        match open.save_to(path) {
            Ok(()) => {
                if let Some(replacement) = then {
                    self.carry_out(ctx, replacement);
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

    /// Replaces the open document without asking about unsaved changes.
    fn carry_out(&mut self, ctx: &egui::Context, replacement: Replacement) {
        match replacement {
            Replacement::New => self.content = Content::Empty,
            Replacement::Load => {
                let Some(path) = self.document_picker.pick_document() else {
                    return;
                };
                match document_file::load(&path) {
                    Ok(loaded) => {
                        self.content = Content::Document(Box::new(OpenDocument::from_loaded(
                            ctx, loaded, path,
                        )));
                    }
                    // The open document, if any, is kept.
                    Err(error) => {
                        self.prompt = Some(Prompt::Error(Dialog::error(
                            "Could not load document",
                            format!("{}: {error}", path.display()),
                        )));
                    }
                }
            }
            Replacement::OpenImage => {
                if let Some(result) = pick_and_decode(self.image_picker.as_ref()) {
                    self.content = match result {
                        Ok(opened) => {
                            Content::Document(Box::new(OpenDocument::from_image(ctx, opened)))
                        }
                        Err(error) => Content::Error(error.to_string()),
                    };
                }
            }
        }
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests;
