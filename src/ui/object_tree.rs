//! Left-hand panel: a tree of the document's objects (Document > Measurements >
//! Point-to-point / Area, and Document > Grids, then each object in document order).
//! Clicking an object makes it the active object.

use eframe::egui;
use egui_ltreeview::{Action as TreeAction, TreeView};
use egui_material_icons::MaterialIcon;
use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::grid::{Grid, GridSource};
use gridtool::object::ObjectId;

use super::format::format_length;
use super::icon_button::icon_button;
use super::icons::{DELETE as DELETE_ICON, DUPLICATE as DUPLICATE_ICON};
use super::selection::Selection;

const DEFAULT_WIDTH: f32 = 220.0;

/// A node in the object tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeNode {
    Document,
    Measurements,
    PointToPoint,
    Areas,
    Grids,
    Object(ObjectId),
}

/// What the user picked in the tree this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreePick {
    /// An object node was selected.
    Object(ObjectId),
    /// A non-object node (or nothing) was selected.
    NoObject,
}

/// What the user did in the panel this frame.
#[derive(Debug, Default)]
pub struct ObjectPanelOutput {
    /// A tree node the user selected.
    pub pick: Option<TreePick>,
    /// A document action from the panel's tool bar.
    pub action: Option<Action>,
}

/// The tool bar commands available for the active object: `None` means the button is
/// disabled.
#[derive(Debug, Default, PartialEq)]
struct ObjectCommands {
    duplicate: Option<Action>,
    delete: Option<Action>,
}

impl ObjectCommands {
    fn for_active(document: &Document, active: Option<ObjectId>) -> Self {
        let Some(id) = active else {
            return Self::default();
        };
        Self {
            duplicate: Action::duplicate_object(document, id),
            delete: Action::delete_object(document, id),
        }
    }

    /// Shows the tool bar, returning the command the user clicked.
    fn show(self, ui: &mut egui::Ui) -> Option<Action> {
        ui.horizontal(|ui| {
            let duplicate = command_button(ui, DUPLICATE_ICON, "Duplicate", self.duplicate);
            let delete = command_button(ui, DELETE_ICON, "Delete", self.delete);
            duplicate.or(delete)
        })
        .inner
    }
}

/// A tool bar button that is enabled exactly when `command` is available; returns the
/// command when clicked.
fn command_button(
    ui: &mut egui::Ui,
    icon: MaterialIcon,
    tooltip: &str,
    command: Option<Action>,
) -> Option<Action> {
    let clicked = icon_button(ui, icon, tooltip, command.is_some());
    command.filter(|_| clicked)
}

/// Shows the panel. `open` is the open document with its tree state and selection.
pub fn show(
    ui: &mut egui::Ui,
    open: Option<(
        &Document,
        &mut egui_ltreeview::TreeViewState<TreeNode>,
        &Selection,
    )>,
) -> ObjectPanelOutput {
    egui::Panel::left("object_panel")
        .resizable(true)
        .default_size(DEFAULT_WIDTH)
        .show(ui, |ui| {
            ui.heading("Objects");
            let commands = match &open {
                Some((document, _, selection)) => {
                    ObjectCommands::for_active(document, selection.active())
                }
                None => ObjectCommands::default(),
            };
            let action = commands.show(ui);
            ui.separator();
            let Some((document, state, selection)) = open else {
                ui.weak("No document open");
                return ObjectPanelOutput::default();
            };
            let pick = show_tree(ui, document, state, selection);
            ObjectPanelOutput { pick, action }
        })
        .inner
}

fn show_tree(
    ui: &mut egui::Ui,
    document: &Document,
    state: &mut egui_ltreeview::TreeViewState<TreeNode>,
    selection: &Selection,
) -> Option<TreePick> {
    if let Some(nodes) = tree_selection_for(selection.active(), state.selected()) {
        state.set_selected(nodes);
    }
    let (_, actions) = TreeView::new(egui::Id::unique("object_tree"))
        .allow_multi_selection(false)
        .allow_drag_and_drop(false)
        .show_state(ui, state, |builder| {
            builder.dir(TreeNode::Document, "Document");
            builder.dir(TreeNode::Measurements, "Measurements");
            builder.dir(TreeNode::PointToPoint, "Point-to-point");
            for (index, measurement) in document.measurements().iter().enumerate() {
                let label = match measurement.distance(document.calibration()) {
                    Ok(distance) => format!(
                        "Measurement {} ({} {})",
                        index + 1,
                        format_length(distance),
                        measurement.unit()
                    ),
                    Err(_) => format!("Measurement {}", index + 1),
                };
                builder.leaf(TreeNode::Object(measurement.id()), label);
            }
            builder.close_dir();
            builder.dir(TreeNode::Areas, "Area");
            for (index, area) in document.areas().iter().enumerate() {
                builder.leaf(TreeNode::Object(area.id()), area_label(document, index));
            }
            builder.close_dir();
            builder.close_dir();
            builder.dir(TreeNode::Grids, "Grids");
            for (index, grid) in document.grids().iter().enumerate() {
                let label = format!("Grid {} ({})", index + 1, source_label(document, grid));
                builder.leaf(TreeNode::Object(grid.id()), label);
            }
            builder.close_dir();
            builder.close_dir();
        });
    actions.into_iter().find_map(|action| match action {
        TreeAction::SetSelected(nodes) => Some(pick_from(&nodes)),
        _ => None,
    })
}

/// "Area <n> (<area> <unit>²)", or "Area <n>" when the area cannot be calculated.
fn area_label(document: &Document, index: usize) -> String {
    let area = &document.areas()[index];
    match area.area(document.calibration()) {
        Ok(value) => format!(
            "Area {} ({} {})",
            index + 1,
            format_length(value),
            area.unit().area_symbol()
        ),
        Err(_) => format!("Area {}", index + 1),
    }
}

/// Describes what a grid is aligned to, e.g. "scale line A" or "measurement 2".
fn source_label(document: &Document, grid: &Grid) -> String {
    match grid.source() {
        GridSource::ScaleLine(line) => format!("scale line {line}"),
        GridSource::Measurement(id) => match document.measurement_index(id) {
            Some(index) => format!("measurement {}", index + 1),
            None => "alignment line missing".to_owned(),
        },
    }
}

/// The tree selection that reflects `active`, or `None` if `current` already does.
/// A selected folder is left alone when there is no active object.
fn tree_selection_for(active: Option<ObjectId>, current: &[TreeNode]) -> Option<Vec<TreeNode>> {
    match active {
        Some(id) => (current != [TreeNode::Object(id)]).then(|| vec![TreeNode::Object(id)]),
        None => current
            .iter()
            .any(|node| matches!(node, TreeNode::Object(_)))
            .then(Vec::new),
    }
}

fn pick_from(nodes: &[TreeNode]) -> TreePick {
    match nodes {
        [TreeNode::Object(id)] => TreePick::Object(*id),
        _ => TreePick::NoObject,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gridtool::action::Action;
    use gridtool::geometry::Point;
    use gridtool::measurement::Measurement;
    use gridtool::units::LengthUnit;

    fn two_ids() -> (ObjectId, ObjectId) {
        let mut document = Document::new(10, 10);
        for _ in 0..2 {
            Action::AddMeasurement(Measurement::new(
                Point::new(1.0, 1.0),
                Point::new(2.0, 2.0),
                LengthUnit::Centimetre,
            ))
            .apply(&mut document)
            .unwrap();
        }
        (
            document.measurements()[0].id(),
            document.measurements()[1].id(),
        )
    }

    #[test]
    fn tree_selects_the_active_object() {
        let (a, b) = two_ids();

        assert_eq!(
            tree_selection_for(Some(a), &[]),
            Some(vec![TreeNode::Object(a)])
        );
        assert_eq!(
            tree_selection_for(Some(a), &[TreeNode::Object(b)]),
            Some(vec![TreeNode::Object(a)])
        );
        assert_eq!(tree_selection_for(Some(a), &[TreeNode::Object(a)]), None);
    }

    #[test]
    fn clearing_the_active_object_deselects_it_but_keeps_a_folder_selection() {
        let (a, _) = two_ids();

        assert_eq!(
            tree_selection_for(None, &[TreeNode::Object(a)]),
            Some(vec![])
        );
        assert_eq!(tree_selection_for(None, &[TreeNode::Measurements]), None);
        assert_eq!(tree_selection_for(None, &[]), None);
    }

    #[test]
    fn commands_are_disabled_without_an_active_object() {
        let document = Document::new(10, 10);

        assert_eq!(
            ObjectCommands::for_active(&document, None),
            ObjectCommands::default()
        );
    }

    #[test]
    fn commands_target_the_active_measurement_or_grid() {
        let mut document = Document::new(100, 100);
        Action::AddMeasurement(Measurement::new(
            Point::new(1.0, 1.0),
            Point::new(2.0, 2.0),
            LengthUnit::Centimetre,
        ))
        .apply(&mut document)
        .unwrap();
        let measurement = document.measurements()[0].id();
        Action::AddGrid(Grid::new(
            GridSource::Measurement(measurement),
            LengthUnit::Centimetre,
        ))
        .apply(&mut document)
        .unwrap();
        let grid = document.grids()[0].id();

        assert_eq!(
            ObjectCommands::for_active(&document, Some(measurement)),
            ObjectCommands {
                duplicate: Some(Action::DuplicateMeasurement { index: 0 }),
                delete: Some(Action::DeleteMeasurement { index: 0 }),
            }
        );
        assert_eq!(
            ObjectCommands::for_active(&document, Some(grid)),
            ObjectCommands {
                duplicate: Some(Action::DuplicateGrid { index: 0 }),
                delete: Some(Action::DeleteGrid { index: 0 }),
            }
        );
    }

    #[test]
    fn only_a_single_object_node_picks_an_object() {
        let (a, _) = two_ids();

        assert_eq!(pick_from(&[TreeNode::Object(a)]), TreePick::Object(a));
        assert_eq!(pick_from(&[TreeNode::Measurements]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::Grids]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::PointToPoint]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::Areas]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::Document]), TreePick::NoObject);
        assert_eq!(pick_from(&[]), TreePick::NoObject);
    }
}
