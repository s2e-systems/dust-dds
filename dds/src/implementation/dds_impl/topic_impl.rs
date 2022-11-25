use crate::builtin_topics::BuiltInTopicKey;
use crate::implementation::rtps::types::Guid;
use crate::infrastructure::instance::InstanceHandle;
use crate::infrastructure::qos::QosKind;
use crate::infrastructure::status::{InconsistentTopicStatus, StatusKind};
use crate::topic_definition::topic_listener::TopicListener;
use crate::{
    builtin_topics::TopicBuiltinTopicData,
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        qos::TopicQos,
    },
};

use crate::implementation::{
    data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
    utils::shared_object::{DdsRwLock, DdsShared, DdsWeak},
};

use super::domain_participant_impl::DomainParticipantImpl;

pub trait AnyTopicListener {
    fn trigger_on_inconsistent_topic(
        &mut self,
        _the_topic: &DdsShared<TopicImpl>,
        _status: InconsistentTopicStatus,
    );
}

impl<Foo> AnyTopicListener for Box<dyn TopicListener<Foo = Foo>> {
    fn trigger_on_inconsistent_topic(
        &mut self,
        _the_topic: &DdsShared<TopicImpl>,
        _status: InconsistentTopicStatus,
    ) {
        todo!()
    }
}

pub struct TopicImpl {
    guid: Guid,
    qos: DdsRwLock<TopicQos>,
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
            qos: DdsRwLock::new(qos),
            type_name,
            topic_name: topic_name.to_string(),
            parent_participant,
            enabled: DdsRwLock::new(false),
        })
    }
}

impl DdsShared<TopicImpl> {
    pub fn get_inconsistent_topic_status(&self) -> DdsResult<InconsistentTopicStatus> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_inconsistent_topic_status()
        todo!()
    }

    pub fn get_participant(&self) -> DdsResult<DdsShared<DomainParticipantImpl>> {
        Ok(self
            .parent_participant
            .upgrade()
            .expect("Failed to get parent participant of topic"))
    }

    pub fn get_type_name(&self) -> DdsResult<&'static str> {
        Ok(self.type_name)
    }

    pub fn get_name(&self) -> DdsResult<String> {
        Ok(self.topic_name.clone())
    }
}

impl DdsShared<TopicImpl> {
    pub fn set_qos(&self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        qos.is_consistent()?;
        if *self.enabled.read_lock() {
            self.qos.read_lock().check_immutability(&qos)?;
        }

        *self.qos.write_lock() = qos;

        Ok(())
    }

    pub fn get_qos(&self) -> DdsResult<TopicQos> {
        Ok(self.qos.read_lock().clone())
    }

    pub fn set_listener(
        &self,
        _a_listener: Option<Box<dyn AnyTopicListener>>,
        _mask: &[StatusKind],
    ) -> DdsResult<()> {
        // rtps_shared_write_lock(&rtps_weak_upgrade(&self.topic_impl)?).set_listener(a_listener, mask)
        todo!()
    }

    pub fn get_statuscondition(&self) -> DdsResult<StatusCondition> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_statuscondition()
        todo!()
    }

    pub fn get_status_changes(&self) -> DdsResult<Vec<StatusKind>> {
        // rtps_shared_read_lock(&rtps_weak_upgrade(&self.topic_impl)?).get_status_changes()
        todo!()
    }

    pub fn enable(&self) -> DdsResult<()> {
        if !self.parent_participant.upgrade()?.is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent participant is disabled".to_string(),
            ));
        }

        self.parent_participant
            .upgrade()?
            .announce_topic(self.into());

        *self.enabled.write_lock() = true;
        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }
}

impl From<&DdsShared<TopicImpl>> for DiscoveredTopicData {
    fn from(val: &DdsShared<TopicImpl>) -> Self {
        let qos = val.qos.read_lock();
        DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: val.guid.into(),
                },
                name: val.topic_name.to_string(),
                type_name: val.type_name.to_string(),
                durability: qos.durability.clone(),
                deadline: qos.deadline.clone(),
                latency_budget: qos.latency_budget.clone(),
                liveliness: qos.liveliness.clone(),
                reliability: qos.reliability.clone(),
                transport_priority: qos.transport_priority.clone(),
                lifespan: qos.lifespan.clone(),
                destination_order: qos.destination_order.clone(),
                history: qos.history.clone(),
                resource_limits: qos.resource_limits.clone(),
                ownership: qos.ownership.clone(),
                topic_data: qos.topic_data.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::rtps::types::{EntityId, GuidPrefix};

    use super::*;

    #[test]
    fn get_instance_handle() {
        let guid = Guid::new(GuidPrefix::from([2; 12]), EntityId::new([3; 3], 1));
        let topic = TopicImpl::new(guid, TopicQos::default(), "", "", DdsWeak::new());
        *topic.enabled.write_lock() = true;

        let expected_instance_handle: InstanceHandle = guid.into();
        let instance_handle = topic.get_instance_handle();
        assert_eq!(expected_instance_handle, instance_handle);
    }
}
