use std::ops::Sub;

#[derive(PartialOrd, PartialEq, Eq, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Duration {
    sec: i32,
    nanosec: u32,
}

impl Duration {
    pub const fn new(sec: i32, nanosec: u32) -> Self {
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
        if rhs.nanosec > self.nanosec {
            Duration {
                sec: self.sec - rhs.sec - 1,
                nanosec: 1_000_000_000 - rhs.nanosec,
            }
        } else {
            Duration {
                sec: self.sec - rhs.sec,
                nanosec: self.nanosec - rhs.nanosec,
            }
        }
    }
}

/// Special constant value representing an infinite duration
pub const DURATION_INFINITE: Duration = Duration {
    sec: 0x7fffffff,
    nanosec: 0x7fffffff,
};

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
}
