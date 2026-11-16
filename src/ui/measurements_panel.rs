//! The Point-to-point table: the document's distance measurements, in document order,
//! with editable coordinates, per-measurement units and colours, and delete buttons. The
//! active measurement's row is selected, and clicking a row makes its measurement active.

use eframe::egui;
use egui_material_icons::MaterialIcon;
use gridtool::action::{Action, StyleTarget};
use gridtool::document::Document;
use gridtool::measurement::{Endpoint, Measurement};
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

fn column_headers() -> [String; 9] {
    let [x1, y1, x2, y2] = coordinate_columns::HEADERS.map(str::to_owned);
    [
        "#".to_owned(),
        tr!("column-color"),
        x1,
        y1,
        x2,
        y2,
        tr!("column-distance"),
        tr!("column-units"),
        String::new(),
    ]
}

#[derive(Default)]
pub struct MeasurementsPanel {
    coordinate_fields: CommitField,
    columns: CoordinateColumns,
}

impl MeasurementsPanel {
    /// Draws the table into `ui`, returning what the user did. The row of the active object
    /// is selected and, if `reveal`, scrolled into view.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        selection: &Selection,
        reveal: bool,
    ) -> TableOutput {
        if document.measurements().is_empty() {
            ui.weak(tr!("measurements-empty"));
            return TableOutput::default();
        }

        let active = selection.active();
        let selected_row = active.and_then(|id| document.measurement_index(id));
        egui::ScrollArea::both()
            .id_salt("measurements_scroll")
            .auto_shrink(false)
            .show(ui, |ui| {
                let field_width = self.columns.field_width(ui);
                let mut rows = TableRows::default();
                let headers = column_headers();
                let table = egui::Grid::new("measurements_table")
                    .with_row_color(row_color(selected_row))
                    .num_columns(headers.len())
                    .show(ui, |ui| {
                        for header in headers {
                            ui.strong(header);
                        }
                        ui.end_row();

                        let mut action = None;
                        for (index, stored) in document.measurements().iter().enumerate() {
                            // Show a resize in progress live.
                            let measurement = selection.displayed(stored.id(), stored);
                            let row_action = rows.row(ui, stored.id(), |ui| {
                                self.row(ui, document, index, &measurement, field_width)
                            });
                            action = action.or(row_action);
                        }
                        action
                    });
                self.columns
                    .laid_out(field_width, table.response.rect.width());
                let pick = rows.finish(ui, table.response.rect, active.filter(|_| reveal));
                TableOutput::new(table.inner, pick)
            })
            .inner
    }

    fn row(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        index: usize,
        measurement: &Measurement,
        field_width: f32,
    ) -> Option<Action> {
        // Every cell must be drawn each frame, so collect actions eagerly.
        let mut actions = Vec::new();

        ui.label((index + 1).to_string());

        actions.extend(
            color_swatch::show(
                ui,
                egui::Id::unique(("measurement_color", index)),
                measurement.line().color,
                color_swatch::ROW_SWATCH_SIZE,
                &tr!("measurement-color-tooltip"),
            )
            .committed
            .map(|color| Action::SetStyle {
                target: StyleTarget::Measurement(index),
                edit: StyleEdit::LineColor(color),
            }),
        );

        for endpoint in Endpoint::ALL {
            let displayed = measurement.point(endpoint);
            for axis in Axis::ALL {
                let id = egui::Id::unique(("measurement_coordinate", index, endpoint, axis));
                let (_, committed) = self.coordinate_fields.show(
                    ui,
                    id,
                    format_coordinate(axis.of(displayed)),
                    field_width,
                );
                actions.extend(
                    committed
                        .and_then(|text| coordinate_edit(document, index, endpoint, axis, &text)),
                );
            }
        }

        match measurement.distance(document.calibration()) {
            Ok(distance) => {
                ui.label(format_length(distance));
            }
            Err(reason) => {
                ui.weak("—")
                    .on_hover_text(tr!("scale-problem", { reason: reason }));
            }
        }

        actions.extend(
            unit_picker::show(
                ui,
                ("measurement_unit", index),
                measurement.unit(),
                &tr!("measurement-unit-tooltip"),
            )
            .map(|unit| Action::SetMeasurementUnit { index, unit }),
        );

        if ui
            .button(inline_icon_text(DELETE_ICON))
            .on_hover_text(tr!("measurement-delete-tooltip"))
            .clicked()
        {
            actions.push(Action::DeleteMeasurement { index });
        }

        actions.into_iter().next()
    }
}

/// The action for typing `text` into one coordinate of a measurement endpoint, or `None`
/// if the text is not a number or would put the point outside the document.
fn coordinate_edit(
    document: &Document,
    index: usize,
    endpoint: Endpoint,
    axis: Axis,
    text: &str,
) -> Option<Action> {
    let current = document.measurements().get(index)?.point(endpoint);
    let to = axis.with(current, parse_number(text)?);
    (document.contains(to) && to != current).then_some(Action::MoveMeasurementPoint {
        index,
        endpoint,
        to,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::geometry::Point;
    use gridtool::units::LengthUnit;

    fn document() -> Document {
        let mut document = Document::new(100, 50);
        Action::AddMeasurement(Measurement::new(
            Point::new(10.0, 20.0),
            Point::new(30.0, 40.0),
            LengthUnit::Centimetre,
        ))
        .apply(&mut document)
        .unwrap();
        document
    }

    #[test]
    fn the_colour_is_in_the_second_column() {
        assert_eq!(column_headers()[1], "Color");
    }

    #[test]
    fn a_wide_table_settles_to_a_steady_layout_when_drawn_by_egui() {
        let ctx = egui::Context::default();
        let mut panel = MeasurementsPanel::default();
        let document = document();
        let selection = Selection::default();
        let mut layouts = Vec::new();
        for _ in 0..30 {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900.0, 400.0),
                )),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                panel.show(ui, &document, &selection, false);
            });
            output.textures_delta.clear();
            layouts.push(panel.columns.layout().unwrap());
        }

        let last = layouts[layouts.len() - 1];
        assert!(
            layouts[20..].iter().all(|layout| *layout == last),
            "{layouts:?}"
        );
        assert!(
            CoordinateColumns::field_width_of(last) > min_field_width_in(&ctx),
            "{last:?}"
        );
    }

    fn min_field_width_in(ctx: &egui::Context) -> f32 {
        let mut width = 0.0;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            width = coordinate_columns::min_field_width(ui);
        });
        output.textures_delta.clear();
        width
    }

    #[test]
    fn editing_one_coordinate_keeps_the_other() {
        let action = coordinate_edit(&document(), 0, Endpoint::End, Axis::Y, " 45.5 ");

        assert_eq!(
            action,
            Some(Action::MoveMeasurementPoint {
                index: 0,
                endpoint: Endpoint::End,
                to: Point::new(30.0, 45.5)
            })
        );
    }

    #[test]
    fn invalid_or_out_of_bounds_coordinates_produce_no_action() {
        let document = document();

        assert_eq!(
            coordinate_edit(&document, 0, Endpoint::Start, Axis::X, "abc"),
            None
        );
        assert_eq!(
            coordinate_edit(&document, 0, Endpoint::Start, Axis::X, "101"),
            None
        );
        assert_eq!(
            coordinate_edit(&document, 0, Endpoint::Start, Axis::Y, "-1"),
            None
        );
    }

    #[test]
    fn unchanged_or_missing_measurements_produce_no_action() {
        let document = document();

        assert_eq!(
            coordinate_edit(&document, 0, Endpoint::Start, Axis::X, "10"),
            None
        );
        assert_eq!(
            coordinate_edit(&document, 1, Endpoint::Start, Axis::X, "5"),
            None
        );
    }
}
