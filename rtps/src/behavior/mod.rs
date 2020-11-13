pub mod types;
pub mod stateful_reader;
pub mod stateful_writer;
pub mod stateless_reader;
pub mod stateless_writer;

pub use stateful_reader::{StatefulReader, WriterProxy};
pub use stateful_writer::{StatefulWriter, ReaderProxy};
pub use stateless_reader::StatelessReader;
pub use stateless_writer::StatelessWriter;

use std::convert::TryInto;

use crate::types::{GUID, GuidPrefix, EntityId, Locator,};
use crate::messages::{types::Endianness, RtpsSubmessage};
use crate::messages::submessages::Data;
use crate::messages::submessages::data_submessage::Payload;
use crate::messages::types::{KeyHash, StatusInfo, PID_KEY_HASH, PID_STATUS_INFO};

use rust_dds_interface::types::{ChangeKind,ParameterList};
use rust_dds_interface::cache_change::CacheChange;

pub const BEHAVIOR_ENDIANNESS: Endianness = Endianness::LittleEndian;

pub enum DestinedMessages {
    SingleDestination{locator: Locator, messages: Vec<RtpsSubmessage>},
    MultiDestination{unicast_locator_list: Vec<Locator>, multicast_locator_list: Vec<Locator>, messages: Vec<RtpsSubmessage>}
}


fn cache_change_from_data(message: Data, guid_prefix: &GuidPrefix) -> CacheChange {
    let writer_id = message.writer_id();
    let writer_sn = message.writer_sn();
    let change_kind = change_kind(&message);
    let key_hash = key_hash(&message).unwrap();
    let (data, mut inline_qos) = message.take_payload_and_qos();

    inline_qos.parameter.retain(|x| x.parameter_id() != PID_KEY_HASH);
    inline_qos.parameter.retain(|x| x.parameter_id() != PID_STATUS_INFO);

    CacheChange::new(
        change_kind,
        GUID::new(*guid_prefix, writer_id ).into(),
        key_hash.0,
        writer_sn,
        Some(data),
        Some(inline_qos),
    )
}

fn data_from_cache_change(cache_change: &CacheChange, reader_id: EntityId) -> Data {
    let writer_guid: GUID = cache_change.writer_guid().try_into().unwrap();
    let writer_id = writer_guid.entity_id();
    let writer_sn = cache_change.sequence_number();

    let mut inline_qos_parameters = match cache_change.inline_qos() {
        Some(parameter_list) => parameter_list.clone(),
        None => ParameterList::new(),
    };

    let change_kind = cache_change.change_kind();
    inline_qos_parameters.parameter.push(change_kind_to_status_info(change_kind).into());

    let payload = match change_kind {
        ChangeKind::Alive => {
            inline_qos_parameters.parameter.push(KeyHash(cache_change.instance_handle()).into());
            Payload::Data(cache_change.data_value().unwrap().clone())
        },
        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
            Payload::Key(cache_change.instance_handle().to_vec())
        }
    };

    Data::new(
        BEHAVIOR_ENDIANNESS,
        reader_id,
        writer_id,
        writer_sn,
        Some(inline_qos_parameters),
        payload,
    )
}

fn change_kind(data_submessage: &Data) -> ChangeKind{
    if data_submessage.data_flag() && !data_submessage.key_flag() {
        ChangeKind::Alive
    } else if !data_submessage.data_flag() && data_submessage.key_flag() {
        // let endianness = Endianness::from(data_submessage.endianness_flag()).into();
        let status_info = data_submessage
            .inline_qos()
            .parameter
            .iter()
            .find(|&x| x.parameter_id() == PID_STATUS_INFO)
            .unwrap()
            .clone();

        status_info_to_change_kind(status_info.try_into().unwrap()).unwrap()
    }
    else {
        panic!("Invalid change kind combination")
    }
}

fn key_hash(data_submessage: &Data) -> Option<KeyHash> {
    if data_submessage.data_flag() && !data_submessage.key_flag() {
        Some(data_submessage
        .inline_qos()
        .parameter
        .iter()
        .find(|&x| x.parameter_id() == PID_KEY_HASH)
        .unwrap()
        .clone()
        .try_into()
        .unwrap())
    } else if !data_submessage.data_flag() && data_submessage.key_flag() {
        let payload = &data_submessage.serialized_payload(); 
        Some(KeyHash(payload[0..16].try_into().ok()?))
    } else {
        None
    }
}

fn status_info_to_change_kind(status_info: StatusInfo) -> Option<ChangeKind>{
    if status_info.disposed_flag() && !status_info.unregistered_flag() && !status_info.filtered_flag() {
        Some(ChangeKind::NotAliveDisposed)
    } else if !status_info.disposed_flag() && status_info.unregistered_flag() && !status_info.filtered_flag() {
        Some(ChangeKind::NotAliveUnregistered)
    } else if !status_info.disposed_flag() && !status_info.unregistered_flag() && status_info.filtered_flag() {
        Some(ChangeKind::AliveFiltered)
    } else if !status_info.disposed_flag() && !status_info.unregistered_flag() && !status_info.filtered_flag() {
        Some(ChangeKind::Alive)
    } else {
        None
    }
}

fn change_kind_to_status_info(change_kind: ChangeKind) -> StatusInfo {
    match change_kind {
        ChangeKind::Alive => StatusInfo([0,0,0,0]),
        ChangeKind::NotAliveDisposed => StatusInfo([0,0,0,StatusInfo::DISPOSED_FLAG_MASK]),
        ChangeKind::NotAliveUnregistered => StatusInfo([0,0,0,StatusInfo::UNREGISTERED_FLAG_MASK]),
        ChangeKind::AliveFiltered => StatusInfo([0,0,0,StatusInfo::FILTERED_FLAG_MASK]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_info_change_kind_conversions() {
        assert_eq!(status_info_to_change_kind(change_kind_to_status_info(ChangeKind::Alive)).unwrap(), ChangeKind::Alive);
        assert_eq!(status_info_to_change_kind(change_kind_to_status_info(ChangeKind::AliveFiltered)).unwrap(), ChangeKind::AliveFiltered);
        assert_eq!(status_info_to_change_kind(change_kind_to_status_info(ChangeKind::NotAliveUnregistered)).unwrap(), ChangeKind::NotAliveUnregistered);
        assert_eq!(status_info_to_change_kind(change_kind_to_status_info(ChangeKind::NotAliveDisposed)).unwrap(), ChangeKind::NotAliveDisposed);
    }
}