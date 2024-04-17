use std::{collections::HashMap, sync::Arc};

use dust_dds_derive::actor_interface;
use fnmatch_regex::glob_to_regex;
use tracing::warn;

use super::{
    any_data_reader_listener::AnyDataReaderListener, data_reader_actor::DataReaderActor,
    subscriber_listener_actor::SubscriberListenerActor, topic_actor::TopicActor,
    type_support_actor::TypeSupportActor,
};
use crate::{
    dds_async::{
        domain_participant::DomainParticipantAsync, subscriber::SubscriberAsync,
        subscriber_listener::SubscriberListenerAsync,
    },
    implementation::{
        actors::{
            domain_participant_listener_actor::DomainParticipantListenerActor,
            status_condition_actor::StatusConditionActor,
        },
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
        rtps::{
            self,
            behavior_types::DURATION_ZERO,
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
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        qos_policy::PartitionQosPolicy,
        status::StatusKind,
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
        listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        data_reader_list: Vec<DataReaderActor>,
        handle: &tokio::runtime::Handle,
    ) -> Self {
        let status_condition = Actor::spawn(StatusConditionActor::default(), handle);
        let listener = Actor::spawn(SubscriberListenerActor::new(listener), handle);
        let data_reader_list = data_reader_list
            .into_iter()
            .map(|dr| (dr.get_instance_handle(), Actor::spawn(dr, handle)))
            .collect();
        SubscriberActor {
            qos,
            rtps_group,
            data_reader_list,
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
    fn create_datareader(
        &mut self,
        type_name: String,
        topic_name: String,
        has_key: bool,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
        mask: Vec<StatusKind>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        runtime_handle: tokio::runtime::Handle,
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
        );

        let reader_actor = Actor::spawn(data_reader, &runtime_handle);
        let reader_address = reader_actor.address();
        self.data_reader_list
            .insert(InstanceHandle::new(guid.into()), reader_actor);

        Ok(reader_address)
    }

    fn delete_datareader(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        let removed_reader = self.data_reader_list.remove(&handle);
        if removed_reader.is_some() {
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Data reader can only be deleted from its parent subscriber".to_string(),
            ))
        }
    }

    async fn lookup_datareader(&self, topic_name: String) -> Option<Actor<DataReaderActor>> {
        for dr in self.data_reader_list.values() {
            if dr.get_topic_name().await == topic_name {
                return Some(dr.clone());
            }
        }
        None
    }

    fn delete_contained_entities(&mut self) -> Vec<InstanceHandle> {
        self.data_reader_list.drain().map(|(h, _)| h).collect()
    }

    fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    fn is_empty(&self) -> bool {
        self.data_reader_list.is_empty()
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_default_datareader_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }

    fn get_default_datareader_qos(&self) -> DataReaderQos {
        self.default_data_reader_qos.clone()
    }

    fn set_qos(&mut self, qos: QosKind<SubscriberQos>) -> DdsResult<()> {
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

    #[allow(clippy::unused_unit)]
    fn enable(&mut self) -> () {
        self.enabled = true;
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.rtps_group.guid().into())
    }

    pub fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    fn get_qos(&self) -> SubscriberQos {
        self.qos.clone()
    }

    fn data_reader_list(&self) -> Vec<ActorAddress<DataReaderActor>> {
        self.data_reader_list
            .values()
            .map(|dr| dr.address())
            .collect()
    }

    fn get_status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }

    async fn send_message(
        &self,
        header: RtpsMessageHeader,
        udp_transport_write: Arc<UdpTransportWrite>,
    ) {
        for data_reader_address in self.data_reader_list.values() {
            data_reader_address
                .send_message(header, udp_transport_write.clone())
                .await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn process_rtps_message(
        &self,
        message: RtpsMessageRead,
        reception_timestamp: rtps::messages::types::Time,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant: DomainParticipantAsync,
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        type_support_actor_address: ActorAddress<TypeSupportActor>,
        topic_list: HashMap<String, Actor<TopicActor>>,
    ) {
        let subscriber_mask_listener = (self.listener.address(), self.status_kind.clone());

        for data_reader_address in self.data_reader_list.values() {
            data_reader_address
                .process_rtps_message(
                    message.clone(),
                    reception_timestamp,
                    data_reader_address.address(),
                    SubscriberAsync::new(
                        subscriber_address.clone(),
                        self.status_condition.address(),
                        participant.clone(),
                    ),
                    subscriber_mask_listener.clone(),
                    participant_mask_listener.clone(),
                    type_support_actor_address.clone(),
                    topic_list.clone(),
                )
                .await;
        }
    }

    #[allow(clippy::too_many_arguments, clippy::unused_unit)]
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
        topic_list: HashMap<String, Actor<TopicActor>>,
    ) -> () {
        if self.is_partition_matched(discovered_writer_data.dds_publication_data().partition()) {
            for data_reader in self.data_reader_list.values() {
                let subscriber_mask_listener = (self.listener.address(), self.status_kind.clone());
                let data_reader_address = data_reader.address();
                let subscriber_qos = self.qos.clone();
                data_reader
                    .add_matched_writer(
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
                        topic_list.clone(),
                    )
                    .await;
            }
        }
    }

    #[allow(clippy::unused_unit)]
    async fn remove_matched_writer(
        &self,
        discovered_writer_handle: InstanceHandle,
        subscriber_address: ActorAddress<SubscriberActor>,
        participant: DomainParticipantAsync,
        participant_mask_listener: (
            ActorAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
        topic_list: HashMap<String, Actor<TopicActor>>,
    ) -> () {
        for data_reader in self.data_reader_list.values() {
            let data_reader_address = data_reader.address();
            let subscriber_mask_listener = (self.listener.address(), self.status_kind.clone());
            data_reader
                .remove_matched_writer(
                    discovered_writer_handle,
                    data_reader_address,
                    SubscriberAsync::new(
                        subscriber_address.clone(),
                        self.status_condition.address(),
                        participant.clone(),
                    ),
                    subscriber_mask_listener,
                    participant_mask_listener.clone(),
                    topic_list.clone(),
                )
                .await;
        }
    }

    #[allow(clippy::unused_unit)]
    fn set_listener(
        &mut self,
        listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        runtime_handle: tokio::runtime::Handle,
    ) -> () {
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
