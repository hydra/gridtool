//! Left-hand panel: a tree of the document's objects (Document > Measurements >
//! Point-to-point / Area, Document > Grids and Document > Shapes, then each object in
//! document order). Clicking an object makes it the active object. An object that becomes
//! active by another route is selected in the tree, with its folders opened and scrolled to.

use eframe::egui;
use egui_ltreeview::{Action as TreeAction, TreeView};
use egui_material_icons::MaterialIcon;
use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::grid::{Grid, GridSource};
use gridtool::object::{ObjectId, ObjectKind};

use super::format::format_length;
use super::icon_button::icon_button;
use super::icons::{DELETE as DELETE_ICON, DUPLICATE as DUPLICATE_ICON};
use super::selection::Selection;
use super::tool_controls::shape_label;

const DEFAULT_WIDTH: f32 = 220.0;

/// A node in the object tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeNode {
    Document,
    Measurements,
    PointToPoint,
    Areas,
    Grids,
    Shapes,
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
            let pick = show_tree(ui, document, state, selection).inner;
            ObjectPanelOutput { pick, action }
        })
        .inner
}

fn show_tree(
    ui: &mut egui::Ui,
    document: &Document,
    state: &mut egui_ltreeview::TreeViewState<TreeNode>,
    selection: &Selection,
) -> egui::scroll_area::ScrollAreaOutput<Option<TreePick>> {
    let mut reveal = None;
    if let Some(nodes) = tree_selection_for(selection.active(), state.selected()) {
        reveal = nodes.first().copied();
        state.set_selected(nodes);
    }
    if let Some(TreeNode::Object(id)) = reveal
        && let Some(kind) = document.object_kind(id)
    {
        for &folder in folders_of(kind) {
            state.set_openness(folder, true);
        }
    }
    egui::ScrollArea::both()
        .id_salt("object_tree_scroll")
        .auto_shrink(false)
        .show(ui, |ui| {
            if let Some(node) = reveal {
                scroll_to_node(ui, document, state, node);
            }
            tree(ui, document, state)
        })
}

/// Scrolls vertically to `node`, which must be in an open folder. The tree view cannot do this
/// itself, as it only draws the rows in view.
fn scroll_to_node(
    ui: &mut egui::Ui,
    document: &Document,
    state: &egui_ltreeview::TreeViewState<TreeNode>,
    node: TreeNode,
) {
    let is_open = |folder| state.is_open(&folder).unwrap_or(true);
    let Some(row) = visible_rows(document, is_open)
        .iter()
        .position(|&visible| visible == node)
    else {
        return;
    };
    let height = ui.spacing().interact_size.y + ui.spacing().item_spacing.y;
    let top = ui.cursor().top() + row as f32 * height;
    let rect = egui::Rect::from_x_y_ranges(ui.clip_rect().x_range(), top..=top + height);
    ui.scroll_to_rect(rect, None);
}

/// The folders that must be open for an object of `kind` to be visible, outermost first.
fn folders_of(kind: ObjectKind) -> &'static [TreeNode] {
    match kind {
        ObjectKind::Measurement => &[
            TreeNode::Document,
            TreeNode::Measurements,
            TreeNode::PointToPoint,
        ],
        ObjectKind::Area => &[TreeNode::Document, TreeNode::Measurements, TreeNode::Areas],
        ObjectKind::Grid => &[TreeNode::Document, TreeNode::Grids],
        ObjectKind::Shape => &[TreeNode::Document, TreeNode::Shapes],
    }
}

/// Every row of the tree, top to bottom, leaving out what is inside a closed folder.
fn visible_rows(document: &Document, is_open: impl Fn(TreeNode) -> bool) -> Vec<TreeNode> {
    let mut rows = vec![TreeNode::Document];
    if !is_open(TreeNode::Document) {
        return rows;
    }
    let folder = |rows: &mut Vec<TreeNode>, node, objects: Vec<ObjectId>| {
        rows.push(node);
        if is_open(node) {
            rows.extend(objects.into_iter().map(TreeNode::Object));
        }
    };
    let measurements = document.measurements().iter().map(|m| m.id()).collect();
    let areas = document.areas().iter().map(|area| area.id()).collect();
    let grids = document.grids().iter().map(|grid| grid.id()).collect();
    let shapes = document.shapes().iter().map(|shape| shape.id()).collect();
    rows.push(TreeNode::Measurements);
    if is_open(TreeNode::Measurements) {
        folder(&mut rows, TreeNode::PointToPoint, measurements);
        folder(&mut rows, TreeNode::Areas, areas);
    }
    folder(&mut rows, TreeNode::Grids, grids);
    folder(&mut rows, TreeNode::Shapes, shapes);
    rows
}

fn tree(
    ui: &mut egui::Ui,
    document: &Document,
    state: &mut egui_ltreeview::TreeViewState<TreeNode>,
) -> Option<TreePick> {
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
            builder.dir(TreeNode::Shapes, "Shapes");
            for (index, shape) in document.shapes().iter().enumerate() {
                builder.leaf(TreeNode::Object(shape.id()), shape_label(index, shape));
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
    fn each_kind_of_object_lists_the_folders_it_is_in() {
        assert_eq!(
            folders_of(ObjectKind::Measurement),
            [
                TreeNode::Document,
                TreeNode::Measurements,
                TreeNode::PointToPoint
            ]
        );
        assert_eq!(
            folders_of(ObjectKind::Area),
            [TreeNode::Document, TreeNode::Measurements, TreeNode::Areas]
        );
        assert_eq!(
            folders_of(ObjectKind::Grid),
            [TreeNode::Document, TreeNode::Grids]
        );
        assert_eq!(
            folders_of(ObjectKind::Shape),
            [TreeNode::Document, TreeNode::Shapes]
        );
    }

    fn document_with_measurements(count: usize) -> Document {
        let mut document = Document::new(10, 10);
        for _ in 0..count {
            Action::AddMeasurement(Measurement::new(
                Point::new(1.0, 1.0),
                Point::new(2.0, 2.0),
                LengthUnit::Centimetre,
            ))
            .apply(&mut document)
            .unwrap();
        }
        document
    }

    #[test]
    fn the_rows_of_the_tree_leave_out_what_is_in_closed_folders() {
        let document = document_with_measurements(2);
        let leaf = |index: usize| TreeNode::Object(document.measurements()[index].id());

        assert_eq!(
            visible_rows(&document, |_| true),
            [
                TreeNode::Document,
                TreeNode::Measurements,
                TreeNode::PointToPoint,
                leaf(0),
                leaf(1),
                TreeNode::Areas,
                TreeNode::Grids,
                TreeNode::Shapes,
            ]
        );
        assert_eq!(
            visible_rows(&document, |node| node != TreeNode::PointToPoint),
            [
                TreeNode::Document,
                TreeNode::Measurements,
                TreeNode::PointToPoint,
                TreeNode::Areas,
                TreeNode::Grids,
                TreeNode::Shapes,
            ]
        );
        assert_eq!(
            visible_rows(&document, |node| node != TreeNode::Measurements),
            [
                TreeNode::Document,
                TreeNode::Measurements,
                TreeNode::Grids,
                TreeNode::Shapes,
            ]
        );
        assert_eq!(
            visible_rows(&document, |node| node != TreeNode::Document),
            [TreeNode::Document]
        );
    }

    #[test]
    fn an_object_activated_elsewhere_has_its_folders_opened_and_is_scrolled_to() {
        let document = document_with_measurements(60);
        let ctx = egui::Context::default();
        let mut state = egui_ltreeview::TreeViewState::default();
        state.set_openness(TreeNode::Document, false);
        state.set_openness(TreeNode::PointToPoint, false);
        let mut selection = Selection::default();
        let mut offset = 0.0;
        let mut frame = |state: &mut _, selection: &Selection| {
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(300.0, 200.0),
                )),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                offset = show_tree(ui, &document, state, selection)
                    .state
                    .clamped_offset()
                    .y;
            });
            output.textures_delta.clear();
            offset
        };
        assert_eq!(frame(&mut state, &selection), 0.0);

        selection.select(Some(document.measurements()[59].id()));
        let mut settled = 0.0;
        for _ in 0..200 {
            settled = frame(&mut state, &selection);
        }

        assert_eq!(state.is_open(&TreeNode::Document), Some(true));
        assert_eq!(state.is_open(&TreeNode::Measurements), Some(true));
        assert_eq!(state.is_open(&TreeNode::PointToPoint), Some(true));
        assert!(settled > 100.0, "{settled}");
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
    fn commands_target_the_active_shape() {
        let mut document = Document::new(100, 100);
        Action::AddShape(
            gridtool::shape::DrawnShape::open(
                vec![Point::new(1.0, 1.0), Point::new(2.0, 2.0)],
                gridtool::units::LengthUnit::Centimetre,
            )
            .unwrap(),
        )
        .apply(&mut document)
        .unwrap();

        assert_eq!(
            ObjectCommands::for_active(&document, Some(document.shapes()[0].id())),
            ObjectCommands {
                duplicate: Some(Action::DuplicateShape { index: 0 }),
                delete: Some(Action::DeleteShape { index: 0 }),
            }
        );
    }

    #[test]
    fn only_a_single_object_node_picks_an_object() {
        let (a, _) = two_ids();

        assert_eq!(pick_from(&[TreeNode::Object(a)]), TreePick::Object(a));
        assert_eq!(pick_from(&[TreeNode::Measurements]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::Grids]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::Shapes]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::PointToPoint]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::Areas]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::Document]), TreePick::NoObject);
        assert_eq!(pick_from(&[]), TreePick::NoObject);
    }
}
