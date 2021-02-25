use crate::rtps_domain_participant::{RtpsDomainParticipant, RtpsTopic};
use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    dds_type::DDSType,
    domain::domain_participant::DomainParticipantChild,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::{DDSError, DDSResult},
    topic::{topic::Topic, topic_description::TopicDescription, topic_listener::TopicListener},
};

impl<'a, T: DDSType> DomainParticipantChild<'a> for RtpsTopic<'a, T> {
    type DomainParticipantType = RtpsDomainParticipant;
}

impl<'a, T: DDSType> Topic<'a> for RtpsTopic<'a, T> {
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a, T: DDSType> TopicDescription<'a> for RtpsTopic<'a, T> {
    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        self.parent.0
    }

    fn get_type_name(&self) -> DDSResult<&'static str> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .get_type_name())
    }

    fn get_name(&self) -> DDSResult<String> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .get_name())
    }
}

impl<'a, T: DDSType> Entity for RtpsTopic<'a, T> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        self.impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .set_qos(qos)
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .get_qos())
    }

    fn set_listener(&self, a_listener: Option<Self::Listener>, mask: StatusMask) -> DDSResult<()> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .set_listener(a_listener, mask))
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        Ok(self
            .impl_ref
            .upgrade()
            .ok_or(DDSError::AlreadyDeleted)?
            .get_listener())
    }

    fn get_statuscondition(&self) -> StatusCondition {
        todo!()
    }

    fn get_status_changes(&self) -> StatusMask {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}
