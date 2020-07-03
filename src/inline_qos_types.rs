/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// in the sub clauses of 9.6.3 ParameterId Definitions used to Represent In-line QoS
///  
 
use std::convert::{TryInto, From};
use crate::serdes::{RtpsSerialize, RtpsDeserialize, Endianness, RtpsSerdesResult, SizeCheck};
use crate::types::{ChangeKind, };
use crate::messages::types::{Pid, ParameterId};

const PID_TOPIC_NAME : ParameterId = 0x0005;
const PID_KEY_HASH : ParameterId = 0x0070;
const PID_STATUS_INFO : ParameterId = 0x0071;
  

    //     #[derive(Debug, PartialEq, FromPrimitive, ToPrimitive, Eq, Clone, Copy)]
    // #[repr(u16)]
    // pub enum ParameterIdT {
    //     TopicName = 0x0005,
    //     Durability = 0x001d,
    //     Presentation = 0x0021,
    //     Deadline = 0x0023,
    //     LatencyBudget = 0x0027,
    //     Ownership = 0x001f,
    //     OwnershipStrength = 0x0006,
    //     Liveliness = 0x001b,
    //     Partition = 0x0029,
    //     Reliability = 0x001a,
    //     TransportPriority = 0x0049,
    //     Lifespan = 0x002b,
    //     DestinationOrder = 0x0025,
    //     ContentFilterInfo = 0x0055,
    //     CoherentSet = 0x0056,
    //     DirectedWrite = 0x0057,
    //     OriginalWriterInfo = 0x0061,
    //     GroupCoherentSet = 0x0063,
    //     GroupSeqNum = 0x0064,
    //     WriterGroupInfo = 0x0065,
    //     SecureWriterGroupInfo = 0x0066,
    //     KeyHash = 0x0070,
    //     StatusInfo = 0x0071,
    //     Sentinel = 0x0001,
    //     VendorTest0 = 0x0000 | 0x8000,
    //     VendorTest1 = 0x0001 | 0x8000,
    //     VendorTest3 = 0x0003 | 0x8000,
    //     VendorTest4 = 0x0004 | 0x8000,
    //     VendorTest5 = 0x0005 | 0x8000,
    //     VendorTestShort = 0x0006 | 0x8000,
    // }
    // const 
// }

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct TopicName(pub Vec<u8>);
impl Pid for TopicName {
    fn pid() -> ParameterId {
        PID_TOPIC_NAME
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub struct KeyHash(pub [u8; 16]);

impl Pid for KeyHash {
    fn pid() -> ParameterId {
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

impl Pid for StatusInfo {
    fn pid() -> ParameterId {
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
