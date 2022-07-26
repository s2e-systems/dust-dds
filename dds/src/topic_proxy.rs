use std::marker::PhantomData;

use crate::api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::DdsResult,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener, Topic},
};
use crate::implementation::{dds_impl::topic_impl::TopicImpl, utils::shared_object::DdsWeak};

use crate::domain_participant_proxy::DomainParticipantProxy;

pub struct TopicProxy<Foo> {
    topic_attributes: DdsWeak<TopicImpl>,
    phantom: PhantomData<Foo>,
}

// Not automatically derived because in that case it is only available if Foo: Clone
impl<Foo> Clone for TopicProxy<Foo> {
    fn clone(&self) -> Self {
        Self {
            topic_attributes: self.topic_attributes.clone(),
            phantom: self.phantom,
        }
    }
}

impl<Foo> TopicProxy<Foo> {
    pub fn new(topic_attributes: DdsWeak<TopicImpl>) -> Self {
        Self {
            topic_attributes,
            phantom: PhantomData,
        }
    }
}

impl<Foo> AsRef<DdsWeak<TopicImpl>> for TopicProxy<Foo> {
    fn as_ref(&self) -> &DdsWeak<TopicImpl> {
        &self.topic_attributes
    }
}

impl<Foo> Topic for TopicProxy<Foo> {
    fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        self.topic_attributes
            .upgrade()?
            .get_inconsistent_topic_status()
    }
}

impl<Foo> TopicDescription for TopicProxy<Foo> {
    type DomainParticipant = DomainParticipantProxy;

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

impl<Foo> Entity for TopicProxy<Foo> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener>;

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
