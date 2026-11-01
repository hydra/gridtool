use eframe::egui;
use gridtool::action::Action;

use crate::file_picker::{RfdDocumentPicker, RfdImagePicker};
use crate::open_document::OpenDocument;
use crate::session::{DocumentId, Session};
use crate::ui::object_tree::TreePick;
use crate::ui::tool_controls::ToolControls;
use crate::ui::tool_keys::{ToolKeyPresses, ToolKeys};
use crate::ui::tool_mode::ToolMode;
use crate::ui::top_panel::{self, ToolBarState, TopPanelAction};
use crate::ui::workspace::{Panel, Workspace, WorkspaceOutput};

/// Where the panels' arrangement is kept between runs.
const LAYOUT_KEY: &str = "workspace_layout";

/// The panels' arrangement saved by the last run, if any.
pub fn saved_layout(storage: &dyn eframe::Storage) -> Option<String> {
    storage.get_string(LAYOUT_KEY)
}

pub struct GridToolApp {
    session: Session,
    workspace: Workspace,
    /// Holds only what is being edited in the row, so it is reset when the document changes.
    tool_controls: ToolControls,
    tool_keys: ToolKeys,
    /// The window title last sent to the window.
    title: String,
    /// The active document in the previous frame.
    shown_document: Option<DocumentId>,
    /// The document the user clicked. It becomes active after one more frame, in which a field
    /// the click took the focus from finishes its edit on the document it belongs to.
    activating: Option<DocumentId>,
}

impl GridToolApp {
    pub fn new(session: Session, workspace: Workspace) -> Self {
        Self {
            session,
            workspace,
            tool_controls: ToolControls::default(),
            tool_keys: ToolKeys::default(),
            title: crate::APP_NAME.to_owned(),
            shown_document: None,
            activating: None,
        }
    }

    /// The application as it runs, with the panels arranged by a layout from [`saved_layout`].
    pub fn with_saved_layout(layout: Option<&str>) -> Self {
        Self::new(
            Session::new(Box::new(RfdImagePicker), Box::new(RfdDocumentPicker)),
            Workspace::from_saved_layout(layout),
        )
    }

    fn tool_bar_state(&self) -> ToolBarState {
        let state = self
            .session
            .document()
            .map(|open| ToolBarState {
                has_document: true,
                has_unsaved_changes: open.has_unsaved_changes(),
                can_undo: open.editor.can_undo(),
                can_redo: open.editor.can_redo(),
                ..Default::default()
            })
            .unwrap_or_default();
        ToolBarState {
            open_panels: Panel::ALL.map(|panel| self.workspace.is_open(panel)),
            ..state
        }
    }

    fn handle(&mut self, ctx: &egui::Context, action: TopPanelAction) {
        let session = &mut self.session;
        match action {
            TopPanelAction::New => session.open_image(ctx),
            TopPanelAction::Load => session.load(ctx),
            TopPanelAction::Save => session.save(ctx),
            TopPanelAction::SaveAs => session.save_as(ctx),
            TopPanelAction::Undo => session.document_mut().map_or((), OpenDocument::undo),
            TopPanelAction::Redo => session.document_mut().map_or((), OpenDocument::redo),
            TopPanelAction::TogglePanel(panel) => self.workspace.toggle(panel),
            TopPanelAction::ResetLayout => self.workspace.reset_layout(),
        }
        ctx.request_repaint();
    }

    /// Shows the row under the tool bar, returning any document action from it.
    fn show_tool_controls(&mut self, ui: &mut egui::Ui, min_height: f32) -> Option<Action> {
        let mode = self.active_mode();
        let open = self
            .session
            .document_mut()
            .map(|open| (&mut open.view, open.editor.document()));
        self.tool_controls.show(ui, mode, min_height, open)
    }

    /// The active document's tool. Without one there is no active tool, and this is what the
    /// tool bar is told while it shows none.
    fn active_mode(&self) -> ToolMode {
        self.session
            .document()
            .map_or_else(ToolMode::default, |open| open.view.mode)
    }

    /// Escape abandons what the tool was part-way through, and switching tool mode leaves it
    /// altogether. Backspace removes the last point placed; Enter finishes a line being drawn, returning
    /// the action that adds it.
    fn apply_tool_keys(&mut self, keys: ToolKeyPresses, mode_changed: bool) -> Option<Action> {
        let open = self.session.document_mut()?;
        if mode_changed {
            open.view.leave_tool();
        } else if keys.cancel {
            open.view.cancel_tool();
        }
        if keys.remove_last_point {
            open.view.remove_last_point();
        }
        let unit = open.editor.document().calibration().unit();
        keys.finish.then(|| open.view.finish_line(unit)).flatten()
    }

    /// Keeps the window open while the session asks about unsaved changes.
    fn handle_close_request(&mut self, ctx: &egui::Context) {
        if ctx.input(|input| input.viewport().close_requested()) && !self.session.request_close(ctx)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = self.session.window_title();
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    /// Shows the dialog the session is waiting on, if any, and passes on the answer.
    fn show_dialog(&mut self, ctx: &egui::Context) {
        if let Some(button) = self.session.dialog().and_then(|dialog| dialog.show(ctx)) {
            self.session.answer(ctx, button);
            ctx.request_repaint();
        }
    }

    /// What is being edited in the tool controls and tables belongs to the document it was
    /// started on, so none of it carries over to another.
    fn forget_edits_if_the_document_changed(&mut self, ctx: &egui::Context) {
        let active = self.session.active_id();
        if active != self.shown_document {
            self.shown_document = active;
            self.tool_controls = ToolControls::default();
            self.workspace.forget_table_state();
            ctx.memory_mut(egui::Memory::stop_text_input);
        }
    }

    /// Applies what the user did in the workspace, to the active document unless it says
    /// otherwise.
    fn apply(&mut self, ctx: &egui::Context, actions: Vec<Action>, output: WorkspaceOutput) {
        if let Some(open) = self.session.document_mut() {
            for action in actions.into_iter().chain(output.actions) {
                open.perform(action);
            }
            let pick = output.tree_pick.or(output.table_pick.map(TreePick::Object));
            if let Some(pick) = pick {
                open.pick(pick);
            }
        }
        if let Some(id) = self.activating.take() {
            self.session.activate(id);
            ctx.request_repaint();
        }
        if output.activate.is_some() {
            self.activating = output.activate;
            ctx.request_repaint();
        }
        if let Some(id) = output.close {
            self.session.close(ctx, id);
            ctx.request_repaint();
        }
    }

    /// Shows one frame of the whole application.
    fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.workspace.sync(&mut self.session);
        self.forget_edits_if_the_document_changed(&ctx);
        self.handle_close_request(&ctx);
        let state = self.tool_bar_state();
        // A dialog answers Escape itself.
        let keys = match self.session.dialog() {
            Some(_) => ToolKeyPresses::default(),
            None => self.tool_keys.read(&ctx),
        };
        let previous_mode = self.active_mode();
        let mut mode = previous_mode;
        let tool_bar = top_panel::show(ui, &mut mode, state);
        if let Some(open) = self.session.document_mut() {
            open.view.mode = mode;
        }
        let mut actions: Vec<Action> = self
            .apply_tool_keys(keys, mode != previous_mode)
            .into_iter()
            .collect();
        if let Some(action) = tool_bar.action {
            self.handle(&ctx, action);
        }
        actions.extend(self.show_tool_controls(ui, tool_bar.content_height));

        let output = self.workspace.show(ui, &mut self.session);
        self.apply(&ctx, actions, output);
        // A different active object brings its tool, whose controls show from the next frame.
        if let Some(open) = self.session.document_mut()
            && let Some(mode) = open.view.tool_of_new_active_object(open.editor.document())
        {
            open.view.mode = mode;
            ctx.request_repaint();
        }

        self.show_dialog(&ctx);
        self.update_title(&ctx);
        self.tool_keys.end_frame(&ctx);
    }
}

impl eframe::App for GridToolApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(layout) = self.workspace.save_layout() {
            storage.set_string(LAYOUT_KEY, layout);
        }
    }

    /// Only the layout of the panels is restored.
    fn persist_egui_memory(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests;
