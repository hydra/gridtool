//! The active object, and dragging its points or bounding-box handles.
//!
//! Selection is presentation state: it is not part of the document and is not undoable.
//! A drag produces a document [`Action`] when it ends.

use gridtool::action::Action;
use gridtool::area::Area;
use gridtool::document::Document;
use gridtool::geometry::{Bounds, Point};
use gridtool::grid::Grid;
use gridtool::measurement::Measurement;
use gridtool::object::ObjectId;

use super::object_drag::{Grip, ObjectDrag, Shape};

#[derive(Debug, Default)]
pub struct Selection {
    active: Option<ObjectId>,
    drag: Option<ObjectDrag>,
    /// Uncommitted style edits (colour, opacity) of the active grid, shown live while
    /// the user is still choosing.
    grid_preview: Option<Grid>,
}

impl Selection {
    pub fn active(&self) -> Option<ObjectId> {
        self.active
    }

    /// Makes `id` the active object (or clears it with `None`), abandoning any drag or
    /// preview.
    pub fn select(&mut self, id: Option<ObjectId>) {
        if self.active != id {
            self.drag = None;
            self.grid_preview = None;
        }
        self.active = id;
    }

    /// Re-aligns with the document after it changed (an action, undo or redo): drops a
    /// selection whose object no longer exists and abandons any drag or preview.
    pub fn sync_to(&mut self, document: &Document) {
        if self.active.is_some_and(|id| !document.contains_object(id)) {
            self.active = None;
        }
        self.drag = None;
        self.grid_preview = None;
    }

    /// Abandons a drag in progress, without committing anything.
    pub fn cancel_drag(&mut self) {
        self.drag = None;
    }

    /// How to display the grid with `id`: the preview if it is the active grid and has
    /// uncommitted edits, otherwise the grid itself.
    pub fn displayed_grid(&self, id: ObjectId, grid: &Grid) -> Grid {
        match &self.grid_preview {
            Some(preview) if self.active == Some(id) => preview.clone(),
            _ => grid.clone(),
        }
    }

    /// Shows `grid` in place of the active grid until the preview is cleared.
    pub fn preview_grid(&mut self, grid: Grid) {
        if self.active.is_some() {
            self.grid_preview = Some(grid);
        }
    }

    pub fn clear_grid_preview(&mut self) {
        self.grid_preview = None;
    }

    /// The drag preview of the object with `id`, if it is being dragged.
    fn previewed(&self, id: ObjectId) -> Option<&Shape> {
        self.drag
            .as_ref()
            .filter(|_| self.active == Some(id))
            .map(ObjectDrag::preview)
    }

    /// How to display the measurement with `id`: the drag preview if it is being
    /// dragged, otherwise the measurement itself.
    pub fn displayed(&self, id: ObjectId, measurement: &Measurement) -> Measurement {
        match self.previewed(id) {
            Some(Shape::Measurement(preview)) => preview.clone(),
            _ => measurement.clone(),
        }
    }

    /// How to display the area with `id`: the drag preview if it is being dragged,
    /// otherwise the area itself.
    pub fn displayed_area(&self, id: ObjectId, area: &Area) -> Area {
        match self.previewed(id) {
            Some(Shape::Area(preview)) => preview.clone(),
            _ => area.clone(),
        }
    }

    /// The active object's shape as currently displayed, if it has one (measurements and
    /// areas do; grids do not).
    fn active_shape(&self, document: &Document) -> Option<Shape> {
        let id = self.active?;
        match self.previewed(id) {
            Some(preview) => Some(preview.clone()),
            None => Shape::of(document, id),
        }
    }

    /// Bounding box of the active object as currently displayed.
    pub fn active_bounds(&self, document: &Document) -> Option<Bounds> {
        self.active_shape(document).map(|shape| shape.bounds())
    }

    /// The active object's draggable points as currently displayed; empty if it has none.
    pub fn active_points(&self, document: &Document) -> Vec<Point> {
        self.active_shape(document)
            .map(|shape| shape.points())
            .unwrap_or_default()
    }

    /// Starts dragging `grip` of the active object, with the pointer pressed at image
    /// point `press`. Returns `false` if there is no active object with a shape.
    pub fn begin_drag(&mut self, document: &Document, grip: Grip, press: Point) -> bool {
        let Some(shape) = self.active.and_then(|id| Shape::of(document, id)) else {
            return false;
        };
        self.drag = Some(ObjectDrag::new(shape, grip, press));
        true
    }

    pub fn is_dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Updates the preview for the pointer at image point `pointer`. Points stay inside
    /// the document.
    pub fn drag_to(&mut self, document: &Document, pointer: Point) {
        if let Some(drag) = &mut self.drag {
            drag.drag_to(document, pointer);
        }
    }

    /// Finishes the drag, returning the action if the object actually changed.
    pub fn end_drag(&mut self, document: &Document) -> Option<Action> {
        let drag = self.drag.take()?;
        drag.finish(document, self.active?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::geometry::BoundsHandle;
    use gridtool::units::LengthUnit;

    fn document_with_two_measurements() -> Document {
        let mut document = Document::new(100, 100);
        for (start, end) in [((10.0, 10.0), (40.0, 30.0)), ((50.0, 50.0), (60.0, 90.0))] {
            Action::AddMeasurement(Measurement::new(
                Point::new(start.0, start.1),
                Point::new(end.0, end.1),
                LengthUnit::Centimetre,
            ))
            .apply(&mut document)
            .unwrap();
        }
        document
    }

    fn id_of(document: &Document, index: usize) -> ObjectId {
        document.measurements()[index].id()
    }

    #[test]
    fn starts_with_nothing_selected() {
        let selection = Selection::default();

        assert_eq!(selection.active(), None);
        assert_eq!(
            selection.active_bounds(&document_with_two_measurements()),
            None
        );
    }

    #[test]
    fn active_bounds_follow_the_selected_measurement() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();

        selection.select(Some(id_of(&document, 1)));

        assert_eq!(
            selection.active_bounds(&document),
            Some(Bounds::around(
                Point::new(50.0, 50.0),
                Point::new(60.0, 90.0)
            ))
        );
    }

    #[test]
    fn an_active_area_has_bounds_and_points_that_follow_a_drag() {
        let mut document = document_with_two_measurements();
        Action::AddArea(
            Area::new(
                vec![
                    Point::new(10.0, 50.0),
                    Point::new(30.0, 50.0),
                    Point::new(20.0, 70.0),
                ],
                gridtool::units::LengthUnit::Metre,
            )
            .unwrap(),
        )
        .apply(&mut document)
        .unwrap();
        let stored = &document.areas()[0];
        let mut selection = Selection::default();
        selection.select(Some(stored.id()));
        assert_eq!(selection.active_points(&document), stored.points());

        assert!(selection.begin_drag(&document, Grip::Point(1), Point::new(30.0, 50.0)));
        selection.drag_to(&document, Point::new(40.0, 45.0));

        assert_eq!(
            selection.displayed_area(stored.id(), stored).points()[1],
            Point::new(40.0, 45.0)
        );
        assert_eq!(
            selection.active_points(&document)[1],
            Point::new(40.0, 45.0)
        );
        assert_eq!(
            selection.active_bounds(&document),
            Some(Bounds::around(
                Point::new(10.0, 45.0),
                Point::new(40.0, 70.0)
            ))
        );
        assert_eq!(
            selection.end_drag(&document),
            Some(Action::MoveAreaPoint {
                index: 0,
                point: 1,
                to: Point::new(40.0, 45.0),
            })
        );
    }

    #[test]
    fn cancelling_a_resize_commits_nothing_and_keeps_the_selection() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        let id = id_of(&document, 0);
        selection.select(Some(id));
        selection.begin_drag(
            &document,
            Grip::Bounds(BoundsHandle::BottomRight),
            Point::new(40.0, 30.0),
        );
        selection.drag_to(&document, Point::new(45.0, 38.0));

        selection.cancel_drag();

        assert!(!selection.is_dragging());
        assert_eq!(selection.end_drag(&document), None);
        let measurement = &document.measurements()[0];
        assert_eq!(selection.displayed(id, measurement), **measurement);
        assert_eq!(selection.active(), Some(id));
    }

    #[test]
    fn resizing_previews_and_ends_with_one_action() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        let id = id_of(&document, 0);
        selection.select(Some(id));

        assert!(selection.begin_drag(
            &document,
            Grip::Bounds(BoundsHandle::BottomRight),
            Point::new(40.0, 30.0)
        ));
        selection.drag_to(&document, Point::new(45.0, 38.0));

        let preview = selection.displayed(id, &document.measurements()[0]);
        assert_eq!(preview.end(), Point::new(45.0, 38.0));
        assert_eq!(
            selection.end_drag(&document),
            Some(Action::ResizeMeasurement {
                index: 0,
                start: Point::new(10.0, 10.0),
                end: Point::new(45.0, 38.0),
            })
        );
        assert!(!selection.is_dragging());
    }

    #[test]
    fn resizing_keeps_endpoints_inside_the_document() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        selection.select(Some(id_of(&document, 0)));
        selection.begin_drag(
            &document,
            Grip::Bounds(BoundsHandle::Left),
            Point::new(10.0, 20.0),
        );

        selection.drag_to(&document, Point::new(-500.0, 20.0));

        let Some(Action::ResizeMeasurement { start, .. }) = selection.end_drag(&document) else {
            panic!("expected a resize action");
        };
        assert_eq!(start, Point::new(0.0, 10.0));
    }

    #[test]
    fn a_resize_that_changes_nothing_produces_no_action() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        selection.select(Some(id_of(&document, 0)));
        selection.begin_drag(
            &document,
            Grip::Bounds(BoundsHandle::Top),
            Point::new(25.0, 10.0),
        );

        assert_eq!(selection.end_drag(&document), None);
    }

    #[test]
    fn cannot_resize_without_an_active_object() {
        let mut selection = Selection::default();

        assert!(!selection.begin_drag(
            &document_with_two_measurements(),
            Grip::Bounds(BoundsHandle::Top),
            Point::default()
        ));
    }

    #[test]
    fn only_the_active_measurement_shows_the_preview() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        selection.select(Some(id_of(&document, 0)));
        selection.begin_drag(
            &document,
            Grip::Bounds(BoundsHandle::Right),
            Point::new(40.0, 20.0),
        );
        selection.drag_to(&document, Point::new(70.0, 20.0));

        let other = &document.measurements()[1];
        assert_eq!(selection.displayed(other.id(), other), **other);
    }

    #[test]
    fn sync_clears_a_deleted_selection_and_any_resize() {
        let mut document = document_with_two_measurements();
        let mut selection = Selection::default();
        let id = id_of(&document, 1);
        selection.select(Some(id));
        selection.begin_drag(&document, Grip::Bounds(BoundsHandle::Top), Point::default());

        selection.sync_to(&document);
        assert_eq!(selection.active(), Some(id));
        assert!(!selection.is_dragging());

        Action::DeleteMeasurement { index: 1 }
            .apply(&mut document)
            .unwrap();
        selection.sync_to(&document);
        assert_eq!(selection.active(), None);
    }

    #[test]
    fn selection_follows_the_object_when_earlier_ones_are_deleted() {
        let mut document = document_with_two_measurements();
        let mut selection = Selection::default();
        let id = id_of(&document, 1);
        selection.select(Some(id));

        Action::DeleteMeasurement { index: 0 }
            .apply(&mut document)
            .unwrap();
        selection.sync_to(&document);

        assert_eq!(selection.active(), Some(id));
        assert_eq!(
            selection.active_bounds(&document).map(|b| b.min),
            Some(Point::new(50.0, 50.0))
        );
    }

    fn document_with_grid() -> (Document, ObjectId) {
        use gridtool::grid::GridSource;

        let mut document = document_with_two_measurements();
        let source = GridSource::Measurement(id_of(&document, 0));
        Action::AddGrid(Grid::new(source, LengthUnit::Centimetre))
            .apply(&mut document)
            .unwrap();
        let id = document.grids()[0].id();
        (document, id)
    }

    #[test]
    fn grids_have_no_selection_box() {
        let (document, id) = document_with_grid();
        let mut selection = Selection::default();

        selection.select(Some(id));

        assert_eq!(selection.active_bounds(&document), None);
        assert!(!selection.begin_drag(
            &document,
            Grip::Bounds(BoundsHandle::Top),
            Point::default()
        ));
    }

    #[test]
    fn grid_preview_shows_only_for_the_active_grid_until_cleared() {
        use gridtool::color::Rgb;

        let (document, id) = document_with_grid();
        let stored = &document.grids()[0];
        let mut selection = Selection::default();
        selection.select(Some(id));

        selection.preview_grid(stored.with_color(Rgb::new(1, 2, 3)));
        assert_eq!(
            selection.displayed_grid(id, stored).color(),
            Rgb::new(1, 2, 3)
        );

        selection.clear_grid_preview();
        assert_eq!(selection.displayed_grid(id, stored), **stored);
    }

    #[test]
    fn changing_or_syncing_the_selection_drops_the_grid_preview() {
        let (document, id) = document_with_grid();
        let stored = &document.grids()[0];
        let mut selection = Selection::default();
        selection.select(Some(id));
        selection.preview_grid(stored.with_opacity(0.1));

        selection.sync_to(&document);
        assert_eq!(
            selection.displayed_grid(id, stored).opacity(),
            stored.opacity()
        );

        selection.preview_grid(stored.with_opacity(0.1));
        selection.select(None);
        selection.select(Some(id));
        assert_eq!(
            selection.displayed_grid(id, stored).opacity(),
            stored.opacity()
        );
    }
}
