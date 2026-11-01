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

    /// Shows one frame of the whole application.
    fn show(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        // With no document there is no active tool, so the next document starts afresh.
        if self.session.document().is_none() {
            self.mode = ToolMode::default();
        }
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
        self.show(ui);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::file_picker::{DocumentFilePicker, ImageFilePicker};

    const WINDOW: egui::Rect =
        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::Pos2::new(1280.0, 800.0));
    const IMAGE_WIDTH: u32 = 2000;
    const IMAGE_HEIGHT: u32 = 1500;

    /// Always picks the same files, so that the file dialogs need no user.
    #[derive(Clone)]
    struct Picks {
        image: PathBuf,
        document: PathBuf,
    }

    impl ImageFilePicker for Picks {
        fn pick_image(&self) -> Option<PathBuf> {
            Some(self.image.clone())
        }
    }

    impl DocumentFilePicker for Picks {
        fn pick_document(&self) -> Option<PathBuf> {
            Some(self.document.clone())
        }

        fn pick_save_path(&self, _suggestion: &Path) -> Option<PathBuf> {
            None
        }
    }

    /// The whole application, with a temporary image bigger than its window to open.
    struct Fixture {
        folder: PathBuf,
        ctx: egui::Context,
        app: GridToolApp,
        picks: Picks,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let folder =
                std::env::temp_dir().join(format!("gridtool_app_{name}_{}", std::process::id()));
            fs::create_dir_all(&folder).unwrap();
            let picks = Picks {
                image: folder.join("plan.png"),
                document: folder.join("plan.ron"),
            };
            image::RgbaImage::new(IMAGE_WIDTH, IMAGE_HEIGHT)
                .save(&picks.image)
                .unwrap();
            let session = Session::new(Box::new(picks.clone()), Box::new(picks.clone()));
            Self {
                folder,
                ctx: egui::Context::default(),
                app: GridToolApp::new(session),
                picks,
            }
        }

        fn frame(&mut self) {
            let input = egui::RawInput {
                screen_rect: Some(WINDOW),
                ..Default::default()
            };
            let ctx = self.ctx.clone();
            let mut output = ctx.run_ui(input, |ui| self.app.show(ui));
            // There is no renderer to upload textures to.
            output.textures_delta.clear();
        }

        fn replace(&mut self, replacement: Replacement) {
            let ctx = self.ctx.clone();
            self.app.session.replace(&ctx, replacement);
        }

        fn open(&mut self) -> &mut OpenDocument {
            self.app.session.document_mut().expect("a document is open")
        }

        /// Shows frames as the running application would, then checks that the whole document
        /// is inside the canvas area.
        fn assert_document_fits_the_canvas(&mut self) {
            for _ in 0..4 {
                self.frame();
            }
            let view = &self.open().view;
            let canvas = view.canvas_rect.expect("the canvas was drawn");
            let size = egui::Vec2::new(IMAGE_WIDTH as f32, IMAGE_HEIGHT as f32);
            let corners = view.viewport.transform(canvas.center(), size).corners();
            assert!(
                corners.iter().all(|corner| canvas.contains(*corner)),
                "{corners:?} outside {canvas:?}"
            );
            assert!(view.viewport.zoom() < 1.0);
            assert!(
                canvas.width() < WINDOW.width(),
                "the panels take their share of the window"
            );
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.folder);
        }
    }

    #[test]
    fn an_image_opened_in_a_new_document_fits_the_canvas() {
        let mut fixture = Fixture::new("open_image");
        fixture.frame();

        fixture.replace(Replacement::OpenImage);

        fixture.assert_document_fits_the_canvas();
    }

    #[test]
    fn a_loaded_document_fits_the_canvas() {
        let mut fixture = Fixture::new("load");
        fixture.replace(Replacement::OpenImage);
        let path = fixture.picks.document.clone();
        fixture.open().save_to(&path).unwrap();
        // Zoomed in, so that the fit below can only come from loading.
        fixture.frame();
        fixture.frame();
        let center = fixture.open().view.canvas_rect.unwrap().center();
        fixture
            .open()
            .view
            .viewport
            .zoom_about(center, center, 100.0);

        fixture.replace(Replacement::Load);

        fixture.assert_document_fits_the_canvas();
    }

    #[test]
    fn zooming_after_opening_is_kept() {
        let mut fixture = Fixture::new("zoom_kept");
        fixture.replace(Replacement::OpenImage);
        fixture.assert_document_fits_the_canvas();
        let center = fixture.open().view.canvas_rect.unwrap().center();
        let fitted = fixture.open().view.viewport.zoom();
        fixture.open().view.viewport.zoom_about(center, center, 2.0);

        fixture.frame();

        assert_eq!(fixture.open().view.viewport.zoom(), fitted * 2.0);
    }

    #[test]
    fn fitting_leaves_the_document_and_its_history_alone() {
        let mut fixture = Fixture::new("document_unchanged");
        fixture.replace(Replacement::OpenImage);
        let before = fixture.open().editor.document().clone();

        fixture.assert_document_fits_the_canvas();

        assert_eq!(fixture.open().editor.document(), &before);
        assert!(!fixture.open().editor.can_undo());
    }
}
