//! A 0–100 % opacity slider. Dragging previews live and commits one action on release;
//! clicks and keyboard changes commit immediately.

use eframe::egui;

/// What happened to a slider this frame.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct OpacityEdit {
    /// The opacity being chosen, when it changed this frame.
    pub preview: Option<f32>,
    /// The user has finished choosing, so the opacity should be committed.
    pub finished: bool,
}

/// Shows a slider at `displayed`, the opacity as currently previewed.
pub fn show(ui: &mut egui::Ui, displayed: f32) -> OpacityEdit {
    let mut percent = displayed * 100.0;
    let response = ui.add(
        egui::Slider::new(&mut percent, 0.0..=100.0)
            .suffix("%")
            .fixed_decimals(0),
    );
    OpacityEdit {
        preview: response.changed().then_some(percent.round() / 100.0),
        finished: response.drag_stopped() || (response.changed() && !response.dragged()),
    }
}
