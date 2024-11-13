use super::status_condition_actor::{self, StatusConditionActor};
use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    dds_async::topic_listener::TopicListenerAsync,
    implementation::{
        actor::{Actor, ActorAddress},
        data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
        runtime::{
            executor::{block_on, ExecutorHandle},
            mpsc::{mpsc_channel, MpscSender},
        },
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::TopicQos,
        status::{InconsistentTopicStatus, StatusKind},
    },
    xtypes::dynamic_type::DynamicType,
};
use std::{sync::Arc, thread::JoinHandle};

pub enum TopicListenerOperation {}

pub struct TopicListenerMessage {
    pub _listener_operation: TopicListenerOperation,
}

pub struct TopicListenerThread {
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
    pub qos: TopicQos,
    pub type_name: String,
    pub topic_name: String,
    pub instance_handle: InstanceHandle,
    pub enabled: bool,
    pub inconsistent_topic_status: InconsistentTopicStatus,
    pub status_condition: Actor<StatusConditionActor>,
    pub topic_listener_thread: Option<TopicListenerThread>,
    pub status_kind: Vec<StatusKind>,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
}
