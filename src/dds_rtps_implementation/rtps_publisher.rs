use crate::types::{DDSType, Duration, InstanceHandle, ReturnCode};

use crate::dds_rtps_implementation::rtps_data_writer::RtpsDataWriter;
use crate::dds_rtps_implementation::rtps_object::RtpsObjectReference;

use crate::dds_infrastructure::entity::{Entity, StatusCondition};
use crate::dds_infrastructure::publisher_listener::PublisherListener;
use crate::dds_infrastructure::qos::{DataWriterQos, PublisherQos, TopicQos};
use crate::dds_infrastructure::status::StatusMask;

#[derive(Default)]
pub struct RtpsPublisherInner;

pub type RtpsPublisher<'a> = RtpsObjectReference<'a, RtpsPublisherInner>;

impl<'a> RtpsPublisher<'a> {
    pub fn create_datawriter<T: DDSType>(&self) -> Option<RtpsDataWriter<T>> {
        todo!()
    }

    pub fn delete_datawriter<T: DDSType>(
        &self,
        _a_datawriter: &RtpsDataWriter<T>,
    ) -> ReturnCode<()> {
        todo!()
    }

    pub fn lookup_datawriter<T: DDSType>(&self, _topic_name: &str) -> Option<RtpsDataWriter<T>> {
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

impl<'a> Entity for RtpsPublisher<'a> {
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener>;

    fn set_qos(&self, _qos: Self::Qos) -> ReturnCode<()> {
        todo!()
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        todo!()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self) -> &Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        todo!()
    }
}
