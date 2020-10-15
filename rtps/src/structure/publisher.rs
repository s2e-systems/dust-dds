use std::sync::{Arc, Weak, Mutex};
use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolWriter, ProtocolPublisher};
use rust_dds_interface::qos::DataWriterQos;

use crate::types::{GUID, EntityId, EntityKind};
use crate::behavior::StatefulWriter;

pub struct RtpsPublisher {
    guid: GUID,
    writer_list: [Weak<Mutex<StatefulWriter>>;32],
}

impl RtpsPublisher {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid,
            writer_list: Default::default(),
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
    fn create_writer(&mut self, topic_kind: TopicKind, data_writer_qos: &DataWriterQos) -> Arc<Mutex<dyn ProtocolWriter>> {
        let index = self.writer_list.iter().position(|x| x.strong_count() == 0).unwrap();

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

        let new_writer = Arc::new(Mutex::new(StatefulWriter::new(
            writer_guid,
            topic_kind,
            data_writer_qos
        )));

        self.writer_list[index] = Arc::downgrade(&new_writer);

        new_writer
    }

    fn create_builtin_stateless_writer(&self, _topic_kind: TopicKind, _data_writer_qos: &DataWriterQos) -> Arc<dyn ProtocolWriter> {
        todo!()
    }

    fn create_builtin_stateful_writer(&self, _topic_kind: TopicKind, _data_writer_qos: &DataWriterQos) -> Arc<dyn ProtocolWriter> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_writer() {
        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];
        let entity_id = EntityId::new([0,0,0], EntityKind::UserDefinedWriterGroup);
        let guid = GUID::new(guid_prefix, entity_id);
        let mut publisher = RtpsPublisher::new(guid);

        let data_writer_qos = DataWriterQos::default();

        assert_eq!(publisher.writer_list[0].strong_count(),0);
        assert_eq!(publisher.writer_list[1].strong_count(),0);

        let writer1_arc = publisher.create_writer(TopicKind::WithKey, &data_writer_qos);
        let writer1 = writer1_arc.lock().unwrap();
        let writer1_entityid = [0,0,0,2];
        assert_eq!(writer1.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(writer1.get_instance_handle()[12..16], writer1_entityid);


        assert_eq!(publisher.writer_list[0].strong_count(),1);
        assert_eq!(publisher.writer_list[1].strong_count(),0);

        let writer2 = publisher.create_writer(TopicKind::NoKey, &data_writer_qos);
        let writer2 = writer2.lock().unwrap();
        let writer2_entityid = [0,0,1,3];
        assert_eq!(writer2.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(writer2.get_instance_handle()[12..16], writer2_entityid);

        assert_eq!(publisher.writer_list[0].strong_count(),1);
        assert_eq!(publisher.writer_list[1].strong_count(),1);

        std::mem::drop(writer1);
        std::mem::drop(writer1_arc);

        assert_eq!(publisher.writer_list[0].strong_count(),0);
        assert_eq!(publisher.writer_list[1].strong_count(),1);

        let writer3 = publisher.create_writer(TopicKind::NoKey, &data_writer_qos);
        let writer3 = writer3.lock().unwrap();
        let writer3_entityid = [0,0,0,3];
        assert_eq!(writer3.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(writer3.get_instance_handle()[12..16], writer3_entityid);

        assert_eq!(publisher.writer_list[0].strong_count(),1);
        assert_eq!(publisher.writer_list[1].strong_count(),1);
    }

    #[test]
    fn create_writer_different_publishers() {
        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];
        let entity_id1 = EntityId::new([0,0,0], EntityKind::UserDefinedWriterGroup);
        let entity_id2 = EntityId::new([2,0,0], EntityKind::UserDefinedWriterGroup);
        let guid1 = GUID::new(guid_prefix, entity_id1);
        let guid2 = GUID::new(guid_prefix, entity_id2);
        let mut publisher1 = RtpsPublisher::new(guid1);
        let mut publisher2 = RtpsPublisher::new(guid2);

        let data_writer_qos = DataWriterQos::default();

        let writer11 = publisher1.create_writer(TopicKind::WithKey, &data_writer_qos);
        let writer11 = writer11.lock().unwrap();
        let writer11_entityid = [0,0,0,2];
        assert_eq!(writer11.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(writer11.get_instance_handle()[12..16], writer11_entityid);

        let writer12 = publisher1.create_writer(TopicKind::NoKey, &data_writer_qos);
        let writer12 = writer12.lock().unwrap();
        let writer12_entityid = [0,0,1,3];
        assert_eq!(writer12.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(writer12.get_instance_handle()[12..16], writer12_entityid);

        let writer21 = publisher2.create_writer(TopicKind::NoKey, &data_writer_qos);
        let writer21 = writer21.lock().unwrap();
        let writer21_entityid = [2,0,0,3];
        assert_eq!(writer21.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(writer21.get_instance_handle()[12..16], writer21_entityid);

        let writer22 = publisher2.create_writer(TopicKind::WithKey, &data_writer_qos);
        let writer22 = writer22.lock().unwrap();
        let writer22_entityid = [2,0,1,2];
        assert_eq!(writer22.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(writer22.get_instance_handle()[12..16], writer22_entityid);
    }
}