use crate::{
    implementation::utils::{
        node::{ChildNode, RootNode},
        shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{error::DdsResult, instance::InstanceHandle},
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    builtin_stateful_reader::BuiltinStatefulReader, builtin_subscriber::BuiltInSubscriber,
    domain_participant_impl::DomainParticipantImpl, status_condition_impl::StatusConditionImpl,
};

#[derive(PartialEq, Debug)]
pub struct BuiltinDataReaderStatefulNode(
    ChildNode<BuiltinStatefulReader, ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>>,
);

impl BuiltinDataReaderStatefulNode {
    pub fn new(
        node: ChildNode<
            BuiltinStatefulReader,
            ChildNode<BuiltInSubscriber, RootNode<DomainParticipantImpl>>,
        >,
    ) -> Self {
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

    pub fn get_statuscondition(&self) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        Ok(self.0.get()?.get_statuscondition())
    }
}
