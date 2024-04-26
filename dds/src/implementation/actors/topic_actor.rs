use std::sync::Arc;

use dust_dds_derive::actor_interface;

use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
    dds_async::topic_listener::TopicListenerAsync,
    implementation::actor::{Actor, ActorAddress, DEFAULT_ACTOR_BUFFER_SIZE},
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::TopicQos,
        status::{InconsistentTopicStatus, StatusKind},
    },
    rtps::types::Guid,
    topic_definition::type_support::DynamicTypeInterface,
};

use super::{
    status_condition_actor::StatusConditionActor, topic_listener_actor::TopicListenerActor,
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

pub struct TopicActor {
    guid: Guid,
    qos: TopicQos,
    type_name: String,
    topic_name: String,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: Actor<StatusConditionActor>,
    _listener: Actor<TopicListenerActor>,
    type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
}

impl TopicActor {
    pub fn new(
        guid: Guid,
        qos: TopicQos,
        type_name: String,
        topic_name: &str,
        listener: Option<Box<dyn TopicListenerAsync + Send>>,
        type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
        handle: &tokio::runtime::Handle,
    ) -> Self {
        let status_condition = Actor::spawn(
            StatusConditionActor::default(),
            handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        let listener = Actor::spawn(
            TopicListenerActor::new(listener),
            handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        Self {
            guid,
            qos,
            type_name,
            topic_name: topic_name.to_string(),
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition,
            _listener: listener,
            type_support,
        }
    }
}

#[actor_interface]
impl TopicActor {
    fn get_type_name(&self) -> String {
        self.type_name.clone()
    }

    fn get_name(&self) -> String {
        self.topic_name.clone()
    }

    fn guid(&self) -> Guid {
        self.guid
    }

    fn set_qos(&mut self, qos: TopicQos) -> DdsResult<()> {
        if self.enabled {
            self.qos.check_immutability(&qos)?
        }

        self.qos = qos;

        Ok(())
    }

    fn get_qos(&self) -> TopicQos {
        self.qos.clone()
    }

    #[allow(clippy::unused_unit)]
    fn enable(&mut self) -> () {
        self.enabled = true;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.guid.into())
    }

    pub fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    fn as_discovered_topic_data(&self) -> DiscoveredTopicData {
        DiscoveredTopicData::new(TopicBuiltinTopicData::new(
            BuiltInTopicKey {
                value: self.guid.into(),
            },
            self.topic_name.to_string(),
            self.type_name.to_string(),
            self.qos.clone(),
        ))
    }

    async fn get_inconsistent_topic_status(&mut self) -> DdsResult<InconsistentTopicStatus> {
        let status = self.inconsistent_topic_status.read_and_reset();
        self.status_condition
            .remove_communication_state(StatusKind::InconsistentTopic)
            .await;
        Ok(status)
    }

    #[allow(clippy::unused_unit)]
    async fn process_discovered_topic(&mut self, discovered_topic_data: DiscoveredTopicData) -> () {
        if discovered_topic_data
            .topic_builtin_topic_data()
            .get_type_name()
            == self.get_type_name()
            && discovered_topic_data.topic_builtin_topic_data().name() == self.get_name()
            && !is_discovered_topic_consistent(&self.qos, &discovered_topic_data)
        {
            self.inconsistent_topic_status.increment();
            self.status_condition
                .add_communication_state(StatusKind::InconsistentTopic)
                .await;
        }
    }

    pub fn get_type_support(
        &self,
        type_name: String,
    ) -> Arc<dyn DynamicTypeInterface + Send + Sync> {
        self.type_support_list.get(&type_name).cloned()
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
