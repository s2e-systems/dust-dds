use std::sync::Weak;

use crate::dds::types::{StatusKind, ReturnCode, Duration};
use crate::dds::domain::domain_participant::DomainParticipant;
use crate::dds::domain::domain_participant_impl::DomainParticipantImpl;
use crate::dds::topic::topic::Topic;
use crate::dds::topic::qos::TopicQos;
use crate::dds::publication::publisher::qos::PublisherQos;
use crate::dds::publication::data_writer_listener::DataWriterListener;
use crate::dds::publication::data_writer::DataWriter;
use crate::dds::publication::data_writer::qos::DataWriterQos;
use crate::dds::publication::publisher_listener::PublisherListener;
use crate::dds::infrastructure::entity::StatusCondition;
use crate::dds::types::InstanceHandle;
pub struct PublisherImpl{
    parent_participant: Weak<DomainParticipantImpl>,    
}

impl PublisherImpl {
    pub(crate) fn create_datawriter(
        _this: &Weak<PublisherImpl>,
        _a_topic: Topic,
        _qos: DataWriterQos,
        _a_listener: Box<dyn DataWriterListener>,
        _mask: &[StatusKind]
    ) -> DataWriter {
        todo!()
    }

    pub(crate) fn delete_datawriter(
        _this: &Weak<PublisherImpl>,
        _a_datawriter: DataWriter
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn lookup_datawriter(
        _this: &Weak<PublisherImpl>,
        _topic_name: String,
    ) -> DataWriter {
        todo!()
    }

    pub(crate) fn suspend_publications(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub(crate) fn resume_publications(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub(crate) fn begin_coherent_changes(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub(crate) fn end_coherent_changes(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub(crate) fn wait_for_acknowledgments(
        _this: &Weak<PublisherImpl>,
        _max_wait: Duration
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_participant(this: &Weak<PublisherImpl>) -> DomainParticipant {
        DomainParticipant(this.upgrade().unwrap().parent_participant.upgrade().unwrap())
    }

    pub(crate) fn delete_contained_entities(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub(crate) fn set_default_datawriter_qos(
        _this: &Weak<PublisherImpl>,
        _qos_list: DataWriterQos,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_default_datawriter_qos (
        _this: &Weak<PublisherImpl>,
        _qos_list: &mut DataWriterQos,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn copy_from_topic_qos(
        _this: &Weak<PublisherImpl>,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode {
        todo!()
    }

    ///////////////// Entity trait methods
    pub(crate) fn set_qos(_this: &Weak<PublisherImpl>, _qos_list: PublisherQos) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_qos(_this: &Weak<PublisherImpl>, _qos_list: &mut PublisherQos) -> ReturnCode {
        todo!()
    }

    pub(crate) fn set_listener(_this: &Weak<PublisherImpl>, _a_listener: Box<dyn PublisherListener>, _mask: &[StatusKind]) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_listener(_this: &Weak<PublisherImpl>, ) -> Box<dyn PublisherListener> {
        todo!()
    }

    pub(crate) fn get_statuscondition(_this: &Weak<PublisherImpl>, ) -> StatusCondition {
        todo!()
    }

    pub(crate) fn get_status_changes(_this: &Weak<PublisherImpl>, ) -> StatusKind {
        todo!()
    }

    pub(crate) fn enable(_this: &Weak<PublisherImpl>, ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_instance_handle(_this: &Weak<PublisherImpl>, ) -> InstanceHandle {
        todo!()
    }



    //////////////// From here on are the functions that do not belong to the standard API
    pub(crate) fn new(parent_participant: Weak<DomainParticipantImpl>
    ) -> Self {
        Self{
            parent_participant
        }
    }
}