//! Using this library is very simple:
//!
//! ```
//! use std::time::Duration;
//! use fancy_duration::FancyDuration;
//!
//! pub fn main() {
//!     assert_eq!(FancyDuration(Duration::new(20, 0)).to_string(), "20s");
//!     assert_eq!(FancyDuration(Duration::new(600, 0)).to_string(), "10m");
//!     assert_eq!(FancyDuration(Duration::new(120, 0)).to_string(), "2m");
//!     assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");
//! }
//! ```
//!

use std::time::Duration;

/// To implement a fancier duration, just have your duration return the number of seconds as a part
/// of the following method call.
pub trait AsSecs {
    fn as_secs(&self) -> u64;
}

impl AsSecs for Duration {
    fn as_secs(&self) -> u64 {
        self.as_secs_f64() as u64
    }
}

impl AsSecs for time::Duration {
    fn as_secs(&self) -> u64 {
        self.whole_seconds() as u64
    }
}

pub struct FancyDuration<D: AsSecs>(pub D);

impl<D> FancyDuration<D>
where
    D: AsSecs + Sized,
{
    /// Construct a fancier duration!
    pub fn new(d: D) -> Self {
        Self(d)
    }

    /// Show the duration in a fancier format!
    pub fn format(&self) -> String {
        let mut time = self.0.as_secs();

        if time == 0 {
            return "0".to_string();
        }

        let months = time / 30 / 24 / 60 / 60;
        time -= months * 30 * 24 * 60 * 60;
        let weeks = time / 7 / 24 / 60 / 60;
        time -= weeks * 7 * 24 * 60 * 60;
        let days = time / 24 / 60 / 60;
        time -= days * 24 * 60 * 60;
        let hours = time / 60 / 60;
        time -= hours * 60 * 60;
        let minutes = time / 60;
        time -= minutes * 60;

        let mut itoa = itoa::Buffer::new();

        let s = if months >= 1 {
            itoa.format(months).to_string() + "m "
        } else {
            "".to_string()
        } + &(if weeks >= 1 {
            itoa.format(weeks).to_string() + "w "
        } else {
            "".to_string()
        }) + &(if days >= 1 {
            itoa.format(days).to_string() + "d "
        } else {
            "".to_string()
        }) + &(if hours >= 1 {
            itoa.format(hours).to_string() + "h "
        } else {
            "".to_string()
        }) + &(if minutes >= 1 {
            itoa.format(minutes).to_string() + "m "
        } else {
            "".to_string()
        }) + &(if time >= 1 {
            itoa.format(time).to_string() + "s"
        } else {
            "".to_string()
        });

        s.trim_end().to_string()
    }
}

impl<D> ToString for FancyDuration<D>
where
    D: AsSecs,
{
    fn to_string(&self) -> String {
        self.format()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::FancyDuration;

    #[test]
    fn test_duration_to_string() {
        assert_eq!(FancyDuration(Duration::new(600, 0)).to_string(), "10m");
        assert_eq!(FancyDuration(Duration::new(120, 0)).to_string(), "2m");
        assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");
        assert_eq!(
            FancyDuration(Duration::new(24 * 60 * 60, 0)).to_string(),
            "1d"
        );
        assert_eq!(FancyDuration(Duration::new(324, 0)).to_string(), "5m 24s");
        assert_eq!(
            FancyDuration(Duration::new(24 * 60 * 60 + 324, 0)).to_string(),
            "1d 5m 24s"
        );
        assert_eq!(
            FancyDuration(Duration::new(27 * 24 * 60 * 60 + 324, 0)).to_string(),
            "3w 6d 5m 24s"
        );
        assert_eq!(
            FancyDuration(Duration::new(99 * 24 * 60 * 60 + 324, 0)).to_string(),
            "3m 1w 2d 5m 24s"
        );
    }

    #[test]
    fn test_time_duration_to_string() {
        assert_eq!(
            FancyDuration(time::Duration::new(600, 0)).to_string(),
            "10m"
        );
        assert_eq!(FancyDuration(time::Duration::new(120, 0)).to_string(), "2m");
        assert_eq!(
            FancyDuration(time::Duration::new(185, 0)).to_string(),
            "3m 5s"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(24 * 60 * 60, 0)).to_string(),
            "1d"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(324, 0)).to_string(),
            "5m 24s"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(24 * 60 * 60 + 324, 0)).to_string(),
            "1d 5m 24s"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(27 * 24 * 60 * 60 + 324, 0)).to_string(),
            "3w 6d 5m 24s"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(99 * 24 * 60 * 60 + 324, 0)).to_string(),
            "3m 1w 2d 5m 24s"
        );
    }
}
