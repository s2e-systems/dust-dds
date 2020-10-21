use std::sync::{Arc, Weak, Mutex};
use std::sync::mpsc;

use rust_dds_interface::types::{ReturnCode, InstanceHandle, TopicKind};
use rust_dds_interface::protocol::{ProtocolEntity, ProtocolReader, ProtocolSubscriber};
use rust_dds_interface::qos::DataReaderQos;

use crate::types::{GUID, EntityKind, EntityId, Locator};
use crate::messages::RtpsSubmessage;

use crate::behavior::StatefulReader;

pub struct RtpsSubscriber{
    guid: GUID,
    reader_list: [Weak<Mutex<StatefulReader>>;32],
    sender: mpsc::Sender<(Vec<Locator>,RtpsSubmessage)>,
    receiver: mpsc::Receiver<(Vec<Locator>,RtpsSubmessage)>,
}

impl RtpsSubscriber {
    pub fn new(guid: GUID) -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            guid,
            reader_list: Default::default(),
            sender,
            receiver,
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
    fn create_reader(&mut self, topic_kind: TopicKind, data_reader_qos: &DataReaderQos) -> Arc<Mutex<dyn ProtocolReader>> {
        let index = self.reader_list.iter().position(|x| x.strong_count() == 0).unwrap();

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

        let new_reader = Arc::new(Mutex::new(StatefulReader::new(
            reader_guid,
            topic_kind,
            data_reader_qos,
            self.sender.clone(),
        )));

        self.reader_list[index] = Arc::downgrade(&new_reader);

        new_reader
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_reader() {
        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];
        let entity_id = EntityId::new([0,0,0], EntityKind::UserDefinedWriterGroup);
        let guid = GUID::new(guid_prefix, entity_id);
        let mut subscriber = RtpsSubscriber::new(guid);

        let data_reader_qos = DataReaderQos::default();

        assert_eq!(subscriber.reader_list[0].strong_count(),0);
        assert_eq!(subscriber.reader_list[1].strong_count(),0);

        let reader1_arc = subscriber.create_reader(TopicKind::WithKey, &data_reader_qos);
        let reader1 = reader1_arc.lock().unwrap();
        let reader1_entityid = [0,0,0,4];
        assert_eq!(reader1.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(reader1.get_instance_handle()[12..16], reader1_entityid);


        assert_eq!(subscriber.reader_list[0].strong_count(),1);
        assert_eq!(subscriber.reader_list[1].strong_count(),0);

        let reader2 = subscriber.create_reader(TopicKind::NoKey, &data_reader_qos);
        let reader2 = reader2.lock().unwrap();
        let reader2_entityid = [0,0,1,7];
        assert_eq!(reader2.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(reader2.get_instance_handle()[12..16], reader2_entityid);

        assert_eq!(subscriber.reader_list[0].strong_count(),1);
        assert_eq!(subscriber.reader_list[1].strong_count(),1);

        std::mem::drop(reader1);
        std::mem::drop(reader1_arc);

        assert_eq!(subscriber.reader_list[0].strong_count(),0);
        assert_eq!(subscriber.reader_list[1].strong_count(),1);

        let reader3 = subscriber.create_reader(TopicKind::NoKey, &data_reader_qos);
        let reader3 = reader3.lock().unwrap();
        let reader3_entityid = [0,0,0,7];
        assert_eq!(reader3.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(reader3.get_instance_handle()[12..16], reader3_entityid);

        assert_eq!(subscriber.reader_list[0].strong_count(),1);
        assert_eq!(subscriber.reader_list[1].strong_count(),1);
    }

    #[test]
    fn create_writer_different_publishers() {
        let guid_prefix = [5, 6, 7, 8, 9, 5, 1, 2, 3, 4, 10, 11];
        let entity_id1 = EntityId::new([0,0,0], EntityKind::UserDefinedWriterGroup);
        let entity_id2 = EntityId::new([2,0,0], EntityKind::UserDefinedWriterGroup);
        let guid1 = GUID::new(guid_prefix, entity_id1);
        let guid2 = GUID::new(guid_prefix, entity_id2);
        let mut subscriber1 = RtpsSubscriber::new(guid1);
        let mut subscriber2 = RtpsSubscriber::new(guid2);

        let data_reader_qos = DataReaderQos::default();

        let reader11 = subscriber1.create_reader(TopicKind::WithKey, &data_reader_qos);
        let reader11 = reader11.lock().unwrap();
        let reader11_entityid = [0,0,0,4];
        assert_eq!(reader11.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(reader11.get_instance_handle()[12..16], reader11_entityid);

        let reader12 = subscriber1.create_reader(TopicKind::NoKey, &data_reader_qos);
        let reader12 = reader12.lock().unwrap();
        let reader12_entityid = [0,0,1,7];
        assert_eq!(reader12.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(reader12.get_instance_handle()[12..16], reader12_entityid);

        let reader21 = subscriber2.create_reader(TopicKind::NoKey, &data_reader_qos);
        let reader21 = reader21.lock().unwrap();
        let reader21_entityid = [2,0,0,7];
        assert_eq!(reader21.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(reader21.get_instance_handle()[12..16], reader21_entityid);

        let reader22 = subscriber2.create_reader(TopicKind::WithKey, &data_reader_qos);
        let reader22 = reader22.lock().unwrap();
        let reader22_entityid = [2,0,1,4];
        assert_eq!(reader22.get_instance_handle()[0..12], guid_prefix);
        assert_eq!(reader22.get_instance_handle()[12..16], reader22_entityid);
    }
}