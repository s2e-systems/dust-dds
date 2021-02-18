use std::sync::Arc;

use crate::{inner::rtps_topic_inner::RtpsTopicImpl, utils::node::Node};
use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    dds_type::DDSType,
    domain::domain_participant::DomainParticipantChild,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::{DDSResult},
    topic::{topic::Topic, topic_description::TopicDescription, topic_listener::TopicListener},
};

use super::rtps_domain_participant::RtpsDomainParticipant;

pub type RtpsTopic<'a, T> = Node<'a, &'a RtpsDomainParticipant, Arc<RtpsTopicImpl<T>>>;

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
        &self.get_parent()
    }

    fn get_type_name(&self) -> DDSResult<&str> {
        todo!()
    }

    fn get_name(&self) -> DDSResult<String> {
        todo!()
    }
}

impl<'a, T: DDSType> Entity for RtpsTopic<'a, T> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener + 'a>;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        // self.topic_ref.set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // self.topic_ref.get_qos()
        todo!()
    }

    fn set_listener(&self, _a_listener: Self::Listener, _mask: StatusMask) -> DDSResult<()> {
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

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_type_name() {
        // let parent_participant = RtpsDomainParticipant::new(domain_id, qos, userdata_transport, metatraffic_transport, a_listener, mask);
        // let topic_ref =
        // let topic = RtpsTopic{parent_participant, topic_ref, phantom_data:PhantomData};
    }
}
