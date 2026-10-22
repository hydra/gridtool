use std::fmt::Debug;
use std::hash::Hash;

use eframe::egui;
use gridtool::units::LengthUnit;

const WIDTH: f32 = 50.0;

/// A drop-down of length units ("cm", "m"). Returns the newly chosen unit if the user
/// picked one different from `current`.
pub fn show(
    ui: &mut egui::Ui,
    id_salt: impl Hash + Debug,
    current: LengthUnit,
    tooltip: &str,
) -> Option<LengthUnit> {
    picker(ui, id_salt, current, tooltip, LengthUnit::symbol)
}

/// A drop-down of area units ("cm²", "m²"), returning the length unit they are the
/// square of, as [`show`] does.
pub fn show_area(
    ui: &mut egui::Ui,
    id_salt: impl Hash + Debug,
    current: LengthUnit,
    tooltip: &str,
) -> Option<LengthUnit> {
    picker(ui, id_salt, current, tooltip, LengthUnit::area_symbol)
}

fn picker(
    ui: &mut egui::Ui,
    id_salt: impl Hash + Debug,
    current: LengthUnit,
    tooltip: &str,
    symbol: fn(LengthUnit) -> &'static str,
) -> Option<LengthUnit> {
    let mut selected = current;
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(symbol(current))
        .width(WIDTH)
        .show_ui(ui, |ui| {
            for unit in LengthUnit::ALL {
                ui.selectable_value(&mut selected, unit, symbol(unit));
            }
        })
        .response
        .on_hover_text(tooltip);
    (selected != current).then_some(selected)
}
