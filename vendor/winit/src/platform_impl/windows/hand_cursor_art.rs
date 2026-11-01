//! GridTool patch (see `PATCHES.md`): draws the open and closed hands. Windows has no system
//! cursor for either. They are drawn rather than stored so that they stay sharp at the size
//! of the system's cursors, whatever the display scale.

/// The size of the box the shapes below are laid out in.
const DESIGN_SIZE: f32 = 32.0;
/// How far the black outline extends beyond the white shape.
const OUTLINE: f32 = 1.0;
/// Samples along each side of a pixel, to smooth the edges.
const SUBPIXELS: u32 = 4;

/// A line with rounded ends and a thickness of twice `radius`.
#[derive(Clone, Copy)]
struct Capsule {
    from: (f32, f32),
    to: (f32, f32),
    radius: f32,
}

const fn capsule(from: (f32, f32), to: (f32, f32), radius: f32) -> Capsule {
    Capsule { from, to, radius }
}

const FINGER: f32 = 1.6;

/// The back of a hand, with fingers spread up and thumb out to the left.
const OPEN_HAND: [Capsule; 6] = [
    capsule((14.5, 21.5), (19.0, 21.5), 5.2),
    capsule((10.6, 18.0), (10.6, 8.5), FINGER),
    capsule((14.7, 18.0), (14.7, 6.0), FINGER),
    capsule((18.8, 18.0), (18.8, 6.5), FINGER),
    capsule((22.9, 19.0), (22.9, 9.5), FINGER),
    capsule((12.0, 23.0), (6.0, 15.5), FINGER),
];

/// The same hand with its fingers curled over and thumb tucked in.
const CLOSED_HAND: [Capsule; 6] = [
    capsule((14.5, 21.5), (19.0, 21.5), 5.2),
    capsule((10.6, 18.0), (10.6, 14.0), FINGER),
    capsule((14.7, 18.0), (14.7, 12.5), FINGER),
    capsule((18.8, 18.0), (18.8, 12.5), FINGER),
    capsule((22.9, 19.0), (22.9, 14.0), FINGER),
    capsule((11.5, 23.0), (8.5, 19.5), FINGER),
];

/// The distance from `point` to the edge of `shape`, negative inside it.
fn distance(shape: Capsule, point: (f32, f32)) -> f32 {
    let along = (shape.to.0 - shape.from.0, shape.to.1 - shape.from.1);
    let from_start = (point.0 - shape.from.0, point.1 - shape.from.1);
    let length_squared = along.0 * along.0 + along.1 * along.1;
    let t = ((from_start.0 * along.0 + from_start.1 * along.1) / length_squared).clamp(0.0, 1.0);
    let offset = (from_start.0 - along.0 * t, from_start.1 - along.1 * t);
    (offset.0 * offset.0 + offset.1 * offset.1).sqrt() - shape.radius
}

/// Unmultiplied RGBA pixels of a hand, `size` pixels square.
pub(super) fn hand_rgba(closed: bool, size: u32) -> Vec<u8> {
    let shapes = if closed { &CLOSED_HAND } else { &OPEN_HAND };
    let scale = DESIGN_SIZE / (size * SUBPIXELS) as f32;
    let samples = SUBPIXELS * SUBPIXELS;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let (mut white, mut black) = (0, 0);
            for sub_y in 0..SUBPIXELS {
                for sub_x in 0..SUBPIXELS {
                    let point = (
                        ((x * SUBPIXELS + sub_x) as f32 + 0.5) * scale,
                        ((y * SUBPIXELS + sub_y) as f32 + 0.5) * scale,
                    );
                    let nearest = shapes
                        .iter()
                        .map(|&shape| distance(shape, point))
                        .fold(f32::MAX, f32::min);
                    if nearest < 0.0 {
                        white += 1;
                    } else if nearest < OUTLINE {
                        black += 1;
                    }
                }
            }
            let covered = white + black;
            let grey = if covered == 0 { 0 } else { 255 * white / covered };
            let alpha = 255 * covered / samples;
            rgba.extend([grey as u8, grey as u8, grey as u8, alpha as u8]);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opaque_pixels(rgba: &[u8]) -> usize {
        rgba.chunks_exact(4).filter(|pixel| pixel[3] == 255).count()
    }

    #[test]
    fn hands_have_every_pixel_and_stay_inside_their_box() {
        for size in [32, 48, 64] {
            for closed in [false, true] {
                let rgba = hand_rgba(closed, size);
                assert_eq!(rgba.len(), (size * size * 4) as usize);
                let side = size as usize;
                for (index, pixel) in rgba.chunks_exact(4).enumerate() {
                    let (x, y) = (index % side, index / side);
                    if x == 0 || y == 0 || x == side - 1 || y == side - 1 {
                        assert_eq!(pixel[3], 0, "{closed} {size} ({x}, {y})");
                    }
                }
            }
        }
    }

    #[test]
    fn the_closed_hand_is_smaller_than_the_open_one() {
        assert!(opaque_pixels(&hand_rgba(true, 32)) < opaque_pixels(&hand_rgba(false, 32)));
    }

    #[test]
    fn the_middle_of_the_palm_is_opaque_white() {
        for closed in [false, true] {
            let rgba = hand_rgba(closed, 32);
            let middle = (21 * 32 + 16) * 4;
            assert_eq!(rgba[middle..middle + 4], [255, 255, 255, 255]);
        }
    }
}
