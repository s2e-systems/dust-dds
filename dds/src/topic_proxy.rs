use std::marker::PhantomData;

use dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::DdsResult,
    topic::{topic::Topic, topic_description::TopicDescription, topic_listener::TopicListener},
};
use dds_implementation::utils::shared_object::{DdsShared, DdsWeak};

use crate::domain_participant_proxy::DomainParticipantProxy;

pub struct TopicProxy<Foo, I> {
    topic_attributes: DdsWeak<I>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo, I> Clone for TopicProxy<Foo, I> {
    fn clone(&self) -> Self {
        Self {
            topic_attributes: self.topic_attributes.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<Foo, I> TopicProxy<Foo, I> {
    pub fn new(topic_attributes: DdsWeak<I>) -> Self {
        Self {
            topic_attributes,
            phantom: PhantomData,
        }
    }
}

impl<Foo, I> AsRef<DdsWeak<I>> for TopicProxy<Foo, I> {
    fn as_ref(&self) -> &DdsWeak<I> {
        &self.topic_attributes
    }
}

impl<Foo, I> Topic for TopicProxy<Foo, I>
where
    I: Topic,
{
    fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        self.topic_attributes
            .upgrade()?
            .get_inconsistent_topic_status()
    }
}

impl<Foo, I, DP> TopicDescription for TopicProxy<Foo, I>
where
    DdsShared<I>: TopicDescription<DomainParticipant = DdsShared<DP>>,
{
    type DomainParticipant = DomainParticipantProxy<DP>;

    fn get_participant(&self) -> DdsResult<Self::DomainParticipant> {
        self.topic_attributes
            .upgrade()?
            .get_participant()
            .map(|x| DomainParticipantProxy::new(x.downgrade()))
    }

    fn get_type_name(&self) -> DdsResult<&'static str> {
        self.topic_attributes.upgrade()?.get_type_name()
    }

    fn get_name(&self) -> DdsResult<String> {
        self.topic_attributes.upgrade()?.get_name()
    }
}

impl<Foo, I> Entity for TopicProxy<Foo, I>
where
    DdsShared<I>: Entity<Qos = TopicQos, Listener = Box<dyn TopicListener>>,
{
    type Qos = <DdsShared<I> as Entity>::Qos;
    type Listener = <DdsShared<I> as Entity>::Listener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DdsResult<()> {
        self.topic_attributes.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        self.topic_attributes.upgrade()?.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DdsResult<()> {
        self.topic_attributes
            .upgrade()?
            .set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        self.topic_attributes.upgrade()?.get_listener()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        self.topic_attributes.upgrade()?.get_statuscondition()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        self.topic_attributes.upgrade()?.get_status_changes()
    }

    fn enable(&self) -> DdsResult<()> {
        self.topic_attributes.upgrade()?.enable()
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        self.topic_attributes.upgrade()?.get_instance_handle()
    }
}
