//! Every icon glyph used by the UI, in one place so they can be checked against the
//! bundled fonts.

pub const OPEN_IMAGE: &str = "🗁";
pub const PAN: &str = "✋";
pub const ROTATE: &str = "🔄";
pub const SCALE: &str = "📏";
/// Unicode has no tape-measure symbol; the set square is the closest match.
pub const MEASURE: &str = "📐";
pub const UNDO: &str = "↩";
pub const REDO: &str = "↪";
pub const DELETE: &str = "🗑";

#[cfg(test)]
pub const ALL: [&str; 8] = [OPEN_IMAGE, PAN, ROTATE, SCALE, MEASURE, UNDO, REDO, DELETE];

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
