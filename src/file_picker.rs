use std::path::{Path, PathBuf};

use gridtool::document_file;
use gridtool::imaging::format::SUPPORTED_EXTENSIONS;
use gridtool::tr;

/// Asks the user to choose an image file. Abstracted so the app logic can be tested
/// without showing a native dialog.
pub trait ImageFilePicker {
    /// Returns the chosen file, or `None` if the user cancelled.
    fn pick_image(&self) -> Option<PathBuf>;
}

/// Native file dialog implementation backed by `rfd`.
#[derive(Debug, Default)]
pub struct RfdImagePicker;

impl ImageFilePicker for RfdImagePicker {
    fn pick_image(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr!("picker-open-image"))
            .add_filter(tr!("picker-images"), SUPPORTED_EXTENSIONS)
            .pick_file()
    }
}

/// Asks the user where to load a document from or save it to.
pub trait DocumentFilePicker {
    /// Returns the document file to load, or `None` if the user cancelled.
    fn pick_document(&self) -> Option<PathBuf>;

    /// Returns where to save, starting from `suggestion` (folder and file name), or
    /// `None` if the user cancelled. The native dialog asks before overwriting a file
    /// it returns.
    fn pick_save_path(&self, suggestion: &Path) -> Option<PathBuf>;
}

/// Native file dialog implementation backed by `rfd`.
#[derive(Debug, Default)]
pub struct RfdDocumentPicker;

impl DocumentFilePicker for RfdDocumentPicker {
    fn pick_document(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .set_title(tr!("picker-load-document"))
            .add_filter(tr!("picker-document-filter"), &[document_file::EXTENSION])
            .pick_file()
    }

    fn pick_save_path(&self, suggestion: &Path) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new()
            .set_title(tr!("picker-save-document"))
            .add_filter(tr!("picker-document-filter"), &[document_file::EXTENSION]);
        if let Some(folder) = suggestion.parent().filter(|folder| folder.is_dir()) {
            dialog = dialog.set_directory(folder);
        }
        if let Some(name) = suggestion.file_name() {
            dialog = dialog.set_file_name(name.to_string_lossy());
        }
        dialog.save_file()
    }
}
