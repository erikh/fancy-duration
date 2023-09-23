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

/// AsTimes is the trait that allows [FancyDuration] to represent durations. Implementing these
/// methods will allow any compatible type to work with FancyDuration.
pub trait AsTimes: Sized {
    /// To implement a fancier duration, just have your duration return the seconds and nanoseconds (in
    /// a tuple) as a part of the following method call, as well as a method to handle parsing. The
    /// nanoseconds value should just represent the subsecond count, not the seconds.
    fn as_times(&self) -> (u64, u64);
    /// This function implements parsing to return the inner duration. [FancyDuration::parse_to_ns]
    /// is the standard parser and provides you with data to construct most duration types.
    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error>;
}

impl AsTimes for Duration {
    fn as_times(&self) -> (u64, u64) {
        let secs = self.as_secs();
        let nanos = self.as_nanos();

        (
            secs,
            (nanos - (nanos / 1e9 as u128) * 1e9 as u128)
                .try_into()
                .unwrap(),
        )
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        let ns = FancyDuration::<Duration>::parse_to_ns(s)?;
        Ok(Duration::new(ns.0, ns.1.try_into()?))
    }
}

#[cfg(feature = "chrono")]
impl AsTimes for chrono::Duration {
    fn as_times(&self) -> (u64, u64) {
        let secs = self.num_seconds();
        let nanos = self.num_nanoseconds().unwrap();

        (
            secs.try_into().unwrap(),
            (nanos - (nanos / 1e9 as i64) * 1e9 as i64)
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

/// A [FancyDuration] contains a duration of type that implements [AsTimes]. It is capable of that
/// point at parsing strings as well as returning the duration value encapsulated. If included in a
/// serde serializing or deserializing workflow, it will automatically construct the appropriate
/// duration as a part of the process.
///
/// Support for [time] and [chrono] are available as a part of this library via feature flags.
///
/// A duration is "human-readable" when it follows the following format:
///
/// ```ignore
/// <count><timespec>...
/// ```
///
/// This pattern repeats in an expected and prescribed order of precedence based on what duration
/// is supplied. Certain durations are order-dependent (like months and minutes), but most are not;
/// that said it should be desired to represent your durations in precedence order. If you express
/// standard formatting, each unit is separated by whitespace, such as "2m 5s 30ms", compact
/// formatting removes the whitespace: "2m5s30ms".
///
/// `count` is simply an integer value with no leading zero-padding. `timespec` is a one or two
/// character identifier that specifies the unit of time the count represents. The following
/// timespecs are supported, and more may be added in the future based on demand.
///
/// The order here is precedence-order. So to express this properly, one might say "5y2d30m" which
/// means "5 years, 2 days and 30 minutes", but "5y30m2d" means "5 years, 30 months, and 2 days".
///
/// - y: years
/// - m: months (must appear before `minutes`)
/// - w: weeks
/// - d: days
/// - h: hours
/// - m: minutes (must appear after `months`)
/// - s: seconds
/// - ms: milliseconds
/// - us: microseconds
/// - ns: nanoseconds
///
/// Simplifications:
///
/// Some time units have been simplified:
///
/// - Years is 365 days
/// - Months is 30 days
///
/// These durations do not account for variations in the potential unit based on the current time.
/// Perhaps in a future release.
///
#[derive(Clone, Debug, PartialEq)]
pub struct FancyDuration<D: AsTimes>(pub D);

impl<D> FancyDuration<D>
where
    D: AsTimes,
{
    /// Construct a fancier duration!
    ///
    /// Accept input of a Duration type that implements [AsTimes]. From here, strings containing
    /// human-friendly durations can be constructed, or the inner duration can be retrieved.
    pub fn new(d: D) -> Self {
        Self(d)
    }

    /// Retrieve the inner duration.
    pub fn duration(&self) -> D
    where
        D: Clone,
    {
        self.0.clone()
    }

    /// Parse a string that contains a human-readable duration. See [FancyDuration] for more
    /// information on how times are represented.
    pub fn parse(s: &str) -> Result<Self, anyhow::Error> {
        Ok(FancyDuration::new(D::parse_to_duration(s)?))
    }

    /// Supply the standard formatted human-readable representation of the duration. This format
    /// contains whitespace.
    pub fn format(&self) -> String {
        self.format_internal(true)
    }

    /// Supply the compact formatted human-readable representation of the duration. This format
    /// does not contain whitespace.
    pub fn format_compact(&self) -> String {
        self.format_internal(false)
    }

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

        let ms = times.1 / 1e6 as u64;
        times.1 -= ms * 1e6 as u64;
        let us = times.1 / 1e3 as u64;
        times.1 -= us * 1e3 as u64;

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
            itoa.format(times.0).to_string() + "s" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if ms > 0 {
            itoa.format(ms).to_string() + "ms" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if us > 0 {
            itoa.format(us).to_string() + "us" + if pad { " " } else { "" }
        } else {
            "".to_string()
        }) + &(if times.1 > 0 {
            itoa.format(times.1).to_string() + "ns" + if pad { " " } else { "" }
        } else {
            "".to_string()
        });

        s.trim_end().to_string()
    }

    /// Parse a string in fancy duration format to a tuple of (seconds, nanoseconds). Nanoseconds
    /// is simply a subsecond count and does not contain the seconds represented as nanoseconds. If
    /// a parsing error occurs that will appear in the result.
    pub fn parse_to_ns(s: &str) -> Result<(u64, u64), anyhow::Error> {
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
                    subseconds += result * 1e6 as u64;
                }
                "us" => {
                    let result: u64 = value.parse()?;
                    subseconds += result * 1e3 as u64;
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
        assert_eq!(FancyDuration(Duration::new(0, 600)).to_string(), "600ns");
        assert_eq!(FancyDuration(Duration::new(0, 600000)).to_string(), "600us");
        assert_eq!(
            FancyDuration(Duration::new(0, 600000000)).to_string(),
            "600ms"
        );
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
            FancyDuration(time::Duration::new(0, 600)).to_string(),
            "600ns"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(0, 600000)).to_string(),
            "600us"
        );
        assert_eq!(
            FancyDuration(time::Duration::new(0, 600000000)).to_string(),
            "600ms"
        );
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
            FancyDuration(chrono::Duration::nanoseconds(600)).to_string(),
            "600ns"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::microseconds(600)).to_string(),
            "600us"
        );
        assert_eq!(
            FancyDuration(chrono::Duration::milliseconds(600)).to_string(),
            "600ms"
        );
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
            ("1m 10ms", Duration::new(60, 10000000)),
            ("1h 30us", Duration::new(60 * 60, 30000)),
            ("1d 30ns", Duration::new(60 * 60 * 24, 30)),
            ("10s", Duration::new(10, 0)),
            ("3m 5s", Duration::new(185, 0)),
            ("3m 2w 2d 10m 10s", Duration::new(9159010, 0)),
        ];

        let compact_duration_table = [
            ("10s30ns", Duration::new(10, 30)),
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
                ("1m 10ms", time::Duration::new(60, 10000000)),
                ("1h 30us", time::Duration::new(60 * 60, 30000)),
                ("1d 30ns", time::Duration::new(60 * 60 * 24, 30)),
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
                (
                    "1m 10ms",
                    chrono::Duration::seconds(60) + chrono::Duration::milliseconds(10),
                ),
                (
                    "1h 30us",
                    chrono::Duration::hours(1) + chrono::Duration::microseconds(30),
                ),
                (
                    "1d 30ns",
                    chrono::Duration::days(1) + chrono::Duration::nanoseconds(30),
                ),
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
            ("{\"duration\":\"10ns\"}", Duration::new(0, 10)),
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
                ("{\"duration\":\"10ns\"}", time::Duration::new(0, 10)),
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
                ("{\"duration\":\"10ns\"}", chrono::Duration::nanoseconds(10)),
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
