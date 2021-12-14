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

use super::topic_impl::TopicImpl;

pub struct TopicProxy<'t, Foo> {
    participant: &'t dyn DomainParticipant,
    topic_impl: RtpsWeak<TopicImpl>,
    phantom: PhantomData<&'t Foo>,
}

impl<'t, Foo> TopicProxy<'t, Foo> {
    pub fn new(participant: &'t dyn DomainParticipant, topic_impl: RtpsWeak<TopicImpl>) -> Self {
        Self {
            participant,
            topic_impl,
            phantom: PhantomData,
        }
    }
}

impl<Foo> AsRef<RtpsWeak<TopicImpl>> for TopicProxy<'_, Foo> {
    fn as_ref(&self) -> &RtpsWeak<TopicImpl> {
        &self.topic_impl
    }
}

impl<'t, Foo> Topic<Foo> for TopicProxy<'t, Foo> {
    fn get_inconsistent_topic_status(&self) -> DDSResult<InconsistentTopicStatus> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_inconsistent_topic_status()
    }
}

impl<'t, Foo> TopicDescription<Foo> for TopicProxy<'t, Foo> {
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

impl<'t, Foo> Entity for TopicProxy<'t, Foo> {
    type Qos = <TopicImpl as Entity>::Qos;
    type Listener = <TopicImpl as Entity>::Listener;

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
