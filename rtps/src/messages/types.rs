/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///  
use std::time::SystemTime;
use std::convert::TryFrom;

use rust_dds_interface::types::{ParameterId, Parameter};

pub mod constants {
    use super::Time;
    use super::ProtocolId;

    pub const TIME_ZERO: Time = Time {
        seconds: 0,
        fraction: 0,
    };
    pub const TIME_INFINITE: Time = Time {
        seconds: std::u32::MAX,
        fraction: std::u32::MAX - 1,
    };
    pub const TIME_INVALID: Time = Time {
        seconds: std::u32::MAX,
        fraction: std::u32::MAX,
    };    
    pub const PROTOCOL_RTPS: ProtocolId = [b'R', b'T', b'P', b'S'];
}

pub type ProtocolId = [u8; 4];

pub type SubmessageFlag = bool;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum SubmessageKind {
    Pad,
    AckNack,
    Heartbeat,
    Gap,
    InfoTimestamp,
    InfoSource,
    InfoReplyIP4,
    InfoDestination,
    InfoReply,
    NackFrag,
    HeartbeatFrag,
    Data,
    DataFrag,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct Time {
    seconds: u32,
    fraction: u32,
}
impl Time {
    pub fn new (seconds: u32, fraction: u32) -> Self { Self {seconds, fraction, } }
    pub fn seconds(&self) -> u32 {self.seconds}
    pub fn fraction(&self) -> u32 {self.fraction}
    pub fn now() -> Self {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        Time{seconds: current_time.as_secs() as u32 , fraction: current_time.subsec_nanos() as u32}
    }
}

pub type Count = i32;

pub type FragmentNumber = u32;

// /////////// GroupDigest_t /////////
//  todo



const PID_KEY_HASH : ParameterId = 0x0070;
const PID_STATUS_INFO : ParameterId = 0x0071;
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct KeyHash(pub [u8; 16]);

impl From<KeyHash> for Parameter {
    fn from(input: KeyHash) -> Self {
        Parameter::new(PID_KEY_HASH, input.0)
    }
}

impl TryFrom<Parameter> for KeyHash {
    type Error = ();
    fn try_from(parameter: Parameter) -> Result<Self, Self::Error> {
        if parameter.parameter_id() == PID_KEY_HASH {
            Ok(KeyHash(parameter.value()))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct StatusInfo(pub [u8;4]);

impl StatusInfo {
    pub const DISPOSED_FLAG_MASK : u8 = 0b0000_0001;
    pub const UNREGISTERED_FLAG_MASK : u8 = 0b0000_0010;
    pub const FILTERED_FLAG_MASK : u8 = 0b0000_0100;

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

impl From<StatusInfo> for Parameter {
    fn from(input: StatusInfo) -> Self {
        Parameter::new(PID_STATUS_INFO, input.0)
    }
}

impl TryFrom<Parameter> for StatusInfo {
    type Error = ();
    fn try_from(parameter: Parameter) -> Result<Self, Self::Error> {
        if parameter.parameter_id() == PID_STATUS_INFO {
            Ok(StatusInfo(parameter.value()))
        } else {
            Err(())
        }
    }
}