//! A colour swatch button that opens a colour picker. The colour being chosen is shown
//! live and committed once, when the picker closes.

use eframe::egui::{self, Popup, PopupCloseBehavior};
use gridtool::color::Rgb;

use super::color::{from_color32, to_color32};

/// The size of the swatches in table rows.
pub const ROW_SWATCH_SIZE: egui::Vec2 = egui::Vec2::new(28.0, 14.0);

/// What happened to a swatch this frame.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ColorEdit {
    /// The colour being chosen, when it changed this frame.
    pub preview: Option<Rgb>,
    /// The picker closed this frame.
    pub closed: bool,
    /// The chosen colour, when the picker closed on a colour other than the current one.
    pub committed: Option<Rgb>,
}

/// Shows a swatch of `size` filled with `current` (or, while its picker is open, the
/// colour being chosen). Clicking it opens the picker; clicking outside closes it.
pub fn show(
    ui: &mut egui::Ui,
    id: egui::Id,
    current: Rgb,
    size: egui::Vec2,
    tooltip: &str,
) -> ColorEdit {
    let popup_id = id.with("color_popup");
    let was_open = Popup::is_id_open(ui.ctx(), popup_id);
    let working = was_open
        .then(|| ui.data(|data| data.get_temp::<Rgb>(popup_id)))
        .flatten()
        .unwrap_or(current);
    let mut color = to_color32(working);

    let swatch = ui
        .add(egui::Button::new("").fill(color).min_size(size))
        .on_hover_text(tooltip);
    let mut changed = false;
    Popup::menu(&swatch)
        .id(popup_id)
        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            changed = egui::color_picker::color_picker_color32(
                ui,
                &mut color,
                egui::color_picker::Alpha::Opaque,
            );
        });
    let chosen = from_color32(color);
    // Remember the colour being chosen only while the picker is open, so it starts from
    // `current` every time it opens.
    let open = Popup::is_id_open(ui.ctx(), popup_id);
    ui.data_mut(|data| {
        if open {
            data.insert_temp(popup_id, chosen);
        } else {
            data.remove::<Rgb>(popup_id);
        }
    });

    let closed = was_open && !open;
    ColorEdit {
        preview: changed.then_some(chosen),
        closed,
        committed: (closed && chosen != current).then_some(chosen),
    }
}
