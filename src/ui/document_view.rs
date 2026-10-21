use eframe::egui::TextureHandle;

use super::scale_tool::ScaleTool;
use super::viewport::Viewport;

/// Presentation state for one open document: its image texture, how it is viewed, and
/// in-progress tool interactions. Nothing here is part of the document.
pub struct DocumentView {
    pub texture: TextureHandle,
    pub viewport: Viewport,
    pub scale_tool: ScaleTool,
    pub is_dragging_rotate_handle: bool,
}

impl DocumentView {
    pub fn new(texture: TextureHandle) -> Self {
        Self {
            texture,
            viewport: Viewport::default(),
            scale_tool: ScaleTool::default(),
            is_dragging_rotate_handle: false,
        }
    }
}
