/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///

pub trait ProtocolIdPIM {
    type ProtocolIdType;
    const PROTOCOL_RTPS: Self::ProtocolIdType;
}

pub type SubmessageFlag = bool;

pub trait SubmessageKindPIM {
    type SubmessageKindType;
    const DATA: Self::SubmessageKindType;
    const GAP: Self::SubmessageKindType;
    const HEARTBEAT: Self::SubmessageKindType;
    const ACKNACK: Self::SubmessageKindType;
    const PAD: Self::SubmessageKindType;
    const INFO_TS: Self::SubmessageKindType;
    const INFO_REPLY: Self::SubmessageKindType;
    const INFO_DST: Self::SubmessageKindType;
    const INFO_SRC: Self::SubmessageKindType;
    const DATA_FRAG: Self::SubmessageKindType;
    const NACK_FRAG: Self::SubmessageKindType;
    const HEARTBEAT_FRAG: Self::SubmessageKindType;
}

pub trait TimePIM {
    type TimeType;
    const TIME_ZERO: Self::TimeType;
    const TIME_INVALID: Self::TimeType;
    const TIME_INFINITE: Self::TimeType;
}

pub trait CountPIM {
    type CountType;
}

pub trait ParameterIdPIM {
    type ParameterIdType;
}

pub type FragmentNumber = u32;

pub trait GroupDigestPIM {
    type GroupDigestType;
}
