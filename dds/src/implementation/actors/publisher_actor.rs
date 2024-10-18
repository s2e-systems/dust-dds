use super::{
    any_data_writer_listener::AnyDataWriterListener,
    data_writer_actor::{self, DataWriterActor},
    domain_participant_actor::ParticipantListenerMessage,
    message_sender_actor::MessageSenderActor,
    status_condition_actor::StatusConditionActor,
    topic_actor::TopicActor,
};
use crate::{
    dds_async::{
        data_writer::DataWriterAsync, publisher::PublisherAsync,
        publisher_listener::PublisherListenerAsync, topic::TopicAsync,
    },
    implementation::{
        actor::{Actor, ActorAddress, Mail, MailHandler},
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
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
        group::RtpsGroup,
        participant::RtpsParticipant,
        types::{
            EntityId, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
    },
};
use fnmatch_regex::glob_to_regex;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use tracing::warn;

pub enum PublisherListenerOperation {
    _LivelinessLost(LivelinessLostStatus),
    OfferedDeadlineMissed(OfferedDeadlineMissedStatus),
    OfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    PublicationMatched(PublicationMatchedStatus),
}

pub struct PublisherListenerMessage {
    pub listener_operation: PublisherListenerOperation,
    pub writer_address: ActorAddress<DataWriterActor>,
    pub status_condition_address: ActorAddress<StatusConditionActor>,
    pub publisher: PublisherAsync,
    pub topic: TopicAsync,
}

struct PublisherListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<PublisherListenerMessage>,
}

impl PublisherListenerThread {
    fn new(mut listener: Box<dyn PublisherListenerAsync + Send>) -> Self {
        todo!()
        // let (sender, receiver) = mpsc_channel::<PublisherListenerMessage>();
        // let thread = std::thread::Builder::new()
        //     .name("Publisher listener".to_string())
        //     .spawn(move || {
        //         block_on(async {
        //             while let Some(m) = receiver.recv().await {
        //                 let data_writer = DataWriterAsync::new(
        //                     m.writer_address,
        //                     m.status_condition_address,
        //                     m.publisher,
        //                     m.topic,
        //                 );
        //                 match m.listener_operation {
        //                     PublisherListenerOperation::_LivelinessLost(status) => {
        //                         listener.on_liveliness_lost(data_writer, status).await
        //                     }
        //                     PublisherListenerOperation::OfferedDeadlineMissed(status) => {
        //                         listener
        //                             .on_offered_deadline_missed(data_writer, status)
        //                             .await
        //                     }
        //                     PublisherListenerOperation::OfferedIncompatibleQos(status) => {
        //                         listener
        //                             .on_offered_incompatible_qos(data_writer, status)
        //                             .await
        //                     }
        //                     PublisherListenerOperation::PublicationMatched(status) => {
        //                         listener.on_publication_matched(data_writer, status).await
        //                     }
        //                 }
        //             }
        //         });
        //     })
        //     .expect("failed to spawn thread");
        // Self { thread, sender }
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
    transport: Arc<Mutex<RtpsParticipant>>,
    data_writer_list: HashMap<InstanceHandle, DataWriterActor>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
    publisher_listener_thread: Option<PublisherListenerThread>,
    status_kind: Vec<StatusKind>,
    status_condition: StatusConditionActor,
}

impl PublisherActor {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        transport: Arc<Mutex<RtpsParticipant>>,
        listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        data_writer_list: Vec<DataWriterActor>,
    ) -> Self {
        let data_writer_list = data_writer_list
            .into_iter()
            .map(|dw| (dw.get_instance_handle(), dw))
            .collect();
        let publisher_listener_thread = listener.map(PublisherListenerThread::new);
        Self {
            qos,
            transport,
            rtps_group,
            data_writer_list,
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
            publisher_listener_thread,
            status_kind,
            status_condition: StatusConditionActor::default(),
        }
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

    pub fn create_datawriter(
        &mut self,
        a_topic: &TopicActor,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        mask: Vec<StatusKind>,
    ) -> DdsResult<Guid> {
        let qos = match qos {
            QosKind::Default => self.default_datawriter_qos.clone(),
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
        let entity_kind = match has_key {
            true => USER_DEFINED_WRITER_WITH_KEY,
            false => USER_DEFINED_WRITER_NO_KEY,
        };
        let entity_key = [
            self.rtps_group.guid().entity_id().entity_key()[0],
            self.get_unique_writer_id(),
            0,
        ];
        let entity_id = EntityId::new(entity_key, entity_kind);
        let guid = Guid::new(guid_prefix, entity_id);

        let rtps_writer_impl = self.transport.lock().unwrap().create_writer(guid);

        let data_writer = DataWriterActor::new(
            rtps_writer_impl,
            guid,
            topic_name,
            type_name,
            a_listener,
            mask,
            qos,
        );

        self.data_writer_list
            .insert(InstanceHandle::new(guid.into()), data_writer);

        Ok(guid)
    }

    pub fn get_datawriter_by_guid(&self, datawriter_guid: Guid) -> &DataWriterActor {
        self.data_writer_list
            .get(&InstanceHandle::new(datawriter_guid.into()))
            .expect("Must exist")
    }

    pub fn get_mut_datawriter_by_guid(&mut self, datawriter_guid: Guid) -> &mut DataWriterActor {
        self.data_writer_list
            .get_mut(&InstanceHandle::new(datawriter_guid.into()))
            .expect("Must exist")
    }

    pub fn lookup_datawriter_by_topic_name(
        &mut self,
        topic_name: &str,
    ) -> Option<&mut DataWriterActor> {
        self.data_writer_list
            .values_mut()
            .find(|dw| dw.get_topic_name() == topic_name)
    }

    pub fn enable(&mut self) -> DdsResult<()> {
        if !self.enabled {
            self.enabled = true;
        }
        Ok(())
    }

    pub fn get_qos(&self) -> &PublisherQos {
        &self.qos
    }

    pub fn add_matched_reader(
        &mut self,
        discovered_reader_data: DiscoveredReaderData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) {
        if let Some(dw) = self.data_writer_list.values_mut().find(|dw| {
            dw.get_topic_name()
                == discovered_reader_data
                    .subscription_builtin_topic_data()
                    .topic_name()
        }) {
            dw.add_matched_reader(
                discovered_reader_data,
                default_unicast_locator_list,
                default_multicast_locator_list,
                &self.qos,
            );
        }
    }

    pub fn get_guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    pub fn is_empty(&self) -> bool {
        self.data_writer_list.is_empty()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_default_data_writer_qos(&mut self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        self.default_datawriter_qos = qos;
        Ok(())
    }

    pub fn set_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
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

pub struct GetDefaultDatawriterQos;
impl Mail for GetDefaultDatawriterQos {
    type Result = DataWriterQos;
}
impl MailHandler<GetDefaultDatawriterQos> for PublisherActor {
    fn handle(&mut self, _: GetDefaultDatawriterQos) -> <GetDefaultDatawriterQos as Mail>::Result {
        self.default_datawriter_qos.clone()
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

pub struct AddMatchedReader {
    pub discovered_reader_data: DiscoveredReaderData,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub publisher_address: ActorAddress<PublisherActor>,
    // pub participant: DomainParticipantAsync,
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
                todo!()
                // let data_writer_address = data_writer.address();
                // let publisher_mask_listener = (
                //     self.publisher_listener_thread
                //         .as_ref()
                //         .map(|l| l.sender().clone()),
                //     self.status_kind.clone(),
                // );

                // data_writer.send_actor_mail(data_writer_actor::AddMatchedReader {
                //     discovered_reader_data: message.discovered_reader_data.clone(),
                //     default_unicast_locator_list: message.default_unicast_locator_list.clone(),
                //     default_multicast_locator_list: message.default_multicast_locator_list.clone(),
                //     data_writer_address,
                //     // publisher: PublisherAsync::new(
                //     //     message.publisher_address.clone(),
                //     //     self.status_condition.address(),
                //     //     message.participant.clone(),
                //     // ),
                //     publisher_qos: self.qos.clone(),
                //     publisher_mask_listener,
                //     participant_mask_listener: message.participant_mask_listener.clone(),
                //     message_sender_actor: message.message_sender_actor.clone(),
                // });
            }
        }
        Ok(())
    }
}

pub struct RemoveMatchedReader {
    pub discovered_reader_handle: InstanceHandle,
    // pub publisher_address: ActorAddress<PublisherActor>,
    // pub participant: DomainParticipantAsync,
    // pub participant_mask_listener: (
    //     Option<MpscSender<ParticipantListenerMessage>>,
    //     Vec<StatusKind>,
    // ),
}
impl Mail for RemoveMatchedReader {
    type Result = DdsResult<()>;
}
impl MailHandler<RemoveMatchedReader> for PublisherActor {
    fn handle(&mut self, message: RemoveMatchedReader) -> <RemoveMatchedReader as Mail>::Result {
        for data_writer in self.data_writer_list.values() {
            todo!()
            // let data_writer_address = data_writer.address();
            // let publisher_mask_listener = (
            //     self.publisher_listener_thread
            //         .as_ref()
            //         .map(|l| l.sender().clone()),
            //     self.status_kind.clone(),
            // );
            // data_writer.send_actor_mail(data_writer_actor::RemoveMatchedReader {
            //     discovered_reader_handle: message.discovered_reader_handle,
            //     // data_writer_address,
            //     // publisher: PublisherAsync::new(
            //     //     message.publisher_address.clone(),
            //     //     self.status_condition.address(),
            //     //     message.participant.clone(),
            //     // ),
            //     // publisher_mask_listener,
            //     // participant_mask_listener: message.participant_mask_listener.clone(),
            // });
        }
        Ok(())
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

pub struct GetListener;
impl Mail for GetListener {
    type Result = (
        Option<MpscSender<PublisherListenerMessage>>,
        Vec<StatusKind>,
    );
}
impl MailHandler<GetListener> for PublisherActor {
    fn handle(&mut self, _: GetListener) -> <GetListener as Mail>::Result {
        (
            self.publisher_listener_thread
                .as_ref()
                .map(|l| l.sender().clone()),
            self.status_kind.clone(),
        )
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
