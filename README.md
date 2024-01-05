# Fancier Durations now in an easily consumable library

A "fancy duration" is a text description of the duration. For example, "1h 20m 30s" which might be read as "one hour, twenty minutes and thirty seconds". Expression in a duration type is transparent through a variety of means; chrono and the time crate are supported, as well as serialization into and from string types with serde. Time support starts in years and funnels down to nanoseconds.

Here are the [docs](https://docs.rs/fancy_duration).

What follows are some usage examples. You can either wrap your duration-like type in a FancyDuration struct, or use types which allow for monkeypatched methods that allow you to work directly on the target type. For example, use AsFancyDuration to inject fancy_duration calls to perform the construction (which can be formatted or converted to string) and ParseFancyDuration to inject parse_fancy_duration constructors to accept strings into your favorite type. std::time::Duration, time::Duration, and chrono::Duration are all supported (some features may need to be required) and you can make more types eligible by implementing the AsTimes trait.

```rust
use std::time::Duration;
use fancy_duration::FancyDuration;

pub fn main() {
    assert_eq!(FancyDuration(Duration::new(20, 0)).to_string(), "20s");
    assert_eq!(FancyDuration(Duration::new(600, 0)).to_string(), "10m");
    assert_eq!(FancyDuration(Duration::new(120, 0)).to_string(), "2m");
    assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");
    assert_eq!(FancyDuration::<Duration>::parse("3m 5s").unwrap().duration(), Duration::new(185, 0));
    assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");

    // these traits are also implemented for chrono and time
    use fancy_duration::{ParseFancyDuration, AsFancyDuration};
    assert_eq!(Duration::new(20, 0).fancy_duration().to_string(), "20s");
    assert_eq!(Duration::new(600, 0).fancy_duration().to_string(), "10m");
    assert_eq!(Duration::new(120, 0).fancy_duration().to_string(), "2m");
    assert_eq!(Duration::new(185, 0).fancy_duration().to_string(), "3m 5s");
    assert_eq!(Duration::parse_fancy_duration("3m 5s".to_string()).unwrap(), Duration::new(185, 0));
    assert_eq!(Duration::new(185, 0).fancy_duration().to_string(), "3m 5s");

    #[cfg(feature = "time")]
    {
        // also works with time::Duration from the `time` crate
        assert_eq!(FancyDuration(time::Duration::new(20, 0)).to_string(), "20s");
        assert_eq!(FancyDuration(time::Duration::new(600, 0)).to_string(), "10m");
        assert_eq!(FancyDuration(time::Duration::new(120, 0)).to_string(), "2m");
        assert_eq!(FancyDuration(time::Duration::new(185, 0)).to_string(), "3m 5s");
        assert_eq!(FancyDuration::<time::Duration>::parse("3m 5s").unwrap().duration(), time::Duration::new(185, 0));
        assert_eq!(FancyDuration(time::Duration::new(185, 0)).to_string(), "3m 5s");
    }

    #[cfg(feature = "chrono")]
    {
        // also works with chrono!
        assert_eq!(FancyDuration(chrono::Duration::seconds(20)).to_string(), "20s");
        assert_eq!(FancyDuration(chrono::Duration::seconds(600)).to_string(), "10m");
        assert_eq!(FancyDuration(chrono::Duration::seconds(120)).to_string(), "2m");
        assert_eq!(FancyDuration(chrono::Duration::seconds(185)).to_string(), "3m 5s");
        assert_eq!(FancyDuration::<chrono::Duration>::parse("3m 5s").unwrap().duration(), chrono::Duration::seconds(185));
        assert_eq!(FancyDuration(chrono::Duration::seconds(185)).to_string(), "3m 5s");
    }
}
```

## Benchmarks

Each interval test increases the number of things that will be formatted. The first set of parse tests parse one string with several terms, the others parse a static set of 5 terms in a single iteration.

Benchmarking system was a Ryzen 5900X with 64GB RAM running Linux 6.6.8 in a desktop configuration.

As of 0.9.0:

```
fancy duration format seconds: std
                        time:   [656.09 ps 659.41 ps 663.30 ps]
fancy duration format seconds: time
                        time:   [6.8162 ns 6.8188 ns 6.8215 ns]
fancy duration format seconds: chrono
                        time:   [671.98 ps 672.46 ps 672.94 ps]
fancy duration format minutes: std
                        time:   [666.71 ps 667.09 ps 667.50 ps]
fancy duration format minutes: time
                        time:   [6.8165 ns 6.8193 ns 6.8222 ns]
fancy duration format minutes: chrono
                        time:   [672.28 ps 672.77 ps 673.32 ps]
fancy duration format hours: std
                        time:   [670.27 ps 670.54 ps 670.83 ps]
fancy duration format hours: time
                        time:   [6.8001 ns 6.8028 ns 6.8059 ns]
fancy duration format hours: chrono
                        time:   [674.97 ps 675.44 ps 675.95 ps]
fancy duration format days: std
                        time:   [665.85 ps 666.24 ps 666.67 ps]
fancy duration format days: time
                        time:   [6.6753 ns 6.6955 ns 6.7188 ns]
fancy duration format days: chrono
                        time:   [669.73 ps 670.17 ps 670.67 ps]
fancy duration format weeks: std
                        time:   [667.88 ps 668.16 ps 668.47 ps]
fancy duration format weeks: time
                        time:   [6.8532 ns 6.8562 ns 6.8595 ns]
fancy duration format weeks: chrono
                        time:   [671.99 ps 672.38 ps 672.80 ps]
fancy duration format months: std
                        time:   [642.25 ps 643.25 ps 644.49 ps]
fancy duration format months: time
                        time:   [6.7654 ns 6.7885 ns 6.8113 ns]
fancy duration format months: chrono
                        time:   [669.46 ps 669.80 ps 670.15 ps]
fancy duration parse one: std
                        time:   [75.938 µs 76.011 µs 76.089 µs]
fancy duration parse one: time
                        time:   [75.263 µs 75.593 µs 76.024 µs]
fancy duration parse one: chrono
                        time:   [75.311 µs 75.408 µs 75.495 µs]
fancy duration parse 5 distinct items: std
                        time:   [378.16 µs 379.05 µs 379.89 µs]
fancy duration parse 5 distinct items: time
                        time:   [376.00 µs 377.52 µs 379.09 µs]
fancy duration parse 5 distinct items: chrono
                        time:   [364.04 µs 364.37 µs 364.72 µs]
```

## Author

Erik Hollensbe <git@hollensbe.org>

## License

MIT
