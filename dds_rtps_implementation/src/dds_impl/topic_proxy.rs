use crate::utils::shared_object::{
    rtps_shared_read_lock, rtps_shared_write_lock, rtps_weak_upgrade, RtpsWeak,
};
use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipant,
    infrastructure::entity::{Entity, StatusCondition},
    return_type::DDSResult,
    topic::{topic::Topic, topic_description::TopicDescription},
};
use std::marker::PhantomData;

pub struct TopicProxy<'t, Foo, T> {
    participant: &'t dyn DomainParticipant,
    topic_impl: RtpsWeak<T>,
    phantom: PhantomData<&'t Foo>,
}

impl<'t, Foo, T> TopicProxy<'t, Foo, T> {
    pub fn new(participant: &'t dyn DomainParticipant, topic_impl: RtpsWeak<T>) -> Self {
        Self {
            participant,
            topic_impl,
            phantom: PhantomData,
        }
    }
}

impl<Foo, T> AsRef<RtpsWeak<T>> for TopicProxy<'_, Foo, T> {
    fn as_ref(&self) -> &RtpsWeak<T> {
        &self.topic_impl
    }
}

impl<'t, Foo, T> Topic<Foo> for TopicProxy<'t, Foo, T>
where
    T: Topic<Foo>,
{
    fn get_inconsistent_topic_status(&self) -> DDSResult<InconsistentTopicStatus> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_inconsistent_topic_status()
    }
}

impl<'t, Foo, T> TopicDescription<Foo> for TopicProxy<'t, Foo, T>
where
    T: TopicDescription<Foo> + 't,
{
    fn get_participant(&self) -> &dyn DomainParticipant {
        self.participant
    }

    fn get_type_name(&self) -> DDSResult<&'static str> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_type_name()
    }

    fn get_name(&self) -> DDSResult<&'static str> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_name()
    }
}

impl<'t, Foo, T> Entity for TopicProxy<'t, Foo, T>
where
    T: Entity,
{
    type Qos = T::Qos;
    type Listener = T::Listener;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.topic_impl)?).set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        rtps_shared_write_lock(&rtps_weak_upgrade(&self.topic_impl)?).set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_listener()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_statuscondition()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_status_changes()
    }

    fn enable(&self) -> DDSResult<()> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_instance_handle()
    }
}
