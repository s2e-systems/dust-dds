use std::marker::PhantomData;

use dds_api::{
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
        data_writer::{DataWriter, DataWriterGetPublisher, DataWriterGetTopic, FooDataWriter},
        data_writer_listener::DataWriterListener,
    },
    return_type::DdsResult,
};
use dds_implementation::{
    dds_impl::data_writer_impl::AnyDataWriterListener,
    utils::shared_object::{DdsShared, DdsWeak},
};

use crate::{publisher_proxy::PublisherProxy, topic_proxy::TopicProxy};

pub struct DataWriterProxy<Foo, I> {
    data_writer_attributes: DdsWeak<I>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo, I> Clone for DataWriterProxy<Foo, I> {
    fn clone(&self) -> Self {
        Self {
            data_writer_attributes: self.data_writer_attributes.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<Foo, I> DataWriterProxy<Foo, I> {
    pub fn new(data_writer_attributes: DdsWeak<I>) -> Self {
        Self {
            data_writer_attributes,
            phantom: PhantomData,
        }
    }
}

impl<Foo, I> AsRef<DdsWeak<I>> for DataWriterProxy<Foo, I> {
    fn as_ref(&self) -> &DdsWeak<I> {
        &self.data_writer_attributes
    }
}

impl<Foo, I> FooDataWriter<Foo> for DataWriterProxy<Foo, I>
where
    DdsShared<I>: FooDataWriter<Foo>,
{
    fn register_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        self.data_writer_attributes
            .upgrade()?
            .register_instance(instance)
    }

    fn register_instance_w_timestamp(
        &self,
        instance: &Foo,
        timestamp: Time,
    ) -> DdsResult<Option<InstanceHandle>> {
        self.data_writer_attributes
            .upgrade()?
            .register_instance_w_timestamp(instance, timestamp)
    }

    fn unregister_instance(&self, instance: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        self.data_writer_attributes
            .upgrade()?
            .unregister_instance(instance, handle)
    }

    fn unregister_instance_w_timestamp(
        &self,
        instance: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.data_writer_attributes
            .upgrade()?
            .unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    fn get_key_value(&self, key_holder: &mut Foo, handle: InstanceHandle) -> DdsResult<()> {
        self.data_writer_attributes
            .upgrade()?
            .get_key_value(key_holder, handle)
    }

    fn lookup_instance(&self, instance: &Foo) -> DdsResult<Option<InstanceHandle>> {
        self.data_writer_attributes
            .upgrade()?
            .lookup_instance(instance)
    }

    fn write(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        self.data_writer_attributes.upgrade()?.write(data, handle)
    }

    fn write_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.data_writer_attributes
            .upgrade()?
            .write_w_timestamp(data, handle, timestamp)
    }

    fn dispose(&self, data: Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        self.data_writer_attributes.upgrade()?.dispose(data, handle)
    }

    fn dispose_w_timestamp(
        &self,
        data: Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.data_writer_attributes
            .upgrade()?
            .dispose_w_timestamp(data, handle, timestamp)
    }
}

impl<Foo, I, P, T> DataWriter for DataWriterProxy<Foo, I>
where
    DdsShared<I>: DataWriter
        + DataWriterGetPublisher<PublisherType = DdsShared<P>>
        + DataWriterGetTopic<TopicType = DdsShared<T>>,
{
    fn wait_for_acknowledgments(&self, max_wait: Duration) -> DdsResult<()> {
        DataWriter::wait_for_acknowledgments(&self.data_writer_attributes.upgrade()?, max_wait)
    }

    fn get_liveliness_lost_status(&self) -> DdsResult<LivelinessLostStatus> {
        DataWriter::get_liveliness_lost_status(&self.data_writer_attributes.upgrade()?)
    }

    fn get_offered_deadline_missed_status(&self) -> DdsResult<OfferedDeadlineMissedStatus> {
        DataWriter::get_offered_deadline_missed_status(&self.data_writer_attributes.upgrade()?)
    }

    fn get_offered_incompatible_qos_status(&self) -> DdsResult<OfferedIncompatibleQosStatus> {
        DataWriter::get_offered_incompatible_qos_status(&self.data_writer_attributes.upgrade()?)
    }

    fn get_publication_matched_status(&self) -> DdsResult<PublicationMatchedStatus> {
        DataWriter::get_publication_matched_status(&self.data_writer_attributes.upgrade()?)
    }

    fn assert_liveliness(&self) -> DdsResult<()> {
        DataWriter::assert_liveliness(&self.data_writer_attributes.upgrade()?)
    }

    fn get_matched_subscription_data(
        &self,
        subscription_handle: InstanceHandle,
    ) -> DdsResult<SubscriptionBuiltinTopicData> {
        DataWriter::get_matched_subscription_data(
            &self.data_writer_attributes.upgrade()?,
            subscription_handle,
        )
    }

    fn get_matched_subscriptions(&self) -> DdsResult<Vec<InstanceHandle>> {
        DataWriter::get_matched_subscriptions(&self.data_writer_attributes.upgrade()?)
    }
}

impl<Foo, I, P> DataWriterGetPublisher for DataWriterProxy<Foo, I>
where
    DdsShared<I>: DataWriter + DataWriterGetPublisher<PublisherType = DdsShared<P>>,
{
    type PublisherType = PublisherProxy<P>;

    fn datawriter_get_publisher(&self) -> DdsResult<Self::PublisherType> {
        DataWriter::get_publisher(&self.data_writer_attributes.upgrade()?)
            .map(|x| PublisherProxy::new(x.downgrade()))
    }
}

impl<Foo, I, T> DataWriterGetTopic for DataWriterProxy<Foo, I>
where
    DdsShared<I>: DataWriter + DataWriterGetTopic<TopicType = DdsShared<T>>,
{
    type TopicType = TopicProxy<Foo, T>;

    fn datawriter_get_topic(&self) -> DdsResult<Self::TopicType> {
        DataWriter::get_topic(&self.data_writer_attributes.upgrade()?)
            .map(|x| TopicProxy::new(x.downgrade()))
    }
}

impl<Foo, I> Entity for DataWriterProxy<Foo, I>
where
    DdsShared<I>: Entity<
            Qos = DataWriterQos,
            Listener = Box<dyn AnyDataWriterListener<DdsShared<I>> + Send + Sync>,
        > + FooDataWriter<Foo>,
    Foo: 'static,
{
    type Qos = <DdsShared<I> as Entity>::Qos;
    type Listener = Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        self.data_writer_attributes.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        self.data_writer_attributes.upgrade()?.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DdsResult<()> {
        self.data_writer_attributes.upgrade()?.set_listener(
            a_listener.map::<Box<dyn AnyDataWriterListener<DdsShared<I>> + Send + Sync>, _>(|l| {
                Box::new(l)
            }),
            mask,
        )
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.data_writer_attributes.upgrade()?.get_statuscondition()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        self.data_writer_attributes.upgrade()?.get_status_changes()
    }

    fn enable(&self) -> DdsResult<()> {
        self.data_writer_attributes.upgrade()?.enable()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.data_writer_attributes.upgrade()?.get_instance_handle()
    }
}
