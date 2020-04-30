use crate::types::{Parameter, ParameterList, KeyHash, StatusInfo};
use crate::serdes::{RtpsSerialize, EndianessFlag, RtpsSerdesResult};

pub type InlineQosParameterList = ParameterList<InlineQosParameter>;

// #[derive(PartialEq, Debug)]
// pub enum InlineQosPid {
//     Pad = 0x0000,
//     Sentinel = 0x0001,
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
// }

#[derive(PartialEq, Debug)]
pub enum InlineQosParameter {
    KeyHash(KeyHash),
    StatusInfo(StatusInfo),
    // TopicName([char;256]),
}

impl Parameter for InlineQosParameter {
    fn parameter_id(&self) -> u16 {
        match self {
            InlineQosParameter::KeyHash(_) => 0x0070,
            InlineQosParameter::StatusInfo(_) => 0x0071,
        }
    }
}

impl InlineQosParameter {
    pub fn is_key_hash(&self) -> bool {
        match self {
            InlineQosParameter::KeyHash(_) => true,
            _ => false,
        }
    }

    pub fn is_status_info(&self) -> bool {
        match self {
            InlineQosParameter::StatusInfo(_) => true,
            _ => false,
        }
    }
}

impl RtpsSerialize for InlineQosParameter 
{
    fn serialize(&self, writer: &mut impl std::io::Write, endianess: EndianessFlag) -> RtpsSerdesResult<()> {
        match self {
            InlineQosParameter::KeyHash(value) => value.serialize(writer, endianess),
            InlineQosParameter::StatusInfo(value) => value.serialize(writer, endianess),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_qos_parameter_list_serialization() {
        let mut writer = Vec::new();
        let mut inline_qos_parameter_list = InlineQosParameterList::new();


        let serialized_inline_qos_parameter_list = [0x00, 0x70, 0x00, 0x10, 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, 0x00, 0x01, 0x00, 0x00];
        inline_qos_parameter_list.push(InlineQosParameter::KeyHash(KeyHash::new([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16])));
        inline_qos_parameter_list.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
        assert_eq!(writer, serialized_inline_qos_parameter_list);
    }
}
