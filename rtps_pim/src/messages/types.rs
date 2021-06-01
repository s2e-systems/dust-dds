/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///

pub trait ProtocolIdType {
    type ProtocolId: Copy;
    const PROTOCOL_RTPS: Self::ProtocolId;
}

pub trait SubmessageFlagType {
    type SubmessageFlag: Into<bool> + From<bool> + Copy;
}

pub trait SubmessageKindType {
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
}

pub trait TimeType {
    type Time: Copy;
    const TIME_ZERO: Self::Time;
    const TIME_INVALID: Self::Time;
    const TIME_INFINITE: Self::Time;
}

pub trait CountType {
    type Count: Copy;
}

pub trait ParameterIdPIM {
    type ParameterId: Copy;
}

pub trait FragmentNumberType {
    type FragmentNumber: Copy;
}

pub trait GroupDigestType {
    type GroupDigest: Copy;
}
