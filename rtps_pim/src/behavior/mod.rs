pub mod reader;
// pub mod stateful_reader;
// pub  parameter_id: (), length: (), value: ()mod stateful_writer;
// pub mod stateless_reader;
// pub mod stateless_writer;
mod types;
pub mod writer;

pub use reader::RTPSReader;
pub use types::Types;
// pub use stateful_reader::{RTPSStatefulReader, RTPSWriterProxy};
// pub use stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter};
// pub use stateless_reader::RTPSStatelessReader;
// pub use stateless_writer::RTPSStatelessWriter;
pub use writer::RTPSWriter;

// use crate::{
//     messages::{
//         submessages::{self, submessage_elements::SerializedData},
//         types::StatusInfo,
//     },
//     structure::RTPSCacheChange,
//     types::{ChangeKind, EntityId, GUID},
// };

// fn _cache_change_from_data<'a, T, C>(mut message: Data<'a>, guid_prefix: &GuidPrefix) -> C
// where
//     T: From<SerializedData<'a>>,
//     C: RTPSCacheChange<Data = T>,
// {
//     let change_kind = change_kind(&message);
//     let key_hash = key_hash(&message).unwrap();

//     message
//         .inline_qos
//         .parameter
//         .retain(|x| x.parameter_id() != PID_KEY_HASH);
//     message
//         .inline_qos
//         .parameter
//         .retain(|x| x.parameter_id() != PID_STATUS_INFO);

//     C::new(
//         change_kind,
//         GUID::new(*guid_prefix, message.writer_id).into(),
//         key_hash.0,
//         message.writer_sn,
//         message.serialized_payload.into(),
//         message.inline_qos,
//     )
// }
// type EntityId: types::EntityId;
// type SequenceNumber: types::SequenceNumber;
// type ParameterId: messages::types::ParameterId;
// type ParameterValue: AsRef<[u8]>;
// type ParameterList: IntoIterator<
//     Item = submessage_elements::Parameter<Self::ParameterId, Self::ParameterValue>,
// >;
// type SerializedData: AsRef<[u8]>;

// pub fn data_submessage_from_cache_change<
//     'a,
//     DataSubmessage,
//     GuidPrefix: crate::types::GuidPrefix,
//     InstanceHandle: crate::types::InstanceHandle,
//     Data: AsRef<[u8]>,
//     ParameterId: messages::types::ParameterId,
//     ParameterValue: AsRef<[u8]> + Clone,
//     ParameterList: IntoIterator<Item = submessage_elements::Parameter<ParameterId, ParameterValue>> + Clone,
// >(
//     cache_change: &'a RTPSCacheChange<
//         GuidPrefix,
//         DataSubmessage::EntityId,
//         InstanceHandle,
//         DataSubmessage::SequenceNumber,
//         Data,
//         ParameterId,
//         ParameterValue,
//         ParameterList,
//     >,
//     reader_id: DataSubmessage::EntityId,
// ) -> DataSubmessage
// where
//     DataSubmessage: submessages::data_submessage::Data<
//         SerializedData = &'a [u8],
//         ParameterId = ParameterId,
//         ParameterValue = ParameterValue,
//         ParameterList = ParameterList,
//     >,
//     // &'a ParameterList:
//     // IntoIterator<Item = submessage_elements::Parameter<ParameterId, ParameterValue>>,
// {
//     let endianness_flag = true.into();
//     let non_standard_payload_flag = false.into();
//     let reader_id = submessage_elements::EntityId { value: reader_id };
//     let writer_id = submessage_elements::EntityId {
//         value: cache_change.writer_guid.entity_id,
//     };
//     let writer_sn = submessage_elements::SequenceNumber {
//         value: cache_change.sequence_number,
//     };

//     let inline_qos = cache_change.inline_qos.clone();
//     let serialized_payload = submessage_elements::SerializedData {
//         value: cache_change.data_value.as_ref(),
//     };

//     match cache_change.kind {
//         crate::types::ChangeKind::Alive => {
//             let data_flag = true.into();
//             let key_flag = false.into();
//             let inline_qos_flag = true.into();
//             DataSubmessage::new(
//                 endianness_flag,
//                 inline_qos_flag,
//                 data_flag,
//                 key_flag,
//                 non_standard_payload_flag,
//                 reader_id,
//                 writer_id,
//                 writer_sn,
//                 inline_qos,
//                 serialized_payload,
//             )
//         }
//         ChangeKind::NotAliveDisposed
//         | ChangeKind::NotAliveUnregistered
//         | ChangeKind::AliveFiltered => {
//             let data_flag = false.into();
//             let key_flag = true.into();
//             let inline_qos_flag = true.into();
//             DataSubmessage::new(
//                 endianness_flag,
//                 inline_qos_flag,
//                 data_flag,
//                 key_flag,
//                 non_standard_payload_flag,
//                 reader_id,
//                 writer_id,
//                 writer_sn,
//                 inline_qos,
//                 serialized_payload,
//             )
//         }
//     }
// }

// // fn change_kind(data_submessage: &Data) -> ChangeKind {
// //     if data_submessage.data_flag && !data_submessage.key_flag {
// //         ChangeKind::Alive
// //     } else if !data_submessage.data_flag && data_submessage.key_flag {
// //         // let endianness = Endianness::from(data_submessage.endianness_flag()).into();
// //         let status_info = data_submessage
// //             .inline_qos
// //             .parameter
// //             .iter()
// //             .find(|&x| x.parameter_id() == PID_STATUS_INFO)
// //             .unwrap()
// //             .clone();

// //         status_info_to_change_kind(status_info.try_into().unwrap()).unwrap()
// //     } else {
// //         panic!("Invalid change kind combination")
// //     }
// // }

// // fn key_hash(data_submessage: &Data) -> Option<KeyHash> {
// //     if data_submessage.data_flag && !data_submessage.key_flag {
// //         Some(
// //             data_submessage
// //                 .inline_qos
// //                 .parameter
// //                 .iter()
// //                 .find(|&x| x.parameter_id() == PID_KEY_HASH)
// //                 .unwrap()
// //                 .clone()
// //                 .try_into()
// //                 .unwrap(),
// //         )
// //     } else if !data_submessage.data_flag && data_submessage.key_flag {
// //         let payload = &data_submessage.serialized_payload;
// //         Some(KeyHash(payload[0..16].try_into().ok()?))
// //     } else {
// //         None
// //     }
// // }

// // fn status_info_to_change_kind(status_info: StatusInfo) -> Option<ChangeKind> {
// //     if status_info.disposed_flag()
// //         && !status_info.unregistered_flag()
// //         && !status_info.filtered_flag()
// //     {
// //         Some(ChangeKind::NotAliveDisposed)
// //     } else if !status_info.disposed_flag()
// //         && status_info.unregistered_flag()
// //         && !status_info.filtered_flag()
// //     {
// //         Some(ChangeKind::NotAliveUnregistered)
// //     } else if !status_info.disposed_flag()
// //         && !status_info.unregistered_flag()
// //         && status_info.filtered_flag()
// //     {
// //         Some(ChangeKind::AliveFiltered)
// //     } else if !status_info.disposed_flag()
// //         && !status_info.unregistered_flag()
// //         && !status_info.filtered_flag()
// //     {
// //         Some(ChangeKind::Alive)
// //     } else {
// //         None
// //     }
// // }

// pub fn change_kind_to_status_info(change_kind: ChangeKind) -> StatusInfo {
//     match change_kind {
//         ChangeKind::Alive => StatusInfo([0, 0, 0, 0]),
//         ChangeKind::NotAliveDisposed => StatusInfo([0, 0, 0, StatusInfo::DISPOSED_FLAG_MASK]),
//         ChangeKind::NotAliveUnregistered => {
//             StatusInfo([0, 0, 0, StatusInfo::UNREGISTERED_FLAG_MASK])
//         }
//         ChangeKind::AliveFiltered => StatusInfo([0, 0, 0, StatusInfo::FILTERED_FLAG_MASK]),
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         messages::submessages::{submessage_elements::ParameterList, Submessage},
//         types::GUID,
//     };

//     #[derive(Clone, Copy)]
//     struct MockSubmessageKind(u8);

//     impl crate::messages::types::SubmessageKind for MockSubmessageKind {
//         const DATA: Self = Self(0);
//         const GAP: Self = Self(0);
//         const HEARTBEAT: Self = Self(0);
//         const ACKNACK: Self = Self(0);
//         const PAD: Self = Self(0);
//         const INFO_TS: Self = Self(0);
//         const INFO_REPLY: Self = Self(0);
//         const INFO_DST: Self = Self(0);
//         const INFO_SRC: Self = Self(0);
//         const DATA_FRAG: Self = Self(0);
//         const NACK_FRAG: Self = Self(0);
//         const HEARTBEAT_FRAG: Self = Self(0);
//     }

//     #[derive(Clone, Copy)]
//     struct MockSubmessageFlag(bool);
//     impl crate::messages::types::SubmessageFlag for MockSubmessageFlag {}

//     impl From<bool> for MockSubmessageFlag {
//         fn from(value: bool) -> Self {
//             Self(value)
//         }
//     }

//     impl Into<bool> for MockSubmessageFlag {
//         fn into(self) -> bool {
//             todo!()
//         }
//     }

//     impl crate::types::EntityId for [u8; 4] {
//         const ENTITYID_UNKNOWN: Self = [0; 4];
//     }

//     impl crate::types::SequenceNumber for i64 {
//         const SEQUENCE_NUMBER_UNKNOWN: Self = 0;
//     }

//     impl crate::messages::types::ParameterId for u16 {}

//     impl crate::types::GuidPrefix for [u8; 12] {
//         const GUIDPREFIX_UNKNOWN: Self = [0; 12];
//     }

//     impl crate::types::InstanceHandle for i32 {}

//     impl crate::messages::types::ParameterId for i16 {}

//     struct MockDataSubmessage<'a> {
//         endianness_flag: MockSubmessageFlag,
//         inline_qos_flag: MockSubmessageFlag,
//         data_flag: MockSubmessageFlag,
//         key_flag: MockSubmessageFlag,
//         non_standard_payload_flag: MockSubmessageFlag,
//         reader_id: submessage_elements::EntityId<[u8; 4]>,
//         writer_id: submessage_elements::EntityId<[u8; 4]>,
//         writer_sn: submessage_elements::SequenceNumber<i64>,
//         inline_qos: submessage_elements::ParameterList<
//             u16,
//             Vec<u8>,
//             Vec<submessage_elements::Parameter<u16, Vec<u8>>>,
//         >,
//         serialized_payload: submessage_elements::SerializedData<&'a [u8]>,
//     }

//     impl<'a> Submessage for MockDataSubmessage<'a> {
//         type SubmessageKind = MockSubmessageKind;
//         type SubmessageFlag = MockSubmessageFlag;

//         fn submessage_header(
//             &self,
//         ) -> submessages::SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag> {
//             todo!()
//         }
//     }

//     impl<'a> crate::messages::submessages::data_submessage::Data for MockDataSubmessage<'a> {
//         type EntityId = [u8; 4];
//         type SequenceNumber = i64;
//         type ParameterId = u16;
//         type ParameterValue = Vec<u8>;
//         type ParameterList =
//             Vec<submessage_elements::Parameter<Self::ParameterId, Self::ParameterValue>>;
//         type SerializedData = &'a [u8];

//         fn new(
//             endianness_flag: MockSubmessageFlag,
//             inline_qos_flag: MockSubmessageFlag,
//             data_flag: MockSubmessageFlag,
//             key_flag: MockSubmessageFlag,
//             non_standard_payload_flag: MockSubmessageFlag,
//             reader_id: submessage_elements::EntityId<Self::EntityId>,
//             writer_id: submessage_elements::EntityId<Self::EntityId>,
//             writer_sn: submessage_elements::SequenceNumber<Self::SequenceNumber>,
//             inline_qos: submessage_elements::ParameterList<
//                 Self::ParameterId,
//                 Self::ParameterValue,
//                 Self::ParameterList,
//             >,
//             serialized_payload: submessage_elements::SerializedData<Self::SerializedData>,
//         ) -> Self {
//             Self {
//                 endianness_flag,
//                 inline_qos_flag,
//                 data_flag,
//                 key_flag,
//                 non_standard_payload_flag,
//                 reader_id,
//                 writer_id,
//                 writer_sn,
//                 inline_qos,
//                 serialized_payload,
//             }
//         }

//         fn endianness_flag(&self) -> MockSubmessageFlag {
//             todo!()
//         }

//         fn inline_qos_flag(&self) -> MockSubmessageFlag {
//             todo!()
//         }

//         fn data_flag(&self) -> MockSubmessageFlag {
//             todo!()
//         }

//         fn key_flag(&self) -> MockSubmessageFlag {
//             todo!()
//         }

//         fn non_standard_payload_flag(&self) -> MockSubmessageFlag {
//             todo!()
//         }

//         fn reader_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
//             todo!()
//         }

//         fn writer_id(&self) -> &submessage_elements::EntityId<Self::EntityId> {
//             todo!()
//         }

//         fn writer_sn(&self) -> &submessage_elements::SequenceNumber<Self::SequenceNumber> {
//             todo!()
//         }

//         fn inline_qos(
//             &self,
//         ) -> &ParameterList<Self::ParameterId, Self::ParameterValue, Self::ParameterList> {
//             todo!()
//         }

//         fn serialized_payload(&self) -> &submessage_elements::SerializedData<Self::SerializedData> {
//             todo!()
//         }
//     }

//     #[test]
//     fn create_data_submessage_from_cache_change() {
//         let change_kind = ChangeKind::Alive;
//         let writer_id = [1, 2, 4, 5];
//         let writer_guid = GUID {
//             guid_prefix: [1; 12],
//             entity_id: writer_id,
//         };
//         let instance_handle = 0;
//         let sequence_number = 2;
//         let data_value = vec![1, 2, 3, 4];
//         let parameter1 = submessage_elements::Parameter {
//             parameter_id: 100,
//             length: 4,
//             value: vec![10, 12, 14, 16],
//         };
//         let inline_qos: submessage_elements::ParameterList<
//             u16,
//             Vec<u8>,
//             Vec<submessage_elements::Parameter<u16, Vec<u8>>>,
//         > = submessage_elements::ParameterList {
//             parameter: vec![parameter1.clone()],
//         };
//         let cache_change = RTPSCacheChange {
//             kind: change_kind,
//             writer_guid,
//             instance_handle,
//             sequence_number,
//             data_value: data_value.clone(),
//             inline_qos,
//         };

//         let reader_id = [5; 4];

//         let data_submessage: MockDataSubmessage =
//             data_submessage_from_cache_change(&cache_change, reader_id);

//         assert_eq!(data_submessage.endianness_flag.0, true);
//         assert_eq!(data_submessage.inline_qos_flag.0, true);
//         assert_eq!(data_submessage.data_flag.0, true);
//         assert_eq!(data_submessage.key_flag.0, false);
//         assert_eq!(data_submessage.non_standard_payload_flag.0, false);
//         assert_eq!(data_submessage.reader_id.value, reader_id);
//         assert_eq!(data_submessage.writer_id.value, writer_id);
//         assert_eq!(data_submessage.writer_sn.value, sequence_number);
//         assert_eq!(
//             data_submessage.inline_qos.parameter[0].parameter_id,
//             parameter1.parameter_id
//         );
//         assert_eq!(
//             data_submessage.inline_qos.parameter[0].length,
//             parameter1.length
//         );
//         assert_eq!(
//             data_submessage.inline_qos.parameter[0].value,
//             parameter1.value
//         );
//         assert_eq!(data_submessage.serialized_payload.value, &data_value);
//     }
// }
