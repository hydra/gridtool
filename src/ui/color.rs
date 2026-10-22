//! Conversions between the document's colour type and egui's.

use eframe::egui::Color32;
use gridtool::color::Rgb;

pub fn to_color32(rgb: Rgb) -> Color32 {
    Color32::from_rgb(rgb.r, rgb.g, rgb.b)
}

/// `rgb` with `opacity` (0 to 1) applied as alpha.
pub fn with_opacity(rgb: Rgb, opacity: f32) -> Color32 {
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgba_unmultiplied(rgb.r, rgb.g, rgb.b, alpha)
}

pub fn from_color32(color: Color32) -> Rgb {
    Rgb::new(color.r(), color.g(), color.b())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_opaque_colours() {
        let rgb = Rgb::new(12, 200, 99);

        assert_eq!(from_color32(to_color32(rgb)), rgb);
    }

    #[test]
    fn applies_opacity_as_alpha() {
        let color = with_opacity(Rgb::new(0, 255, 0), 0.5);

        assert_eq!(color.a(), 128);
        assert_eq!(with_opacity(Rgb::new(0, 0, 0), 2.0).a(), 255);
    }
}
