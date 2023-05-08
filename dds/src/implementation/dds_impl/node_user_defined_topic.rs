use crate::{
    implementation::rtps::types::Guid,
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::{
    any_topic_listener::AnyTopicListener, dds_domain_participant::DdsDomainParticipant,
    node_domain_participant::DomainParticipantNode,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedTopicNode {
    this: Guid,
    parent: Guid,
}

impl UserDefinedTopicNode {
    pub fn new(this: Guid, parent: Guid) -> Self {
        Self { this, parent }
    }

    pub fn guid(&self) -> Guid {
        self.this
    }

    pub fn get_inconsistent_topic_status(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<InconsistentTopicStatus> {
        Ok(domain_participant
            .topic_list()
            .iter()
            .find(|t| t.guid() == self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_inconsistent_topic_status())
    }

    pub fn get_participant(&self) -> DomainParticipantNode {
        DomainParticipantNode::new(self.parent)
    }

    pub fn get_type_name(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<&'static str> {
        Ok(domain_participant
            .topic_list()
            .iter()
            .find(|t| t.guid() == self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_type_name())
    }

    pub fn get_name(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<String> {
        Ok(domain_participant
            .topic_list()
            .iter()
            .find(|t| t.guid() == self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_name())
    }

    pub fn set_qos(
        &self,
        domain_participant: &DdsDomainParticipant,
        qos: QosKind<TopicQos>,
    ) -> DdsResult<()> {
        domain_participant
            .topic_list()
            .iter()
            .find(|t| t.guid() == self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .set_qos(qos)
    }

    pub fn get_qos(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<TopicQos> {
        Ok(domain_participant
            .topic_list()
            .iter()
            .find(|t| t.guid() == self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos())
    }

    pub fn get_statuscondition(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<StatusCondition> {
        Ok(StatusCondition::new(
            domain_participant
                .topic_list()
                .iter()
                .find(|t| t.guid() == self.this)
                .ok_or(DdsError::AlreadyDeleted)?
                .get_statuscondition(),
        ))
    }

    pub fn get_status_changes(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<Vec<StatusKind>> {
        Ok(domain_participant
            .topic_list()
            .iter()
            .find(|t| t.guid() == self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_status_changes())
    }

    pub fn enable(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<()> {
        // if !self.node.upgrade()?.get_participant().is_enabled() {
        //     return Err(DdsError::PreconditionNotMet(
        //         "Parent participant is disabled".to_string(),
        //     ));
        // }

        domain_participant
            .topic_list()
            .iter()
            .find(|t| t.guid() == self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .enable()?;

        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.this.into())
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        todo!()
        // self.0.get()?.set_listener(a_listener, mask);
        // Ok(())
    }
}
