use pixel_map::{IRect, PixelMap, Region};

const MAP_SIZE: u16 = 4;

fn main() {
    let mut pixel_map = PixelMap::new(
        Region::new(0, 0, MAP_SIZE),
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

    pixel_map.draw_rect(&IRect::from_corners((2, 2), (6, 6)), true);

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

    pixel_map.draw_rect(&IRect::from_corners((0, 0), (4, 4)), true);

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
