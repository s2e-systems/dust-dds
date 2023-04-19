use crate::{
    implementation::{
        rtps::stateless_reader::RtpsStatelessReader,
        utils::{
            node::{ChildNode, RootNode},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{error::DdsResult, instance::InstanceHandle, qos::DataReaderQos},
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    dcps_service::DcpsService, dds_data_reader::DdsDataReader, dds_subscriber::DdsSubscriber,
    dds_domain_participant::DdsDomainParticipant, status_condition_impl::StatusConditionImpl,
};

type BuiltinDataReaderStatelessNodeType = ChildNode<
    DdsDataReader<RtpsStatelessReader>,
    ChildNode<DdsSubscriber, ChildNode<DdsDomainParticipant, RootNode<DcpsService>>>,
>;

#[derive(PartialEq, Debug)]
pub struct BuiltinDataReaderStatelessNode(BuiltinDataReaderStatelessNodeType);

impl BuiltinDataReaderStatelessNode {
    pub fn new(node: BuiltinDataReaderStatelessNodeType) -> Self {
        Self(node)
    }

    pub fn read<Foo>(
        &self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.0.get()?.read(
            max_samples,
            sample_states,
            view_states,
            instance_states,
            specific_instance_handle,
        )
    }

    pub fn read_next_instance<Foo>(
        &self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
    ) -> DdsResult<Vec<Sample<Foo>>>
    where
        Foo: for<'de> DdsDeserialize<'de>,
    {
        self.0.get()?.read_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn get_qos(&self) -> DdsResult<DataReaderQos> {
        Ok(self.0.get()?.get_qos())
    }

    pub fn get_statuscondition(&self) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        Ok(self.0.get()?.get_statuscondition())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(InstanceHandle::from(self.0.get()?.guid()))
    }
}
