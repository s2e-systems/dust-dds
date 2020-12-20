use crate::dds_infrastructure::qos::{DataReaderQos, SubscriberQos, TopicQos};
use crate::dds_infrastructure::status::SampleLostStatus;
use crate::dds_rtps_implementation::rtps_data_reader::{RtpsDataReader, RtpsDataReaderInner};
use crate::dds_rtps_implementation::rtps_object::{RtpsObject, RtpsObjectList};
use crate::rtps::structure::Group;
use crate::rtps::types::GUID;
use crate::types::{ReturnCode, InstanceHandle};
use std::sync::RwLockReadGuard;
use std::sync::{atomic, Mutex};

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
    pub fn create_datareader(&self) -> Option<RtpsDataReader> {
        todo!()
    }

    pub fn delete_datareader(&self, _a_datareader: &RtpsDataReader) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_datareader(&self, _topic_name: &str) -> Option<RtpsDataReader> {
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

    pub fn set_default_datareader_qos(&self, _qos: DataReaderQos) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_default_datareader_qos(&self) -> ReturnCode<DataReaderQos> {
        todo!()
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
