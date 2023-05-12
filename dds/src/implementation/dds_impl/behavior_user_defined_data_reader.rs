use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::rtps::types::Guid,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
        },
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    dds_domain_participant::{AnnounceKind, DdsDomainParticipant},
    nodes::{SubscriberNode, TopicNode},
};

pub fn read<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
    max_samples: i32,
    sample_states: &[SampleStateKind],
    view_states: &[ViewStateKind],
    instance_states: &[InstanceStateKind],
    specific_instance_handle: Option<InstanceHandle>,
) -> DdsResult<Vec<Sample<Foo>>>
where
    Foo: for<'de> DdsDeserialize<'de>,
{
    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
}

pub fn take<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
    max_samples: i32,
    sample_states: &[SampleStateKind],
    view_states: &[ViewStateKind],
    instance_states: &[InstanceStateKind],
    specific_instance_handle: Option<InstanceHandle>,
) -> DdsResult<Vec<Sample<Foo>>>
where
    Foo: for<'de> DdsDeserialize<'de>,
{
    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .take(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
}

pub fn read_next_instance<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
    max_samples: i32,
    previous_handle: Option<InstanceHandle>,
    sample_states: &[SampleStateKind],
    view_states: &[ViewStateKind],
    instance_states: &[InstanceStateKind],
) -> DdsResult<Vec<Sample<Foo>>>
where
    Foo: for<'de> DdsDeserialize<'de>,
{
    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .read_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
}

pub fn take_next_instance<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
    max_samples: i32,
    previous_handle: Option<InstanceHandle>,
    sample_states: &[SampleStateKind],
    view_states: &[ViewStateKind],
    instance_states: &[InstanceStateKind],
) -> DdsResult<Vec<Sample<Foo>>>
where
    Foo: for<'de> DdsDeserialize<'de>,
{
    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .take_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
}

pub fn get_key_value<Foo>(
    domain_participant: &DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
    key_holder: &mut Foo,
    handle: InstanceHandle,
) -> DdsResult<()> {
    domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_key_value(key_holder, handle)
}

pub fn lookup_instance<Foo>(
    domain_participant: &DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
    instance: &Foo,
) -> DdsResult<Option<InstanceHandle>> {
    domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .lookup_instance(instance)
}

pub fn get_liveliness_changed_status(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<LivelinessChangedStatus> {
    Ok(domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_liveliness_changed_status())
}

pub fn get_requested_deadline_missed_status(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<RequestedDeadlineMissedStatus> {
    Ok(domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_requested_deadline_missed_status())
}

pub fn get_requested_incompatible_qos_status(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<RequestedIncompatibleQosStatus> {
    Ok(domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_requested_incompatible_qos_status())
}

pub fn get_sample_lost_status(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<SampleLostStatus> {
    Ok(domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_sample_lost_status())
}

pub fn get_sample_rejected_status(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<SampleRejectedStatus> {
    Ok(domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_sample_rejected_status())
}

pub fn get_subscription_matched_status(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<SubscriptionMatchedStatus> {
    Ok(domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_subscription_matched_status())
}

pub fn get_topicdescription(
    domain_participant: &DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<TopicNode> {
    let data_reader = domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?;
    let topic = domain_participant
        .topic_list()
        .iter()
        .find(|t| {
            t.get_name() == data_reader.get_topic_name()
                && t.get_type_name() == data_reader.get_type_name()
        })
        .expect("Topic must exist");

    Ok(TopicNode::new(topic.guid(), domain_participant.guid()))
}

pub fn get_subscriber(
    domain_participant: &DdsDomainParticipant,
    subscriber_guid: Guid,
) -> SubscriberNode {
    SubscriberNode::new(subscriber_guid, domain_participant.guid())
}

pub fn is_historical_data_received(
    domain_participant: &DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<bool> {
    domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .is_historical_data_received()
}

pub fn get_matched_publication_data(
    domain_participant: &DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
    publication_handle: InstanceHandle,
) -> DdsResult<PublicationBuiltinTopicData> {
    domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_matched_publication_data(publication_handle)
}

pub fn get_matched_publications(
    domain_participant: &DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<Vec<InstanceHandle>> {
    Ok(domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_matched_publications())
}

pub fn set_qos(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
    qos: QosKind<DataReaderQos>,
) -> DdsResult<()> {
    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .set_qos(qos)?;

    let data_reader = domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?;
    if domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .is_enabled()
    {
        let topic = domain_participant
            .topic_list()
            .iter()
            .find(|t| {
                t.get_name() == data_reader.get_topic_name()
                    && t.get_type_name() == data_reader.get_type_name()
            })
            .expect("Topic must exist");
        let subscriber_qos = domain_participant
            .get_subscriber(subscriber_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos();
        let discovered_reader_data = domain_participant
            .get_subscriber(subscriber_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_stateful_data_reader(reader_guid)
            .ok_or(DdsError::AlreadyDeleted)?
            .as_discovered_reader_data(&topic.get_qos(), &subscriber_qos);
        domain_participant
            .announce_sender()
            .send(AnnounceKind::CreatedDataReader(discovered_reader_data))
            .ok();
    }

    Ok(())
}

pub fn get_qos(
    domain_participant: &DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<DataReaderQos> {
    Ok(domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_qos())
}

pub fn enable(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
    subscriber_guid: Guid,
) -> DdsResult<()> {
    if !domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .is_enabled()
    {
        return Err(DdsError::PreconditionNotMet(
            "Parent subscriber disabled".to_string(),
        ));
    }

    domain_participant
        .get_subscriber_mut(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader_mut(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .enable()?;

    let data_reader = domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?;
    let topic = domain_participant
        .topic_list()
        .iter()
        .find(|t| {
            t.get_name() == data_reader.get_topic_name()
                && t.get_type_name() == data_reader.get_type_name()
        })
        .expect("Topic must exist");
    let subscriber_qos = domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_qos();
    let discovered_reader_data = domain_participant
        .get_subscriber(subscriber_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .as_discovered_reader_data(&topic.get_qos(), &subscriber_qos);
    domain_participant
        .announce_sender()
        .send(AnnounceKind::CreatedDataReader(discovered_reader_data))
        .ok();

    Ok(())
}

pub fn get_instance_handle(reader_guid: Guid) -> DdsResult<InstanceHandle> {
    Ok(reader_guid.into())
}
