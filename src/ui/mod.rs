//! Presentation layer. Reads the document and emits [`gridtool::action::Action`]s; never
//! modifies the document directly.

mod angle;
pub mod canvas;
pub mod document_view;
mod format;
pub mod history_panel;
mod hit_test;
pub mod placeholder;
mod rotate_handle;
pub mod rotation_bar;
pub mod scale_bar;
mod scale_tool;
pub mod tool_mode;
pub mod top_panel;
mod viewport;
