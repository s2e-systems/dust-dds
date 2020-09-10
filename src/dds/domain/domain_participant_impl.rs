use std::any::Any;

use crate::dds::types::{StatusKind, StatusMask, ReturnCode, Duration, InstanceHandle, DomainId, Time};
use crate::dds::topic::topic::Topic;
use crate::dds::topic::topic_listener::TopicListener;
use crate::dds::topic::topic_description::TopicDescription;
use crate::dds::subscription::subscriber::Subscriber;
use crate::dds::subscription::subscriber_listener::SubscriberListener;
use crate::dds::publication::publisher::Publisher;
use crate::dds::publication::publisher_listener::PublisherListener;
use crate::dds::infrastructure::qos_policy::QosPolicy;
use crate::dds::infrastructure::entity::Entity;
use crate::dds::domain::domain_participant_listener::{DomainParticipantListener, NoListener};

use super::domain_participant::qos::DomainParticipantQos;
use super::domain_participant::TopicBuiltinTopicData;
use super::domain_participant::ParticipantBuiltinTopicData;

pub struct DomainParticipantImpl{
    domain_id: DomainId,
    qos: DomainParticipantQos,
    a_listener: Box<dyn DomainParticipantListener>,
    mask: StatusMask
}

impl DomainParticipantImpl{
    pub fn new(
        domain_id: DomainId,
        qos: DomainParticipantQos,
        a_listener: impl DomainParticipantListener,
        mask: StatusMask,
    ) -> Self {
        
        if !Any::is::<NoListener>(&a_listener) {
            println!("TODO: Use the real listener")
        }

        Self {
            domain_id,
            qos,
            a_listener: Box::new(a_listener),
            mask,
        }
    }


    /////////// From here down is the copy of interface of DomainParticipant  /////////////

    pub fn create_publisher(
        &self,
        _qos_list: &[&dyn QosPolicy],
        _a_listener: Box<dyn PublisherListener>,
        _mask: &[StatusKind]
    ) -> Publisher {
        todo!()
    }

    pub fn delete_publisher(
        &self,
        _a_publisher: Publisher
    ) -> ReturnCode {
        todo!()
    }

    pub fn create_subscriber(
        &self,
        _qos_list: &[&dyn QosPolicy],
        _a_listener: Box<dyn SubscriberListener>,
        _mask: &[StatusKind]
    ) -> Subscriber {
        todo!()
    }

    pub fn delete_subscriber(
        &self,
        _a_subscriber: Subscriber,
    ) -> ReturnCode {
        todo!()
    }

    pub fn create_topic(
        &self,
        _topic_name: String,
        _type_name: String,
        _qos_list: &[&dyn QosPolicy],
        _a_listener: Box<dyn TopicListener>,
        _mask: &[StatusKind]
    ) -> Topic {
        todo!()
    }

    pub fn delete_topic(
        &self,
        _a_topic: Topic,
    ) -> ReturnCode {
        todo!()
    }

    pub fn find_topic(
        &self,
        _topic_name: String,
        _timeout: Duration,
    ) -> Topic {
        todo!()
    }

    pub fn lookup_topicdescription(
        &self,
        _name: String,
    ) -> TopicDescription {
        todo!()
    }

    pub fn get_builtin_subscriber(&self,) -> Subscriber {
        todo!()
    }

    pub fn ignore_participant(
        &self,
        _handle: InstanceHandle
    ) -> ReturnCode{
        todo!()
    }

    pub fn ignore_topic(
        &self,
        _handle: InstanceHandle
    ) -> ReturnCode{
        todo!()
    }

    pub fn ignore_publication(
        &self,
        _handle: InstanceHandle
    ) -> ReturnCode{
        todo!()
    }

    pub fn ignore_subscription(
        &self,
        _handle: InstanceHandle
    ) -> ReturnCode{
        todo!()
    }

    pub fn get_domain_id(&self,) -> DomainId {
        todo!()
    }

    pub fn delete_contained_entities(&self,) -> ReturnCode {
        todo!()   
    }

    pub fn assert_liveliness(&self,) -> ReturnCode {
        todo!()   
    }

    pub fn set_default_publisher_qos(
        &self,
        _qos_list: &[&dyn QosPolicy],
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_default_publisher_qos(
        &self,
        _qos_list: &mut [&dyn QosPolicy],
    ) -> ReturnCode {
        todo!()
    }

    pub fn set_default_subscriber_qos(
        &self,
        _qos_list: &[&dyn QosPolicy],
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_default_subscriber_qos(
        &self,
        _qos_list: &mut [&dyn QosPolicy],
    ) -> ReturnCode {
        todo!()
    }

    pub fn set_default_topic_qos(
        &self,
        _qos_list: &[&dyn QosPolicy],
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_default_topic_qos(
        &self,
        _qos_list: &[&dyn QosPolicy],
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_discovered_participants(
        &self,
        _participant_handles: &mut [InstanceHandle]
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_discovered_participant_data(
        &self,
        _participant_data: ParticipantBuiltinTopicData,
        _participant_handle: InstanceHandle
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_discovered_topics(
        &self,
        _topic_handles: &mut [InstanceHandle]
    ) -> ReturnCode {
        todo!()
    }

    pub fn get_discovered_topic_data(
        &self,
        _topic_data: TopicBuiltinTopicData,
        _topic_handle: InstanceHandle
    ) -> ReturnCode {
        todo!()
    }

    pub fn contains_entity(
        &self,
        _a_handle: InstanceHandle
    ) -> bool {
        todo!()
    }

    pub fn get_current_time(
        &self,
        _current_time: Time,
    ) -> ReturnCode {
        todo!()
    }
}

impl Entity for DomainParticipantImpl
{
    type Listener = Box<dyn DomainParticipantListener>;

    fn set_qos(&self, _qos_list: &[&dyn QosPolicy]) -> ReturnCode {
        todo!()
    }

    fn get_qos(&self, _qos_list: &mut [&dyn QosPolicy]) -> ReturnCode {
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

    fn get_instance_handle(&self, ) -> InstanceHandle {
        todo!()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::dds::infrastructure::qos_policy::{UserDataQosPolicy, EntityFactoryQosPolicy};
// }