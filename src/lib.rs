//!
//! A "fancy duration" is a text description of the duration. For example, "1h 20m 30s" which might
//! be read as "one hour, twenty minutes and thirty seconds". Expression in a duration type is
//! transparent through a variety of means; chrono and the time crate are supported, as well as
//! serialization into and from string types with serde. Time support starts in years and funnels
//! down to nanoseconds.
//!
//! Feature matrix:
//!   - serde: enables serde support including serialization and deseralization from strings
//!   - time: enables traits that implement fancy duration features for the `time` crate
//!   - chrono: enables traits that implement fancy duration features for the `chrono` crate
//!
//! What follows are some usage examples. You can either wrap your duration-like type in a
//! FancyDuration struct, or use types which allow for monkeypatched methods that allow you to work
//! directly on the target type. For example, use AsFancyDuration to inject fancy_duration calls to
//! perform the construction (which can be formatted or converted to string) and ParseFancyDuration
//! to inject parse_fancy_duration constructors to accept strings into your favorite type.
//! std::time::Duration, time::Duration, and chrono::Duration are all supported (some features may
//! need to be required) and you can make more types eligible by implementing the AsTimes trait.
//!
//! ```
//! use std::time::Duration;
//! use fancy_duration::FancyDuration;
//!
//! pub fn main() {
//!     // use struct-wrapped or monkeypatched approaches with fancy_duration::AsFancyDuration;
//!     assert_eq!(FancyDuration(Duration::new(20, 0)).to_string(), "20s");
//!     assert_eq!(FancyDuration(Duration::new(600, 0)).to_string(), "10m");
//!     assert_eq!(FancyDuration(Duration::new(120, 0)).to_string(), "2m");
//!     assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");
//!     assert_eq!(FancyDuration::<Duration>::parse("3m 5s").unwrap().duration(), Duration::new(185, 0));
//!     assert_eq!(FancyDuration(Duration::new(185, 0)).to_string(), "3m 5s");
//!
//!     // these traits are also implemented for chrono and time
//!     use fancy_duration::{ParseFancyDuration, AsFancyDuration};
//!     assert_eq!(Duration::new(20, 0).fancy_duration().to_string(), "20s");
//!     assert_eq!(Duration::new(600, 0).fancy_duration().to_string(), "10m");
//!     assert_eq!(Duration::new(120, 0).fancy_duration().to_string(), "2m");
//!     assert_eq!(Duration::new(185, 0).fancy_duration().to_string(), "3m 5s");
//!     assert_eq!(Duration::parse_fancy_duration("3m 5s".to_string()).unwrap(), Duration::new(185, 0));
//!     assert_eq!(Duration::new(185, 0).fancy_duration().to_string(), "3m 5s");
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
//!         assert_eq!(FancyDuration(chrono::TimeDelta::try_seconds(20).unwrap_or_default()).to_string(), "20s");
//!         assert_eq!(FancyDuration(chrono::TimeDelta::try_seconds(600).unwrap_or_default()).to_string(), "10m");
//!         assert_eq!(FancyDuration(chrono::TimeDelta::try_seconds(120).unwrap_or_default()).to_string(), "2m");
//!         assert_eq!(FancyDuration(chrono::TimeDelta::try_seconds(185).unwrap_or_default()).to_string(), "3m 5s");
//!         assert_eq!(FancyDuration::<chrono::Duration>::parse("3m 5s").unwrap().duration(), chrono::TimeDelta::try_seconds(185).unwrap_or_default());
//!         assert_eq!(FancyDuration(chrono::TimeDelta::try_seconds(185).unwrap_or_default()).to_string(), "3m 5s");
//!     }
//! }
//! ```

lazy_static::lazy_static! {
    static ref FANCY_FORMAT: regex::Regex = regex::Regex::new(r#"([0-9]+)([a-zA-Z]{1,2})\s*"#).unwrap();
}

#[cfg(feature = "serde")]
use serde::{de::Visitor, Deserialize, Serialize};
#[cfg(feature = "serde")]
use std::marker::PhantomData;
use std::time::Duration;

/// Implement AsFancyDuration for your Duration type, it will annotate those types with the
/// `fancy_duration` function which allows trivial and explicit conversion into a fancy duration.
pub trait AsFancyDuration<T>
where
    Self: Sized,
    T: AsTimes + Clone,
{
    /// Convert T to a fancy_duration, which can be converted to a string representation of the
    /// duration.
    fn fancy_duration(&self) -> FancyDuration<T>;
}

/// Implement ParseFancyDuration for your Duration type to implement parsing constructors for your
/// Duration. A more generic `parse` implementation for String and &str may come in a future
/// version.
pub trait ParseFancyDuration<T>
where
    Self: Sized,
    T: AsTimes + Clone,
{
    /// Parse T from String, which allows the construction of a T from the fancy duration specified
    /// in the string.
    fn parse_fancy_duration(s: String) -> Result<Self, anyhow::Error>;
}

impl ParseFancyDuration<Duration> for Duration {
    fn parse_fancy_duration(s: String) -> Result<Self, anyhow::Error> {
        Ok(FancyDuration::<Duration>::parse(&s)?.duration())
    }
}

impl AsFancyDuration<Duration> for Duration {
    fn fancy_duration(&self) -> FancyDuration<Duration> {
        FancyDuration::new(self.clone())
    }
}

impl<D> std::str::FromStr for FancyDuration<D>
where
    D: AsTimes + Clone,
{
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(feature = "time")]
impl ParseFancyDuration<time::Duration> for time::Duration {
    fn parse_fancy_duration(s: String) -> Result<Self, anyhow::Error> {
        Ok(FancyDuration::<time::Duration>::parse(&s)?.duration())
    }
}

#[cfg(feature = "time")]
impl AsFancyDuration<time::Duration> for time::Duration {
    fn fancy_duration(&self) -> FancyDuration<time::Duration> {
        FancyDuration::new(self.clone())
    }
}

#[cfg(feature = "chrono")]
impl ParseFancyDuration<chrono::Duration> for chrono::Duration {
    fn parse_fancy_duration(s: String) -> Result<Self, anyhow::Error> {
        Ok(FancyDuration::<chrono::Duration>::parse(&s)?.duration())
    }
}

#[cfg(feature = "chrono")]
impl AsFancyDuration<chrono::Duration> for chrono::Duration {
    fn fancy_duration(&self) -> FancyDuration<chrono::Duration> {
        FancyDuration::new(self.clone())
    }
}

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
    /// Yield one of this implementing duration from a pair of (seconds, nanoseconds).
    fn from_times(&self, s: u64, ns: u64) -> Self;
}

impl AsTimes for Duration {
    fn as_times(&self) -> (u64, u64) {
        (self.as_secs(), self.subsec_nanos() as u64)
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        let ns = FancyDuration::<Duration>::parse_to_ns(s)?;
        Ok(Duration::new(ns.0, ns.1.try_into()?))
    }

    fn from_times(&self, s: u64, ns: u64) -> Self {
        Duration::new(s, ns.try_into().unwrap())
    }
}

#[cfg(feature = "chrono")]
impl AsTimes for chrono::Duration {
    fn as_times(&self) -> (u64, u64) {
        let secs = self.num_seconds();
        let nanos = self.subsec_nanos();

        (secs as u64, nanos as u64)
    }

    fn parse_to_duration(s: &str) -> Result<Self, anyhow::Error> {
        let ns = FancyDuration::<chrono::Duration>::parse_to_ns(s)?;

        Ok(
            chrono::TimeDelta::try_seconds(ns.0.try_into()?).unwrap_or_default()
                + chrono::Duration::nanoseconds(ns.1.try_into()?),
        )
    }

    fn from_times(&self, s: u64, ns: u64) -> Self {
        chrono::TimeDelta::try_seconds(s.try_into().unwrap()).unwrap_or_default()
            + chrono::Duration::nanoseconds(ns.try_into().unwrap())
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

    fn from_times(&self, s: u64, ns: u64) -> Self {
        time::Duration::new(s.try_into().unwrap(), ns.try_into().unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DurationPart {
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
    Milliseconds,
    Microseconds,
    Nanoseconds,
}

#[derive(Debug, Clone)]
pub(crate) struct DurationBreakdown {
    pub(crate) years: u64,
    pub(crate) months: u64,
    pub(crate) weeks: u64,
    pub(crate) days: u64,
    pub(crate) hours: u64,
    pub(crate) minutes: u64,
    pub(crate) seconds: u64,
    pub(crate) milliseconds: u64,
    pub(crate) microseconds: u64,
    pub(crate) nanoseconds: u64,
}

const MINUTE: u64 = 60;
const HOUR: u64 = 60 * MINUTE;
const DAY: u64 = 24 * HOUR;
const WEEK: u64 = 7 * DAY;
const MONTH: u64 = 30 * DAY;
const YEAR: u64 = 365 * DAY;

impl DurationBreakdown {
    pub(crate) fn new(mut s: u64, mut ns: u64) -> Self {
        let years = s / YEAR;
        s -= years * YEAR;
        let months = s / MONTH;
        s -= months * MONTH;
        let weeks = s / WEEK;
        s -= weeks * WEEK;
        let days = s / DAY;
        s -= days * DAY;
        let hours = s / HOUR;
        s -= hours * HOUR;
        let minutes = s / MINUTE;
        s -= minutes * MINUTE;

        let ms = ns / 1e6 as u64;
        ns -= ms * 1e6 as u64;
        let us = ns / 1e3 as u64;
        ns -= us * 1e3 as u64;

        Self {
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds: s,
            milliseconds: ms,
            microseconds: us,
            nanoseconds: ns,
        }
    }

    pub(crate) fn truncate(&self, mut limit: usize) -> Self {
        let mut obj = self.clone();
        let mut limit_started = false;

        for val in [
            &mut obj.years,
            &mut obj.months,
            &mut obj.weeks,
            &mut obj.days,
            &mut obj.hours,
            &mut obj.minutes,
            &mut obj.seconds,
            &mut obj.milliseconds,
            &mut obj.microseconds,
            &mut obj.nanoseconds,
        ] {
            if limit_started || *val > 0 {
                limit_started = true;

                if limit == 0 {
                    *val = 0
                }

                if limit != 0 {
                    limit -= 1;
                }
            }
        }

        obj
    }

    pub fn filter(&self, filter: &[DurationPart]) -> Self {
        let mut obj = self.clone();

        let all = &[
            DurationPart::Years,
            DurationPart::Months,
            DurationPart::Weeks,
            DurationPart::Days,
            DurationPart::Hours,
            DurationPart::Minutes,
            DurationPart::Seconds,
            DurationPart::Milliseconds,
            DurationPart::Microseconds,
            DurationPart::Nanoseconds,
        ];

        for part in all {
            if !filter.contains(part) {
                match part {
                    DurationPart::Years => obj.years = 0,
                    DurationPart::Months => obj.months = 0,
                    DurationPart::Weeks => obj.weeks = 0,
                    DurationPart::Days => obj.days = 0,
                    DurationPart::Hours => obj.hours = 0,
                    DurationPart::Minutes => obj.minutes = 0,
                    DurationPart::Seconds => obj.seconds = 0,
                    DurationPart::Milliseconds => obj.milliseconds = 0,
                    DurationPart::Microseconds => obj.microseconds = 0,
                    DurationPart::Nanoseconds => obj.nanoseconds = 0,
                }
            }
        }

        obj
    }

    pub fn as_times(&self) -> (u64, u64) {
        let mut s = 0;
        let mut ns = 0;

        s += self.years * YEAR
            + self.months * MONTH
            + self.weeks * WEEK
            + self.days * DAY
            + self.hours * HOUR
            + self.minutes * MINUTE
            + self.seconds;
        ns += self.milliseconds * 1e6 as u64 + self.microseconds * 1e3 as u64 + self.nanoseconds;

        (s, ns)
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
pub struct FancyDuration<D: AsTimes + Clone>(pub D);

impl<D> FancyDuration<D>
where
    D: AsTimes + Clone,
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

    /// Supply a filter of allowed time values, others will be zeroed out and the time recalculated
    /// as if they didn't exist.
    pub fn filter(&self, filter: &[DurationPart]) -> Self {
        let mut obj = self.clone();
        let times = self.0.as_times();
        let filtered = DurationBreakdown::new(times.0, times.1)
            .filter(filter)
            .as_times();
        obj.0 = self.0.from_times(filtered.0, filtered.1);
        obj
    }

    /// Truncate to the most significant consecutive values. This will take a number like "1y 2m 3w
    /// 4d" and with a value of 2 reduce it to "1y 2m". Since it works consecutively, minor values
    /// will also be dropped, such as "1h 2m 30us", truncated to 3, would still produce "1h 2m"
    /// because "30us" is below the seconds value, which is more significant and would have been
    /// counted. "1h 2m 3s" would truncate to 3 with "1h 2m 3s".
    pub fn truncate(&self, limit: usize) -> Self {
        let mut obj = self.clone();
        let times = self.0.as_times();
        let truncated = DurationBreakdown::new(times.0, times.1)
            .truncate(limit)
            .as_times();
        obj.0 = self.0.from_times(truncated.0, truncated.1);
        obj
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
        let times = self.0.as_times();

        if times.0 == 0 && times.1 == 0 {
            return "0".to_string();
        }

        let breakdown = DurationBreakdown::new(times.0, times.1);

        let mut s = String::new();

        let spad = if pad { " " } else { "" };

        if breakdown.years > 0 {
            s += &format!("{}y{}", breakdown.years, spad)
        }

        if breakdown.months > 0 {
            s += &format!("{}m{}", breakdown.months, spad)
        }

        if breakdown.weeks > 0 {
            s += &format!("{}w{}", breakdown.weeks, spad)
        }

        if breakdown.days > 0 {
            s += &format!("{}d{}", breakdown.days, spad)
        }

        if breakdown.hours > 0 {
            s += &format!("{}h{}", breakdown.hours, spad)
        }

        if breakdown.minutes > 0 {
            s += &format!("{}m{}", breakdown.minutes, spad)
        }

        if breakdown.seconds > 0 {
            s += &format!("{}s{}", breakdown.seconds, spad)
        }

        if breakdown.milliseconds > 0 {
            s += &format!("{}ms{}", breakdown.milliseconds, spad)
        }

        if breakdown.microseconds > 0 {
            s += &format!("{}us{}", breakdown.microseconds, spad)
        }

        if breakdown.nanoseconds > 0 {
            s += &format!("{}ns{}", breakdown.nanoseconds, spad)
        }

        if pad {
            s.truncate(s.len() - 1);
        }

        s
    }

    /// Parse a string in fancy duration format to a tuple of (seconds, nanoseconds). Nanoseconds
    /// is simply a subsecond count and does not contain the seconds represented as nanoseconds. If
    /// a parsing error occurs that will appear in the result.
    pub fn parse_to_ns(s: &str) -> Result<(u64, u64), anyhow::Error> {
        let mut subseconds: u64 = 0;
        let mut seconds: u64 = 0;
        let mut past_minutes = false;

        let mut list: Vec<(&str, &str)> = Vec::new();

        for item in FANCY_FORMAT.captures_iter(s) {
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

impl<D> std::fmt::Display for FancyDuration<D>
where
    D: AsTimes + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format())
    }
}

#[cfg(feature = "serde")]
impl<D> Serialize for FancyDuration<D>
where
    D: AsTimes + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
struct FancyDurationVisitor<D: AsTimes>(PhantomData<D>);

#[cfg(feature = "serde")]
impl<D> Visitor<'_> for FancyDurationVisitor<D>
where
    D: AsTimes + Clone,
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

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for FancyDuration<T>
where
    T: AsTimes + Clone,
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
    fn test_fancy_duration_call() {
        use super::{AsFancyDuration, ParseFancyDuration};

        assert_eq!(Duration::new(0, 600).fancy_duration().to_string(), "600ns");
        #[cfg(feature = "time")]
        assert_eq!(
            time::Duration::new(0, 600).fancy_duration().to_string(),
            "600ns"
        );
        #[cfg(feature = "chrono")]
        assert_eq!(
            chrono::Duration::nanoseconds(600)
                .fancy_duration()
                .to_string(),
            "600ns"
        );
        assert_eq!(
            Duration::parse_fancy_duration("600ns".to_string()).unwrap(),
            Duration::new(0, 600)
        );
        #[cfg(feature = "time")]
        assert_eq!(
            time::Duration::parse_fancy_duration("600ns".to_string()).unwrap(),
            time::Duration::new(0, 600)
        );
        #[cfg(feature = "chrono")]
        assert_eq!(
            chrono::Duration::parse_fancy_duration("600ns".to_string()).unwrap(),
            chrono::Duration::nanoseconds(600)
        );
    }

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
            FancyDuration(Duration::new(365 * 24 * 60 * 60)).to_string(),
            "1y"
        );

        assert_eq!(
            FancyDuration(Duration::new(365 * 24 * 60 * 60 + 10 * 24 * 60 * 60, 0)).to_string(),
            "1y 1w 3d"
        );

        assert_eq!(
            FancyDuration(Duration::new(365 * 24 * 60 * 60 + 10 * 24 * 60 * 60, 0))
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
                365 * 24 * 60 * 60 + 10 * 24 * 60 * 60,
                0
            ))
            .to_string(),
            "1y 1w 3d"
        );

        assert_eq!(
            FancyDuration(time::Duration::new(
                365 * 24 * 60 * 60 + 10 * 24 * 60 * 60,
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
            FancyDuration(chrono::TimeDelta::try_milliseconds(600).unwrap_or_default()).to_string(),
            "600ms"
        );
        assert_eq!(
            FancyDuration(chrono::TimeDelta::try_seconds(600).unwrap_or_default()).to_string(),
            "10m"
        );
        assert_eq!(
            FancyDuration(chrono::TimeDelta::try_seconds(120).unwrap_or_default()).to_string(),
            "2m"
        );
        assert_eq!(
            FancyDuration(chrono::TimeDelta::try_seconds(185).unwrap_or_default()).to_string(),
            "3m 5s"
        );
        assert_eq!(
            FancyDuration(chrono::TimeDelta::try_seconds(24 * 60 * 60).unwrap_or_default())
                .to_string(),
            "1d"
        );
        assert_eq!(
            FancyDuration(chrono::TimeDelta::try_seconds(324).unwrap_or_default()).to_string(),
            "5m 24s"
        );
        assert_eq!(
            FancyDuration(chrono::TimeDelta::try_seconds(24 * 60 * 60 + 324).unwrap_or_default())
                .to_string(),
            "1d 5m 24s"
        );
        assert_eq!(
            FancyDuration(
                chrono::TimeDelta::try_seconds(27 * 24 * 60 * 60 + 324).unwrap_or_default()
            )
            .to_string(),
            "3w 6d 5m 24s"
        );
        assert_eq!(
            FancyDuration(
                chrono::TimeDelta::try_seconds(99 * 24 * 60 * 60 + 324).unwrap_or_default()
            )
            .to_string(),
            "3m 1w 2d 5m 24s"
        );

        assert_eq!(
            FancyDuration(
                chrono::Duration::try_seconds(12 * 30 * 24 * 60 * 60 + 10 * 24 * 60 * 60,)
                    .unwrap_or_default()
            )
            .to_string(),
            "1y 1w 3d"
        );

        assert_eq!(
            FancyDuration(
                chrono::TimeDelta::try_seconds(12 * 30 * 24 * 60 * 60 + 10 * 24 * 60 * 60,)
                    .unwrap_or_default()
            )
            .format_compact(),
            "1y1w3d"
        );
        assert_eq!(
            FancyDuration(chrono::TimeDelta::try_seconds(24 * 60 * 60 + 324).unwrap_or_default())
                .format_compact(),
            "1d5m24s"
        );
        assert_eq!(
            FancyDuration(
                chrono::TimeDelta::try_seconds(27 * 24 * 60 * 60 + 324).unwrap_or_default()
            )
            .format_compact(),
            "3w6d5m24s"
        );
        assert_eq!(
            FancyDuration(
                chrono::TimeDelta::try_seconds(99 * 24 * 60 * 60 + 324).unwrap_or_default()
            )
            .format_compact(),
            "3m1w2d5m24s"
        );
    }

    #[test]
    fn test_parse_filter() {
        use super::DurationPart;
        let duration_table = [
            (
                "1m 5s 10ms",
                vec![DurationPart::Minutes, DurationPart::Milliseconds],
                "1m 10ms",
            ),
            (
                "1h 1m 30us",
                vec![DurationPart::Minutes, DurationPart::Microseconds],
                "1m 30us",
            ),
            ("1d 1h 30ns", vec![DurationPart::Days], "1d"),
            (
                "10s",
                vec![DurationPart::Seconds, DurationPart::Minutes],
                "10s",
            ),
            (
                "3m 5s",
                vec![
                    DurationPart::Hours,
                    DurationPart::Minutes,
                    DurationPart::Seconds,
                ],
                "3m 5s",
            ),
            (
                "3m 2w 2d 10m 10s",
                vec![
                    DurationPart::Months,
                    DurationPart::Weeks,
                    DurationPart::Days,
                ],
                "3m 2w 2d",
            ),
        ];

        for (orig_duration, filter, new_duration) in &duration_table {
            assert_eq!(
                *new_duration,
                FancyDuration::<Duration>::parse(orig_duration)
                    .unwrap()
                    .filter(&filter)
                    .to_string()
            )
        }

        #[cfg(feature = "time")]
        for (orig_duration, filter, new_duration) in &duration_table {
            assert_eq!(
                *new_duration,
                FancyDuration::<time::Duration>::parse(orig_duration)
                    .unwrap()
                    .filter(&filter)
                    .to_string()
            )
        }

        #[cfg(feature = "chrono")]
        for (orig_duration, filter, new_duration) in &duration_table {
            assert_eq!(
                *new_duration,
                FancyDuration::<chrono::Duration>::parse(orig_duration)
                    .unwrap()
                    .filter(&filter)
                    .to_string()
            )
        }
    }
    #[test]
    fn test_parse_truncate() {
        let duration_table = [
            ("1m 5s 10ms", 2, "1m 5s"),
            ("1h 1m 30us", 3, "1h 1m"),
            ("1d 1h 30ns", 1, "1d"),
            ("10s", 3, "10s"),
            ("3m 5s", 2, "3m 5s"),
            ("3m 2w 2d 10m 10s", 3, "3m 2w 2d"),
        ];

        for (orig_duration, truncate, new_duration) in &duration_table {
            assert_eq!(
                *new_duration,
                FancyDuration::<Duration>::parse(orig_duration)
                    .unwrap()
                    .truncate(*truncate)
                    .to_string()
            )
        }

        #[cfg(feature = "time")]
        for (orig_duration, truncate, new_duration) in &duration_table {
            assert_eq!(
                *new_duration,
                FancyDuration::<time::Duration>::parse(orig_duration)
                    .unwrap()
                    .truncate(*truncate)
                    .to_string()
            )
        }

        #[cfg(feature = "chrono")]
        for (orig_duration, truncate, new_duration) in &duration_table {
            assert_eq!(
                *new_duration,
                FancyDuration::<chrono::Duration>::parse(orig_duration)
                    .unwrap()
                    .truncate(*truncate)
                    .to_string()
            )
        }
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
                    chrono::TimeDelta::try_seconds(60).unwrap_or_default()
                        + chrono::TimeDelta::try_milliseconds(10).unwrap_or_default(),
                ),
                (
                    "1h 30us",
                    chrono::TimeDelta::try_hours(1).unwrap_or_default()
                        + chrono::Duration::microseconds(30),
                ),
                (
                    "1d 30ns",
                    chrono::TimeDelta::try_days(1).unwrap_or_default()
                        + chrono::Duration::nanoseconds(30),
                ),
                (
                    "10s",
                    chrono::TimeDelta::try_seconds(10).unwrap_or_default(),
                ),
                (
                    "3m 5s",
                    chrono::TimeDelta::try_seconds(185).unwrap_or_default(),
                ),
                (
                    "3m 2w 2d 10m 10s",
                    chrono::TimeDelta::try_seconds(9159010).unwrap_or_default(),
                ),
            ];

            let compact_chrono_table = [
                (
                    "3m5s",
                    chrono::TimeDelta::try_seconds(185).unwrap_or_default(),
                ),
                (
                    "3m2w2d10m10s",
                    chrono::TimeDelta::try_seconds(9159010).unwrap_or_default(),
                ),
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

    #[cfg(feature = "serde")]
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
                (
                    "{\"duration\":\"10s\"}",
                    chrono::TimeDelta::try_seconds(10).unwrap_or_default(),
                ),
                (
                    "{\"duration\":\"3m 5s\"}",
                    chrono::TimeDelta::try_seconds(185).unwrap_or_default(),
                ),
                (
                    "{\"duration\":\"1y 3m 2w 2d 10m 10s\"}",
                    chrono::TimeDelta::try_seconds(40263010).unwrap_or_default(),
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
