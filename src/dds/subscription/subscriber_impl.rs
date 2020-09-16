use std::sync::Weak;

use crate::dds::types::{
    ReturnCode,
    StatusKind,
    SampleStateKind,
    ViewStateKind,
    InstanceStateKind,
};

use crate::dds::infrastructure::status::SampleLostStatus;
use crate::dds::domain::domain_participant::DomainParticipant;
use crate::dds::domain::domain_participant_impl::DomainParticipantImpl;
use crate::dds::topic::topic_description::TopicDescription;
use crate::dds::topic::qos::TopicQos;
use crate::dds::subscription::data_reader::DataReader;
use crate::dds::subscription::data_reader_listener::DataReaderListener;
use crate::dds::subscription::data_reader::qos::DataReaderQos;
use crate::dds::subscription::subscriber_listener::SubscriberListener;
use crate::dds::subscription::subscriber::qos::SubscriberQos;

pub struct SubscriberImpl{
    parent_participant: Weak<DomainParticipantImpl>,
}

impl SubscriberImpl {
    pub(crate) fn create_datareader(
        _this: &Weak<SubscriberImpl>,
        _a_topic: &dyn TopicDescription,
        _qos: DataReaderQos,
        _a_listener: Box<dyn DataReaderListener>,
        _mask: &[StatusKind]
    ) -> DataReader {
        todo!()
    }

    pub(crate) fn delete_datareader(
        _this: &Weak<SubscriberImpl>,
        _a_datareader: DataReader
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn lookup_datareader(
        _this: &Weak<SubscriberImpl>,
        _topic_name: String
    ) -> DataReader {
        todo!()
    }

    pub(crate) fn begin_access(
        _this: &Weak<SubscriberImpl>,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn end_access(
        _this: &Weak<SubscriberImpl>,
    ) -> ReturnCode {
        todo!()
    }

   
    pub(crate) fn get_datareaders(
        _this: &Weak<SubscriberImpl>,
        _readers: &mut [DataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn notify_datareaders(
        _this: &Weak<SubscriberImpl>,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_sample_lost_status(
        _this: &Weak<SubscriberImpl>,
        _status: &mut SampleLostStatus) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_participant(
        this: &Weak<SubscriberImpl>,
    ) -> DomainParticipant {
        DomainParticipant(this.upgrade().unwrap().parent_participant.upgrade().unwrap())
    }

    pub(crate) fn delete_contained_entities(
        _this: &Weak<SubscriberImpl>,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn set_default_datareader_qos(
        _this: &Weak<SubscriberImpl>,
        _qos_list: DataReaderQos,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_default_datareader_qos(
        _this: &Weak<SubscriberImpl>,
        _qos_list: &mut DataReaderQos,
    ) -> ReturnCode {
        todo!()
    }

    pub(crate) fn copy_from_topic_qos(
        _this: &Weak<SubscriberImpl>,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> ReturnCode {
        todo!()
    }

    //////////////// Entity trait methods
    pub(crate) fn set_qos(_this: &Weak<SubscriberImpl>, _qos_list: SubscriberQos) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_qos(_this: &Weak<SubscriberImpl>, _qos_list: &mut SubscriberQos) -> ReturnCode {
        todo!()
    }

    pub(crate) fn set_listener(_this: &Weak<SubscriberImpl>, _a_listener: Box<dyn SubscriberListener>, _mask: &[StatusKind]) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_listener(_this: &Weak<SubscriberImpl>) -> Box<dyn SubscriberListener> {
        todo!()
    }

    pub(crate) fn get_statuscondition(_this: &Weak<SubscriberImpl>) -> crate::dds::infrastructure::entity::StatusCondition {
        todo!()
    }

    pub(crate) fn get_status_changes(_this: &Weak<SubscriberImpl>) -> StatusKind {
        todo!()
    }

    pub(crate) fn enable(_this: &Weak<SubscriberImpl>) -> ReturnCode {
        todo!()
    }

    pub(crate) fn get_instance_handle(_this: &Weak<SubscriberImpl>) -> crate::dds::types::InstanceHandle {
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