use eframe::egui::TextureHandle;
use egui_ltreeview::TreeViewState;

use super::grid_tool::GridTool;
use super::measure_tool::MeasureTool;
use super::object_tree::TreeNode;
use super::scale_tool::ScaleTool;
use super::selection::Selection;
use super::viewport::Viewport;

/// Presentation state for one open document: its image texture, how it is viewed, the
/// active object, and in-progress tool interactions. Nothing here is part of the document.
pub struct DocumentView {
    pub texture: TextureHandle,
    pub viewport: Viewport,
    pub selection: Selection,
    pub scale_tool: ScaleTool,
    pub measure_tool: MeasureTool,
    pub grid_tool: GridTool,
    pub is_dragging_rotate_handle: bool,
    pub tree_state: TreeViewState<TreeNode>,
}

impl DocumentView {
    pub fn new(texture: TextureHandle) -> Self {
        Self {
            texture,
            viewport: Viewport::default(),
            selection: Selection::default(),
            scale_tool: ScaleTool::default(),
            measure_tool: MeasureTool::default(),
            grid_tool: GridTool::default(),
            is_dragging_rotate_handle: false,
            tree_state: TreeViewState::default(),
        }
    }
}
