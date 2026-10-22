use eframe::egui;
use gridtool::action::Action;

use crate::file_picker::{RfdDocumentPicker, RfdImagePicker};
use crate::open_document::OpenDocument;
use crate::session::{Content, Replacement, Session};
use crate::ui::canvas;
use crate::ui::grid_bar::GridBar;
use crate::ui::object_tree;
use crate::ui::placeholder;
use crate::ui::rotation_bar::RotationBar;
use crate::ui::scale_bar::ScaleBar;
use crate::ui::side_panel::SidePanel;
use crate::ui::tool_keys::{ToolKeyPresses, ToolKeys};
use crate::ui::tool_mode::ToolMode;
use crate::ui::top_panel::{self, ToolBarState, TopPanelAction};

pub struct GridToolApp {
    session: Session,
    mode: ToolMode,
    rotation_bar: RotationBar,
    scale_bar: ScaleBar,
    grid_bar: GridBar,
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
            rotation_bar: RotationBar::default(),
            scale_bar: ScaleBar::default(),
            grid_bar: GridBar::default(),
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

    /// Shows the options row for the active tool, returning any document action from it.
    fn show_tool_options(&mut self, ui: &mut egui::Ui) -> Option<Action> {
        let open = self.session.document_mut()?;
        match self.mode {
            ToolMode::Pan | ToolMode::Measure | ToolMode::Area | ToolMode::Origin => None,
            ToolMode::Rotate => {
                self.rotation_bar
                    .show(ui, &mut open.view.viewport, open.editor.document());
                None
            }
            ToolMode::Scale => {
                self.scale_bar
                    .show(ui, open.editor.document(), &mut open.view.scale_tool)
            }
            ToolMode::Grid => {
                open.view.grid_tool.show_options(ui, open.editor.document());
                None
            }
        }
    }

    /// Shows the properties row of the active object, if it has one (grids do).
    fn show_object_properties(&mut self, ui: &mut egui::Ui) -> Option<Action> {
        let open = self.session.document_mut()?;
        self.grid_bar.show(
            ui,
            open.editor.document(),
            &mut open.view.selection,
            open.view.viewport.zoom(),
        )
    }

    /// Escape and switching tool mode both abandon what the tool was part-way through;
    /// Backspace removes the last point placed.
    fn apply_tool_keys(&mut self, keys: ToolKeyPresses, mode_changed: bool) {
        let Some(open) = self.session.document_mut() else {
            return;
        };
        if keys.cancel || mode_changed {
            open.view.cancel_tool();
        }
        if keys.remove_last_point {
            open.view.remove_last_point();
        }
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
        if let Some(action) = top_panel::show(ui, &mut self.mode, state) {
            self.handle(&ctx, action);
        }
        self.apply_tool_keys(keys, self.mode != previous_mode);

        let mut actions: Vec<Action> = self.show_tool_options(ui).into_iter().collect();
        actions.extend(self.show_object_properties(ui));

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
        actions.extend(self.side_panel.show(ui, open));

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

        self.show_dialog(&ctx);
        self.update_title(&ctx);
        self.tool_keys.end_frame(&ctx);
    }
}
