use crate::reader::Reader;
use crate::structure::RtpsGroup;
use crate::types::{EntityId, EntityKey, EntityKind, GuidPrefix, GUID};

use rust_dds_interface::protocol::{ProtocolEntity, ProtocolReader, ProtocolSubscriber};
use rust_dds_interface::qos::DataReaderQos;
use rust_dds_interface::types::{InstanceHandle, TopicKind};

pub struct Subscriber {
    group: RtpsGroup,
    reader_counter: usize,
}

impl Subscriber {
    pub fn new(guid_prefix: GuidPrefix, entity_key: EntityKey) -> Self {
        let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedReaderGroup);
        let subscriber_guid = GUID::new(guid_prefix, entity_id);
        let group = RtpsGroup::new(subscriber_guid);
        Self {
            group,
            reader_counter: 0,
        }
    }
}

impl ProtocolEntity for Subscriber {
    fn get_instance_handle(&self) -> InstanceHandle {
        self.group.entity.guid.into()
    }
}

impl ProtocolSubscriber for Subscriber {
    fn create_reader(
        &mut self,
        topic_kind: TopicKind,
        data_reader_qos: &DataReaderQos,
    ) -> Box<dyn ProtocolReader> {
        let guid_prefix = self.group.entity.guid.prefix();
        let entity_key = [
            self.group.entity.guid.entity_id().entity_key()[0],
            self.reader_counter as u8,
            0,
        ];
        self.reader_counter += 1;
        Box::new(Reader::new(
            guid_prefix,
            entity_key,
            topic_kind,
            data_reader_qos,
        ))
    }
}
