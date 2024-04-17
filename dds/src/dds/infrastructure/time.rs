use std::ops::Sub;

use crate::serialized_payload::cdr::{
    deserialize::CdrDeserialize, deserializer::CdrDeserializer, serialize::CdrSerialize,
    serializer::CdrSerializer,
};

/// Enumeration representing whether a duration is finite or infinite
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DurationKind {
    /// Finite duration with the corresponding associated value
    Finite(Duration),
    /// Infinite duration
    Infinite,
}

const DURATION_INFINITE_SEC: i32 = 0x7fffffff;
const DURATION_INFINITE_NSEC: u32 = 0xffffffff;
const DURATION_INFINITE: Duration = Duration {
    sec: DURATION_INFINITE_SEC,
    nanosec: DURATION_INFINITE_NSEC,
};

impl CdrSerialize for DurationKind {
    fn serialize(&self, serializer: &mut impl CdrSerializer) -> Result<(), std::io::Error> {
        match self {
            DurationKind::Finite(d) => d,
            DurationKind::Infinite => &DURATION_INFINITE,
        }
        .serialize(serializer)
    }
}

impl<'de> CdrDeserialize<'de> for DurationKind {
    fn deserialize(deserializer: &mut impl CdrDeserializer<'de>) -> Result<Self, std::io::Error> {
        let duration: Duration = CdrDeserialize::deserialize(deserializer)?;
        if duration == DURATION_INFINITE {
            Ok(DurationKind::Infinite)
        } else {
            Ok(DurationKind::Finite(duration))
        }
    }
}

impl PartialOrd<DurationKind> for DurationKind {
    fn partial_cmp(&self, other: &DurationKind) -> Option<std::cmp::Ordering> {
        match self {
            DurationKind::Finite(this) => match other {
                DurationKind::Finite(v) => Duration::partial_cmp(this, v),
                DurationKind::Infinite => Some(std::cmp::Ordering::Less),
            },
            DurationKind::Infinite => match other {
                DurationKind::Finite(_) => Some(std::cmp::Ordering::Greater),
                DurationKind::Infinite => Some(std::cmp::Ordering::Equal),
            },
        }
    }
}

/// Structure representing a time interval with a nanosecond resolution.
#[derive(PartialOrd, PartialEq, Eq, Debug, Clone, Copy, CdrSerialize, CdrDeserialize)]
pub struct Duration {
    sec: i32,
    nanosec: u32,
}

impl Duration {
    /// Construct a new [`Duration`] with the corresponding seconds and nanoseconds.
    pub const fn new(sec: i32, nanosec: u32) -> Self {
        let sec = sec + (nanosec / 1_000_000_000) as i32;
        let nanosec = nanosec % 1_000_000_000;
        Self { sec, nanosec }
    }

    /// Get a reference to the duration's sec.
    pub fn sec(&self) -> i32 {
        self.sec
    }

    /// Get a reference to the duration's nanosec.
    pub fn nanosec(&self) -> u32 {
        self.nanosec
    }
}

impl std::ops::Sub<Duration> for Duration {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Self::Output {
        let mut sec = self.sec - rhs.sec;
        let nanosec_diff = (self.nanosec as i64) - (rhs.nanosec as i64);
        let nanosec = if nanosec_diff < 0 {
            sec -= 1;
            (1_000_000_000 + nanosec_diff) as u32
        } else {
            self.nanosec - rhs.nanosec
        };
        Self { sec, nanosec }
    }
}

impl From<crate::implementation::rtps::behavior_types::Duration> for Duration {
    fn from(value: crate::implementation::rtps::behavior_types::Duration) -> Self {
        let sec = value.seconds();
        let nanosec = (value.fraction() as f64 / 2f64.powf(32.0) * 1_000_000_000.0).round() as u32;
        Self { sec, nanosec }
    }
}

impl From<Duration> for crate::implementation::rtps::behavior_types::Duration {
    fn from(value: Duration) -> Self {
        let seconds = value.sec;
        let fraction = (value.nanosec as f64 / 1_000_000_000.0 * 2f64.powf(32.0)).round() as u32;
        crate::implementation::rtps::behavior_types::Duration::new(seconds, fraction)
    }
}

impl From<std::time::Duration> for Duration {
    fn from(x: std::time::Duration) -> Self {
        Self::new(x.as_secs() as i32, x.subsec_nanos())
    }
}

impl From<Duration> for std::time::Duration {
    fn from(x: Duration) -> Self {
        std::time::Duration::new(x.sec as u64, x.nanosec)
    }
}

/// Structure representing a time in Unix Epoch with a nanosecond resolution.
#[derive(Clone, PartialEq, Debug, Copy, PartialOrd, Eq, Ord)]
pub struct Time {
    sec: i32,
    nanosec: u32,
}

impl Time {
    /// Create a new [`Time`] with a number of seconds and nanoseconds
    pub const fn new(sec: i32, nanosec: u32) -> Self {
        let sec = sec + (nanosec / 1_000_000_000) as i32;
        let nanosec = nanosec % 1_000_000_000;
        Self { sec, nanosec }
    }

    /// Get the number of seconds contained by this time
    pub const fn sec(&self) -> i32 {
        self.sec
    }

    /// Get the number of nanoseconds contained by this time
    pub const fn nanosec(&self) -> u32 {
        self.nanosec
    }
}

impl Sub<Time> for Time {
    type Output = Duration;

    fn sub(self, rhs: Time) -> Self::Output {
        let lhs = Duration::new(self.sec, self.nanosec);
        let rhs = Duration::new(rhs.sec, rhs.nanosec);
        lhs - rhs
    }
}

/// Pre-defined value representing a zero duration seconds
pub const DURATION_ZERO_SEC: i32 = 0;
/// Pre-defined value representing a zero duration nano seconds
pub const DURATION_ZERO_NSEC: u32 = 0;

/// Pre-defined value representing an invalid time
pub const TIME_INVALID_SEC: i32 = -1;
/// v value representing an invalid time nano seconds
pub const TIME_INVALID_NSEC: u32 = 0xffffffff;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dds_duration_to_rtps_duration() {
        let dds_duration = Duration::new(13, 500_000_000);

        let rtps_duration =
            crate::implementation::rtps::behavior_types::Duration::from(dds_duration);

        assert_eq!(rtps_duration.seconds(), 13);
        assert_eq!(rtps_duration.fraction(), 2u32.pow(31));
    }

    #[test]
    fn rtps_duration_to_dds_duration() {
        let rtps_duration =
            crate::implementation::rtps::behavior_types::Duration::new(13, 2u32.pow(31));

        let dds_duration = Duration::from(rtps_duration);

        assert_eq!(dds_duration.sec, 13);
        assert_eq!(dds_duration.nanosec, 500_000_000);
    }

    #[test]
    fn dds_duration_to_rtps_duration_to_dds_duration() {
        let dds_time = Duration::new(13, 200);

        let rtps_time = crate::implementation::rtps::behavior_types::Duration::from(dds_time);
        let dds_time_from_rtps_time = Duration::from(rtps_time);

        assert_eq!(dds_time, dds_time_from_rtps_time)
    }
}
