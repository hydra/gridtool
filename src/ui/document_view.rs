use eframe::egui::TextureHandle;
use egui_ltreeview::TreeViewState;

use super::area_tool::AreaTool;
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
    pub area_tool: AreaTool,
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
            area_tool: AreaTool::default(),
            grid_tool: GridTool::default(),
            is_dragging_rotate_handle: false,
            tree_state: TreeViewState::default(),
        }
    }

    /// Abandons whatever a tool is part-way through, without committing anything:
    /// pending scale, measurement or area points, a scale point drag, a drag of the active
    /// object, and a rotate-handle drag. Used by Escape and when switching tool mode.
    pub fn cancel_tool(&mut self) {
        self.scale_tool.cancel();
        self.measure_tool.cancel();
        self.area_tool.cancel();
        self.selection.cancel_drag();
        self.is_dragging_rotate_handle = false;
    }

    /// Removes the most recently placed point of the area being placed (Backspace).
    pub fn remove_last_point(&mut self) {
        self.area_tool.remove_last_point();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui;
    use gridtool::document::Document;
    use gridtool::geometry::Point;
    use gridtool::imaging::DecodedImage;
    use gridtool::units::LengthUnit;

    use crate::ui::canvas::upload_texture;

    fn view() -> DocumentView {
        let image = DecodedImage {
            width: 1,
            height: 1,
            rgba: vec![255; 4],
        };
        DocumentView::new(upload_texture(&egui::Context::default(), "test", &image))
    }

    #[test]
    fn cancel_tool_abandons_every_operation_in_progress() {
        let document = Document::new(100, 100);
        let mut view = view();
        view.scale_tool
            .place(document.calibration(), Point::new(1.0, 1.0));
        view.measure_tool
            .place(Point::new(2.0, 2.0), LengthUnit::Metre);
        view.is_dragging_rotate_handle = true;

        view.cancel_tool();

        assert!(view.scale_tool.points(document.calibration()).is_empty());
        assert_eq!(view.measure_tool.pending_start(), None);
        assert!(!view.is_dragging_rotate_handle);
    }
}
