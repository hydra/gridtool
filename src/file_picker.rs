use std::path::PathBuf;

use gridtool::imaging::format::SUPPORTED_EXTENSIONS;

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
            .set_title("Open image")
            .add_filter("Images", SUPPORTED_EXTENSIONS)
            .pick_file()
    }
}
