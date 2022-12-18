/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///

/// ProtocolId_t
/// Enumeration used to identify the protocol.
/// The following values are reserved by the protocol: PROTOCOL_RTPS
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
pub enum ProtocolId {
    PROTOCOL_RTPS,
}

/// SubmessageFlag
/// Type used to specify a Submessage flag.
/// A Submessage flag takes a boolean value and affects the parsing of the Submessage by the receiver.
pub type SubmessageFlag = bool;

/// SubmessageKind
/// Enumeration used to identify the kind of Submessage.
/// The following values are reserved by this version of the protocol:
/// DATA, GAP, HEARTBEAT, ACKNACK, PAD, INFO_TS, INFO_REPLY, INFO_DST, INFO_SRC, DATA_FRAG, NACK_FRAG, HEARTBEAT_FRAG
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
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
    UNKNOWN,
}

/// ParameterId_t
/// Type used to uniquely identify a parameter in a parameter list.
/// Used extensively by the Discovery Module mainly to define QoS Parameters. A range of values is reserved for protocol-defined parameters, while another range can be used for vendor-defined parameters, see 8.3.5.9.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParameterId(pub u16);

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, derive_more::Into, derive_more::Sub, derive_more::Add,
)]
pub struct FragmentNumber(u32);

impl FragmentNumber {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, derive_more::Into)]
pub struct GroupDigest([u8; 4]);

impl GroupDigest {
    pub const fn new(value: [u8; 4]) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Into)]
pub struct UShort(u16);

impl UShort {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, derive_more::Into)]
pub struct ULong(u32);

impl ULong {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}
