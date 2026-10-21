use crate::calibration::Calibration;
use crate::geometry::{Bounds, Point};
use crate::grid::Grid;
use crate::imaging::DecodedImage;
use crate::measurement::Measurement;
use crate::object::{Identified, ObjectId};

/// The thing being edited. Modified only through [`crate::action::Action`]s.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    width: u32,
    height: u32,
    calibration: Calibration,
    measurements: Vec<Identified<Measurement>>,
    grids: Vec<Identified<Grid>>,
    /// The image's origin: the point the view rotates about. Always inside the document.
    origin: Point,
    /// The id the next added object will get. Ids are never reused within a history.
    next_object_id: u64,
}

impl Document {
    /// Creates a document of `width` × `height` pixels, with its origin at the centre.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            calibration: Calibration::default(),
            measurements: Vec::new(),
            grids: Vec::new(),
            origin: Point::new(f64::from(width) / 2.0, f64::from(height) / 2.0),
            next_object_id: 1,
        }
    }

    /// The image's origin, in image coordinates.
    pub fn origin(&self) -> Point {
        self.origin
    }

    pub(crate) fn set_origin(&mut self, origin: Point) {
        self.origin = origin;
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

    /// The document rectangle, from the origin to (width, height).
    pub fn bounds(&self) -> Bounds {
        Bounds::around(
            Point::new(0.0, 0.0),
            Point::new(f64::from(self.width), f64::from(self.height)),
        )
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

    /// Distance measurements, in the order they were added.
    pub fn measurements(&self) -> &[Identified<Measurement>] {
        &self.measurements
    }

    /// List position of the measurement with `id`, if it exists.
    pub fn measurement_index(&self, id: ObjectId) -> Option<usize> {
        self.measurements.iter().position(|m| m.id() == id)
    }

    /// Grids, in the order they were added.
    pub fn grids(&self) -> &[Identified<Grid>] {
        &self.grids
    }

    /// List position of the grid with `id`, if it exists.
    pub fn grid_index(&self, id: ObjectId) -> Option<usize> {
        self.grids.iter().position(|grid| grid.id() == id)
    }

    /// Whether any object with `id` exists.
    pub fn contains_object(&self, id: ObjectId) -> bool {
        self.measurement_index(id).is_some() || self.grid_index(id).is_some()
    }

    /// The image point to bring into view for an object: the centre of a measurement's
    /// bounding box, or the midpoint of a grid's alignment line. `None` if the object
    /// (or a grid's alignment line) does not exist.
    pub fn object_focus_point(&self, id: ObjectId) -> Option<Point> {
        if let Some(index) = self.measurement_index(id) {
            return Some(self.measurements[index].bounds().center());
        }
        let grid = &self.grids[self.grid_index(id)?];
        let (start, end) = grid.source_line(self).ok()?;
        Some(Bounds::around(start, end).center())
    }

    /// The most recently added object, if it still exists.
    pub fn newest_object(&self) -> Option<ObjectId> {
        let id = ObjectId::new(self.next_object_id.checked_sub(1)?);
        self.contains_object(id).then_some(id)
    }

    pub(crate) fn measurements_mut(&mut self) -> &mut Vec<Identified<Measurement>> {
        &mut self.measurements
    }

    pub(crate) fn grids_mut(&mut self) -> &mut Vec<Identified<Grid>> {
        &mut self.grids
    }

    /// The id the next added object will get.
    pub(crate) fn next_object_id(&self) -> u64 {
        self.next_object_id
    }

    /// Rebuilds a document from its parts, e.g. when loading a saved file. The caller
    /// is responsible for the parts being consistent.
    pub(crate) fn from_parts(
        width: u32,
        height: u32,
        calibration: Calibration,
        measurements: Vec<Identified<Measurement>>,
        grids: Vec<Identified<Grid>>,
        origin: Point,
        next_object_id: u64,
    ) -> Self {
        Self {
            width,
            height,
            calibration,
            measurements,
            grids,
            origin,
            next_object_id,
        }
    }

    pub(crate) fn allocate_object_id(&mut self) -> ObjectId {
        let id = ObjectId::new(self.next_object_id);
        self.next_object_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::units::LengthUnit;

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
    fn the_origin_defaults_to_the_centre() {
        assert_eq!(Document::new(100, 50).origin(), Point::new(50.0, 25.0));
        assert_eq!(Document::new(5, 3).origin(), Point::new(2.5, 1.5));
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

    fn add_measurement(document: &mut Document) {
        Action::AddMeasurement(Measurement::new(
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            LengthUnit::Centimetre,
        ))
        .apply(document)
        .unwrap();
    }

    #[test]
    fn objects_get_distinct_ids_that_survive_deleting_others() {
        let mut document = Document::new(10, 10);
        for _ in 0..3 {
            add_measurement(&mut document);
        }
        let ids: Vec<_> = document.measurements().iter().map(|m| m.id()).collect();
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);

        Action::DeleteMeasurement { index: 0 }
            .apply(&mut document)
            .unwrap();

        assert_eq!(document.measurement_index(ids[2]), Some(1));
        assert!(!document.contains_object(ids[0]));
    }

    #[test]
    fn the_newest_object_is_the_last_added_while_it_exists() {
        let mut document = Document::new(10, 10);
        assert_eq!(document.newest_object(), None);

        add_measurement(&mut document);
        add_measurement(&mut document);
        assert_eq!(
            document.newest_object(),
            Some(document.measurements()[1].id())
        );

        Action::DeleteMeasurement { index: 1 }
            .apply(&mut document)
            .unwrap();
        assert_eq!(document.newest_object(), None);
    }

    #[test]
    fn ids_are_not_reused_after_a_delete() {
        let mut document = Document::new(10, 10);
        add_measurement(&mut document);
        let first = document.measurements()[0].id();
        Action::DeleteMeasurement { index: 0 }
            .apply(&mut document)
            .unwrap();

        add_measurement(&mut document);

        assert_ne!(document.measurements()[0].id(), first);
    }
}
