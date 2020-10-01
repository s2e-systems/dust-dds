use std::sync::{Arc, Weak, Mutex};

use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolReader, ProtocolSubscriber};
use rust_dds_interface::qos::DataReaderQos;

use crate::types::{GUID, EntityKind, EntityId};
use crate::behavior;

use super::stateful_reader::StatefulReader;

pub struct RtpsSubscriber{
    guid: GUID,
    reader_list: Mutex<[Weak<StatefulReader>;32]>,
}

impl RtpsSubscriber {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid,
            reader_list: Mutex::new(Default::default()),
        }
    }
}

impl ProtocolEntity for RtpsSubscriber {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }
}

impl ProtocolSubscriber for RtpsSubscriber {
    fn create_reader(&self, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> Arc<dyn ProtocolReader> {
        let mut reader_list = self.reader_list.lock().unwrap();
        let index = reader_list.iter().position(|x| x.strong_count() == 0).unwrap();

        let guid_prefix = self.guid.prefix();
        let publisher_entity_key = self.guid.entity_id().entity_key();
        let entity_key_msb = (index & 0xFF00) as u8;
        let entity_key_lsb = (index & 0x00FF) as u8;

        let entity_kind = match topic_kind {
            TopicKind::WithKey => EntityKind::UserDefinedReaderWithKey,
            TopicKind::NoKey => EntityKind::UserDefinedReaderNoKey,
        };

        let entity_id = EntityId::new([publisher_entity_key[0],entity_key_msb,entity_key_lsb], entity_kind);
        let reader_guid = GUID::new(guid_prefix, entity_id);

        let reliability_level = data_reader_qos.reliability.kind.into();

        let expects_inline_qos = false;
        let heartbeat_response_delay = behavior::types::constants::DURATION_ZERO;

        let new_reader = Arc::new(StatefulReader::new(
            reader_guid,
            topic_kind,
            reliability_level,
            expects_inline_qos,
            heartbeat_response_delay,
        ));

        reader_list[index] = Arc::downgrade(&new_reader);

        new_reader
    }
}