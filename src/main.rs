mod app;
mod file_picker;
mod open_document;
mod open_image;
mod session;
mod ui;

use eframe::egui;

use app::GridToolApp;

const APP_NAME: &str = "GridTool";

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Ok(Box::<GridToolApp>::default())),
    )
}
