///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.46 - Types definitions for the Behavior Module
///


#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Duration(pub i64);

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
