use crate::types::{DDSType, ReturnCode, Duration};
use crate::dds_infrastructure::qos::{DataWriterQos, TopicQos};
use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriter;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;

#[derive(Default)]
pub struct RtpsPublisherInner;

pub type RtpsPublisher<'a> = RtpsObjectReference<'a, RtpsPublisherInner>;

impl<'a> RtpsPublisher<'a> {
    pub fn create_datawriter<T:DDSType>(&self) -> Option<RtpsDataWriter<T>> {
        todo!()
    }

    pub fn delete_datawriter<T: DDSType>(
        &self,
        _a_datawriter: &RtpsDataWriter<T>
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_datawriter<T: DDSType>(
        &self,
        _topic_name: &str,
    ) -> Option<RtpsDataWriter<T>> {
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

    pub fn wait_for_acknowledgments(
        &self,
        _max_wait: Duration
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn delete_contained_entities(&self) -> ReturnCode<()> {
        todo!()
    }

    pub fn set_default_datawriter_qos(
        &self,
        _qos: DataWriterQos,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn get_default_datawriter_qos (
        &self
    ) -> ReturnCode<DataWriterQos> {
        todo!()
    }

    pub fn copy_from_topic_qos(
        &self,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode<()> {
        todo!()
    }
}