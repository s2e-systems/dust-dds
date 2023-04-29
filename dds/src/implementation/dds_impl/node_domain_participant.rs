use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{rtps::types::Guid, utils::node::ChildNode},
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
        time::{Duration, Time},
    },
    publication::publisher_listener::PublisherListener,
    subscription::subscriber_listener::SubscriberListener,
};

use super::{
    any_topic_listener::AnyTopicListener, dds_domain_participant::DdsDomainParticipant,
    dds_domain_participant_factory::THE_DDS_DOMAIN_PARTICIPANT_FACTORY,
    node_builtin_subscriber::BuiltinSubscriberNode,
    node_user_defined_publisher::UserDefinedPublisherNode,
    node_user_defined_subscriber::UserDefinedSubscriberNode,
    node_user_defined_topic::UserDefinedTopicNode,
};

#[derive(PartialEq, Debug)]
pub struct DomainParticipantNode(pub(crate) Guid);

impl DomainParticipantNode {
    pub fn new(node: Guid) -> Self {
        Self(node)
    }

    pub fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedPublisherNode> {
        self.call_participant_method(move |dp| {
            dp.create_publisher(qos, a_listener, mask).map(|x| {
                UserDefinedPublisherNode::new(ChildNode::new(x.downgrade(), self.0))
            })
        })
    }

    pub fn delete_publisher(&self, publisher_handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.delete_publisher(publisher_handle))
    }

    pub fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedSubscriberNode> {
        self.call_participant_method(|dp| {
            dp.create_subscriber(qos, a_listener, mask).map(|x| {
                UserDefinedSubscriberNode::new(ChildNode::new(x.downgrade(), self.0))
            })
        })
    }

    pub fn delete_subscriber(&self, subscriber_handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.delete_subscriber(subscriber_handle))
    }

    pub fn create_topic(
        &self,
        topic_name: &str,
        type_name: &'static str,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedTopicNode> {
        self.call_participant_method(|dp| {
            dp.create_topic(topic_name, type_name, qos, a_listener, mask)
                .map(|x| UserDefinedTopicNode::new(ChildNode::new(x.downgrade(), self.0)))
        })
    }

    pub fn delete_topic(&self, topic_handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.delete_topic(topic_handle))
    }

    pub fn find_topic(
        &self,
        topic_name: &str,
        type_name: &'static str,
        timeout: Duration,
    ) -> DdsResult<UserDefinedTopicNode> {
        self.call_participant_method(|dp| {
            dp.find_topic(topic_name, type_name, timeout)
                .map(|x| UserDefinedTopicNode::new(ChildNode::new(x.downgrade(), self.0)))
        })
    }

    pub fn lookup_topicdescription(
        &self,
        topic_name: &str,
        type_name: &str,
    ) -> DdsResult<Option<UserDefinedTopicNode>> {
        self.call_participant_method(|dp| {
            Ok(dp
                .topic_list()
                .into_iter()
                .find(|topic| topic.get_name() == topic_name && topic.get_type_name() == type_name)
                .map(|x| UserDefinedTopicNode::new(ChildNode::new(x.downgrade(), self.0))))
        })
    }

    pub fn get_builtin_subscriber(&self) -> DdsResult<BuiltinSubscriberNode> {
        let builtin_subcriber =
            self.call_participant_method_if_enabled(|dp| Ok(dp.get_builtin_subscriber()))?;

        Ok(BuiltinSubscriberNode::new(ChildNode::new(
            builtin_subcriber.downgrade(),
            self.0,
        )))
    }

    pub fn ignore_participant(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_method_if_enabled(|dp| {
            dp.ignore_participant(handle);
            Ok(())
        })
    }

    pub fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_method_if_enabled(|dp| {
            dp.ignore_topic(handle);
            Ok(())
        })
    }

    pub fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_method_if_enabled(|dp| {
            dp.ignore_publication(handle);
            Ok(())
        })
    }

    pub fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        self.call_participant_method_if_enabled(|dp| {
            dp.ignore_subscription(handle);
            Ok(())
        })
    }

    pub fn get_domain_id(&self) -> DdsResult<DomainId> {
        self.call_participant_method(|dp| Ok(dp.get_domain_id()))
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.delete_contained_entities())
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        self.call_participant_method_if_enabled(|dp| dp.assert_liveliness())
    }

    pub fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.set_default_publisher_qos(qos))
    }

    pub fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        self.call_participant_method(|dp| Ok(dp.get_default_publisher_qos()))
    }

    pub fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.set_default_subscriber_qos(qos))
    }

    pub fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        self.call_participant_method(|dp| Ok(dp.get_default_subscriber_qos()))
    }

    pub fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.set_default_topic_qos(qos))
    }

    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        self.call_participant_method(|dp| Ok(dp.get_default_topic_qos()))
    }

    pub fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.call_participant_method_if_enabled(|dp| {
            Ok(dp
                .discovered_participant_list()
                .into_iter()
                .map(|(&key, _)| key)
                .collect())
        })
    }

    pub fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        self.call_participant_method_if_enabled(|dp| {
            Ok(dp
                .discovered_participant_list()
                .into_iter()
                .find(|&(handle, _)| handle == &participant_handle)
                .ok_or(DdsError::BadParameter)?
                .1
                .dds_participant_data
                .clone())
        })
    }

    pub fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.call_participant_method_if_enabled(|dp| dp.get_discovered_topics())
    }

    pub fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        self.call_participant_method_if_enabled(|dp| dp.get_discovered_topic_data(topic_handle))
    }

    pub fn contains_entity(&self, a_handle: InstanceHandle) -> DdsResult<bool> {
        self.call_participant_method_if_enabled(|dp| dp.contains_entity(a_handle))
    }

    pub fn get_current_time(&self) -> DdsResult<Time> {
        self.call_participant_method_if_enabled(|dp| Ok(dp.get_current_time()))
    }

    pub fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.set_qos(qos))
    }

    pub fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.call_participant_method(|dp| Ok(dp.get_qos()))
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.call_participant_method(|dp| Ok(StatusCondition::new(dp.get_statuscondition())))
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        self.call_participant_method(|dp| Ok(dp.get_status_changes()))
    }

    pub fn enable(&self) -> DdsResult<()> {
        self.call_participant_method(|dp| dp.enable())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.into())
    }

    fn call_participant_method<F, O>(&self, f: F) -> DdsResult<O>
    where
        F: FnOnce(&DdsDomainParticipant) -> DdsResult<O>,
    {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .domain_participant_list()
            .get_participant(&self.0, |dp| f(dp.ok_or(DdsError::AlreadyDeleted)?))
    }

    fn call_participant_method_if_enabled<F, O>(&self, f: F) -> DdsResult<O>
    where
        F: FnOnce(&DdsDomainParticipant) -> DdsResult<O>,
    {
        THE_DDS_DOMAIN_PARTICIPANT_FACTORY
            .domain_participant_list()
            .get_participant(&self.0, |dp| {
                let dp = dp.ok_or(DdsError::AlreadyDeleted)?;
                if dp.is_enabled() {
                    f(dp)
                } else {
                    Err(DdsError::NotEnabled)
                }
            })
    }
}
