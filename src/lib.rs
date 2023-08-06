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

#[cfg(feature = "chrono")]
impl AsSecs for chrono::Duration {
    fn as_secs(&self) -> i64 {
        self.num_seconds() as i64
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        Ok(chrono::Duration::seconds(
            FancyDuration::<chrono::Duration>::parse_to_seconds(s)?,
        ))
    }
}

#[cfg(feature = "time")]
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

#[derive(Clone, Debug, PartialEq)]
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

    pub fn format(&self) -> String {
        self.format_internal(true)
    }

    pub fn format_compact(&self) -> String {
        self.format_internal(false)
    }

    /// Show the duration in a fancier format!
    fn format_internal(&self, pad: bool) -> String {
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

        // I should fix this someday
        let s = if months >= 1 {
            itoa.format(months).to_string() + "m" + if pad { " " } else { "" }
        } else {
            "".to_string()
        } + &(if weeks >= 1 {
            itoa.format(weeks).to_string() + "w" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if days >= 1 {
            itoa.format(days).to_string() + "d" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if hours >= 1 {
            itoa.format(hours).to_string() + "h" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if minutes >= 1 {
            itoa.format(minutes).to_string() + "m" + if pad { " " } else { "" }
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

        let rx = regex::Regex::new(r#"([0-9]+[a-zA-Z])\s*"#)?;
        let list: Vec<&str> = rx
            .captures_iter(s)
            .flat_map(|c| {
                c.iter()
                    .skip(1)
                    .filter_map(|s| s.map(|h| h.as_str()))
                    .collect::<Vec<_>>()
            })
            .collect();

        for item in list.iter().rev() {
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

        assert_eq!(
            FancyDuration(Duration::new(324, 0)).format_compact(),
            "5m24s"
        );
        assert_eq!(
            FancyDuration(Duration::new(24 * 60 * 60 + 324, 0)).format_compact(),
            "1d5m24s"
        );
        assert_eq!(
            FancyDuration(Duration::new(27 * 24 * 60 * 60 + 324, 0)).format_compact(),
            "3w6d5m24s"
        );
        assert_eq!(
            FancyDuration(Duration::new(99 * 24 * 60 * 60 + 324, 0)).format_compact(),
            "3m1w2d5m24s"
        );
    }

    #[test]
    #[cfg(feature = "time")]
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

        assert_eq!(
            FancyDuration(time::Duration::new(24 * 60 * 60 + 324, 0)).format_compact(),
            "1d5m24s"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(27 * 24 * 60 * 60 + 324, 0)).format_compact(),
            "3w6d5m24s"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(99 * 24 * 60 * 60 + 324, 0)).format_compact(),
            "3m1w2d5m24s"
        );
    }

    #[test]
    #[cfg(feature = "chrono")]
    fn test_chrono_duration_to_string() {
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(600)).to_string(),
            "10m"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(120)).to_string(),
            "2m"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(185)).to_string(),
            "3m 5s"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(24 * 60 * 60)).to_string(),
            "1d"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(324)).to_string(),
            "5m 24s"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(24 * 60 * 60 + 324)).to_string(),
            "1d 5m 24s"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(27 * 24 * 60 * 60 + 324)).to_string(),
            "3w 6d 5m 24s"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(99 * 24 * 60 * 60 + 324)).to_string(),
            "3m 1w 2d 5m 24s"
        );

        assert_eq!(
            FancyDuration(chrono::Duration::seconds(24 * 60 * 60 + 324)).format_compact(),
            "1d5m24s"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(27 * 24 * 60 * 60 + 324)).format_compact(),
            "3w6d5m24s"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::seconds(99 * 24 * 60 * 60 + 324)).format_compact(),
            "3m1w2d5m24s"
        );
    }

    #[test]
    fn test_parse_duration() {
        let duration_table = [
            ("10s", Duration::new(10, 0)),
            ("3m 5s", Duration::new(185, 0)),
            ("3m 2w 2d 10m 10s", Duration::new(9159010, 0)),
        ];

        let compact_duration_table = [
            ("3m5s", Duration::new(185, 0)),
            ("3m2w2d10m10s", Duration::new(9159010, 0)),
        ];

        for item in duration_table {
            let fancy = FancyDuration::<Duration>::parse(item.0).unwrap();
            assert_eq!(fancy.duration(), item.1);
            assert_eq!(FancyDuration::new(item.1).to_string(), item.0);
        }

        for item in compact_duration_table {
            let fancy = FancyDuration::<Duration>::parse(item.0).unwrap();
            assert_eq!(fancy.duration(), item.1);
            assert_eq!(FancyDuration::new(item.1).format_compact(), item.0);
        }

        #[cfg(feature = "time")]
        {
            let time_table = [
                ("10s", time::Duration::new(10, 0)),
                ("3m 5s", time::Duration::new(185, 0)),
                ("3m 2w 2d 10m 10s", time::Duration::new(9159010, 0)),
            ];

            let compact_time_table = [
                ("3m5s", time::Duration::new(185, 0)),
                ("3m2w2d10m10s", time::Duration::new(9159010, 0)),
            ];
            for item in time_table {
                let fancy = FancyDuration::<time::Duration>::parse(item.0).unwrap();
                assert_eq!(fancy.duration(), item.1);
                assert_eq!(FancyDuration::new(item.1).to_string(), item.0);
            }

            for item in compact_time_table {
                let fancy = FancyDuration::<time::Duration>::parse(item.0).unwrap();
                assert_eq!(fancy.duration(), item.1);
                assert_eq!(FancyDuration::new(item.1).format_compact(), item.0);
            }
        }

        #[cfg(feature = "chrono")]
        {
            let chrono_table = [
                ("10s", chrono::Duration::seconds(10)),
                ("3m 5s", chrono::Duration::seconds(185)),
                ("3m 2w 2d 10m 10s", chrono::Duration::seconds(9159010)),
            ];

            let compact_chrono_table = [
                ("3m5s", chrono::Duration::seconds(185)),
                ("3m2w2d10m10s", chrono::Duration::seconds(9159010)),
            ];
            for item in chrono_table {
                let fancy = FancyDuration::<chrono::Duration>::parse(item.0).unwrap();
                assert_eq!(fancy.duration(), item.1);
                assert_eq!(FancyDuration::new(item.1).to_string(), item.0);
            }

            for item in compact_chrono_table {
                let fancy = FancyDuration::<chrono::Duration>::parse(item.0).unwrap();
                assert_eq!(fancy.duration(), item.1);
                assert_eq!(FancyDuration::new(item.1).format_compact(), item.0);
            }
        }
    }

    #[test]
    fn test_serde() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize)]
        struct StdDuration {
            duration: FancyDuration<std::time::Duration>,
        }

        let duration_table = [
            ("{\"duration\":\"10s\"}", Duration::new(10, 0)),
            ("{\"duration\":\"3m 5s\"}", Duration::new(185, 0)),
            (
                "{\"duration\":\"3m 2w 2d 10m 10s\"}",
                Duration::new(9159010, 0),
            ),
        ];

        for item in duration_table {
            let md: StdDuration = serde_json::from_str(item.0).unwrap();
            assert_eq!(md.duration.duration(), item.1);
            assert_eq!(serde_json::to_string(&md).unwrap(), item.0);
        }

        #[cfg(feature = "time")]
        {
            #[derive(Serialize, Deserialize)]
            struct TimeDuration {
                duration: FancyDuration<time::Duration>,
            }

            let time_table = [
                ("{\"duration\":\"10s\"}", time::Duration::new(10, 0)),
                ("{\"duration\":\"3m 5s\"}", time::Duration::new(185, 0)),
                (
                    "{\"duration\":\"3m 2w 2d 10m 10s\"}",
                    time::Duration::new(9159010, 0),
                ),
            ];

            for item in time_table {
                let md: TimeDuration = serde_json::from_str(item.0).unwrap();
                assert_eq!(md.duration.duration(), item.1);
                assert_eq!(serde_json::to_string(&md).unwrap(), item.0);
            }
        }

        #[cfg(feature = "chrono")]
        {
            #[derive(Serialize, Deserialize)]
            struct ChronoDuration {
                duration: FancyDuration<chrono::Duration>,
            }

            let chrono_table = [
                ("{\"duration\":\"10s\"}", chrono::Duration::seconds(10)),
                ("{\"duration\":\"3m 5s\"}", chrono::Duration::seconds(185)),
                (
                    "{\"duration\":\"3m 2w 2d 10m 10s\"}",
                    chrono::Duration::seconds(9159010),
                ),
            ];

            for item in chrono_table {
                let md: ChronoDuration = serde_json::from_str(item.0).unwrap();
                assert_eq!(md.duration.duration(), item.1);
                assert_eq!(serde_json::to_string(&md).unwrap(), item.0);
            }
        }
    }
}
