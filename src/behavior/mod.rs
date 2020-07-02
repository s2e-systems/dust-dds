pub mod types;
pub mod stateful_writer;
pub mod stateful_reader;
pub mod stateless_writer;
pub mod stateless_reader;

use std::convert::{TryFrom, TryInto};

use crate::types::{GUID, GuidPrefix, EntityId, ChangeKind};
use crate::cache::CacheChange;
use crate::messages::{Data, Payload};
use crate::inline_qos_types::{KeyHash, StatusInfo};
use crate::serdes::Endianness;
use crate::serialized_payload::SerializedPayload;

pub use stateful_writer::StatefulWriterBehavior;
pub use stateful_reader::StatefulReaderBehavior;
pub use stateless_reader::StatelessReaderBehavior;
pub use stateless_writer::StatelessWriterBehavior;

fn cache_change_from_data(message: &Data, guid_prefix: &GuidPrefix) -> CacheChange {
    let change_kind = change_kind(&message);
    let key_hash = key_hash(&message).unwrap();
    
    CacheChange::new(
        change_kind,
        GUID::new(*guid_prefix, *message.writer_id() ),
        key_hash.0,
        *message.writer_sn(),
        None,
        None,
    )
}

fn data_from_cache_change(cache_change: &CacheChange, endianness: Endianness, reader_id: EntityId) -> Data {
    let writer_id: EntityId = *cache_change.writer_guid().entity_id();
    let writer_sn = *cache_change.sequence_number();

    let change_kind = *cache_change.change_kind();
    let status_info = StatusInfo::from(change_kind);

    // TODO: Bring inline_qos of cache change to the data message
    // if let Some(inline_qos) = cache_change.inline_qos() {
    //     parameter.extend(inline_qos.parameter().to_vec());
    // };

    // let inline_qos_parameters: Vec<&dyn ParameterOps> = parameter.iter().map(|x| x as &dyn ParameterOps).collect();

    match change_kind {
        ChangeKind::Alive => {
            let key_hash = KeyHash(*cache_change.instance_handle());
            let payload = Payload::Data(SerializedPayload(cache_change.data_value().unwrap().to_vec()));

            Data::new(
                endianness,
                reader_id,
                writer_id,
                writer_sn,
                Some(&[&status_info, &key_hash]),
                payload,
            )
        },
        ChangeKind::NotAliveDisposed | ChangeKind::NotAliveUnregistered | ChangeKind::AliveFiltered => {
            let payload = Payload::Key(SerializedPayload(cache_change.instance_handle().to_vec()));
            Data::new(
                endianness,
                reader_id,
                writer_id,
                writer_sn,
                Some(&[&status_info]),
                payload,
            )
        }
    }
}

fn change_kind(data_submessage: &Data) -> ChangeKind{
    if data_submessage.data_flag() && !data_submessage.key_flag() {
        ChangeKind::Alive
    } else if !data_submessage.data_flag() && data_submessage.key_flag() {
        let inline_qos = data_submessage.inline_qos().as_ref().unwrap();
        let endianness = data_submessage.endianness_flag().into();
        let status_info = inline_qos.find::<StatusInfo>(endianness).unwrap();           

        ChangeKind::try_from(status_info).unwrap()
    }
    else {
        panic!("Invalid change kind combination")
    }
}

fn key_hash(data_submessage: &Data) -> Option<KeyHash> {
    if data_submessage.data_flag() && !data_submessage.key_flag() {
        data_submessage.inline_qos().as_ref()?.find::<KeyHash>(data_submessage.endianness_flag().into())
    } else if !data_submessage.data_flag() && data_submessage.key_flag() {
        let payload = data_submessage.serialized_payload().as_ref()?; 
        Some(KeyHash(payload.0[0..16].try_into().ok()?))
    } else {
        None
    }
}