use bevy_math::UVec2;
use pixel_map::PixelMap;

pub fn create_checker_board(size: &UVec2) -> PixelMap {
    let mut pixel_map = PixelMap::new(size, false, 1);
    for x in 0..size.x {
        for y in 0..size.y {
            pixel_map.set_pixel((x, y), (x + y) % 2 != 0);
        }
    }
    pixel_map
}
