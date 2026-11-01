// A release build opens no console window on Windows. A debug build keeps one, for panics and logs.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod file_picker;
mod open_document;
mod open_image;
mod session;
mod ui;

use eframe::egui;

use app::GridToolApp;

const APP_NAME: &str = "GridTool";
const APP_ICON_PNG: &[u8] = include_bytes!("../assets/icons/gridtool.png");

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size([1280.0, 800.0])
            .with_icon(app_icon()),
        // Only the layout of the panels is restored; see `GridToolApp::save`.
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
            Ok(Box::new(GridToolApp::with_saved_layout(layout.as_deref())))
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
