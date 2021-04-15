use rust_dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        Duration, InstanceHandle, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusMask, Time,
    },
    dds_type::DDSType,
    domain::domain_participant::DomainParticipant,
    infrastructure::{entity::StatusCondition, qos::DataWriterQos},
    publication::data_writer_listener::DataWriterListener,
    return_type::{DDSError, DDSResult},
};

use super::{
    domain_participant::{Publisher, Topic},
    publisher::DataWriter,
};

impl<'a, T: DDSType> rust_dds_api::publication::publisher::PublisherChild<'a>
    for DataWriter<'a, T>
{
    type PublisherType = Publisher<'a>;
}

impl<'a, T: DDSType> rust_dds_api::domain::domain_participant::TopicGAT<'a, T>
    for DataWriter<'a, T>
{
    type TopicType = Topic<'a, T>;
}

impl<'a, T: DDSType> rust_dds_api::publication::data_writer::DataWriter<'a, T>
    for DataWriter<'a, T>
{
    fn register_instance(&self, instance: T) -> DDSResult<Option<InstanceHandle>> {
        let timestamp = self.parent.0.parent.get_current_time()?;
        self.register_instance_w_timestamp(instance, timestamp)
    }

    fn register_instance_w_timestamp(
        &self,
        _instance: T,
        _timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        todo!()
        // self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .register_instance_w_timestamp(instance, timestamp)
    }

    fn unregister_instance(&self, _instance: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn unregister_instance_w_timestamp(
        &self,
        _instance: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &T) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn write_w_timestamp(
        &self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        self.impl_ref.upgrade().ok_or(DDSError::AlreadyDeleted)?.lock().unwrap().write_w_timestamp()
    }

    fn dispose(&self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &self,
        _data: T,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        todo!()
    }

    fn wait_for_acknowledgments(&self, _max_wait: Duration) -> DDSResult<()> {
        todo!()
    }

    fn get_liveliness_lost_status(&self, _status: &mut LivelinessLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_deadline_missed_status(
        &self,
        _status: &mut OfferedDeadlineMissedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_offered_incompatible_qos_status(
        &self,
        _status: &mut OfferedIncompatibleQosStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_publication_matched_status(
        &self,
        _status: &mut PublicationMatchedStatus,
    ) -> DDSResult<()> {
        todo!()
    }

    /// This operation returns the Topic associated with the DataWriter. This is the same Topic that was used to create the DataWriter.
    fn get_topic(
        &self,
    ) -> &<Self as rust_dds_api::domain::domain_participant::TopicGAT<'a, T>>::TopicType {
        // self.parent.1
        todo!()
    }

    /// This operation returns the Publisher to which the publisher child object belongs.
    fn get_publisher(
        &self,
    ) -> &<Self as rust_dds_api::publication::publisher::PublisherChild<'a>>::PublisherType {
        // self.parent.0
        todo!()
    }

    fn assert_liveliness(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscription_data(
        &self,
        _subscription_data: SubscriptionBuiltinTopicData,
        _subscription_handle: InstanceHandle,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_matched_subscriptions(
        &self,
        _subscription_handles: &mut [InstanceHandle],
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a, T: DDSType> rust_dds_api::infrastructure::entity::Entity for DataWriter<'a, T> {
    type Qos = DataWriterQos;

    type Listener = Box<dyn DataWriterListener<DataType = T> + 'a>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

impl<'a, T: DDSType> rust_dds_api::publication::data_writer::AnyDataWriter for DataWriter<'a, T> {}
