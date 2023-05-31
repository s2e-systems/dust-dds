use std::io::Read;

use crate::implementation::rtps_udp_psm::mapping_rtps_messages::submessages::submessage_header::{
    ACKNACK, DATA, DATA_FRAG, GAP, HEARTBEAT, HEARTBEAT_FRAG, INFO_DST, INFO_REPLY, INFO_SRC,
    INFO_TS, NACK_FRAG, PAD,
};

use super::overall_structure::IntoBytes;

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

impl IntoBytes for ProtocolId {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        b"RTPS".as_slice().read(buf).unwrap()
    }
}

/// SubmessageFlag
/// Type used to specify a Submessage flag.
/// A Submessage flag takes a boolean value and affects the parsing of the Submessage by the receiver.
pub type SubmessageFlag = bool;

impl IntoBytes for [SubmessageFlag; 8] {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        let mut flags = 0b_0000_0000_u8;
        for (i, &item) in self.iter().enumerate() {
            if item {
                flags |= 0b_0000_0001 << i
            }
        }
        buf[0] = flags;
        1
    }
}

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
}

impl IntoBytes for SubmessageKind {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        buf[0] = match self {
            SubmessageKind::DATA => DATA,
            SubmessageKind::GAP => GAP,
            SubmessageKind::HEARTBEAT => HEARTBEAT,
            SubmessageKind::ACKNACK => ACKNACK,
            SubmessageKind::PAD => PAD,
            SubmessageKind::INFO_TS => INFO_TS,
            SubmessageKind::INFO_REPLY => INFO_REPLY,
            SubmessageKind::INFO_DST => INFO_DST,
            SubmessageKind::INFO_SRC => INFO_SRC,
            SubmessageKind::DATA_FRAG => DATA_FRAG,
            SubmessageKind::NACK_FRAG => NACK_FRAG,
            SubmessageKind::HEARTBEAT_FRAG => HEARTBEAT_FRAG,
        };
        1
    }
}

/// ParameterId_t
/// Type used to uniquely identify a parameter in a parameter list.
/// Used extensively by the Discovery Module mainly to define QoS Parameters. A range of values is reserved for protocol-defined parameters, while another range can be used for vendor-defined parameters, see 8.3.5.9.
#[derive(Clone, Copy, PartialEq, Eq, Debug, derive_more::Into)]
pub struct ParameterId(pub u16);

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    derive_more::Into,
    derive_more::Sub,
    derive_more::SubAssign,
    derive_more::Add,
    derive_more::AddAssign,
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

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    derive_more::Into,
    derive_more::Sub,
    derive_more::SubAssign,
    derive_more::Add,
    derive_more::AddAssign,
)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Time {
    seconds: i32,
    fraction: u32,
}

impl Time {
    pub const fn new(seconds: i32, fraction: u32) -> Self {
        Self { seconds, fraction }
    }

    pub fn seconds(&self) -> i32 {
        self.seconds
    }

    pub fn fraction(&self) -> u32 {
        self.fraction
    }
}

pub const TIME_INVALID: Time = Time {
    seconds: 0xffff,
    fraction: 0xffff,
};

#[derive(Debug, PartialEq, Eq, derive_more::Into, derive_more::From)]
pub struct SerializedPayload<'a>(&'a [u8]);

impl<'a> SerializedPayload<'a> {
    pub fn new(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl IntoBytes for SerializedPayload<'_> {
    fn into_bytes<E: byteorder::ByteOrder>(&self, buf: &mut [u8]) -> usize {
        buf[..self.0.len()].copy_from_slice(self.0);
        self.0.len()
    }
}

impl<'a> From<&'_ SerializedPayload<'a>> for &'a [u8] {
    fn from(value: &'_ SerializedPayload<'a>) -> Self {
        value.0
    }
}
