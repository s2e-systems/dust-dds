use std::sync::Arc;

use crate::{
    implementation::{
        listeners::topic_listener::TopicListenerActor,
        status_condition::status_condition_actor::{self, StatusConditionActor},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::TopicQos,
        status::{InconsistentTopicStatus, StatusKind},
    },
    runtime::actor::Actor,
    xtypes::dynamic_type::DynamicType,
};

pub struct TopicEntity {
    qos: TopicQos,
    type_name: String,
    topic_name: String,
    instance_handle: InstanceHandle,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: Actor<StatusConditionActor>,
    _listener: Option<Actor<TopicListenerActor>>,
    _status_kind: Vec<StatusKind>,
    type_support: Arc<dyn DynamicType + Send + Sync>,
}

impl TopicEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        qos: TopicQos,
        type_name: String,
        topic_name: String,
        instance_handle: InstanceHandle,
        status_condition: Actor<StatusConditionActor>,
        listener: Option<Actor<TopicListenerActor>>,
        status_kind: Vec<StatusKind>,
        type_support: Arc<dyn DynamicType + Send + Sync>,
    ) -> Self {
        Self {
            qos,
            type_name,
            topic_name,
            instance_handle,
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition,
            _listener: listener,
            _status_kind: status_kind,
            type_support,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn enabled(&mut self) -> bool {
        self.enabled
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn instance_handle(&self) -> InstanceHandle {
        self.instance_handle
    }

    pub fn type_support(&self) -> &Arc<dyn DynamicType + Send + Sync> {
        &self.type_support
    }

    pub fn status_condition(&self) -> &Actor<StatusConditionActor> {
        &self.status_condition
    }

    pub fn qos(&self) -> &TopicQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: TopicQos) -> DdsResult<()> {
        qos.is_consistent()?;

        if self.enabled {
            if self.qos.durability != qos.durability
                || self.qos.liveliness != qos.liveliness
                || self.qos.reliability != qos.reliability
                || self.qos.destination_order != qos.destination_order
                || self.qos.history != qos.history
                || self.qos.resource_limits != qos.resource_limits
                || self.qos.ownership != qos.ownership
            {
                return Err(DdsError::ImmutablePolicy);
            }
        }

        self.qos = qos;
        Ok(())
    }

    pub fn get_inconsistent_topic_status(&mut self) -> InconsistentTopicStatus {
        let status = self.inconsistent_topic_status.clone();
        self.inconsistent_topic_status.total_count_change = 0;
        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::InconsistentTopic,
            });
        status
    }

    pub fn increment_inconsistent_topic_status(&mut self) {
        self.inconsistent_topic_status.total_count += 1;
        self.inconsistent_topic_status.total_count_change += 1;
        self.status_condition
            .send_actor_mail(status_condition_actor::AddCommunicationState {
                state: StatusKind::InconsistentTopic,
            });
    }
}
