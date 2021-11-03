use crate::utils::shared_object::{
    rtps_shared_read_lock, rtps_shared_write_lock, rtps_weak_upgrade, RtpsWeak,
};
use rust_dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        Duration, InstanceHandle, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusMask, Time,
    },
    infrastructure::entity::{Entity, StatusCondition},
    publication::{
        data_writer::{AnyDataWriter, DataWriter},
        publisher::Publisher,
    },
    return_type::DDSResult,
    topic::topic::Topic,
};

pub struct DataWriterProxy<'dw, T, DW> {
    publisher: &'dw dyn Publisher,
    topic: &'dw dyn Topic<T>,
    data_writer_impl: RtpsWeak<DW>,
}

impl<'dw, T, DW> DataWriterProxy<'dw, T, DW> {
    pub fn new(
        publisher: &'dw dyn Publisher,
        topic: &'dw dyn Topic<T>,
        data_writer_impl: RtpsWeak<DW>,
    ) -> Self {
        Self {
            publisher,
            topic,
            data_writer_impl,
        }
    }

    pub(crate) fn data_writer_impl(&self) -> &RtpsWeak<DW> {
        &self.data_writer_impl
    }
}

impl<'dw, T, DW> DataWriter<T> for DataWriterProxy<'dw, T, DW>
where
    DW: DataWriter<T>,
{
    fn register_instance(&mut self, instance: T) -> DDSResult<Option<InstanceHandle>> {
        let timestamp = self.publisher.get_participant().get_current_time()?;
        self.register_instance_w_timestamp(instance, timestamp)
    }

    fn register_instance_w_timestamp(
        &mut self,
        instance: T,
        timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
            .register_instance_w_timestamp(instance, timestamp)
    }

    fn unregister_instance(
        &mut self,
        instance: T,
        handle: Option<InstanceHandle>,
    ) -> DDSResult<()> {
        let timestamp = self.publisher.get_participant().get_current_time()?;
        self.unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    fn unregister_instance_w_timestamp(
        &mut self,
        instance: T,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
            .unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    fn get_key_value(&self, _key_holder: &mut T, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &T) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&mut self, data: &T, handle: Option<InstanceHandle>) -> DDSResult<()> {
        let timestamp = self.publisher.get_participant().get_current_time()?;
        self.write_w_timestamp(data, handle, timestamp)
    }

    fn write_w_timestamp(
        &mut self,
        data: &T,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
            .write_w_timestamp(data, handle, timestamp)
    }

    fn dispose(&mut self, _data: T, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &mut self,
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

    fn get_topic(&self) -> &dyn Topic<T> {
        self.topic
    }

    fn get_publisher(&self) -> &dyn Publisher {
        self.publisher
    }
}

impl<'dw, T, DW> Entity for DataWriterProxy<'dw, T, DW>
where
    DW: Entity,
{
    type Qos = DW::Qos;
    type Listener = DW::Listener;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
            .set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_listener()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_statuscondition()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_status_changes()
    }

    fn enable(&self) -> DDSResult<()> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_instance_handle()
    }
}

impl<'dw, T, DW> AnyDataWriter for DataWriterProxy<'dw, T, DW> {}
