use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    sync::Arc,
};

use dust_dds_derive::actor_interface;
use tracing::debug;

use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, ReaderProxy},
            discovered_writer_data::DiscoveredWriterData,
        },
        data_representation_inline_qos::{
            parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
            types::{
                StatusInfo, STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED,
                STATUS_INFO_UNREGISTERED,
            },
        },
        payload_serializer_deserializer::{
            cdr_deserializer::ClassicCdrDeserializer, endianness::CdrEndianness,
        },
        rtps::{
            message_receiver::MessageReceiver,
            messages::{
                overall_structure::{RtpsMessageHeader, RtpsMessageRead, RtpsSubmessageReadKind},
                submessage_elements::{Data, Parameter, ParameterList},
                submessages::{
                    data::DataSubmessageRead, data_frag::DataFragSubmessageRead,
                    gap::GapSubmessageRead, heartbeat::HeartbeatSubmessageRead,
                    heartbeat_frag::HeartbeatFragSubmessageRead,
                },
            },
            reader::RtpsReader,
            reader_history_cache::{InstanceState, RtpsReaderCacheChange},
            types::{
                ChangeKind, Guid, GuidPrefix, Locator, SequenceNumber, ENTITYID_UNKNOWN,
                GUID_UNKNOWN,
            },
            writer_proxy::RtpsWriterProxy,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::{
            actor::{Actor, ActorAddress},
            instance_handle_from_key::get_instance_handle_from_key,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::{
            DestinationOrderQosPolicyKind, DurabilityQosPolicyKind, HistoryQosPolicyKind,
            QosPolicyId, ReliabilityQosPolicyKind, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID,
        },
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::{DurationKind, Time},
    },
    serialized_payload::cdr::deserialize::CdrDeserialize,
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    topic_definition::type_support::{DdsKey, DynamicTypeInterface},
};

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    data_reader_listener_actor::{self, DataReaderListenerActor},
    domain_participant_actor::{DomainParticipantActor, ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER},
    domain_participant_listener_actor::{self, DomainParticipantListenerActor},
    status_condition_actor::{self, StatusConditionActor},
    subscriber_actor::SubscriberActor,
    subscriber_listener_actor::{self, SubscriberListenerActor},
    type_support_actor::{self, TypeSupportActor},
};

fn build_instance_handle(
    type_support: &Arc<dyn DynamicTypeInterface + Send + Sync>,
    change_kind: ChangeKind,
    data: &[u8],
    inline_qos: &[Parameter],
) -> DdsResult<InstanceHandle> {
    Ok(match change_kind {
        ChangeKind::Alive | ChangeKind::AliveFiltered => {
            type_support.instance_handle_from_serialized_foo(data)?
        }
        ChangeKind::NotAliveDisposed
        | ChangeKind::NotAliveUnregistered
        | ChangeKind::NotAliveDisposedUnregistered => match inline_qos
            .iter()
            .find(|&x| x.parameter_id() == PID_KEY_HASH)
        {
            Some(p) => {
                if let Ok(key) = <[u8; 16]>::try_from(p.value()) {
                    InstanceHandle::new(key)
                } else {
                    type_support.instance_handle_from_serialized_key(data)?
                }
            }
            None => type_support.instance_handle_from_serialized_key(data)?,
        },
    })
}

impl SampleLostStatus {
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

impl SampleRejectedStatus {
    fn increment(
        &mut self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
    ) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_instance_handle = instance_handle;
        self.last_reason = rejected_reason;
    }

    fn read_and_reset(&mut self) -> Self {
        let status = self.clone();

        self.total_count_change = 0;

        status
    }
}

#[derive(Default)]
struct ReaderRequestedDeadlineMissedStatus {
    total_count: i32,
    total_count_change: i32,
    last_instance_handle: InstanceHandle,
}

#[actor_interface]
impl ReaderRequestedDeadlineMissedStatus {
    async fn increment_requested_deadline_missed_status(
        &mut self,
        instance_handle: InstanceHandle,
    ) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_instance_handle = instance_handle;
    }

    async fn read_requested_deadline_missed_status(&mut self) -> RequestedDeadlineMissedStatus {
        let status = RequestedDeadlineMissedStatus {
            total_count: self.total_count,
            total_count_change: self.total_count_change,
            last_instance_handle: self.last_instance_handle,
        };

        self.total_count_change = 0;

        status
    }
}

impl LivelinessChangedStatus {
    fn _read_and_reset(&mut self) -> Self {
        let status = self.clone();

        self.alive_count_change = 0;
        self.not_alive_count_change = 0;

        status
    }
}

impl RequestedIncompatibleQosStatus {
    fn increment(&mut self, incompatible_qos_policy_list: Vec<QosPolicyId>) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_policy_id = incompatible_qos_policy_list[0];
        for incompatible_qos_policy in incompatible_qos_policy_list.into_iter() {
            if let Some(policy_count) = self
                .policies
                .iter_mut()
                .find(|x| x.policy_id == incompatible_qos_policy)
            {
                policy_count.count += 1;
            } else {
                self.policies.push(QosPolicyCount {
                    policy_id: incompatible_qos_policy,
                    count: 1,
                })
            }
        }
    }

    fn read_and_reset(&mut self) -> Self {
        let status = self.clone();
        self.total_count_change = 0;
        status
    }
}

impl SubscriptionMatchedStatus {
    fn increment(&mut self, instance_handle: InstanceHandle) {
        self.total_count += 1;
        self.total_count_change += 1;
        self.last_publication_handle = instance_handle;
        self.current_count += 1;
        self.current_count_change += 1;
    }

    fn read_and_reset(&mut self, current_count: i32) -> Self {
        let last_current_count = self.current_count;
        self.current_count = current_count;
        self.current_count_change = current_count - last_current_count;
        let status = self.clone();

        self.total_count_change = 0;

        status
    }
}

struct IndexedSample {
    index: usize,
    sample: (Option<Data>, SampleInfo),
}

pub struct DataReaderActor {
    rtps_reader: RtpsReader,
    matched_writers: Vec<RtpsWriterProxy>,
    changes: Vec<RtpsReaderCacheChange>,
    qos: DataReaderQos,
    type_name: String,
    topic_name: String,
    _liveliness_changed_status: LivelinessChangedStatus,
    requested_deadline_missed_status: Actor<ReaderRequestedDeadlineMissedStatus>,
    requested_incompatible_qos_status: RequestedIncompatibleQosStatus,
    sample_lost_status: SampleLostStatus,
    sample_rejected_status: SampleRejectedStatus,
    subscription_matched_status: SubscriptionMatchedStatus,
    matched_publication_list: HashMap<InstanceHandle, PublicationBuiltinTopicData>,
    enabled: bool,
    data_available_status_changed_flag: bool,
    incompatible_writer_list: HashSet<InstanceHandle>,
    status_condition: Actor<StatusConditionActor>,
    listener: Actor<DataReaderListenerActor>,
    status_kind: Vec<StatusKind>,
    instances: HashMap<InstanceHandle, InstanceState>,
    instance_deadline_missed_task: HashMap<InstanceHandle, tokio::task::AbortHandle>,
}

impl DataReaderActor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        rtps_reader: RtpsReader,
        type_name: String,
        topic_name: String,
        qos: DataReaderQos,
        listener: Box<dyn AnyDataReaderListener + Send>,
        status_kind: Vec<StatusKind>,
        handle: &tokio::runtime::Handle,
    ) -> Self {
        let status_condition = Actor::spawn(StatusConditionActor::default(), handle);
        let listener = Actor::spawn(DataReaderListenerActor::new(listener), handle);

        DataReaderActor {
            rtps_reader,
            matched_writers: Vec::new(),
            changes: Vec::new(),
            type_name,
            topic_name,
            _liveliness_changed_status: LivelinessChangedStatus::default(),
            requested_deadline_missed_status: Actor::spawn(
                ReaderRequestedDeadlineMissedStatus::default(),
                handle,
            ),
            requested_incompatible_qos_status: RequestedIncompatibleQosStatus::default(),
            sample_lost_status: SampleLostStatus::default(),
            sample_rejected_status: SampleRejectedStatus::default(),
            subscription_matched_status: SubscriptionMatchedStatus::default(),
            matched_publication_list: HashMap::new(),
            enabled: false,
            data_available_status_changed_flag: false,
            incompatible_writer_list: HashSet::new(),
            status_condition,
            status_kind,
            listener,
            qos,
            instances: HashMap::new(),
            instance_deadline_missed_task: HashMap::new(),
        }
    }

    fn get_requested_incompatible_qos_status(&mut self) -> RequestedIncompatibleQosStatus {
        self.requested_incompatible_qos_status.read_and_reset()
    }

    fn get_sample_lost_status(&mut self) -> SampleLostStatus {
        self.sample_lost_status.read_and_reset()
    }

    fn get_sample_rejected_status(&mut self) -> SampleRejectedStatus {
        self.sample_rejected_status.read_and_reset()
    }

    async fn on_data_available(
        &self,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber_address: &ActorAddress<SubscriberActor>,
        participant_address: &ActorAddress<DomainParticipantActor>,
        subscriber_status_condition: &ActorAddress<StatusConditionActor>,
        (subscriber_listener_address, subscriber_listener_mask): &(
            ActorAddress<SubscriberListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: &tokio::runtime::Handle,
    ) -> DdsResult<()> {
        subscriber_status_condition
            .send_mail_and_await_reply(status_condition_actor::add_communication_state::new(
                StatusKind::DataOnReaders,
            ))
            .await?;
        self.status_condition
            .address()
            .send_mail_and_await_reply(status_condition_actor::add_communication_state::new(
                StatusKind::DataAvailable,
            ))
            .await?;
        if subscriber_listener_mask.contains(&StatusKind::DataOnReaders) {
            subscriber_listener_address
                .send_mail(subscriber_listener_actor::trigger_on_data_on_readers::new(
                    subscriber_address.clone(),
                    participant_address.clone(),
                    runtime_handle.clone(),
                ))
                .await?;
        } else if self.status_kind.contains(&StatusKind::DataAvailable) {
            self.listener
                .send_mail(data_reader_listener_actor::trigger_on_data_available::new(
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                    runtime_handle.clone(),
                ))
                .await;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn on_data_submessage_received(
        &mut self,
        data_submessage: &DataSubmessageRead,
        type_support: &Arc<dyn DynamicTypeInterface + Send + Sync>,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<Time>,
        reception_timestamp: Time,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber_address: &ActorAddress<SubscriberActor>,
        participant_address: &ActorAddress<DomainParticipantActor>,
        subscriber_status_condition: &ActorAddress<StatusConditionActor>,
        subscriber_mask_listener: &(ActorAddress<SubscriberListenerActor>, Vec<StatusKind>),
        participant_mask_listener: &(
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: &tokio::runtime::Handle,
    ) -> DdsResult<()> {
        let writer_guid = Guid::new(source_guid_prefix, data_submessage.writer_id());
        let sequence_number = data_submessage.writer_sn();
        let message_reader_id = data_submessage.reader_id();
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_guid)
        {
            //Stateful reader behavior
            match self.qos.reliability.kind {
                ReliabilityQosPolicyKind::BestEffort => {
                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                    if sequence_number >= expected_seq_num {
                        writer_proxy.received_change_set(sequence_number);
                        if sequence_number > expected_seq_num {
                            writer_proxy.lost_changes_update(sequence_number);
                            self.on_sample_lost(
                                data_reader_address,
                                subscriber_address,
                                participant_address,
                                subscriber_mask_listener,
                                participant_mask_listener,
                                runtime_handle,
                            )
                            .await?;
                        }
                        match self.convert_received_data_to_cache_change(
                            writer_guid,
                            data_submessage.key_flag(),
                            data_submessage.inline_qos(),
                            data_submessage.serialized_payload(),
                            source_timestamp,
                            reception_timestamp,
                            type_support,
                        ) {
                            Ok(change) => {
                                self.add_change(
                                    change,
                                    data_reader_address,
                                    subscriber_address,
                                    participant_address,
                                    subscriber_status_condition,
                                    subscriber_mask_listener,
                                    participant_mask_listener,
                                    runtime_handle,
                                )
                                .await?;
                            }
                            Err(e) => debug!(
                                "Received invalid data on reader with GUID {guid:?}. Error: {err:?}.
                                 Message writer ID: {writer_id:?}
                                 Message reader ID: {reader_id:?}
                                 Data submessage payload: {payload:?}",
                                guid = self.rtps_reader.guid(),
                                err = e,
                                writer_id = data_submessage.writer_id(),
                                reader_id = data_submessage.reader_id(),
                                payload = data_submessage.serialized_payload(),
                            ),
                        }
                    }
                }
                ReliabilityQosPolicyKind::Reliable => {
                    let expected_seq_num = writer_proxy.available_changes_max() + 1;
                    if sequence_number == expected_seq_num {
                        writer_proxy.received_change_set(sequence_number);
                        match self.convert_received_data_to_cache_change(
                            writer_guid,
                            data_submessage.key_flag(),
                            data_submessage.inline_qos(),
                            data_submessage.serialized_payload(),
                            source_timestamp,
                            reception_timestamp,
                            type_support,
                        ) {
                            Ok(change) => {
                                self.add_change(
                                    change,
                                    data_reader_address,
                                    subscriber_address,
                                    participant_address,
                                    subscriber_status_condition,
                                    subscriber_mask_listener,
                                    participant_mask_listener,
                                    runtime_handle,
                                )
                                .await?;
                            }
                            Err(e) => debug!(
                                "Received invalid data on reader with GUID {guid:?}. Error: {err:?}.
                                 Message writer ID: {writer_id:?}
                                 Message reader ID: {reader_id:?}
                                 Data submessage payload: {payload:?}",
                                guid = self.rtps_reader.guid(),
                                err = e,
                                writer_id = data_submessage.writer_id(),
                                reader_id = data_submessage.reader_id(),
                                payload = data_submessage.serialized_payload(),
                            ),
                        }
                    }
                }
            }
        } else if message_reader_id == ENTITYID_UNKNOWN
            || (message_reader_id == self.rtps_reader.guid().entity_id()
                && message_reader_id == ENTITYID_SPDP_BUILTIN_PARTICIPANT_READER)
        // Additional condition only for discovery interoperability with FastDDS
        {
            // Stateless reader behavior. We add the change if the data is correct. No error is printed
            // because all readers would get changes marked with ENTITYID_UNKNOWN
            if let Ok(change) = self.convert_received_data_to_cache_change(
                writer_guid,
                data_submessage.key_flag(),
                data_submessage.inline_qos(),
                data_submessage.serialized_payload(),
                source_timestamp,
                reception_timestamp,
                type_support,
            ) {
                self.add_change(
                    change,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                    subscriber_status_condition,
                    subscriber_mask_listener,
                    participant_mask_listener,
                    runtime_handle,
                )
                .await?;
            }
        } else {
            // Do nothing
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn on_data_frag_submessage_received(
        &mut self,
        data_frag_submessage: &DataFragSubmessageRead,
        type_support: &Arc<dyn DynamicTypeInterface + Send + Sync>,
        source_guid_prefix: GuidPrefix,
        source_timestamp: Option<Time>,
        reception_timestamp: Time,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber_address: &ActorAddress<SubscriberActor>,
        participant_address: &ActorAddress<DomainParticipantActor>,
        subscriber_status_condition: &ActorAddress<StatusConditionActor>,
        subscriber_mask_listener: &(ActorAddress<SubscriberListenerActor>, Vec<StatusKind>),
        participant_mask_listener: &(
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: &tokio::runtime::Handle,
    ) -> DdsResult<()> {
        let sequence_number = data_frag_submessage.writer_sn();
        let writer_guid = Guid::new(source_guid_prefix, data_frag_submessage.writer_id());

        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|wp| wp.remote_writer_guid() == writer_guid)
        {
            writer_proxy.push_data_frag(data_frag_submessage.clone());
            if let Some(data_submessage) = writer_proxy.reconstruct_data_from_frag(sequence_number)
            {
                self.on_data_submessage_received(
                    &data_submessage,
                    type_support,
                    source_guid_prefix,
                    source_timestamp,
                    reception_timestamp,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                    subscriber_status_condition,
                    subscriber_mask_listener,
                    participant_mask_listener,
                    runtime_handle,
                )
                .await?;
            }
        }
        Ok(())
    }

    fn on_heartbeat_submessage_received(
        &mut self,
        heartbeat_submessage: &HeartbeatSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_submessage.writer_id());

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count() < heartbeat_submessage.count() {
                    writer_proxy.set_last_received_heartbeat_count(heartbeat_submessage.count());

                    writer_proxy.set_must_send_acknacks(
                        !heartbeat_submessage.final_flag()
                            || (!heartbeat_submessage.liveliness_flag()
                                && !writer_proxy.missing_changes().count() == 0),
                    );

                    if !heartbeat_submessage.final_flag() {
                        writer_proxy.set_must_send_acknacks(true);
                    }
                    writer_proxy.missing_changes_update(heartbeat_submessage.last_sn());
                    writer_proxy.lost_changes_update(heartbeat_submessage.first_sn());
                }
            }
        }
    }

    fn on_heartbeat_frag_submessage_received(
        &mut self,
        heartbeat_frag_submessage: &HeartbeatFragSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        if self.qos.reliability.kind == ReliabilityQosPolicyKind::Reliable {
            let writer_guid = Guid::new(source_guid_prefix, heartbeat_frag_submessage.writer_id());

            if let Some(writer_proxy) = self
                .matched_writers
                .iter_mut()
                .find(|x| x.remote_writer_guid() == writer_guid)
            {
                if writer_proxy.last_received_heartbeat_count() < heartbeat_frag_submessage.count()
                {
                    writer_proxy
                        .set_last_received_heartbeat_frag_count(heartbeat_frag_submessage.count());
                }

                // todo!()
            }
        }
    }

    fn get_discovered_writer_incompatible_qos_policy_list(
        &self,
        discovered_writer_data: &DiscoveredWriterData,
        subscriber_qos: &SubscriberQos,
    ) -> Vec<QosPolicyId> {
        let writer_info = discovered_writer_data.dds_publication_data();

        let mut incompatible_qos_policy_list = Vec::new();

        if subscriber_qos.presentation.access_scope > writer_info.presentation().access_scope
            || subscriber_qos.presentation.coherent_access
                != writer_info.presentation().coherent_access
            || subscriber_qos.presentation.ordered_access
                != writer_info.presentation().ordered_access
        {
            incompatible_qos_policy_list.push(PRESENTATION_QOS_POLICY_ID);
        }
        if &self.qos.durability > writer_info.durability() {
            incompatible_qos_policy_list.push(DURABILITY_QOS_POLICY_ID);
        }
        if &self.qos.deadline > writer_info.deadline() {
            incompatible_qos_policy_list.push(DEADLINE_QOS_POLICY_ID);
        }
        if &self.qos.latency_budget > writer_info.latency_budget() {
            incompatible_qos_policy_list.push(LATENCYBUDGET_QOS_POLICY_ID);
        }
        if &self.qos.liveliness > writer_info.liveliness() {
            incompatible_qos_policy_list.push(LIVELINESS_QOS_POLICY_ID);
        }
        if self.qos.reliability.kind > writer_info.reliability().kind {
            incompatible_qos_policy_list.push(RELIABILITY_QOS_POLICY_ID);
        }
        if &self.qos.destination_order > writer_info.destination_order() {
            incompatible_qos_policy_list.push(DESTINATIONORDER_QOS_POLICY_ID);
        }

        incompatible_qos_policy_list
    }

    fn on_gap_submessage_received(
        &mut self,
        gap_submessage: &GapSubmessageRead,
        source_guid_prefix: GuidPrefix,
    ) {
        let writer_guid = Guid::new(source_guid_prefix, gap_submessage.writer_id());
        if let Some(writer_proxy) = self
            .matched_writers
            .iter_mut()
            .find(|x| x.remote_writer_guid() == writer_guid)
        {
            for seq_num in
                i64::from(gap_submessage.gap_start())..i64::from(gap_submessage.gap_list().base())
            {
                writer_proxy.irrelevant_change_set(SequenceNumber::from(seq_num))
            }

            for seq_num in gap_submessage.gap_list().set() {
                writer_proxy.irrelevant_change_set(seq_num)
            }
        }
    }

    async fn on_sample_lost(
        &mut self,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber_address: &ActorAddress<SubscriberActor>,
        participant_address: &ActorAddress<DomainParticipantActor>,
        (subscriber_listener_address, subscriber_listener_mask): &(
            ActorAddress<SubscriberListenerActor>,
            Vec<StatusKind>,
        ),
        (participant_listener_address, participant_listener_mask): &(
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: &tokio::runtime::Handle,
    ) -> DdsResult<()> {
        self.sample_lost_status.increment();
        self.status_condition
            .address()
            .send_mail_and_await_reply(status_condition_actor::add_communication_state::new(
                StatusKind::SampleLost,
            ))
            .await?;
        if self.status_kind.contains(&StatusKind::SampleLost) {
            let status = self.get_sample_lost_status();
            self.listener
                .send_mail(data_reader_listener_actor::trigger_on_sample_lost::new(
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                    runtime_handle.clone(),
                    status,
                ))
                .await;
        } else if subscriber_listener_mask.contains(&StatusKind::SampleLost) {
            let status = self.get_sample_lost_status();
            subscriber_listener_address
                .send_mail(subscriber_listener_actor::trigger_on_sample_lost::new(
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                    runtime_handle.clone(),
                    status,
                ))
                .await?;
        } else if participant_listener_mask.contains(&StatusKind::SampleLost) {
            let status = self.get_sample_lost_status();
            participant_listener_address
                .send_mail(
                    domain_participant_listener_actor::trigger_on_sample_lost::new(
                        data_reader_address.clone(),
                        subscriber_address.clone(),
                        participant_address.clone(),
                        runtime_handle.clone(),
                        status,
                    ),
                )
                .await?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn on_subscription_matched(
        &mut self,
        instance_handle: InstanceHandle,
        data_reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        (subscriber_listener_address, subscriber_listener_mask): &(
            ActorAddress<SubscriberListenerActor>,
            Vec<StatusKind>,
        ),
        (participant_listener_address, participant_listener_mask): &(
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: &tokio::runtime::Handle,
    ) {
        self.subscription_matched_status.increment(instance_handle);
        self.status_condition
            .send_mail_and_await_reply(status_condition_actor::add_communication_state::new(
                StatusKind::SubscriptionMatched,
            ))
            .await;
        const SUBSCRIPTION_MATCHED_STATUS_KIND: &StatusKind = &StatusKind::SubscriptionMatched;
        if self.status_kind.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
            let status = self.get_subscription_matched_status().await;
            self.listener
                .send_mail(
                    data_reader_listener_actor::trigger_on_subscription_matched::new(
                        data_reader_address,
                        subscriber_address,
                        participant_address,
                        runtime_handle.clone(),
                        status,
                    ),
                )
                .await;
        } else if subscriber_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
            let status = self.get_subscription_matched_status().await;
            subscriber_listener_address
                .send_mail(
                    subscriber_listener_actor::trigger_on_subscription_matched::new(
                        data_reader_address.clone(),
                        subscriber_address.clone(),
                        participant_address.clone(),
                        runtime_handle.clone(),
                        status,
                    ),
                )
                .await
                .expect("Subscriber is guaranteed to exist");
        } else if participant_listener_mask.contains(SUBSCRIPTION_MATCHED_STATUS_KIND) {
            let status = self.get_subscription_matched_status().await;
            participant_listener_address
                .send_mail(
                    domain_participant_listener_actor::trigger_on_subscription_matched::new(
                        data_reader_address.clone(),
                        subscriber_address.clone(),
                        participant_address.clone(),
                        runtime_handle.clone(),
                        status,
                    ),
                )
                .await
                .expect("Participant is guaranteed to exist");
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn on_sample_rejected(
        &mut self,
        instance_handle: InstanceHandle,
        rejected_reason: SampleRejectedStatusKind,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber_address: &ActorAddress<SubscriberActor>,
        participant_address: &ActorAddress<DomainParticipantActor>,
        (subscriber_listener_address, subscriber_listener_mask): &(
            ActorAddress<SubscriberListenerActor>,
            Vec<StatusKind>,
        ),
        (participant_listener_address, participant_listener_mask): &(
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: &tokio::runtime::Handle,
    ) -> DdsResult<()> {
        self.sample_rejected_status
            .increment(instance_handle, rejected_reason);
        self.status_condition
            .address()
            .send_mail_and_await_reply(status_condition_actor::add_communication_state::new(
                StatusKind::SampleRejected,
            ))
            .await?;
        if self.status_kind.contains(&StatusKind::SampleRejected) {
            let status = self.get_sample_rejected_status();

            self.listener
                .send_mail(data_reader_listener_actor::trigger_on_sample_rejected::new(
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                    runtime_handle.clone(),
                    status,
                ))
                .await;
        } else if subscriber_listener_mask.contains(&StatusKind::SampleRejected) {
            let status = self.get_sample_rejected_status();

            subscriber_listener_address
                .send_mail(subscriber_listener_actor::trigger_on_sample_rejected::new(
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                    runtime_handle.clone(),
                    status,
                ))
                .await?;
        } else if participant_listener_mask.contains(&StatusKind::SampleRejected) {
            let status = self.get_sample_rejected_status();
            participant_listener_address
                .send_mail(
                    domain_participant_listener_actor::trigger_on_sample_rejected::new(
                        data_reader_address.clone(),
                        subscriber_address.clone(),
                        participant_address.clone(),
                        runtime_handle.clone(),
                        status,
                    ),
                )
                .await?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn on_requested_incompatible_qos(
        &mut self,
        incompatible_qos_policy_list: Vec<QosPolicyId>,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber_address: &ActorAddress<SubscriberActor>,
        participant_address: &ActorAddress<DomainParticipantActor>,
        (subscriber_listener_address, subscriber_listener_mask): &(
            ActorAddress<SubscriberListenerActor>,
            Vec<StatusKind>,
        ),
        (participant_listener_address, participant_listener_mask): &(
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: &tokio::runtime::Handle,
    ) {
        self.requested_incompatible_qos_status
            .increment(incompatible_qos_policy_list);
        self.status_condition
            .send_mail_and_await_reply(status_condition_actor::add_communication_state::new(
                StatusKind::RequestedIncompatibleQos,
            ))
            .await;

        if self
            .status_kind
            .contains(&StatusKind::RequestedIncompatibleQos)
        {
            let status = self.get_requested_incompatible_qos_status();

            self.listener
                .send_mail(
                    data_reader_listener_actor::trigger_on_requested_incompatible_qos::new(
                        data_reader_address.clone(),
                        subscriber_address.clone(),
                        participant_address.clone(),
                        runtime_handle.clone(),
                        status,
                    ),
                )
                .await;
        } else if subscriber_listener_mask.contains(&StatusKind::RequestedIncompatibleQos) {
            let status = self.get_requested_incompatible_qos_status();
            subscriber_listener_address
                .send_mail(
                    subscriber_listener_actor::trigger_on_requested_incompatible_qos::new(
                        data_reader_address.clone(),
                        subscriber_address.clone(),
                        participant_address.clone(),
                        runtime_handle.clone(),
                        status,
                    ),
                )
                .await
                .expect("Subscriber listener should exist");
        } else if participant_listener_mask.contains(&StatusKind::RequestedIncompatibleQos) {
            let status = self.get_requested_incompatible_qos_status();
            participant_listener_address
                .send_mail(
                    domain_participant_listener_actor::trigger_on_requested_incompatible_qos::new(
                        data_reader_address.clone(),
                        subscriber_address.clone(),
                        participant_address.clone(),
                        runtime_handle.clone(),
                        status,
                    ),
                )
                .await
                .expect("Participant listener should exist");
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn convert_received_data_to_cache_change(
        &mut self,
        writer_guid: Guid,
        key_flag: bool,
        inline_qos: ParameterList,
        data: Data,
        source_timestamp: Option<Time>,
        reception_timestamp: Time,
        type_support: &Arc<dyn DynamicTypeInterface + Send + Sync>,
    ) -> DdsResult<RtpsReaderCacheChange> {
        let change_kind = if key_flag {
            if let Some(p) = inline_qos
                .parameter()
                .iter()
                .find(|&x| x.parameter_id() == PID_STATUS_INFO)
            {
                let mut deserializer =
                    ClassicCdrDeserializer::new(p.value(), CdrEndianness::LittleEndian);
                let status_info: StatusInfo =
                    CdrDeserialize::deserialize(&mut deserializer).unwrap();
                match status_info {
                    STATUS_INFO_DISPOSED => Ok(ChangeKind::NotAliveDisposed),
                    STATUS_INFO_UNREGISTERED => Ok(ChangeKind::NotAliveUnregistered),
                    STATUS_INFO_DISPOSED_UNREGISTERED => {
                        Ok(ChangeKind::NotAliveDisposedUnregistered)
                    }
                    _ => Err(DdsError::Error("Unknown status info value".to_string())),
                }
            } else {
                Err(DdsError::Error(
                    "Missing mandatory StatusInfo parameter in parameter list".to_string(),
                ))
            }
        } else {
            Ok(ChangeKind::Alive)
        }?;

        let instance_handle = build_instance_handle(
            type_support,
            change_kind,
            data.as_ref(),
            inline_qos.parameter(),
        )?;

        match change_kind {
            ChangeKind::Alive | ChangeKind::AliveFiltered => {
                self.instances
                    .entry(instance_handle)
                    .or_insert_with(InstanceState::new)
                    .update_state(change_kind);
                Ok(())
            }
            ChangeKind::NotAliveDisposed
            | ChangeKind::NotAliveUnregistered
            | ChangeKind::NotAliveDisposedUnregistered => {
                match self.instances.get_mut(&instance_handle) {
                    Some(instance) => {
                        instance.update_state(change_kind);
                        Ok(())
                    }
                    None => Err(DdsError::Error(
                        "Received message changing state of unknown instance".to_string(),
                    )),
                }
            }
        }?;

        Ok(RtpsReaderCacheChange {
            kind: change_kind,
            writer_guid,
            data,
            inline_qos,
            source_timestamp,
            instance_handle,
            sample_state: SampleStateKind::NotRead,
            disposed_generation_count: self.instances[&instance_handle]
                .most_recent_disposed_generation_count,
            no_writers_generation_count: self.instances[&instance_handle]
                .most_recent_no_writers_generation_count,
            reception_timestamp,
        })
    }

    #[allow(clippy::too_many_arguments)]
    async fn add_change(
        &mut self,
        change: RtpsReaderCacheChange,
        data_reader_address: &ActorAddress<DataReaderActor>,
        subscriber_address: &ActorAddress<SubscriberActor>,
        participant_address: &ActorAddress<DomainParticipantActor>,
        subscriber_status_condition: &ActorAddress<StatusConditionActor>,
        subscriber_mask_listener: &(ActorAddress<SubscriberListenerActor>, Vec<StatusKind>),
        participant_mask_listener: &(
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: &tokio::runtime::Handle,
    ) -> DdsResult<()> {
        if self.is_sample_of_interest_based_on_time(&change) {
            if self.is_max_samples_limit_reached(&change) {
                self.on_sample_rejected(
                    change.instance_handle,
                    SampleRejectedStatusKind::RejectedBySamplesLimit,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                    subscriber_mask_listener,
                    participant_mask_listener,
                    runtime_handle,
                )
                .await?;
            } else if self.is_max_instances_limit_reached(&change) {
                self.on_sample_rejected(
                    change.instance_handle,
                    SampleRejectedStatusKind::RejectedByInstancesLimit,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                    subscriber_mask_listener,
                    participant_mask_listener,
                    runtime_handle,
                )
                .await?;
            } else if self.is_max_samples_per_instance_limit_reached(&change) {
                self.on_sample_rejected(
                    change.instance_handle,
                    SampleRejectedStatusKind::RejectedBySamplesPerInstanceLimit,
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                    subscriber_mask_listener,
                    participant_mask_listener,
                    runtime_handle,
                )
                .await?;
            } else {
                let num_alive_samples_of_instance = self
                    .changes
                    .iter()
                    .filter(|cc| {
                        cc.instance_handle == change.instance_handle && cc.kind == ChangeKind::Alive
                    })
                    .count() as i32;

                if let HistoryQosPolicyKind::KeepLast(depth) = self.qos.history.kind {
                    if depth == num_alive_samples_of_instance {
                        let index_sample_to_remove = self
                            .changes
                            .iter()
                            .position(|cc| {
                                cc.instance_handle == change.instance_handle
                                    && cc.kind == ChangeKind::Alive
                            })
                            .expect("Samples must exist");
                        self.changes.remove(index_sample_to_remove);
                    }
                }

                self.start_deadline_missed_task(
                    change.instance_handle,
                    data_reader_address.clone(),
                    subscriber_address.clone(),
                    participant_address.clone(),
                    subscriber_mask_listener,
                    participant_mask_listener,
                    runtime_handle.clone(),
                );

                self.changes.push(change);
                self.data_available_status_changed_flag = true;

                match self.qos.destination_order.kind {
                    DestinationOrderQosPolicyKind::BySourceTimestamp => {
                        self.changes.sort_by(|a, b| {
                            a.source_timestamp
                                .as_ref()
                                .expect("Missing source timestamp")
                                .cmp(
                                    b.source_timestamp
                                        .as_ref()
                                        .expect("Missing source timestamp"),
                                )
                        });
                    }
                    DestinationOrderQosPolicyKind::ByReceptionTimestamp => self
                        .changes
                        .sort_by(|a, b| a.reception_timestamp.cmp(&b.reception_timestamp)),
                }

                self.on_data_available(
                    data_reader_address,
                    subscriber_address,
                    participant_address,
                    subscriber_status_condition,
                    subscriber_mask_listener,
                    runtime_handle,
                )
                .await?;
            }
        }

        Ok(())
    }

    fn is_sample_of_interest_based_on_time(&self, change: &RtpsReaderCacheChange) -> bool {
        let closest_timestamp_before_received_sample = self
            .changes
            .iter()
            .filter(|cc| cc.instance_handle == change.instance_handle)
            .filter(|cc| cc.source_timestamp <= change.source_timestamp)
            .map(|cc| cc.source_timestamp)
            .max();

        if let Some(Some(t)) = closest_timestamp_before_received_sample {
            if let Some(sample_source_time) = change.source_timestamp {
                let sample_separation = sample_source_time - t;
                DurationKind::Finite(sample_separation)
                    >= self.qos.time_based_filter.minimum_separation
            } else {
                true
            }
        } else {
            true
        }
    }

    fn is_max_samples_limit_reached(&self, _change: &RtpsReaderCacheChange) -> bool {
        let total_samples = self
            .changes
            .iter()
            .filter(|cc| cc.kind == ChangeKind::Alive)
            .count();

        total_samples == self.qos.resource_limits.max_samples
    }

    fn is_max_instances_limit_reached(&self, change: &RtpsReaderCacheChange) -> bool {
        let instance_handle_list: HashSet<_> =
            self.changes.iter().map(|cc| cc.instance_handle).collect();

        if instance_handle_list.contains(&change.instance_handle) {
            false
        } else {
            instance_handle_list.len() == self.qos.resource_limits.max_instances
        }
    }

    fn is_max_samples_per_instance_limit_reached(&self, change: &RtpsReaderCacheChange) -> bool {
        let total_samples_of_instance = self
            .changes
            .iter()
            .filter(|cc| cc.instance_handle == change.instance_handle)
            .count();

        total_samples_of_instance == self.qos.resource_limits.max_samples_per_instance
    }

    fn create_indexed_sample_collection(
        &mut self,
        max_samples: i32,
        sample_states: &[SampleStateKind],
        view_states: &[ViewStateKind],
        instance_states: &[InstanceStateKind],
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<IndexedSample>> {
        if let Some(h) = specific_instance_handle {
            if !self.instances.contains_key(&h) {
                return Err(DdsError::BadParameter);
            }
        };

        let mut indexed_samples = Vec::new();

        let instances = &self.instances;
        let mut instances_in_collection = HashMap::new();
        for (index, cache_change) in self
            .changes
            .iter()
            .enumerate()
            .filter(|(_, cc)| {
                sample_states.contains(&cc.sample_state)
                    && view_states.contains(&instances[&cc.instance_handle].view_state)
                    && instance_states.contains(&instances[&cc.instance_handle].instance_state)
                    && if let Some(h) = specific_instance_handle {
                        h == cc.instance_handle
                    } else {
                        true
                    }
            })
            .take(max_samples as usize)
        {
            instances_in_collection
                .entry(cache_change.instance_handle)
                .or_insert_with(InstanceState::new);

            instances_in_collection
                .get_mut(&cache_change.instance_handle)
                .unwrap()
                .update_state(cache_change.kind);
            let sample_state = cache_change.sample_state;
            let view_state = self.instances[&cache_change.instance_handle].view_state;
            let instance_state = self.instances[&cache_change.instance_handle].instance_state;

            let absolute_generation_rank = (self.instances[&cache_change.instance_handle]
                .most_recent_disposed_generation_count
                + self.instances[&cache_change.instance_handle]
                    .most_recent_no_writers_generation_count)
                - (instances_in_collection[&cache_change.instance_handle]
                    .most_recent_disposed_generation_count
                    + instances_in_collection[&cache_change.instance_handle]
                        .most_recent_no_writers_generation_count);

            let (data, valid_data) = match cache_change.kind {
                ChangeKind::Alive | ChangeKind::AliveFiltered => {
                    (Some(cache_change.data.clone()), true)
                }
                ChangeKind::NotAliveDisposed
                | ChangeKind::NotAliveUnregistered
                | ChangeKind::NotAliveDisposedUnregistered => (None, false),
            };

            let sample_info = SampleInfo {
                sample_state,
                view_state,
                instance_state,
                disposed_generation_count: cache_change.disposed_generation_count,
                no_writers_generation_count: cache_change.no_writers_generation_count,
                sample_rank: 0,     // To be filled up after collection is created
                generation_rank: 0, // To be filled up after collection is created
                absolute_generation_rank,
                source_timestamp: cache_change.source_timestamp,
                instance_handle: cache_change.instance_handle,
                publication_handle: InstanceHandle::new(cache_change.writer_guid.into()),
                valid_data,
            };

            let sample = (data, sample_info);

            indexed_samples.push(IndexedSample { index, sample })
        }

        // After the collection is created, update the relative generation rank values and mark the read instances as viewed
        for handle in instances_in_collection.into_keys() {
            let most_recent_sample_absolute_generation_rank = indexed_samples
                .iter()
                .filter(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.instance_handle == handle,
                )
                .map(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.absolute_generation_rank,
                )
                .last()
                .expect("Instance handle must exist on collection");

            let mut total_instance_samples_in_collection = indexed_samples
                .iter()
                .filter(
                    |IndexedSample {
                         sample: (_, sample_info),
                         ..
                     }| sample_info.instance_handle == handle,
                )
                .count();

            for IndexedSample {
                sample: (_, sample_info),
                ..
            } in indexed_samples.iter_mut().filter(
                |IndexedSample {
                     sample: (_, sample_info),
                     ..
                 }| sample_info.instance_handle == handle,
            ) {
                sample_info.generation_rank = sample_info.absolute_generation_rank
                    - most_recent_sample_absolute_generation_rank;

                total_instance_samples_in_collection -= 1;
                sample_info.sample_rank = total_instance_samples_in_collection as i32;
            }

            self.instances
                .get_mut(&handle)
                .expect("Sample must exist on hash map")
                .mark_viewed()
        }

        if indexed_samples.is_empty() {
            Err(DdsError::NoData)
        } else {
            Ok(indexed_samples)
        }
    }

    fn next_instance(&self, previous_handle: Option<InstanceHandle>) -> Option<InstanceHandle> {
        match previous_handle {
            Some(p) => self.instances.keys().filter(|&h| h > &p).min().cloned(),
            None => self.instances.keys().min().cloned(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn start_deadline_missed_task(
        &mut self,
        change_instance_handle: InstanceHandle,
        data_reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_mask_listener: &(ActorAddress<SubscriberListenerActor>, Vec<StatusKind>),
        participant_mask_listener: &(
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: tokio::runtime::Handle,
    ) {
        if let Some(t) = self
            .instance_deadline_missed_task
            .remove(&change_instance_handle)
        {
            t.abort();
        }

        if let DurationKind::Finite(deadline_missed_period) = self.qos.deadline.period {
            let mut deadline_missed_interval = tokio::time::interval(tokio::time::Duration::new(
                deadline_missed_period.sec() as u64,
                deadline_missed_period.nanosec(),
            ));
            let reader_status_condition = self.status_condition.address();
            let requested_deadline_missed_status = self.requested_deadline_missed_status.address();
            let reader_listener_address = self.listener.address();
            let reader_listener_mask = self.status_kind.clone();
            let subscriber_listener_address = subscriber_mask_listener.0.clone();
            let subscriber_listener_mask = subscriber_mask_listener.1.clone();
            let participant_listener_address = participant_mask_listener.0.clone();
            let participant_listener_mask = participant_mask_listener.1.clone();
            let deadline_missed_task = tokio::spawn(async move {
                loop {
                    deadline_missed_interval.tick().await;

                    let r: DdsResult<()> = async {
                        requested_deadline_missed_status
                            .send_mail_and_await_reply(
                                increment_requested_deadline_missed_status::new(
                                    change_instance_handle,
                                ),
                            )
                            .await?;

                        reader_status_condition
                            .send_mail_and_await_reply(
                                status_condition_actor::add_communication_state::new(
                                    StatusKind::RequestedDeadlineMissed,
                                ),
                            )
                            .await?;
                        if reader_listener_mask.contains(&StatusKind::RequestedDeadlineMissed) {
                            let status = requested_deadline_missed_status
                                .send_mail_and_await_reply(read_requested_deadline_missed_status::new())
                                .await?;

                            reader_listener_address
                                .send_mail(
                                    data_reader_listener_actor::trigger_on_requested_deadline_missed::new(
                                    data_reader_address.clone(),
                                    subscriber_address.clone(),
                                    participant_address.clone(),
                                    runtime_handle.clone(),
                                    status,
                                ),
                            )
                            .await?;
                        } else if subscriber_listener_mask
                            .contains(&StatusKind::RequestedDeadlineMissed)
                        {
                            let status = requested_deadline_missed_status
                                .send_mail_and_await_reply(read_requested_deadline_missed_status::new())
                                .await?;
                            subscriber_listener_address
                                .send_mail(
                                    subscriber_listener_actor::trigger_on_requested_deadline_missed::new(
                                    data_reader_address.clone(),
                                    subscriber_address.clone(),
                                    participant_address.clone(),
                                    runtime_handle.clone(),
                                    status,
                                ),
                            )
                            .await?;
                        } else if participant_listener_mask
                            .contains(&StatusKind::RequestedDeadlineMissed)
                        {
                            let status = requested_deadline_missed_status
                                .send_mail_and_await_reply(read_requested_deadline_missed_status::new())
                                .await?;
                            participant_listener_address.send_mail(
                                    domain_participant_listener_actor::trigger_on_requested_deadline_missed::new(
                                    data_reader_address.clone(),
                                    subscriber_address.clone(),
                                    participant_address.clone(),
                                    runtime_handle.clone(),
                                    status,
                                ),
                            )
                            .await?;
                        }
                        Ok(())
                    }
                    .await;

                    if r.is_err() {
                        break;
                    }
                }
            });

            self.instance_deadline_missed_task
                .insert(change_instance_handle, deadline_missed_task.abort_handle());
        }
    }
}

#[actor_interface]
impl DataReaderActor {
    async fn read(
        &mut self,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<Data>, SampleInfo)>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.status_condition
            .address()
            .send_mail_and_await_reply(status_condition_actor::remove_communication_state::new(
                StatusKind::DataAvailable,
            ))
            .await?;

        let indexed_sample_list = self.create_indexed_sample_collection(
            max_samples,
            &sample_states,
            &view_states,
            &instance_states,
            specific_instance_handle,
        )?;

        let change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list
            .into_iter()
            .map(|IndexedSample { index, sample }| (index, sample))
            .unzip();

        for index in change_index_list {
            self.changes[index].sample_state = SampleStateKind::Read;
        }

        Ok(samples)
    }

    async fn take(
        &mut self,
        max_samples: i32,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
        specific_instance_handle: Option<InstanceHandle>,
    ) -> DdsResult<Vec<(Option<Data>, SampleInfo)>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        let indexed_sample_list = self.create_indexed_sample_collection(
            max_samples,
            &sample_states,
            &view_states,
            &instance_states,
            specific_instance_handle,
        )?;

        self.status_condition
            .address()
            .send_mail_and_await_reply(status_condition_actor::remove_communication_state::new(
                StatusKind::DataAvailable,
            ))
            .await?;

        let mut change_index_list: Vec<usize>;
        let samples;

        (change_index_list, samples) = indexed_sample_list
            .into_iter()
            .map(|IndexedSample { index, sample }| (index, sample))
            .unzip();

        while let Some(index) = change_index_list.pop() {
            self.changes.remove(index);
        }

        Ok(samples)
    }

    async fn is_historical_data_received(&self) -> DdsResult<bool> {
        if !self.enabled {
            Err(DdsError::NotEnabled)
        } else {
            Ok(())
        }?;

        match self.qos.durability.kind {
            DurabilityQosPolicyKind::Volatile => Err(DdsError::IllegalOperation),
            DurabilityQosPolicyKind::TransientLocal => Ok(()),
        }?;

        Ok(!self
            .matched_writers
            .iter()
            .any(|p| !p.is_historical_data_received()))
    }

    async fn as_discovered_reader_data(
        &self,
        topic_qos: TopicQos,
        subscriber_qos: SubscriberQos,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        xml_type: String,
    ) -> DiscoveredReaderData {
        let guid = self.rtps_reader.guid();

        let unicast_locator_list = if self.rtps_reader.unicast_locator_list().is_empty() {
            default_unicast_locator_list
        } else {
            self.rtps_reader.unicast_locator_list().to_vec()
        };

        let multicast_locator_list = if self.rtps_reader.unicast_locator_list().is_empty() {
            default_multicast_locator_list
        } else {
            self.rtps_reader.multicast_locator_list().to_vec()
        };

        DiscoveredReaderData::new(
            ReaderProxy::new(
                guid,
                guid.entity_id(),
                unicast_locator_list,
                multicast_locator_list,
                false,
            ),
            SubscriptionBuiltinTopicData::new(
                BuiltInTopicKey { value: guid.into() },
                BuiltInTopicKey {
                    value: GUID_UNKNOWN.into(),
                },
                self.topic_name.clone(),
                self.type_name.to_string(),
                self.qos.durability.clone(),
                self.qos.deadline.clone(),
                self.qos.latency_budget.clone(),
                self.qos.liveliness.clone(),
                self.qos.reliability.clone(),
                self.qos.ownership.clone(),
                self.qos.destination_order.clone(),
                self.qos.user_data.clone(),
                self.qos.time_based_filter.clone(),
                subscriber_qos.presentation.clone(),
                subscriber_qos.partition.clone(),
                topic_qos.topic_data,
                subscriber_qos.group_data,
                xml_type,
            ),
        )
    }

    async fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.rtps_reader.guid().into())
    }

    async fn set_qos(&mut self, qos: DataReaderQos) -> DdsResult<()> {
        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        qos.is_consistent()?;
        self.qos = qos;

        Ok(())
    }

    async fn get_qos(&self) -> DataReaderQos {
        self.qos.clone()
    }

    async fn matched_writer_remove(&mut self, a_writer_guid: Guid) {
        self.matched_writers
            .retain(|x| x.remote_writer_guid() != a_writer_guid)
    }

    async fn get_matched_publication_data(
        &self,
        publication_handle: InstanceHandle,
    ) -> DdsResult<PublicationBuiltinTopicData> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        self.matched_publication_list
            .get(&publication_handle)
            .cloned()
            .ok_or(DdsError::BadParameter)
    }

    async fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn enable(&mut self) {
        self.enabled = true;
    }

    async fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    async fn get_matched_publications(&self) -> Vec<InstanceHandle> {
        self.matched_publication_list
            .iter()
            .map(|(&key, _)| key)
            .collect()
    }

    async fn take_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<Data>, SampleInfo)>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => {
                self.take(
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    Some(next_handle),
                )
                .await
            }
            None => Err(DdsError::NoData),
        }
    }

    async fn get_subscription_matched_status(&mut self) -> SubscriptionMatchedStatus {
        self.status_condition
            .send_mail_and_await_reply(status_condition_actor::remove_communication_state::new(
                StatusKind::SubscriptionMatched,
            ))
            .await;

        self.subscription_matched_status
            .read_and_reset(self.matched_publication_list.len() as i32)
    }

    async fn read_next_instance(
        &mut self,
        max_samples: i32,
        previous_handle: Option<InstanceHandle>,
        sample_states: Vec<SampleStateKind>,
        view_states: Vec<ViewStateKind>,
        instance_states: Vec<InstanceStateKind>,
    ) -> DdsResult<Vec<(Option<Data>, SampleInfo)>> {
        if !self.enabled {
            return Err(DdsError::NotEnabled);
        }

        match self.next_instance(previous_handle) {
            Some(next_handle) => {
                self.read(
                    max_samples,
                    sample_states,
                    view_states,
                    instance_states,
                    Some(next_handle),
                )
                .await
            }
            None => Err(DdsError::NoData),
        }
    }

    async fn matched_writer_add(&mut self, a_writer_proxy: RtpsWriterProxy) {
        if !self
            .matched_writers
            .iter()
            .any(|x| x.remote_writer_guid() == a_writer_proxy.remote_writer_guid())
        {
            self.matched_writers.push(a_writer_proxy);
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn add_matched_writer(
        &mut self,
        discovered_writer_data: DiscoveredWriterData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        data_reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_qos: SubscriberQos,
        subscriber_mask_listener: (ActorAddress<SubscriberListenerActor>, Vec<StatusKind>),
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: tokio::runtime::Handle,
    ) {
        let publication_builtin_topic_data = discovered_writer_data.dds_publication_data();
        if publication_builtin_topic_data.topic_name() == self.topic_name
            && publication_builtin_topic_data.get_type_name() == self.type_name
        {
            let instance_handle =
                get_instance_handle_from_key(&discovered_writer_data.get_key().unwrap()).unwrap();
            let incompatible_qos_policy_list = self
                .get_discovered_writer_incompatible_qos_policy_list(
                    &discovered_writer_data,
                    &subscriber_qos,
                );
            if incompatible_qos_policy_list.is_empty() {
                let unicast_locator_list = if discovered_writer_data
                    .writer_proxy()
                    .unicast_locator_list()
                    .is_empty()
                {
                    default_unicast_locator_list
                } else {
                    discovered_writer_data
                        .writer_proxy()
                        .unicast_locator_list()
                        .to_vec()
                };

                let multicast_locator_list = if discovered_writer_data
                    .writer_proxy()
                    .multicast_locator_list()
                    .is_empty()
                {
                    default_multicast_locator_list
                } else {
                    discovered_writer_data
                        .writer_proxy()
                        .multicast_locator_list()
                        .to_vec()
                };

                let writer_proxy = RtpsWriterProxy::new(
                    discovered_writer_data.writer_proxy().remote_writer_guid(),
                    &unicast_locator_list,
                    &multicast_locator_list,
                    discovered_writer_data
                        .writer_proxy()
                        .data_max_size_serialized(),
                    discovered_writer_data
                        .writer_proxy()
                        .remote_group_entity_id(),
                );

                if !self
                    .matched_writers
                    .iter()
                    .any(|x| x.remote_writer_guid() == writer_proxy.remote_writer_guid())
                {
                    self.matched_writers.push(writer_proxy);
                }

                let insert_matched_publication_result = self
                    .matched_publication_list
                    .insert(instance_handle, publication_builtin_topic_data.clone());
                match insert_matched_publication_result {
                    Some(value) if &value != publication_builtin_topic_data => {
                        self.on_subscription_matched(
                            instance_handle,
                            data_reader_address,
                            subscriber_address,
                            participant_address,
                            &subscriber_mask_listener,
                            &participant_mask_listener,
                            &runtime_handle,
                        )
                        .await;
                    }
                    None => {
                        self.on_subscription_matched(
                            instance_handle,
                            data_reader_address,
                            subscriber_address,
                            participant_address,
                            &subscriber_mask_listener,
                            &participant_mask_listener,
                            &runtime_handle,
                        )
                        .await;
                    }
                    _ => (),
                }
            } else if self.incompatible_writer_list.insert(instance_handle) {
                self.on_requested_incompatible_qos(
                    incompatible_qos_policy_list,
                    &data_reader_address,
                    &subscriber_address,
                    &participant_address,
                    &subscriber_mask_listener,
                    &participant_mask_listener,
                    &runtime_handle,
                )
                .await;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn remove_matched_writer(
        &mut self,
        discovered_writer_handle: InstanceHandle,
        data_reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_mask_listener: (ActorAddress<SubscriberListenerActor>, Vec<StatusKind>),
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        runtime_handle: tokio::runtime::Handle,
    ) {
        let matched_publication = self
            .matched_publication_list
            .remove(&discovered_writer_handle);
        if let Some(w) = matched_publication {
            self.matched_writer_remove(w.key().value.into()).await;

            self.on_subscription_matched(
                discovered_writer_handle,
                data_reader_address,
                subscriber_address,
                participant_address,
                &subscriber_mask_listener,
                &participant_mask_listener,
                &runtime_handle,
            )
            .await;
        }
    }

    async fn get_topic_name(&mut self) -> String {
        self.topic_name.clone()
    }

    async fn get_type_name(&self) -> String {
        self.type_name.to_string()
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_rtps_message(
        &mut self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
        data_reader_address: ActorAddress<DataReaderActor>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        subscriber_status_condition: ActorAddress<StatusConditionActor>,
        subscriber_mask_listener: (ActorAddress<SubscriberListenerActor>, Vec<StatusKind>),
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        type_support_actor_address: ActorAddress<TypeSupportActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> DdsResult<()> {
        let mut message_receiver = MessageReceiver::new(&message);
        let type_support = type_support_actor_address
            .send_mail_and_await_reply(type_support_actor::get_type_support::new(
                self.type_name.clone(),
            ))
            .await?
            .ok_or_else(|| {
                DdsError::PreconditionNotMet(format!(
                    "Type with name {} not registered with parent domain participant",
                    self.type_name
                ))
            })?;

        while let Some(submessage) = message_receiver.next() {
            match submessage {
                RtpsSubmessageReadKind::Data(data_submessage) => {
                    self.on_data_submessage_received(
                        &data_submessage,
                        &type_support,
                        message_receiver.source_guid_prefix(),
                        message_receiver.source_timestamp(),
                        reception_timestamp,
                        &data_reader_address,
                        &subscriber_address,
                        &participant_address,
                        &subscriber_status_condition,
                        &subscriber_mask_listener,
                        &participant_mask_listener,
                        &runtime_handle,
                    )
                    .await?;
                }
                RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => {
                    self.on_data_frag_submessage_received(
                        &data_frag_submessage,
                        &type_support,
                        message_receiver.source_guid_prefix(),
                        message_receiver.source_timestamp(),
                        reception_timestamp,
                        &data_reader_address,
                        &subscriber_address,
                        &participant_address,
                        &subscriber_status_condition,
                        &subscriber_mask_listener,
                        &participant_mask_listener,
                        &runtime_handle,
                    )
                    .await?;
                }
                RtpsSubmessageReadKind::Gap(gap_submessage) => {
                    self.on_gap_submessage_received(
                        &gap_submessage,
                        message_receiver.source_guid_prefix(),
                    );
                }
                RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => self
                    .on_heartbeat_submessage_received(
                        &heartbeat_submessage,
                        message_receiver.source_guid_prefix(),
                    ),
                RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) => self
                    .on_heartbeat_frag_submessage_received(
                        &heartbeat_frag_submessage,
                        message_receiver.source_guid_prefix(),
                    ),
                _ => (),
            }
        }
        Ok(())
    }

    async fn send_message(
        &mut self,
        header: RtpsMessageHeader,
        udp_transport_write: Arc<UdpTransportWrite>,
    ) {
        for writer_proxy in self.matched_writers.iter_mut() {
            writer_proxy.send_message(&self.rtps_reader.guid(), header, &udp_transport_write)
        }
    }

    async fn set_listener(
        &mut self,
        listener: Box<dyn AnyDataReaderListener + Send>,
        status_kind: Vec<StatusKind>,
        runtime_handle: tokio::runtime::Handle,
    ) {
        self.listener = Actor::spawn(DataReaderListenerActor::new(listener), &runtime_handle);
        self.status_kind = status_kind;
    }

    async fn get_requested_deadline_missed_status(&mut self) -> RequestedDeadlineMissedStatus {
        self.requested_deadline_missed_status
            .send_mail_and_await_reply(read_requested_deadline_missed_status::new())
            .await
    }
}
