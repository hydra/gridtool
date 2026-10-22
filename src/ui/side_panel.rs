//! The right-hand panel: the Point-to-point table, the Areas table and the history,
//! stacked top to bottom with draggable dividers between them.

use eframe::egui;
use gridtool::action::Action;
use gridtool::editor::Editor;

use super::areas_panel;
use super::history_panel;
use super::measurements_panel::{self, MeasurementsPanel};
use super::selection::Selection;
use super::vertical_split::VerticalSplit;

const DEFAULT_WIDTH: f32 = 460.0;
/// The initial share of the panel's height of each section: Point-to-point, Areas and
/// History.
const DEFAULT_SECTION_FRACTIONS: [f32; 3] = [0.33, 0.34, 0.33];

const POINT_TO_POINT: usize = 0;
const AREAS: usize = 1;

pub struct SidePanel {
    measurements: MeasurementsPanel,
    sections: VerticalSplit,
}

impl Default for SidePanel {
    fn default() -> Self {
        Self {
            measurements: MeasurementsPanel::default(),
            sections: VerticalSplit::new(&DEFAULT_SECTION_FRACTIONS),
        }
    }
}

impl SidePanel {
    /// Shows the side panel, returning the document action the user completed, if any.
    /// `open` is the open document's editor and selection, if a document is open.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        open: Option<(&Editor, &Selection)>,
    ) -> Option<Action> {
        let measurements = &mut self.measurements;
        let sections = &mut self.sections;
        egui::Panel::right("side_panel")
            .resizable(true)
            .default_size(DEFAULT_WIDTH)
            .show(ui, |ui| {
                let actions = sections.show(ui, "side_panel_sections", |section, ui| {
                    match (section, open) {
                        (POINT_TO_POINT, Some((editor, selection))) => {
                            measurements.show(ui, editor.document(), selection)
                        }
                        (AREAS, Some((editor, selection))) => {
                            areas_panel::show(ui, editor.document(), selection)
                        }
                        (POINT_TO_POINT, None) => no_document(ui, measurements_panel::HEADING),
                        (AREAS, None) => no_document(ui, areas_panel::HEADING),
                        _ => {
                            history_panel::show(ui, open.map(|(editor, _)| editor));
                            None
                        }
                    }
                });
                actions.into_iter().flatten().next()
            })
            .inner
    }
}

fn no_document(ui: &mut egui::Ui, heading: &str) -> Option<Action> {
    ui.heading(heading);
    ui.separator();
    ui.weak("No document open");
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Pos2, RawInput, Rect};
    use gridtool::area::Area;
    use gridtool::document::Document;
    use gridtool::geometry::Point;
    use gridtool::measurement::Measurement;
    use gridtool::units::LengthUnit;

    fn editor() -> Editor {
        let mut editor = Editor::new(Document::new(100, 100));
        let points = [(1.0, 1.0), (50.0, 1.0), (20.0, 40.0)].map(|(x, y)| Point::new(x, y));
        editor
            .perform(Action::AddMeasurement(Measurement::new(
                points[0],
                points[1],
                LengthUnit::Centimetre,
            )))
            .unwrap();
        editor
            .perform(Action::AddArea(
                Area::new(points.to_vec(), LengthUnit::Metre).unwrap(),
            ))
            .unwrap();
        editor
    }

    #[test]
    fn shows_every_section_with_and_without_a_document() {
        let ctx = egui::Context::default();
        let mut panel = SidePanel::default();
        let editor = editor();
        let selection = Selection::default();

        for open in [None, Some((&editor, &selection))] {
            let input = RawInput {
                screen_rect: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(1200.0, 800.0))),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                assert_eq!(panel.show(ui, open), None);
            });
            // There is no renderer to upload textures to.
            output.textures_delta.clear();
        }

        assert_eq!(
            panel.sections,
            VerticalSplit::new(&DEFAULT_SECTION_FRACTIONS)
        );
    }
}
