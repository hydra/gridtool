//! Every icon used by the UI, in one place so they can be checked against the bundled
//! font. Icons are Google Material Symbols (<https://fonts.google.com/icons>) from the
//! `egui_material_icons` crate, whose font must be registered at start-up
//! ([`egui_material_icons::initialize`]).

use egui_material_icons::MaterialIcon;
use egui_material_icons::icons as material;

pub const NEW: MaterialIcon = material::ICON_NOTE_ADD;
pub const LOAD: MaterialIcon = material::ICON_FOLDER_OPEN;
pub const SAVE: MaterialIcon = material::ICON_SAVE;
pub const SAVE_AS: MaterialIcon = material::ICON_SAVE_AS;
pub const OPEN_IMAGE: MaterialIcon = material::ICON_IMAGE;
pub const PAN: MaterialIcon = material::ICON_PAN_TOOL;
pub const ROTATE: MaterialIcon = material::ICON_ROTATE_RIGHT;
/// A ruler.
pub const SCALE: MaterialIcon = material::ICON_STRAIGHTEN;
pub const MEASURE: MaterialIcon = material::ICON_MEASURING_TAPE;
pub const GRID: MaterialIcon = material::ICON_GRID_ON;
/// A crosshair.
pub const ORIGIN: MaterialIcon = material::ICON_MY_LOCATION;
pub const UNDO: MaterialIcon = material::ICON_UNDO;
pub const REDO: MaterialIcon = material::ICON_REDO;
pub const DELETE: MaterialIcon = material::ICON_DELETE;
pub const DUPLICATE: MaterialIcon = material::ICON_CONTENT_COPY;

#[cfg(test)]
pub const ALL: [MaterialIcon; 15] = [
    NEW, LOAD, SAVE, SAVE_AS, OPEN_IMAGE, PAN, ROTATE, SCALE, MEASURE, GRID, ORIGIN, UNDO, REDO,
    DELETE, DUPLICATE,
];

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{self, FontId, RawInput};

    #[test]
    fn every_icon_is_a_single_distinct_character() {
        for icon in ALL {
            assert_eq!(
                icon.codepoint.chars().count(),
                1,
                "{icon:?} is not a single glyph"
            );
        }
        let distinct: std::collections::HashSet<_> =
            ALL.iter().map(|icon| icon.codepoint).collect();
        assert_eq!(distinct.len(), ALL.len());
    }

    /// The icon font family holds only the bundled Material Symbols font, so this proves
    /// each icon comes from it; system fonts are not guaranteed to be present.
    #[test]
    fn every_icon_is_in_the_bundled_icon_font() {
        let ctx = egui::Context::default();
        egui_material_icons::initialize(&ctx);
        // Fonts are (re)built at the start of a frame.
        let mut output = ctx.run_ui(RawInput::default(), |_| {});
        output.textures_delta.clear();

        for icon in ALL {
            let font = FontId::new(18.0, icon.font_family());
            assert!(
                ctx.fonts_mut(|fonts| fonts.has_glyphs(&font, icon.codepoint)),
                "{icon:?} has no glyph"
            );
        }
    }
}
