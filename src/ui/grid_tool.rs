//! The grid tool: pick a scale line or measurement line to align a new grid to.

use eframe::egui;
use gridtool::action::Action;
use gridtool::calibration::Line;
use gridtool::document::Document;
use gridtool::geometry::Point;
use gridtool::grid::{Grid, GridSource};

use super::selection::Selection;

/// What kind of line new grids are aligned to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GridVia {
    #[default]
    Scale,
    Measurement,
}

impl GridVia {
    pub const ALL: [GridVia; 2] = [GridVia::Scale, GridVia::Measurement];

    pub fn name(self) -> &'static str {
        match self {
            GridVia::Scale => "Via scale",
            GridVia::Measurement => "Via measurement",
        }
    }
}

#[derive(Debug, Default)]
pub struct GridTool {
    via: GridVia,
    /// The line under the pointer this frame, if any.
    hovered: Option<GridSource>,
}

impl GridTool {
    pub fn set_via(&mut self, via: GridVia) {
        self.via = via;
    }

    pub fn hovered(&self) -> Option<GridSource> {
        self.hovered
    }

    pub fn set_hovered(&mut self, source: Option<GridSource>) {
        self.hovered = source;
    }

    /// The lines a grid can be aligned to in the current mode, in image coordinates.
    pub fn candidate_lines(
        &self,
        document: &Document,
        selection: &Selection,
    ) -> Vec<(GridSource, Point, Point)> {
        match self.via {
            GridVia::Scale => document
                .calibration()
                .points()
                .map(|points| {
                    Line::ALL
                        .into_iter()
                        .map(|line| {
                            let (start, end) = points.line(line);
                            (GridSource::ScaleLine(line), start, end)
                        })
                        .collect()
                })
                .unwrap_or_default(),
            GridVia::Measurement => document
                .measurements()
                .iter()
                .map(|stored| {
                    let measurement = selection.displayed(stored.id(), stored);
                    (
                        GridSource::Measurement(stored.id()),
                        measurement.start(),
                        measurement.end(),
                    )
                })
                .collect(),
        }
    }

    /// Shows the tool options row: the alignment mode buttons and a hint.
    pub fn show_options(&mut self, ui: &mut egui::Ui, document: &Document) {
        egui::Panel::top("grid_tool_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                for via in GridVia::ALL {
                    if ui.selectable_label(self.via == via, via.name()).clicked() {
                        self.set_via(via);
                    }
                }
                ui.separator();
                ui.weak(hint(self.via, document));
            });
        });
    }
}

/// The action that adds a grid aligned to `source`, with cell sizes in the source's unit.
pub fn add_grid(document: &Document, source: GridSource) -> Option<Action> {
    let unit = match source {
        GridSource::ScaleLine(_) => document.calibration().unit(),
        GridSource::Measurement(id) => {
            document.measurements()[document.measurement_index(id)?].unit()
        }
    };
    Some(Action::AddGrid(Grid::new(source, unit)))
}

fn hint(via: GridVia, document: &Document) -> &'static str {
    match via {
        GridVia::Scale if document.calibration().points().is_none() => {
            "Place the scale points first (Scale tool)."
        }
        GridVia::Scale => "Click a scale line to add a grid aligned to it.",
        GridVia::Measurement if document.measurements().is_empty() => {
            "Add a measurement first (Measure distance tool)."
        }
        GridVia::Measurement => "Click a measurement line to add a grid aligned to it.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::calibration::{CalibrationMethod, ScalePoints};
    use gridtool::measurement::Measurement;
    use gridtool::units::LengthUnit;

    fn document() -> Document {
        let mut document = Document::new(100, 100);
        let points = ScalePoints::new(
            CalibrationMethod::FourPoint,
            vec![
                Point::new(0.0, 0.0),
                Point::new(10.0, 0.0),
                Point::new(0.0, 20.0),
                Point::new(0.0, 30.0),
            ],
        )
        .unwrap();
        for action in [
            Action::SetScalePoints(points),
            Action::SetUnit(LengthUnit::Metre),
            Action::AddMeasurement(Measurement::new(
                Point::new(5.0, 5.0),
                Point::new(50.0, 5.0),
                LengthUnit::Centimetre,
            )),
        ] {
            action.apply(&mut document).unwrap();
        }
        document
    }

    #[test]
    fn via_scale_is_the_default() {
        assert_eq!(GridTool::default().via, GridVia::Scale);
    }

    #[test]
    fn via_scale_offers_both_scale_lines() {
        let tool = GridTool::default();

        let lines = tool.candidate_lines(&document(), &Selection::default());

        assert_eq!(
            lines,
            [
                (
                    GridSource::ScaleLine(Line::A),
                    Point::new(0.0, 0.0),
                    Point::new(10.0, 0.0)
                ),
                (
                    GridSource::ScaleLine(Line::B),
                    Point::new(0.0, 20.0),
                    Point::new(0.0, 30.0)
                ),
            ]
        );
    }

    #[test]
    fn via_measurement_offers_every_measurement() {
        let tool = GridTool {
            via: GridVia::Measurement,
            ..GridTool::default()
        };
        let document = document();

        let lines = tool.candidate_lines(&document, &Selection::default());

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].0,
            GridSource::Measurement(document.measurements()[0].id())
        );
    }

    #[test]
    fn no_scale_points_means_no_scale_lines() {
        let tool = GridTool::default();

        assert!(
            tool.candidate_lines(&Document::new(10, 10), &Selection::default())
                .is_empty()
        );
    }

    #[test]
    fn new_grids_take_their_unit_from_the_source() {
        let document = document();
        let measurement = GridSource::Measurement(document.measurements()[0].id());

        let Some(Action::AddGrid(scale_grid)) = add_grid(&document, GridSource::ScaleLine(Line::B))
        else {
            panic!("expected an add-grid action");
        };
        let Some(Action::AddGrid(measurement_grid)) = add_grid(&document, measurement) else {
            panic!("expected an add-grid action");
        };

        assert_eq!(scale_grid.unit(), LengthUnit::Metre);
        assert_eq!(measurement_grid.unit(), LengthUnit::Centimetre);
        assert_eq!(measurement_grid.source(), measurement);
    }
}
