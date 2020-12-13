use crate::dds_infrastructure::qos::{DataWriterQos, TopicQos};
use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriter;
use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriterInner;
use crate::dds_rtps_implementation::rtps_object::RtpsObject;
use crate::types::{Duration, ReturnCode};
use std::cell::Ref;

#[derive(Default)]
pub struct RtpsPublisherInner {
    writer_list: [RtpsObject<RtpsDataWriterInner>; 32],
}

pub type RtpsPublisher<'a> = Ref<'a, RtpsObject<RtpsPublisherInner>>;

impl RtpsObject<RtpsPublisherInner> {
    pub fn create_datawriter(&self) -> Option<RtpsDataWriter> {
        // let datawriter_object = self
        //     .value()
        //     .ok()?
        //     .writer_list
        //     .iter()
        //     .find(|x| x.is_empty())?;
        // let new_datawriter_inner = RtpsDataWriterInner::new();
        // datawriter_object.initialize(new_datawriter_inner).ok()?;
        // datawriter_object
        //     .get_reference()
        //     .ok()
        todo!()
    }

    pub fn delete_datawriter(&self, _a_datawriter: &RtpsDataWriter) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_datawriter(&self, _topic_name: &str) -> Option<RtpsDataWriter> {
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
