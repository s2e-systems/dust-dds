use crate::dds_infrastructure::qos::{DataWriterQos, PublisherQos, TopicQos};
use crate::dds_rtps_implementation::discovery::sedp::SimpleEndpointDiscoveryProtocol;
use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriter;
use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriterInner;
use crate::dds_rtps_implementation::rtps_object::{RtpsObject, RtpsObjectList};
use crate::dds_rtps_implementation::rtps_topic::RtpsTopic;
use crate::rtps::structure::Group;
use crate::rtps::types::{EntityId, EntityKind, GUID};
use crate::types::{Duration, InstanceHandle, ReturnCode, TopicKind};
use std::sync::RwLockReadGuard;
use std::sync::{atomic, Mutex};

pub struct RtpsPublisherInner {
    pub group: Group,
    pub writer_list: RtpsObjectList<RtpsDataWriterInner>,
    pub writer_count: atomic::AtomicU8,
    pub default_datawriter_qos: Mutex<DataWriterQos>,
    pub qos: PublisherQos,
}

impl RtpsPublisherInner {
    pub fn new(guid: GUID, qos: PublisherQos) -> Self {
        Self {
            group: Group::new(guid),
            writer_list: Default::default(),
            writer_count: atomic::AtomicU8::new(0),
            default_datawriter_qos: Mutex::new(DataWriterQos::default()),
            qos,
        }
    }
}

pub type RtpsPublisher<'a> = RwLockReadGuard<'a, RtpsObject<RtpsPublisherInner>>;

impl RtpsObject<RtpsPublisherInner> {
    pub fn create_datawriter(
        &self,
        topic: &RtpsTopic,
        qos: Option<DataWriterQos>,
        discovery: &SimpleEndpointDiscoveryProtocol,
    ) -> Option<RtpsDataWriter> {
        let this = self.value().ok()?;
        let topic = topic.value().ok()?.clone();
        let guid_prefix = this.group.entity.guid.prefix();
        let entity_key = [
            0,
            this.writer_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match topic.topic_kind {
            TopicKind::WithKey => EntityKind::UserDefinedWriterWithKey,
            TopicKind::NoKey => EntityKind::UserDefinedWriterNoKey,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_writer_guid = GUID::new(guid_prefix, entity_id);
        let new_writer_qos = qos.unwrap_or(self.get_default_datawriter_qos().ok()?);
        let new_writer = RtpsDataWriterInner::new(new_writer_guid, topic, new_writer_qos);
        let datawriter = this.writer_list.add(new_writer)?;
        discovery.insert_writer(&datawriter).ok()?;
        Some(datawriter)
    }

    pub fn delete_datawriter(
        &self,
        a_datawriter: &RtpsDataWriter,
        discovery: &SimpleEndpointDiscoveryProtocol,
    ) -> ReturnCode<()> {
        a_datawriter.value()?.topic.lock().unwrap().take(); // Drop the topic
        discovery.remove_writer(a_datawriter)?;
        a_datawriter.delete();
        Ok(())
    }

    pub fn lookup_datawriter(&self, _topic: &RtpsTopic) -> Option<RtpsDataWriter> {
        todo!()
    }

    pub fn suspend_publications(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn resume_publications(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn begin_coherent_changes(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn end_coherent_changes(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn wait_for_acknowledgments(&self, _max_wait: Duration) -> ReturnCode<()> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_default_datawriter_qos(&self, _qos: DataWriterQos) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_default_datawriter_qos(&self) -> ReturnCode<DataWriterQos> {
        Ok(self.value()?.default_datawriter_qos.lock().unwrap().clone())
    }

    pub fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_qos(&self) -> ReturnCode<PublisherQos> {
        Ok(self.value()?.qos.clone())
    }

    pub fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        Ok(self.value()?.group.entity.guid.into())
    }
}
