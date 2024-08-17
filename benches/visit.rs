mod util;

use pprof::criterion::{Output, PProfProfiler};

use bevy_math::UVec2;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pixel_map::PixelMap;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("visit");
    group.sample_size(100);

    let size = 1024;
    let pixel_map: PixelMap<bool, u16> = util::create_checker_board(black_box(&UVec2::splat(size)));
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

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(PProfProfiler::new(100, Output::Flamegraph(None)));
    targets = criterion_benchmark
}
criterion_main!(benches);
