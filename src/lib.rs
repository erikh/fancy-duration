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
//!     assert_eq!(FancyDuration::<Duration>::parse("3m 5s").unwrap().duration(), Duration::new(185, 0));
//!     assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");
//!
//!     // also works with time::Duration from the `time` crate
//!     assert_eq!(FancyDuration(time::Duration::new(20, 0)).to_string(), "20s");
//!     assert_eq!(FancyDuration(time::Duration::new(600, 0)).to_string(), "10m");
//!     assert_eq!(FancyDuration(time::Duration::new(120, 0)).to_string(), "2m");
//!     assert_eq!(FancyDuration(time::Duration::new(185, 0)).to_string(), "3m 5s");
//!     assert_eq!(FancyDuration::<time::Duration>::parse("3m 5s").unwrap().duration(), time::Duration::new(185, 0));
//!     assert_eq!(FancyDuration(time::Duration::new(185, 0)).to_string(), "3m 5s");
//! }
//! ```
//!
//! It also has [serde] support.
//!

use serde::{de::Visitor, Deserialize, Serialize};
use std::{marker::PhantomData, time::Duration};

/// To implement a fancier duration, just have your duration return the number of seconds as a part
/// of the following method call, as well as a method to handle parsing.
pub trait AsSecs: Sized {
    fn as_secs(&self) -> i64;
    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error>;
}

impl AsSecs for Duration {
    fn as_secs(&self) -> i64 {
        self.as_secs_f64() as i64
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        Ok(Duration::new(
            FancyDuration::<Duration>::parse_to_seconds(s).map(|s| s.abs() as u64)?,
            0,
        ))
    }
}

impl AsSecs for time::Duration {
    fn as_secs(&self) -> i64 {
        self.whole_seconds() as i64
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        Ok(time::Duration::new(
            FancyDuration::<time::Duration>::parse_to_seconds(s)?,
            0,
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct FancyDuration<D: AsSecs>(pub D);

impl<D> FancyDuration<D>
where
    D: AsSecs,
{
    /// Construct a fancier duration!
    pub fn new(d: D) -> Self {
        Self(d)
    }

    pub fn duration(&self) -> D
    where
        D: Clone,
    {
        self.0.clone()
    }

    pub fn parse(s: &str) -> Result<Self, anyhow::Error> {
        Ok(FancyDuration::new(D::parse_to_duration(s)?))
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

    fn parse_to_seconds(s: &str) -> Result<i64, anyhow::Error> {
        let mut secs: i64 = 0;
        let mut past_minutes = false;

        for item in s.split(" ").collect::<Vec<&str>>().iter().rev() {
            let item = item.trim();

            match item.chars().last() {
                Some('s') => {
                    let item = item.strip_suffix('s').unwrap();
                    let result: i64 = item.parse()?;
                    secs += result
                }
                Some('m') => {
                    let item = item.strip_suffix('m').unwrap();
                    let result: i64 = item.parse()?;
                    secs += if past_minutes {
                        result * 60 * 60 * 24 * 30
                    } else {
                        past_minutes = true;
                        result * 60
                    }
                }
                Some('h') => {
                    past_minutes = true;
                    let item = item.strip_suffix('h').unwrap();
                    let result: i64 = item.parse()?;
                    secs += result * 60 * 60;
                }
                Some('d') => {
                    past_minutes = true;
                    let item = item.strip_suffix('d').unwrap();
                    let result: i64 = item.parse()?;
                    secs += result * 60 * 60 * 24;
                }
                Some('w') => {
                    past_minutes = true;
                    let item = item.strip_suffix('w').unwrap();
                    let result: i64 = item.parse()?;
                    secs += result * 60 * 60 * 24 * 7;
                }
                Some(_) | None => {}
            }
        }

        Ok(secs)
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

impl<D> Serialize for FancyDuration<D>
where
    D: AsSecs,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct FancyDurationVisitor<D: AsSecs>(PhantomData<D>);

impl<D> Visitor<'_> for FancyDurationVisitor<D>
where
    D: AsSecs,
{
    type Value = FancyDuration<D>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a duration in 'fancy' format")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match FancyDuration::parse(v) {
            Ok(res) => Ok(res),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

impl<'de, T> Deserialize<'de> for FancyDuration<T>
where
    T: AsSecs,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FancyDurationVisitor(PhantomData::default()))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{AsSecs, FancyDuration};

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

    #[test]
    fn test_parse_duration() {
        let duration_table = [
            ("10s", Duration::new(10, 0)),
            ("3m 5s", Duration::new(185, 0)),
            ("3m 2w 2d 10m 10s", Duration::new(9159010, 0)),
        ];

        let time_table = [
            ("10s", time::Duration::new(10, 0)),
            ("3m 5s", time::Duration::new(185, 0)),
            ("3m 2w 2d 10m 10s", time::Duration::new(9159010, 0)),
        ];

        for item in duration_table {
            let fancy = FancyDuration::<Duration>::parse(item.0).unwrap();
            assert_eq!(fancy.duration(), item.1);
            assert_eq!(FancyDuration::new(item.1).to_string(), item.0);
        }

        for item in time_table {
            let fancy = FancyDuration::<Duration>::parse(item.0).unwrap();
            assert_eq!(fancy.duration(), item.1);
            assert_eq!(FancyDuration::new(item.1).to_string(), item.0);
        }
    }

    #[test]
    fn test_serde() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct MyDuration<D: AsSecs> {
            duration: FancyDuration<D>,
        }

        let duration_table = [
            ("{\"duration\":\"10s\"}", Duration::new(10, 0)),
            ("{\"duration\":\"3m 5s\"}", Duration::new(185, 0)),
            (
                "{\"duration\":\"3m 2w 2d 10m 10s\"}",
                Duration::new(9159010, 0),
            ),
        ];

        let time_table = [
            ("{\"duration\":\"10s\"}", time::Duration::new(10, 0)),
            ("{\"duration\":\"3m 5s\"}", time::Duration::new(185, 0)),
            (
                "{\"duration\":\"3m 2w 2d 10m 10s\"}",
                time::Duration::new(9159010, 0),
            ),
        ];

        for item in duration_table {
            let md: MyDuration<Duration> = serde_json::from_str(item.0).unwrap();
            assert_eq!(md.duration.duration(), item.1);
            assert_eq!(serde_json::to_string(&md).unwrap(), item.0);
        }

        for item in time_table {
            let md: MyDuration<time::Duration> = serde_json::from_str(item.0).unwrap();
            assert_eq!(md.duration.duration(), item.1);
            assert_eq!(serde_json::to_string(&md).unwrap(), item.0);
        }
    }
}
