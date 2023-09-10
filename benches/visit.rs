use bevy_math::UVec2;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixel_map::PixelMap;

// TODO: DRY
pub fn create_checker_board(size: &UVec2) -> PixelMap {
    let mut pixel_map = PixelMap::new(size, false, 1);
    for x in 0..size.x {
        for y in 0..size.y {
            if (x + y) % 2 == 0 {
                pixel_map.set_pixel((x, y), false);
            } else {
                pixel_map.set_pixel((x, y), true);
            }
        }
    }
    pixel_map
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("visit");
    group.sample_size(100);

    let size = 1024;
    let pixel_map: PixelMap<bool, u16> = create_checker_board(black_box(&UVec2::splat(size)));
    group.bench_function("all", |b| {
        b.iter(|| {
            pixel_map.visit(|n, sub_rect| {
                black_box(n);
                black_box(sub_rect);
            });
        });
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
