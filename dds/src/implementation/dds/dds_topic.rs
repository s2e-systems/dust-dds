use tokio::sync::mpsc::Sender;

use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        rtps::types::Guid,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{QosKind, TopicQos},
        status::InconsistentTopicStatus,
    },
};

use super::{dds_domain_participant::AnnounceKind, status_listener::ListenerTriggerKind};

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
    type_name: &'static str,
    topic_name: String,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    announce_sender: Sender<AnnounceKind>,
}

impl DdsTopic {
    pub fn new(
        guid: Guid,
        qos: TopicQos,
        type_name: &'static str,
        topic_name: &str,
        announce_sender: Sender<AnnounceKind>,
    ) -> Self {
        Self {
            guid,
            qos,
            type_name,
            topic_name: topic_name.to_string(),
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            announce_sender,
        }
    }
}

impl DdsTopic {
    pub fn get_inconsistent_topic_status(&mut self) -> InconsistentTopicStatus {
        self.inconsistent_topic_status.read_and_reset()
    }

    pub fn get_type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn get_name(&self) -> String {
        self.topic_name.clone()
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }

    pub fn set_qos(&mut self, qos: QosKind<TopicQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        qos.is_consistent()?;
        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        self.qos = qos;

        Ok(())
    }

    pub fn get_qos(&self) -> TopicQos {
        self.qos.clone()
    }

    pub fn enable(&mut self) -> DdsResult<()> {
        self.announce_sender
            .try_send(AnnounceKind::CratedTopic(self.as_discovered_topic_data()))
            .ok();

        self.enabled = true;
        Ok(())
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.guid.into()
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

    pub fn process_discovered_topic(
        &mut self,
        discovered_topic_data: DiscoveredTopicData,
        parent_participant_guid: Guid,
        listener_sender: tokio::sync::mpsc::Sender<ListenerTriggerKind>,
    ) {
        todo!()
        // if discovered_topic_data
        //     .topic_builtin_topic_data()
        //     .get_type_name()
        //     == self.get_type_name()
        //     && discovered_topic_data.topic_builtin_topic_data().name() == self.get_name()
        //     && !is_discovered_topic_consistent(&self.qos, discovered_topic_data)
        // {
        //     self.inconsistent_topic_status.increment();
        //     listener_sender
        //         .try_send(ListenerTriggerKind::InconsistentTopic(TopicNode::new(
        //             self.guid(),
        //             parent_participant_guid,
        //         )))
        //         .ok();
        // }
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
        let (announce_sender, _) = tokio::sync::mpsc::channel(1);
        let mut topic = DdsTopic::new(guid, TopicQos::default(), "", "", announce_sender);
        topic.enabled = true;

        let expected_instance_handle: InstanceHandle = guid.into();
        let instance_handle = topic.get_instance_handle();
        assert_eq!(expected_instance_handle, instance_handle);
    }
}
