use eframe::egui::{Rect, TextureHandle};
use egui_ltreeview::TreeViewState;
use gridtool::action::Action;
use gridtool::units::LengthUnit;

use super::area_tool::AreaTool;
use super::grid_tool::GridTool;
use super::line_tool::LineTool;
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
    pub line_tool: LineTool,
    pub is_dragging_rotate_handle: bool,
    pub tree_state: TreeViewState<TreeNode>,
    /// Where the canvas was drawn last frame, in screen points; `None` before it first is.
    pub canvas_rect: Option<Rect>,
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
            line_tool: LineTool::default(),
            is_dragging_rotate_handle: false,
            tree_state: TreeViewState::default(),
            canvas_rect: None,
        }
    }

    /// Abandons whatever a tool is part-way through, without committing anything:
    /// pending scale, measurement, area or line points, a scale point drag, a drag of the
    /// active object, and a rotate-handle drag. Used by Escape and when switching tool mode.
    pub fn cancel_tool(&mut self) {
        self.scale_tool.cancel();
        self.measure_tool.cancel();
        self.area_tool.outline.cancel();
        self.line_tool.outline.cancel();
        self.selection.cancel_drag();
        self.is_dragging_rotate_handle = false;
    }

    /// Removes the most recently placed point of the area or line being placed (Backspace).
    pub fn remove_last_point(&mut self) {
        self.area_tool.outline.remove_last_point();
        self.line_tool.outline.remove_last_point();
    }

    /// Finishes the line being drawn, leaving it open (Enter): the action that adds it, if
    /// there is enough of a line to add.
    pub fn finish_line(&mut self, unit: LengthUnit) -> Option<Action> {
        self.line_tool.finish_open(unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui;
    use gridtool::document::Document;
    use gridtool::geometry::Point;
    use gridtool::imaging::DecodedImage;

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
        view.area_tool.outline.click(Point::new(3.0, 3.0), None);
        view.line_tool.outline.click(Point::new(4.0, 4.0), None);
        view.is_dragging_rotate_handle = true;

        view.cancel_tool();

        assert!(view.scale_tool.points(document.calibration()).is_empty());
        assert_eq!(view.measure_tool.pending_start(), None);
        assert!(view.area_tool.outline.points().is_empty());
        assert!(view.line_tool.outline.points().is_empty());
        assert!(!view.is_dragging_rotate_handle);
    }

    #[test]
    fn finishing_a_line_needs_two_points_and_backspace_removes_them() {
        let mut view = view();
        view.line_tool.outline.click(Point::new(1.0, 1.0), None);
        assert_eq!(view.finish_line(LengthUnit::Metre), None);

        view.line_tool.outline.click(Point::new(2.0, 2.0), None);
        view.remove_last_point();
        assert_eq!(view.line_tool.outline.points().len(), 1);

        view.line_tool.outline.click(Point::new(3.0, 3.0), None);
        assert!(matches!(
            view.finish_line(LengthUnit::Metre),
            Some(Action::AddShape(_))
        ));
        assert!(view.line_tool.outline.points().is_empty());
    }
}
