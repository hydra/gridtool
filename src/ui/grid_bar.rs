//! Properties row for the active grid: cell shape, cell size, colour and opacity.

use eframe::egui::{self, Popup, PopupCloseBehavior};
use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::grid::{CellDimension, CellShape, Grid};
use gridtool::object::ObjectId;

use super::color::{from_color32, to_color32};
use super::commit_field::CommitField;
use super::format::{format_length, parse_number};
use super::selection::Selection;

const SIZE_FIELD_WIDTH: f32 = 60.0;
const COLOR_SWATCH_SIZE: egui::Vec2 = egui::Vec2::new(36.0, 18.0);

#[derive(Default)]
pub struct GridBar {
    size_fields: CommitField,
}

impl GridBar {
    /// Shows the row if the active object is a grid, returning the action the user
    /// completed, if any.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        document: &Document,
        selection: &mut Selection,
    ) -> Option<Action> {
        let id = selection.active()?;
        let index = document.grid_index(id)?;
        let stored: &Grid = &document.grids()[index];

        egui::Panel::top("grid_bar")
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    // Every widget must be drawn each frame, so collect actions eagerly.
                    let mut actions = Vec::new();
                    ui.strong(format!("Grid {}", index + 1));
                    ui.separator();
                    actions.extend(shape_buttons(ui, index, stored));
                    ui.separator();
                    actions.extend(self.size_fields(ui, id, index, stored));
                    ui.separator();
                    actions.extend(color_picker(ui, id, index, stored, selection));
                    ui.separator();
                    actions.extend(opacity_slider(ui, id, index, stored, selection));
                    if let Err(reason) = selection.displayed_grid(id, stored).lines(document) {
                        ui.separator();
                        ui.weak(format!("Grid not shown: {reason}"));
                    }
                    actions.into_iter().next()
                })
                .inner
            })
            .inner
    }

    fn size_fields(
        &mut self,
        ui: &mut egui::Ui,
        id: ObjectId,
        index: usize,
        grid: &Grid,
    ) -> Option<Action> {
        let dimensions: &[(CellDimension, &str)] = match grid.shape() {
            CellShape::Square => &[(CellDimension::Side, "Side length of each square cell")],
            CellShape::Rectangle => &[
                (CellDimension::A, "Cell size along the alignment line"),
                (
                    CellDimension::B,
                    "Cell size perpendicular to the alignment line",
                ),
            ],
        };
        let mut actions = Vec::new();
        for &(dimension, tooltip) in dimensions {
            let label = ui.label(dimension.to_string()).on_hover_text(tooltip);
            let field_id = egui::Id::unique(("grid_size", id, dimension));
            let current = grid.size(dimension);
            let (field, committed) =
                self.size_fields
                    .show(ui, field_id, format_length(current), SIZE_FIELD_WIDTH);
            field.labelled_by(label.id).on_hover_text(tooltip);
            actions.extend(
                committed
                    .and_then(|text| parse_number(&text))
                    .filter(|&size| size > 0.0 && size != current)
                    .map(|size| Action::SetGridSize {
                        index,
                        dimension,
                        size,
                    }),
            );
        }
        ui.label(grid.unit().symbol());
        actions.into_iter().next()
    }
}

fn shape_buttons(ui: &mut egui::Ui, index: usize, grid: &Grid) -> Option<Action> {
    ui.label("Cell shape:");
    let mut action = None;
    for shape in CellShape::ALL {
        if ui
            .selectable_label(grid.shape() == shape, shape.name())
            .clicked()
            && shape != grid.shape()
        {
            action = Some(Action::SetGridShape { index, shape });
        }
    }
    action
}

/// A colour swatch that opens a colour picker. The colour previews live while the picker
/// is open and is committed as one action when it closes.
fn color_picker(
    ui: &mut egui::Ui,
    id: ObjectId,
    index: usize,
    stored: &Grid,
    selection: &mut Selection,
) -> Option<Action> {
    ui.label("Color:");
    let popup_id = egui::Id::unique(("grid_color_popup", id));
    let was_open = Popup::is_id_open(ui.ctx(), popup_id);
    let displayed = selection.displayed_grid(id, stored);
    let mut color = to_color32(displayed.color());

    let swatch = ui
        .add(
            egui::Button::new("")
                .fill(color)
                .min_size(COLOR_SWATCH_SIZE),
        )
        .on_hover_text("Choose the grid colour");
    Popup::menu(&swatch)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            if egui::color_picker::color_picker_color32(
                ui,
                &mut color,
                egui::color_picker::Alpha::Opaque,
            ) {
                selection.preview_grid(displayed.with_color(from_color32(color)));
            }
        });

    let closed = was_open && !Popup::is_id_open(ui.ctx(), popup_id);
    if !closed {
        return None;
    }
    let chosen = selection.displayed_grid(id, stored).color();
    selection.clear_grid_preview();
    (chosen != stored.color()).then_some(Action::SetGridColor {
        index,
        color: chosen,
    })
}

/// A 0–100 % slider. Dragging previews live and commits one action on release; clicks
/// and keyboard changes commit immediately.
fn opacity_slider(
    ui: &mut egui::Ui,
    id: ObjectId,
    index: usize,
    stored: &Grid,
    selection: &mut Selection,
) -> Option<Action> {
    ui.label("Opacity:");
    let displayed = selection.displayed_grid(id, stored);
    let mut percent = displayed.opacity() * 100.0;
    let response = ui.add(
        egui::Slider::new(&mut percent, 0.0..=100.0)
            .suffix("%")
            .fixed_decimals(0),
    );
    if response.changed() {
        selection.preview_grid(displayed.with_opacity(percent.round() / 100.0));
    }

    let finished = response.drag_stopped() || (response.changed() && !response.dragged());
    if !finished {
        return None;
    }
    let chosen = selection.displayed_grid(id, stored).opacity();
    selection.clear_grid_preview();
    (chosen != stored.opacity()).then_some(Action::SetGridOpacity {
        index,
        opacity: chosen,
    })
}
