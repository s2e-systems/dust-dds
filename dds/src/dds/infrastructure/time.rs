use std::ops::Sub;

use crate::cdr::{
    deserialize::CdrDeserialize, deserializer::CdrDeserializer, error::CdrResult,
    serialize::CdrSerialize, serializer::CdrSerializer,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DurationKind {
    Finite(Duration),
    Infinite,
}

const DURATION_INFINITE_SEC: i32 = 0x7fffffff;
const DURATION_INFINITE_NSEC: u32 = 0xffffffff;
const DURATION_INFINITE: Duration = Duration {
    sec: DURATION_INFINITE_SEC,
    nanosec: DURATION_INFINITE_NSEC,
};

impl CdrSerialize for DurationKind {
    fn serialize(&self, serializer: &mut CdrSerializer) -> CdrResult<()> {
        match self {
            DurationKind::Finite(d) => d,
            DurationKind::Infinite => &DURATION_INFINITE,
        }
        .serialize(serializer)
    }
}

impl<'de> CdrDeserialize<'de> for DurationKind {
    fn deserialize(deserializer: &mut CdrDeserializer<'de>) -> CdrResult<Self> {
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

#[derive(PartialOrd, PartialEq, Eq, Debug, Clone, Copy, CdrSerialize, CdrDeserialize)]
pub struct Duration {
    sec: i32,
    nanosec: u32,
}

impl Duration {
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

#[derive(Clone, PartialEq, Debug, Copy, PartialOrd, Eq, Ord)]
pub struct Time {
    sec: i32,
    nanosec: u32,
}

impl Time {
    pub const fn new(sec: i32, nanosec: u32) -> Self {
        let sec = sec + (nanosec / 1_000_000_000) as i32;
        let nanosec = nanosec % 1_000_000_000;
        Self { sec, nanosec }
    }

    pub const fn sec(&self) -> i32 {
        self.sec
    }

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

/// Special constant value representing a zero duration
pub const DURATION_ZERO: Duration = Duration { sec: 0, nanosec: 0 };

/// Special constant value representing an invalid time
pub const TIME_INVALID: Time = Time {
    sec: -1,
    nanosec: 0xffffffff,
};
