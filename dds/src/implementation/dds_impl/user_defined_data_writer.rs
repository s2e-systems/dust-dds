use crate::{
    builtin_topics::SubscriptionBuiltinTopicData,
    implementation::utils::{
        node::{ChildNode, RootNode},
        shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, QosKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::{Duration, Time},
    },
    topic_definition::type_support::{DdsSerializedKey, DdsType},
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    domain_participant_impl::DomainParticipantImpl, status_condition_impl::StatusConditionImpl,
    topic_impl::TopicImpl, user_defined_data_writer_impl::UserDefinedDataWriterImpl,
    user_defined_publisher_impl::UserDefinedPublisherImpl,
};

pub struct UserDefinedDataWriter(
    ChildNode<
        UserDefinedDataWriterImpl,
        ChildNode<UserDefinedPublisherImpl, RootNode<DomainParticipantImpl>>,
    >,
);

impl UserDefinedDataWriter {
    pub fn new(
        node: ChildNode<
            UserDefinedDataWriterImpl,
            ChildNode<UserDefinedPublisherImpl, RootNode<DomainParticipantImpl>>,
        >,
    ) -> Self {
        Self(node)
    }

    pub fn register_instance_w_timestamp(
        &self,
        instance_serialized_key: DdsSerializedKey,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        self.0
            .get()?
            .register_instance_w_timestamp(instance_serialized_key, timestamp)
    }

    pub fn unregister_instance_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.0
            .get()?
            .unregister_instance_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn get_key_value<Foo>(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()>
    where
        Foo: DdsType,
    {
        self.0.get()?.get_key_value(key_holder, handle)
    }

    pub fn lookup_instance(
        &self,
        instance_serialized_key: DdsSerializedKey,
    ) -> DdsResult<Option<InstanceHandle>> {
        self.0.get()?.lookup_instance(instance_serialized_key)
    }

    pub fn write_w_timestamp(
        &self,
        serialized_data: Vec<u8>,
        instance_serialized_key: DdsSerializedKey,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.0
            .get()?
            .write_w_timestamp(serialized_data, instance_serialized_key, handle, timestamp)
    }

    pub fn dispose_w_timestamp(
        &self,
        instance_serialized_key: Vec<u8>,
        handle: InstanceHandle,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.0
            .get()?
            .dispose_w_timestamp(instance_serialized_key, handle, timestamp)
    }

    pub fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        self.0.get()?.wait_for_acknowledgments(max_wait)
    }

    pub fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        Ok(self.0.get()?.get_liveliness_lost_status())
    }

    pub fn get_offered_deadline_missed_status(&self) -> DdsResult<OfferedDeadlineMissedStatus> {
        Ok(self.0.get()?.get_offered_deadline_missed_status())
    }

    pub fn get_offered_incompatible_qos_status(&self) -> DdsResult<OfferedIncompatibleQosStatus> {
        Ok(self.0.get()?.get_offered_incompatible_qos_status())
    }

    pub fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        Ok(self.0.get()?.get_publication_matched_status())
    }

    pub fn get_topic(&self) -> DdsResult<DdsShared<TopicImpl>> {
        Ok(self.0.get()?.get_topic())
    }

    pub fn get_publisher(&self) -> DdsShared<UserDefinedPublisherImpl> {
        todo!()
    }

    pub fn assert_liveliness(&self) -> DdsResult<()> {
        self.0.get()?.assert_liveliness()
    }

    pub fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        self.0
            .get()?
            .get_matched_subscription_data(subscription_handle)
    }

    pub fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        self.0.get()?.get_matched_subscriptions()
    }

    pub fn set_qos(&self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        self.0.get()?.set_qos(qos)
    }

    pub fn get_qos(&self) -> DdsResult<DataWriterQos> {
        Ok(self.0.get()?.get_qos())
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<()> {
        Ok(self.0.get()?.set_listener(a_listener, mask))
    }

    pub fn get_statuscondition(&self) -> DdsResult<DdsShared<DdsRwLock<StatusConditionImpl>>> {
        Ok(self.0.get()?.get_statuscondition())
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        Ok(self.0.get()?.get_status_changes())
    }

    pub fn enable(&self) -> DdsResult<()> {
        self.0.get()?.enable()
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        Ok(self.0.get()?.get_instance_handle())
    }
}
