use std::fmt;
use std::path::{Path, PathBuf};

use super::format;

/// An image decoded into tightly packed 8-bit RGBA pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl DecodedImage {
    pub fn size(&self) -> [usize; 2] {
        [self.width as usize, self.height as usize]
    }
}

#[derive(Debug)]
pub enum ImageLoadError {
    UnsupportedFormat(PathBuf),
    Io(std::io::Error),
    Decode(image::ImageError),
}

impl fmt::Display for ImageLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedFormat(path) => {
                write!(f, "unsupported image format: {}", path.display())
            }
            Self::Io(error) => write!(f, "could not read file: {error}"),
            Self::Decode(error) => write!(f, "could not decode image: {error}"),
        }
    }
}

impl std::error::Error for ImageLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::UnsupportedFormat(_) => None,
            Self::Io(error) => Some(error),
            Self::Decode(error) => Some(error),
        }
    }
}

impl From<std::io::Error> for ImageLoadError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<image::ImageError> for ImageLoadError {
    fn from(error: image::ImageError) -> Self {
        Self::Decode(error)
    }
}

/// Decodes encoded image bytes (any supported format) into RGBA pixels.
pub fn decode_bytes(bytes: &[u8]) -> Result<DecodedImage, ImageLoadError> {
    let rgba = image::load_from_memory(bytes)?.to_rgba8();
    Ok(DecodedImage {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}

/// Reads and decodes the image file at `path`, rejecting unsupported extensions.
pub fn decode_file(path: &Path) -> Result<DecodedImage, ImageLoadError> {
    if !format::is_supported(path) {
        return Err(ImageLoadError::UnsupportedFormat(path.to_path_buf()));
    }
    decode_bytes(&std::fs::read(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    fn encode(image: &RgbaImage, format: ImageFormat) -> Vec<u8> {
        let mut bytes = Cursor::new(Vec::new());
        image
            .write_to(&mut bytes, format)
            .expect("encoding should succeed");
        bytes.into_inner()
    }

    fn two_by_one_image() -> RgbaImage {
        let mut image = RgbaImage::new(2, 1);
        image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        image.put_pixel(1, 0, Rgba([0, 0, 255, 128]));
        image
    }

    #[test]
    fn decodes_png_pixels_and_dimensions() {
        let decoded = decode_bytes(&encode(&two_by_one_image(), ImageFormat::Png)).unwrap();

        assert_eq!(decoded.size(), [2, 1]);
        assert_eq!(decoded.rgba, vec![255, 0, 0, 255, 0, 0, 255, 128]);
    }

    #[test]
    fn decodes_bmp_dimensions() {
        let decoded = decode_bytes(&encode(&two_by_one_image(), ImageFormat::Bmp)).unwrap();

        assert_eq!(decoded.size(), [2, 1]);
        assert_eq!(decoded.rgba.len(), 2 * 4);
    }

    #[test]
    fn rejects_invalid_image_data() {
        let result = decode_bytes(b"definitely not an image");

        assert!(matches!(result, Err(ImageLoadError::Decode(_))));
    }

    #[test]
    fn rejects_files_with_unsupported_extension() {
        let result = decode_file(Path::new("picture.gif"));

        assert!(matches!(result, Err(ImageLoadError::UnsupportedFormat(_))));
    }

    #[test]
    fn reports_io_error_for_missing_file() {
        let result = decode_file(Path::new("this/file/does/not/exist.png"));

        assert!(matches!(result, Err(ImageLoadError::Io(_))));
    }
}
