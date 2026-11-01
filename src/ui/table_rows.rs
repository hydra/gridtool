//! Rows of a table that each stand for an object. A row is selected exactly when its object
//! is the active object, so at most one row is selected across every table.

use eframe::egui::{self, Color32, Rangef, Rect, Style};
use gridtool::action::Action;
use gridtool::object::ObjectId;

/// What the user did in a table this frame.
#[derive(Debug, Default, PartialEq)]
pub struct TableOutput {
    /// An action the user completed.
    pub action: Option<Action>,
    /// The object of a row the user clicked.
    pub pick: Option<ObjectId>,
}

impl TableOutput {
    /// A click on a row's delete button deletes that row's object, so does not pick it.
    pub fn new(action: Option<Action>, pick: Option<ObjectId>) -> Self {
        let deleting = matches!(
            action,
            Some(Action::DeleteMeasurement { .. } | Action::DeleteArea { .. })
        );
        Self {
            action,
            pick: pick.filter(|_| !deleting),
        }
    }
}

/// The colours of a table's rows, including its header row: striped, with `selected`, an index
/// into the rows after the header, standing out.
pub fn row_color(
    selected: Option<usize>,
) -> impl Fn(usize, &Style) -> Option<Color32> + Send + Sync + 'static {
    move |row, style| {
        if selected.is_some_and(|selected| row == selected + 1) {
            Some(style.visuals.selection.bg_fill)
        } else {
            (row % 2 == 1).then_some(style.visuals.faint_bg_color)
        }
    }
}

/// Where the rows of a table were laid out this frame.
#[derive(Debug, Default)]
pub struct TableRows {
    rows: Vec<(ObjectId, Rangef)>,
}

impl TableRows {
    /// Shows one row of a grid for the object `id` and ends the row.
    pub fn row<R>(
        &mut self,
        ui: &mut egui::Ui,
        id: ObjectId,
        show: impl FnOnce(&mut egui::Ui) -> R,
    ) -> R {
        let top = ui.cursor().top();
        let inner = show(ui);
        ui.end_row();
        self.rows.push((id, Rangef::new(top, ui.cursor().top())));
        inner
    }

    /// Call once the grid is drawn, inside its scroll area, with the grid's `table` rect.
    /// Scrolls the row of `reveal` into view, and returns the object of the row clicked this
    /// frame. A click on a widget in a row counts as a click on the row, but one on something
    /// drawn over the table, such as a drop-down, does not.
    pub fn finish(
        &self,
        ui: &mut egui::Ui,
        table: Rect,
        reveal: Option<ObjectId>,
    ) -> Option<ObjectId> {
        if let Some(&(_, y)) = self.rows.iter().find(|(id, _)| Some(*id) == reveal) {
            // The whole visible width, so that this cannot scroll sideways.
            ui.scroll_to_rect(Rect::from_x_y_ranges(ui.clip_rect().x_range(), y), None);
        }
        if !ui.input(|input| input.pointer.primary_clicked()) {
            return None;
        }
        self.rows
            .iter()
            .find(|(_, y)| ui.rect_contains_pointer(Rect::from_x_y_ranges(table.x_range(), *y)))
            .map(|&(id, _)| id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Modifiers, PointerButton, Pos2, RawInput, Vec2};
    use gridtool::document::Document;
    use gridtool::geometry::Point;
    use gridtool::measurement::Measurement;
    use gridtool::units::LengthUnit;

    const ROW_COUNT: usize = 40;

    fn ids() -> Vec<ObjectId> {
        let mut document = Document::new(100, 100);
        for _ in 0..ROW_COUNT {
            Action::AddMeasurement(Measurement::new(
                Point::new(1.0, 1.0),
                Point::new(2.0, 2.0),
                LengthUnit::Centimetre,
            ))
            .apply(&mut document)
            .unwrap();
        }
        document.measurements().iter().map(|m| m.id()).collect()
    }

    struct Frame {
        pick: Option<ObjectId>,
        offset: f32,
    }

    /// One frame of a table too tall for its 200 point high window.
    fn frame(
        ctx: &egui::Context,
        ids: &[ObjectId],
        reveal: Option<ObjectId>,
        events: Vec<Event>,
    ) -> Frame {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 200.0))),
            events,
            ..Default::default()
        };
        let mut result = None;
        let mut output = ctx.run_ui(input, |ui| {
            let scroll = egui::ScrollArea::vertical().show(ui, |ui| {
                let mut rows = TableRows::default();
                let table = egui::Grid::new("table").show(ui, |ui| {
                    for (index, &id) in ids.iter().enumerate() {
                        rows.row(ui, id, |ui| ui.label(index.to_string()));
                    }
                });
                rows.finish(ui, table.response.rect, reveal)
            });
            result = Some(Frame {
                pick: scroll.inner,
                offset: scroll.state.clamped_offset().y,
            });
        });
        output.textures_delta.clear();
        result.unwrap()
    }

    fn click_events(pos: Pos2) -> [Vec<Event>; 3] {
        let button = |pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        [
            vec![Event::PointerMoved(pos)],
            vec![button(true)],
            vec![button(false)],
        ]
    }

    #[test]
    fn a_row_out_of_view_is_scrolled_to_and_one_in_view_is_not() {
        let ctx = egui::Context::default();
        let ids = ids();
        assert_eq!(frame(&ctx, &ids, None, Vec::new()).offset, 0.0);

        frame(&ctx, &ids, Some(ids[0]), Vec::new());
        assert_eq!(frame(&ctx, &ids, None, Vec::new()).offset, 0.0);

        frame(&ctx, &ids, Some(ids[ROW_COUNT - 1]), Vec::new());
        // Animated, so give it time to arrive.
        let mut settled = 0.0;
        for _ in 0..200 {
            settled = frame(&ctx, &ids, None, Vec::new()).offset;
        }
        assert!(settled > 100.0, "{settled}");
    }

    #[test]
    fn clicking_a_row_picks_its_object() {
        let ctx = egui::Context::default();
        let ids = ids();
        frame(&ctx, &ids, None, Vec::new());

        let mut picks = Vec::new();
        for events in click_events(Pos2::new(5.0, 12.0)) {
            picks.extend(frame(&ctx, &ids, None, events).pick);
        }

        assert_eq!(picks, [ids[0]]);
    }

    #[test]
    fn a_click_outside_the_rows_picks_nothing() {
        let ctx = egui::Context::default();
        let ids = ids();
        frame(&ctx, &ids, None, Vec::new());

        let mut picks = Vec::new();
        // Beyond the right of the table.
        for events in click_events(Pos2::new(290.0, 12.0)) {
            picks.extend(frame(&ctx, &ids, None, events).pick);
        }

        assert_eq!(picks, []);
    }

    #[test]
    fn only_the_selected_row_stands_out_and_the_rest_are_striped() {
        let style = Style::default();
        let color = row_color(Some(1));

        assert_eq!(color(0, &style), None);
        assert_eq!(color(1, &style), Some(style.visuals.faint_bg_color));
        assert_eq!(color(2, &style), Some(style.visuals.selection.bg_fill));
        assert_eq!(color(3, &style), Some(style.visuals.faint_bg_color));
        assert_eq!(row_color(None)(2, &style), None);
    }

    #[test]
    fn deleting_a_row_does_not_also_pick_it() {
        let id = ids()[0];

        let edit = Action::SetMeasurementUnit {
            index: 0,
            unit: LengthUnit::Metre,
        };
        assert_eq!(TableOutput::new(Some(edit), Some(id)).pick, Some(id));
        let delete = Action::DeleteMeasurement { index: 0 };
        assert_eq!(TableOutput::new(Some(delete), Some(id)).pick, None);
        let delete = Action::DeleteArea { index: 0 };
        assert_eq!(TableOutput::new(Some(delete), Some(id)).pick, None);
    }
}
