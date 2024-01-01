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

## Author

Erik Hollensbe <git@hollensbe.org>

## License

MIT
