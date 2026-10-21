//! Every icon glyph used by the UI, in one place so they can be checked against the
//! bundled fonts.

/// "Empty document" (U+1F5CB).
pub const NEW: &str = "🗋";
/// "Open folder" (U+1F5C1).
pub const LOAD: &str = "🗁";
/// "Floppy disk" (U+1F4BE).
pub const SAVE: &str = "💾";
/// "Memo" (U+1F4DD): a page being written. Unicode has no "save as" symbol.
pub const SAVE_AS: &str = "📝";
/// "Document with picture" (U+1F5BB): an image file.
pub const OPEN_IMAGE: &str = "🖻";
pub const PAN: &str = "✋";
pub const ROTATE: &str = "🔄";
pub const SCALE: &str = "📏";
/// Unicode has no tape-measure symbol; the set square is the closest match.
pub const MEASURE: &str = "📐";
/// "Squared plus" (U+229E): a square divided into four cells.
pub const GRID: &str = "\u{229E}";
/// "Position indicator" (U+2316): a crosshair.
pub const ORIGIN: &str = "\u{2316}";
pub const UNDO: &str = "↩";
pub const REDO: &str = "↪";
pub const DELETE: &str = "🗑";
/// "Two pages" (U+1F5D0): a copy of a page.
pub const DUPLICATE: &str = "🗐";

#[cfg(test)]
pub const ALL: [&str; 15] = [
    NEW, LOAD, SAVE, SAVE_AS, OPEN_IMAGE, PAN, ROTATE, SCALE, MEASURE, GRID, ORIGIN, UNDO, REDO,
    DELETE, DUPLICATE,
];

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::epaint::text::{FontDefinitions, Fonts, TextOptions};
    use eframe::egui::{FontFamily, FontId};

    #[test]
    fn every_icon_is_a_single_character() {
        for icon in ALL {
            assert_eq!(icon.chars().count(), 1, "{icon:?} is not a single glyph");
        }
    }

    /// The same fonts the egui font book lists; system fonts are not guaranteed.
    #[test]
    fn every_icon_is_in_the_bundled_fonts() {
        let mut fonts = Fonts::new(TextOptions::default(), FontDefinitions::default());
        let font = FontId::new(14.0, FontFamily::Proportional);

        for icon in ALL {
            assert!(fonts.has_glyphs(&font, icon), "{icon:?} has no glyph");
        }
    }
}
