use std::marker::PhantomData;

use crate::{
    dds_type::{DdsSerialize, LittleEndian},
    utils::{rtps_structure::RtpsStructure, shared_object::RtpsWeak},
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
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
    return_type::{DDSError, DDSResult},
};
use rust_rtps_pim::{behavior::writer::writer::{RtpsWriterOperations, RtpsWriterAttributes}, structure::{types::ChangeKind, history_cache::RtpsHistoryCacheOperations}};

use super::{
    publisher_proxy::{PublisherAttributes, PublisherProxy},
    topic_proxy::{TopicAttributes, TopicProxy},
};

pub enum RtpsWriter<Rtps>
where
    Rtps: RtpsStructure,
{
    Stateless(Rtps::StatelessWriter),
    Stateful(Rtps::StatefulWriter),
}

impl<Rtps> RtpsWriter<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn try_as_stateless_writer(&mut self) -> DDSResult<&mut Rtps::StatelessWriter> {
        match self {
            RtpsWriter::Stateless(x) => Ok(x),
            RtpsWriter::Stateful(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateless writer".to_string(),
            )),
        }
    }
    pub fn try_as_stateful_writer(&mut self) -> DDSResult<&mut Rtps::StatefulWriter> {
        match self {
            RtpsWriter::Stateless(_) => Err(DDSError::PreconditionNotMet(
                "Not a stateful writer".to_string(),
            )),
            RtpsWriter::Stateful(x) => Ok(x),
        }
    }
}

pub struct DataWriterAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub _qos: DataWriterQos,
    pub rtps_writer: RtpsWriter<Rtps>,
    pub _listener: Option<Box<dyn DataWriterListener + Send + Sync>>,
    pub topic: RtpsWeak<TopicAttributes<Rtps>>,
    pub publisher: RtpsWeak<PublisherAttributes<Rtps>>,
}

impl<Rtps> DataWriterAttributes<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(
        qos: DataWriterQos,
        rtps_writer: RtpsWriter<Rtps>,
        topic: RtpsWeak<TopicAttributes<Rtps>>,
        publisher: RtpsWeak<PublisherAttributes<Rtps>>,
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

pub struct DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    data_writer_impl: RtpsWeak<DataWriterAttributes<Rtps>>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo, Rtps> Clone for DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    fn clone(&self) -> Self {
        Self {
            data_writer_impl: self.data_writer_impl.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<Foo, Rtps> DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(data_writer_impl: RtpsWeak<DataWriterAttributes<Rtps>>) -> Self {
        Self {
            data_writer_impl,
            phantom: PhantomData,
        }
    }
}

impl<Foo, Rtps> AsRef<RtpsWeak<DataWriterAttributes<Rtps>>> for DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
    fn as_ref(&self) -> &RtpsWeak<DataWriterAttributes<Rtps>> {
        &self.data_writer_impl
    }
}

impl<Foo, Rtps> DataWriter<Foo> for DataWriterProxy<Foo, Rtps>
where
    Foo: DdsSerialize,
    Rtps: RtpsStructure,
{
    type Publisher = PublisherProxy<Rtps>;
    type Topic = TopicProxy<Foo, Rtps>;

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
        data.serialize::<_, LittleEndian>(&mut serialized_data)?;

        let data_writer_shared = self.data_writer_impl.upgrade()?;
        let rtps_writer = &mut data_writer_shared.write()
            .map_err(|_| DDSError::Error)?
            .rtps_writer;

        match rtps_writer {
            RtpsWriter::Stateless(rtps_writer) => {
                let change =
                    rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                rtps_writer.writer_cache().add_change(change);
            }
            RtpsWriter::Stateful(rtps_writer) => {
                let change =
                    rtps_writer.new_change(ChangeKind::Alive, serialized_data, vec![], 0);
                rtps_writer.writer_cache().add_change(change);
            }
        }

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

impl<Foo, Rtps> Entity for DataWriterProxy<Foo, Rtps>
where
    Rtps: RtpsStructure,
{
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
