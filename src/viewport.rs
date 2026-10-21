//! Where the document sits on screen: pan offset and zoom level.

use eframe::egui::emath::Rot2;
use eframe::egui::{Pos2, Vec2};

const MIN_ZOOM: f32 = 0.02;
const MAX_ZOOM: f32 = 64.0;

/// User-controlled camera over the document. The document's center is drawn at
/// `view_center + pan`, scaled by `zoom`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pan: Vec2,
    zoom: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

impl Viewport {
    /// Moves the document by `delta` screen points.
    pub fn pan_by(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    /// Multiplies the zoom by `factor`, keeping the document point under `anchor`
    /// (a screen position) fixed. The zoom is clamped to a sensible range.
    pub fn zoom_about(&mut self, view_center: Pos2, anchor: Pos2, factor: f32) {
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let anchor_from_document_center = anchor - view_center - self.pan;
        self.pan = anchor - view_center - anchor_from_document_center * (new_zoom / self.zoom);
        self.zoom = new_zoom;
    }

    /// The document-to-screen mapping for this frame.
    pub fn transform(&self, view_center: Pos2, rotation_radians: f32) -> DocumentTransform {
        DocumentTransform {
            center: view_center + self.pan,
            zoom: self.zoom,
            rotation: Rot2::from_angle(rotation_radians),
        }
    }
}

/// Maps document coordinates (pixels, origin at the document's center) to screen positions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DocumentTransform {
    center: Pos2,
    zoom: f32,
    rotation: Rot2,
}

impl DocumentTransform {
    /// Screen position of the document's center.
    pub fn center(self) -> Pos2 {
        self.center
    }

    pub fn to_screen(self, document_point: Vec2) -> Pos2 {
        self.center + self.rotation * (document_point * self.zoom)
    }

    /// Screen positions of the corners of a `size` document, in the order top-left,
    /// top-right, bottom-right, bottom-left.
    pub fn corners(self, size: Vec2) -> [Pos2; 4] {
        self.corners_outset(size, 0.0)
    }

    /// Like [`Self::corners`], but pushed outward by `outset` screen points on every side.
    pub fn corners_outset(self, size: Vec2, outset: f32) -> [Pos2; 4] {
        let half = size * 0.5 + Vec2::splat(outset / self.zoom);
        [
            Vec2::new(-half.x, -half.y),
            Vec2::new(half.x, -half.y),
            Vec2::new(half.x, half.y),
            Vec2::new(-half.x, half.y),
        ]
        .map(|corner| self.to_screen(corner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    const VIEW_CENTER: Pos2 = Pos2::new(400.0, 300.0);

    fn assert_near(actual: Pos2, expected: Pos2) {
        assert!(
            (actual - expected).length() < 1e-3,
            "expected {expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn default_centers_document_at_actual_size() {
        let transform = Viewport::default().transform(VIEW_CENTER, 0.0);

        assert_eq!(transform.center(), VIEW_CENTER);
        assert_near(transform.to_screen(Vec2::new(10.0, 5.0)), Pos2::new(410.0, 305.0));
    }

    #[test]
    fn panning_moves_the_document() {
        let mut viewport = Viewport::default();

        viewport.pan_by(Vec2::new(15.0, -20.0));

        assert_eq!(
            viewport.transform(VIEW_CENTER, 0.0).center(),
            Pos2::new(415.0, 280.0)
        );
    }

    #[test]
    fn zooming_keeps_the_point_under_the_anchor_fixed() {
        let mut viewport = Viewport::default();
        viewport.pan_by(Vec2::new(30.0, 10.0));
        let anchor = Pos2::new(500.0, 200.0);
        let before = viewport.transform(VIEW_CENTER, 0.0);
        let document_point = (anchor - before.center()) / before.zoom;

        viewport.zoom_about(VIEW_CENTER, anchor, 2.5);

        assert_near(viewport.transform(VIEW_CENTER, 0.0).to_screen(document_point), anchor);
        assert_eq!(viewport.zoom, 2.5);
    }

    #[test]
    fn zoom_is_clamped() {
        let mut viewport = Viewport::default();

        viewport.zoom_about(VIEW_CENTER, VIEW_CENTER, 1e6);
        assert_eq!(viewport.zoom, MAX_ZOOM);

        viewport.zoom_about(VIEW_CENTER, VIEW_CENTER, 1e-12);
        assert_eq!(viewport.zoom, MIN_ZOOM);
    }

    #[test]
    fn corners_are_scaled_by_zoom() {
        let mut viewport = Viewport::default();
        viewport.zoom_about(VIEW_CENTER, VIEW_CENTER, 2.0);

        let corners = viewport.transform(VIEW_CENTER, 0.0).corners(Vec2::new(100.0, 50.0));

        assert_near(corners[0], Pos2::new(300.0, 250.0));
        assert_near(corners[2], Pos2::new(500.0, 350.0));
    }

    #[test]
    fn corners_rotate_clockwise_about_the_center() {
        let corners = Viewport::default()
            .transform(VIEW_CENTER, FRAC_PI_2)
            .corners(Vec2::new(100.0, 50.0));

        // A quarter turn clockwise moves the top-left corner to the top-right.
        assert_near(corners[0], Pos2::new(425.0, 250.0));
    }

    #[test]
    fn outset_is_measured_in_screen_points_regardless_of_zoom() {
        let mut viewport = Viewport::default();
        viewport.zoom_about(VIEW_CENTER, VIEW_CENTER, 4.0);

        let corners = viewport
            .transform(VIEW_CENTER, 0.0)
            .corners_outset(Vec2::new(10.0, 10.0), 3.0);

        assert_near(corners[0], Pos2::new(400.0 - 20.0 - 3.0, 300.0 - 20.0 - 3.0));
    }
}
