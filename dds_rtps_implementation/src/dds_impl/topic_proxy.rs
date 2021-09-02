use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipant,
    infrastructure::entity::{Entity, StatusCondition},
    return_type::DDSResult,
    topic::{topic::Topic, topic_description::TopicDescription},
};

use crate::utils::shared_object::RtpsWeak;

pub struct TopicProxy<'t, T, TT> {
    participant: &'t dyn DomainParticipant,
    topic_impl: RtpsWeak<TT>,
    phantom: PhantomData<&'t T>,
}

impl<'t, T, TT> TopicProxy<'t, T, TT> {
    pub fn new(participant: &'t dyn DomainParticipant, topic_impl: RtpsWeak<TT>) -> Self {
        Self {
            participant,
            topic_impl,
            phantom: PhantomData,
        }
    }
}

impl<'t, T, TT> Topic<T> for TopicProxy<'t, T, TT>
where
    TT: Topic<T>,
{
    fn get_inconsistent_topic_status(&self) -> DDSResult<InconsistentTopicStatus> {
        self.topic_impl.upgrade()?.get_inconsistent_topic_status()
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
        self.topic_impl.upgrade()?.get_type_name()
    }

    fn get_name(&self) -> DDSResult<&'static str> {
        self.topic_impl.upgrade()?.get_name()
    }
}

impl<'t, T, TT> Entity for TopicProxy<'t, T, TT>
where
    TT: Entity,
{
    type Qos = TT::Qos;
    type Listener = TT::Listener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.topic_impl.upgrade()?.set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        self.topic_impl.upgrade()?.get_qos()
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        self.topic_impl.upgrade()?.set_listener(a_listener, mask)
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        self.topic_impl.upgrade()?.get_listener()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        self.topic_impl.upgrade()?.get_statuscondition()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        self.topic_impl.upgrade()?.get_status_changes()
    }

    fn enable(&self) -> DDSResult<()> {
        self.topic_impl.upgrade()?.enable()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        self.topic_impl.upgrade()?.get_instance_handle()
    }
}
