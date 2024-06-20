use std::{collections::HashMap, thread::JoinHandle};

use fnmatch_regex::glob_to_regex;
use tracing::warn;

use crate::{
    data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
    dds_async::{
        domain_participant::DomainParticipantAsync, publisher::PublisherAsync,
        publisher_listener::PublisherListenerAsync,
    },
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        runtime::{
            executor::{block_on, ExecutorHandle},
            mpsc::{mpsc_channel, MpscSender},
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        qos_policy::PartitionQosPolicy,
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
        time::Duration,
    },
    rtps::{
        behavior_types::DURATION_ZERO,
        endpoint::RtpsEndpoint,
        group::RtpsGroup,
        messages::submessages::{ack_nack::AckNackSubmessage, nack_frag::NackFragSubmessage},
        types::{
            EntityId, Guid, GuidPrefix, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
        writer::RtpsWriter,
    },
};

use super::{
    any_data_writer_listener::AnyDataWriterListener,
    data_writer_actor::{self, DataWriterActor},
    domain_participant_actor::ParticipantListenerMessage,
    message_sender_actor::MessageSenderActor,
    status_condition_actor::StatusConditionActor,
    topic_actor::TopicActor,
};

pub enum PublisherListenerOperation {
    _LivelinessLost(LivelinessLostStatus),
    _OfferedDeadlineMissed(OfferedDeadlineMissedStatus),
    OfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    PublicationMatched(PublicationMatchedStatus),
}

pub struct PublisherListenerMessage {
    pub listener_operation: PublisherListenerOperation,
}

struct PublisherListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<PublisherListenerMessage>,
}

impl PublisherListenerThread {
    fn new(mut listener: Box<dyn PublisherListenerAsync + Send>) -> Self {
        let (sender, receiver) = mpsc_channel::<PublisherListenerMessage>();
        let thread = std::thread::spawn(move || {
            block_on(async {
                while let Some(m) = receiver.recv().await {
                    match m.listener_operation {
                        PublisherListenerOperation::_LivelinessLost(status) => {
                            listener.on_liveliness_lost(&(), status).await
                        }
                        PublisherListenerOperation::_OfferedDeadlineMissed(status) => {
                            listener.on_offered_deadline_missed(&(), status).await
                        }
                        PublisherListenerOperation::OfferedIncompatibleQos(status) => {
                            listener.on_offered_incompatible_qos(&(), status).await
                        }
                        PublisherListenerOperation::PublicationMatched(status) => {
                            listener.on_publication_matched(&(), status).await
                        }
                    }
                }
            });
        });
        Self { thread, sender }
    }

    fn sender(&self) -> &MpscSender<PublisherListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}

pub struct PublisherActor {
    qos: PublisherQos,
    rtps_group: RtpsGroup,
    data_writer_list: HashMap<InstanceHandle, Actor<DataWriterActor>>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
    publisher_listener_thread: Option<PublisherListenerThread>,
    status_kind: Vec<StatusKind>,
    status_condition: Actor<StatusConditionActor>,
}

impl PublisherActor {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        data_writer_list: Vec<DataWriterActor>,
        handle: &ExecutorHandle,
    ) -> Self {
        let data_writer_list = data_writer_list
            .into_iter()
            .map(|dw| (dw.get_instance_handle(), Actor::spawn(dw, handle)))
            .collect();
        let publisher_listener_thread = listener.map(PublisherListenerThread::new);
        Self {
            qos,
            rtps_group,
            data_writer_list,
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
            publisher_listener_thread,
            status_kind,
            status_condition: Actor::spawn(StatusConditionActor::default(), handle),
        }
    }

    pub fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    fn is_partition_matched(&self, discovered_partition_qos_policy: &PartitionQosPolicy) -> bool {
        let is_any_name_matched = discovered_partition_qos_policy
            .name
            .iter()
            .any(|n| self.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_partition_qos_policy
            .name
            .iter()
            .filter_map(|n| match glob_to_regex(n) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    warn!(
                        "Received invalid partition regex name {:?}. Error {:?}",
                        n, e
                    );
                    None
                }
            })
            .any(|regex| self.qos.partition.name.iter().any(|n| regex.is_match(n)));

        let is_any_local_regex_matched_with_received_partition_qos = self
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| match glob_to_regex(n) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    warn!(
                        "Invalid partition regex name on publisher qos {:?}. Error {:?}",
                        n, e
                    );
                    None
                }
            })
            .any(|regex| {
                discovered_partition_qos_policy
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        discovered_partition_qos_policy == &self.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos
    }
}

pub struct CreateDatawriter {
    pub topic_address: ActorAddress<TopicActor>,
    pub topic_name: String,
    pub type_name: String,
    pub topic_status_condition: ActorAddress<StatusConditionActor>,
    pub has_key: bool,
    pub data_max_size_serialized: usize,
    pub qos: QosKind<DataWriterQos>,
    pub a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
    pub mask: Vec<StatusKind>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub executor_handle: ExecutorHandle,
}
impl Mail for CreateDatawriter {
    type Result = DdsResult<ActorAddress<DataWriterActor>>;
}
impl MailHandler<CreateDatawriter> for PublisherActor {
    fn handle(&mut self, message: CreateDatawriter) -> <CreateDatawriter as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => self.default_datawriter_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let guid_prefix = self.rtps_group.guid().prefix();
        let (entity_kind, topic_kind) = match message.has_key {
            true => (USER_DEFINED_WRITER_WITH_KEY, TopicKind::WithKey),
            false => (USER_DEFINED_WRITER_NO_KEY, TopicKind::NoKey),
        };
        let entity_key = [
            self.rtps_group.guid().entity_id().entity_key()[0],
            self.get_unique_writer_id(),
            0,
        ];
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = Guid::new(guid_prefix, entity_id);

        let rtps_writer_impl = RtpsWriter::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                &message.default_unicast_locator_list,
                &message.default_multicast_locator_list,
            ),
            true,
            Duration::new(0, 200_000_000).into(),
            DURATION_ZERO,
            DURATION_ZERO,
            message.data_max_size_serialized,
        );

        let data_writer = DataWriterActor::new(
            rtps_writer_impl,
            message.topic_address,
            message.topic_name,
            message.type_name,
            message.topic_status_condition,
            message.a_listener,
            message.mask,
            qos,
            &message.executor_handle,
        );
        let data_writer_actor = Actor::spawn(data_writer, &message.executor_handle);
        let data_writer_address = data_writer_actor.address();
        self.data_writer_list
            .insert(InstanceHandle::new(guid.into()), data_writer_actor);

        Ok(data_writer_address)
    }
}

pub struct DeleteDatawriter {
    pub handle: InstanceHandle,
}
impl Mail for DeleteDatawriter {
    type Result = DdsResult<Actor<DataWriterActor>>;
}
impl MailHandler<DeleteDatawriter> for PublisherActor {
    fn handle(&mut self, message: DeleteDatawriter) -> <DeleteDatawriter as Mail>::Result {
        if let Some(removed_writer) = self.data_writer_list.remove(&message.handle) {
            Ok(removed_writer)
        } else {
            Err(DdsError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))
        }
    }
}

pub struct Enable;
impl Mail for Enable {
    type Result = ();
}
impl MailHandler<Enable> for PublisherActor {
    fn handle(&mut self, _: Enable) -> <Enable as Mail>::Result {
        self.enabled = true;
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for PublisherActor {
    fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.enabled
    }
}

pub struct IsEmpty;
impl Mail for IsEmpty {
    type Result = bool;
}
impl MailHandler<IsEmpty> for PublisherActor {
    fn handle(&mut self, _: IsEmpty) -> <IsEmpty as Mail>::Result {
        self.data_writer_list.is_empty()
    }
}

pub struct DrainDataWriterList;
impl Mail for DrainDataWriterList {
    type Result = Vec<Actor<DataWriterActor>>;
}
impl MailHandler<DrainDataWriterList> for PublisherActor {
    fn handle(&mut self, _: DrainDataWriterList) -> <DrainDataWriterList as Mail>::Result {
        self.data_writer_list.drain().map(|(_, a)| a).collect()
    }
}

pub struct SetDefaultDatawriterQos {
    pub qos: DataWriterQos,
}
impl Mail for SetDefaultDatawriterQos {
    type Result = ();
}
impl MailHandler<SetDefaultDatawriterQos> for PublisherActor {
    fn handle(
        &mut self,
        message: SetDefaultDatawriterQos,
    ) -> <SetDefaultDatawriterQos as Mail>::Result {
        self.default_datawriter_qos = message.qos;
    }
}

pub struct GetDefaultDatawriterQos;
impl Mail for GetDefaultDatawriterQos {
    type Result = DataWriterQos;
}
impl MailHandler<GetDefaultDatawriterQos> for PublisherActor {
    fn handle(&mut self, _: GetDefaultDatawriterQos) -> <GetDefaultDatawriterQos as Mail>::Result {
        self.default_datawriter_qos.clone()
    }
}

pub struct SetQos {
    pub qos: QosKind<PublisherQos>,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for PublisherActor {
    fn handle(&mut self, message: SetQos) -> <SetQos as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        self.qos = qos;

        Ok(())
    }
}

pub struct GetGuid;
impl Mail for GetGuid {
    type Result = Guid;
}
impl MailHandler<GetGuid> for PublisherActor {
    fn handle(&mut self, _: GetGuid) -> <GetGuid as Mail>::Result {
        self.rtps_group.guid()
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for PublisherActor {
    fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        InstanceHandle::new(self.rtps_group.guid().into())
    }
}

pub struct GetStatusKind;
impl Mail for GetStatusKind {
    type Result = Vec<StatusKind>;
}
impl MailHandler<GetStatusKind> for PublisherActor {
    fn handle(&mut self, _: GetStatusKind) -> <GetStatusKind as Mail>::Result {
        self.status_kind.clone()
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = PublisherQos;
}
impl MailHandler<GetQos> for PublisherActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct GetDataWriterList;
impl Mail for GetDataWriterList {
    type Result = Vec<ActorAddress<DataWriterActor>>;
}
impl MailHandler<GetDataWriterList> for PublisherActor {
    fn handle(&mut self, _: GetDataWriterList) -> <GetDataWriterList as Mail>::Result {
        self.data_writer_list
            .values()
            .map(|x| x.address())
            .collect()
    }
}

pub struct ProcessAckNackSubmessage {
    pub acknack_submessage: AckNackSubmessage,
    pub source_guid_prefix: GuidPrefix,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for ProcessAckNackSubmessage {
    type Result = ();
}
impl MailHandler<ProcessAckNackSubmessage> for PublisherActor {
    fn handle(
        &mut self,
        message: ProcessAckNackSubmessage,
    ) -> <ProcessAckNackSubmessage as Mail>::Result {
        for data_writer_actor in self.data_writer_list.values() {
            data_writer_actor.send_actor_mail(data_writer_actor::ProcessAckNackSubmessage {
                acknack_submessage: message.acknack_submessage.clone(),
                source_guid_prefix: message.source_guid_prefix,
                message_sender_actor: message.message_sender_actor.clone(),
            });
        }
    }
}

pub struct ProcessNackFragSubmessage {
    pub nackfrag_submessage: NackFragSubmessage,
    pub source_guid_prefix: GuidPrefix,
}
impl Mail for ProcessNackFragSubmessage {
    type Result = ();
}
impl MailHandler<ProcessNackFragSubmessage> for PublisherActor {
    fn handle(
        &mut self,
        message: ProcessNackFragSubmessage,
    ) -> <ProcessNackFragSubmessage as Mail>::Result {
        for data_writer_actor in self.data_writer_list.values() {
            data_writer_actor.send_actor_mail(data_writer_actor::ProcessNackFragSubmessage {
                nackfrag_submessage: message.nackfrag_submessage.clone(),
                source_guid_prefix: message.source_guid_prefix,
            });
        }
    }
}

pub struct AddMatchedReader {
    pub discovered_reader_data: DiscoveredReaderData,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub publisher_address: ActorAddress<PublisherActor>,
    pub participant: DomainParticipantAsync,
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for AddMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<AddMatchedReader> for PublisherActor {
    fn handle(&mut self, message: AddMatchedReader) -> <AddMatchedReader as Mail>::Result {
        if self.is_partition_matched(
            message
                .discovered_reader_data
                .subscription_builtin_topic_data()
                .partition(),
        ) {
            for data_writer in self.data_writer_list.values() {
                let data_writer_address = data_writer.address();
                let publisher_mask_listener = (
                    self.publisher_listener_thread
                        .as_ref()
                        .map(|l| l.sender().clone()),
                    self.status_kind.clone(),
                );

                data_writer.send_actor_mail(data_writer_actor::AddMatchedReader {
                    discovered_reader_data: message.discovered_reader_data.clone(),
                    default_unicast_locator_list: message.default_unicast_locator_list.clone(),
                    default_multicast_locator_list: message.default_multicast_locator_list.clone(),
                    data_writer_address,
                    publisher: PublisherAsync::new(
                        message.publisher_address.clone(),
                        self.status_condition.address(),
                        message.participant.clone(),
                    ),
                    publisher_qos: self.qos.clone(),
                    publisher_mask_listener,
                    participant_mask_listener: message.participant_mask_listener.clone(),
                    message_sender_actor: message.message_sender_actor.clone(),
                });
            }
        }
        Ok(())
    }
}

pub struct RemoveMatchedReader {
    pub discovered_reader_handle: InstanceHandle,
    pub publisher_address: ActorAddress<PublisherActor>,
    pub participant: DomainParticipantAsync,
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
}
impl Mail for RemoveMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedReader> for PublisherActor {
    fn handle(&mut self, message: RemoveMatchedReader) -> <RemoveMatchedReader as Mail>::Result {
        for data_writer in self.data_writer_list.values() {
            let data_writer_address = data_writer.address();
            let publisher_mask_listener = (
                self.publisher_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.status_kind.clone(),
            );
            data_writer.send_actor_mail(data_writer_actor::RemoveMatchedReader {
                discovered_reader_handle: message.discovered_reader_handle,
                data_writer_address,
                publisher: PublisherAsync::new(
                    message.publisher_address.clone(),
                    self.status_condition.address(),
                    message.participant.clone(),
                ),
                publisher_mask_listener,
                participant_mask_listener: message.participant_mask_listener.clone(),
            });
        }
        Ok(())
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for PublisherActor {
    fn handle(&mut self, _: GetStatuscondition) -> <GetStatuscondition as Mail>::Result {
        self.status_condition.address()
    }
}

pub struct SetListener {
    pub listener: Option<Box<dyn PublisherListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for PublisherActor {
    fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        if let Some(l) = self.publisher_listener_thread.take() {
            l.join()?;
        }
        self.publisher_listener_thread = message.listener.map(PublisherListenerThread::new);
        self.status_kind = message.status_kind;
        Ok(())
    }
}

impl PublisherQos {
    fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.presentation != other.presentation {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}
