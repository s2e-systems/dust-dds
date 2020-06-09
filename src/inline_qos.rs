// use std::convert::TryInto;
// use num_traits::FromPrimitive;
// use num_derive::FromPrimitive;

// use crate::inline_qos_types::{KeyHash, StatusInfo, };
// use crate::messages::submessage_elements::{Parameter, ParameterList, };

// #[derive(FromPrimitive)]
// pub enum InlineQosPid {
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

// impl From<InlineQosPid> for u16 {
//     fn from(value: InlineQosPid) -> Self {
//         value as u16
//     }
// }

// #[derive(PartialEq, Debug, Clone, Eq)]
// pub enum InlineQosParameter {
//     KeyHash(KeyHash),
//     StatusInfo(StatusInfo),
//     // TopicName([char;256]),
// }

// impl Parameter for InlineQosParameter {
//     fn new_from(parameter_id: u16, value: &[u8]) -> Option<Self> {
//         let inline_qos_parameter_id = InlineQosPid::from_u16(parameter_id)?;
//         match inline_qos_parameter_id {
//             InlineQosPid::KeyHash => Some(InlineQosParameter::KeyHash(KeyHash::new(value.try_into().ok()?))),
//             InlineQosPid::StatusInfo => Some(InlineQosParameter::StatusInfo(StatusInfo(value.try_into().ok()?))),
//             _ => None,
//         }
//     }

//     fn parameter_id(&self) -> u16 {
//         match self {
//             InlineQosParameter::KeyHash(_) => InlineQosPid::KeyHash.into(),
//             InlineQosParameter::StatusInfo(_) => InlineQosPid::StatusInfo.into(),
//         }
//     }

//     fn value(&self) -> &[u8] {
//         match self {
//             InlineQosParameter::KeyHash(key_hash) => key_hash.get_value(),
//             InlineQosParameter::StatusInfo(status_info) => &status_info.0,
//         }
//     }

// }

// impl InlineQosParameter {
//     pub fn is_key_hash(&self) -> bool {
//         match self {
//             InlineQosParameter::KeyHash(_) => true,
//             _ => false,
//         }
//     }

//     pub fn is_status_info(&self) -> bool {
//         match self {
//             InlineQosParameter::StatusInfo(_) => true,
//             _ => false,
//         }
//     }
    
//     pub fn is_valid(&self) -> bool {
//         todo!()
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     use crate::serdes::{RtpsSerialize, RtpsDeserialize, EndianessFlag};

//     #[test]
//     fn test_inline_qos_parameter_list_serialization_deserialization_big_endian() {
//         let mut writer = Vec::new();
//         let mut inline_qos_parameter_list = InlineQosParameterList::new();


//         let serialized_inline_qos_parameter_list = [0x00, 0x70, 0x00, 0x10, 1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, 0x00, 0x01, 0x00, 0x00];
//         inline_qos_parameter_list.push(InlineQosParameter::KeyHash(KeyHash::new([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16])));
//         inline_qos_parameter_list.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
//         assert_eq!(writer, serialized_inline_qos_parameter_list);
//         assert_eq!(InlineQosParameterList::deserialize(&writer, EndianessFlag::BigEndian).unwrap(), inline_qos_parameter_list);
//     }

//     #[test]
//     fn test_inline_qos_parameter_list_serialization_deserialization_little_endian() {
//         let mut writer = Vec::new();
//         let mut inline_qos_parameter_list = InlineQosParameterList::new();


//         let serialized_inline_qos_parameter_list = [0x70,0x00, 0x10, 0x00,  1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16, 0x01, 0x00, 0x00, 0x00];
//         inline_qos_parameter_list.push(InlineQosParameter::KeyHash(KeyHash::new([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16])));
//         inline_qos_parameter_list.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
//         assert_eq!(writer, serialized_inline_qos_parameter_list);
//         assert_eq!(InlineQosParameterList::deserialize(&writer, EndianessFlag::LittleEndian).unwrap(), inline_qos_parameter_list);
//     }

//     #[test]
//     fn test_inline_qos_empty_parameter_list_serialization_deserialization() {
//         let mut writer = Vec::new();
//         let inline_qos_parameter_list = InlineQosParameterList::new();

//         let serialized_inline_qos_parameter_list_little_endian = [0x01, 0x00, 0x00, 0x00];
//         inline_qos_parameter_list.serialize(&mut writer, EndianessFlag::LittleEndian).unwrap();
//         assert_eq!(writer, serialized_inline_qos_parameter_list_little_endian);
//         assert_eq!(InlineQosParameterList::deserialize(&writer, EndianessFlag::LittleEndian).unwrap(), inline_qos_parameter_list);

//         writer.clear();

//         let serialized_inline_qos_parameter_list_big_endian = [0x00, 0x01, 0x00, 0x00];
//         inline_qos_parameter_list.serialize(&mut writer, EndianessFlag::BigEndian).unwrap();
//         assert_eq!(writer, serialized_inline_qos_parameter_list_big_endian);
//         assert_eq!(InlineQosParameterList::deserialize(&writer, EndianessFlag::BigEndian).unwrap(), inline_qos_parameter_list);
//     }
// }
