use rust_dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateKind, SampleLostStatus, SampleStateKind, StatusMask,
        ViewStateKind,
    },
    domain::domain_participant::DomainParticipant,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, TopicQos},
    },
    return_type::DDSResult,
    subscription::{
        data_reader::AnyDataReader,
        data_reader_listener::DataReaderListener,
        subscriber::{DataReaderGAT, Subscriber},
    },
};

use crate::utils::shared_object::RtpsWeak;

use super::{
    data_reader_impl::DataReaderImpl, data_reader_proxy::DataReaderProxy, topic_impl::TopicImpl,
    topic_proxy::TopicProxy,
};

pub struct SubscriberProxy<'s, S> {
    participant: &'s dyn DomainParticipant,
    subscriber_impl: RtpsWeak<S>,
}

impl<'s, S> SubscriberProxy<'s, S> {
    pub(crate) fn new(
        participant: &'s dyn DomainParticipant,
        subscriber_impl: RtpsWeak<S>,
    ) -> Self {
        Self {
            participant,
            subscriber_impl,
        }
    }

    /// Get a reference to the subscriber impl's subscriber storage.
    pub(crate) fn subscriber_impl(&self) -> &RtpsWeak<S> {
        &self.subscriber_impl
    }
}

impl<'dr, 's, 't, T, S> DataReaderGAT<'dr, 't, T> for SubscriberProxy<'s, S>
where
    T: 't + 'dr,
{
    type TopicType = TopicProxy<'t, T, TopicImpl>;
    type DataReaderType = DataReaderProxy<'dr, T, DataReaderImpl>;

    fn create_datareader_gat(
        &'dr self,
        a_topic: &'dr Self::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<&'static dyn DataReaderListener<DataPIM = T>>,
        mask: StatusMask,
    ) -> Option<Self::DataReaderType> {
        todo!()
        // let reader_storage_weak = self
        //     .subscriber_storage
        //     .upgrade()
        //     .ok()?
        //     .lock()
        //     .create_datareader((), qos, a_listener, mask)?;
        // let data_reader = DataReaderProxy::new(self, a_topic, reader_storage_weak);
        // Some(data_reader)
    }

    fn delete_datareader_gat(&self, _a_datareader: &Self::DataReaderType) -> DDSResult<()> {
        todo!()
    }

    fn lookup_datareader_gat<'a>(
        &'a self,
        _topic: &'a Self::TopicType,
    ) -> Option<Self::DataReaderType> {
        todo!()
    }
}

impl<'s, S> Subscriber for SubscriberProxy<'s, S> {
    fn begin_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn end_access(&self) -> DDSResult<()> {
        todo!()
    }

    fn notify_datareaders(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_sample_lost_status(&self, _status: &mut SampleLostStatus) -> DDSResult<()> {
        todo!()
    }

    fn delete_contained_entities(&self) -> DDSResult<()> {
        todo!()
    }

    fn set_default_datareader_qos(&self, _qos: Option<DataReaderQos>) -> DDSResult<()> {
        todo!()
    }

    fn get_default_datareader_qos(&self) -> DDSResult<DataReaderQos> {
        todo!()
    }

    fn copy_from_topic_qos(
        &self,
        _a_datareader_qos: &mut DataReaderQos,
        _a_topic_qos: &TopicQos,
    ) -> DDSResult<()> {
        todo!()
    }

    fn get_datareaders(
        &self,
        _readers: &mut [&mut dyn AnyDataReader],
        _sample_states: &[SampleStateKind],
        _view_states: &[ViewStateKind],
        _instance_states: &[InstanceStateKind],
    ) -> DDSResult<()> {
        todo!()
    }

    /// This operation returns the DomainParticipant to which the Subscriber belongs.
    fn get_participant(&self) -> &dyn DomainParticipant {
        self.participant
    }
}

impl<'s, S> Entity for SubscriberProxy<'s, S>
where
    S: Entity,
{
    type Qos = S::Qos;
    type Listener = S::Listener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.subscriber_impl.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        self.subscriber_impl.upgrade()?.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        self.subscriber_impl
            .upgrade()?
            .set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        self.subscriber_impl.upgrade()?.get_listener()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        self.subscriber_impl.upgrade()?.get_statuscondition()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        self.subscriber_impl.upgrade()?.get_status_changes()
    }

    fn enable(&self) -> DDSResult<()> {
        self.subscriber_impl.upgrade()?.enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        self.subscriber_impl.upgrade()?.get_instance_handle()
    }
}
