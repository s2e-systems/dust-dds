use super::submessages::submessage_elements::Parameter;
use serde::Serialize;

/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///

pub mod constants {
    use super::ProtocolId;
    use super::Time;

    pub const TIME_ZERO: Time = Time {
        seconds: 0,
        fraction: 0,
    };
    pub const TIME_INFINITE: Time = Time {
        seconds: core::u32::MAX,
        fraction: core::u32::MAX - 1,
    };
    pub const TIME_INVALID: Time = Time {
        seconds: core::u32::MAX,
        fraction: core::u32::MAX,
    };
    pub const PROTOCOL_RTPS: ProtocolId = [b'R', b'T', b'P', b'S'];

    pub const SUBMESSAGE_KIND_PAD: u8 = 0x01;
    pub const SUBMESSAGE_KIND_ACK_NACK: u8 = 0x06;
    pub const SUBMESSAGE_KIND_HEARTBEAT: u8 = 0x07;
    pub const SUBMESSAGE_KIND_GAP: u8 = 0x08;
    pub const SUBMESSAGE_KIND_INFO_TIMESTAMP: u8 = 0x09;
    pub const SUBMESSAGE_KIND_INFO_SOURCE: u8 = 0x0c;
    pub const SUBMESSAGE_KIND_INFO_REPLY_IP4: u8 = 0x0d;
    pub const SUBMESSAGE_KIND_INFO_DESTINATION: u8 = 0x0e;
    pub const SUBMESSAGE_KIND_INFO_REPLY: u8 = 0x0f;
    pub const SUBMESSAGE_KIND_NACK_FRAG: u8 = 0x12;
    pub const SUBMESSAGE_KIND_HEARTBEAT_FRAG: u8 = 0x13;
    pub const SUBMESSAGE_KIND_DATA: u8 = 0x15;
    pub const SUBMESSAGE_KIND_DATA_FRAG: u8 = 0x16;
}

pub type ProtocolId = [u8; 4];

pub type SubmessageFlag = bool;

pub type SubmessageKind = u8;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Serialize)]
pub struct Time {
    seconds: u32,
    fraction: u32,
}
impl Time {
    pub fn new(seconds: u32, fraction: u32) -> Self {
        Self { seconds, fraction }
    }
    pub fn seconds(&self) -> u32 {
        self.seconds
    }
    pub fn fraction(&self) -> u32 {
        self.fraction
    }
}

pub type Count = i32;

pub type FragmentNumber = u32;

pub type ParameterId = i16;

// /////////// KeyHash and StatusInfo /////////////

pub const PID_KEY_HASH: ParameterId = 0x0070;
pub const PID_STATUS_INFO: ParameterId = 0x0071;
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct KeyHash(pub [u8; 16]);

impl Parameter for KeyHash {
    fn parameter_id(&self) -> ParameterId {
        todo!()
    }

    fn length(&self) -> i16 {
        todo!()
    }

    fn value(&self) -> &[u8] {
        todo!()
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct StatusInfo(pub [u8; 4]);

impl StatusInfo {
    pub const DISPOSED_FLAG_MASK: u8 = 0b0000_0001;
    pub const UNREGISTERED_FLAG_MASK: u8 = 0b0000_0010;
    pub const FILTERED_FLAG_MASK: u8 = 0b0000_0100;

    pub fn disposed_flag(&self) -> bool {
        self.0[3] & StatusInfo::DISPOSED_FLAG_MASK == StatusInfo::DISPOSED_FLAG_MASK
    }

    pub fn unregistered_flag(&self) -> bool {
        self.0[3] & StatusInfo::UNREGISTERED_FLAG_MASK == StatusInfo::UNREGISTERED_FLAG_MASK
    }

    pub fn filtered_flag(&self) -> bool {
        self.0[3] & StatusInfo::FILTERED_FLAG_MASK == StatusInfo::FILTERED_FLAG_MASK
    }
}
