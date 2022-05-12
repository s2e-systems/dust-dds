use dds_api::{
    dcps_psm::{
        InstanceHandle, InstanceStateMask, SampleLostStatus, SampleStateMask, StatusMask,
        ViewStateMask,
    },
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::{DataReaderQos, SubscriberQos, TopicQos},
    },
    return_type::DdsResult,
    subscription::{
        data_reader::AnyDataReader,
        subscriber::{Subscriber, SubscriberDataReaderFactory},
        subscriber_listener::SubscriberListener,
    },
};
use dds_implementation::{
    dds_impl::{
        data_reader_attributes::AnyDataReaderListener, subscriber_attributes::SubscriberAttributes,
    },
    dds_type::{DdsDeserialize, DdsType},
    utils::{rtps_structure::RtpsStructure, shared_object::DdsWeak, timer::ThreadTimer},
};

use crate::{
    data_reader_proxy::DataReaderProxy, domain_participant_proxy::DomainParticipantProxy,
    topic_proxy::TopicProxy,
};

#[derive(Clone)]
pub struct SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    subscriber_attributes: DdsWeak<SubscriberAttributes<Rtps>>,
}

impl<Rtps> SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    pub fn new(subscriber_attributes: DdsWeak<SubscriberAttributes<Rtps>>) -> Self {
        Self {
            subscriber_attributes,
        }
    }
}

impl<Rtps> AsRef<DdsWeak<SubscriberAttributes<Rtps>>> for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    fn as_ref(&self) -> &DdsWeak<SubscriberAttributes<Rtps>> {
        &self.subscriber_attributes
    }
}

impl<Foo, Rtps> SubscriberDataReaderFactory<Foo> for SubscriberProxy<Rtps>
where
    Foo: DdsType + for<'a> DdsDeserialize<'a> + Send + Sync + 'static,
    Rtps: RtpsStructure,
{
    type TopicType = TopicProxy<Foo, Rtps>;
    type DataReaderType = DataReaderProxy<Foo, Rtps, ThreadTimer>;

    fn datareader_factory_create_datareader(
        &self,
        a_topic: &Self::TopicType,
        qos: Option<DataReaderQos>,
        a_listener: Option<<Self::DataReaderType as Entity>::Listener>,
        mask: StatusMask,
    ) -> DdsResult<Self::DataReaderType> {
        SubscriberDataReaderFactory::<Foo>::datareader_factory_create_datareader(
            &self.subscriber_attributes.upgrade()?,
            &a_topic.as_ref().upgrade()?,
            qos,
            a_listener.map::<Box<dyn AnyDataReaderListener<Rtps, ThreadTimer> + Send + Sync>, _>(
                |x| Box::new(x),
            ),
            mask,
        )
        .map(|x| DataReaderProxy::new(x.downgrade()))
    }

    fn datareader_factory_delete_datareader(
        &self,
        a_datareader: &Self::DataReaderType,
    ) -> DdsResult<()> {
        SubscriberDataReaderFactory::<Foo>::datareader_factory_delete_datareader(
            &self.subscriber_attributes.upgrade()?,
            &a_datareader.as_ref().upgrade()?,
        )
    }

    fn datareader_factory_lookup_datareader(
        &self,
        topic: &Self::TopicType,
    ) -> DdsResult<Self::DataReaderType> {
        SubscriberDataReaderFactory::<Foo>::datareader_factory_lookup_datareader(
            &self.subscriber_attributes.upgrade()?,
            &topic.as_ref().upgrade()?,
        )
        .map(|x| DataReaderProxy::new(x.downgrade()))
    }
}

impl<Rtps> Subscriber for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type DomainParticipant = DomainParticipantProxy<Rtps>;

    fn begin_access(&self) -> DdsResult<()> {
        self.subscriber_attributes.upgrade()?.begin_access()
    }

    fn end_access(&self) -> DdsResult<()> {
        self.subscriber_attributes.upgrade()?.end_access()
    }

    fn get_datareaders(
        &self,
        readers: &mut [&mut dyn AnyDataReader],
        sample_states: SampleStateMask,
        view_states: ViewStateMask,
        instance_states: InstanceStateMask,
    ) -> DdsResult<()> {
        self.subscriber_attributes.upgrade()?.get_datareaders(
            readers,
            sample_states,
            view_states,
            instance_states,
        )
    }

    fn notify_datareaders(&self) -> DdsResult<()> {
        self.subscriber_attributes.upgrade()?.notify_datareaders()
    }

    fn get_sample_lost_status(&self, status: &mut SampleLostStatus) -> DdsResult<()> {
        self.subscriber_attributes
            .upgrade()?
            .get_sample_lost_status(status)
    }

    fn delete_contained_entities(&self) -> DdsResult<()> {
        self.subscriber_attributes
            .upgrade()?
            .delete_contained_entities()
    }

    fn set_default_datareader_qos(&self, qos: Option<DataReaderQos>) -> DdsResult<()> {
        self.subscriber_attributes
            .upgrade()?
            .set_default_datareader_qos(qos)
    }

    fn get_default_datareader_qos(&self) -> DdsResult<DataReaderQos> {
        self.subscriber_attributes
            .upgrade()?
            .get_default_datareader_qos()
    }

    fn copy_from_topic_qos(
        &self,
        a_datareader_qos: &mut DataReaderQos,
        a_topic_qos: &TopicQos,
    ) -> DdsResult<()> {
        self.subscriber_attributes
            .upgrade()?
            .copy_from_topic_qos(a_datareader_qos, a_topic_qos)
    }

    fn get_participant(&self) -> DdsResult<Self::DomainParticipant> {
        self.subscriber_attributes
            .upgrade()?
            .get_participant()
            .map(|x| DomainParticipantProxy::new(x))
    }
}

impl<Rtps> Entity for SubscriberProxy<Rtps>
where
    Rtps: RtpsStructure,
{
    type Qos = SubscriberQos;
    type Listener = Box<dyn SubscriberListener>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        self.subscriber_attributes.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        self.subscriber_attributes.upgrade()?.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DdsResult<()> {
        self.subscriber_attributes
            .upgrade()?
            .set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        self.subscriber_attributes.upgrade()?.get_listener()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.subscriber_attributes.upgrade()?.get_statuscondition()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        self.subscriber_attributes.upgrade()?.get_status_changes()
    }

    fn enable(&self) -> DdsResult<()> {
        self.subscriber_attributes.upgrade()?.enable()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.subscriber_attributes.upgrade()?.get_instance_handle()
    }
}
