///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.4.6 - Types definitions for the Behavior Module
///


/// Duration_t
/// Type used to hold time differences.
/// Should have at least nano-second resolution.
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


/// ChangeForReaderStatusKind
/// Enumeration used to indicate the status of a ChangeForReader. It can take the values:
/// UNSENT, UNACKNOWLEDGED, REQUESTED, ACKNOWLEDGED, UNDERWAY
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChangeForReaderStatusKind {
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}


/// ChangeFromWriterStatusKind
/// Enumeration used to indicate the status of a ChangeFromWriter. It can take the values:
/// LOST, MISSING, RECEIVED, UNKNOWN
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChangeFromWriterStatusKind {
    Lost,
    Missing,
    Received,
    Unknown,
}


/// InstanceHandle_t
/// Already defined in Structure module Table 8.2


/// ParticipantMessageData
/// Type used to hold data exchanged between Participants. The most notable use of this type is for the Writer Liveliness Protocol.
pub type ParticipantMessageData = ();