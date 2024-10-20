use super::{handle::TopicHandle, status_condition_actor::StatusConditionActor};
use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    dds_async::topic_listener::TopicListenerAsync,
    implementation::{
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        runtime::{
            executor::block_on,
            mpsc::{mpsc_channel, MpscSender},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::TopicQos,
        status::{InconsistentTopicStatus, StatusKind},
    },
    rtps::types::Guid,
    xtypes::dynamic_type::DynamicType,
};
use std::{sync::Arc, thread::JoinHandle};

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

pub enum TopicListenerOperation {}

pub struct TopicListenerMessage {
    pub _listener_operation: TopicListenerOperation,
}

struct TopicListenerThread {
    _thread: JoinHandle<()>,
    _sender: MpscSender<TopicListenerMessage>,
}

impl TopicListenerThread {
    fn new(_listener: Box<dyn TopicListenerAsync + Send>) -> Self {
        let (sender, _receiver) = mpsc_channel::<TopicListenerMessage>();
        let thread = std::thread::Builder::new()
            .name("Topic listener".to_string())
            .spawn(move || {
                block_on(async {
                    // TODO
                    // while let Some(m) = receiver.recv().await {
                    //     match m.listener_operation {}
                    // }
                });
            })
            .expect("failed to spawn thread");
        Self {
            _thread: thread,
            _sender: sender,
        }
    }

    fn _sender(&self) -> &MpscSender<TopicListenerMessage> {
        &self._sender
    }

    fn _join(self) -> DdsResult<()> {
        self._sender.close();
        self._thread.join()?;
        Ok(())
    }
}

pub struct TopicActor {
    guid: Guid,
    qos: TopicQos,
    type_name: String,
    topic_name: String,
    topic_handle: TopicHandle,
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: StatusConditionActor,
    _topic_listener_thread: Option<TopicListenerThread>,
    status_kind: Vec<StatusKind>,
    type_support: Arc<dyn DynamicType + Send + Sync>,
}

impl TopicActor {
    pub fn new(
        guid: Guid,
        qos: TopicQos,
        type_name: String,
        topic_name: &str,
        listener: Option<Box<dyn TopicListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        type_support: Arc<dyn DynamicType + Send + Sync>,
        topic_handle: TopicHandle,
    ) -> Self {
        let topic_listener_thread = listener.map(TopicListenerThread::new);

        Self {
            guid,
            qos,
            type_name,
            topic_name: topic_name.to_string(),
            topic_handle,
            enabled: false,
            inconsistent_topic_status: InconsistentTopicStatus::default(),
            status_condition: StatusConditionActor::default(),
            _topic_listener_thread: topic_listener_thread,
            status_kind,
            type_support,
        }
    }

    pub fn get_inconsistent_topic_status(&mut self) -> DdsResult<InconsistentTopicStatus> {
        let status = self.inconsistent_topic_status.read_and_reset();
        self.status_condition
            .remove_communication_state(StatusKind::InconsistentTopic);

        Ok(status)
    }

    pub fn get_qos(&self) -> &TopicQos {
        &self.qos
    }

    pub fn set_qos(&mut self, topic_qos: TopicQos) -> DdsResult<()> {
        topic_qos.is_consistent()?;

        if self.enabled {
            self.qos.check_immutability(&topic_qos)?
        }

        self.qos = topic_qos;

        Ok(())
    }

    pub fn enable(&mut self) -> DdsResult<()> {
        self.enabled = true;
        Ok(())
    }

    pub fn get_type_support(&self) -> &Arc<dyn DynamicType + Send + Sync> {
        &self.type_support
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn get_name(&self) -> &str {
        &self.topic_name
    }

    pub fn get_handle(&self) -> TopicHandle {
        self.topic_handle
    }

    pub fn as_discovered_topic_data(&self) -> DiscoveredTopicData {
        let topic_qos = self.qos.clone();
        DiscoveredTopicData {
            topic_builtin_topic_data: TopicBuiltinTopicData {
                key: BuiltInTopicKey {
                    value: self.guid.into(),
                },
                name: self.topic_name.to_string(),
                type_name: self.type_name.to_string(),
                durability: topic_qos.durability,
                deadline: topic_qos.deadline,
                latency_budget: topic_qos.latency_budget,
                liveliness: topic_qos.liveliness,
                reliability: topic_qos.reliability,
                transport_priority: topic_qos.transport_priority,
                lifespan: topic_qos.lifespan,
                destination_order: topic_qos.destination_order,
                history: topic_qos.history,
                resource_limits: topic_qos.resource_limits,
                ownership: topic_qos.ownership,
                topic_data: topic_qos.topic_data,
                representation: topic_qos.representation,
            },
        }
    }

    fn process_discovered_topic(&mut self, discovered_topic_data: DiscoveredTopicData) {
        if discovered_topic_data
            .topic_builtin_topic_data
            .get_type_name()
            == self.type_name
            && discovered_topic_data.topic_builtin_topic_data.name() == self.topic_name
            && !is_discovered_topic_consistent(&self.qos, &discovered_topic_data)
        {
            self.inconsistent_topic_status.increment();
            self.status_condition
                .add_communication_state(StatusKind::InconsistentTopic);
        }
    }
}

fn is_discovered_topic_consistent(
    topic_qos: &TopicQos,
    discovered_topic_data: &DiscoveredTopicData,
) -> bool {
    &topic_qos.topic_data == discovered_topic_data.topic_builtin_topic_data.topic_data()
        && &topic_qos.durability == discovered_topic_data.topic_builtin_topic_data.durability()
        && &topic_qos.deadline == discovered_topic_data.topic_builtin_topic_data.deadline()
        && &topic_qos.latency_budget
            == discovered_topic_data
                .topic_builtin_topic_data
                .latency_budget()
        && &topic_qos.liveliness == discovered_topic_data.topic_builtin_topic_data.liveliness()
        && &topic_qos.reliability == discovered_topic_data.topic_builtin_topic_data.reliability()
        && &topic_qos.destination_order
            == discovered_topic_data
                .topic_builtin_topic_data
                .destination_order()
        && &topic_qos.history == discovered_topic_data.topic_builtin_topic_data.history()
        && &topic_qos.resource_limits
            == discovered_topic_data
                .topic_builtin_topic_data
                .resource_limits()
        && &topic_qos.transport_priority
            == discovered_topic_data
                .topic_builtin_topic_data
                .transport_priority()
        && &topic_qos.lifespan == discovered_topic_data.topic_builtin_topic_data.lifespan()
        && &topic_qos.ownership == discovered_topic_data.topic_builtin_topic_data.ownership()
}
