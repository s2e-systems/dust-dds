use crate::utils::shared_object::RtpsWeak;
use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipant,
    infrastructure::entity::{Entity, StatusCondition},
    return_type::DDSResult,
    topic::{topic::Topic, topic_description::TopicDescription},
};
use std::marker::PhantomData;

pub struct TopicProxy<'t, T, I> {
    participant: &'t dyn DomainParticipant,
    topic_impl: RtpsWeak<I>,
    phantom: PhantomData<&'t T>,
}

impl<'t, T, I> TopicProxy<'t, T, I> {
    pub(crate) fn _new(participant: &'t dyn DomainParticipant, topic_impl: RtpsWeak<I>) -> Self {
        Self {
            participant,
            topic_impl,
            phantom: PhantomData,
        }
    }

    pub(crate) fn topic_impl(&self) -> &RtpsWeak<I> {
        &self.topic_impl
    }
}

impl<'t, T, I> Topic<T> for TopicProxy<'t, T, I>
where
    I: Topic<T>,
{
    fn get_inconsistent_topic_status(&self) -> DDSResult<InconsistentTopicStatus> {
        self.topic_impl
            .upgrade()?
            .read()
            .get_inconsistent_topic_status()
    }
}

impl<'t, T, TT> TopicDescription<T> for TopicProxy<'t, T, TT>
where
    TT: TopicDescription<T> + 't,
{
    fn get_participant(&self) -> &dyn DomainParticipant {
        self.participant
    }

    fn get_type_name(&self) -> DDSResult<&'static str> {
        self.topic_impl.upgrade()?.read().get_type_name()
    }

    fn get_name(&self) -> DDSResult<&'static str> {
        self.topic_impl.upgrade()?.read().get_name()
    }
}

impl<'t, T, TT> Entity for TopicProxy<'t, T, TT>
where
    TT: Entity,
{
    type Qos = TT::Qos;
    type Listener = TT::Listener;

    fn set_qos(&mut self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.topic_impl.upgrade()?.write().set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        self.topic_impl.upgrade()?.read().get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        self.topic_impl
            .upgrade()?
            .write()
            .set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        self.topic_impl.upgrade()?.read().get_listener()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        self.topic_impl.upgrade()?.read().get_statuscondition()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        self.topic_impl.upgrade()?.read().get_status_changes()
    }

    fn enable(&self) -> DDSResult<()> {
        self.topic_impl.upgrade()?.read().enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        self.topic_impl.upgrade()?.read().get_instance_handle()
    }
}
