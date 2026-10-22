//! The active object and resizing it through its bounding-box handles.
//!
//! Selection is presentation state: it is not part of the document and is not undoable.
//! Resizing produces a document [`Action`] when the drag ends.

use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::geometry::{Bounds, BoundsHandle, Point};
use gridtool::grid::Grid;
use gridtool::measurement::Measurement;
use gridtool::object::ObjectId;

#[derive(Debug, Clone, PartialEq)]
struct Resize {
    handle: BoundsHandle,
    press: Point,
    original: Measurement,
    preview: Measurement,
}

#[derive(Debug, Default)]
pub struct Selection {
    active: Option<ObjectId>,
    resize: Option<Resize>,
    /// Uncommitted style edits (colour, opacity) of the active grid, shown live while
    /// the user is still choosing.
    grid_preview: Option<Grid>,
}

impl Selection {
    pub fn active(&self) -> Option<ObjectId> {
        self.active
    }

    /// Makes `id` the active object (or clears it with `None`), abandoning any resize or
    /// preview.
    pub fn select(&mut self, id: Option<ObjectId>) {
        if self.active != id {
            self.resize = None;
            self.grid_preview = None;
        }
        self.active = id;
    }

    /// Re-aligns with the document after it changed (an action, undo or redo): drops a
    /// selection whose object no longer exists and abandons any resize or preview.
    pub fn sync_to(&mut self, document: &Document) {
        if self.active.is_some_and(|id| !document.contains_object(id)) {
            self.active = None;
        }
        self.resize = None;
        self.grid_preview = None;
    }

    /// Abandons a resize in progress, without committing anything.
    pub fn cancel_resize(&mut self) {
        self.resize = None;
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

    /// How to display the measurement with `id`: the resize preview if it is being
    /// resized, otherwise the measurement itself.
    pub fn displayed(&self, id: ObjectId, measurement: &Measurement) -> Measurement {
        match &self.resize {
            Some(resize) if self.active == Some(id) => resize.preview.clone(),
            _ => measurement.clone(),
        }
    }

    /// Bounding box of the active object as currently displayed.
    pub fn active_bounds(&self, document: &Document) -> Option<Bounds> {
        let (id, measurement) = self.active_measurement(document)?;
        Some(self.displayed(id, measurement).bounds())
    }

    /// Starts resizing the active object by `handle`, with the pointer pressed at image
    /// point `press`. Returns `false` if there is no active object.
    pub fn begin_resize(
        &mut self,
        document: &Document,
        handle: BoundsHandle,
        press: Point,
    ) -> bool {
        let Some((_, measurement)) = self.active_measurement(document) else {
            return false;
        };
        self.resize = Some(Resize {
            handle,
            press,
            original: measurement.clone(),
            preview: measurement.clone(),
        });
        true
    }

    pub fn is_resizing(&self) -> bool {
        self.resize.is_some()
    }

    /// Updates the preview for the pointer at image point `pointer`. The grabbed edges
    /// move by the pointer's movement since the press; endpoints stay inside the document.
    pub fn resize_to(&mut self, document: &Document, pointer: Point) {
        if let Some(resize) = &mut self.resize {
            let resized = resize.original.resized(
                resize.handle,
                pointer.x - resize.press.x,
                pointer.y - resize.press.y,
            );
            resize.preview = Measurement::new(
                document.clamp(resized.start()),
                document.clamp(resized.end()),
                resized.unit(),
            );
        }
    }

    /// Finishes the resize, returning the action if the measurement actually changed.
    pub fn end_resize(&mut self, document: &Document) -> Option<Action> {
        let resize = self.resize.take()?;
        let index = document.measurement_index(self.active?)?;
        (resize.preview != resize.original).then(|| Action::ResizeMeasurement {
            index,
            start: resize.preview.start(),
            end: resize.preview.end(),
        })
    }

    fn active_measurement<'a>(
        &self,
        document: &'a Document,
    ) -> Option<(ObjectId, &'a Measurement)> {
        let id = self.active?;
        let index = document.measurement_index(id)?;
        Some((id, &document.measurements()[index]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn cancelling_a_resize_commits_nothing_and_keeps_the_selection() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        let id = id_of(&document, 0);
        selection.select(Some(id));
        selection.begin_resize(&document, BoundsHandle::BottomRight, Point::new(40.0, 30.0));
        selection.resize_to(&document, Point::new(45.0, 38.0));

        selection.cancel_resize();

        assert!(!selection.is_resizing());
        assert_eq!(selection.end_resize(&document), None);
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

        assert!(selection.begin_resize(
            &document,
            BoundsHandle::BottomRight,
            Point::new(40.0, 30.0)
        ));
        selection.resize_to(&document, Point::new(45.0, 38.0));

        let preview = selection.displayed(id, &document.measurements()[0]);
        assert_eq!(preview.end(), Point::new(45.0, 38.0));
        assert_eq!(
            selection.end_resize(&document),
            Some(Action::ResizeMeasurement {
                index: 0,
                start: Point::new(10.0, 10.0),
                end: Point::new(45.0, 38.0),
            })
        );
        assert!(!selection.is_resizing());
    }

    #[test]
    fn resizing_keeps_endpoints_inside_the_document() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        selection.select(Some(id_of(&document, 0)));
        selection.begin_resize(&document, BoundsHandle::Left, Point::new(10.0, 20.0));

        selection.resize_to(&document, Point::new(-500.0, 20.0));

        let Some(Action::ResizeMeasurement { start, .. }) = selection.end_resize(&document) else {
            panic!("expected a resize action");
        };
        assert_eq!(start, Point::new(0.0, 10.0));
    }

    #[test]
    fn a_resize_that_changes_nothing_produces_no_action() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        selection.select(Some(id_of(&document, 0)));
        selection.begin_resize(&document, BoundsHandle::Top, Point::new(25.0, 10.0));

        assert_eq!(selection.end_resize(&document), None);
    }

    #[test]
    fn cannot_resize_without_an_active_object() {
        let mut selection = Selection::default();

        assert!(!selection.begin_resize(
            &document_with_two_measurements(),
            BoundsHandle::Top,
            Point::default()
        ));
    }

    #[test]
    fn only_the_active_measurement_shows_the_preview() {
        let document = document_with_two_measurements();
        let mut selection = Selection::default();
        selection.select(Some(id_of(&document, 0)));
        selection.begin_resize(&document, BoundsHandle::Right, Point::new(40.0, 20.0));
        selection.resize_to(&document, Point::new(70.0, 20.0));

        let other = &document.measurements()[1];
        assert_eq!(selection.displayed(other.id(), other), **other);
    }

    #[test]
    fn sync_clears_a_deleted_selection_and_any_resize() {
        let mut document = document_with_two_measurements();
        let mut selection = Selection::default();
        let id = id_of(&document, 1);
        selection.select(Some(id));
        selection.begin_resize(&document, BoundsHandle::Top, Point::default());

        selection.sync_to(&document);
        assert_eq!(selection.active(), Some(id));
        assert!(!selection.is_resizing());

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
        assert!(!selection.begin_resize(&document, BoundsHandle::Top, Point::default()));
    }

    #[test]
    fn grid_preview_shows_only_for_the_active_grid_until_cleared() {
        use gridtool::grid::Rgb;

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
