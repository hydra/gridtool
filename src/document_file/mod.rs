//! Saving and loading documents as `.ron` files.
//!
//! The runtime [`Document`] is converted to a versioned description ([`DocumentFile`])
//! before it is written, and a loaded description is converted back and validated. The
//! history is not saved. The image is not embedded: the file refers to it by path,
//! relative to the document file when the image is in the same folder or below it.

mod v1;
mod v2;
mod v3;
mod v4;
mod v5;

use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

use crate::document::Document;
use crate::geometry::Point;
use crate::imaging::{DecodedImage, ImageLoadError, decode_file};
use crate::tr;

pub use v1::DocumentV1;
pub use v2::DocumentV2;
pub use v3::DocumentV3;
pub use v4::DocumentV4;
pub use v5::DocumentV5;

/// The extension of document files, without the dot.
pub const EXTENSION: &str = "ron";

/// A saved document, in one of the supported format versions. New versions are added as
/// new variants; old variants stay so that old files can still be loaded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentFile {
    V1(DocumentV1),
    V2(DocumentV2),
    V3(DocumentV3),
    V4(DocumentV4),
    V5(DocumentV5),
    /// The fields of version 5, with fill angles allowed up to a full turn.
    V6(DocumentV5),
    /// The fields of version 6, with the 2-point calibration method.
    V7(DocumentV5),
}

impl DocumentFile {
    /// Describes `document` in the current format version.
    pub fn describe(document: &Document, image: String) -> Self {
        Self::V7(DocumentV5::describe(document, image))
    }

    /// The stored image reference.
    pub fn image(&self) -> &str {
        match self {
            Self::V1(file) => &file.image,
            Self::V2(file) => &file.image,
            Self::V3(file) => &file.image,
            Self::V4(file) => &file.image,
            Self::V5(file) | Self::V6(file) | Self::V7(file) => &file.image,
        }
    }

    /// Re-creates the runtime document, migrating older versions first.
    pub fn to_document(&self) -> Result<Document, InvalidDocument> {
        match self {
            Self::V1(file) => {
                let v2 = DocumentV2::from(file.clone());
                DocumentV5::from(DocumentV4::from(DocumentV3::from(v2)))
                    .to_document_within_half_turn()
            }
            Self::V2(file) => {
                let v3 = DocumentV3::from(file.clone());
                DocumentV5::from(DocumentV4::from(v3)).to_document_within_half_turn()
            }
            Self::V3(file) => {
                DocumentV5::from(DocumentV4::from(file.clone())).to_document_within_half_turn()
            }
            Self::V4(file) => DocumentV5::from(file.clone()).to_document_within_half_turn(),
            Self::V5(file) => file.to_document_within_half_turn(),
            Self::V6(file) => file.to_document_without_two_point(),
            Self::V7(file) => file.to_document(),
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
    /// A 2-point scale in a file of a version from before there were any.
    TwoPointNotInVersion,
    TooFewAreaPoints(usize),
    TooFewShapePoints(usize),
    /// The id of an open shape stored with a fill.
    FilledOpenShape(u64),
    NotPositive(f64),
    BadOpacity(f32),
    BadFillAngle(f64),
    /// Zero, duplicated, or not below the stored next object id.
    BadObjectId(u64),
}

impl fmt::Display for InvalidDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            Self::EmptySize => tr!("error-document-empty"),
            Self::PointOutside(point) => tr!("error-document-point-outside", { point: point }),
            Self::WrongScalePointCount => tr!("error-document-scale-point-count"),
            Self::TwoPointNotInVersion => tr!("error-document-two-point-version"),
            Self::TooFewAreaPoints(count) => tr!("error-document-area-points", { count: count }),
            Self::TooFewShapePoints(count) => {
                tr!("error-document-shape-points", { count: count })
            }
            Self::FilledOpenShape(id) => tr!("error-document-filled-open-shape", { id: id }),
            Self::NotPositive(value) => tr!("error-document-not-positive", { value: value }),
            Self::BadOpacity(value) => tr!("error-document-opacity", { value: value }),
            Self::BadFillAngle(value) => tr!("error-document-fill-angle", { value: value }),
            Self::BadObjectId(id) => tr!("error-document-object-id", { id: id }),
        })
    }
}

#[derive(Debug)]
pub enum SaveError {
    Serialize(ron::Error),
    Io(io::Error),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            Self::Serialize(error) => tr!("error-save-encode", { error: error }),
            Self::Io(error) => tr!("error-save-write", { error: error }),
        })
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
        f.write_str(&match self {
            Self::Io(error) => tr!("error-load-read", { error: error }),
            Self::Parse(error) => tr!("error-load-parse", { error: error }),
            Self::Invalid(reason) => tr!("error-load-invalid", { reason: reason }),
            Self::Image { path, error } => {
                tr!("error-load-image", { path: path.display(), error: error })
            }
            Self::ImageSizeMismatch {
                path,
                expected,
                actual,
            } => tr!("error-load-image-size", {
                path: path.display(),
                actual_width: actual.0,
                actual_height: actual.1,
                expected_width: expected.0,
                expected_height: expected.1,
            }),
        })
    }
}

impl std::error::Error for LoadError {}

#[cfg(test)]
mod tests;
