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
use crate::dds::infrastructure::entity::Entity;
use crate::dds::infrastructure::entity::DomainEntity;
use crate::dds::publication::publisher_listener::PublisherListener;
pub struct PublisherImpl{
    parent_participant: Weak<DomainParticipantImpl>,    
}

impl PublisherImpl {
    pub fn create_datawriter(
        _this: &Weak<PublisherImpl>,
        _a_topic: Topic,
        _qos: DataWriterQos,
        _a_listener: Box<dyn DataWriterListener>,
        _mask: &[StatusKind]
    ) -> DataWriter {
        todo!()
    }

    pub fn delete_datawriter(
        _this: &Weak<PublisherImpl>,
        _a_datawriter: DataWriter
    ) -> ReturnCode {
        todo!()
    }

    pub fn lookup_datawriter(
        _this: &Weak<PublisherImpl>,
        _topic_name: String,
    ) -> DataWriter {
        todo!()
    }

    pub fn suspend_publications(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub fn resume_publications(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub fn begin_coherent_changes(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub fn end_coherent_changes(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub fn wait_for_acknowledgments(
        _this: &Weak<PublisherImpl>,
        _max_wait: Duration
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_participant(this: &Weak<PublisherImpl>) -> DomainParticipant {
        DomainParticipant(this.upgrade().unwrap().parent_participant.upgrade().unwrap())
    }

    pub fn delete_contained_entities(_this: &Weak<PublisherImpl>) -> ReturnCode {
        todo!()
    }

    pub fn set_default_datawriter_qos(
        _this: &Weak<PublisherImpl>,
        _qos_list: DataWriterQos,
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_default_datawriter_qos (
        _this: &Weak<PublisherImpl>,
        _qos_list: &mut DataWriterQos,
    ) -> ReturnCode {
        todo!()
    }

    pub fn copy_from_topic_qos(
        _this: &Weak<PublisherImpl>,
        _a_datawriter_qos: &mut DataWriterQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode {
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

impl Entity for PublisherImpl{
    type Qos = PublisherQos;
    type Listener = Box<dyn PublisherListener>;

    fn set_qos(&self, _qos_list: Self::Qos) -> ReturnCode {
        todo!()
    }

    fn get_qos(&self, _qos_list: &mut Self::Qos) -> ReturnCode {
        todo!()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: &[StatusKind]) -> ReturnCode {
        todo!()
    }

    fn get_listener(&self, ) -> Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self, ) -> crate::dds::infrastructure::entity::StatusCondition {
        todo!()
    }

    fn get_status_changes(&self, ) -> StatusKind {
        todo!()
    }

    fn enable(&self, ) -> ReturnCode {
        todo!()
    }

    fn get_instance_handle(&self, ) -> crate::dds::types::InstanceHandle {
        todo!()
    }
}

impl DomainEntity for PublisherImpl{}