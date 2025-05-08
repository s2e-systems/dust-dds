use alloc::sync::Arc;

use super::{
    actor::Actor, listeners::domain_participant_listener::ListenerMail, runtime::DdsRuntime,
    status_condition::StatusCondition, status_condition_actor::StatusConditionActor,
};
use crate::{
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::TopicQos,
        status::{InconsistentTopicStatus, StatusKind},
    },
    xtypes::dynamic_type::DynamicType,
};
use alloc::{string::String, vec::Vec};

pub struct TopicEntity<R: DdsRuntime> {
    qos: TopicQos,
    type_name: String,
    topic_name: String,
    instance_handle: InstanceHandle,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: Actor<R, StatusConditionActor<R>>,
    _listener_sender: R::ChannelSender<ListenerMail<R>>,
    _status_kind: Vec<StatusKind>,
    type_support: Arc<dyn DynamicType + Send + Sync>,
}

impl<R: DdsRuntime> TopicEntity<R> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        qos: TopicQos,
        type_name: String,
        topic_name: String,
        instance_handle: InstanceHandle,
        status_condition: Actor<R, StatusConditionActor<R>>,
        listener_sender: R::ChannelSender<ListenerMail<R>>,
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
            _listener_sender: listener_sender,
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

    pub fn status_condition(&self) -> &Actor<R, StatusConditionActor<R>> {
        &self.status_condition
    }

    pub fn qos(&self) -> &TopicQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: TopicQos) -> DdsResult<()> {
        qos.is_consistent()?;

        if self.enabled
            && (self.qos.durability != qos.durability
                || self.qos.liveliness != qos.liveliness
                || self.qos.reliability != qos.reliability
                || self.qos.destination_order != qos.destination_order
                || self.qos.history != qos.history
                || self.qos.resource_limits != qos.resource_limits
                || self.qos.ownership != qos.ownership)
        {
            return Err(DdsError::ImmutablePolicy);
        }

        self.qos = qos;
        Ok(())
    }

    pub async fn get_inconsistent_topic_status(&mut self) -> InconsistentTopicStatus {
        let status = self.inconsistent_topic_status.clone();
        self.inconsistent_topic_status.total_count_change = 0;
        self.status_condition
            .remove_state(StatusKind::InconsistentTopic)
            .await;
        status
    }

    pub async fn increment_inconsistent_topic_status(&mut self) {
        self.inconsistent_topic_status.total_count += 1;
        self.inconsistent_topic_status.total_count_change += 1;
        self.status_condition
            .add_state(StatusKind::InconsistentTopic)
            .await;
    }
}
