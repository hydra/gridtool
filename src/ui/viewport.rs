//! How the document is viewed: pan offset, zoom level and view rotation. None of this
//! is part of the document itself.

use eframe::egui::emath::Rot2;
use eframe::egui::{Pos2, Rect, Vec2};
use gridtool::geometry::{Bounds, Point};

use super::angle::normalize_degrees;

const MIN_ZOOM: f32 = 0.02;
const MAX_ZOOM: f32 = 1024.0;

/// User-controlled camera over the document. The document's center is drawn at
/// `view_center + pan`, scaled by `zoom` and rotated clockwise by the view rotation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pan: Vec2,
    zoom: f32,
    rotation_degrees: f32,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            rotation_degrees: 0.0,
        }
    }
}

impl Viewport {
    /// Clockwise view rotation, in degrees within `[0, 360)`.
    pub fn rotation_degrees(&self) -> f32 {
        self.rotation_degrees
    }

    pub fn rotation_radians(&self) -> f32 {
        self.rotation_degrees.to_radians()
    }

    /// Sets the view rotation, pivoting about the document's centre.
    pub fn set_rotation_degrees(&mut self, degrees: f32) {
        self.rotation_degrees = normalize_degrees(degrees);
    }

    #[cfg(test)]
    pub fn set_rotation_radians(&mut self, radians: f32) {
        self.set_rotation_degrees(radians.to_degrees());
    }

    /// Sets the view rotation, pivoting about image point `pivot` (e.g. the document's
    /// origin): `pivot` stays at the same screen position.
    pub fn rotate_about(&mut self, degrees: f32, pivot: Point, document_size: Vec2) {
        let before = Rot2::from_angle(self.rotation_radians());
        self.set_rotation_degrees(degrees);
        let after = Rot2::from_angle(self.rotation_radians());
        // The pivot is drawn at `pan + R · (pivot − size/2) · zoom` from the view centre;
        // adjust the pan so that stays the same under the new rotation.
        let from_center =
            (Vec2::new(pivot.x as f32, pivot.y as f32) - document_size * 0.5) * self.zoom;
        self.pan += before * from_center - after * from_center;
    }

    /// Screen points per image pixel.
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Moves the document by `delta` screen points.
    pub fn pan_by(&mut self, delta: Vec2) {
        self.pan += delta;
    }

    /// Pans so that image `point` is drawn at the centre of the view, keeping zoom and
    /// rotation.
    pub fn center_on(&mut self, point: Point, document_size: Vec2) {
        let from_center = Vec2::new(point.x as f32, point.y as f32) - document_size * 0.5;
        self.pan = -(Rot2::from_angle(self.rotation_radians()) * (from_center * self.zoom));
    }

    /// Multiplies the zoom by `factor`, keeping the document point under `anchor`
    /// (a screen position) fixed. The zoom is clamped to a sensible range.
    pub fn zoom_about(&mut self, view_center: Pos2, anchor: Pos2, factor: f32) {
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let anchor_from_document_center = anchor - view_center - self.pan;
        self.pan = anchor - view_center - anchor_from_document_center * (new_zoom / self.zoom);
        self.zoom = new_zoom;
    }

    /// The image-to-screen mapping for this frame, for a document of `document_size` pixels.
    pub fn transform(&self, view_center: Pos2, document_size: Vec2) -> DocumentTransform {
        DocumentTransform {
            center: view_center + self.pan,
            half_size: document_size * 0.5,
            zoom: self.zoom,
            rotation: Rot2::from_angle(self.rotation_radians()),
        }
    }
}

/// Maps between image pixel coordinates (origin top-left of the un-rotated image) and
/// screen positions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DocumentTransform {
    center: Pos2,
    half_size: Vec2,
    zoom: f32,
    rotation: Rot2,
}

impl DocumentTransform {
    /// Screen position of the document's center.
    #[cfg(test)]
    pub fn center(self) -> Pos2 {
        self.center
    }

    pub fn image_to_screen(self, point: Point) -> Pos2 {
        let from_center = Vec2::new(point.x as f32, point.y as f32) - self.half_size;
        self.center_offset_to_screen(from_center)
    }

    pub fn screen_to_image(self, position: Pos2) -> Point {
        let from_center = self.rotation.inverse() * (position - self.center) / self.zoom;
        let image = from_center + self.half_size;
        Point::new(f64::from(image.x), f64::from(image.y))
    }

    /// Screen directions (unit vectors) of the image's x and y axes.
    pub fn image_axes(self) -> [Vec2; 2] {
        [self.rotation * Vec2::X, self.rotation * Vec2::Y]
    }

    /// Screen positions of the document's corners, in the order top-left, top-right,
    /// bottom-right, bottom-left.
    pub fn corners(self) -> [Pos2; 4] {
        self.corners_outset(0.0)
    }

    /// Like [`Self::corners`], but pushed outward by `outset` screen points on every side.
    pub fn corners_outset(self, outset: f32) -> [Pos2; 4] {
        let size = self.half_size * 2.0;
        let document = Bounds {
            min: Point::new(0.0, 0.0),
            max: Point::new(f64::from(size.x), f64::from(size.y)),
        };
        self.bounds_corners(document, outset)
    }

    /// Screen positions of the corners of image-space `bounds`, pushed outward by `outset`
    /// screen points on every side, in the order top-left, top-right, bottom-right,
    /// bottom-left (in image orientation).
    pub fn bounds_corners(self, bounds: Bounds, outset: f32) -> [Pos2; 4] {
        let Bounds { min, max } = bounds.expanded(self.screen_to_image_length(outset));
        [
            Point::new(min.x, min.y),
            Point::new(max.x, min.y),
            Point::new(max.x, max.y),
            Point::new(min.x, max.y),
        ]
        .map(|corner| self.image_to_screen(corner))
    }

    /// The image-pixel length that `length` screen points cover at this zoom.
    pub fn screen_to_image_length(self, length: f32) -> f64 {
        f64::from(length / self.zoom)
    }

    /// Screen points per image pixel.
    pub fn zoom(self) -> f32 {
        self.zoom
    }

    /// The smallest image-space box containing everything drawn inside screen `rect`
    /// (larger than `rect` itself when the view is rotated).
    pub fn image_bounds_of(self, rect: Rect) -> Bounds {
        let [first, rest @ ..] = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ]
        .map(|corner| self.screen_to_image(corner));
        rest.into_iter()
            .fold(Bounds::around(first, first), |bounds, p| Bounds {
                min: Point::new(bounds.min.x.min(p.x), bounds.min.y.min(p.y)),
                max: Point::new(bounds.max.x.max(p.x), bounds.max.y.max(p.y)),
            })
    }

    fn center_offset_to_screen(self, from_center: Vec2) -> Pos2 {
        self.center + self.rotation * (from_center * self.zoom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW_CENTER: Pos2 = Pos2::new(400.0, 300.0);
    const DOCUMENT_SIZE: Vec2 = Vec2::new(100.0, 50.0);

    fn assert_near(actual: Pos2, expected: Pos2) {
        assert!(
            (actual - expected).length() < 1e-3,
            "expected {expected:?}, got {actual:?}"
        );
    }

    fn transform(viewport: &Viewport) -> DocumentTransform {
        viewport.transform(VIEW_CENTER, DOCUMENT_SIZE)
    }

    #[test]
    fn center_on_puts_the_point_at_the_view_centre_at_any_zoom_and_rotation() {
        let mut viewport = Viewport::default();
        viewport.set_rotation_degrees(30.0);
        viewport.zoom_about(VIEW_CENTER, Pos2::new(10.0, 20.0), 2.5);
        viewport.pan_by(Vec2::new(123.0, -45.0));
        let point = Point::new(80.0, 10.0);

        viewport.center_on(point, DOCUMENT_SIZE);

        assert_near(transform(&viewport).image_to_screen(point), VIEW_CENTER);
        assert_eq!(viewport.zoom, 2.5);
        assert!((viewport.rotation_degrees() - 30.0).abs() < 1e-4);
    }

    #[test]
    fn bounds_corners_are_outset_in_screen_points() {
        let mut viewport = Viewport::default();
        viewport.zoom_about(VIEW_CENTER, VIEW_CENTER, 2.0);
        let bounds = Bounds::around(Point::new(10.0, 20.0), Point::new(30.0, 20.0));

        let corners = transform(&viewport).bounds_corners(bounds, 8.0);

        // Image (10, 20) is 40 left of and 5 above the document centre; zoom doubles it.
        assert_near(
            corners[0],
            Pos2::new(400.0 - 80.0 - 8.0, 300.0 - 10.0 - 8.0),
        );
        assert_near(
            corners[2],
            Pos2::new(400.0 - 40.0 + 8.0, 300.0 - 10.0 + 8.0),
        );
    }

    #[test]
    fn image_bounds_of_a_screen_rect_follow_zoom_and_rotation() {
        let mut viewport = Viewport::default();
        viewport.zoom_about(VIEW_CENTER, VIEW_CENTER, 2.0);
        let rect = Rect::from_center_size(VIEW_CENTER, Vec2::new(40.0, 20.0));

        // Document centre (50, 25) is at the view centre; 40 × 20 screen is 20 × 10 image.
        let bounds = transform(&viewport).image_bounds_of(rect);
        assert!((bounds.min.x - 40.0).abs() < 1e-3 && (bounds.max.x - 60.0).abs() < 1e-3);
        assert!((bounds.min.y - 20.0).abs() < 1e-3 && (bounds.max.y - 30.0).abs() < 1e-3);

        // Rotated a quarter turn, the box swaps its width and height.
        viewport.set_rotation_degrees(90.0);
        let bounds = transform(&viewport).image_bounds_of(rect);
        assert!((bounds.min.x - 45.0).abs() < 1e-3 && (bounds.max.x - 55.0).abs() < 1e-3);
        assert!((bounds.min.y - 15.0).abs() < 1e-3 && (bounds.max.y - 35.0).abs() < 1e-3);
    }

    #[test]
    fn default_centers_document_at_actual_size() {
        let transform = transform(&Viewport::default());

        assert_eq!(transform.center(), VIEW_CENTER);
        assert_near(
            transform.image_to_screen(Point::new(0.0, 0.0)),
            Pos2::new(350.0, 275.0),
        );
        assert_near(
            transform.image_to_screen(Point::new(60.0, 30.0)),
            Pos2::new(410.0, 305.0),
        );
    }

    #[test]
    fn panning_moves_the_document() {
        let mut viewport = Viewport::default();

        viewport.pan_by(Vec2::new(15.0, -20.0));

        assert_eq!(transform(&viewport).center(), Pos2::new(415.0, 280.0));
    }

    #[test]
    fn zooming_keeps_the_point_under_the_anchor_fixed() {
        let mut viewport = Viewport::default();
        viewport.pan_by(Vec2::new(30.0, 10.0));
        let anchor = Pos2::new(500.0, 200.0);
        let image_point = transform(&viewport).screen_to_image(anchor);

        viewport.zoom_about(VIEW_CENTER, anchor, 2.5);

        assert_near(transform(&viewport).image_to_screen(image_point), anchor);
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
    fn rotation_is_normalized() {
        let mut viewport = Viewport::default();

        viewport.set_rotation_degrees(-90.0);
        assert_eq!(viewport.rotation_degrees(), 270.0);

        viewport.set_rotation_radians(std::f32::consts::PI);
        assert!((viewport.rotation_degrees() - 180.0).abs() < 1e-4);
    }

    #[test]
    fn rotating_about_a_pivot_keeps_it_in_place() {
        let mut viewport = Viewport::default();
        viewport.zoom_about(VIEW_CENTER, Pos2::new(700.0, 100.0), 8.0);
        viewport.pan_by(Vec2::new(-30.0, 12.0));
        let pivot = Point::new(80.0, 10.0);
        let before = transform(&viewport).image_to_screen(pivot);

        for degrees in [90.0, 33.0, 300.0] {
            viewport.rotate_about(degrees, pivot, DOCUMENT_SIZE);

            assert_near(transform(&viewport).image_to_screen(pivot), before);
            assert!((viewport.rotation_degrees() - degrees).abs() < 1e-3);
        }
    }

    #[test]
    fn corners_are_scaled_by_zoom() {
        let mut viewport = Viewport::default();
        viewport.zoom_about(VIEW_CENTER, VIEW_CENTER, 2.0);

        let corners = transform(&viewport).corners();

        assert_near(corners[0], Pos2::new(300.0, 250.0));
        assert_near(corners[2], Pos2::new(500.0, 350.0));
    }

    #[test]
    fn view_rotation_turns_the_document_clockwise_about_its_center() {
        let mut viewport = Viewport::default();
        viewport.set_rotation_degrees(90.0);
        let transform = transform(&viewport);

        // A quarter turn clockwise moves the top-left corner to the top-right.
        assert_near(transform.corners()[0], Pos2::new(425.0, 250.0));
        assert_near(transform.image_axes()[0].to_pos2(), Pos2::new(0.0, 1.0));
    }

    #[test]
    fn screen_to_image_inverts_image_to_screen() {
        let mut viewport = Viewport::default();
        viewport.set_rotation_degrees(33.0);
        viewport.zoom_about(VIEW_CENTER, Pos2::new(123.0, 456.0), 1.7);
        viewport.pan_by(Vec2::new(-12.0, 34.0));
        let transform = transform(&viewport);
        let point = Point::new(12.5, 40.0);

        let round_trip = transform.screen_to_image(transform.image_to_screen(point));

        assert!((round_trip.x - point.x).abs() < 1e-3);
        assert!((round_trip.y - point.y).abs() < 1e-3);
    }

    #[test]
    fn outset_is_measured_in_screen_points_regardless_of_zoom() {
        let mut viewport = Viewport::default();
        viewport.zoom_about(VIEW_CENTER, VIEW_CENTER, 4.0);

        let corners = transform(&viewport).corners_outset(3.0);

        assert_near(
            corners[0],
            Pos2::new(400.0 - 200.0 - 3.0, 300.0 - 100.0 - 3.0),
        );
    }
}
