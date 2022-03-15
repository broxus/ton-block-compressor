use std::time::Duration;
use std::{fs::File, os::raw::c_int, path::Path};

use criterion::profiler::Profiler;
use criterion::{criterion_group, criterion_main, Criterion};
use pprof::ProfilerGuard;
use rand::Rng;

use ton_block_compressor::ZstdWrapper;

fn bench_encode_8mb(c: &mut Criterion) {
    let mut input = vec![8; 1024 * 1024 * 8];
    let mut encoder = ZstdWrapper::new();
    c.bench_function("Encode optimistic", |b| {
        b.iter(|| {
            encoder.compress(&input).unwrap();
        })
    });

    let encoded = encoder.compress(&input).unwrap().to_vec();
    c.bench_function("Decode optimistic", |b| {
        b.iter(|| {
            encoder.decompress(&encoded).unwrap();
        })
    });

    rand::thread_rng().fill(input.as_mut_slice());
    c.bench_function("Encode pessimistic", |b| {
        b.iter(|| {
            encoder.compress(&input).unwrap();
        })
    });

    let encoded = encoder.compress(&input).unwrap().to_vec();
    c.bench_function("Decode pessimistic", |b| {
        b.iter(|| {
            encoder.decompress(&encoded).unwrap();
        })
    });

    let mut encoder = ZstdWrapper::with_level(5);
    c.bench_function("Encode pessimistic level 5", |b| {
        b.iter(|| {
            encoder.compress(&input).unwrap();
        })
    });
}

fn bench_encode_1mb(c: &mut Criterion) {
    let mut input = vec![8; 1024 * 1024];
    let mut encoder = ZstdWrapper::new();
    c.bench_function("Encode optimistic 1mb", |b| {
        b.iter(|| {
            encoder.compress(&input).unwrap();
        })
    });

    let encoded = encoder.compress(&input).unwrap().to_vec();
    c.bench_function("Decode optimistic 1mb", |b| {
        b.iter(|| {
            encoder.decompress(&encoded).unwrap();
        })
    });

    rand::thread_rng().fill(input.as_mut_slice());
    c.bench_function("Encode pessimistic 1mb", |b| {
        b.iter(|| {
            encoder.compress(&input).unwrap();
        })
    });

    let encoded = encoder.compress(&input).unwrap().to_vec();
    c.bench_function("Decode pessimistic 1mb", |b| {
        b.iter(|| {
            encoder.decompress(&encoded).unwrap();
        })
    });

    let mut encoder = ZstdWrapper::with_level(5);
    c.bench_function("Encode pessimistic level 5 1mb", |b| {
        b.iter(|| {
            encoder.compress(&input).unwrap();
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10)).with_profiler(FlamegraphProfiler::new(1000));
    targets = bench_encode_8mb,bench_encode_1mb
}
criterion_main!(benches);

/// Small custom profiler that can be used with Criterion to create a flamegraph for benchmarks.
/// Also see [the Criterion documentation on this][custom-profiler].
///
/// ## Example on how to enable the custom profiler:
///
/// ```
/// mod perf;
/// use perf::FlamegraphProfiler;
///
/// fn fibonacci_profiled(criterion: &mut Criterion) {
///     // Use the criterion struct as normal here.
/// }
///
/// fn custom() -> Criterion {
///     Criterion::default().with_profiler(FlamegraphProfiler::new())
/// }
///
/// criterion_group! {
///     name = benches;
///     config = custom();
///     targets = fibonacci_profiled
/// }
/// ```
///
/// The neat thing about this is that it will sample _only_ the benchmark, and not other stuff like
/// the setup process.
///
/// Further, it will only kick in if `--profile-time <time>` is passed to the benchmark binary.
/// A flamegraph will be created for each individual benchmark in its report directory under
/// `profile/flamegraph.svg`.
///
/// [custom-profiler]: https://bheisler.github.io/criterion.rs/book/user_guide/profiling.html#implementing-in-process-profiling-hooks
pub struct FlamegraphProfiler<'a> {
    frequency: c_int,
    active_profiler: Option<ProfilerGuard<'a>>,
}

impl<'a> FlamegraphProfiler<'a> {
    #[allow(dead_code)]
    pub fn new(frequency: c_int) -> Self {
        FlamegraphProfiler {
            frequency,
            active_profiler: None,
        }
    }
}

impl<'a> Profiler for FlamegraphProfiler<'a> {
    fn start_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        self.active_profiler = Some(ProfilerGuard::new(self.frequency).unwrap());
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
        std::fs::create_dir_all(benchmark_dir).unwrap();
        let flamegraph_path = benchmark_dir.join("flamegraph.svg");
        let flamegraph_file = File::create(&flamegraph_path)
            .expect("File system error while creating flamegraph.svg");
        if let Some(profiler) = self.active_profiler.take() {
            profiler
                .report()
                .build()
                .unwrap()
                .flamegraph(flamegraph_file)
                .expect("Error writing flamegraph");
        }
    }
}
