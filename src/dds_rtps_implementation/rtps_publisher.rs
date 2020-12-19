use crate::dds_infrastructure::qos::{DataWriterQos, PublisherQos, TopicQos};
use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriter;
use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriterInner;
use crate::dds_rtps_implementation::rtps_object::{RtpsObject, RtpsObjectList};
use crate::rtps::structure::Entity;
use crate::rtps::types::GUID;
use crate::rtps::types::constants::GUID_UNKNOWN;
use crate::types::{Duration, ReturnCode};
use std::cell::Ref;

pub struct RtpsPublisherInner {
    entity: Entity,
    writer_list: RtpsObjectList<RtpsDataWriterInner>,
    qos: PublisherQos,
}

impl Default for RtpsPublisherInner{
    fn default() -> Self {
        Self {
            entity: Entity{guid: GUID_UNKNOWN},
            writer_list: Default::default(),
            qos: PublisherQos::default(),
        }
    }
}

impl RtpsPublisherInner {
    pub fn new(guid: GUID, qos: PublisherQos) -> Self {
        Self {
            entity: Entity{guid},
            writer_list: Default::default(),
            qos
        }
    }
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
