use std::time::Duration as StdDuration;
#[cfg(feature = "time")]
use time::Duration as TimeDuration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fancy_duration::FancyDuration;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fancy duration std seconds", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(20), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration time seconds", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(20), 0)))
    });

    c.bench_function("fancy duration std minutes", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration time minutes", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(61), 0)))
    });

    c.bench_function("fancy duration std hours", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(2 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration time hours", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(2 * 60 * 60 + 61), 0)))
    });

    c.bench_function("fancy duration std days", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(2 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration time days", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(2 * 24 * 60 * 60 + 61), 0)))
    });

    c.bench_function("fancy duration std weeks", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(2 * 7 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration time weeks", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(2 * 7 * 24 * 60 * 60 + 61), 0)))
    });

    c.bench_function("fancy duration std months", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(31 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration time months", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(31 * 24 * 60 * 60 + 61), 0)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
