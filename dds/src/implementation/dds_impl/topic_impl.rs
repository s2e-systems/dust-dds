use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        rtps::types::Guid,
        utils::shared_object::{DdsRwLock, DdsShared, DdsWeak},
    },
    infrastructure::{
        condition::StatusCondition,
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::{any_topic_listener::AnyTopicListener, domain_participant_impl::DomainParticipantImpl};

impl InconsistentTopicStatus {
    fn increment(&mut self) {
        self.total_count += 1;
        self.total_count_change += 1;
    }

    fn read_and_reset(&mut self) -> Self {
        let status = self.clone();
        self.total_count_change = 0;
        status
    }
}

pub struct TopicImpl {
    guid: Guid,
    qos: DdsRwLock<TopicQos>,
    type_name: &'static str,
    topic_name: String,
    parent_participant: DdsWeak<DomainParticipantImpl>,
    enabled: DdsRwLock<bool>,
    listener: DdsRwLock<Option<Box<dyn AnyTopicListener + Send + Sync>>>,
    listener_status_mask: DdsRwLock<Vec<StatusKind>>,
    inconsistent_topic_status: DdsRwLock<InconsistentTopicStatus>,
}

impl TopicImpl {
    pub fn new(
        guid: Guid,
        qos: TopicQos,
        type_name: &'static str,
        topic_name: &str,
        listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: &[StatusKind],
        parent_participant: DdsWeak<DomainParticipantImpl>,
    ) -> DdsShared<Self> {
        DdsShared::new(Self {
            guid,
            qos: DdsRwLock::new(qos),
            type_name,
            topic_name: topic_name.to_string(),
            parent_participant,
            enabled: DdsRwLock::new(false),
            listener: DdsRwLock::new(listener),
            listener_status_mask: DdsRwLock::new(mask.to_vec()),
            inconsistent_topic_status: DdsRwLock::new(InconsistentTopicStatus::default()),
        })
    }
}

impl DdsShared<TopicImpl> {
    pub fn get_inconsistent_topic_status(&self) -> InconsistentTopicStatus {
        self.inconsistent_topic_status.write_lock().read_and_reset()
    }

    pub fn get_participant(&self) -> DdsShared<DomainParticipantImpl> {
        self.parent_participant
            .upgrade()
            .expect("Parent participant of topic must exist")
    }

    pub fn get_type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn get_name(&self) -> String {
        self.topic_name.clone()
    }

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

    pub fn get_qos(&self) -> TopicQos {
        self.qos.read_lock().clone()
    }

    pub fn set_listener(
        &self,
        a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: &[StatusKind],
    ) {
        *self.listener.write_lock() = a_listener;
        *self.listener_status_mask.write_lock() = mask.to_vec();
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
        if !self.get_participant().is_enabled() {
            return Err(DdsError::PreconditionNotMet(
                "Parent participant is disabled".to_string(),
            ));
        }

        self.get_participant()
            .announce_topic(self.as_discovered_topic_data());

        *self.enabled.write_lock() = true;
        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }

    pub fn as_discovered_topic_data(&self) -> DiscoveredTopicData {
        let qos = self.qos.read_lock();
        DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.guid.into(),
                },
                name: self.topic_name.to_string(),
                type_name: self.type_name.to_string(),
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

    pub fn process_discovered_topic(&self, discovered_topic_data: &DiscoveredTopicData) {
        if discovered_topic_data.topic_builtin_topic_data.type_name == self.type_name
            && discovered_topic_data.topic_builtin_topic_data.name == self.topic_name
        {
            if !self.is_discovered_topic_consistent(discovered_topic_data) {
                self.inconsistent_topic_status.write_lock().increment();
                todo!("Trigger listener if enabled")
            }
        }
    }

    fn is_discovered_topic_consistent(&self, discovered_topic_data: &DiscoveredTopicData) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::rtps::types::{
        EntityId, EntityKey, GuidPrefix, BUILT_IN_PARTICIPANT,
    };

    use super::*;

    #[test]
    fn get_instance_handle() {
        let guid = Guid::new(
            GuidPrefix::new([2; 12]),
            EntityId::new(EntityKey::new([3; 3]), BUILT_IN_PARTICIPANT),
        );
        let topic = TopicImpl::new(guid, TopicQos::default(), "", "", None, &[], DdsWeak::new());
        *topic.enabled.write_lock() = true;

        let expected_instance_handle: InstanceHandle = guid.into();
        let instance_handle = topic.get_instance_handle();
        assert_eq!(expected_instance_handle, instance_handle);
    }
}
