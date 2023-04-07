use crate::{
    implementation::utils::node::{ChildNode, RootNode},
    infrastructure::{
        condition::StatusCondition,
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::{
    any_topic_listener::AnyTopicListener, domain_participant_impl::DomainParticipantImpl,
    node_domain_participant::DomainParticipantNode, topic_impl::TopicImpl,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedTopicNode(ChildNode<TopicImpl, RootNode<DomainParticipantImpl>>);

impl UserDefinedTopicNode {
    pub fn new(node: ChildNode<TopicImpl, RootNode<DomainParticipantImpl>>) -> Self {
        Self(node)
    }

    pub fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        Ok(self.0.get()?.get_inconsistent_topic_status())
    }

    pub fn get_participant(&self) -> DomainParticipantNode {
        DomainParticipantNode::new(self.0.parent().clone())
    }

    pub fn get_type_name(&self) -> DdsResult<&'static str> {
        Ok(self.0.get()?.get_type_name())
    }

    pub fn get_name(&self) -> DdsResult<String> {
        Ok(self.0.get()?.get_name())
    }

    pub fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        self.0.get()?.set_qos(qos)
    }

    pub fn get_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.0.get()?.get_qos())
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        Ok(StatusCondition::new(self.0.get()?.get_statuscondition()))
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self.0.get()?.get_status_changes())
    }

    pub fn enable(&self) -> DdsResult<()> {
        // if !self.node.upgrade()?.get_participant().is_enabled() {
        //     return Err(DdsError::PreconditionNotMet(
        //         "Parent participant is disabled".to_string(),
        //     ));
        // }

        self.0.get()?.enable()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.0.get()?.set_listener(a_listener, mask);
        Ok(())
    }
}
