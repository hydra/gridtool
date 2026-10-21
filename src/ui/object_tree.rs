//! Left-hand panel: a tree of the document's objects (Document > Measurements > each
//! measurement, in document order). Clicking an object makes it the active object.

use eframe::egui;
use egui_ltreeview::{Action as TreeAction, TreeView};
use gridtool::document::Document;
use gridtool::object::ObjectId;

use super::format::format_length;
use super::selection::Selection;

const DEFAULT_WIDTH: f32 = 220.0;

/// A node in the object tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeNode {
    Document,
    Measurements,
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

/// Shows the panel. `open` is the open document with its tree state and selection.
pub fn show(
    ui: &mut egui::Ui,
    open: Option<(
        &Document,
        &mut egui_ltreeview::TreeViewState<TreeNode>,
        &Selection,
    )>,
) -> Option<TreePick> {
    egui::Panel::left("object_panel")
        .resizable(true)
        .default_size(DEFAULT_WIDTH)
        .show(ui, |ui| {
            ui.heading("Objects");
            ui.separator();
            let Some((document, state, selection)) = open else {
                ui.weak("No document open");
                return None;
            };
            if let Some(nodes) = tree_selection_for(selection.active(), state.selected()) {
                state.set_selected(nodes);
            }
            let (_, actions) = TreeView::new(egui::Id::unique("object_tree"))
                .allow_multi_selection(false)
                .allow_drag_and_drop(false)
                .show_state(ui, state, |builder| {
                    builder.dir(TreeNode::Document, "Document");
                    builder.dir(TreeNode::Measurements, "Measurements");
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
                    builder.close_dir();
                });
            actions.into_iter().find_map(|action| match action {
                TreeAction::SetSelected(nodes) => Some(pick_from(&nodes)),
                _ => None,
            })
        })
        .inner
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
    fn only_a_single_object_node_picks_an_object() {
        let (a, _) = two_ids();

        assert_eq!(pick_from(&[TreeNode::Object(a)]), TreePick::Object(a));
        assert_eq!(pick_from(&[TreeNode::Measurements]), TreePick::NoObject);
        assert_eq!(pick_from(&[TreeNode::Document]), TreePick::NoObject);
        assert_eq!(pick_from(&[]), TreePick::NoObject);
    }
}
