//! The window below the tool bars: the Documents area, with the panels docked around it.
//! Panels can be dragged, resized, grouped as tabs and closed. The Documents area stays where
//! it is and never shares a tab group, so a panel can only ever be dropped beside it.

use std::collections::HashSet;

use eframe::egui;
use egui_tiles::{
    Behavior, Container, ContainerKind, LinearDir, SimplificationOptions, Tile, TileId, Tiles,
    Tree, UiResponse,
};
use gridtool::action::Action;
use gridtool::object::ObjectId;
use serde::{Deserialize, Serialize};

use crate::session::{DocumentId, Session};

use super::areas_panel;
use super::document_area::DocumentArea;
use super::history_panel;
use super::measurements_panel::MeasurementsPanel;
use super::object_tree::{self, TreePick};
use super::table_rows::TableOutput;

const PANEL_MARGIN: f32 = 6.0;
/// The default width of the column of Objects, of the Documents area and of the column of
/// tables, in points. The window divides its width in these proportions.
const DEFAULT_OBJECTS_WIDTH: f32 = 220.0;
const DEFAULT_DOCUMENTS_WIDTH: f32 = 600.0;
const DEFAULT_TABLES_WIDTH: f32 = 460.0;
/// A reopened panel's share of the workspace's width.
const REOPENED_PANEL_FRACTION: f32 = 0.2;

/// A panel showing information about the active document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Panel {
    Objects,
    PointToPoint,
    Areas,
    History,
}

impl Panel {
    pub const ALL: [Self; 4] = [
        Self::Objects,
        Self::PointToPoint,
        Self::Areas,
        Self::History,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Objects => "Objects",
            Self::PointToPoint => "Point-to-point",
            Self::Areas => "Areas",
            Self::History => "History",
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Objects => 0,
            Self::PointToPoint => 1,
            Self::Areas => 2,
            Self::History => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum WorkspacePane {
    Documents,
    Panel(Panel),
}

/// What the user did in the workspace this frame.
#[derive(Debug, Default)]
pub struct WorkspaceOutput {
    /// Document actions, all for the active document.
    pub actions: Vec<Action>,
    /// What was picked in the object tree.
    pub tree_pick: Option<TreePick>,
    /// The object whose row was picked in a table.
    pub table_pick: Option<ObjectId>,
    /// The document the user clicked, to become the active one.
    pub activate: Option<DocumentId>,
    /// The document whose tab the user asked to close.
    pub close: Option<DocumentId>,
}

impl WorkspaceOutput {
    fn add_table(&mut self, table: TableOutput) {
        self.actions.extend(table.action);
        self.table_pick = self.table_pick.or(table.pick);
    }
}

/// What the tables remember between frames, for the document they are showing.
#[derive(Default)]
struct Tables {
    measurements: MeasurementsPanel,
    /// The active object when the tables were last shown.
    last_active: Option<ObjectId>,
}

pub struct Workspace {
    tree: Tree<WorkspacePane>,
    area: DocumentArea,
    tables: Tables,
}

impl Default for Workspace {
    fn default() -> Self {
        Self {
            tree: default_tree(),
            area: DocumentArea::default(),
            tables: Tables::default(),
        }
    }
}

impl Workspace {
    /// The workspace with a layout written by [`Self::save_layout`], or the default layout if
    /// that cannot be read or could not have been arranged.
    pub fn from_saved_layout(layout: Option<&str>) -> Self {
        let tree = layout
            .and_then(|layout| ron::from_str(layout).ok())
            .filter(is_valid_layout);
        Self {
            tree: tree.unwrap_or_else(default_tree),
            ..Self::default()
        }
    }

    /// The panels' arrangement, for restoring in the next run.
    pub fn save_layout(&self) -> Option<String> {
        ron::to_string(&self.tree).ok()
    }

    /// Call once per frame, before anything uses the active document.
    pub fn sync(&mut self, session: &mut Session) {
        self.area.sync(session);
    }

    /// Forgets what the tables remember, which belongs to the document they were showing.
    pub fn forget_table_state(&mut self) {
        self.tables = Tables::default();
    }

    pub fn is_open(&self, panel: Panel) -> bool {
        self.tree
            .tiles
            .find_pane(&WorkspacePane::Panel(panel))
            .is_some()
    }

    /// Closes `panel` or, if it is closed, reopens it at the side of the workspace.
    pub fn toggle(&mut self, panel: Panel) {
        match self.tree.tiles.find_pane(&WorkspacePane::Panel(panel)) {
            Some(tile) => {
                self.tree.remove_recursively(tile);
            }
            None => self.reopen(panel),
        }
    }

    fn reopen(&mut self, panel: Panel) {
        let tiles = &mut self.tree.tiles;
        let pane = tiles.insert_pane(WorkspacePane::Panel(panel));
        let group = tiles.insert_tab_tile(vec![pane]);
        let root = self.tree.root.expect("the Documents area is always there");
        let at_start = panel == Panel::Objects;
        if let Some(Tile::Container(Container::Linear(linear))) = tiles.get_mut(root)
            && linear.dir == LinearDir::Horizontal
        {
            let total: f32 = linear
                .children
                .iter()
                .map(|child| linear.shares[*child])
                .sum();
            linear.shares.set_share(
                group,
                total * REOPENED_PANEL_FRACTION / (1.0 - REOPENED_PANEL_FRACTION),
            );
            let index = if at_start { 0 } else { linear.children.len() };
            linear.children.insert(index, group);
        } else {
            let children = if at_start {
                [group, root]
            } else {
                [root, group]
            };
            let new_root = tiles.insert_horizontal_tile(children.to_vec());
            if let Some(Tile::Container(Container::Linear(linear))) = tiles.get_mut(new_root) {
                linear.shares.set_share(group, REOPENED_PANEL_FRACTION);
                linear.shares.set_share(root, 1.0 - REOPENED_PANEL_FRACTION);
            }
            self.tree.root = Some(new_root);
        }
    }

    /// Puts every panel back where it starts, reopening any that were closed.
    pub fn reset_layout(&mut self) {
        self.tree = default_tree();
    }

    /// Shows everything in the rest of the window.
    pub fn show(&mut self, ui: &mut egui::Ui, session: &mut Session) -> WorkspaceOutput {
        let options = simplification_options();
        self.tree.simplify(&options);
        let mut output = WorkspaceOutput::default();
        let active_object = session
            .document()
            .and_then(|open| open.view.selection.active());
        let reveal = active_object != self.tables.last_active;
        self.tables.last_active = active_object;
        let mut behavior = WorkspaceBehavior {
            documents: self.tree.tiles.find_pane(&WorkspacePane::Documents),
            documents_and_ancestors: documents_and_ancestors(&self.tree),
            area: &mut self.area,
            session,
            tables: &mut self.tables,
            reveal,
            output: &mut output,
        };
        let tree = &mut self.tree;
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ui.style()).inner_margin(0))
            .show(ui, |ui| tree.ui(&mut behavior, ui));
        output
    }

    /// The Documents area. For tests.
    #[cfg(test)]
    pub fn area(&self) -> &DocumentArea {
        &self.area
    }

    /// The Documents area. For tests.
    #[cfg(test)]
    pub fn area_mut(&mut self) -> &mut DocumentArea {
        &mut self.area
    }

    /// Where the Documents area is, once it has been drawn. For tests.
    #[cfg(test)]
    pub fn documents_rect(&self) -> Option<egui::Rect> {
        let tile = self.tree.tiles.find_pane(&WorkspacePane::Documents)?;
        self.tree.tiles.rect(tile)
    }

    /// Where a panel's content is, once it has been drawn. For tests.
    #[cfg(test)]
    pub fn panel_rect(&self, panel: Panel) -> Option<egui::Rect> {
        let tile = self.tree.tiles.find_pane(&WorkspacePane::Panel(panel))?;
        self.tree.tiles.rect(tile)
    }

    /// Whether the panels are arranged the same. For tests.
    #[cfg(test)]
    pub fn has_same_layout_as(&self, other: &Self) -> bool {
        self.tree == other.tree
    }

    /// Whether the Documents area shares a tab group, directly or as part of a bigger tile.
    #[cfg(test)]
    pub fn documents_share_a_tab_group(&self) -> bool {
        !is_valid_layout(&self.tree)
    }
}

fn simplification_options() -> SimplificationOptions {
    SimplificationOptions {
        all_panes_must_have_tabs: true,
        ..Default::default()
    }
}

/// Objects on the left, the Documents area in the middle and the tables on the right.
fn default_tree() -> Tree<WorkspacePane> {
    let mut tiles = Tiles::default();
    let panel_group = |tiles: &mut Tiles<WorkspacePane>, pane| {
        let pane = tiles.insert_pane(pane);
        tiles.insert_tab_tile(vec![pane])
    };
    let objects = panel_group(&mut tiles, WorkspacePane::Panel(Panel::Objects));
    let documents = panel_group(&mut tiles, WorkspacePane::Documents);
    let tables = [Panel::PointToPoint, Panel::Areas, Panel::History]
        .map(|panel| panel_group(&mut tiles, WorkspacePane::Panel(panel)));
    let tables = tiles.insert_vertical_tile(tables.to_vec());
    let root = tiles.insert_horizontal_tile(vec![objects, documents, tables]);
    if let Some(Tile::Container(Container::Linear(linear))) = tiles.get_mut(root) {
        linear.shares.set_share(objects, DEFAULT_OBJECTS_WIDTH);
        linear.shares.set_share(documents, DEFAULT_DOCUMENTS_WIDTH);
        linear.shares.set_share(tables, DEFAULT_TABLES_WIDTH);
    }
    Tree::new(egui::Id::unique("workspace"), root, tiles)
}

/// The tile of the Documents area and every tile that contains it.
fn documents_and_ancestors(tree: &Tree<WorkspacePane>) -> HashSet<TileId> {
    let mut tiles = HashSet::new();
    let mut next = tree.tiles.find_pane(&WorkspacePane::Documents);
    while let Some(tile) = next {
        tiles.insert(tile);
        next = tree.tiles.parent_of(tile);
    }
    tiles
}

/// Whether the layout is one the user could have arranged: the Documents area is there, once,
/// and shares no tab group, and no panel appears twice.
fn is_valid_layout(tree: &Tree<WorkspacePane>) -> bool {
    let count_of = |pane| {
        tree.tiles
            .tiles()
            .filter(|tile| matches!(tile, Tile::Pane(candidate) if *candidate == pane))
            .count()
    };
    let contains_documents = documents_and_ancestors(tree);
    let shares_a_group = contains_documents.iter().any(|tile| {
        matches!(
            tree.tiles.get_container(*tile),
            Some(Container::Tabs(tabs)) if tabs.children.len() > 1
        )
    });
    tree.root.is_some()
        && count_of(WorkspacePane::Documents) == 1
        && Panel::ALL
            .iter()
            .all(|panel| count_of(WorkspacePane::Panel(*panel)) <= 1)
        && !shares_a_group
}

struct WorkspaceBehavior<'a> {
    documents: Option<TileId>,
    documents_and_ancestors: HashSet<TileId>,
    area: &'a mut DocumentArea,
    session: &'a mut Session,
    tables: &'a mut Tables,
    /// The active object has just changed, so the tables scroll to its row.
    reveal: bool,
    output: &'a mut WorkspaceOutput,
}

impl WorkspaceBehavior<'_> {
    fn panel_ui(&mut self, ui: &mut egui::Ui, panel: Panel) {
        // Scroll positions and the like belong to the document.
        let scope = self.session.active_id();
        egui::Frame::new()
            .inner_margin(PANEL_MARGIN)
            .show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.push_id(scope, |ui| self.panel_content(ui, panel));
            });
    }

    fn panel_content(&mut self, ui: &mut egui::Ui, panel: Panel) {
        if panel == Panel::Objects {
            let tree = self.session.document_mut().map(|open| {
                let view = &mut open.view;
                (
                    open.editor.document(),
                    &mut view.tree_state,
                    &view.selection,
                )
            });
            let objects = object_tree::show(ui, tree);
            self.output.actions.extend(objects.action);
            self.output.tree_pick = self.output.tree_pick.or(objects.pick);
            return;
        }
        let Some(open) = self.session.document() else {
            ui.weak("No document open");
            return;
        };
        let (document, selection) = (open.editor.document(), &open.view.selection);
        match panel {
            Panel::PointToPoint => {
                let table = self
                    .tables
                    .measurements
                    .show(ui, document, selection, self.reveal);
                self.output.add_table(table);
            }
            Panel::Areas => {
                let table = areas_panel::show(ui, document, selection, self.reveal);
                self.output.add_table(table);
            }
            Panel::History => history_panel::show(ui, &open.editor),
            Panel::Objects => {}
        }
    }
}

impl Behavior<WorkspacePane> for WorkspaceBehavior<'_> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile: TileId,
        pane: &mut WorkspacePane,
    ) -> UiResponse {
        ui.shrink_clip_rect(ui.max_rect());
        match pane {
            WorkspacePane::Documents => self.area.show(ui, self.session, self.output),
            WorkspacePane::Panel(panel) => self.panel_ui(ui, *panel),
        }
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, pane: &WorkspacePane) -> egui::WidgetText {
        match pane {
            WorkspacePane::Documents => "Documents".into(),
            WorkspacePane::Panel(panel) => panel.title().into(),
        }
    }

    /// Tabs look draggable as they are.
    fn tab_hover_cursor_icon(&self) -> egui::CursorIcon {
        egui::CursorIcon::Default
    }

    fn is_tab_closable(&self, _tiles: &Tiles<WorkspacePane>, tile: TileId) -> bool {
        !self.documents_and_ancestors.contains(&tile)
    }

    /// The Documents area has no tab of its own: nothing could be done with one.
    fn is_tab_bar_visible(&self, tile: TileId, _tabs: &egui_tiles::Tabs) -> bool {
        !self.documents_and_ancestors.contains(&tile)
    }

    fn is_tile_draggable(&self, _tiles: &Tiles<WorkspacePane>, tile: TileId) -> bool {
        !self.documents_and_ancestors.contains(&tile)
    }

    fn simplification_options(&self) -> SimplificationOptions {
        simplification_options()
    }

    /// A panel may be dropped beside the Documents area, but not inside it, and not as a tab
    /// with anything containing it, which could hide it.
    fn allows_drop_on_tile(&self, tile: TileId, kind: ContainerKind) -> bool {
        self.documents != Some(tile)
            && !(kind == ContainerKind::Tabs && self.documents_and_ancestors.contains(&tile))
    }

    fn allows_drops_outside_tree(&self) -> bool {
        false
    }
}
