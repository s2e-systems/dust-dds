use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::DDSResult,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};
use rust_rtps_pim::structure::types::GUIDType;

use super::{domain_participant_impl::DomainParticipantImpl, PIM};

pub struct TopicImpl<'t, T: 'static, PSM: PIM> {
    participant: &'t DomainParticipantImpl<PSM>,
    phantom: PhantomData<&'t T>,
}

impl<'t, T: 'static, PSM: PIM> rust_dds_api::topic::topic::Topic<T> for TopicImpl<'t, T, PSM>
where
    PSM::GUIDType: GUIDType<PSM> + Send + Copy,
    PSM::SequenceNumberType: Copy + Ord + Send,
    PSM::GuidPrefixType: Clone,
    PSM::LocatorType: Clone + PartialEq + Send,
    PSM::DataType: Send,
    PSM::DurationType: Send,
    PSM::EntityIdType: Send,
    PSM::InstanceHandleType: Send,
    PSM::ParameterListSubmessageElementType: Clone + Send,
{
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'t, T: 'static, PSM: PIM> TopicDescription<T> for TopicImpl<'t, T, PSM>
where
    PSM::GUIDType: GUIDType<PSM> + Send + Copy,
    PSM::SequenceNumberType: Copy + Ord + Send,
    PSM::GuidPrefixType: Clone,
    PSM::LocatorType: Clone + PartialEq + Send,
    PSM::DataType: Send,
    PSM::DurationType: Send,
    PSM::EntityIdType: Send,
    PSM::InstanceHandleType: Send,
    PSM::ParameterListSubmessageElementType: Clone + Send,
{
    fn get_participant(&self) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant {
        self.participant
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

    fn get_name(&self) -> DDSResult<&'t str> {
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

impl<'t, T: 'static, PSM: PIM> Entity for TopicImpl<'t, T, PSM> {
    type Qos = TopicQos;
    type Listener = &'static dyn TopicListener<DataPIM = T>;

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
