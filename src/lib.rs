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
//!     #[cfg(feature = "time")]
//!     {
//!         // also works with time::Duration from the `time` crate
//!         assert_eq!(FancyDuration(time::Duration::new(20, 0)).to_string(), "20s");
//!         assert_eq!(FancyDuration(time::Duration::new(600, 0)).to_string(), "10m");
//!         assert_eq!(FancyDuration(time::Duration::new(120, 0)).to_string(), "2m");
//!         assert_eq!(FancyDuration(time::Duration::new(185, 0)).to_string(), "3m 5s");
//!         assert_eq!(FancyDuration::<time::Duration>::parse("3m 5s").unwrap().duration(), time::Duration::new(185, 0));
//!         assert_eq!(FancyDuration(time::Duration::new(185, 0)).to_string(), "3m 5s");
//!     }
//!
//!     #[cfg(feature = "chrono")]
//!     {
//!         // also works with chrono!
//!         assert_eq!(FancyDuration(chrono::Duration::seconds(20)).to_string(), "20s");
//!         assert_eq!(FancyDuration(chrono::Duration::seconds(600)).to_string(), "10m");
//!         assert_eq!(FancyDuration(chrono::Duration::seconds(120)).to_string(), "2m");
//!         assert_eq!(FancyDuration(chrono::Duration::seconds(185)).to_string(), "3m 5s");
//!         assert_eq!(FancyDuration::<chrono::Duration>::parse("3m 5s").unwrap().duration(), chrono::Duration::seconds(185));
//!         assert_eq!(FancyDuration(chrono::Duration::seconds(185)).to_string(), "3m 5s");
//!     }
//! }
//! ```
//!
//! It also has [serde] support.
//!

use serde::{de::Visitor, Deserialize, Serialize};
use std::{marker::PhantomData, time::Duration};

/// To implement a fancier duration, just have your duration return the number of nanoseconds as a
/// part of the following method call, as well as a method to handle parsing.
pub trait AsTimes: Sized {
    fn as_times(&self) -> (u64, u64);
    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error>;
}

impl AsTimes for Duration {
    fn as_times(&self) -> (u64, u64) {
        (self.as_secs(), self.subsec_nanos() as u64)
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        let ns = FancyDuration::<Duration>::parse_to_ns(s)?;
        Ok(Duration::new(ns.0, ns.1.try_into()?))
    }
}

#[cfg(feature = "chrono")]
impl AsTimes for chrono::Duration {
    fn as_times(&self) -> (u64, u64) {
        (
            self.num_seconds().try_into().unwrap(),
            (self.num_nanoseconds().unwrap() / 1e9 as i64)
                .try_into()
                .unwrap(),
        )
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        let ns = FancyDuration::<chrono::Duration>::parse_to_ns(s)?;

        Ok(chrono::Duration::seconds(ns.0.try_into()?)
            + chrono::Duration::nanoseconds(ns.1.try_into()?))
    }
}

#[cfg(feature = "time")]
impl AsTimes for time::Duration {
    fn as_times(&self) -> (u64, u64) {
        (
            self.as_seconds_f64() as u64,
            self.subsec_nanoseconds() as u64,
        )
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        let ns = FancyDuration::<Duration>::parse_to_ns(s)?;
        Ok(time::Duration::new(ns.0.try_into()?, ns.1.try_into()?))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FancyDuration<D: AsTimes>(pub D);

impl<D> FancyDuration<D>
where
    D: AsTimes,
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
        let mut times = self.0.as_times();

        if times.0 == 0 && times.1 == 0 {
            return "0".to_string();
        }

        let years = times.0 / 12 / 30 / 24 / 60 / 60;
        times.0 -= years * 12 * 30 * 24 * 60 * 60;
        let months = times.0 / 30 / 24 / 60 / 60;
        times.0 -= months * 30 * 24 * 60 * 60;
        let weeks = times.0 / 7 / 24 / 60 / 60;
        times.0 -= weeks * 7 * 24 * 60 * 60;
        let days = times.0 / 24 / 60 / 60;
        times.0 -= days * 24 * 60 * 60;
        let hours = times.0 / 60 / 60;
        times.0 -= hours * 60 * 60;
        let minutes = times.0 / 60;
        times.0 -= minutes * 60;

        let mut itoa = itoa::Buffer::new();

        // I should fix this someday
        let s = if years > 0 {
            itoa.format(years).to_string() + "y" + if pad { " " } else { "" }
        } else {
            "".to_string()
        } + &(if months > 0 {
            itoa.format(months).to_string() + "m" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if weeks > 0 {
            itoa.format(weeks).to_string() + "w" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if days > 0 {
            itoa.format(days).to_string() + "d" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if hours > 0 {
            itoa.format(hours).to_string() + "h" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if minutes > 0 {
            itoa.format(minutes).to_string() + "m" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if times.0 > 0 {
            itoa.format(times.0).to_string() + "s"
        } else {
            "".to_string()
        });

        s.trim_end().to_string()
    }

    fn parse_to_ns(s: &str) -> Result<(u64, u64), anyhow::Error> {
        let mut subseconds: u64 = 0;
        let mut seconds: u64 = 0;
        let mut past_minutes = false;

        let rx = regex::Regex::new(r#"([0-9]+)([a-zA-Z]{1,2})\s*"#)?;
        let mut list: Vec<(&str, &str)> = Vec::new();

        for item in rx.captures_iter(s) {
            list.push((item.get(1).unwrap().as_str(), item.get(2).unwrap().as_str()));
        }

        for (value, suffix) in list.iter().rev() {
            match *suffix {
                "ns" => {
                    let result: u64 = value.parse()?;
                    subseconds += result;
                }
                "ms" => {
                    let result: u64 = value.parse()?;
                    subseconds += result / 1e5 as u64;
                }
                "us" => {
                    let result: u64 = value.parse()?;
                    subseconds += result / 1e3 as u64;
                }
                "s" => {
                    let result: u64 = value.parse()?;
                    seconds += result;
                }
                "m" => {
                    let result: u64 = value.parse()?;
                    seconds += if past_minutes {
                        result * 60 * 60 * 24 * 30
                    } else {
                        past_minutes = true;
                        result * 60
                    }
                }
                "h" => {
                    past_minutes = true;
                    let result: u64 = value.parse()?;
                    seconds += result * 60 * 60
                }
                "d" => {
                    past_minutes = true;
                    let result: u64 = value.parse()?;
                    seconds += result * 60 * 60 * 24
                }
                "w" => {
                    past_minutes = true;
                    let result: u64 = value.parse()?;
                    seconds += result * 60 * 60 * 24 * 7
                }
                "y" => {
                    past_minutes = true;
                    let result: u64 = value.parse()?;
                    seconds += result * 12 * 30 * 60 * 60 * 24
                }
                _ => {}
            }
        }

        Ok((seconds, subseconds))
    }
}

impl<D> ToString for FancyDuration<D>
where
    D: AsTimes,
{
    fn to_string(&self) -> String {
        self.format()
    }
}

impl<D> Serialize for FancyDuration<D>
where
    D: AsTimes,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct FancyDurationVisitor<D: AsTimes>(PhantomData<D>);

impl<D> Visitor<'_> for FancyDurationVisitor<D>
where
    D: AsTimes,
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
    T: AsTimes,
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
            FancyDuration(Duration::new(12 * 30 * 24 * 60 * 60 + 10 * 24 * 60 * 60, 0)).to_string(),
            "1y 1w 3d"
        );

        assert_eq!(
            FancyDuration(Duration::new(12 * 30 * 24 * 60 * 60 + 10 * 24 * 60 * 60, 0))
                .format_compact(),
            "1y1w3d"
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
            FancyDuration(time::Duration::new(
                12 * 30 * 24 * 60 * 60 + 10 * 24 * 60 * 60,
                0
            ))
            .to_string(),
            "1y 1w 3d"
        );

        assert_eq!(
            FancyDuration(time::Duration::new(
                12 * 30 * 24 * 60 * 60 + 10 * 24 * 60 * 60,
                0
            ))
            .format_compact(),
            "1y1w3d"
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
            FancyDuration(chrono::Duration::seconds(
                12 * 30 * 24 * 60 * 60 + 10 * 24 * 60 * 60,
            ))
            .to_string(),
            "1y 1w 3d"
        );

        assert_eq!(
            FancyDuration(chrono::Duration::seconds(
                12 * 30 * 24 * 60 * 60 + 10 * 24 * 60 * 60,
            ))
            .format_compact(),
            "1y1w3d"
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
                "{\"duration\":\"1y 3m 2w 2d 10m 10s\"}",
                Duration::new(40263010, 0),
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
                    "{\"duration\":\"1y 3m 2w 2d 10m 10s\"}",
                    time::Duration::new(40263010, 0),
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
                    "{\"duration\":\"1y 3m 2w 2d 10m 10s\"}",
                    chrono::Duration::seconds(40263010),
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
