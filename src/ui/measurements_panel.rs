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

/// The widest coordinate a field must show in full: six digits and a decimal point.
const WIDEST_COORDINATE: &str = "0000.00";
const DELETE_ICON: MaterialIcon = icons::DELETE;
const COLUMN_HEADERS: [&str; 9] = [
    "#", "x1", "y1", "x2", "y2", "Distance", "Units", "Color", "",
];
/// One coordinate column per axis of each endpoint.
const COORDINATE_COLUMNS: usize = 4;

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

/// Spare (or missing) table width smaller than this, in points, is left alone, so that
/// pixel rounding cannot keep the fields changing width.
const WIDTH_TOLERANCE: f32 = 2.0;

/// How wide the table was laid out last frame.
#[derive(Debug, Clone, Copy, PartialEq)]
struct TableLayout {
    field_width: f32,
    table_width: f32,
    /// Whether the frame before used the same field width. The grid places its columns
    /// by the widths of the frame before, so only then does `table_width` reflect
    /// `field_width`.
    settled: bool,
}

#[derive(Default)]
pub struct MeasurementsPanel {
    coordinate_fields: CommitField,
    layout: Option<TableLayout>,
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
                let available = ui.available_width();
                let min_width = min_field_width(ui);
                let field_width = coordinate_field_width(self.layout, available, min_width);
                let table = egui::Grid::new("measurements_table")
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
                            let row_action =
                                self.row(ui, document, index, &measurement, field_width);
                            action = action.or(row_action);
                            ui.end_row();
                        }
                        action
                    });
                let settled = self
                    .layout
                    .is_some_and(|last| last.field_width == field_width);
                self.layout = Some(TableLayout {
                    field_width,
                    table_width: table.response.rect.width(),
                    settled,
                });
                table.inner
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

/// The narrowest a coordinate field may be: wide enough to show [`WIDEST_COORDINATE`]
/// inside the field's padding.
fn min_field_width(ui: &egui::Ui) -> f32 {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let text = ui
        .painter()
        .layout_no_wrap(
            WIDEST_COORDINATE.to_owned(),
            font,
            egui::Color32::PLACEHOLDER,
        )
        .size()
        .x;
    // The theme pads a text field by the button padding on each side; allow for the
    // cursor too.
    text + ui.spacing().button_padding.x * 2.0 + 2.0
}

/// The width of each coordinate field this frame. Before the first layout, fields start at
/// `min_width`. Once the last layout has settled, the space the table left unused (or
/// overran by) is shared equally between the coordinate columns, but no field is narrower
/// than `min_width`. While it has not settled, or the difference is within
/// [`WIDTH_TOLERANCE`], the width is kept, so the fields never oscillate.
fn coordinate_field_width(last: Option<TableLayout>, available: f32, min_width: f32) -> f32 {
    let Some(last) = last else {
        return min_width;
    };
    let spare = available - last.table_width;
    if !last.settled || spare.abs() < WIDTH_TOLERANCE {
        return last.field_width.max(min_width);
    }
    (last.field_width + spare / COORDINATE_COLUMNS as f32).max(min_width)
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
    fn coordinate_fields_start_at_the_minimum_width() {
        assert_eq!(coordinate_field_width(None, 500.0, 60.0), 60.0);
    }

    #[test]
    fn coordinate_fields_share_spare_width_and_shrink_back_to_the_minimum() {
        let last = TableLayout {
            field_width: 60.0,
            table_width: 400.0,
            settled: true,
        };

        // 100 points spare, shared by four columns.
        assert_eq!(coordinate_field_width(Some(last), 500.0, 60.0), 85.0);
        // 40 points too wide: each field gives up 10.
        let wide = TableLayout {
            field_width: 85.0,
            table_width: 500.0,
            settled: true,
        };
        assert_eq!(coordinate_field_width(Some(wide), 460.0, 60.0), 75.0);
        // Never narrower than the minimum, even if the table overflows.
        assert_eq!(coordinate_field_width(Some(wide), 200.0, 60.0), 60.0);
    }

    #[test]
    fn coordinate_fields_keep_their_width_until_the_layout_settles_or_within_tolerance() {
        let unsettled = TableLayout {
            field_width: 60.0,
            table_width: 400.0,
            settled: false,
        };
        assert_eq!(coordinate_field_width(Some(unsettled), 500.0, 60.0), 60.0);

        let settled = TableLayout {
            settled: true,
            ..unsettled
        };
        assert_eq!(coordinate_field_width(Some(settled), 401.5, 60.0), 60.0);
        assert_eq!(coordinate_field_width(Some(settled), 398.5, 60.0), 60.0);
    }

    /// Models the grid placing its columns by the previous frame's widths, as egui does.
    #[test]
    fn coordinate_fields_settle_without_oscillating_when_the_grid_lags_a_frame() {
        const OTHER_COLUMNS: f32 = 200.0;
        const AVAILABLE: f32 = 523.0;
        let mut layout = None;
        let mut laid_out_width = 60.0;
        let mut widths = Vec::new();
        for _ in 0..12 {
            let field_width = coordinate_field_width(layout, AVAILABLE, 60.0);
            let table_width = OTHER_COLUMNS + COORDINATE_COLUMNS as f32 * laid_out_width;
            let settled = layout.is_some_and(|last: TableLayout| last.field_width == field_width);
            layout = Some(TableLayout {
                field_width,
                table_width,
                settled,
            });
            laid_out_width = field_width;
            widths.push(field_width);
        }

        let target = (AVAILABLE - OTHER_COLUMNS) / COORDINATE_COLUMNS as f32;
        assert!(
            widths[4..].iter().all(|width| *width == target),
            "{widths:?}"
        );
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
                panel.show(ui, &document, &selection);
            });
            output.textures_delta.clear();
            layouts.push(panel.layout.unwrap());
        }

        let last = layouts[layouts.len() - 1];
        assert!(
            layouts[20..].iter().all(|layout| *layout == last),
            "{layouts:?}"
        );
        assert!(last.field_width > min_field_width_in(&ctx), "{last:?}");
    }

    fn min_field_width_in(ctx: &egui::Context) -> f32 {
        let mut width = 0.0;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            width = min_field_width(ui);
        });
        output.textures_delta.clear();
        width
    }

    #[test]
    fn the_minimum_field_width_fits_the_widest_coordinate() {
        let ctx = egui::Context::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            let font = egui::TextStyle::Body.resolve(ui.style());
            let text_width = ui
                .painter()
                .layout_no_wrap("9999.99".to_owned(), font, egui::Color32::WHITE)
                .size()
                .x;
            let padding = ui.spacing().button_padding.x * 2.0;
            assert!(min_field_width(ui) >= text_width + padding);
        });
        output.textures_delta.clear();
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
