use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

use ico::{IconDir, IconDirEntry, IconImage, ResourceType};
use image::imageops::{self, FilterType};
use winresource::WindowsResource;

#[path = "src/readme_about.rs"]
mod readme_about;

const ICON_PNG: &str = "assets/icons/gridtool.png";
const README: &str = "README.md";

/// The sizes Windows picks from for the title bar, task bar, shortcuts and Explorer.
const ICO_SIZES: [u32; 6] = [16, 24, 32, 48, 64, 256];

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed={ICON_PNG}");
    println!("cargo:rerun-if-changed={README}");

    let out_dir = std::env::var("OUT_DIR")?;
    let out_dir = Path::new(&out_dir);
    write_about(Path::new(README), &out_dir.join("about.txt"))?;

    // Windows shows the icon in an executable's resources, which other platforms don't have.
    if std::env::var("CARGO_CFG_TARGET_OS")? == "windows" {
        let ico_path = out_dir.join("gridtool.ico");
        write_ico(Path::new(ICON_PNG), &ico_path)?;
        WindowsResource::new()
            .set_icon(ico_path.to_str().ok_or("OUT_DIR is not valid UTF-8")?)
            .compile()?;
        fs::copy(&ico_path, profile_dir(out_dir)?.join("gridtool.ico"))?;
    }
    Ok(())
}

/// The directory of the build's executable, `target/<profile>`, from the build script's `OUT_DIR`,
/// `target/<profile>/build/<package>/out`. The ICO file is put there because the installer needs a
/// path that does not change, which `installer-icon` in `Cargo.toml` names.
fn profile_dir(out_dir: &Path) -> Result<&Path, Box<dyn Error>> {
    Ok(out_dir
        .ancestors()
        .nth(3)
        .ok_or("OUT_DIR is not inside a profile directory")?)
}

/// Writes the text of the README's About section for the About dialog to compile in. A README
/// without one fails the build, rather than the dialog being empty.
fn write_about(readme_path: &Path, about_path: &Path) -> Result<(), Box<dyn Error>> {
    let about = readme_about::about_text(&fs::read_to_string(readme_path)?);
    if about.is_empty() {
        return Err(format!("{} has no About section", readme_path.display()).into());
    }
    fs::write(about_path, about)?;
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
