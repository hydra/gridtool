//! The Point-to-point table: the document's distance measurements, in document order,
//! with editable coordinates, per-measurement units and colours, and delete buttons.

use eframe::egui;
use egui_material_icons::MaterialIcon;
use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::geometry::Point;
use gridtool::measurement::{Endpoint, Measurement};

use super::color_swatch;
use super::commit_field::CommitField;
use super::format::{format_coordinate, format_length, parse_number};
use super::icon_button::inline_icon_text;
use super::icons;
use super::selection::Selection;
use super::unit_picker;

const COORDINATE_FIELD_WIDTH: f32 = 55.0;
const DELETE_ICON: MaterialIcon = icons::DELETE;
const COLUMN_HEADERS: [&str; 7] = ["#", "x1, y1", "x2, y2", "Distance", "Units", "Color", ""];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Axis {
    X,
    Y,
}

impl Axis {
    const ALL: [Axis; 2] = [Axis::X, Axis::Y];

    fn of(self, point: Point) -> f64 {
        match self {
            Axis::X => point.x,
            Axis::Y => point.y,
        }
    }

    fn with(self, point: Point, value: f64) -> Point {
        match self {
            Axis::X => Point::new(value, point.y),
            Axis::Y => Point::new(point.x, value),
        }
    }
}

/// The panel's title.
pub const HEADING: &str = "Point-to-point";

#[derive(Default)]
pub struct MeasurementsPanel {
    coordinate_fields: CommitField,
}

impl MeasurementsPanel {
    /// Draws the table into `ui`, returning the action the user completed, if any.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        selection: &Selection,
    ) -> Option<Action> {
        ui.heading(HEADING);
        ui.separator();
        if document.measurements().is_empty() {
            ui.weak("No measurements yet. Use the Measure distance tool to add one.");
            return None;
        }

        egui::ScrollArea::both()
            .id_salt("measurements_scroll")
            .auto_shrink(false)
            .show(ui, |ui| {
                egui::Grid::new("measurements_table")
                    .striped(true)
                    .num_columns(COLUMN_HEADERS.len())
                    .show(ui, |ui| {
                        for header in COLUMN_HEADERS {
                            ui.strong(header);
                        }
                        ui.end_row();

                        let mut action = None;
                        for (index, stored) in document.measurements().iter().enumerate() {
                            // Show a resize in progress live.
                            let measurement = selection.displayed(stored.id(), stored);
                            let row_action = self.row(ui, document, index, &measurement);
                            action = action.or(row_action);
                            ui.end_row();
                        }
                        action
                    })
                    .inner
            })
            .inner
    }

    fn row(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        index: usize,
        measurement: &Measurement,
    ) -> Option<Action> {
        // Every cell must be drawn each frame, so collect actions eagerly.
        let mut actions = Vec::new();

        ui.label((index + 1).to_string());

        for endpoint in Endpoint::ALL {
            let displayed = measurement.point(endpoint);
            ui.horizontal(|ui| {
                for axis in Axis::ALL {
                    let id = egui::Id::unique(("measurement_coordinate", index, endpoint, axis));
                    let (_, committed) = self.coordinate_fields.show(
                        ui,
                        id,
                        format_coordinate(axis.of(displayed)),
                        COORDINATE_FIELD_WIDTH,
                    );
                    actions.extend(
                        committed.and_then(|text| {
                            coordinate_edit(document, index, endpoint, axis, &text)
                        }),
                    );
                }
            });
        }

        match measurement.distance(document.calibration()) {
            Ok(distance) => {
                ui.label(format_length(distance));
            }
            Err(reason) => {
                ui.weak("—").on_hover_text(format!("Scale: {reason}"));
            }
        }

        actions.extend(
            unit_picker::show(
                ui,
                ("measurement_unit", index),
                measurement.unit(),
                "Units of this measurement",
            )
            .map(|unit| Action::SetMeasurementUnit { index, unit }),
        );

        actions.extend(
            color_swatch::show(
                ui,
                egui::Id::unique(("measurement_color", index)),
                measurement.color(),
                color_swatch::ROW_SWATCH_SIZE,
                "Choose the colour of this measurement",
            )
            .committed
            .map(|color| Action::SetMeasurementColor { index, color }),
        );

        if ui
            .button(inline_icon_text(DELETE_ICON))
            .on_hover_text("Delete measurement")
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
