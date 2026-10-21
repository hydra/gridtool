//! Headless core of GridTool: the document model and every action that can be applied
//! to it, with undo/redo history.
//!
//! Nothing in this crate depends on the GUI, so all document operations can be driven
//! from unit tests or a command-line front end.

pub mod action;
pub mod calibration;
pub mod document;
pub mod editor;
pub mod geometry;
pub mod imaging;
pub mod measurement;
pub mod object;
pub mod units;
