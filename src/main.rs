mod angle;
mod app;
mod document;
mod file_picker;
mod imaging;
mod open_image;
mod rotate_handle;
mod tool_mode;
mod ui;
mod viewport;

use eframe::egui;

use app::GridToolApp;

const APP_NAME: &str = "GridTool";

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|_cc| Ok(Box::<GridToolApp>::default())),
    )
}
