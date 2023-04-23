use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixel_map::{PixelMap, Region};

// TODO: DRY
pub fn create_checker_board(region: Region) -> PixelMap {
    let size = region.size();
    let mut pixel_map = PixelMap::new(region, false, 1);
    for x in 0..size {
        for y in 0..size {
            if (x + y) % 2 == 0 {
                pixel_map.set_pixel((x as i32, y as i32), false);
            } else {
                pixel_map.set_pixel((x as i32, y as i32), true);
            }
        }
    }
    pixel_map
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("visit");
    group.sample_size(100);

    let size = 1024;
    let region = Region::new(black_box(0), black_box(0), black_box(size));

    let pixel_map: PixelMap<bool, u16> = create_checker_board(region);
    group.bench_function("all", |b| {
        b.iter(|| {
            pixel_map.visit(|n| {
                black_box(n);
            });
        });
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
