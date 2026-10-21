//! Presentation layer. Reads the document and emits [`gridtool::action::Action`]s; never
//! modifies the document directly.

mod angle;
pub mod canvas;
mod commit_field;
pub mod document_view;
mod format;
mod history_panel;
mod hit_test;
mod icons;
mod measure_tool;
pub mod measurements_panel;
pub mod object_tree;
pub mod placeholder;
mod rotate_handle;
pub mod rotation_bar;
pub mod scale_bar;
mod scale_tool;
mod selection;
pub mod side_panel;
pub mod tool_mode;
pub mod top_panel;
mod unit_picker;
mod viewport;
