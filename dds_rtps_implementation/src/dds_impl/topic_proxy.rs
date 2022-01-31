use std::marker::PhantomData;

use crate::utils::shared_object::{rtps_shared_read_lock, rtps_weak_upgrade, RtpsWeak};
use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::DDSResult,
    topic::{topic::Topic, topic_description::TopicDescription, topic_listener::TopicListener},
};

use super::domain_participant_proxy::DomainParticipantProxy;

pub struct TopicAttributes {
    pub _qos: TopicQos,
    pub type_name: &'static str,
    pub topic_name: String,
}

impl TopicAttributes {
    pub fn new(qos: TopicQos, type_name: &'static str, topic_name: &str) -> Self {
        Self {
            _qos: qos,
            type_name,
            topic_name: topic_name.to_string(),
        }
    }
}

pub struct TopicProxy<Foo> {
    participant: DomainParticipantProxy,
    topic_impl: RtpsWeak<TopicAttributes>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo> Clone for TopicProxy<Foo> {
    fn clone(&self) -> Self {
        Self {
            participant: self.participant.clone(),
            topic_impl: self.topic_impl.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<Foo> TopicProxy<Foo> {
    pub fn new(participant: DomainParticipantProxy, topic_impl: RtpsWeak<TopicAttributes>) -> Self {
        Self {
            participant,
            topic_impl,
            phantom: PhantomData,
        }
    }
}

impl<Foo> AsRef<RtpsWeak<TopicAttributes>> for TopicProxy<Foo> {
    fn as_ref(&self) -> &RtpsWeak<TopicAttributes> {
        &self.topic_impl
    }
}

impl<Foo> Topic for TopicProxy<Foo> {
    fn get_inconsistent_topic_status(&self) -> DDSResult<InconsistentTopicStatus> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_inconsistent_topic_status()
        todo!()
    }
}

impl<Foo> TopicDescription for TopicProxy<Foo> {
    type DomainParticipant = DomainParticipantProxy;

    fn get_participant(&self) -> Self::DomainParticipant {
        self.participant.clone()
    }

    fn get_type_name(&self) -> DDSResult<&'static str> {
        Ok(rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).type_name)
    }

    fn get_name(&self) -> DDSResult<String> {
        Ok(rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?)
            .topic_name
            .clone())
    }
}

impl<Foo> Entity for TopicProxy<Foo> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener>;

    fn set_qos(&mut self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.topic_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.topic_impl)?).set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).enable()
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_instance_handle()
        todo!()
    }
}
