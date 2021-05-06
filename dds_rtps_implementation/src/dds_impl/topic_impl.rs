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

use super::domain_participant_impl::DomainParticipantImpl;

pub struct TopicImpl<'topic, 'participant: 'topic, T: 'topic, PSM: rust_rtps_pim::PIM> {
    parent: &'topic DomainParticipantImpl<'participant, PSM>,
    phantom: PhantomData<&'topic T>,
}

impl<'topic, 'participant: 'topic, T: 'topic, PSM: rust_rtps_pim::PIM>
    TopicImpl<'topic, 'participant, T, PSM>
{
    pub(crate) fn new(parent: &'topic DomainParticipantImpl<'participant, PSM>) -> Self {
        Self {
            parent,
            phantom: PhantomData,
        }
    }
}

impl<'topic, 'participant: 'topic, T: 'topic, PSM: rust_rtps_pim::PIM>
    rust_dds_api::topic::topic::Topic<'topic, 'participant, T>
    for TopicImpl<'topic, 'participant, T, PSM>
{
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'topic, 'participant: 'topic, T: 'topic, PSM: rust_rtps_pim::PIM>
    TopicDescription<'topic, 'participant, T> for TopicImpl<'topic, 'participant, T, PSM>
{
    fn get_participant(&self) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant<'participant> {
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

    fn get_name(&self) -> DDSResult<&'topic str> {
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

impl<'topic, 'participant: 'topic, T: 'topic, PSM: rust_rtps_pim::PIM> Entity
    for TopicImpl<'topic, 'participant, T, PSM>
{
    type Qos = TopicQos<'topic>;
    type Listener = &'topic (dyn TopicListener<DataType = T> + 'topic);

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
