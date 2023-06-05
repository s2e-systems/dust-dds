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
        rtps::{
            messages::overall_structure::{RtpsMessageRead, RtpsMessageWrite},
            types::Locator,
        },
        utils::actor::{ActorAddress, Mail, MailHandler},
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

impl ActorAddress<DdsDomainParticipant> {
    pub fn enable(&self) -> DdsResult<()> {
        struct Enable;

        impl Mail for Enable {
            type Result = ();
        }

        impl MailHandler<Enable> for DdsDomainParticipant {
            fn handle(&mut self, _message: Enable) -> <Enable as Mail>::Result {
                self.enable()
            }
        }

        self.send_blocking(Enable)
    }

    pub fn get_qos(&self) -> DdsResult<DomainParticipantQos> {
        struct GetQos;

        impl Mail for GetQos {
            type Result = DomainParticipantQos;
        }

        impl MailHandler<GetQos> for DdsDomainParticipant {
            fn handle(&mut self, _message: GetQos) -> <GetQos as Mail>::Result {
                self.get_qos()
            }
        }

        self.send_blocking(GetQos)
    }

    pub fn set_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        struct SetQos {
            qos: QosKind<DomainParticipantQos>,
        }

        impl Mail for SetQos {
            type Result = ();
        }

        impl MailHandler<SetQos> for DdsDomainParticipant {
            fn handle(&mut self, mail: SetQos) -> <SetQos as Mail>::Result {
                self.set_qos(mail.qos).ok();
            }
        }

        self.send_blocking(SetQos { qos })
    }

    pub fn get_domain_id(&self) -> DdsResult<DomainId> {
        struct GetDomainId;

        impl Mail for GetDomainId {
            type Result = DomainId;
        }

        impl MailHandler<GetDomainId> for DdsDomainParticipant {
            fn handle(&mut self, _message: GetDomainId) -> <GetDomainId as Mail>::Result {
                self.domain_id()
            }
        }

        self.send_blocking(GetDomainId)
    }

    pub fn get_instance_handle(&self) -> DdsResult<InstanceHandle> {
        struct GetInstanceHandle;

        impl Mail for GetInstanceHandle {
            type Result = InstanceHandle;
        }

        impl MailHandler<GetInstanceHandle> for DdsDomainParticipant {
            fn handle(
                &mut self,
                _message: GetInstanceHandle,
            ) -> <GetInstanceHandle as Mail>::Result {
                self.guid().into()
            }
        }

        self.send_blocking(GetInstanceHandle)
    }

    pub fn is_empty(&self) -> DdsResult<bool> {
        pub struct IsEmpty;

        impl Mail for IsEmpty {
            type Result = bool;
        }

        impl MailHandler<IsEmpty> for DdsDomainParticipant {
            fn handle(&mut self, _: IsEmpty) -> <IsEmpty as Mail>::Result {
                self.is_empty()
            }
        }

        self.send_blocking(IsEmpty)
    }

    pub fn create_topic(
        &self,
        topic_name: String,
        type_name: &'static str,
        qos: QosKind<TopicQos>,
        a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<ActorAddress<DdsTopic>> {
        struct CreateTopic {
            topic_name: String,
            type_name: &'static str,
            qos: QosKind<TopicQos>,
            a_listener: Option<Box<dyn AnyTopicListener + Send + Sync>>,
            mask: Vec<StatusKind>,
        }

        impl Mail for CreateTopic {
            type Result = DdsResult<ActorAddress<DdsTopic>>;
        }

        impl MailHandler<CreateTopic> for DdsDomainParticipant {
            fn handle(&mut self, message: CreateTopic) -> <CreateTopic as Mail>::Result {
                self.create_topic(&message.topic_name, message.type_name, message.qos)
            }
        }

        self.send_blocking(CreateTopic {
            topic_name,
            type_name,
            qos,
            a_listener,
            mask,
        })?
    }

    pub fn create_publisher(
        &self,
        qos: QosKind<PublisherQos>,
        a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<ActorAddress<DdsPublisher>> {
        struct CreatePublisher {
            qos: QosKind<PublisherQos>,
            a_listener: Option<Box<dyn PublisherListener + Send + Sync>>,
            mask: Vec<StatusKind>,
        }

        impl Mail for CreatePublisher {
            type Result = DdsResult<ActorAddress<DdsPublisher>>;
        }

        impl MailHandler<CreatePublisher> for DdsDomainParticipant {
            fn handle(&mut self, message: CreatePublisher) -> <CreatePublisher as Mail>::Result {
                self.create_publisher(message.qos)
            }
        }

        self.send_blocking(CreatePublisher {
            qos,
            a_listener,
            mask,
        })?
    }

    pub fn create_subscriber(
        &self,
        qos: QosKind<SubscriberQos>,
        a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<ActorAddress<DdsSubscriber>> {
        struct CreateSubscriber {
            qos: QosKind<SubscriberQos>,
            a_listener: Option<Box<dyn SubscriberListener + Send + Sync>>,
            mask: Vec<StatusKind>,
        }

        impl Mail for CreateSubscriber {
            type Result = DdsResult<ActorAddress<DdsSubscriber>>;
        }

        impl MailHandler<CreateSubscriber> for DdsDomainParticipant {
            fn handle(&mut self, message: CreateSubscriber) -> <CreateSubscriber as Mail>::Result {
                self.create_subscriber(message.qos)
            }
        }

        self.send_blocking(CreateSubscriber {
            qos,
            a_listener,
            mask,
        })?
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

impl Mail for ReceiveBuiltinMessage {
    type Result = ();
}

impl MailHandler<ReceiveBuiltinMessage> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _message: ReceiveBuiltinMessage,
    ) -> <ReceiveBuiltinMessage as Mail>::Result {
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

impl Mail for ReceiveUserDefinedMessage {
    type Result = ();
}

impl MailHandler<ReceiveUserDefinedMessage> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _message: ReceiveUserDefinedMessage,
    ) -> <ReceiveUserDefinedMessage as Mail>::Result {
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

impl Mail for AnnounceEntity {
    type Result = ();
}

impl MailHandler<AnnounceEntity> for DdsDomainParticipant {
    fn handle(&mut self, _message: AnnounceEntity) -> <AnnounceEntity as Mail>::Result {
        // todo!();
    }
}

pub struct AnnounceParticipant;

impl Mail for AnnounceParticipant {
    type Result = ();
}

impl MailHandler<AnnounceParticipant> for DdsDomainParticipant {
    fn handle(&mut self, _message: AnnounceParticipant) -> <AnnounceParticipant as Mail>::Result {
        self.announce_participant().ok();
    }
}

pub struct GetDefaultUnicastLocatorList;

impl Mail for GetDefaultUnicastLocatorList {
    type Result = Vec<Locator>;
}

impl MailHandler<GetDefaultUnicastLocatorList> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _mail: GetDefaultUnicastLocatorList,
    ) -> <GetDefaultUnicastLocatorList as Mail>::Result {
        self.default_unicast_locator_list().to_vec()
    }
}

pub struct GetDefaultMulticastLocatorList;

impl Mail for GetDefaultMulticastLocatorList {
    type Result = Vec<Locator>;
}

impl MailHandler<GetDefaultMulticastLocatorList> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _mail: GetDefaultMulticastLocatorList,
    ) -> <GetDefaultMulticastLocatorList as Mail>::Result {
        self.default_multicast_locator_list().to_vec()
    }
}

pub struct GetDataMaxSizeSerialized;

impl Mail for GetDataMaxSizeSerialized {
    type Result = usize;
}

impl MailHandler<GetDataMaxSizeSerialized> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _mail: GetDataMaxSizeSerialized,
    ) -> <GetDataMaxSizeSerialized as Mail>::Result {
        self.data_max_size_serialized()
    }
}

pub struct DeleteContainedEntities;

impl Mail for DeleteContainedEntities {
    type Result = ();
}

impl MailHandler<DeleteContainedEntities> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _mail: DeleteContainedEntities,
    ) -> <DeleteContainedEntities as Mail>::Result {
        self.delete_contained_entities().ok();
    }
}

pub struct GetUserDefinedRtpsMessageChannelSender;

impl Mail for GetUserDefinedRtpsMessageChannelSender {
    type Result = tokio::sync::mpsc::Sender<(RtpsMessageWrite, Vec<Locator>)>;
}

impl MailHandler<GetUserDefinedRtpsMessageChannelSender> for DdsDomainParticipant {
    fn handle(
        &mut self,
        _mail: GetUserDefinedRtpsMessageChannelSender,
    ) -> <GetUserDefinedRtpsMessageChannelSender as Mail>::Result {
        self.get_user_defined_rtps_message_channel_sender()
    }
}
