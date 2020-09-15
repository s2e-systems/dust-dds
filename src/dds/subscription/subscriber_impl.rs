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
use crate::dds::infrastructure::entity::Entity;
use crate::dds::infrastructure::entity::DomainEntity;
use crate::dds::subscription::subscriber_listener::SubscriberListener;
use crate::dds::subscription::subscriber::qos::SubscriberQos;

pub struct SubscriberImpl{
    parent_participant: Weak<DomainParticipantImpl>,
}

impl SubscriberImpl {
    pub fn create_datareader(
        _this: &Weak<SubscriberImpl>,
        _a_topic: &dyn TopicDescription,
        _qos: DataReaderQos,
        _a_listener: Box<dyn DataReaderListener>,
        _mask: &[StatusKind]
    ) -> DataReader {
        todo!()
    }

    pub fn delete_datareader(
        _this: &Weak<SubscriberImpl>,
        _a_datareader: DataReader
    ) -> ReturnCode {
        todo!()
    }

    pub fn lookup_datareader(
        _this: &Weak<SubscriberImpl>,
        _topic_name: String
    ) -> DataReader {
        todo!()
    }

    pub fn begin_access(
        _this: &Weak<SubscriberImpl>,
    ) -> ReturnCode {
        todo!()
    }

    pub fn end_access(
        _this: &Weak<SubscriberImpl>,
    ) -> ReturnCode {
        todo!()
    }

   
    pub fn get_datareaders(
        _this: &Weak<SubscriberImpl>,
        _readers: &mut [DataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> ReturnCode {
        todo!()
    }

    pub fn notify_datareaders(
        _this: &Weak<SubscriberImpl>,
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_sample_lost_status(
        _this: &Weak<SubscriberImpl>,
        _status: &mut SampleLostStatus) -> ReturnCode {
        todo!()
    }

    pub fn get_participant(
        this: &Weak<SubscriberImpl>,
    ) -> DomainParticipant {
        DomainParticipant(this.upgrade().unwrap().parent_participant.upgrade().unwrap())
    }

    pub fn delete_contained_entities(
        _this: &Weak<SubscriberImpl>,
    ) -> ReturnCode {
        todo!()
    }

    pub fn set_default_datareader_qos(
        _this: &Weak<SubscriberImpl>,
        _qos_list: DataReaderQos,
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_default_datareader_qos(
        _this: &Weak<SubscriberImpl>,
        _qos_list: &mut DataReaderQos,
    ) -> ReturnCode {
        todo!()
    }

    pub fn copy_from_topic_qos(
        _this: &Weak<SubscriberImpl>,
        _a_datareader_qos: &mut DataReaderQos,
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

impl Entity for SubscriberImpl {
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener>;

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

impl DomainEntity for SubscriberImpl{}