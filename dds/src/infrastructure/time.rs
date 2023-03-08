use std::ops::Sub;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DurationKind {
    Finite(Duration),
    Infinite,
}

const DURATION_INFINITE_SEC: i32 = 0x7fffffff;
const DURATION_INFINITE_NSEC: u32 = 0x7fffffff;

impl serde::Serialize for DurationKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde::Serialize::serialize(
            &match self {
                DurationKind::Finite(d) => &d,
                DurationKind::Infinite => &Duration {
                    sec: DURATION_INFINITE_SEC,
                    nanosec: DURATION_INFINITE_NSEC,
                },
            },
            serializer,
        )
    }
}

struct DurationKindVisitor;

impl<'de> serde::de::Visitor<'de> for DurationKindVisitor {
    type Value = DurationKind;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("DurationKind")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: serde::de::SeqAccess<'de>,
    {
        let sec = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let nanosec = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        Ok(
            if sec == DURATION_INFINITE_SEC && nanosec == DURATION_INFINITE_NSEC {
                DurationKind::Infinite
            } else {
                DurationKind::Finite(Duration::new(sec, nanosec))
            },
        )
    }
}

impl<'de> serde::Deserialize<'de> for DurationKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, DurationKindVisitor)
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

#[derive(PartialOrd, PartialEq, Eq, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_subtraction() {
        let duration = Time { sec: 2, nanosec: 0 }
            - Time {
                sec: 1,
                nanosec: 900_000_000,
            };
        assert_eq!(
            duration,
            Duration {
                sec: 0,
                nanosec: 100_000_000
            }
        );
    }

    #[test]
    fn time_subtraction_nanosec_bigger_one_second() {
        let duration = Time {
            sec: 4,
            nanosec: 700_000_000,
        } - Time {
            sec: 1,
            nanosec: 2_900_000_000,
        };
        assert_eq!(
            duration,
            Duration {
                sec: 0,
                nanosec: 800_000_000
            }
        );
    }

    #[test]
    fn time_subtraction_nanosec_bigger_one_second_lhs_and_rhs() {
        let duration = Time {
            sec: 0,
            nanosec: 3_700_000_000,
        } - Time {
            sec: 0,
            nanosec: 2_900_000_000,
        };
        assert_eq!(
            duration,
            Duration {
                sec: 0,
                nanosec: 800_000_000
            }
        );
    }

    #[test]
    fn duration_kind_partial_ord() {
        assert!(DurationKind::Infinite == DurationKind::Infinite);
        assert!(DurationKind::Infinite > DurationKind::Finite(DURATION_ZERO));
        assert!(DurationKind::Finite(DURATION_ZERO) < DurationKind::Infinite);
    }

    #[test]
    fn duration_serialize_deserialize() {
        let duration = DurationKind::Infinite;
        let serialized = cdr::serialize::<_, _, cdr::CdrLe>(&duration, cdr::Infinite).unwrap();
        println!("Data: {:?}", serialized);
        assert_eq!(
            cdr::deserialize::<DurationKind>(&serialized).unwrap(),
            duration
        );
    }
}
