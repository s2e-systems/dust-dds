/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.13 - Types used to define RTPS messages
///  
use std::time::SystemTime;
use num_derive::{FromPrimitive, };

pub mod constants {
    use super::Time;
    use super::ProtocolId;

    const TIME_ZERO: Time = Time {
        seconds: 0,
        fraction: 0,
    };
    const TIME_INFINITE: Time = Time {
        seconds: std::u32::MAX,
        fraction: std::u32::MAX - 1,
    };
    const TIME_INVALID: Time = Time {
        seconds: std::u32::MAX,
        fraction: std::u32::MAX,
    };    
    pub const PROTOCOL_RTPS: ProtocolId = [b'R', b'T', b'P', b'S'];
}

pub type ProtocolId = [u8; 4];

pub type SubmessageFlag = bool;

#[derive(FromPrimitive, PartialEq, Copy, Clone, Debug)]
pub enum SubmessageKind {
    Pad = 0x01,
    AckNack = 0x06,
    Heartbeat = 0x07,
    Gap = 0x08,
    InfoTimestamp = 0x09,
    InfoSource = 0x0c,
    InfoReplyIP4 = 0x0d,
    InfoDestination = 0x0e,
    InfoReply = 0x0f,
    NackFrag = 0x12,
    HeartbeatFrag = 0x13,
    Data = 0x15,
    DataFrag = 0x16,
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

pub type ParameterId = i16;

pub type FragmentNumber = u32;

// /////////// GroupDigest_t /////////
//  todo

