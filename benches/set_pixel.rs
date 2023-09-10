use bevy_math::UVec2;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixel_map::PixelMap;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("set_pixel");
    group.sample_size(100);

    let size = 1024;
    let mut pixel_map: PixelMap<bool, u16> = PixelMap::new(
        black_box(&UVec2::splat(size)),
        black_box(false),
        black_box(1),
    );

    group.bench_function("common", |b| {
        b.iter(|| {
            for x in 0..size {
                for y in 0..size {
                    pixel_map.set_pixel((black_box(x), black_box(y)), black_box(true));
                }
            }
        })
    });

    let mut pixel_map: PixelMap<bool, u16> = PixelMap::new(
        black_box(&UVec2::splat(size)),
        black_box(false),
        black_box(1),
    );
    let mut value = false;

    group.bench_function("alternating", |b| {
        b.iter(|| {
            for x in 0..size {
                for y in 0..size {
                    pixel_map.set_pixel((black_box(x), black_box(y)), black_box(value));
                    value = !value;
                }
            }
            value = !value;
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
