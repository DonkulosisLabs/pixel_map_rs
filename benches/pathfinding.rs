mod util;

use pprof::criterion::{Output, PProfProfiler};
use std::path::Path;

use crate::util::load_image;
use bevy_math::uvec2;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use image::Rgba;
use pixel_map::{pathfinding, PixelMap};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathfinding");
    group.sample_size(50);

    let white: Rgba<u8> = Rgba::from([255, 255, 255, 255]);

    let size = 1024;
    let img = image::open(Path::new("benches/fixtures/pathfinding_1024x.png")).unwrap();
    let mut pixel_map: PixelMap<Rgba<u8>, u16> = PixelMap::new(&uvec2(size, size), white, 1);
    load_image(&mut pixel_map, &img);

    group.bench_function("pathfind_a_star_grid", |b| {
        b.iter(|| {
            let result = pixel_map
                .pathfind_a_star_grid(
                    black_box(&pixel_map.region().as_urect()),
                    black_box(16),
                    black_box(uvec2(64, 64)),
                    black_box(uvec2(size - 64, size - 64)),
                    black_box(pathfinding::euclidean_heuristic),
                    |n, _| *n.value() == white,
                )
                .unwrap();

            black_box(result);
        });
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
criterion_main!(benches);
