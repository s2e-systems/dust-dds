use crate::{
    implementation::{
        rtps::types::Guid,
        utils::shared_object::{DdsRwLock, DdsShared},
    },
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

use super::{
    dds_domain_participant::DdsDomainParticipant, status_condition_impl::StatusConditionImpl,
};

#[derive(PartialEq, Eq, Debug)]
pub struct BuiltinDataReaderStatelessNode {
    this: Guid,
    parent_subcriber: Guid,
    parent_participant: Guid,
}

impl BuiltinDataReaderStatelessNode {
    pub fn new(this: Guid, parent_subcriber: Guid, parent_participant: Guid) -> Self {
        Self {
            this,
            parent_subcriber,
            parent_participant,
        }
    }

    pub fn guid(&self) -> DdsResult<Guid> {
        Ok(self.this)
    }

    pub fn read<Foo>(
        &self,
        domain_participant: &DdsDomainParticipant,
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
            .get_builtin_subscriber()
            .get_stateless_data_reader(self.this)
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
        &self,
        domain_participant: &DdsDomainParticipant,
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
            .get_builtin_subscriber()
            .get_stateless_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .read_next_instance(
                max_samples,
                previous_handle,
                sample_states,
                view_states,
                instance_states,
            )
    }

    pub fn get_qos(&self, domain_participant: &DdsDomainParticipant) -> DdsResult<DataReaderQos> {
        Ok(domain_participant
            .get_builtin_subscriber()
            .get_stateless_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_qos())
    }

    pub fn get_statuscondition(
        &self,
        domain_participant: &DdsDomainParticipant,
    ) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        Ok(domain_participant
            .get_builtin_subscriber()
            .get_stateless_data_reader(self.this)
            .ok_or(DdsError::AlreadyDeleted)?
            .get_statuscondition())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.this.into())
    }
}
