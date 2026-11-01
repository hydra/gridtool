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
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| {
            egui_material_icons::initialize(&cc.egui_ctx);
            ui::app_style::apply(&cc.egui_ctx);
            Ok(Box::<GridToolApp>::default())
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
