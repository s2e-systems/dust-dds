/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///

pub trait Types {
    type ProtocolId: Copy;
    const PROTOCOL_RTPS: Self::ProtocolId;

    type SubmessageFlag: Into<bool> + From<bool> + Copy;

    type SubmessageKind: Copy;
    const DATA: Self::SubmessageKind;
    const GAP: Self::SubmessageKind;
    const HEARTBEAT: Self::SubmessageKind;
    const ACKNACK: Self::SubmessageKind;
    const PAD: Self::SubmessageKind;
    const INFO_TS: Self::SubmessageKind;
    const INFO_REPLY: Self::SubmessageKind;
    const INFO_DST: Self::SubmessageKind;
    const INFO_SRC: Self::SubmessageKind;
    const DATA_FRAG: Self::SubmessageKind;
    const NACK_FRAG: Self::SubmessageKind;
    const HEARTBEAT_FRAG: Self::SubmessageKind;

    type Time: Copy;
    const TIME_ZERO: Self::Time;
    const TIME_INVALID: Self::Time;
    const TIME_INFINITE: Self::Time;

    type Count: Copy;
    type ParameterId: Copy;
    type FragmentNumber: Copy;
    type GroupDigest: Copy;

    // Additions to represent lists which are used but not explicitly defined in the standard
    type FragmentNumberVector: IntoIterator<Item = Self::FragmentNumber>;
}
