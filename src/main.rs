// A release build opens no console window on Windows. A debug build keeps one, for panics and logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod crash_report;
mod file_picker;
mod open_document;
mod open_image;
mod preferences;
#[cfg(test)]
mod readme_about;
mod session;
mod ui;

use eframe::egui;

use app::GridToolApp;
use preferences::Preferences;

const APP_NAME: &str = "GridTool";
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON_PNG: &[u8] = include_bytes!("../assets/icons/gridtool.png");

fn main() -> eframe::Result {
    crash_report::install();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size([1280.0, 800.0])
            .with_icon(app_icon()),
        // Only the layout of the panels and the preferences are restored; see `GridToolApp::save`.
        persist_window: false,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            egui_material_icons::initialize(&cc.egui_ctx);
            ui::app_style::apply(&cc.egui_ctx);
            let layout = cc.storage.and_then(app::saved_layout);
            let preferences = cc
                .storage
                .map_or_else(Preferences::default, Preferences::load);
            preferences.apply(&cc.egui_ctx);
            Ok(Box::new(GridToolApp::with_saved_state(
                layout.as_deref(),
                preferences,
            )))
        }),
    )
}

fn app_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(APP_ICON_PNG).expect("the bundled icon is a valid PNG")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_app_icon_is_a_square_image() {
        let icon = app_icon();

        assert!(icon.width > 0);
        assert_eq!(icon.width, icon.height);
    }
}
