use std::{collections::HashMap, sync::Arc};

use dust_dds_derive::actor_interface;
use fnmatch_regex::glob_to_regex;
use tracing::warn;

use super::{
    any_data_reader_listener::AnyDataReaderListener,
    data_reader_actor::{self, DataReaderActor},
    subscriber_listener_actor::{SubscriberListenerActor, SubscriberListenerAsyncDyn},
    topic_actor::TopicActor,
    type_support_actor::TypeSupportActor,
};
use crate::{
    dds_async::{domain_participant::DomainParticipantAsync, subscriber::SubscriberAsync},
    implementation::{
        actors::{
            domain_participant_listener_actor::DomainParticipantListenerActor,
            status_condition_actor::StatusConditionActor,
        },
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::overall_structure::{RtpsMessageHeader, RtpsMessageRead},
            reader::{RtpsReader, RtpsReaderKind, RtpsStatefulReader},
            types::{
                EntityId, Guid, Locator, TopicKind, USER_DEFINED_READER_NO_KEY,
                USER_DEFINED_READER_WITH_KEY,
            },
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{Actor, ActorAddress},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        qos_policy::PartitionQosPolicy,
        status::StatusKind,
        time::{Time, DURATION_ZERO},
    },
};

pub struct SubscriberActor {
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    data_reader_list: HashMap<InstanceHandle, Actor<DataReaderActor>>,
    enabled: bool,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    status_condition: Actor<StatusConditionActor>,
    listener: Actor<SubscriberListenerActor>,
    status_kind: Vec<StatusKind>,
}

impl SubscriberActor {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        listener: Box<dyn SubscriberListenerAsyncDyn + Send>,
        status_kind: Vec<StatusKind>,
        handle: &tokio::runtime::Handle,
    ) -> Self {
        let status_condition = Actor::spawn(StatusConditionActor::default(), handle);
        let listener = Actor::spawn(SubscriberListenerActor::new(listener), handle);
        SubscriberActor {
            qos,
            rtps_group,
            data_reader_list: HashMap::new(),
            enabled: false,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
            status_condition,
            listener,
            status_kind,
        }
    }

    fn get_unique_reader_id(&mut self) -> u8 {
        let counter = self.user_defined_data_reader_counter;
        self.user_defined_data_reader_counter += 1;
        counter
    }
}

#[actor_interface]
impl SubscriberActor {
    #[allow(clippy::too_many_arguments)]
    async fn create_datareader(
        &mut self,
        type_name: String,
        topic_name: String,
        has_key: bool,
        qos: QosKind<DataReaderQos>,
        a_listener: Box<dyn AnyDataReaderListener + Send>,
        mask: Vec<StatusKind>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        runtime_handle: tokio::runtime::Handle,
        topic_address: ActorAddress<TopicActor>,
        topic_status_condition: ActorAddress<StatusConditionActor>,
    ) -> DdsResult<ActorAddress<DataReaderActor>> {
        let qos = match qos {
            QosKind::Default => self.default_data_reader_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let entity_kind = match has_key {
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

        let topic_kind = match has_key {
            true => TopicKind::WithKey,
            false => TopicKind::NoKey,
        };

        let rtps_reader = RtpsReaderKind::Stateful(RtpsStatefulReader::new(RtpsReader::new(
            RtpsEndpoint::new(
                guid,
                topic_kind,
                &default_unicast_locator_list,
                &default_multicast_locator_list,
            ),
            DURATION_ZERO,
            DURATION_ZERO,
            false,
        )));

        let status_kind = mask.to_vec();
        let data_reader = DataReaderActor::new(
            rtps_reader,
            type_name,
            topic_name,
            qos,
            a_listener,
            status_kind,
            &runtime_handle,
            topic_address,
            topic_status_condition,
        );

        let reader_actor = Actor::spawn(data_reader, &runtime_handle);
        let reader_address = reader_actor.address();
        self.data_reader_list
            .insert(InstanceHandle::new(guid.into()), reader_actor);

        Ok(reader_address)
    }

    async fn lookup_datareader(&self, topic_name: String) -> Option<ActorAddress<DataReaderActor>> {
        for dr in self.data_reader_list.values() {
            if dr
                .send_mail_and_await_reply(data_reader_actor::get_topic_name::new())
                .await
                == topic_name
            {
                return Some(dr.address());
            }
        }
        None
    }
    async fn delete_contained_entities(&mut self) {}

    async fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    async fn is_empty(&self) -> bool {
        self.data_reader_list.is_empty()
    }

    async fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn data_reader_add(
        &mut self,
        instance_handle: InstanceHandle,
        data_reader: Actor<DataReaderActor>,
    ) {
        self.data_reader_list.insert(instance_handle, data_reader);
    }

    async fn data_reader_delete(&mut self, handle: InstanceHandle) {
        self.data_reader_list.remove(&handle);
    }

    async fn set_default_datareader_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }

    async fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_data_reader_qos.clone()
    }

    async fn set_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
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

    async fn enable(&mut self) {
        self.enabled = true;
    }

    async fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.rtps_group.guid().into())
    }

    async fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    async fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    async fn data_reader_list(&self) -> Vec<ActorAddress<DataReaderActor>> {
        self.data_reader_list
            .values()
            .map(|dr| dr.address())
            .collect()
    }

    async fn get_listener(&self) -> ActorAddress<SubscriberListenerActor> {
        self.listener.address()
    }

    async fn get_status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }

    async fn send_message(
        &self,
        header: RtpsMessageHeader,
        udp_transport_write: Arc<UdpTransportWrite>,
    ) {
        for data_reader_address in self.data_reader_list.values() {
            data_reader_address
                .send_mail(data_reader_actor::send_message::new(
                    header,
                    udp_transport_write.clone(),
                ))
                .await;
        }
    }

    async fn process_rtps_message(
        &self,
        message: RtpsMessageRead,
        reception_timestamp: Time,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant: DomainParticipantAsync,
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        type_support_actor_address: ActorAddress<TypeSupportActor>,
    ) -> DdsResult<()> {
        let subscriber_mask_listener = (self.listener.address(), self.status_kind.clone());

        for data_reader_address in self.data_reader_list.values().map(|a| a.address()) {
            data_reader_address
                .send_mail_and_await_reply(data_reader_actor::process_rtps_message::new(
                    message.clone(),
                    reception_timestamp,
                    data_reader_address.clone(),
                    SubscriberAsync::new(
                        subscriber_address.clone(),
                        self.status_condition.address(),
                        participant.clone(),
                    ),
                    subscriber_mask_listener.clone(),
                    participant_mask_listener.clone(),
                    type_support_actor_address.clone(),
                ))
                .await??;
        }
        Ok(())
    }

    async fn add_matched_writer(
        &self,
        discovered_writer_data: DiscoveredWriterData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant: DomainParticipantAsync,
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
    ) {
        if self.is_partition_matched(discovered_writer_data.dds_publication_data().partition()) {
            for data_reader in self.data_reader_list.values() {
                let subscriber_mask_listener = (self.listener.address(), self.status_kind.clone());
                let data_reader_address = data_reader.address();
                let subscriber_qos = self.qos.clone();
                data_reader
                    .send_mail_and_await_reply(data_reader_actor::add_matched_writer::new(
                        discovered_writer_data.clone(),
                        default_unicast_locator_list.clone(),
                        default_multicast_locator_list.clone(),
                        data_reader_address,
                        SubscriberAsync::new(
                            subscriber_address.clone(),
                            self.status_condition.address(),
                            participant.clone(),
                        ),
                        subscriber_qos,
                        subscriber_mask_listener,
                        participant_mask_listener.clone(),
                    ))
                    .await;
            }
        }
    }

    async fn remove_matched_writer(
        &self,
        discovered_writer_handle: InstanceHandle,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant: DomainParticipantAsync,
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
    ) {
        for data_reader in self.data_reader_list.values() {
            let data_reader_address = data_reader.address();
            let subscriber_mask_listener = (self.listener.address(), self.status_kind.clone());
            data_reader
                .send_mail_and_await_reply(data_reader_actor::remove_matched_writer::new(
                    discovered_writer_handle,
                    data_reader_address,
                    SubscriberAsync::new(
                        subscriber_address.clone(),
                        self.status_condition.address(),
                        participant.clone(),
                    ),
                    subscriber_mask_listener,
                    participant_mask_listener.clone(),
                ))
                .await;
        }
    }

    async fn set_listener(
        &mut self,
        listener: Box<dyn SubscriberListenerAsyncDyn + Send>,
        status_kind: Vec<StatusKind>,
        runtime_handle: tokio::runtime::Handle,
    ) {
        self.listener = Actor::spawn(SubscriberListenerActor::new(listener), &runtime_handle);
        self.status_kind = status_kind;
    }
}

impl SubscriberActor {
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
