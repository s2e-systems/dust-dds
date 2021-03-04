///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.46 - Types definitions for the Behavior Module
///
pub mod constants {
    use super::Duration;

    pub const DURATION_ZERO: Duration = Duration {
        seconds: 0,
        fraction: 0,
    };

    pub const DURATION_INFINITE: Duration = Duration {
        seconds: 0x7fffffff,
        fraction: 0xffffffff,
    };
}

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone, Copy)]
pub struct Duration {
    seconds: i32,
    fraction: u32,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum ChangeForReaderStatusKind {
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}

#[derive(Eq, PartialEq, Clone)]
pub enum ChangeFromWriterStatusKind {
    Lost,
    Missing,
    Received,
    Unknown,
}

// InstanceHandle should already be defined in structre types

// todo: ParticipantMessageData
