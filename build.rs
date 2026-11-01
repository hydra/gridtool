use std::error::Error;
use std::fs::File;
use std::path::Path;

use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use image::imageops::{self, FilterType};
use winresource::WindowsResource;

const ICON_PNG: &str = "assets/icons/gridtool.png";

/// The sizes Windows picks from for the title bar, task bar, shortcuts and Explorer.
const ICO_SIZES: [u32; 6] = [16, 24, 32, 48, 64, 256];

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={ICON_PNG}");

    // Windows shows the icon in an executable's resources, which other platforms don't have.
    if std::env::var("CARGO_CFG_TARGET_OS")? == "windows" {
        let ico_path = Path::new(&std::env::var("OUT_DIR")?).join("gridtool.ico");
        write_ico(Path::new(ICON_PNG), &ico_path)?;
        WindowsResource::new()
            .set_icon(ico_path.to_str().ok_or("OUT_DIR is not valid UTF-8")?)
            .compile()?;
    }
    Ok(())
}

/// Writes the PNG as an ICO file holding each of `ICO_SIZES`.
fn write_ico(png_path: &Path, ico_path: &Path) -> Result<(), Box<dyn Error>> {
    let png = image::open(png_path)?.into_rgba8();
    let mut icon = IconDir::new(ResourceType::Icon);
    for size in ICO_SIZES {
        let resized = imageops::resize(&png, size, size, FilterType::Lanczos3);
        let image = IconImage::from_rgba_data(size, size, resized.into_raw());
        icon.add_entry(IconDirEntry::encode(&image)?);
    }
    icon.write(File::create(ico_path)?)?;
    Ok(())
}
