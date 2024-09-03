use bevy_math::{uvec2, UVec2};
use image::{DynamicImage, GenericImageView, Rgba};
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

pub fn load_image(pixel_map: &mut PixelMap<Rgba<u8>>, image: &DynamicImage) {
    let region_width = image.width();
    let region_height = image.height();

    for y in 0..region_height {
        for x in 0..region_width {
            let image_x = x;
            let image_y = y;
            let color = image.get_pixel(image_x, image_y);
            let point = uvec2(x, region_height - y - 1);
            pixel_map.set_pixel(point, color);
        }
    }
}
