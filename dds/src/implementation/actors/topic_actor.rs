use std::{sync::Arc, thread::JoinHandle};

use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
    dds_async::topic_listener::TopicListenerAsync,
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
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
    rtps::types::Guid,
    topic_definition::type_support::DynamicTypeInterface,
};

use super::status_condition_actor::{self, AddCommunicationState, StatusConditionActor};

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
        let thread = std::thread::spawn(move || {
            block_on(async {
                // TODO
                // while let Some(m) = receiver.recv().await {
                //     match m.listener_operation {}
                // }
            });
        });
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
    enabled: bool,
    inconsistent_topic_status: InconsistentTopicStatus,
    status_condition: Actor<StatusConditionActor>,
    _topic_listener_thread: Option<TopicListenerThread>,
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
        handle: &ExecutorHandle,
    ) -> (Self, ActorAddress<StatusConditionActor>) {
        let status_condition = Actor::spawn(StatusConditionActor::default(), handle);
        let status_condition_address = status_condition.address();
        let topic_listener_thread = listener.map(TopicListenerThread::new);
        (
            Self {
                guid,
                qos,
                type_name,
                topic_name: topic_name.to_string(),
                enabled: false,
                inconsistent_topic_status: InconsistentTopicStatus::default(),
                status_condition,
                _topic_listener_thread: topic_listener_thread,
                type_support,
            },
            status_condition_address,
        )
    }
}

pub struct GetTypeName;
impl Mail for GetTypeName {
    type Result = String;
}
impl MailHandler<GetTypeName> for TopicActor {
    fn handle(&mut self, _: GetTypeName) -> <GetTypeName as Mail>::Result {
        self.type_name.clone()
    }
}

pub struct GetName;
impl Mail for GetName {
    type Result = String;
}
impl MailHandler<GetName> for TopicActor {
    fn handle(&mut self, _: GetName) -> <GetName as Mail>::Result {
        self.topic_name.clone()
    }
}

pub struct GetGuid;
impl Mail for GetGuid {
    type Result = Guid;
}
impl MailHandler<GetGuid> for TopicActor {
    fn handle(&mut self, _: GetGuid) -> <GetGuid as Mail>::Result {
        self.guid
    }
}

pub struct SetQos {
    pub qos: TopicQos,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for TopicActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        message.qos.is_consistent()?;

        if self.enabled {
            self.qos.check_immutability(&message.qos)?
        }

        self.qos = message.qos;

        Ok(())
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = TopicQos;
}
impl MailHandler<GetQos> for TopicActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct Enable;
impl Mail for Enable {
    type Result = ();
}
impl MailHandler<Enable> for TopicActor {
    fn handle(&mut self, _: Enable) -> <Enable as Mail>::Result {
        self.enabled = true;
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for TopicActor {
    fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.enabled
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for TopicActor {
    fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        InstanceHandle::new(self.guid.into())
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for TopicActor {
    fn handle(&mut self, _: GetStatuscondition) -> <GetStatuscondition as Mail>::Result {
        self.status_condition.address()
    }
}

pub struct AsDiscoveredTopicData;
impl Mail for AsDiscoveredTopicData {
    type Result = DiscoveredTopicData;
}
impl MailHandler<AsDiscoveredTopicData> for TopicActor {
    fn handle(&mut self, _: AsDiscoveredTopicData) -> <AsDiscoveredTopicData as Mail>::Result {
        DiscoveredTopicData::new(TopicBuiltinTopicData::new(
            BuiltInTopicKey {
                value: self.guid.into(),
            },
            self.topic_name.to_string(),
            self.type_name.to_string(),
            self.qos.clone(),
        ))
    }
}

pub struct GetInconsistentTopicStatus;
impl Mail for GetInconsistentTopicStatus {
    type Result = InconsistentTopicStatus;
}
impl MailHandler<GetInconsistentTopicStatus> for TopicActor {
    fn handle(
        &mut self,
        _: GetInconsistentTopicStatus,
    ) -> <GetInconsistentTopicStatus as Mail>::Result {
        let status = self.inconsistent_topic_status.read_and_reset();
        self.status_condition
            .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                state: StatusKind::InconsistentTopic,
            });
        status
    }
}

pub struct ProcessDiscoveredTopic {
    pub discovered_topic_data: DiscoveredTopicData,
}
impl Mail for ProcessDiscoveredTopic {
    type Result = ();
}
impl MailHandler<ProcessDiscoveredTopic> for TopicActor {
    fn handle(
        &mut self,
        message: ProcessDiscoveredTopic,
    ) -> <ProcessDiscoveredTopic as Mail>::Result {
        if message
            .discovered_topic_data
            .topic_builtin_topic_data()
            .get_type_name()
            == self.type_name
            && message
                .discovered_topic_data
                .topic_builtin_topic_data()
                .name()
                == self.topic_name
            && !is_discovered_topic_consistent(&self.qos, &message.discovered_topic_data)
        {
            self.inconsistent_topic_status.increment();
            self.status_condition
                .send_actor_mail(AddCommunicationState {
                    state: StatusKind::InconsistentTopic,
                });
        }
    }
}

pub struct GetTypeSupport;
impl Mail for GetTypeSupport {
    type Result = Arc<dyn DynamicTypeInterface + Send + Sync>;
}
impl MailHandler<GetTypeSupport> for TopicActor {
    fn handle(&mut self, _: GetTypeSupport) -> <GetTypeSupport as Mail>::Result {
        self.type_support.clone()
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
