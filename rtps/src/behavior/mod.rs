pub mod reader;
pub mod stateful_reader;
pub mod stateful_writer;
pub mod stateless_reader;
pub mod stateless_writer;
pub mod types;
pub mod writer;

use core::convert::TryInto;

pub use reader::RTPSReader;
pub use stateful_reader::{RTPSStatefulReader, RTPSWriterProxy};
pub use stateful_writer::{RTPSReaderProxy, RTPSStatefulWriter};
pub use stateless_reader::RTPSStatelessReader;
pub use stateless_writer::RTPSStatelessWriter;
pub use writer::RTPSWriter;

use crate::{
    messages::{
        submessages::{self, submessage_elements::SerializedData},
        types::StatusInfo,
    },
    structure::RTPSCacheChange,
    types::{ChangeKind, EntityId, GUID},
};

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

pub fn data_from_cache_change<'a>(
    cache_change: &'a impl RTPSCacheChange<Data = impl AsRef<SerializedData> + 'a>,
    reader_id: EntityId,
) -> submessages::Data<'a> {
    let writer_guid: GUID = cache_change.writer_guid().try_into().unwrap();
    let writer_id = writer_guid.entity_id();
    let writer_sn = cache_change.sequence_number();

    let mut inline_qos = cache_change.inline_qos().clone();

    let change_kind = cache_change.kind();
    inline_qos
        .parameter
        .push(change_kind_to_status_info(change_kind).into());

    match change_kind {
        ChangeKind::Alive => submessages::Data {
            endianness_flag: false,
            reader_id,
            writer_id,
            writer_sn,
            serialized_payload: cache_change.data_value().as_ref(),
            inline_qos,
            inline_qos_flag: true,
            data_flag: true,
            key_flag: false,
            non_standard_payload_flag: false,
        },
        ChangeKind::NotAliveDisposed
        | ChangeKind::NotAliveUnregistered
        | ChangeKind::AliveFiltered => submessages::Data {
            endianness_flag: false,
            reader_id,
            writer_id,
            writer_sn,
            serialized_payload: cache_change.data_value().as_ref(),
            inline_qos,
            inline_qos_flag: true,
            data_flag: false,
            key_flag: true,
            non_standard_payload_flag: false,
        },
    }
}

// fn change_kind(data_submessage: &Data) -> ChangeKind {
//     if data_submessage.data_flag && !data_submessage.key_flag {
//         ChangeKind::Alive
//     } else if !data_submessage.data_flag && data_submessage.key_flag {
//         // let endianness = Endianness::from(data_submessage.endianness_flag()).into();
//         let status_info = data_submessage
//             .inline_qos
//             .parameter
//             .iter()
//             .find(|&x| x.parameter_id() == PID_STATUS_INFO)
//             .unwrap()
//             .clone();

//         status_info_to_change_kind(status_info.try_into().unwrap()).unwrap()
//     } else {
//         panic!("Invalid change kind combination")
//     }
// }

// fn key_hash(data_submessage: &Data) -> Option<KeyHash> {
//     if data_submessage.data_flag && !data_submessage.key_flag {
//         Some(
//             data_submessage
//                 .inline_qos
//                 .parameter
//                 .iter()
//                 .find(|&x| x.parameter_id() == PID_KEY_HASH)
//                 .unwrap()
//                 .clone()
//                 .try_into()
//                 .unwrap(),
//         )
//     } else if !data_submessage.data_flag && data_submessage.key_flag {
//         let payload = &data_submessage.serialized_payload;
//         Some(KeyHash(payload[0..16].try_into().ok()?))
//     } else {
//         None
//     }
// }

// fn status_info_to_change_kind(status_info: StatusInfo) -> Option<ChangeKind> {
//     if status_info.disposed_flag()
//         && !status_info.unregistered_flag()
//         && !status_info.filtered_flag()
//     {
//         Some(ChangeKind::NotAliveDisposed)
//     } else if !status_info.disposed_flag()
//         && status_info.unregistered_flag()
//         && !status_info.filtered_flag()
//     {
//         Some(ChangeKind::NotAliveUnregistered)
//     } else if !status_info.disposed_flag()
//         && !status_info.unregistered_flag()
//         && status_info.filtered_flag()
//     {
//         Some(ChangeKind::AliveFiltered)
//     } else if !status_info.disposed_flag()
//         && !status_info.unregistered_flag()
//         && !status_info.filtered_flag()
//     {
//         Some(ChangeKind::Alive)
//     } else {
//         None
//     }
// }

pub fn change_kind_to_status_info(change_kind: ChangeKind) -> StatusInfo {
    match change_kind {
        ChangeKind::Alive => StatusInfo([0, 0, 0, 0]),
        ChangeKind::NotAliveDisposed => StatusInfo([0, 0, 0, StatusInfo::DISPOSED_FLAG_MASK]),
        ChangeKind::NotAliveUnregistered => {
            StatusInfo([0, 0, 0, StatusInfo::UNREGISTERED_FLAG_MASK])
        }
        ChangeKind::AliveFiltered => StatusInfo([0, 0, 0, StatusInfo::FILTERED_FLAG_MASK]),
    }
}

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
