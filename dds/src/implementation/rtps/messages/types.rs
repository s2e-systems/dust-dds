use byteorder::ByteOrder;

use super::overall_structure::WriteBytes;
use std::io::Read;

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

impl WriteBytes for ProtocolId {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        b"RTPS".as_slice().read(buf).unwrap()
    }
}

/// SubmessageFlag
/// Type used to specify a Submessage flag.
/// A Submessage flag takes a boolean value and affects the parsing of the Submessage by the receiver.
pub type SubmessageFlag = bool;

impl WriteBytes for [SubmessageFlag; 8] {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
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

pub const DATA: u8 = 0x15;
pub const GAP: u8 = 0x08;
pub const HEARTBEAT: u8 = 0x07;
pub const ACKNACK: u8 = 0x06;
pub const PAD: u8 = 0x01;
pub const INFO_TS: u8 = 0x09;
pub const INFO_REPLY: u8 = 0x0f;
pub const INFO_DST: u8 = 0x0e;
pub const INFO_SRC: u8 = 0x0c;
pub const DATA_FRAG: u8 = 0x16;
pub const NACK_FRAG: u8 = 0x12;
pub const HEARTBEAT_FRAG: u8 = 0x13;

impl WriteBytes for SubmessageKind {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
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

/// Time_t
/// Type used to hold a timestamp.
/// Should have at least nano-second resolution.
/// The following values are reserved by the protocol: TIME_ZERO
/// TIME_INVALID TIME_INFINITE

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

impl WriteBytes for Time {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.seconds.write_bytes(&mut buf[0..]) + self.fraction.write_bytes(&mut buf[4..])
    }
}

pub const TIME_INVALID: Time = Time {
    seconds: 0xffff,
    fraction: 0xffff,
};

/// Count_t
/// Type used to hold a count that is incremented monotonically, used to identify message duplicates.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Default,
    derive_more::Into,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
)]
pub struct Count(i32);

impl Count {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }
    pub const fn wrapping_add(self, rhs: i32) -> Self {
        Self(self.0.wrapping_add(rhs))
    }
}
impl PartialOrd<Count> for Count {
    fn partial_cmp(&self, other: &Count) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl WriteBytes for Count {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        byteorder::LittleEndian::write_i32(buf, self.0);
        4
    }
}

/// ParameterId_t
/// Type used to uniquely identify a parameter in a parameter list.
/// Used extensively by the Discovery Module mainly to define QoS Parameters. A range of values is reserved for protocol-defined parameters, while another range can be used for vendor-defined parameters, see 8.3.5.9.
#[derive(Clone, Copy, PartialEq, Eq, Debug, derive_more::Into)]
pub struct ParameterId(pub u16);

/// FragmentNumber_t
/// Type used to hold fragment numbers.
/// Must be possible to represent using 32 bits.
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

impl WriteBytes for FragmentNumber {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        byteorder::LittleEndian::write_u32(buf, self.0);
        4
    }
}

/// GroupDigest_t
/// Type used to hold a digest value that uniquely identifies a group of Entities belonging to the same Participant.
#[derive(Clone, Copy, PartialEq, Eq, Debug, derive_more::Into)]
pub struct GroupDigest([u8; 4]);

impl GroupDigest {
    pub const fn new(value: [u8; 4]) -> Self {
        Self(value)
    }
}
