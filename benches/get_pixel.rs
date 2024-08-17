mod util;

use bevy_math::UVec2;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixel_map::PixelMap;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("get_pixel");
    group.sample_size(100);

    let size = 1024;
    let pixel_map: PixelMap<bool, u16> = util::create_checker_board(black_box(&UVec2::splat(size)));
    group.bench_function("all", |b| {
        b.iter(|| {
            for x in 0..size {
                for y in 0..size {
                    let r = pixel_map.get_pixel((black_box(x), black_box(y)));
                    black_box(r);
                }
            }
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
