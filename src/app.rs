use eframe::egui;
use gridtool::action::Action;
use gridtool::document::Document;
use gridtool::editor::Editor;
use gridtool::imaging::ImageLoadError;

use crate::file_picker::{ImageFilePicker, RfdImagePicker};
use crate::open_image::{OpenedImage, pick_and_decode};
use crate::ui::canvas;
use crate::ui::document_view::DocumentView;
use crate::ui::grid_bar::GridBar;
use crate::ui::measurements_panel::MeasurementsPanel;
use crate::ui::object_tree::{self, TreePick};
use crate::ui::placeholder;
use crate::ui::rotation_bar::RotationBar;
use crate::ui::scale_bar::ScaleBar;
use crate::ui::side_panel;
use crate::ui::tool_mode::ToolMode;
use crate::ui::top_panel::{self, HistoryAvailability, TopPanelAction};

/// A document being edited, with its presentation state.
struct OpenDocument {
    editor: Editor,
    view: DocumentView,
}

impl OpenDocument {
    fn perform(&mut self, action: Action) {
        let activates_new_object = action.activates_new_object();
        let changed = match self.editor.perform(action) {
            Ok(changed) => changed,
            Err(error) => {
                eprintln!("action rejected: {error}");
                false
            }
        };
        let document = self.editor.document();
        self.view.selection.sync_to(document);
        if activates_new_object && changed {
            // A newly added grid or a duplicate becomes the active object.
            self.view.selection.select(document.newest_object());
        }
    }

    /// Applies what the user picked in the object tree: an object becomes the active
    /// object and the view is centred on it.
    fn pick(&mut self, pick: TreePick) {
        let selection = &mut self.view.selection;
        match pick {
            TreePick::Object(id) => {
                selection.select(Some(id));
                let document = self.editor.document();
                if let Some(focus) = document.object_focus_point(id) {
                    let size = egui::Vec2::new(document.width() as f32, document.height() as f32);
                    self.view.viewport.center_on(focus, size);
                }
            }
            TreePick::NoObject => selection.select(None),
        }
    }

    fn undo(&mut self) {
        if self.editor.undo() {
            self.on_history_changed();
        }
    }

    fn redo(&mut self) {
        if self.editor.redo() {
            self.on_history_changed();
        }
    }

    fn on_history_changed(&mut self) {
        self.view
            .scale_tool
            .sync_to(self.editor.document().calibration());
        self.view.selection.sync_to(self.editor.document());
    }

    fn history_availability(&self) -> HistoryAvailability {
        HistoryAvailability {
            can_undo: self.editor.can_undo(),
            can_redo: self.editor.can_redo(),
        }
    }
}

/// What the central area currently holds.
enum Content {
    Empty,
    Error(String),
    Document(Box<OpenDocument>),
}

impl Content {
    fn document(&self) -> Option<&OpenDocument> {
        match self {
            Content::Document(open) => Some(open),
            Content::Empty | Content::Error(_) => None,
        }
    }

    fn document_mut(&mut self) -> Option<&mut OpenDocument> {
        match self {
            Content::Document(open) => Some(open),
            Content::Empty | Content::Error(_) => None,
        }
    }
}

pub struct GridToolApp {
    picker: Box<dyn ImageFilePicker>,
    mode: ToolMode,
    content: Content,
    rotation_bar: RotationBar,
    scale_bar: ScaleBar,
    grid_bar: GridBar,
    measurements_panel: MeasurementsPanel,
}

impl GridToolApp {
    pub fn new(picker: Box<dyn ImageFilePicker>) -> Self {
        Self {
            picker,
            mode: ToolMode::default(),
            content: Content::Empty,
            rotation_bar: RotationBar::default(),
            scale_bar: ScaleBar::default(),
            grid_bar: GridBar::default(),
            measurements_panel: MeasurementsPanel::default(),
        }
    }

    fn handle(&mut self, ctx: &egui::Context, action: TopPanelAction) {
        match action {
            TopPanelAction::OpenImage => {
                if let Some(result) = pick_and_decode(self.picker.as_ref()) {
                    self.open(ctx, result);
                }
            }
            TopPanelAction::Undo => self.content.document_mut().map_or((), OpenDocument::undo),
            TopPanelAction::Redo => self.content.document_mut().map_or((), OpenDocument::redo),
        }
    }

    fn open(&mut self, ctx: &egui::Context, result: Result<OpenedImage, ImageLoadError>) {
        self.content = match result {
            Ok(opened) => {
                let name = opened.path.display().to_string();
                Content::Document(Box::new(OpenDocument {
                    editor: Editor::new(Document::sized_to(&opened.image)),
                    view: DocumentView::new(canvas::upload_texture(ctx, &name, &opened.image)),
                }))
            }
            Err(error) => Content::Error(error.to_string()),
        };
    }

    /// Shows the options row for the active tool, returning any document action from it.
    fn show_tool_options(&mut self, ui: &mut egui::Ui) -> Option<Action> {
        let open = self.content.document_mut()?;
        match self.mode {
            ToolMode::Pan | ToolMode::Measure | ToolMode::Origin => None,
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
        let open = self.content.document_mut()?;
        self.grid_bar.show(
            ui,
            open.editor.document(),
            &mut open.view.selection,
            open.view.viewport.zoom(),
        )
    }
}

impl Default for GridToolApp {
    fn default() -> Self {
        Self::new(Box::new(RfdImagePicker))
    }
}

impl eframe::App for GridToolApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let history = self
            .content
            .document()
            .map(OpenDocument::history_availability)
            .unwrap_or_default();
        if let Some(action) = top_panel::show(ui, &mut self.mode, history) {
            let ctx = ui.ctx().clone();
            self.handle(&ctx, action);
        }

        let mut actions: Vec<Action> = self.show_tool_options(ui).into_iter().collect();
        actions.extend(self.show_object_properties(ui));

        let tree = self.content.document_mut().map(|open| {
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
            && let Some(open) = self.content.document_mut()
        {
            open.pick(pick);
        }

        let open = self
            .content
            .document()
            .map(|open| (&open.editor, &open.view.selection));
        actions.extend(side_panel::show(ui, &mut self.measurements_panel, open));

        egui::CentralPanel::default().show(ui, |ui| match &mut self.content {
            Content::Empty => placeholder::show(ui, "Open an image to get started."),
            Content::Error(message) => {
                let color = ui.visuals().error_fg_color;
                placeholder::show(ui, egui::RichText::new(message.as_str()).color(color));
            }
            Content::Document(open) => {
                actions.extend(canvas::show(
                    ui,
                    self.mode,
                    &mut open.view,
                    open.editor.document(),
                ));
            }
        });

        if let Some(open) = self.content.document_mut() {
            for action in actions {
                open.perform(action);
            }
        }
    }
}
