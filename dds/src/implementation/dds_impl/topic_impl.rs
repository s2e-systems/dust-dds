use std::sync::mpsc::SyncSender;

use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        rtps::types::Guid,
        utils::shared_object::{DdsRwLock, DdsShared},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::{
    any_topic_listener::AnyTopicListener, domain_participant_impl::AnnounceKind,
    node_listener_topic::ListenerTopicNode, status_condition_impl::StatusConditionImpl,
    status_listener::StatusListener,
};

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
    enabled: DdsRwLock<bool>,
    status_listener: DdsRwLock<StatusListener<dyn AnyTopicListener + Send + Sync>>,
    inconsistent_topic_status: DdsRwLock<InconsistentTopicStatus>,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
    announce_sender: SyncSender<AnnounceKind>,
}

impl TopicImpl {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        guid: Guid,
        qos: TopicQos,
        type_name: &'static str,
        topic_name: &str,
        listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: &[StatusKind],
        announce_sender: SyncSender<AnnounceKind>,
    ) -> DdsShared<Self> {
        DdsShared::new(Self {
            guid,
            qos: DdsRwLock::new(qos),
            type_name,
            topic_name: topic_name.to_string(),
            enabled: DdsRwLock::new(false),
            status_listener: DdsRwLock::new(StatusListener::new(listener, mask)),
            inconsistent_topic_status: DdsRwLock::new(InconsistentTopicStatus::default()),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
            announce_sender,
        })
    }
}

impl DdsShared<TopicImpl> {
    pub fn get_inconsistent_topic_status(&self) -> InconsistentTopicStatus {
        self.status_condition
            .write_lock()
            .remove_communication_state(StatusKind::InconsistentTopic);
        self.inconsistent_topic_status.write_lock().read_and_reset()
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
        *self.status_listener.write_lock() = StatusListener::new(a_listener, mask);
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn get_status_changes(&self) -> Vec<StatusKind> {
        self.status_condition.read_lock().get_status_changes()
    }

    pub fn enable(&self) -> DdsResult<()> {
        self.announce_sender
            .send(AnnounceKind::CratedTopic(self.as_discovered_topic_data()))
            .ok();

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

    pub fn process_discovered_topic(
        &self,
        discovered_topic_data: &DiscoveredTopicData,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        if discovered_topic_data.topic_builtin_topic_data.type_name == self.type_name
            && discovered_topic_data.topic_builtin_topic_data.name == self.topic_name
            && !is_discovered_topic_consistent(&self.qos.read_lock(), discovered_topic_data)
        {
            self.inconsistent_topic_status.write_lock().increment();

            self.trigger_on_inconsistent_topic_listener(
                &mut self.status_listener.write_lock(),
                participant_status_listener,
            );

            self.status_condition
                .write_lock()
                .add_communication_state(StatusKind::InconsistentTopic);
        }
    }

    fn trigger_on_inconsistent_topic_listener(
        &self,
        topic_status_listener: &mut StatusListener<dyn AnyTopicListener + Send + Sync>,
        participant_status_listener: &mut StatusListener<
            dyn DomainParticipantListener + Send + Sync,
        >,
    ) {
        let inconsistent_topic_status_kind = &StatusKind::InconsistentTopic;

        if topic_status_listener.is_enabled(inconsistent_topic_status_kind) {
            topic_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .trigger_on_inconsistent_topic(
                    ListenerTopicNode,
                    self.get_inconsistent_topic_status(),
                )
        } else if participant_status_listener.is_enabled(inconsistent_topic_status_kind) {
            participant_status_listener
                .listener_mut()
                .as_mut()
                .expect("Listener should be some")
                .on_inconsistent_topic(&ListenerTopicNode, self.get_inconsistent_topic_status())
        }
    }
}

fn is_discovered_topic_consistent(
    topic_qos: &TopicQos,
    discovered_topic_data: &DiscoveredTopicData,
) -> bool {
    topic_qos.topic_data == discovered_topic_data.topic_builtin_topic_data.topic_data
        && topic_qos.durability == discovered_topic_data.topic_builtin_topic_data.durability
        && topic_qos.deadline == discovered_topic_data.topic_builtin_topic_data.deadline
        && topic_qos.latency_budget
            == discovered_topic_data
                .topic_builtin_topic_data
                .latency_budget
        && topic_qos.liveliness == discovered_topic_data.topic_builtin_topic_data.liveliness
        && topic_qos.reliability == discovered_topic_data.topic_builtin_topic_data.reliability
        && topic_qos.destination_order
            == discovered_topic_data
                .topic_builtin_topic_data
                .destination_order
        && topic_qos.history == discovered_topic_data.topic_builtin_topic_data.history
        && topic_qos.resource_limits
            == discovered_topic_data
                .topic_builtin_topic_data
                .resource_limits
        && topic_qos.transport_priority
            == discovered_topic_data
                .topic_builtin_topic_data
                .transport_priority
        && topic_qos.lifespan == discovered_topic_data.topic_builtin_topic_data.lifespan
        && topic_qos.ownership == discovered_topic_data.topic_builtin_topic_data.ownership
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
        let (announce_sender, _) = std::sync::mpsc::sync_channel(1);
        let topic = TopicImpl::new(
            guid,
            TopicQos::default(),
            "",
            "",
            None,
            &[],
            announce_sender,
        );
        *topic.enabled.write_lock() = true;

        let expected_instance_handle: InstanceHandle = guid.into();
        let instance_handle = topic.get_instance_handle();
        assert_eq!(expected_instance_handle, instance_handle);
    }
}
