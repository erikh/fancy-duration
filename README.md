# Fancier Durations now in an easily consumable library

Using this library is very simple:

```rust
use std::time::Duration;
use fancy_duration::FancyDuration;

pub fn main() {
    assert_eq!(FancyDuration(Duration::new(20, 0)).to_string(), "20s");
    assert_eq!(FancyDuration(Duration::new(600, 0)).to_string(), "10m");
    assert_eq!(FancyDuration(Duration::new(120, 0)).to_string(), "2m");
    assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");
}
```

Also works with the `time` crate.

## Author

Erik Hollensbe <github@hollensbe.org>

## License

MIT
