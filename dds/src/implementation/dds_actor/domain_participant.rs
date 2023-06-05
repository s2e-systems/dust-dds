use crate::{
    domain::domain_participant_factory::DomainId,
    implementation::{
        dds::{
            any_topic_listener::AnyTopicListener,
            dds_domain_participant::{AnnounceKind, DdsDomainParticipant},
            dds_publisher::DdsPublisher,
            dds_subscriber::DdsSubscriber,
            dds_topic::DdsTopic,
        },
        rtps::{messages::overall_structure::RtpsMessageRead, types::Locator},
        utils::actor::{ActorAddress, Handler, Message},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DomainParticipantQos, PublisherQos, QosKind, SubscriberQos, TopicQos},
        status::StatusKind,
    },
    publication::publisher_listener::PublisherListener,
    subscription::subscriber_listener::SubscriberListener,
};

pub struct Enable;

impl Message for Enable {
    type Result = ();
}

impl Handler<Enable> for DdsDomainParticipant {
    fn handle(&mut self, _message: Enable) -> <Enable as Message>::Result {
        self.enable()
    }
}

pub struct GetQos;

impl Message for GetQos {
    type Result = DomainParticipantQos;
}

impl Handler<GetQos> for DdsDomainParticipant {
    fn handle(&mut self, _message: GetQos) -> <GetQos as Message>::Result {
        self.get_qos()
    }
}

pub struct GetDomainId;

impl Message for GetDomainId {
    type Result = DomainId;
}

impl Handler<GetDomainId> for DdsDomainParticipant {
    fn handle(&mut self, _message: GetDomainId) -> <GetDomainId as Message>::Result {
        self.domain_id()
    }
}

pub struct GetInstanceHandle;

impl Message for GetInstanceHandle {
    type Result = InstanceHandle;
}

impl Handler<GetInstanceHandle> for DdsDomainParticipant {
    fn handle(&mut self, _message: GetInstanceHandle) -> <GetInstanceHandle as Message>::Result {
        self.guid().into()
    }
}

pub struct IsEmpty;

impl Message for IsEmpty {
    type Result = bool;
}

impl Handler<IsEmpty> for DdsDomainParticipant {
    fn handle(&mut self, _message: IsEmpty) -> <IsEmpty as Message>::Result {
        self.is_empty()
    }
}

pub struct ReceiveBuiltinMessage {
    locator: Locator,
    message: RtpsMessageRead,
}

impl ReceiveBuiltinMessage {
    pub fn new(locator: Locator, message: RtpsMessageRead) -> Self {
        Self { locator, message }
    }
}

impl Message for ReceiveBuiltinMessage {
    type Result = ();
}

impl Handler<ReceiveBuiltinMessage> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _message: ReceiveBuiltinMessage,
    ) -> <ReceiveBuiltinMessage as Message>::Result {
        // self.receive_builtin_data(locator, message, listener_sender)
        //     .ok();

        // discover_matched_participants(domain_participant, sedp_condvar).ok();
        // domain_participant
        //     .discover_matched_readers(listener_sender)
        //     .ok();
        // discover_matched_writers(domain_participant, listener_sender).ok();
        // domain_participant
        //     .discover_matched_topics(listener_sender)
        //     .ok();
    }
}

pub struct ReceiveUserDefinedMessage {
    locator: Locator,
    message: RtpsMessageRead,
}

impl ReceiveUserDefinedMessage {
    pub fn new(locator: Locator, message: RtpsMessageRead) -> Self {
        Self { locator, message }
    }
}

impl Message for ReceiveUserDefinedMessage {
    type Result = ();
}

impl Handler<ReceiveUserDefinedMessage> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _message: ReceiveUserDefinedMessage,
    ) -> <ReceiveUserDefinedMessage as Message>::Result {
        // todo!();
    }
}

pub struct AnnounceEntity {
    announce_kind: AnnounceKind,
}

impl AnnounceEntity {
    pub fn new(announce_kind: AnnounceKind) -> Self {
        Self { announce_kind }
    }
}

impl Message for AnnounceEntity {
    type Result = ();
}

impl Handler<AnnounceEntity> for DdsDomainParticipant {
    fn handle(&mut self, _message: AnnounceEntity) -> <AnnounceEntity as Message>::Result {
        // todo!();
    }
}

pub struct AnnounceParticipant;

impl Message for AnnounceParticipant {
    type Result = ();
}

impl Handler<AnnounceParticipant> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _message: AnnounceParticipant,
    ) -> <AnnounceParticipant as Message>::Result {
        // todo!();
    }
}

pub struct CreateTopic {
    topic_name: String,
    type_name: &'static str,
    qos: QosKind<TopicQos>,
    a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
    mask: Vec<StatusKind>,
}

impl CreateTopic {
    pub fn new(
        topic_name: String,
        type_name: &'static str,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            topic_name,
            type_name,
            qos,
            a_listener,
            mask,
        }
    }
}

impl Message for CreateTopic {
    type Result = DdsResult<ActorAddress<DdsTopic>>;
}

impl Handler<CreateTopic> for DdsDomainParticipant {
    fn handle(&mut self, message: CreateTopic) -> <CreateTopic as Message>::Result {
        self.create_topic(&message.topic_name, message.type_name, message.qos)
    }
}

pub struct CreatePublisher {
    qos: QosKind<PublisherQos>,
    a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
    mask: Vec<StatusKind>,
}

impl CreatePublisher {
    pub fn new(
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            qos,
            a_listener,
            mask,
        }
    }
}

impl Message for CreatePublisher {
    type Result = DdsResult<ActorAddress<DdsPublisher>>;
}

impl Handler<CreatePublisher> for DdsDomainParticipant {
    fn handle(&mut self, message: CreatePublisher) -> <CreatePublisher as Message>::Result {
        self.create_publisher(message.qos)
    }
}

pub struct CreateSubscriber {
    qos: QosKind<SubscriberQos>,
    a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
    mask: Vec<StatusKind>,
}

impl CreateSubscriber {
    pub fn new(
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> Self {
        Self {
            qos,
            a_listener,
            mask,
        }
    }
}

impl Message for CreateSubscriber {
    type Result = DdsResult<ActorAddress<DdsSubscriber>>;
}

impl Handler<CreateSubscriber> for DdsDomainParticipant {
    fn handle(&mut self, message: CreateSubscriber) -> <CreateSubscriber as Message>::Result {
        self.create_subscriber(message.qos)
    }
}

pub struct GetDefaultUnicastLocatorList;

impl Message for GetDefaultUnicastLocatorList {
    type Result = Vec<Locator>;
}

impl Handler<GetDefaultUnicastLocatorList> for DdsDomainParticipant {
    fn handle(
        &mut self,
        message: GetDefaultUnicastLocatorList,
    ) -> <GetDefaultUnicastLocatorList as Message>::Result {
        self.default_unicast_locator_list().to_vec()
    }
}

pub struct GetDefaultMulticastLocatorList;

impl Message for GetDefaultMulticastLocatorList {
    type Result = Vec<Locator>;
}

impl Handler<GetDefaultMulticastLocatorList> for DdsDomainParticipant {
    fn handle(
        &mut self,
        message: GetDefaultMulticastLocatorList,
    ) -> <GetDefaultMulticastLocatorList as Message>::Result {
        self.default_multicast_locator_list().to_vec()
    }
}

pub struct GetDataMaxSizeSerialized;

impl Message for GetDataMaxSizeSerialized {
    type Result = usize;
}

impl Handler<GetDataMaxSizeSerialized> for DdsDomainParticipant {
    fn handle(
        &mut self,
        message: GetDataMaxSizeSerialized,
    ) -> <GetDataMaxSizeSerialized as Message>::Result {
        self.data_max_size_serialized()
    }
}
