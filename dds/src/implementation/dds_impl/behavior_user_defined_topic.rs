use crate::{
    implementation::rtps::types::Guid,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::InconsistentTopicStatus,
    },
};

use super::{dds_domain_participant::DdsDomainParticipant, nodes::DomainParticipantNode};

pub fn get_inconsistent_topic_status(
    domain_participant: &mut DdsDomainParticipant,
    topic_guid: Guid,
) -> DdsResult<InconsistentTopicStatus> {
    Ok(domain_participant
        .topic_list_mut()
        .iter_mut()
        .find(|t| t.guid() == topic_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_inconsistent_topic_status())
}

pub fn get_participant(participant_guid: Guid) -> DomainParticipantNode {
    DomainParticipantNode::new(participant_guid)
}

pub fn get_type_name(
    domain_participant: &DdsDomainParticipant,
    topic_guid: Guid,
) -> DdsResult<&'static str> {
    Ok(domain_participant
        .topic_list()
        .iter()
        .find(|t| t.guid() == topic_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_type_name())
}

pub fn get_name(domain_participant: &DdsDomainParticipant, topic_guid: Guid) -> DdsResult<String> {
    Ok(domain_participant
        .topic_list()
        .iter()
        .find(|t| t.guid() == topic_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_name())
}

pub fn set_qos(
    domain_participant: &mut DdsDomainParticipant,
    topic_guid: Guid,
    qos: QosKind<TopicQos>,
) -> DdsResult<()> {
    domain_participant
        .topic_list_mut()
        .iter_mut()
        .find(|t| t.guid() == topic_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .set_qos(qos)
}

pub fn get_qos(domain_participant: &DdsDomainParticipant, topic_guid: Guid) -> DdsResult<TopicQos> {
    Ok(domain_participant
        .topic_list()
        .iter()
        .find(|t| t.guid() == topic_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_qos())
}

pub fn enable(domain_participant: &mut DdsDomainParticipant, topic_guid: Guid) -> DdsResult<()> {
    // if !self.node.upgrade()?.get_participant().is_enabled() {
    //     return Err(DdsError::PreconditionNotMet(
    //         "Parent participant is disabled".to_string(),
    //     ));
    // }

    domain_participant
        .topic_list_mut()
        .iter_mut()
        .find(|t| t.guid() == topic_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .enable()?;

    Ok(())
}

pub fn get_instance_handle(topic_guid: Guid) -> DdsResult<InstanceHandle> {
    Ok(topic_guid.into())
}
