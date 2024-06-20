use std::{collections::HashMap, sync::Arc, thread::JoinHandle};

use fnmatch_regex::glob_to_regex;
use tracing::warn;

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    data_reader_actor::{self, DataReaderActor},
    domain_participant_actor::ParticipantListenerMessage,
    message_sender_actor::MessageSenderActor,
    topic_actor::TopicActor,
};
use crate::{
    data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
    dds_async::{
        domain_participant::DomainParticipantAsync, subscriber::SubscriberAsync,
        subscriber_listener::SubscriberListenerAsync,
    },
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        actors::status_condition_actor::StatusConditionActor,
        runtime::{
            executor::{block_on, ExecutorHandle},
            mpsc::{mpsc_channel, MpscSender},
            timer::TimerHandle,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        qos_policy::PartitionQosPolicy,
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
    },
    rtps::{
        self,
        behavior_types::DURATION_ZERO,
        endpoint::RtpsEndpoint,
        group::RtpsGroup,
        messages::submessages::{
            data::DataSubmessage, data_frag::DataFragSubmessage, gap::GapSubmessage,
            heartbeat::HeartbeatSubmessage, heartbeat_frag::HeartbeatFragSubmessage,
        },
        reader::{RtpsReader, RtpsReaderKind, RtpsStatefulReader},
        types::{
            EntityId, Guid, GuidPrefix, Locator, TopicKind, USER_DEFINED_READER_NO_KEY,
            USER_DEFINED_READER_WITH_KEY,
        },
    },
    topic_definition::type_support::DynamicTypeInterface,
};

pub enum SubscriberListenerOperation {
    DataOnReaders(SubscriberAsync),
    _DataAvailable,
    SampleRejected(SampleRejectedStatus),
    _LivenessChanged(LivelinessChangedStatus),
    RequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    RequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    SubscriptionMatched(SubscriptionMatchedStatus),
    SampleLost(SampleLostStatus),
}

pub struct SubscriberListenerMessage {
    pub listener_operation: SubscriberListenerOperation,
}

struct SubscriberListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<SubscriberListenerMessage>,
}

impl SubscriberListenerThread {
    fn new(mut listener: Box<dyn SubscriberListenerAsync + Send>) -> Self {
        let (sender, receiver) = mpsc_channel::<SubscriberListenerMessage>();
        let thread = std::thread::spawn(move || {
            block_on(async {
                while let Some(m) = receiver.recv().await {
                    match m.listener_operation {
                        SubscriberListenerOperation::DataOnReaders(the_subscriber) => {
                            listener.on_data_on_readers(the_subscriber).await
                        }
                        SubscriberListenerOperation::_DataAvailable => {
                            listener.on_data_available(&()).await
                        }
                        SubscriberListenerOperation::SampleRejected(status) => {
                            listener.on_sample_rejected(&(), status).await
                        }
                        SubscriberListenerOperation::_LivenessChanged(status) => {
                            listener.on_liveliness_changed(&(), status).await
                        }
                        SubscriberListenerOperation::RequestedDeadlineMissed(status) => {
                            listener.on_requested_deadline_missed(&(), status).await
                        }
                        SubscriberListenerOperation::RequestedIncompatibleQos(status) => {
                            listener.on_requested_incompatible_qos(&(), status).await
                        }
                        SubscriberListenerOperation::SubscriptionMatched(status) => {
                            listener.on_subscription_matched(&(), status).await
                        }
                        SubscriberListenerOperation::SampleLost(status) => {
                            listener.on_sample_lost(&(), status).await
                        }
                    }
                }
            });
        });
        Self { thread, sender }
    }

    fn sender(&self) -> &MpscSender<SubscriberListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}

pub struct SubscriberActor {
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    data_reader_list: HashMap<InstanceHandle, Actor<DataReaderActor>>,
    enabled: bool,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    status_condition: Actor<StatusConditionActor>,
    subscriber_listener_thread: Option<SubscriberListenerThread>,
    status_kind: Vec<StatusKind>,
}

impl SubscriberActor {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        data_reader_list: Vec<DataReaderActor>,
        handle: &ExecutorHandle,
    ) -> (Self, ActorAddress<StatusConditionActor>) {
        let status_condition = Actor::spawn(StatusConditionActor::default(), handle);
        let subscriber_listener_thread = listener.map(SubscriberListenerThread::new);
        let data_reader_list = data_reader_list
            .into_iter()
            .map(|dr| (dr.get_instance_handle(), Actor::spawn(dr, handle)))
            .collect();
        let status_condition_address = status_condition.address();
        (
            SubscriberActor {
                qos,
                rtps_group,
                data_reader_list,
                enabled: false,
                user_defined_data_reader_counter: 0,
                default_data_reader_qos: Default::default(),
                status_condition,
                subscriber_listener_thread,
                status_kind,
            },
            status_condition_address,
        )
    }

    fn get_unique_reader_id(&mut self) -> u8 {
        let counter = self.user_defined_data_reader_counter;
        self.user_defined_data_reader_counter += 1;
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
                        "Invalid partition regex name on subscriber qos {:?}. Error {:?}",
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

pub struct CreateDatareader {
    pub topic_address: ActorAddress<TopicActor>,
    pub topic_name: String,
    pub type_name: String,
    pub topic_status_condition: ActorAddress<StatusConditionActor>,
    pub type_support: Arc<dyn DynamicTypeInterface + Send + Sync>,
    pub has_key: bool,
    pub qos: QosKind<DataReaderQos>,
    pub a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
    pub mask: Vec<StatusKind>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub executor_handle: ExecutorHandle,
}
impl Mail for CreateDatareader {
    type Result = DdsResult<ActorAddress<DataReaderActor>>;
}
impl MailHandler<CreateDatareader> for SubscriberActor {
    fn handle(&mut self, message: CreateDatareader) -> <CreateDatareader as Mail>::Result {
        let qos = match message.qos {
            QosKind::Default => self.default_data_reader_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let entity_kind = match message.has_key {
            true => USER_DEFINED_READER_WITH_KEY,
            false => USER_DEFINED_READER_NO_KEY,
        };
        let subscriber_guid = self.rtps_group.guid();

        let entity_key: [u8; 3] = [
            subscriber_guid.entity_id().entity_key()[0],
            self.get_unique_reader_id(),
            0,
        ];

        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = Guid::new(subscriber_guid.prefix(), entity_id);

        let topic_kind = match message.has_key {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };

        let rtps_reader = RtpsReaderKind::Stateful(RtpsStatefulReader::new(RtpsReader::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                &message.default_unicast_locator_list,
                &message.default_multicast_locator_list,
            ),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        )));

        let status_kind = message.mask.to_vec();
        let data_reader = DataReaderActor::new(
            rtps_reader,
            message.topic_address,
            message.topic_name,
            message.type_name,
            message.topic_status_condition,
            message.type_support,
            qos,
            message.a_listener,
            status_kind,
            &message.executor_handle,
        );

        let reader_actor = Actor::spawn(data_reader, &message.executor_handle);
        let reader_address = reader_actor.address();
        self.data_reader_list
            .insert(InstanceHandle::new(guid.into()), reader_actor);

        Ok(reader_address)
    }
}

pub struct DeleteDatareader {
    pub handle: InstanceHandle,
}
impl Mail for DeleteDatareader {
    type Result = DdsResult<Actor<DataReaderActor>>;
}
impl MailHandler<DeleteDatareader> for SubscriberActor {
    fn handle(&mut self, message: DeleteDatareader) -> <DeleteDatareader as Mail>::Result {
        if let Some(removed_reader) = self.data_reader_list.remove(&message.handle) {
            Ok(removed_reader)
        } else {
            Err(DdsError::PreconditionNotMet(
                "Data reader can only be deleted from its parent subscriber".to_string(),
            ))
        }
    }
}

pub struct DrainDataReaderList;
impl Mail for DrainDataReaderList {
    type Result = Vec<Actor<DataReaderActor>>;
}
impl MailHandler<DrainDataReaderList> for SubscriberActor {
    fn handle(&mut self, _: DrainDataReaderList) -> <DrainDataReaderList as Mail>::Result {
        self.data_reader_list.drain().map(|(_, a)| a).collect()
    }
}

pub struct GetGuid;
impl Mail for GetGuid {
    type Result = Guid;
}
impl MailHandler<GetGuid> for SubscriberActor {
    fn handle(&mut self, _: GetGuid) -> <GetGuid as Mail>::Result {
        self.rtps_group.guid()
    }
}

pub struct IsEmpty;
impl Mail for IsEmpty {
    type Result = bool;
}
impl MailHandler<IsEmpty> for SubscriberActor {
    fn handle(&mut self, _: IsEmpty) -> <IsEmpty as Mail>::Result {
        self.data_reader_list.is_empty()
    }
}

pub struct IsEnabled;
impl Mail for IsEnabled {
    type Result = bool;
}
impl MailHandler<IsEnabled> for SubscriberActor {
    fn handle(&mut self, _: IsEnabled) -> <IsEnabled as Mail>::Result {
        self.enabled
    }
}

pub struct SetDefaultDatareaderQos {
    pub qos: QosKind<DataReaderQos>,
}
impl Mail for SetDefaultDatareaderQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetDefaultDatareaderQos> for SubscriberActor {
    fn handle(
        &mut self,
        message: SetDefaultDatareaderQos,
    ) -> <SetDefaultDatareaderQos as Mail>::Result {
        match message.qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }
}

pub struct GetDefaultDatareaderQos;
impl Mail for GetDefaultDatareaderQos {
    type Result = DataReaderQos;
}
impl MailHandler<GetDefaultDatareaderQos> for SubscriberActor {
    fn handle(&mut self, _: GetDefaultDatareaderQos) -> <GetDefaultDatareaderQos as Mail>::Result {
        self.default_data_reader_qos.clone()
    }
}

pub struct SetQos {
    pub qos: QosKind<SubscriberQos>,
}
impl Mail for SetQos {
    type Result = DdsResult<()>;
}
impl MailHandler<SetQos> for SubscriberActor {
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

pub struct Enable;
impl Mail for Enable {
    type Result = ();
}
impl MailHandler<Enable> for SubscriberActor {
    fn handle(&mut self, _: Enable) -> <Enable as Mail>::Result {
        self.enabled = true;
    }
}

pub struct GetInstanceHandle;
impl Mail for GetInstanceHandle {
    type Result = InstanceHandle;
}
impl MailHandler<GetInstanceHandle> for SubscriberActor {
    fn handle(&mut self, _: GetInstanceHandle) -> <GetInstanceHandle as Mail>::Result {
        InstanceHandle::new(self.rtps_group.guid().into())
    }
}

pub struct GetStatuscondition;
impl Mail for GetStatuscondition {
    type Result = ActorAddress<StatusConditionActor>;
}
impl MailHandler<GetStatuscondition> for SubscriberActor {
    fn handle(&mut self, _: GetStatuscondition) -> <GetStatuscondition as Mail>::Result {
        self.status_condition.address()
    }
}

pub struct GetQos;
impl Mail for GetQos {
    type Result = SubscriberQos;
}
impl MailHandler<GetQos> for SubscriberActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct GetDataReaderList;
impl Mail for GetDataReaderList {
    type Result = Vec<ActorAddress<DataReaderActor>>;
}
impl MailHandler<GetDataReaderList> for SubscriberActor {
    fn handle(&mut self, _: GetDataReaderList) -> <GetDataReaderList as Mail>::Result {
        self.data_reader_list
            .values()
            .map(|dr| dr.address())
            .collect()
    }
}

pub struct GetStatusKind;
impl Mail for GetStatusKind {
    type Result = Vec<StatusKind>;
}
impl MailHandler<GetStatusKind> for SubscriberActor {
    fn handle(&mut self, _: GetStatusKind) -> <GetStatusKind as Mail>::Result {
        self.status_kind.clone()
    }
}

pub struct ProcessDataSubmessage {
    pub data_submessage: DataSubmessage,
    pub source_guid_prefix: GuidPrefix,
    pub source_timestamp: Option<rtps::messages::types::Time>,
    pub reception_timestamp: rtps::messages::types::Time,
    pub subscriber_address: ActorAddress<SubscriberActor>,
    pub participant: DomainParticipantAsync,
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for ProcessDataSubmessage {
    type Result = ();
}
impl MailHandler<ProcessDataSubmessage> for SubscriberActor {
    fn handle(
        &mut self,
        message: ProcessDataSubmessage,
    ) -> <ProcessDataSubmessage as Mail>::Result {
        for data_reader_actor in self.data_reader_list.values() {
            let subscriber_mask_listener = (
                self.subscriber_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.status_kind.clone(),
            );
            data_reader_actor.send_actor_mail(data_reader_actor::ProcessDataSubmessage {
                data_submessage: message.data_submessage.clone(),
                source_guid_prefix: message.source_guid_prefix,
                source_timestamp: message.source_timestamp,
                reception_timestamp: message.reception_timestamp,
                data_reader_address: data_reader_actor.address(),
                subscriber: SubscriberAsync::new(
                    message.subscriber_address.clone(),
                    self.status_condition.address(),
                    message.participant.clone(),
                ),
                subscriber_mask_listener,
                participant_mask_listener: message.participant_mask_listener.clone(),
                executor_handle: message.executor_handle.clone(),
                timer_handle: message.timer_handle.clone(),
            });
        }
    }
}

pub struct ProcessDataFragSubmessage {
    pub data_frag_submessage: DataFragSubmessage,
    pub source_guid_prefix: GuidPrefix,
    pub source_timestamp: Option<rtps::messages::types::Time>,
    pub reception_timestamp: rtps::messages::types::Time,
    pub subscriber_address: ActorAddress<SubscriberActor>,
    pub participant: DomainParticipantAsync,
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for ProcessDataFragSubmessage {
    type Result = ();
}
impl MailHandler<ProcessDataFragSubmessage> for SubscriberActor {
    fn handle(
        &mut self,
        message: ProcessDataFragSubmessage,
    ) -> <ProcessDataFragSubmessage as Mail>::Result {
        for data_reader_actor in self.data_reader_list.values() {
            let subscriber_mask_listener = (
                self.subscriber_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.status_kind.clone(),
            );
            data_reader_actor.send_actor_mail(data_reader_actor::ProcessDataFragSubmessage {
                data_frag_submessage: message.data_frag_submessage.clone(),
                source_guid_prefix: message.source_guid_prefix,
                source_timestamp: message.source_timestamp,
                reception_timestamp: message.reception_timestamp,
                data_reader_address: data_reader_actor.address(),
                subscriber: SubscriberAsync::new(
                    message.subscriber_address.clone(),
                    self.status_condition.address(),
                    message.participant.clone(),
                ),
                subscriber_mask_listener,
                participant_mask_listener: message.participant_mask_listener.clone(),
                executor_handle: message.executor_handle.clone(),
                timer_handle: message.timer_handle.clone(),
            });
        }
    }
}

pub struct ProcessGapSubmessage {
    pub gap_submessage: GapSubmessage,
    pub source_guid_prefix: GuidPrefix,
}
impl Mail for ProcessGapSubmessage {
    type Result = ();
}
impl MailHandler<ProcessGapSubmessage> for SubscriberActor {
    fn handle(&mut self, message: ProcessGapSubmessage) -> <ProcessGapSubmessage as Mail>::Result {
        for data_reader_actor in self.data_reader_list.values() {
            data_reader_actor.send_actor_mail(data_reader_actor::ProcessGapSubmessage {
                gap_submessage: message.gap_submessage.clone(),
                source_guid_prefix: message.source_guid_prefix,
            });
        }
    }
}

pub struct ProcessHeartbeatSubmessage {
    pub heartbeat_submessage: HeartbeatSubmessage,
    pub source_guid_prefix: GuidPrefix,
    pub message_sender_actor: ActorAddress<MessageSenderActor>,
}
impl Mail for ProcessHeartbeatSubmessage {
    type Result = ();
}
impl MailHandler<ProcessHeartbeatSubmessage> for SubscriberActor {
    fn handle(
        &mut self,
        message: ProcessHeartbeatSubmessage,
    ) -> <ProcessHeartbeatSubmessage as Mail>::Result {
        for data_reader_actor in self.data_reader_list.values() {
            data_reader_actor.send_actor_mail(data_reader_actor::ProcessHeartbeatSubmessage {
                heartbeat_submessage: message.heartbeat_submessage.clone(),
                source_guid_prefix: message.source_guid_prefix,
                message_sender_actor: message.message_sender_actor.clone(),
            });
        }
    }
}

pub struct ProcessHeartbeatFragSubmessage {
    pub heartbeat_frag_submessage: HeartbeatFragSubmessage,
    pub source_guid_prefix: GuidPrefix,
}
impl Mail for ProcessHeartbeatFragSubmessage {
    type Result = ();
}
impl MailHandler<ProcessHeartbeatFragSubmessage> for SubscriberActor {
    fn handle(
        &mut self,
        message: ProcessHeartbeatFragSubmessage,
    ) -> <ProcessHeartbeatFragSubmessage as Mail>::Result {
        for data_reader_actor in self.data_reader_list.values() {
            data_reader_actor.send_actor_mail(data_reader_actor::ProcessHeartbeatFragSubmessage {
                heartbeat_frag_submessage: message.heartbeat_frag_submessage.clone(),
                source_guid_prefix: message.source_guid_prefix,
            });
        }
    }
}

pub struct AddMatchedWriter {
    pub discovered_writer_data: DiscoveredWriterData,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub subscriber_address: ActorAddress<SubscriberActor>,
    pub participant: DomainParticipantAsync,
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
}
impl Mail for AddMatchedWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<AddMatchedWriter> for SubscriberActor {
    fn handle(&mut self, message: AddMatchedWriter) -> <AddMatchedWriter as Mail>::Result {
        if self.is_partition_matched(
            message
                .discovered_writer_data
                .dds_publication_data()
                .partition(),
        ) {
            for data_reader in self.data_reader_list.values() {
                let subscriber_mask_listener = (
                    self.subscriber_listener_thread
                        .as_ref()
                        .map(|l| l.sender().clone()),
                    self.status_kind.clone(),
                );
                let data_reader_address = data_reader.address();
                let subscriber_qos = self.qos.clone();
                data_reader.send_actor_mail(data_reader_actor::AddMatchedWriter {
                    discovered_writer_data: message.discovered_writer_data.clone(),
                    default_unicast_locator_list: message.default_unicast_locator_list.clone(),
                    default_multicast_locator_list: message.default_multicast_locator_list.clone(),
                    data_reader_address,
                    subscriber: SubscriberAsync::new(
                        message.subscriber_address.clone(),
                        self.status_condition.address(),
                        message.participant.clone(),
                    ),
                    subscriber_qos,
                    subscriber_mask_listener,
                    participant_mask_listener: message.participant_mask_listener.clone(),
                });
            }
        }
        Ok(())
    }
}

pub struct RemoveMatchedWriter {
    pub discovered_writer_handle: InstanceHandle,
    pub subscriber_address: ActorAddress<SubscriberActor>,
    pub participant: DomainParticipantAsync,
    pub participant_mask_listener: (
        Option<MpscSender<ParticipantListenerMessage>>,
        Vec<StatusKind>,
    ),
}
impl Mail for RemoveMatchedWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedWriter> for SubscriberActor {
    fn handle(&mut self, message: RemoveMatchedWriter) -> <RemoveMatchedWriter as Mail>::Result {
        for data_reader in self.data_reader_list.values() {
            let data_reader_address = data_reader.address();
            let subscriber_mask_listener = (
                self.subscriber_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.status_kind.clone(),
            );
            data_reader.send_actor_mail(data_reader_actor::RemoveMatchedWriter {
                discovered_writer_handle: message.discovered_writer_handle,
                data_reader_address,
                subscriber: SubscriberAsync::new(
                    message.subscriber_address.clone(),
                    self.status_condition.address(),
                    message.participant.clone(),
                ),
                subscriber_mask_listener,
                participant_mask_listener: message.participant_mask_listener.clone(),
            });
        }

        Ok(())
    }
}

pub struct SetListener {
    pub listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
    pub status_kind: Vec<StatusKind>,
}
impl Mail for SetListener {
    type Result = DdsResult<()>;
}
impl MailHandler<SetListener> for SubscriberActor {
    fn handle(&mut self, message: SetListener) -> <SetListener as Mail>::Result {
        if let Some(l) = self.subscriber_listener_thread.take() {
            l.join()?;
        }
        self.subscriber_listener_thread = message.listener.map(SubscriberListenerThread::new);
        self.status_kind = message.status_kind;
        Ok(())
    }
}
