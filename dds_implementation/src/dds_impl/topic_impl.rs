use dds_api::{
    dcps_psm::{InconsistentTopicStatus, InstanceHandle, StatusMask},
    infrastructure::{
        entity::{Entity, StatusCondition},
        qos::TopicQos,
    },
    return_type::{DdsError, DdsResult},
    topic::{topic::Topic, topic_description::TopicDescription, topic_listener::TopicListener},
};
use rtps_pim::structure::types::Guid;

use crate::utils::shared_object::{DdsRwLock, DdsShared, DdsWeak};

use super::domain_participant_impl::DomainParticipantImpl;

pub struct TopicImpl {
    guid: Guid,
    _qos: TopicQos,
    type_name: &'static str,
    topic_name: String,
    parent_participant: DdsWeak<DomainParticipantImpl>,
    enabled: DdsRwLock<bool>,
}

impl TopicImpl {
    pub fn new(
        guid: Guid,
        qos: TopicQos,
        type_name: &'static str,
        topic_name: &str,
        parent_participant: DdsWeak<DomainParticipantImpl>,
    ) -> DdsShared<Self> {
        DdsShared::new(Self {
            guid,
            _qos: qos,
            type_name,
            topic_name: topic_name.to_string(),
            parent_participant,
            enabled: DdsRwLock::new(false),
        })
    }
}

impl Topic for DdsShared<TopicImpl> {
    fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_inconsistent_topic_status()
        todo!()
    }
}

impl TopicDescription for DdsShared<TopicImpl> {
    type DomainParticipant = DdsShared<DomainParticipantImpl>;

    fn get_participant(&self) -> DdsResult<Self::DomainParticipant> {
        Ok(self
            .parent_participant
            .upgrade()
            .expect("Failed to get parent participant of topic"))
    }

    fn get_type_name(&self) -> DdsResult<&'static str> {
        Ok(self.type_name)
    }

    fn get_name(&self) -> DdsResult<String> {
        Ok(self.topic_name.clone())
    }
}

impl Entity for DdsShared<TopicImpl> {
    type Qos = TopicQos;
    type Listener = Box<dyn TopicListener>;

    fn set_qos(&self, _qos: Option<Self::Qos>) -> DdsResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.topic_impl)?).set_qos(qos)
        todo!()
    }

    fn get_qos(&self) -> DdsResult<Self::Qos> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_qos()
        todo!()
    }

    fn set_listener(
        &self,
        _a_listener: Option<Self::Listener>,
        _mask: StatusMask,
    ) -> DdsResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.topic_impl)?).set_listener(a_listener, mask)
        todo!()
    }

    fn get_listener(&self) -> DdsResult<Option<Self::Listener>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_listener()
        todo!()
    }

    fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_statuscondition()
        todo!()
    }

    fn get_status_changes(&self) -> DdsResult<StatusMask> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_status_changes()
        todo!()
    }

    fn enable(&self) -> DdsResult<()> {
        *self.enabled.write_lock() = true;
        Ok(())
    }

    fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        if !*self.enabled.read_lock() {
            return Err(DdsError::NotEnabled);
        }

        Ok(self.guid.into())
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::structure::types::{EntityId, GuidPrefix};

    use super::*;

    #[test]
    fn get_instance_handle() {
        let guid = Guid::new(
            GuidPrefix([2; 12]),
            EntityId {
                entity_key: [3; 3],
                entity_kind: 1,
            },
        );
        let topic = TopicImpl::new(guid, TopicQos::default(), "", "", DdsWeak::new());
        topic.enable().unwrap();

        let expected_instance_handle: [u8; 16] = guid.into();
        let instance_handle = topic.get_instance_handle().unwrap();
        assert_eq!(expected_instance_handle, instance_handle);
    }
}
