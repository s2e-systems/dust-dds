use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::rtps::types::Guid,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::Time,
    },
};

use super::{
    dds_domain_participant::DdsDomainParticipant,
    nodes::{PublisherNode, SubscriberNode, TopicNode},
};

#[derive(PartialEq, Eq, Debug)]
pub struct DomainParticipantNode(Guid);

impl DomainParticipantNode {
    pub fn new(node: Guid) -> Self {
        Self(node)
    }

    pub fn guid(&self) -> Guid {
        self.0
    }

    pub fn create_publisher(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        qos: QosKind<PublisherQos>,
    ) -> DdsResult<PublisherNode> {
        domain_participant
            .create_publisher(qos)
            .map(|x| PublisherNode::new(x, self.0))
    }

    pub fn delete_publisher(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        publisher_guid: Guid,
    ) -> DdsResult<()> {
        domain_participant.delete_publisher(publisher_guid)
    }

    pub fn create_subscriber(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        qos: QosKind<SubscriberQos>,
    ) -> DdsResult<SubscriberNode> {
        domain_participant
            .create_subscriber(qos)
            .map(|x| SubscriberNode::new(x, self.0))
    }

    pub fn delete_subscriber(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        subscriber_guid: Guid,
    ) -> DdsResult<()> {
        domain_participant.delete_subscriber(subscriber_guid)
    }

    pub fn create_topic(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        topic_name: &str,
        type_name: &'static str,
        qos: QosKind<TopicQos>,
    ) -> DdsResult<TopicNode> {
        domain_participant
            .create_topic(topic_name, type_name, qos)
            .map(|x| TopicNode::new(x, self.0))
    }

    pub fn delete_topic(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        topic_guid: Guid,
    ) -> DdsResult<()> {
        domain_participant.delete_topic(topic_guid)
    }

    pub fn find_topic(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        topic_name: &str,
        type_name: &'static str,
    ) -> Option<TopicNode> {
        domain_participant
            .find_topic(topic_name, type_name)
            .map(|x| TopicNode::new(x, self.0))
    }

    pub fn lookup_topicdescription(
        &self,
        domain_participant: &DdsDomainParticipant,
        topic_name: &str,
        type_name: &str,
    ) -> DdsResult<Option<TopicNode>> {
        Ok(domain_participant
            .topic_list()
            .iter()
            .find(|topic| topic.get_name() == topic_name && topic.get_type_name() == type_name)
            .map(|x| TopicNode::new(x.guid(), self.0)))
    }

    pub fn get_builtin_subscriber(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<SubscriberNode> {
        let builtin_subcriber = Ok(domain_participant.get_builtin_subscriber())?;

        Ok(SubscriberNode::new(builtin_subcriber.guid(), self.0))
    }

    pub fn ignore_participant(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        handle: InstanceHandle,
    ) -> DdsResult<()> {
        domain_participant.ignore_participant(handle);
        Ok(())
    }

    pub fn ignore_topic(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        handle: InstanceHandle,
    ) -> DdsResult<()> {
        domain_participant.ignore_topic(handle);
        Ok(())
    }

    pub fn ignore_publication(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        handle: InstanceHandle,
    ) -> DdsResult<()> {
        if !domain_participant.is_enabled() {
            Err(DdsError::NotEnabled)
        } else {
            domain_participant.ignore_publication(handle);
            Ok(())
        }
    }

    pub fn ignore_subscription(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        handle: InstanceHandle,
    ) -> DdsResult<()> {
        domain_participant.ignore_subscription(handle);
        Ok(())
    }

    pub fn get_domain_id(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<DomainId> {
        Ok(domain_participant.get_domain_id())
    }

    pub fn delete_contained_entities(
        &self,
        domain_participant: &mut DdsDomainParticipant,
    ) -> DdsResult<()> {
        domain_participant.delete_contained_entities()
    }

    pub fn assert_liveliness(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<()> {
        domain_participant.assert_liveliness()
    }

    pub fn set_default_publisher_qos(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        qos: QosKind<PublisherQos>,
    ) -> DdsResult<()> {
        domain_participant.set_default_publisher_qos(qos)
    }

    pub fn get_default_publisher_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<PublisherQos> {
        Ok(domain_participant.get_default_publisher_qos())
    }

    pub fn set_default_subscriber_qos(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        qos: QosKind<SubscriberQos>,
    ) -> DdsResult<()> {
        domain_participant.set_default_subscriber_qos(qos)
    }

    pub fn get_default_subscriber_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<SubscriberQos> {
        Ok(domain_participant.get_default_subscriber_qos())
    }

    pub fn set_default_topic_qos(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        qos: QosKind<TopicQos>,
    ) -> DdsResult<()> {
        domain_participant.set_default_topic_qos(qos)
    }

    pub fn get_default_topic_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<TopicQos> {
        Ok(domain_participant.get_default_topic_qos())
    }

    pub fn get_discovered_participants(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<Vec<InstanceHandle>> {
        Ok(domain_participant
            .discovered_participant_list()
            .into_iter()
            .map(|(&key, _)| key)
            .collect())
    }

    pub fn get_discovered_participant_data(
        &self,
        domain_participant: &DdsDomainParticipant,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        Ok(domain_participant
            .discovered_participant_list()
            .into_iter()
            .find(|&(handle, _)| handle == &participant_handle)
            .ok_or(DdsError::BadParameter)?
            .1
            .dds_participant_data()
            .clone())
    }

    pub fn get_discovered_topics(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<Vec<InstanceHandle>> {
        domain_participant.get_discovered_topics()
    }

    pub fn get_discovered_topic_data(
        &self,
        domain_participant: &DdsDomainParticipant,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        domain_participant.get_discovered_topic_data(topic_handle)
    }

    pub fn contains_entity(
        &self,
        domain_participant: &DdsDomainParticipant,
        a_handle: InstanceHandle,
    ) -> DdsResult<bool> {
        domain_participant.contains_entity(a_handle)
    }

    pub fn get_current_time(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<Time> {
        Ok(domain_participant.get_current_time())
    }

    pub fn set_qos(
        &self,
        domain_participant: &mut DdsDomainParticipant,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        domain_participant.set_qos(qos)
    }

    pub fn get_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<DomainParticipantQos> {
        Ok(domain_participant.get_qos())
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn enable(&self, domain_participant: &mut DdsDomainParticipant) -> DdsResult<()> {
        domain_participant.enable()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.into())
    }
}
