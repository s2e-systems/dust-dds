use crate::dds_infrastructure::qos::{DataReaderQos, SubscriberQos, TopicQos};
use crate::dds_infrastructure::status::SampleLostStatus;
use crate::dds_rtps_implementation::discovery::sedp::SimpleEndpointDiscoveryProtocol;
use crate::dds_rtps_implementation::rtps_data_reader::{RtpsDataReader, RtpsDataReaderInner};
use crate::dds_rtps_implementation::rtps_object::{RtpsObject, RtpsObjectList};
use crate::dds_rtps_implementation::rtps_topic::RtpsTopic;
use crate::rtps::structure::Group;
use crate::rtps::types::{EntityId, EntityKind, GUID};
use crate::types::{InstanceHandle, ReturnCode, TopicKind};
use std::sync::{atomic, Mutex, RwLockReadGuard};
pub struct RtpsSubscriberInner {
    pub group: Group,
    pub reader_list: RtpsObjectList<RtpsDataReaderInner>,
    pub reader_count: atomic::AtomicU8,
    pub default_datareader_qos: Mutex<DataReaderQos>,
    pub qos: SubscriberQos,
}

impl RtpsSubscriberInner {
    pub fn new(guid: GUID, qos: SubscriberQos) -> Self {
        Self {
            group: Group::new(guid),
            reader_list: Default::default(),
            reader_count: atomic::AtomicU8::new(0),
            default_datareader_qos: Mutex::new(DataReaderQos::default()),
            qos,
        }
    }
}

pub type RtpsSubscriber<'a> = RwLockReadGuard<'a, RtpsObject<RtpsSubscriberInner>>;

impl RtpsObject<RtpsSubscriberInner> {
    pub fn create_datareader(
        &self,
        topic: &RtpsTopic,
        qos: Option<DataReaderQos>,
        discovery: &SimpleEndpointDiscoveryProtocol,
    ) -> Option<RtpsDataReader> {
        let this = self.value().ok()?;
        let topic = topic.value().ok()?.clone();
        let guid_prefix = this.group.entity.guid.prefix();
        let entity_key = [
            0,
            this.reader_count.fetch_add(1, atomic::Ordering::Relaxed),
            0,
        ];
        let entity_kind = match topic.topic_kind {
            TopicKind::WithKey => EntityKind::UserDefinedReaderWithKey,
            TopicKind::NoKey => EntityKind::UserDefinedReaderNoKey,
        };
        let entity_id = EntityId::new(entity_key, entity_kind);
        let new_reader_guid = GUID::new(guid_prefix, entity_id);
        let new_reader_qos = qos.unwrap_or(self.get_default_datareader_qos().ok()?);
        let new_reader = RtpsDataReaderInner::new(new_reader_guid, topic, new_reader_qos);
        let datareader = this.reader_list.add(new_reader)?;
        discovery.insert_reader(&datareader).ok()?;
        Some(datareader)
    }

    pub fn delete_datareader(
        &self,
        a_datareader: &RtpsDataReader,
        discovery: &SimpleEndpointDiscoveryProtocol,
    ) -> ReturnCode<()> {
        a_datareader.value()?.topic.lock().unwrap().take(); // Drop the topic
        discovery.remove_reader(a_datareader)?;
        a_datareader.delete();
        Ok(())
    }

    pub fn lookup_datareader(&self, _topic: &RtpsTopic) -> Option<RtpsDataReader> {
        todo!()
    }

    pub fn begin_access(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn end_access(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn notify_datareaders(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> ReturnCode<()> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_default_datareader_qos(&self, qos: Option<DataReaderQos>) -> ReturnCode<()> {
        let qos = qos.unwrap_or_default();
        qos.is_consistent()?;
        *self.value()?.default_datareader_qos.lock().unwrap() = qos;
        Ok(())
    }

    pub fn get_default_datareader_qos(&self) -> ReturnCode<DataReaderQos> {
        Ok(self.value()?.default_datareader_qos.lock().unwrap().clone())
    }

    pub fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_qos(&self) -> ReturnCode<SubscriberQos> {
        Ok(self.value()?.qos.clone())
    }

    pub fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        Ok(self.value()?.group.entity.guid.into())
    }
}
