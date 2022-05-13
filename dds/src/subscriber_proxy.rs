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
        data_reader::{AnyDataReader, DataReader},
        subscriber::{Subscriber, SubscriberDataReaderFactory, SubscriberGetParticipant},
        subscriber_listener::SubscriberListener,
    },
};
use dds_implementation::{
    dds_impl::data_reader_attributes::AnyDataReaderListener,
    utils::shared_object::{DdsShared, DdsWeak},
};

use crate::{
    data_reader_proxy::DataReaderProxy, domain_participant_proxy::DomainParticipantProxy,
    topic_proxy::TopicProxy,
};

#[derive(Clone)]
pub struct SubscriberProxy<I> {
    subscriber_attributes: DdsWeak<I>,
}

impl<I> SubscriberProxy<I> {
    pub fn new(subscriber_attributes: DdsWeak<I>) -> Self {
        Self {
            subscriber_attributes,
        }
    }
}

impl<I> AsRef<DdsWeak<I>> for SubscriberProxy<I> {
    fn as_ref(&self) -> &DdsWeak<I> {
        &self.subscriber_attributes
    }
}

impl<Foo, I, T, DR> SubscriberDataReaderFactory<Foo> for SubscriberProxy<I>
where
    DdsShared<I>:
        SubscriberDataReaderFactory<Foo, TopicType = DdsShared<T>, DataReaderType = DdsShared<DR>>,
    DdsShared<DR>: Entity<
            Qos = DataReaderQos,
            Listener = Box<dyn AnyDataReaderListener<DdsShared<DR>> + Send + Sync>,
        > + DataReader<Foo>,
    Foo: 'static,
{
    type TopicType = TopicProxy<Foo, T>;
    type DataReaderType = DataReaderProxy<Foo, DR>;

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
            a_listener.map::<Box<dyn AnyDataReaderListener<DdsShared<DR>> + Send + Sync>, _>(|x| {
                Box::new(x)
            }),
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

impl<I, DP> Subscriber for SubscriberProxy<I>
where
    DdsShared<I>: Subscriber + SubscriberGetParticipant<DomainParticipant = DP>,
{
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
}

impl<I, DP> SubscriberGetParticipant for SubscriberProxy<I>
where
    DdsShared<I>: Subscriber + SubscriberGetParticipant<DomainParticipant = DdsShared<DP>>,
{
    type DomainParticipant = DomainParticipantProxy<DP>;

    fn subscriber_get_participant(&self) -> DdsResult<Self::DomainParticipant> {
        self.subscriber_attributes
            .upgrade()?
            .get_participant()
            .map(|x| DomainParticipantProxy::new(x.downgrade()))
    }
}

impl<I> Entity for SubscriberProxy<I>
where
    DdsShared<I>: Entity<Qos = SubscriberQos, Listener = Box<dyn SubscriberListener>>,
{
    type Qos = <DdsShared<I> as Entity>::Qos;
    type Listener = <DdsShared<I> as Entity>::Listener;

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
