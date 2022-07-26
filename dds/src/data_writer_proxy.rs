use std::marker::PhantomData;

use crate::api::{
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
use crate::implementation::{
    dds_impl::data_writer_impl::{AnyDataWriterListener, DataWriterImpl},
    dds_type::{DdsSerialize, DdsType},
    utils::shared_object::{DdsShared, DdsWeak},
};

use crate::{publisher_proxy::PublisherProxy, topic_proxy::TopicProxy};

pub struct DataWriterProxy<Foo> {
    data_writer_attributes: DdsWeak<DataWriterImpl>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo> Clone for DataWriterProxy<Foo> {
    fn clone(&self) -> Self {
        Self {
            data_writer_attributes: self.data_writer_attributes.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> DataWriterProxy<Foo> {
    pub fn new(data_writer_attributes: DdsWeak<DataWriterImpl>) -> Self {
        Self {
            data_writer_attributes,
            phantom: PhantomData,
        }
    }
}

impl<Foo> AsRef<DdsWeak<DataWriterImpl>> for DataWriterProxy<Foo> {
    fn as_ref(&self) -> &DdsWeak<DataWriterImpl> {
        &self.data_writer_attributes
    }
}

impl<Foo> FooDataWriter<Foo> for DataWriterProxy<Foo>
where
    Foo: DdsType + DdsSerialize,
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

    fn dispose(&self, data: &Foo, handle: Option<InstanceHandle>) -> DdsResult<()> {
        self.data_writer_attributes.upgrade()?.dispose(data, handle)
    }

    fn dispose_w_timestamp(
        &self,
        data: &Foo,
        handle: Option<InstanceHandle>,
        timestamp: Time,
    ) -> DdsResult<()> {
        self.data_writer_attributes
            .upgrade()?
            .dispose_w_timestamp(data, handle, timestamp)
    }
}

impl<Foo> DataWriter for DataWriterProxy<Foo> {
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

impl<Foo> DataWriterGetPublisher for DataWriterProxy<Foo> {
    type PublisherType = PublisherProxy;

    fn datawriter_get_publisher(&self) -> DdsResult<Self::PublisherType> {
        DataWriter::get_publisher(&self.data_writer_attributes.upgrade()?)
            .map(|x| PublisherProxy::new(x.downgrade()))
    }
}

impl<Foo> DataWriterGetTopic for DataWriterProxy<Foo> {
    type TopicType = TopicProxy<Foo>;

    fn datawriter_get_topic(&self) -> DdsResult<Self::TopicType> {
        DataWriter::get_topic(&self.data_writer_attributes.upgrade()?)
            .map(|x| TopicProxy::new(x.downgrade()))
    }
}

impl<Foo> Entity for DataWriterProxy<Foo>
where
    Foo: DdsType + DdsSerialize + 'static,
{
    type Qos = DataWriterQos;
    type Listener = Box<dyn DataWriterListener<Foo = Foo> + Send + Sync>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        self.data_writer_attributes.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        self.data_writer_attributes.upgrade()?.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DdsResult<()> {
        #[allow(clippy::redundant_closure)]
        self.data_writer_attributes.upgrade()?.set_listener(
            a_listener
                .map::<Box<dyn AnyDataWriterListener<DdsShared<DataWriterImpl>> + Send + Sync>, _>(
                    |l| Box::new(l),
                ),
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
