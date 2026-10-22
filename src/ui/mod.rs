//! Presentation layer. Reads the document and emits [`gridtool::action::Action`]s; never
//! modifies the document directly.

mod angle;
mod area_tool;
pub mod areas_panel;
pub mod canvas;
mod color;
mod color_swatch;
mod commit_field;
pub mod dialog;
pub mod document_view;
mod format;
pub mod grid_bar;
mod grid_display;
mod grid_tool;
mod history_panel;
mod hit_test;
mod icon_button;
mod icons;
mod line_hit;
mod measure_tool;
pub mod measurements_panel;
mod object_drag;
pub mod object_tree;
pub mod placeholder;
mod rotate_handle;
pub mod rotation_bar;
pub mod scale_bar;
mod scale_tool;
mod selection;
pub mod side_panel;
pub mod tool_keys;
pub mod tool_mode;
pub mod top_panel;
mod unit_picker;
mod vertical_split;
mod viewport;
