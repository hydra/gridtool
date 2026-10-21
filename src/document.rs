use eframe::egui::Vec2;

use crate::angle::normalize_degrees;
use crate::imaging::DecodedImage;

/// The thing being edited: a rectangle of a given pixel size that can be rotated
/// about its center.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    size: Vec2,
    rotation_degrees: f32,
}

impl Document {
    /// Creates an unrotated document of `size` pixels.
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            rotation_degrees: 0.0,
        }
    }

    /// Creates a document the same size as `image`.
    pub fn sized_to(image: &DecodedImage) -> Self {
        Self::new(Vec2::new(image.width as f32, image.height as f32))
    }

    /// Size in pixels.
    pub fn size(&self) -> Vec2 {
        self.size
    }

    /// Clockwise rotation about the center, in degrees within `[0, 360)`.
    pub fn rotation_degrees(&self) -> f32 {
        self.rotation_degrees
    }

    pub fn rotation_radians(&self) -> f32 {
        self.rotation_degrees.to_radians()
    }

    pub fn set_rotation_degrees(&mut self, degrees: f32) {
        self.rotation_degrees = normalize_degrees(degrees);
    }

    pub fn set_rotation_radians(&mut self, radians: f32) {
        self.set_rotation_degrees(radians.to_degrees());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_its_size_from_the_image() {
        let image = DecodedImage {
            width: 640,
            height: 480,
            rgba: vec![0; 640 * 480 * 4],
        };

        let document = Document::sized_to(&image);

        assert_eq!(document.size(), Vec2::new(640.0, 480.0));
    }

    #[test]
    fn starts_unrotated() {
        assert_eq!(Document::new(Vec2::splat(10.0)).rotation_degrees(), 0.0);
    }

    #[test]
    fn normalizes_rotation() {
        let mut document = Document::new(Vec2::splat(10.0));

        document.set_rotation_degrees(-90.0);

        assert_eq!(document.rotation_degrees(), 270.0);
    }

    #[test]
    fn converts_rotation_between_degrees_and_radians() {
        let mut document = Document::new(Vec2::splat(10.0));

        document.set_rotation_radians(std::f32::consts::FRAC_PI_2);

        assert!((document.rotation_degrees() - 90.0).abs() < 1e-4);
        assert!((document.rotation_radians() - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }
}
