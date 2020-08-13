pub mod types;
pub mod stateful_writer;
pub mod stateful_reader;
pub mod stateless_writer;
pub mod stateless_reader;

use std::convert::{TryFrom, TryInto};

use crate::types::{GUID, GuidPrefix, EntityId, ChangeKind};
use crate::structure::cache::CacheChange;
use crate::messages::{Data, Payload, Endianness, ParameterList};
use crate::inline_qos_types::{KeyHash, StatusInfo};

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
        Some(data.0),
        Some(inline_qos),
    )
}

fn data_from_cache_change(cache_change: &CacheChange, endianness: Endianness, reader_id: EntityId) -> Data {
    let writer_id: EntityId = cache_change.writer_guid().entity_id();
    let writer_sn = *cache_change.sequence_number();

    let mut inline_qos_parameters = match cache_change.inline_qos() {
        Some(inline_qos) => inline_qos.clone(),
        None => ParameterList::new(),
    };

    let change_kind = *cache_change.change_kind();
    inline_qos_parameters.push(StatusInfo::from(change_kind));

    let payload = match change_kind {
        ChangeKind::Alive => {
            inline_qos_parameters.push(KeyHash(*cache_change.instance_handle()));
            Payload::Data(cache_change.data_value().unwrap().to_vec())
        },
        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
            Payload::Key(cache_change.instance_handle().to_vec())
        }
    };

    Data::new(
        endianness,
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
        let endianness = data_submessage.endianness_flag().into();
        let status_info = data_submessage.inline_qos().find::<StatusInfo>(endianness).unwrap();           

        ChangeKind::try_from(status_info).unwrap()
    }
    else {
        panic!("Invalid change kind combination")
    }
}

fn key_hash(data_submessage: &Data) -> Option<KeyHash> {
    if data_submessage.data_flag() && !data_submessage.key_flag() {
        data_submessage.inline_qos().find::<KeyHash>(data_submessage.endianness_flag().into())
    } else if !data_submessage.data_flag() && data_submessage.key_flag() {
        let payload = data_submessage.serialized_payload(); 
        Some(KeyHash(payload[0..16].try_into().ok()?))
    } else {
        None
    }
}