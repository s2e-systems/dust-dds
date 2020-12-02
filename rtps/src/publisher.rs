use std::sync::Mutex;
use crate::structure::RtpsGroup;
use crate::types::{EntityId, EntityKey, EntityKind, GuidPrefix, GUID};

use crate::writer::Writer;

use rust_dds_interface::qos::DataWriterQos;
use rust_dds_interface::types::{InstanceHandle, TopicKind, ReturnCode, ReturnCodes};

struct PublisherInner {
    group: RtpsGroup,
    writer_counter: usize,
}

#[derive(Default)]
pub struct Publisher {
    inner: Mutex<Option<PublisherInner>>,
}

impl Publisher {
    pub fn initialize(&self, guid_prefix: GuidPrefix, entity_key: EntityKey) -> ReturnCode<()> {
        let mut publisher_guard = self.inner.lock().unwrap();
        if let None = *publisher_guard {
            let entity_id = EntityId::new(entity_key, EntityKind::UserDefinedWriterGroup);
            let publisher_guid = GUID::new(guid_prefix, entity_id);
            let group = RtpsGroup::new(publisher_guid);
            *publisher_guard = Some(PublisherInner {
                group,
                writer_counter: 0,
            });
            Ok(())
        } else {
            Err(ReturnCodes::PreconditionNotMet("RTPS publisher already initialized"))
        }
    }

    pub fn create_writer(
        &self,
        topic_kind: TopicKind,
        data_writer_qos: &DataWriterQos,
    ) -> ReturnCode<Writer> {
        let mut publisher_guard = self.inner.lock().unwrap();
        let mut publisher = publisher_guard.as_mut().ok_or(ReturnCodes::AlreadyDeleted("RTPS publisher already deleted"))?;
        let guid_prefix = publisher.group.entity.guid.prefix();
        let entity_key = [
            publisher.group.entity.guid.entity_id().entity_key()[0],
            publisher.writer_counter as u8,
            0,
        ];

        publisher.writer_counter += 1;

        Ok(Writer::new(
            guid_prefix,
            entity_key,
            topic_kind,
            data_writer_qos,
        ))
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.inner.lock().unwrap().as_ref().unwrap().group.entity.guid.into()
    }
}