use crate::{
    builtin_topics::PublicationBuiltinTopicData,
    implementation::utils::{
        node::{ChildNode, RootNode},
        shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind},
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
        time::Duration,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{InstanceStateKind, SampleStateKind, ViewStateKind},
    },
    topic_definition::type_support::DdsDeserialize,
};

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    domain_participant_impl::DomainParticipantImpl,
    node_user_defined_subscriber::UserDefinedSubscriberNode,
    node_user_defined_topic::UserDefinedTopicNode, status_condition_impl::StatusConditionImpl,
    user_defined_data_reader::UserDefinedDataReader,
    user_defined_subscriber::UserDefinedSubscriber,
};

#[derive(PartialEq, Debug)]
pub struct UserDefinedDataReaderNode(
    ChildNode<
        UserDefinedDataReader,
        ChildNode<UserDefinedSubscriber, RootNode<DomainParticipantImpl>>,
    >,
);

impl UserDefinedDataReaderNode {
    pub fn new(
        node: ChildNode<
            UserDefinedDataReader,
            ChildNode<UserDefinedSubscriber, RootNode<DomainParticipantImpl>>,
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

    pub fn take<Foo>(
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
        self.0.get()?.take(
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

    pub fn take_next_instance<Foo>(
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
        self.0.get()?.take_next_instance(
            max_samples,
            previous_handle,
            sample_states,
            view_states,
            instance_states,
        )
    }

    pub fn get_key_value<Foo>(
        &self,
        key_holder: &mut Foo,
        handle: InstanceHandle,
    ) -> DdsResult<()> {
        self.0.get()?.get_key_value(key_holder, handle)
    }

    pub fn lookup_instance<Foo>(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        self.0.get()?.lookup_instance(instance)
    }

    pub fn get_liveliness_changed_status(&self) -> DdsResult<LivelinessChangedStatus> {
        Ok(self.0.get()?.get_liveliness_changed_status())
    }

    pub fn get_requested_deadline_missed_status(&self) -> DdsResult<RequestedDeadlineMissedStatus> {
        Ok(self.0.get()?.get_requested_deadline_missed_status())
    }

    pub fn get_requested_incompatible_qos_status(
        &self,
    ) -> DdsResult<RequestedIncompatibleQosStatus> {
        Ok(self.0.get()?.get_requested_incompatible_qos_status())
    }

    pub fn get_sample_lost_status(&self) -> DdsResult<SampleLostStatus> {
        Ok(self.0.get()?.get_sample_lost_status())
    }

    pub fn get_sample_rejected_status(&self) -> DdsResult<SampleRejectedStatus> {
        Ok(self.0.get()?.get_sample_rejected_status())
    }

    pub fn get_subscription_matched_status(&self) -> DdsResult<SubscriptionMatchedStatus> {
        Ok(self.0.get()?.get_subscription_matched_status())
    }

    pub fn get_topicdescription(&self) -> DdsResult<UserDefinedTopicNode> {
        let topic = self
            .0
            .parent()
            .parent()
            .get()?
            .lookup_topicdescription(
                self.0.get()?.get_topic_name(),
                self.0.get()?.get_type_name(),
            )
            .expect("Topic must exist");

        Ok(UserDefinedTopicNode::new(ChildNode::new(
            topic.downgrade(),
            self.0.parent().parent().clone(),
        )))
    }

    pub fn get_subscriber(&self) -> UserDefinedSubscriberNode {
        UserDefinedSubscriberNode::new(self.0.parent().clone())
    }

    pub fn wait_for_historical_data(&self, max_wait: Duration) -> DdsResult<()> {
        self.0.get()?.wait_for_historical_data(max_wait)
    }

    pub fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        self.0
            .get()?
            .get_matched_publication_data(publication_handle)
    }

    pub fn get_matched_publications(&self) -> DdsResult<Vec<InstanceHandle>> {
        Ok(self.0.get()?.get_matched_publications())
    }

    pub fn set_qos(&self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        self.0.get()?.set_qos(qos)?;

        if self.0.get()?.is_enabled() {
            let topic = self
                .0
                .parent()
                .parent()
                .get()?
                .lookup_topicdescription(
                    self.0.get()?.get_topic_name(),
                    self.0.get()?.get_type_name(),
                )
                .expect("Topic must exist");
            self.0.get()?.announce_reader(&topic.get_qos());
        }

        Ok(())
    }

    pub fn get_qos(&self) -> DdsResult<DataReaderQos> {
        Ok(self.0.get()?.get_qos())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        self.0.get()?.set_listener(a_listener, mask);
        Ok(())
    }

    pub fn get_statuscondition(&self) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        Ok(self.0.get()?.get_statuscondition())
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self.0.get()?.get_status_changes())
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !self.0.parent().get()?.is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent subscriber disabled".to_string(),
            ));
        }

        self.0.get()?.enable()?;

        let topic = self
            .0
            .parent()
            .parent()
            .get()?
            .lookup_topicdescription(
                self.0.get()?.get_topic_name(),
                self.0.get()?.get_type_name(),
            )
            .expect("Topic must exist");
        self.0.get()?.announce_reader(&topic.get_qos());

        Ok(())
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }
}
