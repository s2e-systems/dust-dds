use super::{
    any_data_reader_listener::AnyDataReaderListener,
    data_reader_actor::{self, DataReaderActor, DataReaderActorListener},
    domain_participant_actor::ParticipantListenerMessage,
    status_condition_actor::AddCommunicationState,
    topic_actor::TopicActor,
};
use crate::{
    dds_async::{
        data_reader::DataReaderAsync, domain_participant::DomainParticipantAsync,
        subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync,
        topic::TopicAsync,
    },
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        actors::status_condition_actor::StatusConditionActor,
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
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
        group::RtpsGroup,
        participant::RtpsParticipant,
        reader::{ReaderCacheChange, ReaderHistoryCache},
        types::{
            EntityId, Guid, Locator, USER_DEFINED_READER_NO_KEY, USER_DEFINED_READER_WITH_KEY,
        },
    },
    xtypes::dynamic_type::DynamicType,
};
use fnmatch_regex::glob_to_regex;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use tracing::warn;

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
    pub reader_address: ActorAddress<DataReaderActor>,
    pub status_condition_address: ActorAddress<StatusConditionActor>,
    pub subscriber: SubscriberAsync,
    pub topic: TopicAsync,
}

struct SubscriberListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<SubscriberListenerMessage>,
}

impl SubscriberListenerThread {
    fn new(mut listener: Box<dyn SubscriberListenerAsync + Send>) -> Self {
        // let (sender, receiver) = mpsc_channel::<SubscriberListenerMessage>();
        // let thread = std::thread::Builder::new()
        //     .name("Subscriber listener".to_string())
        //     .spawn(move || {
        //         block_on(async {
        //             while let Some(m) = receiver.recv().await {
        //                 let data_reader = DataReaderAsync::new(
        //                     m.reader_address,
        //                     m.status_condition_address,
        //                     m.subscriber,
        //                     m.topic,
        //                 );
        //                 match m.listener_operation {
        //                     SubscriberListenerOperation::DataOnReaders(the_subscriber) => {
        //                         listener.on_data_on_readers(the_subscriber).await
        //                     }
        //                     SubscriberListenerOperation::_DataAvailable => {
        //                         listener.on_data_available(data_reader).await
        //                     }
        //                     SubscriberListenerOperation::SampleRejected(status) => {
        //                         listener.on_sample_rejected(data_reader, status).await
        //                     }
        //                     SubscriberListenerOperation::_LivenessChanged(status) => {
        //                         listener.on_liveliness_changed(data_reader, status).await
        //                     }
        //                     SubscriberListenerOperation::RequestedDeadlineMissed(status) => {
        //                         listener
        //                             .on_requested_deadline_missed(data_reader, status)
        //                             .await
        //                     }
        //                     SubscriberListenerOperation::RequestedIncompatibleQos(status) => {
        //                         listener
        //                             .on_requested_incompatible_qos(data_reader, status)
        //                             .await
        //                     }
        //                     SubscriberListenerOperation::SubscriptionMatched(status) => {
        //                         listener.on_subscription_matched(data_reader, status).await
        //                     }
        //                     SubscriberListenerOperation::SampleLost(status) => {
        //                         listener.on_sample_lost(data_reader, status).await
        //                     }
        //                 }
        //             }
        //         });
        //     })
        //     .expect("failed to spawn thread");
        // Self { thread, sender }
        todo!()
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
    transport: Arc<Mutex<RtpsParticipant>>,
    data_reader_list: HashMap<InstanceHandle, DataReaderActor>,
    enabled: bool,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    status_condition: StatusConditionActor,
    subscriber_listener_thread: Option<SubscriberListenerThread>,
    subscriber_status_kind: Vec<StatusKind>,
}

impl SubscriberActor {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        transport: Arc<Mutex<RtpsParticipant>>,
        listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        subscriber_status_kind: Vec<StatusKind>,
        data_reader_list: Vec<DataReaderActor>,
    ) -> Self {
        let status_condition = StatusConditionActor::default();
        let subscriber_listener_thread = listener.map(SubscriberListenerThread::new);
        let data_reader_list = data_reader_list
            .into_iter()
            .map(|dr| (dr.get_instance_handle(), dr))
            .collect();

        SubscriberActor {
            qos,
            rtps_group,
            transport,
            data_reader_list,
            enabled: false,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
            status_condition,
            subscriber_listener_thread,
            subscriber_status_kind,
        }
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

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_empty(&self) -> bool {
        self.data_reader_list.is_empty()
    }

    pub fn create_datareader(
        &mut self,
        a_topic: &TopicActor,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<Guid> {
        struct UserDefinedReaderHistoryCache {
            pub subscriber_address: ActorAddress<SubscriberActor>,
            pub reader_instance_handle: InstanceHandle,
        }

        impl ReaderHistoryCache for UserDefinedReaderHistoryCache {
            fn add_change(&mut self, cache_change: ReaderCacheChange) {
                self.subscriber_address
                    .send_actor_mail(AddChange {
                        cache_change,
                        reader_instance_handle: self.reader_instance_handle,
                    })
                    .ok();
            }
        }

        let qos = match qos {
            QosKind::Default => self.default_data_reader_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let guid_prefix = self.rtps_group.guid().prefix();
        let topic_name = a_topic.get_name().to_string();
        let type_name = a_topic.get_type_name().to_string();
        let type_support = a_topic.get_type_support();
        let has_key = {
            let mut has_key = false;
            for index in 0..type_support.get_member_count() {
                if type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };

        let subscriber_guid = self.rtps_group.guid();
        let entity_kind = match has_key {
            true => USER_DEFINED_READER_WITH_KEY,
            false => USER_DEFINED_READER_NO_KEY,
        };
        let entity_key: [u8; 3] = [
            subscriber_guid.entity_id().entity_key()[0],
            self.get_unique_reader_id(),
            0,
        ];

        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = Guid::new(subscriber_guid.prefix(), entity_id);

        let user_defined_reader_history_cache = UserDefinedReaderHistoryCache {
            subscriber_address: todo!(),
            reader_instance_handle: InstanceHandle::new(guid.into()),
        };

        let rtps_reader = self
            .transport
            .lock()
            .unwrap()
            .create_reader(guid, Box::new(user_defined_reader_history_cache));

        let data_reader_status_kind = mask.to_vec();
        let data_reader = DataReaderActor::new(
            guid,
            rtps_reader,
            topic_name,
            type_name,
            type_support.clone(),
            qos,
            a_listener,
            data_reader_status_kind,
        );

        self.data_reader_list
            .insert(InstanceHandle::new(guid.into()), data_reader);

        Ok(guid)
    }
}

pub struct CreateDatareader {
    pub topic_name: String,
    pub type_name: String,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
    pub has_key: bool,
    pub qos: QosKind<DataReaderQos>,
    pub a_listener: Option<DataReaderActorListener>,
    pub mask: Vec<StatusKind>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub subscriber_address: ActorAddress<SubscriberActor>,
    pub executor_handle: ExecutorHandle,
    pub timer_handle: TimerHandle,
}
impl Mail for CreateDatareader {
    type Result = DdsResult<Guid>;
}
impl MailHandler<CreateDatareader> for SubscriberActor {
    fn handle(&mut self, message: CreateDatareader) -> <CreateDatareader as Mail>::Result {}
}

pub struct DeleteDatareader {
    pub handle: InstanceHandle,
}
impl Mail for DeleteDatareader {
    type Result = DdsResult<Actor<DataReaderActor>>;
}
impl MailHandler<DeleteDatareader> for SubscriberActor {
    fn handle(&mut self, message: DeleteDatareader) -> <DeleteDatareader as Mail>::Result {
        todo!()
        // if let Some(removed_reader) = self.data_reader_list.remove(&message.handle) {
        //     Ok(removed_reader)
        // } else {
        //     Err(DdsError::PreconditionNotMet(
        //         "Data reader can only be deleted from its parent subscriber".to_string(),
        //     ))
        // }
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

pub struct GetQos;
impl Mail for GetQos {
    type Result = SubscriberQos;
}
impl MailHandler<GetQos> for SubscriberActor {
    fn handle(&mut self, _: GetQos) -> <GetQos as Mail>::Result {
        self.qos.clone()
    }
}

pub struct GetStatusKind;
impl Mail for GetStatusKind {
    type Result = Vec<StatusKind>;
}
impl MailHandler<GetStatusKind> for SubscriberActor {
    fn handle(&mut self, _: GetStatusKind) -> <GetStatusKind as Mail>::Result {
        self.subscriber_status_kind.clone()
    }
}

pub struct AddMatchedWriter {
    pub discovered_writer_data: DiscoveredWriterData,
    pub subscriber_address: ActorAddress<SubscriberActor>,
    // pub participant: DomainParticipantAsync,
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
                .dds_publication_data
                .partition(),
        ) {
            for data_reader in self.data_reader_list.values() {
                let subscriber_mask_listener = (
                    self.subscriber_listener_thread
                        .as_ref()
                        .map(|l| l.sender().clone()),
                    self.subscriber_status_kind.clone(),
                );

                let subscriber_qos = self.qos.clone();
                // data_reader.send_actor_mail(data_reader_actor::AddMatchedWriter {
                //     discovered_writer_data: message.discovered_writer_data.clone(),
                //     // data_reader_address,
                //     // subscriber: SubscriberAsync::new(
                //     //     message.subscriber_address.clone(),
                //     //     self.status_condition.address(),
                //     //     message.participant.clone(),
                //     // ),
                //     subscriber_qos,
                //     // subscriber_mask_listener,
                //     // participant_mask_listener: message.participant_mask_listener.clone(),
                // });
            }
        }
        Ok(())
    }
}

pub struct RemoveMatchedWriter {
    pub discovered_writer_handle: InstanceHandle,
    // pub subscriber_address: ActorAddress<SubscriberActor>,
    // pub participant: DomainParticipantAsync,
    // pub participant_mask_listener: (
    //     Option<MpscSender<ParticipantListenerMessage>>,
    //     Vec<StatusKind>,
    // ),
}
impl Mail for RemoveMatchedWriter {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedWriter> for SubscriberActor {
    fn handle(&mut self, message: RemoveMatchedWriter) -> <RemoveMatchedWriter as Mail>::Result {
        for data_reader in self.data_reader_list.values() {
            // let data_reader_address = data_reader.address();
            let subscriber_mask_listener = (
                self.subscriber_listener_thread
                    .as_ref()
                    .map(|l| l.sender().clone()),
                self.subscriber_status_kind.clone(),
            );
            // data_reader.send_actor_mail(data_reader_actor::RemoveMatchedWriter {
            //     discovered_writer_handle: message.discovered_writer_handle,
            //     // data_reader_address,
            //     // subscriber: SubscriberAsync::new(
            //     //     message.subscriber_address.clone(),
            //     //     self.status_condition.address(),
            //     //     message.participant.clone(),
            //     // ),
            //     // subscriber_mask_listener,
            //     // participant_mask_listener: message.participant_mask_listener.clone(),
            // });
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
        self.subscriber_status_kind = message.status_kind;

        Ok(())
    }
}

pub struct AddChange {
    pub cache_change: ReaderCacheChange,
    pub reader_instance_handle: InstanceHandle,
}
impl Mail for AddChange {
    type Result = ();
}
impl MailHandler<AddChange> for SubscriberActor {
    fn handle(&mut self, message: AddChange) -> <AddChange as Mail>::Result {
        // if let Some(reader) = self.data_reader_list.get(&message.reader_instance_handle) {
        //     reader.send_actor_mail(data_reader_actor::AddChange {
        //         cache_change: message.cache_change,
        //     });
        // }

        self.status_condition
            .add_communication_state(StatusKind::DataOnReaders);
    }
}
