use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    dds_type::DDSType,
    domain::domain_participant::DomainParticipantChild,
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::DDSResult,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};

use super::domain_participant_impl::DomainParticipantImpl;

pub struct TopicImpl<'a, PSM: rust_rtps_pim::structure::Types, T: DDSType> {
    parent: &'a DomainParticipantImpl<PSM>,
    phantom: PhantomData<&'a T>,
}

impl<'a, PSM: rust_rtps_pim::structure::Types, T: DDSType> TopicImpl<'a, PSM, T> {
    pub(crate) fn new(parent: &'a DomainParticipantImpl<PSM>) -> Self {
        Self {
            parent,
            phantom: PhantomData,
        }
    }
}

impl<'a, PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types, T: DDSType>
    DomainParticipantChild<'a> for TopicImpl<'a, PSM, T>
{
    type DomainParticipantType = DomainParticipantImpl<PSM>;
}

impl<'a, PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types, T: DDSType>
    rust_dds_api::topic::topic::Topic<'a> for TopicImpl<'a, PSM, T>
{
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'a, PSM: rust_rtps_pim::structure::Types + rust_rtps_pim::behavior::Types, T: DDSType>
    TopicDescription<'a> for TopicImpl<'a, PSM, T>
{
    fn get_participant(&self) -> &<Self as DomainParticipantChild<'a>>::DomainParticipantType {
        self.parent
    }

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

    fn get_name(&self) -> DDSResult<String> {
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

impl<'a, PSM: rust_rtps_pim::structure::Types, T: DDSType> Entity for TopicImpl<'a, PSM, T> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener>;

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
