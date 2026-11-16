//! A document's image on the GPU, as tiles no larger than the device allows, so images
//! bigger than the largest texture still open, at full resolution.

use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle};
use gridtool::geometry::{Bounds, Point};
use gridtool::imaging::DecodedImage;
use gridtool::imaging::tiles::{Tile, tile_layout};

use super::viewport::DocumentTransform;

/// One uploaded tile and the image pixels it covers.
struct TileTexture {
    texture: TextureHandle,
    bounds: Bounds,
}

/// The textures that together show a document's image.
pub struct ImageTextures {
    tiles: Vec<TileTexture>,
}

impl ImageTextures {
    /// Uploads `image` to the GPU, split into tiles the device can hold.
    pub fn upload(ctx: &egui::Context, name: &str, image: &DecodedImage) -> Self {
        let max_side = ctx.input(|input| input.max_texture_side);
        let max_side = u32::try_from(max_side).unwrap_or(u32::MAX).max(1);
        let tiles = tile_layout(image.width, image.height, max_side)
            .into_iter()
            .map(|tile| TileTexture {
                texture: upload_tile(ctx, name, image, tile),
                bounds: tile_bounds(tile),
            })
            .collect();
        Self { tiles }
    }

    /// Paints every tile in place on the document as `transform` shows it.
    pub fn paint(&self, painter: &egui::Painter, transform: DocumentTransform) {
        for tile in &self.tiles {
            let corners = transform.bounds_corners(tile.bounds, 0.0);
            painter.add(textured_quad(corners, &tile.texture));
        }
    }
}

fn upload_tile(ctx: &egui::Context, name: &str, image: &DecodedImage, tile: Tile) -> TextureHandle {
    let pixels = image.crop(tile);
    let color_image = egui::ColorImage::from_rgba_unmultiplied(pixels.size(), &pixels.rgba);
    let name = format!("{name} [{}, {}]", tile.x, tile.y);
    ctx.load_texture(name, color_image, egui::TextureOptions::default())
}

fn tile_bounds(tile: Tile) -> Bounds {
    Bounds {
        min: Point::new(f64::from(tile.x), f64::from(tile.y)),
        max: Point::new(
            f64::from(tile.x + tile.width),
            f64::from(tile.y + tile.height),
        ),
    }
}

/// A mesh drawing the whole texture onto the quad with the given corners
/// (top-left, top-right, bottom-right, bottom-left).
fn textured_quad(corners: [Pos2; 4], texture: &TextureHandle) -> egui::Mesh {
    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
    let uvs = [
        uv.left_top(),
        uv.right_top(),
        uv.right_bottom(),
        uv.left_bottom(),
    ];

    let mut mesh = egui::Mesh::with_texture(texture.id());
    for (pos, uv) in corners.into_iter().zip(uvs) {
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv,
            color: Color32::WHITE,
        });
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::RawInput;

    /// The smallest texture limit egui itself can run with.
    const DEVICE_LIMIT: usize = 1024;

    fn image(width: u32, height: u32) -> DecodedImage {
        DecodedImage {
            width,
            height,
            rgba: vec![255; (width * height * 4) as usize],
        }
    }

    /// Uploads `image` in a frame whose device holds textures up to `max_side` a side.
    fn upload(image: &DecodedImage, max_side: usize) -> ImageTextures {
        let ctx = egui::Context::default();
        let input = RawInput {
            max_texture_side: Some(max_side),
            ..Default::default()
        };
        let mut textures = None;
        let mut output = ctx.run_ui(input, |ui| {
            textures = Some(ImageTextures::upload(ui.ctx(), "test", image));
        });
        // There is no renderer to upload textures to.
        output.textures_delta.clear();
        textures.unwrap()
    }

    #[test]
    fn an_image_the_device_can_hold_is_one_texture() {
        let textures = upload(&image(30, 20), DEVICE_LIMIT);
        assert_eq!(textures.tiles.len(), 1);
        assert_eq!(textures.tiles[0].texture.size(), [30, 20]);
    }

    #[test]
    fn an_image_too_tall_for_the_device_is_split_into_textures_it_can_hold() {
        let textures = upload(&image(30, 1500), DEVICE_LIMIT);
        let sizes: Vec<_> = textures
            .tiles
            .iter()
            .map(|tile| tile.texture.size())
            .collect();
        assert_eq!(sizes, vec![[30, 1024], [30, 476]]);
        assert_eq!(
            textures.tiles[1].bounds,
            Bounds {
                min: Point::new(0.0, 1024.0),
                max: Point::new(30.0, 1500.0),
            }
        );
    }
}
