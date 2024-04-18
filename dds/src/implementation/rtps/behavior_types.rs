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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceHandle(pub [u8; 16]);
