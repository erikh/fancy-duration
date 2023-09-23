# Fancier Durations now in an easily consumable library

Using this library is very simple (but here are the
[docs](https://docs.rs/fancy_duration)):

```
use std::time::Duration;
use fancy_duration::FancyDuration;

pub fn main() {
    assert_eq!(FancyDuration(Duration::new(20, 0)).to_string(), "20s");
    assert_eq!(FancyDuration(Duration::new(600, 0)).to_string(), "10m");
    assert_eq!(FancyDuration(Duration::new(120, 0)).to_string(), "2m");
    assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");
    assert_eq!(FancyDuration::<Duration>::parse("3m 5s").unwrap().duration(), Duration::new(185, 0));
    assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");

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

Comes with support for serde, chrono, time as well as public traits you can
implement to bring in your own duration implementation.

## Author

Erik Hollensbe <git@hollensbe.org>

## License

MIT
