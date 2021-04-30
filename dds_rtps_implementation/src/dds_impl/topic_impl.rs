use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipantChild,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::DDSResult,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};

use super::domain_participant_impl::DomainParticipantImpl;

pub struct TopicImpl<'a, PSM: rust_rtps_pim::PIM, T> {
    parent: &'a DomainParticipantImpl<'a, PSM>,
    phantom: PhantomData<&'a T>,
}

impl<'a, PSM: rust_rtps_pim::PIM, T> TopicImpl<'a, PSM, T> {
    pub(crate) fn new(parent: &'a DomainParticipantImpl<'a, PSM>) -> Self {
        Self {
            parent,
            phantom: PhantomData,
        }
    }
}

impl<'a, PSM: rust_rtps_pim::PIM, T> DomainParticipantChild<'a> for TopicImpl<'a, PSM, T> {
    type DomainParticipantType = DomainParticipantImpl<'a, PSM>;

    fn get_participant(&self) -> &Self::DomainParticipantType {
        self.parent
    }
}

impl<'a, PSM: rust_rtps_pim::PIM, T> rust_dds_api::topic::topic::Topic<'a, T>
    for TopicImpl<'a, PSM, T>
{
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a, PSM: rust_rtps_pim::PIM, T> TopicDescription<'a, T> for TopicImpl<'a, PSM, T> {
    fn get_type_name(&self) -> DDSResult<&'static str> {
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .get_type_name())
        todo!()
    }

    fn get_name(&self) -> DDSResult<&'a str> {
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .get_name())
        todo!()
    }
}

impl<'a, PSM: rust_rtps_pim::PIM, T> Entity for TopicImpl<'a, PSM, T> {
    type Qos = TopicQos<'a>;
    type Listener = &'a (dyn TopicListener<DataType = T> + 'a);

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DDSResult<()> {
        // self.impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .get_qos())
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DDSResult<()> {
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .set_listener(a_listener, mask))
        todo!()
    }

    fn get_listener(&self) -> DDSResult<Option<Self::Listener>> {
        // Ok(self
        //     .impl_ref
        //     .upgrade()
        //     .ok_or(DDSError::AlreadyDeleted)?
        //     .lock()
        //     .unwrap()
        //     .get_listener())
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
