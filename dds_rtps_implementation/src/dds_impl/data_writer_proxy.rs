use std::marker::PhantomData;

use crate::{
    dds_type::{DdsSerialize, LittleEndian},
    rtps_impl::{
        rtps_stateful_writer_impl::RtpsStatefulWriterImpl,
        rtps_stateless_writer_impl::RtpsStatelessWriterImpl,
    },
    utils::shared_object::RtpsWeak,
};
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
    },
    return_type::{DDSError, DDSResult},
};

use super::{
    publisher_proxy::{PublisherAttributes, PublisherProxy},
    topic_proxy::{TopicAttributes, TopicProxy},
};

pub enum RtpsWriter {
    Stateless(RtpsStatelessWriterImpl),
    Stateful(RtpsStatefulWriterImpl),
}

impl RtpsWriter {
    pub fn try_as_stateless_writer(&mut self) -> DDSResult<&mut RtpsStatelessWriterImpl> {
        match self {
            RtpsWriter::Stateless(x) => Ok(x),
            RtpsWriter::Stateful(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateless writer".to_string(),
            )),
        }
    }
    pub fn try_as_stateful_writer(&mut self) -> DDSResult<&mut RtpsStatefulWriterImpl> {
        match self {
            RtpsWriter::Stateless(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateful writer".to_string(),
            )),
            RtpsWriter::Stateful(x) => Ok(x),
        }
    }
}

pub struct DataWriterAttributes {
    pub _qos: DataWriterQos,
    pub rtps_writer: RtpsWriter,
    pub _listener: Option<Box<dyn DataWriterListener + Send + Sync>>,
    pub topic: RtpsWeak<TopicAttributes>,
    pub publisher: RtpsWeak<PublisherAttributes>,
}

impl DataWriterAttributes {
    pub fn new(
        qos: DataWriterQos,
        rtps_writer: RtpsWriter,
        topic: RtpsWeak<TopicAttributes>,
        publisher: RtpsWeak<PublisherAttributes>,
    ) -> Self {
        Self {
            _qos: qos,
            rtps_writer,
            _listener: None,
            topic,
            publisher,
        }
    }
}

pub struct DataWriterProxy<Foo> {
    data_writer_impl: RtpsWeak<DataWriterAttributes>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo> Clone for DataWriterProxy<Foo> {
    fn clone(&self) -> Self {
        Self {
            data_writer_impl: self.data_writer_impl.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<Foo> DataWriterProxy<Foo> {
    pub fn new(data_writer_impl: RtpsWeak<DataWriterAttributes>) -> Self {
        Self {
            data_writer_impl,
            phantom: PhantomData,
        }
    }
}

impl<Foo> AsRef<RtpsWeak<DataWriterAttributes>> for DataWriterProxy<Foo> {
    fn as_ref(&self) -> &RtpsWeak<DataWriterAttributes> {
        &self.data_writer_impl
    }
}

impl<Foo> DataWriter<Foo> for DataWriterProxy<Foo>
where
    Foo: DdsSerialize,
{
    type Publisher = PublisherProxy;
    type Topic = TopicProxy<Foo>;

    fn register_instance(&mut self, _instance: Foo) -> DDSResult<Option<InstanceHandle>> {
        // let timestamp = self.publisher.get_participant()?.get_current_time()?;
        // self.register_instance_w_timestamp(instance, timestamp)
        todo!()
    }

    fn register_instance_w_timestamp(
        &mut self,
        _instance: Foo,
        _timestamp: Time,
    ) -> DDSResult<Option<InstanceHandle>> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
        // .register_instance_w_timestamp(instance, timestamp)
        todo!()
    }

    fn unregister_instance(
        &mut self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
    ) -> DDSResult<()> {
        todo!()
        // let timestamp = self.publisher.get_participant()?.get_current_time()?;
        // self.unregister_instance_w_timestamp(instance, handle, timestamp)
    }

    fn unregister_instance_w_timestamp(
        &mut self,
        _instance: Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.data_writer_impl)?)
        // .unregister_instance_w_timestamp(instance, handle, timestamp)
        todo!()
    }

    fn get_key_value(&self, _key_holder: &mut Foo, _handle: InstanceHandle) -> DDSResult<()> {
        todo!()
    }

    fn lookup_instance(&self, _instance: &Foo) -> DDSResult<Option<InstanceHandle>> {
        todo!()
    }

    fn write(&mut self, _data: &Foo, _handle: Option<InstanceHandle>) -> DDSResult<()> {
        // let timestamp = self.publisher.get_participant()?.get_current_time()?;
        // self.write_w_timestamp(data, handle, timestamp)
        todo!()
    }

    fn write_w_timestamp(
        &mut self,
        data: &Foo,
        _handle: Option<InstanceHandle>,
        _timestamp: Time,
    ) -> DDSResult<()> {
        let mut serialized_data = Vec::new();
        data.serialize::<_, LittleEndian>(&mut serialized_data)
            .unwrap();
        // match &mut self.rtps_writer {
        //     RtpsWriter::Stateless(stateless_rtps_writer) => {
        //         let change =
        //             stateless_rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
        //         stateless_rtps_writer.writer_cache().add_change(change);
        //     }
        //     RtpsWriter::Stateful(stateful_rtps_writer) => {
        //         let change =
        //             stateful_rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
        //         stateful_rtps_writer.writer_cache().add_change(change);
        //     }
        // }

        Ok(())
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

    fn get_topic(&self) -> DDSResult<Self::Topic> {
        // Ok(self.topic.clone())
        todo!()
    }

    fn get_publisher(&self) -> DDSResult<Self::Publisher> {
        // Ok(self.publisher.clone())
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

impl<Foo> Entity for DataWriterProxy<Foo> {
    type Qos = DataWriterQos;
    type Listener = Box<dyn DataWriterListener>;

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
