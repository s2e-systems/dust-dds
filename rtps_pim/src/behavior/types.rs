///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.46 - Types definitions for the Behavior Module
///
pub trait Types {
    type Duration: Copy;

    type ChangeForReaderStatusKind: Copy;
    const UNSENT: Self::ChangeForReaderStatusKind;
    const UNACKNOWLEDGED: Self::ChangeForReaderStatusKind;
    const REQUESTED: Self::ChangeForReaderStatusKind;
    const ACKNOWLEDGED: Self::ChangeForReaderStatusKind;
    const UNDERWAY: Self::ChangeForReaderStatusKind;

    type ChangeFromWriterStatusKind: Copy;
    const LOST: Self::ChangeFromWriterStatusKind;
    const MISSING: Self::ChangeFromWriterStatusKind;
    const RECEIVED: Self::ChangeFromWriterStatusKind;
    const UNKNOWN: Self::ChangeFromWriterStatusKind;

    // type InstanceHandle;
    // This type has the same name as in structure Types so it is left out for now

    type ParticipantMessageData: Copy;
}
