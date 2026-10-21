//! Tool options row for the scale tool.

use eframe::egui;
use gridtool::action::Action;
use gridtool::calibration::{Calibration, CalibrationMethod, Line};
use gridtool::document::Document;

use super::commit_field::CommitField;
use super::format::format_trimmed;
use super::scale_tool::ScaleTool;
use super::unit_picker;

const DISTANCE_FIELD_WIDTH: f32 = 70.0;
const DISTANCE_DECIMALS: usize = 6;
const SCALE_DECIMALS: usize = 6;

/// Shows the calibration method, point coordinates, line distances, units and the
/// resulting scale. Edits are returned as actions rather than applied.
#[derive(Default)]
pub struct ScaleBar {
    distance_fields: CommitField,
}

impl ScaleBar {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        tool: &mut ScaleTool,
    ) -> Option<Action> {
        let calibration = document.calibration();
        egui::Panel::top("scale_bar")
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    method_buttons(ui, tool);
                    ui.separator();
                    point_coordinates(ui, tool, calibration);
                    ui.separator();

                    // Every widget must be drawn each frame, so collect actions eagerly.
                    let mut actions = Vec::new();
                    for line in Line::ALL {
                        actions.extend(self.distance_field(ui, line, calibration.distance(line)));
                    }
                    actions.extend(
                        unit_picker::show(
                            ui,
                            "scale_unit",
                            calibration.unit(),
                            "Units of distances A and B",
                        )
                        .map(Action::SetUnit),
                    );

                    ui.separator();
                    scale_result(ui, calibration);
                    actions.into_iter().next()
                })
                .inner
            })
            .inner
    }

    /// A labelled distance field. The value is committed as an action when editing ends.
    fn distance_field(
        &mut self,
        ui: &mut egui::Ui,
        line: Line,
        current: Option<f64>,
    ) -> Option<Action> {
        let label = ui.label(line.to_string());
        let id = egui::Id::unique(("scale_distance", line));
        let (field, committed) =
            self.distance_fields
                .show(ui, id, format_distance(current), DISTANCE_FIELD_WIDTH);
        field.labelled_by(label.id);

        parse_distance(&committed?)
            .filter(|&distance| distance != current)
            .map(|distance| Action::SetDistance { line, distance })
    }
}

fn method_buttons(ui: &mut egui::Ui, tool: &mut ScaleTool) {
    for method in CalibrationMethod::ALL {
        if ui
            .selectable_label(tool.method() == method, method.name())
            .clicked()
        {
            tool.select_method(method);
        }
    }
}

fn point_coordinates(ui: &mut egui::Ui, tool: &ScaleTool, calibration: &Calibration) {
    let points = tool.points(calibration);
    for index in 0..tool.method().point_count() {
        let position = points
            .get(index)
            .map_or_else(|| "—".to_owned(), ToString::to_string);
        ui.label(format!("{}: {position}", index + 1));
    }
}

fn scale_result(ui: &mut egui::Ui, calibration: &Calibration) {
    match calibration.scale() {
        Ok(scale) => {
            let format = |value| format_trimmed(value, SCALE_DECIMALS);
            ui.label(format!(
                "X: {} {unit}/px   Y: {} {unit}/px",
                format(scale.x_per_pixel),
                format(scale.y_per_pixel),
                unit = scale.unit
            ));
        }
        Err(reason) => {
            ui.weak(format!("Scale: {reason}"));
        }
    }
}

/// Parses a distance field: empty clears the distance, a positive number sets it, and
/// anything else is invalid (`None`).
fn parse_distance(text: &str) -> Option<Option<f64>> {
    let text = text.trim();
    if text.is_empty() {
        return Some(None);
    }
    text.parse::<f64>()
        .ok()
        .filter(|distance| distance.is_finite() && *distance > 0.0)
        .map(Some)
}

fn format_distance(distance: Option<f64>) -> String {
    distance
        .map(|distance| format_trimmed(distance, DISTANCE_DECIMALS))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_clears_the_distance() {
        assert_eq!(parse_distance("   "), Some(None));
    }

    #[test]
    fn positive_numbers_set_the_distance() {
        assert_eq!(parse_distance(" 12.5 "), Some(Some(12.5)));
    }

    #[test]
    fn invalid_text_is_rejected() {
        for invalid in ["abc", "0", "-3", "inf", "NaN"] {
            assert_eq!(
                parse_distance(invalid),
                None,
                "{invalid:?} should be rejected"
            );
        }
    }

    #[test]
    fn formats_missing_distance_as_empty() {
        assert_eq!(format_distance(None), "");
        assert_eq!(format_distance(Some(50.0)), "50");
    }
}
