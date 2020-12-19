use crate::dds_infrastructure::qos::{DataReaderQos, SubscriberQos, TopicQos};
use crate::dds_infrastructure::status::SampleLostStatus;
use crate::dds_rtps_implementation::rtps_data_reader::{RtpsDataReader, RtpsDataReaderInner};
use crate::dds_rtps_implementation::rtps_object::{RtpsObject, RtpsObjectList};
use crate::rtps::structure::Entity;
use crate::rtps::types::constants::GUID_UNKNOWN;
use crate::types::ReturnCode;
use std::cell::Ref;

pub struct RtpsSubscriberInner {
    entity: Entity,
    reader_list: RtpsObjectList<RtpsDataReaderInner>,
    qos: SubscriberQos,
}

impl Default for RtpsSubscriberInner {
    fn default() -> Self {
        Self {
            entity: Entity { guid: GUID_UNKNOWN },
            reader_list: Default::default(),
            qos: SubscriberQos::default(),
        }
    }
}

pub type RtpsSubscriber<'a> = Ref<'a, RtpsObject<RtpsSubscriberInner>>;

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
}
