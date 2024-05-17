use std::sync::Arc;

use crate::{
    builtin_topics::{BuiltInTopicKey, TopicBuiltinTopicData},
    data_representation_builtin_endpoints::discovered_topic_data::DiscoveredTopicData,
    dds_async::topic_listener::TopicListenerAsync,
    implementation::actor::{
        Actor, ActorAddress, ActorHandler, Mail, MailHandler, DEFAULT_ACTOR_BUFFER_SIZE,
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

use super::{
    status_condition_actor::{self, AddCommunicationState, StatusConditionActor},
    topic_listener_actor::TopicListenerActor,
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

pub struct GetTypeName;
impl Mail for GetTypeName {
    type Result = String;
}
impl MailHandler<GetTypeName> for TopicActor {
    fn handle(
        &mut self,
        _: GetTypeName,
    ) -> impl std::future::Future<Output = <GetTypeName as Mail>::Result> + Send {
        async { self.type_name.clone() }
    }
}

pub struct GetName;
impl Mail for GetName {
    type Result = String;
}
impl MailHandler<GetName> for TopicActor {
    fn handle(
        &mut self,
        _: GetName,
    ) -> impl std::future::Future<Output = <GetName as Mail>::Result> + Send {
        async { self.topic_name.clone() }
    }
}

pub struct GetGuid;
impl Mail for GetGuid {
    type Result = Guid;
}
impl MailHandler<GetGuid> for TopicActor {
    fn handle(
        &mut self,
        _: GetGuid,
    ) -> impl std::future::Future<Output = <GetGuid as Mail>::Result> + Send {
        async { self.guid }
    }
}

pub struct SetQos {
    pub qos: TopicQos,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for TopicActor {
    fn handle(
        &mut self,
        message: SetQos,
    ) -> impl std::future::Future<Output = <SetQos as Mail>::Result> + Send {
        async move {
            message.qos.is_consistent()?;

            if self.enabled {
                self.qos.check_immutability(&message.qos)?
            }

            self.qos = message.qos;

            Ok(())
        }
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = TopicQos;
}
impl MailHandler<GetQos> for TopicActor {
    fn handle(
        &mut self,
        _: GetQos,
    ) -> impl std::future::Future<Output = <GetQos as Mail>::Result> + Send {
        async move { self.qos.clone() }
    }
}

pub struct Enable;
impl Mail for Enable {
    type Result = ();
}
impl MailHandler<Enable> for TopicActor {
    fn handle(
        &mut self,
        _: Enable,
    ) -> impl std::future::Future<Output = <Enable as Mail>::Result> + Send {
        async move {
            self.enabled = true;
        }
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for TopicActor {
    fn handle(
        &mut self,
        _: IsEnabled,
    ) -> impl std::future::Future<Output = <IsEnabled as Mail>::Result> + Send {
        async move { self.enabled }
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for TopicActor {
    fn handle(
        &mut self,
        _: GetInstanceHandle,
    ) -> impl std::future::Future<Output = <GetInstanceHandle as Mail>::Result> + Send {
        async move { InstanceHandle::new(self.guid.into()) }
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for TopicActor {
    fn handle(
        &mut self,
        _: GetStatuscondition,
    ) -> impl std::future::Future<Output = <GetStatuscondition as Mail>::Result> + Send {
        async move { self.status_condition.address() }
    }
}

pub struct AsDiscoveredTopicData;
impl Mail for AsDiscoveredTopicData {
    type Result = DiscoveredTopicData;
}
impl MailHandler<AsDiscoveredTopicData> for TopicActor {
    fn handle(
        &mut self,
        _: AsDiscoveredTopicData,
    ) -> impl std::future::Future<Output = <AsDiscoveredTopicData as Mail>::Result> + Send {
        async move {
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
}

pub struct GetInconsistentTopicStatus;
impl Mail for GetInconsistentTopicStatus {
    type Result = InconsistentTopicStatus;
}
impl MailHandler<GetInconsistentTopicStatus> for TopicActor {
    fn handle(
        &mut self,
        _: GetInconsistentTopicStatus,
    ) -> impl std::future::Future<Output = <GetInconsistentTopicStatus as Mail>::Result> + Send
    {
        async move {
            let status = self.inconsistent_topic_status.read_and_reset();
            self.status_condition
                .send_actor_mail(status_condition_actor::RemoveCommunicationState {
                    state: StatusKind::InconsistentTopic,
                })
                .await
                .receive_reply()
                .await;
            status
        }
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
    ) -> impl std::future::Future<Output = <ProcessDiscoveredTopic as Mail>::Result> + Send {
        async move {
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
                    })
                    .await
                    .receive_reply()
                    .await;
            }
        }
    }
}

pub struct GetTypeSupport;
impl Mail for GetTypeSupport {
    type Result = Arc<dyn DynamicTypeInterface + Send + Sync>;
}
impl MailHandler<GetTypeSupport> for TopicActor {
    fn handle(
        &mut self,
        _: GetTypeSupport,
    ) -> impl std::future::Future<Output = <GetTypeSupport as Mail>::Result> + Send {
        async move { self.type_support.clone() }
    }
}

impl ActorHandler for TopicActor {
    type Message = ();

    fn handle_message(&mut self, _: Self::Message) -> impl std::future::Future<Output = ()> + Send {
        async {}
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
