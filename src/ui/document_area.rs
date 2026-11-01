//! The Documents area: one tab for each open document, which the user can drag between
//! groups, and split horizontally and vertically. Everything here is a layout of documents;
//! the documents themselves belong to the session.

use eframe::egui;
use egui_tiles::{
    Behavior, Container, ContainerKind, SimplificationOptions, Tile, TileId, Tiles, Tree,
    UiResponse,
};

use crate::session::{DocumentId, Session};

use super::canvas;
use super::placeholder;
use super::workspace::WorkspaceOutput;

/// The outline around the active document, when others are also in view.
const ACTIVE_OUTLINE_WIDTH: f32 = 2.0;

pub struct DocumentArea {
    tree: Tree<DocumentId>,
    /// The active document when last synced with the session.
    last_active: Option<DocumentId>,
}

impl Default for DocumentArea {
    fn default() -> Self {
        Self {
            tree: Tree::empty(egui::Id::unique("document_area")),
            last_active: None,
        }
    }
}

impl DocumentArea {
    /// Brings the layout up to date with the session's documents, and agrees with it which
    /// document is active. Call once per frame, before the tool bar and panels are shown.
    pub fn sync(&mut self, session: &mut Session) {
        let group_of_closed = self.remove_closed(session);
        let opened: Vec<_> = session
            .document_ids()
            .filter(|id| self.tile_of(*id).is_none())
            .collect();
        for id in opened {
            self.add(id);
        }
        match session.active_id() {
            None => {
                if let Some(id) = self.document_to_activate(group_of_closed) {
                    session.activate(id);
                    self.reveal(id);
                }
            }
            // The session chose a document, so it must be in view.
            Some(active) if Some(active) != self.last_active => self.reveal(active),
            // The user brought another tab forward, hiding the active document.
            Some(active) if !self.is_visible(active) => {
                let group = self.tile_of(active).and_then(|tile| self.parent(tile));
                if let Some(id) = self.document_to_activate(group) {
                    session.activate(id);
                }
            }
            Some(_) => {}
        }
        self.last_active = session.active_id();
    }

    /// Removes the tabs of closed documents, returning the group of the active one, if it was
    /// among them.
    fn remove_closed(&mut self, session: &Session) -> Option<TileId> {
        let closed: Vec<_> = self
            .tree
            .tiles
            .iter()
            .filter_map(|(tile, content)| match content {
                Tile::Pane(id) if session.document_by_id(*id).is_none() => Some((*tile, *id)),
                _ => None,
            })
            .collect();
        let mut group = None;
        for (tile, id) in closed {
            if Some(id) == self.last_active {
                group = self.parent(tile);
            }
            self.tree.remove_recursively(tile);
        }
        group
    }

    /// Adds a tab for a newly opened document, in the group of the document that was active.
    fn add(&mut self, id: DocumentId) {
        let tile = self.tree.tiles.insert_pane(id);
        let group = self
            .last_active
            .and_then(|active| self.tile_of(active))
            .and_then(|active| self.parent(active));
        if let Some(group) = group
            && let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get_mut(group)
        {
            tabs.add_child(tile);
            tabs.set_active(tile);
        } else if let Some(root) = self.tree.root {
            // There is no such group: beside everything else, in its own.
            let tabs = self.tree.tiles.insert_tab_tile(vec![tile]);
            self.tree.root = Some(self.tree.tiles.insert_horizontal_tile(vec![root, tabs]));
        } else {
            self.tree.root = Some(self.tree.tiles.insert_tab_tile(vec![tile]));
        }
    }

    /// The document that should become active: the one in view in `group`, else any in view,
    /// else any.
    fn document_to_activate(&mut self, group: Option<TileId>) -> Option<DocumentId> {
        let in_group = group.and_then(|group| match self.tree.tiles.get_container(group) {
            Some(Container::Tabs(tabs)) => tabs.next_active(&self.tree.tiles),
            _ => None,
        });
        in_group
            .into_iter()
            .chain(self.tree.active_tiles())
            .chain(self.tree.tiles.tile_ids())
            .find_map(|tile| self.tree.tiles.get_pane(&tile).copied())
    }

    fn reveal(&mut self, id: DocumentId) {
        self.tree
            .make_active(|_, tile| matches!(tile, Tile::Pane(pane) if *pane == id));
    }

    fn tile_of(&self, id: DocumentId) -> Option<TileId> {
        self.tree.tiles.find_pane(&id)
    }

    fn parent(&self, tile: TileId) -> Option<TileId> {
        self.tree.tiles.parent_of(tile)
    }

    fn is_visible(&self, id: DocumentId) -> bool {
        self.tile_of(id)
            .is_some_and(|tile| self.tree.active_tiles().contains(&tile))
    }

    /// Shows the tabs and canvases, or how to get started if no document is open.
    pub fn show(&mut self, ui: &mut egui::Ui, session: &mut Session, output: &mut WorkspaceOutput) {
        if self.tree.is_empty() {
            placeholder::show(ui, "Open an image to get started.");
            return;
        }
        let documents_in_view = self
            .tree
            .active_tiles()
            .iter()
            .filter(|tile| self.tree.tiles.get_pane(tile).is_some())
            .count();
        let mut behavior = AreaBehavior {
            session,
            output,
            outline_active: documents_in_view > 1,
        };
        self.tree.ui(&mut behavior, ui);
    }

    /// The groups of tabs, each listing its documents in tab order. For tests.
    #[cfg(test)]
    pub fn groups(&self) -> Vec<Vec<DocumentId>> {
        let mut groups: Vec<Vec<DocumentId>> = self
            .tree
            .tiles
            .tiles()
            .filter_map(|tile| match tile {
                Tile::Container(Container::Tabs(tabs)) => Some(
                    tabs.children
                        .iter()
                        .filter_map(|child| self.tree.tiles.get_pane(child).copied())
                        .collect(),
                ),
                _ => None,
            })
            .collect();
        groups.sort();
        groups
    }

    /// The documents in view. For tests.
    #[cfg(test)]
    pub fn visible_documents(&self) -> Vec<DocumentId> {
        let mut visible: Vec<_> = self
            .tree
            .active_tiles()
            .iter()
            .filter_map(|tile| self.tree.tiles.get_pane(tile).copied())
            .collect();
        visible.sort();
        visible
    }

    /// Where the `document`'s tab is, if its group has been drawn. For tests.
    #[cfg(test)]
    pub fn canvas_rect_of(&self, document: DocumentId) -> Option<egui::Rect> {
        self.tree.tiles.rect(self.tile_of(document)?)
    }

    /// Splits the `document` off beside the group it is in, as dragging its tab would. For tests.
    #[cfg(test)]
    pub fn split_off(&mut self, document: DocumentId) {
        let tile = self.tile_of(document).expect("the document has a tab");
        let root = self.tree.root.expect("there is a tab");
        let group = self.tree.tiles.insert_tab_tile(Vec::new());
        self.tree.root = Some(self.tree.tiles.insert_horizontal_tile(vec![root, group]));
        self.tree.move_tile_to_container(tile, group, 0, false);
    }
}

struct AreaBehavior<'a> {
    session: &'a mut Session,
    output: &'a mut WorkspaceOutput,
    outline_active: bool,
}

impl Behavior<DocumentId> for AreaBehavior<'_> {
    fn pane_ui(&mut self, ui: &mut egui::Ui, _tile: TileId, id: &mut DocumentId) -> UiResponse {
        ui.shrink_clip_rect(ui.max_rect());
        let rect = ui.max_rect();
        let is_active = self.session.active_id() == Some(*id);
        if let Some(open) = self.session.document_by_id_mut(*id) {
            let mode = open.view.mode;
            let shown = canvas::show(ui, mode, &mut open.view, open.editor.document(), is_active);
            self.output.actions.extend(shown.action);
            if shown.activate {
                self.output.activate = Some(*id);
            }
        }
        if is_active && self.outline_active {
            let stroke =
                egui::Stroke::new(ACTIVE_OUTLINE_WIDTH, ui.visuals().selection.stroke.color);
            ui.painter()
                .rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
        }
        UiResponse::None
    }

    fn tab_title_for_pane(&mut self, id: &DocumentId) -> egui::WidgetText {
        self.session
            .document_by_id(*id)
            .map_or_else(String::new, |open| {
                let unsaved = if open.has_unsaved_changes() { "*" } else { "" };
                format!("{}{unsaved}", open.name())
            })
            .into()
    }

    /// Tabs look draggable as they are.
    fn tab_hover_cursor_icon(&self) -> egui::CursorIcon {
        egui::CursorIcon::Default
    }

    fn is_tab_closable(&self, _tiles: &Tiles<DocumentId>, _tile: TileId) -> bool {
        true
    }

    /// The session decides whether the document closes, which may first take a question.
    fn on_tab_close(&mut self, tiles: &mut Tiles<DocumentId>, tile: TileId) -> bool {
        self.output.close = tiles.get_pane(&tile).copied();
        false
    }

    fn on_tab_button(
        &mut self,
        tiles: &mut Tiles<DocumentId>,
        tile: TileId,
        button: egui::Response,
    ) -> egui::Response {
        if (button.clicked() || button.drag_started())
            && let Some(id) = tiles.get_pane(&tile)
        {
            self.output.activate = Some(*id);
        }
        button
    }

    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn allows_drop_on_tile(&self, _tile: TileId, kind: ContainerKind) -> bool {
        kind != ContainerKind::Grid
    }

    /// A tab let go of over a panel has no place in the Documents area, so it stays put.
    fn allows_drops_outside_tree(&self) -> bool {
        false
    }
}
