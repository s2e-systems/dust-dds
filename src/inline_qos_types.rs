/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// in the sub clauses of 9.6.3 ParameterId Definitions used to Represent In-line QoS
///  
 
use std::convert::{TryInto, From};
use crate::primitive_types::Short;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, Endianness, RtpsSerdesResult, SizeCheck};
use crate::types::{ChangeKind, };
use crate::messages::types::ParameterId;

const PID_TOPIC_NAME : ParameterId = 0x0005;
const PID_KEY_HASH : ParameterId = 0x0070;
const PID_STATUS_INFO : ParameterId = 0x0071;

pub trait InlineQosParameterId : std::fmt::Debug {
    fn id() -> ParameterId where Self: Sized;
}

pub trait InlineQosParameter : std::fmt::Debug{
    fn parameter_id(&self) -> ParameterId;

    fn length(&self) -> Short;

    fn value(&self, endianness: Endianness) -> Vec<u8>;
}

impl<T> InlineQosParameter for T
    where T: InlineQosParameterId + RtpsSerialize
{
    fn parameter_id(&self) -> ParameterId where Self: Sized {
        T::id()
    }

    fn length(&self) -> Short {
        // rounded up to multple of 4 (that is besides the length of the value may not be a multiple of 4)
        (self.octets() + 3 & !3) as Short
    }

    fn value(&self, endianness: Endianness) -> Vec<u8> {
        let mut writer = Vec::new();
        self.serialize(&mut writer, endianness).unwrap();
        writer
    }
}
  
#[derive(Debug, PartialEq, Clone, Eq)]
pub struct TopicName(pub Vec<u8>);
impl InlineQosParameterId for TopicName {
    fn id() -> ParameterId where Self: Sized {
        PID_TOPIC_NAME
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct KeyHash(pub [u8; 16]);

impl InlineQosParameterId for KeyHash {
    fn id() -> ParameterId where Self: Sized {
        PID_KEY_HASH
    }
}

impl RtpsSerialize for KeyHash {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianness: Endianness) -> RtpsSerdesResult<()> {
        writer.write(&self.0)?;
        Ok(())
    }
}

impl RtpsDeserialize for KeyHash {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
        bytes.check_size_bigger_equal_than(16)?;
        Ok(KeyHash(bytes[0..16].try_into()?))
    }
}


#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct StatusInfo(pub [u8;4]);

impl StatusInfo {
    const DISPOSED_FLAG_MASK : u8 = 0b0000_0001;
    const UNREGISTERED_FLAG_MASK : u8 = 0b0000_0010;
    const FILTERED_FLAG_MASK : u8 = 0b0000_0100;

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

impl InlineQosParameterId for StatusInfo {
    fn id() -> ParameterId
    where Self: Sized {
        PID_STATUS_INFO
    }
}

impl From<ChangeKind> for StatusInfo {
    fn from(change_kind: ChangeKind) -> Self {
        match change_kind {
            ChangeKind::Alive => StatusInfo([0,0,0,0]),
            ChangeKind::NotAliveDisposed => StatusInfo([0,0,0,StatusInfo::DISPOSED_FLAG_MASK]),
            ChangeKind::NotAliveUnregistered => StatusInfo([0,0,0,StatusInfo::UNREGISTERED_FLAG_MASK]),
            ChangeKind::AliveFiltered => StatusInfo([0,0,0,StatusInfo::FILTERED_FLAG_MASK]),
        }
    }
}

impl RtpsSerialize for StatusInfo {
    fn serialize(&self, writer: &mut impl std::io::Write, _endianess: Endianness) -> RtpsSerdesResult<()> {
        writer.write(&self.0)?;
        Ok(())
    }
}

impl RtpsDeserialize for StatusInfo {
    fn deserialize(bytes: &[u8], _endianness: Endianness) -> RtpsSerdesResult<Self> {
        bytes.check_size_bigger_equal_than(4)?;
        Ok(StatusInfo(bytes[0..4].try_into()?))
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;


    ///////////////////////// StatusInfo Tests ////////////////////////
    #[test]
    fn test_status_info_change_kind_conversions() {
        assert_eq!(ChangeKind::try_from(StatusInfo::from(ChangeKind::Alive)).unwrap(), ChangeKind::Alive);
        assert_eq!(ChangeKind::try_from(StatusInfo::from(ChangeKind::AliveFiltered)).unwrap(), ChangeKind::AliveFiltered);
        assert_eq!(ChangeKind::try_from(StatusInfo::from(ChangeKind::NotAliveUnregistered)).unwrap(), ChangeKind::NotAliveUnregistered);
        assert_eq!(ChangeKind::try_from(StatusInfo::from(ChangeKind::NotAliveDisposed)).unwrap(), ChangeKind::NotAliveDisposed);
    }
}
