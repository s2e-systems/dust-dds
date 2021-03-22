/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///

pub trait ProtocolId : Copy {
    const PROTOCOL_RTPS: Self;
}

pub trait SubmessageFlag: Into<bool> + From<bool> + Copy {}

pub trait SubmessageKind : Copy {
    const DATA: Self;
    const GAP: Self;
    const HEARTBEAT: Self;
    const ACKNACK: Self;
    const PAD: Self;
    const INFO_TS: Self;
    const INFO_REPLY: Self;
    const INFO_DST: Self;
    const INFO_SRC: Self;
    const DATA_FRAG: Self;
    const NACK_FRAG: Self;
    const HEARTBEAT_FRAG: Self;
}

pub trait Time : Copy {
    const TIME_ZERO: Self;
    const TIME_INVALID: Self;
    const TIME_INFINITE: Self;
}
pub trait Count : Copy {}

pub trait ParameterId : Copy {}

pub trait FragmentNumber : Copy {}

pub trait GroupDigest : Copy{}
