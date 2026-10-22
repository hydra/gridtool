//! Saving and loading documents as `.ron` files.
//!
//! The runtime [`Document`] is converted to a versioned description ([`DocumentFile`])
//! before it is written, and a loaded description is converted back and validated. The
//! history is not saved. The image is not embedded: the file refers to it by path,
//! relative to the document file when the image is in the same folder or below it.

mod v1;
mod v2;

use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::document::Document;
use crate::geometry::Point;
use crate::imaging::{DecodedImage, ImageLoadError, decode_file};

pub use v1::DocumentV1;
pub use v2::DocumentV2;

/// The extension of document files, without the dot.
pub const EXTENSION: &str = "ron";

/// A saved document, in one of the supported format versions. New versions are added as
/// new variants; old variants stay so that old files can still be loaded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentFile {
    V1(DocumentV1),
    V2(DocumentV2),
}

impl DocumentFile {
    /// Describes `document` in the current format version.
    pub fn describe(document: &Document, image: String) -> Self {
        Self::V2(DocumentV2::describe(document, image))
    }

    /// The stored image reference.
    pub fn image(&self) -> &str {
        match self {
            Self::V1(file) => &file.image,
            Self::V2(file) => &file.image,
        }
    }

    /// Re-creates the runtime document, migrating older versions first.
    pub fn to_document(&self) -> Result<Document, InvalidDocument> {
        match self {
            Self::V1(file) => DocumentV2::from(file.clone()).to_document(),
            Self::V2(file) => file.to_document(),
        }
    }

    pub fn to_ron(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(self, PrettyConfig::new())
    }

    pub fn from_ron(text: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(text)
    }
}

/// A document read from disk, with its image.
#[derive(Debug)]
pub struct LoadedDocument {
    pub document: Document,
    pub image_path: PathBuf,
    pub image: DecodedImage,
}

/// Writes `document`, whose image is at `image_path`, to `path`. The file is written in
/// full to a temporary file first and then moved into place, so a failed save never
/// leaves a half-written document behind.
pub fn save(path: &Path, document: &Document, image_path: &Path) -> Result<(), SaveError> {
    let file = DocumentFile::describe(document, image_reference(image_path, path));
    let text = file.to_ron().map_err(SaveError::Serialize)?;
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    fs::write(&temporary, text)
        .and_then(|()| fs::rename(&temporary, path))
        .map_err(|error| {
            fs::remove_file(&temporary).ok();
            SaveError::Io(error)
        })
}

/// Reads the document at `path` and the image it refers to.
pub fn load(path: &Path) -> Result<LoadedDocument, LoadError> {
    let text = fs::read_to_string(path).map_err(LoadError::Io)?;
    let file = DocumentFile::from_ron(&text).map_err(LoadError::Parse)?;
    let document = file.to_document().map_err(LoadError::Invalid)?;
    let image_path = resolve_image(file.image(), path);
    let image = decode_file(&image_path).map_err(|error| LoadError::Image {
        path: image_path.clone(),
        error,
    })?;
    let expected = (document.width(), document.height());
    if (image.width, image.height) != expected {
        return Err(LoadError::ImageSizeMismatch {
            path: image_path,
            expected,
            actual: (image.width, image.height),
        });
    }
    Ok(LoadedDocument {
        document,
        image_path,
        image,
    })
}

/// `path` with the document extension added if it has a different one, or none.
pub fn with_extension(path: &Path) -> PathBuf {
    let has_extension = path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(EXTENSION));
    if has_extension {
        path.to_path_buf()
    } else {
        let mut name = path.as_os_str().to_owned();
        name.push(".");
        name.push(EXTENSION);
        PathBuf::from(name)
    }
}

/// How the document at `document_path` refers to `image`: relative (with `/`
/// separators) when the image is in the document's folder or below it, otherwise the
/// image's full path.
fn image_reference(image: &Path, document_path: &Path) -> String {
    let folder = document_path.parent().unwrap_or(Path::new(""));
    match image.strip_prefix(folder) {
        Ok(relative) if !folder.as_os_str().is_empty() => relative
            .components()
            .filter_map(|component| match component {
                Component::Normal(part) => Some(part.to_string_lossy()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("/"),
        _ => image.to_string_lossy().into_owned(),
    }
}

/// The image file a stored reference points to: relative references are resolved
/// against the folder of the document at `document_path`.
fn resolve_image(reference: &str, document_path: &Path) -> PathBuf {
    let reference = Path::new(reference);
    if reference.is_absolute() {
        reference.to_path_buf()
    } else {
        document_path
            .parent()
            .unwrap_or(Path::new(""))
            .join(reference)
    }
}

/// Why a stored document cannot be turned back into a valid runtime document.
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidDocument {
    EmptySize,
    PointOutside(Point),
    WrongScalePointCount,
    TooFewAreaPoints(usize),
    NotPositive(f64),
    BadOpacity(f32),
    /// Zero, duplicated, or not below the stored next object id.
    BadObjectId(u64),
}

impl fmt::Display for InvalidDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySize => f.write_str("the document has no size"),
            Self::PointOutside(point) => write!(f, "point {point} is outside the document"),
            Self::WrongScalePointCount => {
                f.write_str("the number of scale points does not match the method")
            }
            Self::TooFewAreaPoints(count) => {
                write!(f, "an area has {count} points, but needs at least 3")
            }
            Self::NotPositive(value) => write!(f, "{value} should be a positive number"),
            Self::BadOpacity(value) => write!(f, "opacity {value} is not between 0 and 1"),
            Self::BadObjectId(id) => write!(f, "object id {id} is invalid or used twice"),
        }
    }
}

#[derive(Debug)]
pub enum SaveError {
    Serialize(ron::Error),
    Io(io::Error),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialize(error) => write!(f, "could not encode the document: {error}"),
            Self::Io(error) => write!(f, "could not write the file: {error}"),
        }
    }
}

impl std::error::Error for SaveError {}

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Parse(ron::error::SpannedError),
    Invalid(InvalidDocument),
    Image {
        path: PathBuf,
        error: ImageLoadError,
    },
    ImageSizeMismatch {
        path: PathBuf,
        expected: (u32, u32),
        actual: (u32, u32),
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "could not read the file: {error}"),
            Self::Parse(error) => write!(f, "the file is not a GridTool document: {error}"),
            Self::Invalid(reason) => write!(f, "the document is invalid: {reason}"),
            Self::Image { path, error } => {
                write!(f, "could not open the image {}: {error}", path.display())
            }
            Self::ImageSizeMismatch {
                path,
                expected,
                actual,
            } => write!(
                f,
                "the image {} is {}×{} pixels, but the document expects {}×{}",
                path.display(),
                actual.0,
                actual.1,
                expected.0,
                expected.1
            ),
        }
    }
}

impl std::error::Error for LoadError {}

#[cfg(test)]
mod tests;
