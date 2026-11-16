//! Splitting an image into tiles no larger than the graphics device can hold, so an image
//! of any size can be shown at full resolution.

use super::DecodedImage;

/// A rectangle of image pixels: `width` × `height` pixels from (`x`, `y`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Tiles, row by row from the top-left, that together cover a `width` × `height` image
/// exactly, none of them wider or taller than `max_side`. An empty image has no tiles.
///
/// # Panics
///
/// If `max_side` is zero.
pub fn tile_layout(width: u32, height: u32, max_side: u32) -> Vec<Tile> {
    assert!(max_side > 0, "tiles must be at least one pixel across");
    let spans = |length: u32| {
        (0..length)
            .step_by(max_side as usize)
            .map(move |start| (start, max_side.min(length - start)))
    };
    spans(height)
        .flat_map(|(y, height)| {
            spans(width).map(move |(x, width)| Tile {
                x,
                y,
                width,
                height,
            })
        })
        .collect()
}

impl DecodedImage {
    /// The pixels inside `tile`, which must lie within the image.
    pub fn crop(&self, tile: Tile) -> DecodedImage {
        assert!(
            tile.x + tile.width <= self.width && tile.y + tile.height <= self.height,
            "tile {tile:?} lies outside the {}×{} image",
            self.width,
            self.height
        );
        let row_bytes = self.width as usize * 4;
        let (start, length) = (tile.x as usize * 4, tile.width as usize * 4);
        let rgba = self
            .rgba
            .chunks_exact(row_bytes)
            .skip(tile.y as usize)
            .take(tile.height as usize)
            .flat_map(|row| &row[start..start + length])
            .copied()
            .collect();
        DecodedImage {
            width: tile.width,
            height: tile.height,
            rgba,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(x: u32, y: u32, width: u32, height: u32) -> Tile {
        Tile {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn an_image_within_the_limit_is_one_tile() {
        assert_eq!(tile_layout(300, 200, 300), vec![tile(0, 0, 300, 200)]);
    }

    #[test]
    fn a_tall_image_is_split_into_rows_with_a_smaller_last_row() {
        // The shape of a 4675×10976 plan on a device limited to 8192 pixels a side.
        assert_eq!(
            tile_layout(4675, 10976, 8192),
            vec![tile(0, 0, 4675, 8192), tile(0, 8192, 4675, 2784)]
        );
    }

    #[test]
    fn a_large_image_is_split_both_ways_row_by_row() {
        assert_eq!(
            tile_layout(5, 3, 2),
            vec![
                tile(0, 0, 2, 2),
                tile(2, 0, 2, 2),
                tile(4, 0, 1, 2),
                tile(0, 2, 2, 1),
                tile(2, 2, 2, 1),
                tile(4, 2, 1, 1),
            ]
        );
    }

    #[test]
    fn an_exact_multiple_has_no_sliver_tiles() {
        assert_eq!(
            tile_layout(4, 2, 2),
            vec![tile(0, 0, 2, 2), tile(2, 0, 2, 2)]
        );
    }

    #[test]
    fn an_empty_image_has_no_tiles() {
        assert!(tile_layout(0, 10, 4).is_empty());
        assert!(tile_layout(10, 0, 4).is_empty());
    }

    #[test]
    #[should_panic]
    fn a_zero_tile_size_is_rejected() {
        tile_layout(1, 1, 0);
    }

    /// A `width` × `height` image whose pixel (x, y) is [x, y, 0, 255].
    fn coordinate_image(width: u32, height: u32) -> DecodedImage {
        let rgba = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| [x as u8, y as u8, 0, 255]))
            .collect();
        DecodedImage {
            width,
            height,
            rgba,
        }
    }

    #[test]
    fn cropping_keeps_the_pixels_inside_the_tile() {
        let cropped = coordinate_image(4, 3).crop(tile(1, 1, 2, 2));
        let expected = DecodedImage {
            width: 2,
            height: 2,
            #[rustfmt::skip]
            rgba: vec![
                1, 1, 0, 255,   2, 1, 0, 255,
                1, 2, 0, 255,   2, 2, 0, 255,
            ],
        };
        assert_eq!(cropped, expected);
    }

    #[test]
    fn the_tiles_of_a_layout_crop_back_to_the_whole_image() {
        let image = coordinate_image(5, 3);
        let tiles = tile_layout(image.width, image.height, 2);
        let mut rebuilt = vec![0; image.rgba.len()];
        for tile in tiles {
            let cropped = image.crop(tile);
            for row in 0..tile.height {
                let from = (row * tile.width * 4) as usize;
                let to = (((tile.y + row) * image.width + tile.x) * 4) as usize;
                let length = (tile.width * 4) as usize;
                rebuilt[to..to + length].copy_from_slice(&cropped.rgba[from..from + length]);
            }
        }
        assert_eq!(rebuilt, image.rgba);
    }

    #[test]
    #[should_panic]
    fn cropping_outside_the_image_is_rejected() {
        coordinate_image(2, 2).crop(tile(1, 0, 2, 1));
    }
}
