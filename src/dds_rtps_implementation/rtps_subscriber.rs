use crate::dds_rtps_implementation::rtps_data_reader::RtpsDataReader;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;
use crate::types::{DDSType, ReturnCode};
use crate::dds_infrastructure::status::SampleLostStatus;
use crate::dds_infrastructure::qos::{DataReaderQos, TopicQos};

#[derive(Default)]
pub struct RtpsSubscriberInner;

pub type RtpsSubscriber<'a> = RtpsObjectReference<'a, RtpsSubscriberInner>;

impl<'a> RtpsSubscriber<'a> {
    pub fn create_datareader<T: DDSType>(&self) -> Option<RtpsDataReader<T>> {
        todo!()
    }

    pub fn delete_datareader<T: DDSType>(&self, _a_datareader: &RtpsDataReader<T>) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_datareader<T: DDSType>(
        &self,
        _topic_name: &str
    ) -> Option<RtpsDataReader<T>> {
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

    pub fn set_default_datareader_qos(
        &self,
        _qos: DataReaderQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_default_datareader_qos(
        &self
    ) -> ReturnCode<DataReaderQos> {
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
