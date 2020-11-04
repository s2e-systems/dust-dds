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

use crate::types::{GUID, GuidPrefix, EntityId, ChangeKind, Locator};
use crate::structure::CacheChange;
use crate::messages::{Endianness, RtpsSubmessage};
use crate::messages::submessages::Data;
use crate::messages::submessages::data_submessage::Payload;
use crate::inline_qos_types::{KeyHash, StatusInfo};

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

    inline_qos.remove::<KeyHash>();
    inline_qos.remove::<StatusInfo>();

    CacheChange::new(
        change_kind,
        GUID::new(*guid_prefix, writer_id ),
        key_hash.0,
        writer_sn,
        Some(data),
        Some(inline_qos),
    )
}

fn data_from_cache_change(cache_change: &CacheChange, reader_id: EntityId) -> Data {
    let writer_id: EntityId = cache_change.writer_guid().entity_id();
    let writer_sn = cache_change.sequence_number();

    let mut inline_qos_parameters = cache_change.inline_qos().clone();

    let change_kind = cache_change.change_kind();
    inline_qos_parameters.push(change_kind_to_status_info(change_kind));

    let payload = match change_kind {
        ChangeKind::Alive => {
            inline_qos_parameters.push(KeyHash(cache_change.instance_handle()));
            Payload::Data(cache_change.data_value().clone())
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
        let endianness = Endianness::from(data_submessage.endianness_flag()).into();
        let status_info = data_submessage.inline_qos().find::<StatusInfo>(endianness).unwrap();           

        status_info_to_change_kind(status_info).unwrap()
    }
    else {
        panic!("Invalid change kind combination")
    }
}

fn key_hash(data_submessage: &Data) -> Option<KeyHash> {
    if data_submessage.data_flag() && !data_submessage.key_flag() {
        data_submessage.inline_qos().find::<KeyHash>(Endianness::from(data_submessage.endianness_flag()).into())
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