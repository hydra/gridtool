//! The Routes table: the document's routes, in document order. Each route is a folder row
//! with its colour, total length, unit and a delete button; it starts collapsed, and opened
//! it lists one row per segment with that segment's editable end coordinates and length.
//! The active route's row is selected. Clicking a route's row makes it active; clicking a
//! segment's row makes its route active and brings the whole segment into view.

use eframe::egui;
use egui_material_icons::MaterialIcon;
use gridtool::action::{Action, StyleTarget};
use gridtool::document::Document;
use gridtool::object::ObjectId;
use gridtool::route::Route;
use gridtool::style::{StyleEdit, Styled};
use gridtool::tr;

use super::color_swatch;
use super::commit_field::CommitField;
use super::coordinate_columns::{self, Axis, CoordinateColumns};
use super::format::{format_coordinate, format_length, parse_number};
use super::icon_button::inline_icon_text;
use super::icons;
use super::selection::Selection;
use super::table_rows::{TableOutput, TableRows, row_color};
use super::unit_picker;

const DELETE_ICON: MaterialIcon = icons::DELETE;

/// A row of the table: a route, or one segment of it (the segment from point `n` to `n + 1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RouteRow {
    Route(ObjectId),
    Segment(ObjectId, usize),
}

/// A segment of a route: the route's object, and which segment, counting from 0.
pub type SegmentPick = (ObjectId, usize);

/// What the user did in the Routes table this frame.
#[derive(Debug, Default, PartialEq)]
pub struct RoutesOutput {
    pub table: TableOutput,
    /// The segment whose row was clicked.
    pub segment: Option<SegmentPick>,
}

/// Which end of a segment: its first point or its second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SegmentEnd {
    Start,
    End,
}

impl SegmentEnd {
    const ALL: [SegmentEnd; 2] = [SegmentEnd::Start, SegmentEnd::End];

    /// The index of the route point at this end of segment `segment`.
    fn point(self, segment: usize) -> usize {
        match self {
            SegmentEnd::Start => segment,
            SegmentEnd::End => segment + 1,
        }
    }
}

fn column_headers() -> [String; 9] {
    let [x1, y1, x2, y2] = coordinate_columns::HEADERS.map(str::to_owned);
    [
        "#".to_owned(),
        tr!("column-color"),
        x1,
        y1,
        x2,
        y2,
        tr!("column-length"),
        tr!("column-units"),
        String::new(),
    ]
}

#[derive(Default)]
pub struct RoutesPanel {
    coordinate_fields: CommitField,
    columns: CoordinateColumns,
}

impl RoutesPanel {
    /// Draws the table into `ui`, returning what the user did. The row of the active route
    /// is selected and, if `reveal`, scrolled into view. Which routes are open is remembered
    /// in `ui`'s scope, so each document keeps its own.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        selection: &Selection,
        reveal: bool,
    ) -> RoutesOutput {
        if document.routes().is_empty() {
            ui.weak(tr!("routes-empty"));
            return RoutesOutput::default();
        }

        let active = selection.active();
        egui::ScrollArea::both()
            .id_salt("routes_scroll")
            .auto_shrink(false)
            .show(ui, |ui| {
                let field_width = self.columns.field_width(ui);
                let open: Vec<bool> = document
                    .routes()
                    .iter()
                    .map(|route| is_open(ui, route.id()))
                    .collect();
                let selected_row = active.and_then(|id| route_row_index(document, &open, id));
                let mut rows = TableRows::default();
                let headers = column_headers();
                let table = egui::Grid::new("routes_table")
                    .with_row_color(row_color(selected_row))
                    .num_columns(headers.len())
                    .show(ui, |ui| {
                        for header in headers {
                            ui.strong(header);
                        }
                        ui.end_row();

                        let mut action = None;
                        for (index, stored) in document.routes().iter().enumerate() {
                            let id = stored.id();
                            // Show a drag in progress live.
                            let route = selection.displayed_route(id, stored);
                            let row_action = rows.row(ui, RouteRow::Route(id), |ui| {
                                route_row(ui, document, (index, id), &route, open[index])
                            });
                            action = action.or(row_action);
                            if !open[index] {
                                continue;
                            }
                            let lengths = route.segment_lengths(document.calibration()).ok();
                            for segment in 0..route.segments().len() {
                                let length = lengths.as_ref().map(|lengths| lengths[segment]);
                                let row_action =
                                    rows.row(ui, RouteRow::Segment(id, segment), |ui| {
                                        self.segment_row(
                                            ui,
                                            document,
                                            (index, segment),
                                            &route,
                                            length,
                                            field_width,
                                        )
                                    });
                                action = action.or(row_action);
                            }
                        }
                        action
                    });
                self.columns
                    .laid_out(field_width, table.response.rect.width());
                let reveal = active.filter(|_| reveal).map(RouteRow::Route);
                let clicked = rows.finish(ui, table.response.rect, reveal);
                output(table.inner, clicked)
            })
            .inner
    }

    fn segment_row(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        (index, segment): (usize, usize),
        route: &Route,
        length: Option<f64>,
        field_width: f32,
    ) -> Option<Action> {
        // Every cell must be drawn each frame, so collect actions eagerly.
        let mut actions = Vec::new();

        ui.weak(format!("{}.{}", index + 1, segment + 1));
        ui.label("");

        for end in SegmentEnd::ALL {
            let point = end.point(segment);
            let displayed = route.points()[point];
            for axis in Axis::ALL {
                let id = egui::Id::unique(("route_coordinate", index, segment, end, axis));
                let (_, committed) = self.coordinate_fields.show(
                    ui,
                    id,
                    format_coordinate(axis.of(displayed)),
                    field_width,
                );
                actions.extend(
                    committed.and_then(|text| coordinate_edit(document, index, point, axis, &text)),
                );
            }
        }

        show_length(ui, length, document);
        ui.label("");
        ui.label("");

        actions.into_iter().next()
    }
}

/// The folder row of the route at `index`, object `id`: its open/close toggle and number, colour, total
/// length, unit and delete button.
fn route_row(
    ui: &mut egui::Ui,
    document: &Document,
    (index, id): (usize, ObjectId),
    route: &Route,
    open: bool,
) -> Option<Action> {
    // Every cell must be drawn each frame, so collect actions eagerly.
    let mut actions = Vec::new();

    let (icon, tooltip) = if open {
        (icons::COLLAPSE, tr!("route-collapse-tooltip"))
    } else {
        (icons::EXPAND, tr!("route-expand-tooltip"))
    };
    let toggle = egui::Button::new(format!("{} {}", icon.codepoint, index + 1)).frame(false);
    if ui.add(toggle).on_hover_text(tooltip).clicked() {
        set_open(ui, id, !open);
    }

    actions.extend(
        color_swatch::show(
            ui,
            egui::Id::unique(("route_color", index)),
            route.line().color,
            color_swatch::ROW_SWATCH_SIZE,
            &tr!("route-color-tooltip"),
        )
        .committed
        .map(|color| Action::SetStyle {
            target: StyleTarget::Route(index),
            edit: StyleEdit::LineColor(color),
        }),
    );

    for _ in 0..coordinate_columns::COORDINATE_COLUMNS {
        ui.label("");
    }

    show_length(ui, route.length(document.calibration()).ok(), document);

    actions.extend(
        unit_picker::show(
            ui,
            ("route_unit", index),
            route.unit(),
            &tr!("route-unit-tooltip"),
        )
        .map(|unit| Action::SetRouteUnit { index, unit }),
    );

    if ui
        .button(inline_icon_text(DELETE_ICON))
        .on_hover_text(tr!("route-delete-tooltip"))
        .clicked()
    {
        actions.push(Action::DeleteRoute { index });
    }

    actions.into_iter().next()
}

/// A length, or a dash explaining why there is none.
fn show_length(ui: &mut egui::Ui, length: Option<f64>, document: &Document) {
    match (length, document.calibration().scale()) {
        (Some(length), _) => {
            ui.label(format_length(length));
        }
        (None, Err(reason)) => {
            ui.weak("—")
                .on_hover_text(tr!("scale-problem", { reason: reason }));
        }
        (None, Ok(_)) => {
            ui.weak("—");
        }
    }
}

fn open_id(ui: &egui::Ui, route: ObjectId) -> egui::Id {
    ui.make_persistent_id(("route_open", route))
}

/// Whether the route's folder row is open. Routes start closed.
fn is_open(ui: &egui::Ui, route: ObjectId) -> bool {
    let id = open_id(ui, route);
    ui.data(|data| data.get_temp(id)).unwrap_or(false)
}

fn set_open(ui: &egui::Ui, route: ObjectId, open: bool) {
    let id = open_id(ui, route);
    ui.data_mut(|data| data.insert_temp(id, open));
}

/// The index, among the rows after the header, of the folder row of route `id`, given which
/// routes are `open`.
fn route_row_index(document: &Document, open: &[bool], id: ObjectId) -> Option<usize> {
    let target = document.route_index(id)?;
    let rows_before = document.routes()[..target]
        .iter()
        .zip(open)
        .map(|(route, &open)| 1 + if open { route.segments().len() } else { 0 })
        .sum();
    Some(rows_before)
}

/// What the user did, given the `action` they completed and the row they `clicked`, if
/// any: a route's row picks the route, and a segment's row picks the segment.
fn output(action: Option<Action>, clicked: Option<RouteRow>) -> RoutesOutput {
    let (route, segment) = match clicked {
        Some(RouteRow::Route(id)) => (Some(id), None),
        Some(RouteRow::Segment(id, segment)) => (None, Some((id, segment))),
        None => (None, None),
    };
    RoutesOutput {
        table: TableOutput::new(action, route),
        segment,
    }
}

/// The action for typing `text` into one coordinate of point `point` of the route at
/// `index`, or `None` if the text is not a number, changes nothing, or would put the point
/// outside the document.
fn coordinate_edit(
    document: &Document,
    index: usize,
    point: usize,
    axis: Axis,
    text: &str,
) -> Option<Action> {
    let current = *document.routes().get(index)?.points().get(point)?;
    let to = axis.with(current, parse_number(text)?);
    (document.contains(to) && to != current).then_some(Action::MoveRoutePoint { index, point, to })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::geometry::Point;
    use gridtool::units::LengthUnit;

    fn route(points: &[(f64, f64)]) -> Route {
        let points = points.iter().map(|&(x, y)| Point::new(x, y)).collect();
        Route::new(points, LengthUnit::Metre).unwrap()
    }

    /// Two routes: one with two segments, then one with one.
    fn document() -> Document {
        let mut document = Document::new(100, 50);
        for route in [
            route(&[(10.0, 10.0), (20.0, 10.0), (20.0, 30.0)]),
            route(&[(1.0, 1.0), (2.0, 2.0)]),
        ] {
            Action::AddRoute(route).apply(&mut document).unwrap();
        }
        document
    }

    #[test]
    fn the_columns_match_the_point_to_point_table_with_length_in_place_of_distance() {
        let headers = column_headers();

        assert_eq!(headers[1], "Color");
        assert_eq!(headers[2..6], ["x1", "y1", "x2", "y2"]);
        assert_eq!(headers[6], "Length");
    }

    #[test]
    fn the_end_of_a_segment_is_the_start_of_the_next() {
        assert_eq!(SegmentEnd::Start.point(1), 1);
        assert_eq!(SegmentEnd::End.point(1), 2);
        assert_eq!(SegmentEnd::End.point(0), SegmentEnd::Start.point(1));
    }

    #[test]
    fn editing_the_end_of_a_segment_moves_the_point_it_shares_with_the_next() {
        let document = document();
        let point = SegmentEnd::End.point(0);

        assert_eq!(
            coordinate_edit(&document, 0, point, Axis::Y, " 15 "),
            Some(Action::MoveRoutePoint {
                index: 0,
                point: 1,
                to: Point::new(20.0, 15.0),
            })
        );
    }

    #[test]
    fn invalid_unchanged_or_missing_coordinates_produce_no_action() {
        let document = document();

        for (index, point, axis, text) in [
            (0, 0, Axis::X, "abc"),
            (0, 0, Axis::X, "101"),
            (0, 2, Axis::Y, "-1"),
            (0, 0, Axis::X, "10"),
            (0, 3, Axis::X, "5"),
            (2, 0, Axis::X, "5"),
        ] {
            assert_eq!(
                coordinate_edit(&document, index, point, axis, text),
                None,
                "{index} {point} {text}"
            );
        }
    }

    #[test]
    fn the_selected_row_skips_the_segments_of_open_routes_before_it() {
        let document = document();
        let second = document.routes()[1].id();

        assert_eq!(route_row_index(&document, &[false, false], second), Some(1));
        assert_eq!(route_row_index(&document, &[true, false], second), Some(3));
        let first = document.routes()[0].id();
        assert_eq!(route_row_index(&document, &[true, true], first), Some(0));
    }

    #[test]
    fn a_route_row_picks_the_route_and_a_segment_row_picks_the_segment() {
        let id = document().routes()[0].id();

        assert_eq!(
            output(None, Some(RouteRow::Route(id))),
            RoutesOutput {
                table: TableOutput::new(None, Some(id)),
                segment: None,
            }
        );
        assert_eq!(
            output(None, Some(RouteRow::Segment(id, 1))),
            RoutesOutput {
                table: TableOutput::default(),
                segment: Some((id, 1)),
            }
        );
    }

    #[test]
    fn deleting_a_route_does_not_pick_it() {
        let id = document().routes()[0].id();

        let output = output(
            Some(Action::DeleteRoute { index: 0 }),
            Some(RouteRow::Route(id)),
        );

        assert_eq!(output.table.pick, None);
        assert_eq!(output.table.action, Some(Action::DeleteRoute { index: 0 }));
    }

    #[test]
    fn routes_start_closed_and_remember_being_opened() {
        let ctx = egui::Context::default();
        let id = document().routes()[0].id();
        let mut states = Vec::new();
        for open in [None, Some(true), None] {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                if let Some(open) = open {
                    set_open(ui, id, open);
                }
                states.push(is_open(ui, id));
            });
            output.textures_delta.clear();
        }

        assert_eq!(states, [false, true, true]);
    }
}
