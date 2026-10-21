use std::path::PathBuf;

use crate::file_picker::ImageFilePicker;
use gridtool::imaging::{DecodedImage, ImageLoadError, decode_file};

/// An image the user picked, together with where it came from.
#[derive(Debug)]
pub struct OpenedImage {
    pub path: PathBuf,
    pub image: DecodedImage,
}

/// Asks the user for an image file and decodes it.
///
/// Returns `None` if the user cancelled the picker.
pub fn pick_and_decode(
    picker: &dyn ImageFilePicker,
) -> Option<Result<OpenedImage, ImageLoadError>> {
    let path = picker.pick_image()?;
    Some(decode_file(&path).map(|image| OpenedImage { path, image }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    struct FakePicker(Option<PathBuf>);

    impl ImageFilePicker for FakePicker {
        fn pick_image(&self) -> Option<PathBuf> {
            self.0.clone()
        }
    }

    #[test]
    fn returns_none_when_user_cancels() {
        assert!(pick_and_decode(&FakePicker(None)).is_none());
    }

    #[test]
    fn returns_error_for_unsupported_file() {
        let picker = FakePicker(Some(PathBuf::from("animation.gif")));

        let result = pick_and_decode(&picker).expect("a file was picked");

        assert!(matches!(result, Err(ImageLoadError::UnsupportedFormat(_))));
    }

    #[test]
    fn decodes_picked_image_and_keeps_its_path() {
        let path = std::env::temp_dir().join(format!(
            "gridtool_open_image_test_{}.png",
            std::process::id()
        ));
        RgbaImage::from_pixel(3, 2, Rgba([10, 20, 30, 255]))
            .save(&path)
            .expect("writing the test image should succeed");

        let result = pick_and_decode(&FakePicker(Some(path.clone())));
        std::fs::remove_file(&path).ok();

        let opened = result
            .expect("a file was picked")
            .expect("decoding should succeed");
        assert_eq!(opened.path, path);
        assert_eq!(opened.image.size(), [3, 2]);
    }
}
