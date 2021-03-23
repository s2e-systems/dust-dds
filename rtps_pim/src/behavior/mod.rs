pub mod reader;
// pub mod stateful_reader;
// pub mod stateful_writer;
// pub mod stateless_reader;
pub mod stateless_writer;
pub mod types;
pub mod writer;

pub use reader::RTPSReader;
// pub use stateful_reader::{RTPSStatefulReader, RTPSWriterProxy};
// pub use stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter};
// pub use stateless_reader::RTPSStatelessReader;
// pub use stateless_writer::RTPSStatelessWriter;
pub use writer::RTPSWriter;

use crate::{
    messages::submessages::{self, submessage_elements},
    structure::RTPSCacheChange,
    types::{ChangeKind, GUID},
};

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

pub fn data_from_cache_change<'a, Data: submessages::data_submessage::Data<'a>>(
    _cache_change: &'a impl RTPSCacheChange<
        SequenceNumber = <<Data as submessages::data_submessage::Data<'a>>::SequenceNumber as submessage_elements::SequenceNumber>::SequenceNumber,
        EntityId = <<Data as submessages::data_submessage::Data<'a>>::EntityId as submessage_elements::EntityId>::EntityId,
        ParameterList = <Data as submessages::data_submessage::Data<'a>>::ParameterList,
        Data = <<Data as submessages::data_submessage::Data<'a>>::SerializedData as submessage_elements::SerializedData<'a>>::SerializedData
    >,
    _reader_id: <<Data as submessages::data_submessage::Data<'a>>::EntityId as submessage_elements::EntityId>::EntityId,
) -> Data {
    todo!()
    // let endianness_flag = true.into();
    // let non_standard_payload_flag = false.into();
    // let reader_id = submessage_elements::EntityId::new(reader_id);
    // let writer_id = submessage_elements::EntityId::new(cache_change.writer_guid().entity_id());
    // let writer_sn = submessage_elements::SequenceNumber::new(cache_change.sequence_number());

    // let inline_qos = cache_change.inline_qos();
    // let serialized_payload = submessage_elements::SerializedData::new(cache_change.data_value());

    // let change_kind = cache_change.kind();

    // match change_kind {
    //     crate::types::ChangeKind::Alive => {
    //         let data_flag = true.into();
    //         let key_flag = false.into();
    //         let inline_qos_flag = true.into();
    //         Data::new(
    //             endianness_flag,
    //             inline_qos_flag,
    //             data_flag,
    //             key_flag,
    //             non_standard_payload_flag,
    //             reader_id,
    //             writer_id,
    //             writer_sn,
    //             inline_qos,
    //             serialized_payload,
    //         )
    //     }
    //     ChangeKind::NotAliveDisposed
    //     | ChangeKind::NotAliveUnregistered
    //     | ChangeKind::AliveFiltered => {
    //         let data_flag = false.into();
    //         let key_flag = true.into();
    //         let inline_qos_flag = true.into();
    //         Data::new(
    //             endianness_flag,
    //             inline_qos_flag,
    //             data_flag,
    //             key_flag,
    //             non_standard_payload_flag,
    //             reader_id,
    //             writer_id,
    //             writer_sn,
    //             inline_qos,
    //             serialized_payload,
    //         )
    //     }
    // }
}

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

//     #[test]
//     fn status_info_change_kind_conversions() {
//         assert_eq!(
//             status_info_to_change_kind(change_kind_to_status_info(ChangeKind::Alive)).unwrap(),
//             ChangeKind::Alive
//         );
//         assert_eq!(
//             status_info_to_change_kind(change_kind_to_status_info(ChangeKind::AliveFiltered))
//                 .unwrap(),
//             ChangeKind::AliveFiltered
//         );
//         assert_eq!(
//             status_info_to_change_kind(change_kind_to_status_info(
//                 ChangeKind::NotAliveUnregistered
//             ))
//             .unwrap(),
//             ChangeKind::NotAliveUnregistered
//         );
//         assert_eq!(
//             status_info_to_change_kind(change_kind_to_status_info(ChangeKind::NotAliveDisposed))
//                 .unwrap(),
//             ChangeKind::NotAliveDisposed
//         );
//     }
// }
