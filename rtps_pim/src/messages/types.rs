/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum ProtocolId {
    PROTOCOL_RTPS,
}

pub type SubmessageFlag = bool;

#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(non_camel_case_types)]
pub enum SubmessageKind {
    DATA,
    GAP,
    HEARTBEAT,
    ACKNACK,
    PAD,
    INFO_TS,
    INFO_REPLY,
    INFO_DST,
    INFO_SRC,
    DATA_FRAG,
    NACK_FRAG,
    HEARTBEAT_FRAG,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Time(pub u64);

pub const TIME_ZERO: Time = Time(0);
pub const TIME_INVALID: Time = Time(u64::MAX);
pub const TIME_INFINITE: Time = Time(u64::MAX - 1);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Count(pub i32);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ParameterId(pub u16);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FragmentNumber(pub u32);

pub type GroupDigest = ();
