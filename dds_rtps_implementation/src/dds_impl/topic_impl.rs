use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::{Duration, InconsistentTopicStatus, InstanceHandle, StatusMask},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::DDSResult,
    topic::{topic_description::TopicDescription, topic_listener::TopicListener},
};

use super::domain_participant_impl::DomainParticipantImpl;

pub struct TopicImpl<'t, 'dp: 't, T: 't, PSM: rust_rtps_pim::PIM> {
    participant: &'t DomainParticipantImpl<'dp, PSM>,
    phantom: PhantomData<&'t T>,
}

impl<'t, 'dp: 't, T: 't, PSM: rust_rtps_pim::PIM>
    rust_dds_api::domain::domain_participant::TopicFactory<'t, 'dp, T>
    for DomainParticipantImpl<'dp, PSM>
{
    type TopicType = TopicImpl<'t, 'dp, T, PSM>;

    fn create_topic(
        &'t self,
        _topic_name: &str,
        _qos: Option<TopicQos>,
        _a_listener: Option<&'t (dyn TopicListener<DataType = T> + 't)>,
        _mask: StatusMask,
    ) -> Option<Self::TopicType> {
        todo!()
    }

    fn delete_topic(&self, _a_topic: &Self::TopicType) -> DDSResult<()> {
        todo!()
    }

    fn find_topic(&self, _topic_name: &str, _timeout: Duration) -> Option<Self::TopicType> {
        todo!()
    }
}

impl<'t, 'dp: 't, T: 't, PSM: rust_rtps_pim::PIM> rust_dds_api::topic::topic::Topic<'t, 'dp, T>
    for TopicImpl<'t, 'dp, T, PSM>
{
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'t, 'dp: 't, T: 't, PSM: rust_rtps_pim::PIM> TopicDescription<'t, 'dp, T>
    for TopicImpl<'t, 'dp, T, PSM>
{
    fn get_participant(
        &self,
    ) -> &dyn rust_dds_api::domain::domain_participant::DomainParticipant<'dp> {
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

impl<'t, 'dp: 't, T: 't, PSM: rust_rtps_pim::PIM> Entity for TopicImpl<'t, 'dp, T, PSM> {
    type Qos = TopicQos<'t>;
    type Listener = &'t (dyn TopicListener<DataType = T> + 't);

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
