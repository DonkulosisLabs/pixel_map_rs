use bevy_math::{IRect, UVec2};
use pixel_map::PixelMap;

const MAP_SIZE: u32 = 4;

fn main() {
    let mut pixel_map = PixelMap::<bool, u16>::new(
        &UVec2::splat(MAP_SIZE),
        false, // initial value
        1,     // pixel size
    );

    // pixel_map state:
    // +---+---+---+---+
    // |               |
    // +               +
    // |               |
    // +       f       +
    // |               |
    // +               +
    // |               |
    // +---+---+---+---+

    pixel_map.set_pixel((1, 1), true);

    // pixel_map state:
    // +---+---+---+---+
    // |       |       |
    // +   f   +   f   +
    // |       |       |
    // +---+---+---+---+
    // | f | t |       |
    // +---+---+   f   +
    // | f | f |       |
    // +---+---+---+---+

    pixel_map.draw_rect(&IRect::from_corners((2, 2).into(), (6, 6).into()), true);

    // pixel_map state:
    // +---+---+---+---+
    // |       |       |
    // +   f   +   t   +
    // |       |       |
    // +---+---+---+---+
    // | f | t |       |
    // +---+---+   f   +
    // | f | f |       |
    // +---+---+---+---+

    pixel_map.draw_rect(&IRect::from_corners((0, 0).into(), (4, 4).into()), true);

    // pixel_map state:
    // +---+---+---+---+
    // |               |
    // +               +
    // |               |
    // +       t       +
    // |               |
    // +               +
    // |               |
    // +---+---+---+---+
}
