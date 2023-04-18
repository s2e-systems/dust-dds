use crate::{
    builtin_topics::{ParticipantBuiltinTopicData, TopicBuiltinTopicData},
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::utils::node::{ChildNode, RootNode},
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
    any_topic_listener::AnyTopicListener, dcps_service::DcpsService,
    domain_participant_impl::DomainParticipantImpl, node_builtin_subscriber::BuiltinSubscriberNode,
    node_user_defined_publisher::UserDefinedPublisherNode,
    node_user_defined_subscriber::UserDefinedSubscriberNode,
    node_user_defined_topic::UserDefinedTopicNode, status_listener::StatusListener,
};

#[derive(PartialEq, Debug)]
pub struct DomainParticipantNode(ChildNode<DomainParticipantImpl, RootNode<DcpsService>>);

impl DomainParticipantNode {
    pub fn new(node: ChildNode<DomainParticipantImpl, RootNode<DcpsService>>) -> Self {
        Self(node)
    }

    pub fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedPublisherNode> {
        self.0
            .get()?
            .create_publisher(qos, a_listener, mask)
            .map(|x| UserDefinedPublisherNode::new(ChildNode::new(x.downgrade(), self.0.clone())))
    }

    pub fn delete_publisher(&self, publisher_handle: InstanceHandle) -> DdsResult<()> {
        self.0.get()?.delete_publisher(publisher_handle)
    }

    pub fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedSubscriberNode> {
        self.0
            .get()?
            .create_subscriber(qos, a_listener, mask)
            .map(|x| UserDefinedSubscriberNode::new(ChildNode::new(x.downgrade(), self.0.clone())))
    }

    pub fn delete_subscriber(&self, subscriber_handle: InstanceHandle) -> DdsResult<()> {
        self.0.get()?.delete_subscriber(subscriber_handle)
    }

    pub fn create_topic(
        &self,
        topic_name: &str,
        type_name: &'static str,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<UserDefinedTopicNode> {
        self.0
            .get()?
            .create_topic(topic_name, type_name, qos, a_listener, mask)
            .map(|x| UserDefinedTopicNode::new(ChildNode::new(x.downgrade(), self.0.clone())))
    }

    pub fn delete_topic(&self, topic_handle: InstanceHandle) -> DdsResult<()> {
        self.0.get()?.delete_topic(topic_handle)
    }

    pub fn find_topic(
        &self,
        topic_name: &str,
        type_name: &'static str,
        timeout: Duration,
    ) -> DdsResult<UserDefinedTopicNode> {
        self.0
            .get()?
            .find_topic(topic_name, type_name, timeout)
            .map(|x| UserDefinedTopicNode::new(ChildNode::new(x.downgrade(), self.0.clone())))
    }

    pub fn lookup_topicdescription(
        &self,
        topic_name: &str,
        type_name: &str,
    ) -> DdsResult<Option<UserDefinedTopicNode>> {
        Ok(self
            .0
            .get()?
            .topic_list()
            .into_iter()
            .find(|topic| topic.get_name() == topic_name && topic.get_type_name() == type_name)
            .map(|x| UserDefinedTopicNode::new(ChildNode::new(x.downgrade(), self.0.clone()))))
    }

    pub fn get_builtin_subscriber(&self) -> DdsResult<BuiltinSubscriberNode> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        let builtin_subcriber = self.0.get()?.get_builtin_subscriber();

        Ok(BuiltinSubscriberNode::new(ChildNode::new(
            builtin_subcriber.downgrade(),
            self.0.clone(),
        )))
    }

    pub fn ignore_participant(&self, handle: InstanceHandle) -> DdsResult<()> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        self.0.get()?.ignore_participant(handle);

        Ok(())
    }

    pub fn ignore_topic(&self, handle: InstanceHandle) -> DdsResult<()> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        self.0.get()?.ignore_topic(handle);

        Ok(())
    }

    pub fn ignore_publication(&self, handle: InstanceHandle) -> DdsResult<()> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        self.0.get()?.ignore_publication(handle);

        Ok(())
    }

    pub fn ignore_subscription(&self, handle: InstanceHandle) -> DdsResult<()> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        self.0.get()?.ignore_subscription(handle);

        Ok(())
    }

    pub fn get_domain_id(&self) -> DdsResult<DomainId> {
        Ok(self.0.get()?.get_domain_id())
    }

    pub fn delete_contained_entities(&self) -> DdsResult<()> {
        self.0.get()?.delete_contained_entities()
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        self.0.get()?.assert_liveliness()
    }

    pub fn set_default_publisher_qos(&self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        self.0.get()?.set_default_publisher_qos(qos)
    }

    pub fn get_default_publisher_qos(&self) -> DdsResult<PublisherQos> {
        Ok(self.0.get()?.get_default_publisher_qos())
    }

    pub fn set_default_subscriber_qos(&self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
        self.0.get()?.set_default_subscriber_qos(qos)
    }

    pub fn get_default_subscriber_qos(&self) -> DdsResult<SubscriberQos> {
        Ok(self.0.get()?.get_default_subscriber_qos())
    }

    pub fn set_default_topic_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.0.get()?.set_default_topic_qos(qos)
    }

    pub fn get_default_topic_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.0.get()?.get_default_topic_qos())
    }

    pub fn get_discovered_participants(&self) -> DdsResult<Vec<InstanceHandle>> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .0
            .get()?
            .discovered_participant_list()
            .into_iter()
            .map(|(&key, _)| key)
            .collect())
    }

    pub fn get_discovered_participant_data(
        &self,
        participant_handle: InstanceHandle,
    ) -> DdsResult<ParticipantBuiltinTopicData> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self
            .0
            .get()?
            .discovered_participant_list()
            .into_iter()
            .find(|&(handle, _)| handle == &participant_handle)
            .ok_or(DdsError::BadParameter)?
            .1
            .dds_participant_data
            .clone())
    }

    pub fn get_discovered_topics(&self) -> DdsResult<Vec<InstanceHandle>> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        self.0.get()?.get_discovered_topics()
    }

    pub fn get_discovered_topic_data(
        &self,
        topic_handle: InstanceHandle,
    ) -> DdsResult<TopicBuiltinTopicData> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        self.0.get()?.get_discovered_topic_data(topic_handle)
    }

    pub fn contains_entity(&self, a_handle: InstanceHandle) -> DdsResult<bool> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        self.0.get()?.contains_entity(a_handle)
    }

    pub fn get_current_time(&self) -> DdsResult<Time> {
        if !self.0.get()?.is_enabled() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.0.get()?.get_current_time())
    }

    pub fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        self.0.get()?.set_qos(qos)
    }

    pub fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        Ok(self.0.get()?.get_qos())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        *self.0.get()?.get_status_listener_lock() = StatusListener::new(a_listener, mask);
        Ok(())
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        Ok(StatusCondition::new(self.0.get()?.get_statuscondition()))
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self.0.get()?.get_status_changes())
    }

    pub fn enable(&self) -> DdsResult<()> {
        self.0.get()?.enable()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.guid().into())
    }
}
