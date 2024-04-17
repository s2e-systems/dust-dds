use super::types::{Long, UnsignedLong};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.5
/// Table 8.52 - Types definitions for the Behavior Module
///

/// Special constant value representing a zero duration
pub const DURATION_ZERO: Duration = Duration {
    seconds: 0,
    fraction: 0,
};

#[derive(Clone, Copy)]
pub struct Duration {
    seconds: Long,
    fraction: UnsignedLong,
}

impl Duration {
    pub fn new(seconds: Long, fraction: UnsignedLong) -> Self {
        Self { seconds, fraction }
    }

    pub fn seconds(&self) -> i32 {
        self.seconds
    }

    pub fn fraction(&self) -> u32 {
        self.fraction
    }
}

impl From<Duration> for std::time::Duration {
    fn from(value: Duration) -> Self {
        let secs = value.seconds as u64;
        let nanos = (value.fraction as f64 / 2f64.powf(32.0) * 1_000_000_000.0).round() as u32;
        std::time::Duration::new(secs, nanos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtps_duration_to_std_duration() {
        let rtps_duration = Duration::new(13, 2u32.pow(31));

        let std_duration = std::time::Duration::from(rtps_duration);

        assert_eq!(std_duration.as_secs(), 13);
        assert_eq!(std_duration.subsec_nanos(), 500_000_000);
    }
}
