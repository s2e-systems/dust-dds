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
        let rhs_sec = rhs.sec + (rhs.nanosec / 1_000_000_000)  as i32;
        let rhs_nanosec = rhs.nanosec % 1_000_000_000;
        let lhs_sec = self.sec + (self.nanosec / 1_000_000_000)  as i32;
        let lhs_nanosec = self.nanosec % 1_000_000_000;
        if rhs_nanosec > lhs_nanosec {
            Duration {
                sec: lhs_sec - rhs_sec - 1,
                nanosec: lhs_nanosec + 1_000_000_000 - rhs_nanosec,
            }
        } else {
            Duration {
                sec: lhs_sec - rhs_sec,
                nanosec: lhs_nanosec - rhs_nanosec,
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
}
