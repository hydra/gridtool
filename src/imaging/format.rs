use std::path::Path;

/// File extensions (lowercase, without the dot) of the image formats the app can open.
pub const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp"];

/// Returns `true` if the path has a supported image extension (case-insensitive).
pub fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            SUPPORTED_EXTENSIONS
                .iter()
                .any(|supported| supported.eq_ignore_ascii_case(extension))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_every_supported_extension() {
        for extension in SUPPORTED_EXTENSIONS {
            let path = format!("picture.{extension}");
            assert!(is_supported(Path::new(&path)), "{path} should be supported");
        }
    }

    #[test]
    fn accepts_extensions_regardless_of_case() {
        assert!(is_supported(Path::new("picture.PNG")));
        assert!(is_supported(Path::new("picture.JpEg")));
    }

    #[test]
    fn rejects_unsupported_extensions() {
        assert!(!is_supported(Path::new("picture.gif")));
        assert!(!is_supported(Path::new("notes.txt")));
    }

    #[test]
    fn rejects_paths_without_an_extension() {
        assert!(!is_supported(Path::new("picture")));
        assert!(!is_supported(Path::new(".png")));
    }
}
