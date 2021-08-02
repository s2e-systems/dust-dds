///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.46 - Types definitions for the Behavior Module
///

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

impl Duration {
    pub fn new(seconds: i32, fraction: u32) -> Self { Self { seconds, fraction } }
    pub fn seconds(&self) -> &i32 {
        &self.seconds
    }
    pub fn fraction(&self) -> &u32 {
        &self.fraction
    }
}

pub const DURATION_ZERO: Duration = Duration {
    seconds: 0,
    fraction: 0,
};

pub const DURATION_INFINITE: Duration = Duration {
    seconds: 0x7fffffff,
    fraction: 0xffffffff,
};

pub type ParticipantMessageData = ();

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChangeForReaderStatusKind {
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChangeFromWriterStatusKind {
    Lost,
    Missing,
    Received,
    Unknown,
}
