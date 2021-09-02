use std::marker::PhantomData;

use rust_dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    domain::domain_participant::DomainParticipant,
    infrastructure::entity::{Entity, StatusCondition},
    return_type::DDSResult,
    topic::{topic::Topic, topic_description::TopicDescription},
};

use crate::utils::shared_object::RtpsWeak;

pub struct TopicProxy<'t, T, TT> {
    participant: &'t dyn DomainParticipant,
    topic_impl: RtpsWeak<TT>,
    phantom: PhantomData<&'t T>,
}

impl<'t, T, TT> TopicProxy<'t, T, TT> {
    pub fn new(participant: &'t dyn DomainParticipant, topic_impl: RtpsWeak<TT>) -> Self {
        Self {
            participant,
            topic_impl,
            phantom: PhantomData,
        }
    }
}

impl<'t, T, TT> Topic<T> for TopicProxy<'t, T, TT> {
    fn get_inconsistent_topic_status(
        &self,
        _status: &mut InconsistentTopicStatus,
    ) -> DDSResult<()> {
        todo!()
    }
}

impl<'t, T, TT> TopicDescription<T> for TopicProxy<'t, T, TT> {
    fn get_participant(&self) -> &dyn DomainParticipant {
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

impl<'t, T, TT> Entity for TopicProxy<'t, T, TT>
where
    TT: Entity,
{
    type Qos = TT::Qos;
    type Listener = TT::Listener;

    fn set_qos(&self, qos: Option<Self::Qos>) -> DDSResult<()> {
        // self.topic_storage.upgrade()?.lock().set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DDSResult<Self::Qos> {
        // Ok(self.topic_storage.upgrade()?.lock().get_qos().clone())
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

    fn get_statuscondition(&self) -> DDSResult<StatusCondition> {
        todo!()
    }

    fn get_status_changes(&self) -> DDSResult<StatusMask> {
        todo!()
    }

    fn enable(&self) -> DDSResult<()> {
        todo!()
    }

    fn get_instance_handle(&self) -> DDSResult<InstanceHandle> {
        todo!()
    }
}
