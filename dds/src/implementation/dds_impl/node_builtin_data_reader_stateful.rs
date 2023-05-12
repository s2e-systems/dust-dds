use crate::{
    implementation::rtps::types::Guid,
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::DataReaderQos,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::dds_domain_participant::DdsDomainParticipant;

#[derive(PartialEq, Eq, Debug)]
pub struct BuiltinDataReaderStatefulNode {
    this: Guid,
    parent_subcriber: Guid,
    parent_participant: Guid,
}

impl BuiltinDataReaderStatefulNode {
    pub fn new(this: Guid, parent_subcriber: Guid, parent_participant: Guid) -> Self {
        Self {
            this,
            parent_subcriber,
            parent_participant,
        }
    }

    pub fn guid(&self) -> Guid {
        self.this
    }
}

pub fn read<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
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
        .get_builtin_subscriber_mut()
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

pub fn read_next_instance<Foo>(
    domain_participant: &mut DdsDomainParticipant,
    reader_guid: Guid,
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
        .get_builtin_subscriber_mut()
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

pub fn get_qos(
    domain_participant: &DdsDomainParticipant,
    reader_guid: Guid,
) -> DdsResult<DataReaderQos> {
    Ok(domain_participant
        .get_builtin_subscriber()
        .get_stateful_data_reader(reader_guid)
        .ok_or(DdsError::AlreadyDeleted)?
        .get_qos())
}

pub fn get_instance_handle(reader_guid: Guid) -> DdsResult<InstanceHandle> {
    Ok(reader_guid.into())
}
