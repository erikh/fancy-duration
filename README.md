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

As of 0.9.1:

```
fancy duration format seconds: std
                        time:   [644.13 ps 645.34 ps 646.96 ps]
fancy duration format seconds: time
                        time:   [6.6756 ns 6.6945 ns 6.7181 ns]
fancy duration format seconds: chrono
                        time:   [658.36 ps 658.73 ps 659.15 ps]
fancy duration format minutes: std
                        time:   [666.00 ps 666.50 ps 667.03 ps]
fancy duration format minutes: time
                        time:   [6.6301 ns 6.6324 ns 6.6349 ns]
fancy duration format minutes: chrono
                        time:   [646.99 ps 647.24 ps 647.50 ps]
fancy duration format hours: std
                        time:   [658.35 ps 660.83 ps 662.88 ps]
fancy duration format hours: time
                        time:   [6.5761 ns 6.5792 ns 6.5826 ns]
fancy duration format hours: chrono
                        time:   [648.05 ps 648.38 ps 648.74 ps]
fancy duration format days: std
                        time:   [641.06 ps 641.33 ps 641.62 ps]
fancy duration format days: time
                        time:   [6.5374 ns 6.5410 ns 6.5452 ns]
fancy duration format days: chrono
                        time:   [662.60 ps 665.24 ps 667.41 ps]
fancy duration format weeks: std
                        time:   [641.06 ps 641.32 ps 641.61 ps]
fancy duration format weeks: time
                        time:   [6.7987 ns 6.8158 ns 6.8288 ns]
fancy duration format weeks: chrono
                        time:   [660.01 ps 662.64 ps 665.05 ps]
fancy duration format months: std
                        time:   [641.66 ps 642.02 ps 642.41 ps]
fancy duration format months: time
                        time:   [6.5435 ns 6.5498 ns 6.5573 ns]
fancy duration format months: chrono
                        time:   [672.00 ps 672.39 ps 672.83 ps]
fancy duration parse one: std
                        time:   [332.00 ns 332.48 ns 332.98 ns]
fancy duration parse one: time
                        time:   [346.25 ns 346.58 ns 346.92 ns]
fancy duration parse one: chrono
                        time:   [369.81 ns 371.26 ns 372.65 ns]
fancy duration parse 5 distinct items: std
                        time:   [1.9150 µs 1.9183 µs 1.9214 µs]
fancy duration parse 5 distinct items: time
                        time:   [1.8446 µs 1.8474 µs 1.8503 µs]
fancy duration parse 5 distinct items: chrono
                        time:   [1.8281 µs 1.8311 µs 1.8345 µs]
```

## Author

Erik Hollensbe <git@hollensbe.org>

## License

MIT
