mod util;

use pprof::criterion::{Output, PProfProfiler};

use bevy_math::{uvec2, URect, UVec2};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixel_map::{Direction, PixelMap};

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("visit_neighbor");
    group.sample_size(100);

    let size = 1024;
    let pixel_map: PixelMap<bool, u16> = util::create_checker_board(black_box(&UVec2::splat(size)));
    group.bench_function("all", |b| {
        let map_region = pixel_map.region().as_urect();
        let target_node_region = {
            let p = size / 2;
            URect::from_corners(uvec2(p, p), uvec2(p + 1, p + 1))
        };

        let directions: Vec<_> = Direction::iter().collect();
        let mut direction_index = 0;

        b.iter(|| {
            pixel_map.visit_neighbors(
                black_box(&map_region),
                black_box(&target_node_region),
                black_box(directions[direction_index % directions.len()]),
                |node, sub_rect| {
                    black_box(node);
                    black_box(sub_rect);
                    true
                },
                |node, sub_rect| {
                    black_box(node);
                    black_box(sub_rect);
                },
            );
            direction_index += 1;
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
