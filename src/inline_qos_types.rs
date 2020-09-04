/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// in the sub clauses of 9.6.3 ParameterId Definitions used to Represent In-line QoS
///  
 
use crate::serialized_payload::{Pid, ParameterId, };

use serde::{Serialize, Deserialize};

const PID_TOPIC_NAME : ParameterId = 0x0005;
const PID_KEY_HASH : ParameterId = 0x0070;
const PID_STATUS_INFO : ParameterId = 0x0071;
  
#[derive(Debug, PartialEq, Clone, Eq, Serialize, Deserialize)]
pub struct TopicName(pub Vec<u8>);
impl Pid for TopicName {
    fn pid() -> ParameterId {
        PID_TOPIC_NAME
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize)]
pub struct KeyHash(pub [u8; 16]);

impl Pid for KeyHash {
    fn pid() -> ParameterId {
        PID_KEY_HASH
    }
}


#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize)]
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

impl Pid for StatusInfo {
    fn pid() -> ParameterId {
        PID_STATUS_INFO
    }
}
