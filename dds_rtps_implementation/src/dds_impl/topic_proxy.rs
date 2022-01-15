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

use super::{domain_participant_proxy::DomainParticipantProxy, topic_impl::TopicImpl};

pub struct TopicProxy<Foo> {
    participant: DomainParticipantProxy,
    topic_impl: RtpsWeak<TopicImpl<Foo>>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo> Clone for TopicProxy<Foo> {
    fn clone(&self) -> Self {
        Self {
            participant: self.participant.clone(),
            topic_impl: self.topic_impl.clone(),
        }
    }
}

impl<Foo> TopicProxy<Foo> {
    pub fn new(participant: DomainParticipantProxy, topic_impl: RtpsWeak<TopicImpl<Foo>>) -> Self {
        Self {
            participant,
            topic_impl,
        }
    }
}

impl<Foo> AsRef<RtpsWeak<TopicImpl<Foo>>> for TopicProxy<Foo> {
    fn as_ref(&self) -> &RtpsWeak<TopicImpl<Foo>> {
        &self.topic_impl
    }
}

impl<Foo> Topic<Foo> for TopicProxy<Foo> {
    fn get_inconsistent_topic_status(&self) -> DDSResult<InconsistentTopicStatus> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_inconsistent_topic_status()
    }
}

impl<Foo> TopicDescription<Foo> for TopicProxy<Foo> {
    fn get_participant(&self) -> &dyn DomainParticipant {
        // self.participant
        todo!()
    }

    fn get_type_name(&self) -> DDSResult<&'static str> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_type_name()
    }

    fn get_name(&self) -> DDSResult<String> {
        rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_name()
    }
}

impl<Foo> Entity for TopicProxy<Foo> {
    type Qos = <TopicImpl<Foo> as Entity>::Qos;
    type Listener = <TopicImpl<Foo> as Entity>::Listener;

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
