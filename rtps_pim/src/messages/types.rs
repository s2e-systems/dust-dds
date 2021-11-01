use core::ops::AddAssign;

/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///

/// ProtocolId_t
/// Enumeration used to identify the protocol.
/// The following values are reserved by the protocol: PROTOCOL_RTPS
#[derive(Clone, Copy, PartialEq, Debug)]
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
    UNKNOWN,
}

/// Time_t
/// Type used to hold a timestamp.
/// Should have at least nano-second resolution.
/// The following values are reserved by the protocol: TIME_ZERO
/// TIME_INVALID TIME_INFINITE
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Time(pub u64);

pub const TIME_ZERO: Time = Time(0);
pub const TIME_INVALID: Time = Time(u64::MAX);
pub const TIME_INFINITE: Time = Time(u64::MAX - 1);

/// Count_t
/// Type used to hold a count that is incremented monotonically, used to identify message duplicates.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Count(pub i32);

impl AddAssign for Count {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

/// ParameterId_t
/// Type used to uniquely identify a parameter in a parameter list.
/// Used extensively by the Discovery Module mainly to define QoS Parameters. A range of values is reserved for protocol-defined parameters, while another range can be used for vendor-defined parameters, see 8.3.5.9.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ParameterId(pub u16);

/// FragmentNumber_t
/// Type used to hold fragment numbers.
/// Must be possible to represent using 32 bits.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FragmentNumber(pub u32);

/// GroupDigest_t
/// Type used to hold a digest value that uniquely identifies a group of Entities belonging to the same Participant.
#[derive(Debug, PartialEq)]
pub struct GroupDigest(pub [u8; 4]);
