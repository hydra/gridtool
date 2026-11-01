use eframe::egui;
use gridtool::action::Action;

use crate::file_picker::{RfdDocumentPicker, RfdImagePicker};
use crate::open_document::OpenDocument;
use crate::session::{Content, Replacement, Session};
use crate::ui::canvas;
use crate::ui::object_tree::{self, TreePick};
use crate::ui::placeholder;
use crate::ui::side_panel::SidePanel;
use crate::ui::tool_controls::ToolControls;
use crate::ui::tool_keys::{ToolKeyPresses, ToolKeys};
use crate::ui::tool_mode::ToolMode;
use crate::ui::top_panel::{self, ToolBarState, TopPanelAction};

pub struct GridToolApp {
    session: Session,
    mode: ToolMode,
    tool_controls: ToolControls,
    side_panel: SidePanel,
    tool_keys: ToolKeys,
    /// The window title last sent to the window.
    title: String,
}

impl GridToolApp {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            mode: ToolMode::default(),
            tool_controls: ToolControls::default(),
            side_panel: SidePanel::default(),
            tool_keys: ToolKeys::default(),
            title: crate::APP_NAME.to_owned(),
        }
    }

    fn tool_bar_state(&self) -> ToolBarState {
        self.session
            .document()
            .map(|open| ToolBarState {
                has_document: true,
                has_unsaved_changes: open.has_unsaved_changes(),
                can_undo: open.editor.can_undo(),
                can_redo: open.editor.can_redo(),
            })
            .unwrap_or_default()
    }

    fn handle(&mut self, ctx: &egui::Context, action: TopPanelAction) {
        let session = &mut self.session;
        match action {
            TopPanelAction::New => session.replace(ctx, Replacement::New),
            TopPanelAction::Load => session.replace(ctx, Replacement::Load),
            TopPanelAction::OpenImage => session.replace(ctx, Replacement::OpenImage),
            TopPanelAction::Save => session.save(ctx),
            TopPanelAction::SaveAs => session.save_as(ctx),
            TopPanelAction::Undo => session.document_mut().map_or((), OpenDocument::undo),
            TopPanelAction::Redo => session.document_mut().map_or((), OpenDocument::redo),
        }
    }

    /// Shows the row under the tool bar, returning any document action from it.
    fn show_tool_controls(&mut self, ui: &mut egui::Ui, min_height: f32) -> Option<Action> {
        let open = self
            .session
            .document_mut()
            .map(|open| (&mut open.view, open.editor.document()));
        self.tool_controls.show(ui, self.mode, min_height, open)
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
        }
    }
}

impl Default for GridToolApp {
    fn default() -> Self {
        Self::new(Session::new(
            Box::new(RfdImagePicker),
            Box::new(RfdDocumentPicker),
        ))
    }
}

impl eframe::App for GridToolApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_close_request(&ctx);
        let state = self.tool_bar_state();
        // A dialog answers Escape itself.
        let keys = match self.session.dialog() {
            Some(_) => ToolKeyPresses::default(),
            None => self.tool_keys.read(&ctx),
        };
        let previous_mode = self.mode;
        let tool_bar = top_panel::show(ui, &mut self.mode, state);
        if let Some(action) = tool_bar.action {
            self.handle(&ctx, action);
        }
        let mut actions: Vec<Action> = self
            .apply_tool_keys(keys, self.mode != previous_mode)
            .into_iter()
            .collect();
        actions.extend(self.show_tool_controls(ui, tool_bar.content_height));

        let tree = self.session.document_mut().map(|open| {
            let view = &mut open.view;
            (
                open.editor.document(),
                &mut view.tree_state,
                &view.selection,
            )
        });
        let objects = object_tree::show(ui, tree);
        actions.extend(objects.action);
        if let Some(pick) = objects.pick
            && let Some(open) = self.session.document_mut()
        {
            open.pick(pick);
        }

        let open = self
            .session
            .document()
            .map(|open| (&open.editor, &open.view.selection));
        let tables = self.side_panel.show(ui, open);
        actions.extend(tables.action);
        if let Some(id) = tables.pick
            && let Some(open) = self.session.document_mut()
        {
            open.pick(TreePick::Object(id));
        }

        let mode = self.mode;
        egui::CentralPanel::default().show(ui, |ui| match self.session.content_mut() {
            Content::Empty => placeholder::show(ui, "Open an image to get started."),
            Content::Error(message) => {
                let color = ui.visuals().error_fg_color;
                placeholder::show(ui, egui::RichText::new(message.as_str()).color(color));
            }
            Content::Document(open) => {
                actions.extend(canvas::show(
                    ui,
                    mode,
                    &mut open.view,
                    open.editor.document(),
                ));
            }
        });

        if let Some(open) = self.session.document_mut() {
            for action in actions {
                open.perform(action);
            }
        }
        // A different active object brings its tool, whose controls show from the next frame.
        if let Some(open) = self.session.document_mut()
            && let Some(mode) = open.view.tool_of_new_active_object(open.editor.document())
        {
            self.mode = mode;
            ctx.request_repaint();
        }

        self.show_dialog(&ctx);
        self.update_title(&ctx);
        self.tool_keys.end_frame(&ctx);
    }
}
