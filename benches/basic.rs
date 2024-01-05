#[cfg(feature = "chrono")]
use chrono::Duration as ChronoDuration;
use std::time::Duration as StdDuration;
#[cfg(feature = "time")]
use time::Duration as TimeDuration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fancy_duration::FancyDuration;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fancy duration format seconds: std", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(20), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration format seconds: time", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(20), 0)))
    });
    #[cfg(feature = "chrono")]
    c.bench_function("fancy duration format seconds: chrono", |b| {
        b.iter(|| FancyDuration(ChronoDuration::seconds(black_box(20))))
    });

    c.bench_function("fancy duration format minutes: std", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration format minutes: time", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(61), 0)))
    });
    #[cfg(feature = "chrono")]
    c.bench_function("fancy duration format minutes: chrono", |b| {
        b.iter(|| FancyDuration(ChronoDuration::seconds(black_box(61))))
    });

    c.bench_function("fancy duration format hours: std", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(2 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration format hours: time", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(2 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "chrono")]
    c.bench_function("fancy duration format hours: chrono", |b| {
        b.iter(|| FancyDuration(ChronoDuration::seconds(black_box(2 * 60 * 60 + 61))))
    });

    c.bench_function("fancy duration format days: std", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(2 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration format days: time", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(2 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "chrono")]
    c.bench_function("fancy duration format days: chrono", |b| {
        b.iter(|| FancyDuration(ChronoDuration::seconds(black_box(2 * 24 * 60 * 60 + 61))))
    });

    c.bench_function("fancy duration format weeks: std", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(2 * 7 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration format weeks: time", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(2 * 7 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "chrono")]
    c.bench_function("fancy duration format weeks: chrono", |b| {
        b.iter(|| {
            FancyDuration(ChronoDuration::seconds(black_box(
                2 * 7 * 24 * 60 * 60 + 61,
            )))
        })
    });

    c.bench_function("fancy duration format months: std", |b| {
        b.iter(|| FancyDuration(StdDuration::new(black_box(31 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration format months: time", |b| {
        b.iter(|| FancyDuration(TimeDuration::new(black_box(31 * 24 * 60 * 60 + 61), 0)))
    });
    #[cfg(feature = "chrono")]
    c.bench_function("fancy duration format months: chrono", |b| {
        b.iter(|| FancyDuration(ChronoDuration::seconds(black_box(31 * 24 * 60 * 60 + 61))))
    });

    let times: [&str; 5] = ["1d2m3s", "1y 2w 3d 5h", "10ns", "3s100ms", "3m 2w 3d 4h 1m"];

    c.bench_function("fancy duration parse one: std", |b| {
        b.iter(|| {
            FancyDuration::<StdDuration>::parse(times[0]).unwrap();
        })
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration parse one: time", |b| {
        b.iter(|| {
            FancyDuration::<TimeDuration>::parse(times[0]).unwrap();
        })
    });
    #[cfg(feature = "chrono")]
    c.bench_function("fancy duration parse one: chrono", |b| {
        b.iter(|| {
            FancyDuration::<ChronoDuration>::parse(times[0]).unwrap();
        })
    });

    c.bench_function("fancy duration parse 5 distinct items: std", |b| {
        b.iter(|| {
            for time in times {
                FancyDuration::<StdDuration>::parse(time).unwrap();
            }
        })
    });
    #[cfg(feature = "time")]
    c.bench_function("fancy duration parse 5 distinct items: time", |b| {
        b.iter(|| {
            for time in times {
                FancyDuration::<TimeDuration>::parse(time).unwrap();
            }
        })
    });
    #[cfg(feature = "chrono")]
    c.bench_function("fancy duration parse 5 distinct items: chrono", |b| {
        b.iter(|| {
            for time in times {
                FancyDuration::<ChronoDuration>::parse(time).unwrap();
            }
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
