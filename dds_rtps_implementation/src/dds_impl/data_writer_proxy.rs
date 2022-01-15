use crate::utils::shared_object::{rtps_shared_write_lock, rtps_weak_upgrade, RtpsWeak};
use rust_dds_api::{
    builtin_topics::SubscriptionBuiltinTopicData,
    dcps_psm::{
        Duration, InstanceHandle, LivelinessLostStatus, OfferedDeadlineMissedStatus,
        OfferedIncompatibleQosStatus, PublicationMatchedStatus, StatusMask, Time,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::DataWriterQos,
    },
    publication::{
        data_writer::{AnyDataWriter, DataWriter},
        data_writer_listener::DataWriterListener,
        publisher::Publisher,
    },
    return_type::DDSResult,
    topic::topic::Topic,
};

use super::{publisher_proxy::PublisherProxy, topic_proxy::TopicProxy};

pub struct DataWriterProxy<Foo> {
    publisher: PublisherProxy,
    _topic: TopicProxy<Foo>,
    data_writer_impl: RtpsWeak<dyn DataWriter<Foo> + Send + Sync>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo> Clone for DataWriterProxy<Foo> {
    fn clone(&self) -> Self {
        Self {
            publisher: self.publisher.clone(),
            _topic: self._topic.clone(),
            data_writer_impl: self.data_writer_impl.clone(),
        }
    }
}

impl<Foo> DataWriterProxy<Foo> {
    pub fn new(
        publisher: PublisherProxy,
        topic: TopicProxy<Foo>,
        data_writer_impl: RtpsWeak<dyn DataWriter<Foo> + Send + Sync>,
    ) -> Self {
        Self {
            publisher,
            _topic: topic,
            data_writer_impl,
        }
    }
}

impl<Foo> AsRef<RtpsWeak<dyn DataWriter<Foo> + Send + Sync>> for DataWriterProxy<Foo> {
    fn as_ref(&self) -> &RtpsWeak<dyn DataWriter<Foo> + Send + Sync> {
        &self.data_writer_impl
    }
}

impl<Foo> DataWriter<Foo> for DataWriterProxy<Foo> {
    fn register_instance(&mut self, instance: Foo) -> DDSResult<Option<InstanceHandle>> {
        let timestamp = self.publisher.get_participant().get_current_time()?;
        self.register_instance_w_timestamp(instance, timestamp)
    }

    fn register_instance_w_timestamp(
        &mut self,
        instance: Foo,
        timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
            .register_instance_w_timestamp(instance, timestamp)
    }

    fn unregister_instance(
        &mut self,
        instance: Foo,
        handle: Option<InstanceHandle>,
    ) -> DDSResult<()> {
        let timestamp = self.publisher.get_participant().get_current_time()?;
        self.unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    fn unregister_instance_w_timestamp(
        &mut self,
        instance: Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
            .unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&mut self, data: &Foo, handle: Option<InstanceHandle>) -> DDSResult<()> {
        let timestamp = self.publisher.get_participant().get_current_time()?;
        self.write_w_timestamp(data, handle, timestamp)
    }

    fn write_w_timestamp(
        &mut self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
            .write_w_timestamp(data, handle, timestamp)
    }

    fn dispose(&mut self, _data: Foo, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        todo!()
    }

    fn dispose_w_timestamp(
        &mut self,
        _data: Foo,
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

    fn get_topic(&self) -> &dyn Topic<Foo> {
        // self.topic
        todo!()
    }

    fn get_publisher(&self) -> &dyn Publisher {
        // self.publisher
        todo!()
    }
}

impl<Foo> Entity for DataWriterProxy<Foo> {
    type Qos = DataWriterQos;
    type Listener = Box<dyn DataWriterListener<DataType = Foo>>;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
        //     .set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.data_writer_impl)?).get_instance_handle()
        todo!()
    }
}

impl<Foo> AnyDataWriter for DataWriterProxy<Foo> {}
