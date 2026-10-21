use crate::calibration::Calibration;
use crate::geometry::Point;
use crate::imaging::DecodedImage;

/// The thing being edited. Modified only through [`crate::action::Action`]s.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    width: u32,
    height: u32,
    calibration: Calibration,
}

impl Document {
    /// Creates a document of `width` × `height` pixels.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            calibration: Calibration::default(),
        }
    }

    /// Creates a document the same size as `image`.
    pub fn sized_to(image: &DecodedImage) -> Self {
        Self::new(image.width, image.height)
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    /// Whether `point` lies within the document (edges included).
    pub fn contains(&self, point: Point) -> bool {
        (0.0..=f64::from(self.width)).contains(&point.x)
            && (0.0..=f64::from(self.height)).contains(&point.y)
    }

    /// The nearest point to `point` that lies within the document.
    pub fn clamp(&self, point: Point) -> Point {
        Point::new(
            point.x.clamp(0.0, f64::from(self.width)),
            point.y.clamp(0.0, f64::from(self.height)),
        )
    }

    pub fn calibration(&self) -> &Calibration {
        &self.calibration
    }

    pub(crate) fn calibration_mut(&mut self) -> &mut Calibration {
        &mut self.calibration
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

        assert_eq!((document.width(), document.height()), (640, 480));
    }

    #[test]
    fn contains_points_inside_and_on_the_edges() {
        let document = Document::new(100, 50);

        assert!(document.contains(Point::new(0.0, 0.0)));
        assert!(document.contains(Point::new(100.0, 50.0)));
        assert!(document.contains(Point::new(42.5, 10.0)));
        assert!(!document.contains(Point::new(-0.1, 10.0)));
        assert!(!document.contains(Point::new(10.0, 50.1)));
    }

    #[test]
    fn clamps_points_to_its_bounds() {
        let document = Document::new(100, 50);

        assert_eq!(
            document.clamp(Point::new(-5.0, 70.0)),
            Point::new(0.0, 50.0)
        );
        assert_eq!(
            document.clamp(Point::new(20.0, 30.0)),
            Point::new(20.0, 30.0)
        );
    }
}
