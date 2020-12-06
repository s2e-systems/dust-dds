use crate::reader::Reader;
use crate::structure::RtpsGroup;
use crate::types::{EntityId, EntityKey, EntityKind, GuidPrefix, GUID};

use rust_dds_api::qos::DataReaderQos;
use rust_dds_api::types::{InstanceHandle, TopicKind, ReturnCode, ReturnCodes};


struct SubscriberInner {
    group: RtpsGroup,
    reader_counter: usize,
}

#[derive(Default)]
pub struct Subscriber {
    inner: Option<SubscriberInner>,
}

impl Subscriber {
    pub fn initialize(&mut self, guid_prefix: GuidPrefix, entity_key: EntityKey) -> ReturnCode<()> {
        if let None = self.inner {
            let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedReaderGroup);
            let subscriber_guid = GUID::new(guid_prefix, entity_id);
            let group = RtpsGroup::new(subscriber_guid);
            self.inner = Some( SubscriberInner {
                group,
                reader_counter: 0,
            });
            Ok(())
        } else {
            Err(ReturnCodes::PreconditionNotMet("RTPS subscriber already initialized"))
        }
    }
    
    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.inner.as_ref().unwrap().group.entity.guid.into()
    }

    pub fn create_reader(
        &mut self,
        topic_kind: TopicKind,
        data_reader_qos: &DataReaderQos,
    ) -> ReturnCode<Reader> {
        let subscriber = self.inner.as_mut().ok_or(ReturnCodes::AlreadyDeleted("RTPS subscriber already deleted"))?;
        let guid_prefix = subscriber.group.entity.guid.prefix();
        let entity_key = [
            subscriber.group.entity.guid.entity_id().entity_key()[0],
            subscriber.reader_counter as u8,
            0,
        ];
        subscriber.reader_counter += 1;
        Ok(Reader::new(
            guid_prefix,
            entity_key,
            topic_kind,
            data_reader_qos,
        ))
    }

}

