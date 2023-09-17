use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        rtps::types::Guid,
        utils::{
            actor::{actor_mailbox_interface, Mail, MailHandler},
            shared_object::{DdsRwLock, DdsShared},
        },
    },
    infrastructure::{
        instance::InstanceHandle,
        qos::TopicQos,
        status::{InconsistentTopicStatus, StatusKind},
    },
};

use super::status_condition_impl::StatusConditionImpl;

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

pub struct DdsTopic {
    guid: Guid,
    qos: TopicQos,
    type_name: String,
    topic_name: String,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: DdsShared<DdsRwLock<StatusConditionImpl>>,
}

impl DdsTopic {
    pub fn new(guid: Guid, qos: TopicQos, type_name: String, topic_name: &str) -> Self {
        Self {
            guid,
            qos,
            type_name,
            topic_name: topic_name.to_string(),
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition: DdsShared::new(DdsRwLock::new(StatusConditionImpl::default())),
        }
    }
}

actor_mailbox_interface! {
impl DdsTopic {
    pub fn get_inconsistent_topic_status(&mut self) -> InconsistentTopicStatus {
        let status = self.inconsistent_topic_status.read_and_reset();
        self.status_condition.write_lock().remove_communication_state(StatusKind::InconsistentTopic);
        status
    }

    pub fn get_type_name(&self) -> String {
        self.type_name.clone()
    }

    pub fn get_name(&self) -> String {
        self.topic_name.clone()
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn set_qos(&mut self, qos: TopicQos) {
        self.qos = qos;
    }

    pub fn get_qos(&self) -> TopicQos {
        self.qos.clone()
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
    }

    pub fn get_statuscondition(&self) -> DdsShared<DdsRwLock<StatusConditionImpl>> {
        self.status_condition.clone()
    }

    pub fn as_discovered_topic_data(&self) -> DiscoveredTopicData {
        let qos = &self.qos;
        DiscoveredTopicData::new(TopicBuiltinTopicData::new(
            BuiltInTopicKey {
                value: self.guid.into(),
            },
            self.topic_name.to_string(),
            self.type_name.to_string(),
            qos.durability.clone(),
            qos.deadline.clone(),
            qos.latency_budget.clone(),
            qos.liveliness.clone(),
            qos.reliability.clone(),
            qos.transport_priority.clone(),
            qos.lifespan.clone(),
            qos.destination_order.clone(),
            qos.history.clone(),
            qos.resource_limits.clone(),
            qos.ownership.clone(),
            qos.topic_data.clone(),
        ))
    }
}
}

pub struct ProcessDiscoveredTopic {
    discovered_topic_data: DiscoveredTopicData,
}

impl ProcessDiscoveredTopic {
    pub fn new(discovered_topic_data: DiscoveredTopicData) -> Self {
        Self {
            discovered_topic_data,
        }
    }
}

impl Mail for ProcessDiscoveredTopic {
    type Result = ();
}

#[async_trait::async_trait]
impl MailHandler<ProcessDiscoveredTopic> for DdsTopic {
    async fn handle(
        &mut self,
        mail: ProcessDiscoveredTopic,
    ) -> <ProcessDiscoveredTopic as Mail>::Result {
        if mail
            .discovered_topic_data
            .topic_builtin_topic_data()
            .get_type_name()
            == self.get_type_name()
            && mail.discovered_topic_data.topic_builtin_topic_data().name() == self.get_name()
            && !is_discovered_topic_consistent(&self.qos, &mail.discovered_topic_data)
        {
            self.inconsistent_topic_status.increment();
            self.status_condition
                .write_lock()
                .add_communication_state(StatusKind::InconsistentTopic);
        }
    }
}

fn is_discovered_topic_consistent(
    topic_qos: &TopicQos,
    discovered_topic_data: &DiscoveredTopicData,
) -> bool {
    &topic_qos.topic_data
        == discovered_topic_data
            .topic_builtin_topic_data()
            .topic_data()
        && &topic_qos.durability
            == discovered_topic_data
                .topic_builtin_topic_data()
                .durability()
        && &topic_qos.deadline == discovered_topic_data.topic_builtin_topic_data().deadline()
        && &topic_qos.latency_budget
            == discovered_topic_data
                .topic_builtin_topic_data()
                .latency_budget()
        && &topic_qos.liveliness
            == discovered_topic_data
                .topic_builtin_topic_data()
                .liveliness()
        && &topic_qos.reliability
            == discovered_topic_data
                .topic_builtin_topic_data()
                .reliability()
        && &topic_qos.destination_order
            == discovered_topic_data
                .topic_builtin_topic_data()
                .destination_order()
        && &topic_qos.history == discovered_topic_data.topic_builtin_topic_data().history()
        && &topic_qos.resource_limits
            == discovered_topic_data
                .topic_builtin_topic_data()
                .resource_limits()
        && &topic_qos.transport_priority
            == discovered_topic_data
                .topic_builtin_topic_data()
                .transport_priority()
        && &topic_qos.lifespan == discovered_topic_data.topic_builtin_topic_data().lifespan()
        && &topic_qos.ownership == discovered_topic_data.topic_builtin_topic_data().ownership()
}

#[cfg(test)]
mod tests {

    use crate::implementation::rtps::types::{EntityId, BUILT_IN_PARTICIPANT};

    use super::*;

    #[test]
    fn get_instance_handle() {
        let guid = Guid::new([2; 12], EntityId::new([3; 3], BUILT_IN_PARTICIPANT));
        let mut topic = DdsTopic::new(guid, TopicQos::default(), "".to_string(), "");
        topic.enabled = true;

        let expected_instance_handle: InstanceHandle = guid.into();
        let instance_handle = topic.get_instance_handle();
        assert_eq!(expected_instance_handle, instance_handle);
    }
}
