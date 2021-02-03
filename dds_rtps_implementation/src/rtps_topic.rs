use std::{
    any::Any,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::{
    inner::rtps_topic_inner::RtpsAnyTopicInnerRef,
    utils::{
        as_any::AsAny,
        maybe_valid::{MaybeValid, MaybeValidRef},
    },
};
use rust_dds_api::{
    domain::domain_participant::{DomainParticipant, DomainParticipantChild},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
        status::{InconsistentTopicStatus, StatusMask},
    },
    publication::publisher::Publisher,
    subscription::subscriber::Subscriber,
    topic::{
        topic::Topic,
        topic_description::{AnyTopic, TopicDescription},
        topic_listener::TopicListener,
    },
};
use rust_dds_types::{DDSType, InstanceHandle, ReturnCode, ReturnCodes, TopicKind};
use rust_rtps::types::GUID;

use super::rtps_domain_participant::RtpsDomainParticipant;

pub struct RtpsTopic<'a, T: DDSType> {
    pub(crate) parent_participant: &'a RtpsDomainParticipant,
    pub(crate) topic_ref: RtpsAnyTopicInnerRef<'a>,
    pub(crate) phantom_data: PhantomData<T>,
}

impl<'a, T: DDSType> RtpsTopic<'a, T> {
    pub fn new(
        parent_participant: &'a RtpsDomainParticipant,
        topic_ref: RtpsAnyTopicInnerRef<'a>,
    ) -> Self {
        Self {
            parent_participant,
            topic_ref,
            phantom_data: PhantomData,
        }
    }
}

impl<'a, T: DDSType> DomainParticipantChild<'a> for RtpsTopic<'a, T> {
    type DomainParticipantType = RtpsDomainParticipant;
}

impl<'a, T: DDSType> Topic<'a, T> for RtpsTopic<'a, T> {
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> ReturnCode<()> {
        todo!()
    }
}

impl<'a, T: DDSType> TopicDescription<'a, T> for RtpsTopic<'a, T> {
    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        &self.parent_participant
    }

    fn get_type_name(&self) -> ReturnCode<&str> {
        todo!()
    }

    fn get_name(&self) -> ReturnCode<String> {
        todo!()
    }
}

impl<'a, T: DDSType> Entity for RtpsTopic<'a, T> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener<T>>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> ReturnCode<()> {
        self.topic_ref.set_qos(qos)
    }

    fn get_qos(&self) -> ReturnCode<Self::Qos> {
        self.topic_ref.get_qos()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> ReturnCode<()> {
        todo!()
    }

    fn get_listener(&self) -> &Self::Listener {
        todo!()
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> ReturnCode<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> ReturnCode<InstanceHandle> {
        todo!()
    }
}

impl<'a, T: DDSType> AnyTopic for RtpsTopic<'a, T> {}
