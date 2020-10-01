use std::sync::{Arc, Weak, Mutex};
use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter, ProtocolPublisher};
use rust_dds_interface::qos::DataWriterQos;

use crate::types::{GUID, EntityId, EntityKind};
use crate::behavior;
use super::stateful_writer::StatefulWriter;

pub struct RtpsPublisher {
    guid: GUID,
    writer_list: Mutex<[Weak<StatefulWriter>;32]>,
}

impl RtpsPublisher {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid,
            writer_list: Mutex::new(Default::default()),
        }
    }
}

impl ProtocolEntity for RtpsPublisher {
    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }
}

impl ProtocolPublisher for RtpsPublisher {
    fn create_writer(&self, topic_kind: TopicKind, data_writer_qos: &DataWriterQos) -> Arc<dyn ProtocolWriter> {
        let mut writer_list = self.writer_list.lock().unwrap();
        let index = writer_list.iter().position(|x| x.strong_count() == 0).unwrap();

        let guid_prefix = self.guid.prefix();
        let publisher_entity_key = self.guid.entity_id().entity_key();
        let entity_key_msb = (index & 0xFF00) as u8;
        let entity_key_lsb = (index & 0x00FF) as u8;

        let entity_kind = match topic_kind {
            TopicKind::WithKey => EntityKind::UserDefinedWriterWithKey,
            TopicKind::NoKey => EntityKind::UserDefinedWriterNoKey,
        };

        let entity_id = EntityId::new([publisher_entity_key[0],entity_key_msb,entity_key_lsb], entity_kind);
        let writer_guid = GUID::new(guid_prefix, entity_id);

        let reliability_level = data_writer_qos.reliability.kind.into();

        let push_mode = true;
        let heartbeat_period = behavior::types::Duration::from_millis(500);
        let nack_response_delay = behavior::types::Duration::from_millis(500);
        let nack_supression_duration = behavior::types::Duration::from_millis(500);

        let new_writer = Arc::new(StatefulWriter::new(
            writer_guid,
            topic_kind,
            reliability_level,
            push_mode,
            heartbeat_period,
            nack_response_delay,
            nack_supression_duration,
        ));

        writer_list[index] = Arc::downgrade(&new_writer);

        new_writer
    }
}