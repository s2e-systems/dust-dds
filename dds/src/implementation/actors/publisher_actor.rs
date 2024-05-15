use std::collections::HashMap;

use dust_dds_derive::actor_interface;
use fnmatch_regex::glob_to_regex;
use tracing::warn;

use crate::{
    data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
    dds_async::{
        domain_participant::DomainParticipantAsync, publisher::PublisherAsync,
        publisher_listener::PublisherListenerAsync,
    },
    implementation::actor::{Actor, ActorWeakAddress, DEFAULT_ACTOR_BUFFER_SIZE},
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        qos_policy::PartitionQosPolicy,
        status::StatusKind,
        time::Duration,
    },
    rtps::{
        behavior_types::DURATION_ZERO,
        endpoint::RtpsEndpoint,
        group::RtpsGroup,
        messages::overall_structure::RtpsMessageRead,
        types::{
            EntityId, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
            USER_DEFINED_WRITER_WITH_KEY,
        },
        writer::RtpsWriter,
    },
};

use super::{
    any_data_writer_listener::AnyDataWriterListener, data_writer_actor::DataWriterActor,
    domain_participant_listener_actor::DomainParticipantListenerActor,
    message_sender_actor::MessageSenderActor, publisher_listener_actor::PublisherListenerActor,
    status_condition_actor::StatusConditionActor, topic_actor::TopicActor,
};

pub struct PublisherActor {
    qos: PublisherQos,
    rtps_group: RtpsGroup,
    data_writer_list: HashMap<InstanceHandle, Actor<DataWriterActor>>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
    listener: Actor<PublisherListenerActor>,
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
        handle: &tokio::runtime::Handle,
    ) -> Self {
        let data_writer_list = data_writer_list
            .into_iter()
            .map(|dw| {
                (
                    dw.get_instance_handle(),
                    Actor::spawn(dw, handle, DEFAULT_ACTOR_BUFFER_SIZE),
                )
            })
            .collect();
        Self {
            qos,
            rtps_group,
            data_writer_list,
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
            listener: Actor::spawn(
                PublisherListenerActor::new(listener),
                handle,
                DEFAULT_ACTOR_BUFFER_SIZE,
            ),
            status_kind,
            status_condition: Actor::spawn(
                StatusConditionActor::default(),
                handle,
                DEFAULT_ACTOR_BUFFER_SIZE,
            ),
        }
    }

    fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }
}

#[actor_interface]
impl PublisherActor {
    #[allow(clippy::too_many_arguments)]
    fn create_datawriter(
        &mut self,
        topic_address: ActorWeakAddress<TopicActor>,
        has_key: bool,
        data_max_size_serialized: usize,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        mask: Vec<StatusKind>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        runtime_handle: tokio::runtime::Handle,
    ) -> DdsResult<ActorWeakAddress<DataWriterActor>> {
        let qos = match qos {
            QosKind::Default => self.default_datawriter_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let guid_prefix = self.rtps_group.guid().prefix();
        let (entity_kind, topic_kind) = match has_key {
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
                &default_unicast_locator_list,
                &default_multicast_locator_list,
            ),
            true,
            Duration::new(0, 200_000_000).into(),
            DURATION_ZERO,
            DURATION_ZERO,
            data_max_size_serialized,
        );

        let data_writer = DataWriterActor::new(
            rtps_writer_impl,
            topic_address,
            a_listener,
            mask,
            qos,
            &runtime_handle,
        );
        let data_writer_actor =
            Actor::spawn(data_writer, &runtime_handle, DEFAULT_ACTOR_BUFFER_SIZE);
        let data_writer_address = data_writer_actor.address();
        self.data_writer_list
            .insert(InstanceHandle::new(guid.into()), data_writer_actor);

        Ok(data_writer_address)
    }

    fn delete_datawriter(&mut self, handle: InstanceHandle) -> DdsResult<Actor<DataWriterActor>> {
        if let Some(removed_writer) = self.data_writer_list.remove(&handle) {
            Ok(removed_writer)
        } else {
            Err(DdsError::PreconditionNotMet(
                "Data writer can only be deleted from its parent publisher".to_string(),
            ))
        }
    }

    async fn lookup_datawriter(
        &self,
        topic_name: String,
    ) -> Option<ActorWeakAddress<DataWriterActor>> {
        for dw in self.data_writer_list.values() {
            if dw.get_topic_name().await.as_ref() == Ok(&topic_name) {
                return Some(dw.address());
            }
        }
        None
    }

    #[allow(clippy::unused_unit)]
    fn enable(&mut self) -> () {
        self.enabled = true;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn is_empty(&self) -> bool {
        self.data_writer_list.is_empty()
    }

    fn drain_data_writer_list(&mut self) -> Vec<Actor<DataWriterActor>> {
        self.data_writer_list.drain().map(|(_, a)| a).collect()
    }

    #[allow(clippy::unused_unit)]
    fn set_default_datawriter_qos(&mut self, qos: DataWriterQos) -> () {
        self.default_datawriter_qos = qos;
    }

    fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.clone()
    }

    fn set_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
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

    fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.rtps_group.guid().into())
    }

    fn get_status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }

    fn get_qos(&self) -> PublisherQos {
        self.qos.clone()
    }

    fn data_writer_list(&self) -> Vec<ActorWeakAddress<DataWriterActor>> {
        self.data_writer_list
            .values()
            .map(|x| x.address())
            .collect()
    }

    async fn process_rtps_message(&self, message: RtpsMessageRead) {
        for data_writer_address in self.data_writer_list.values() {
            data_writer_address
                .process_rtps_message(message.clone())
                .await;
        }
    }

    async fn send_message(&self, message_sender_actor: ActorWeakAddress<MessageSenderActor>) {
        for data_writer_address in self.data_writer_list.values() {
            data_writer_address
                .send_message(message_sender_actor.clone())
                .await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn add_matched_reader(
        &self,
        discovered_reader_data: DiscoveredReaderData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        publisher_address: ActorWeakAddress<PublisherActor>,
        participant: DomainParticipantAsync,
        participant_mask_listener: (
            ActorWeakAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        if self.is_partition_matched(
            discovered_reader_data
                .subscription_builtin_topic_data()
                .partition(),
        ) {
            for data_writer in self.data_writer_list.values() {
                let data_writer_address = data_writer.address();
                let publisher_mask_listener = (self.listener.address(), self.status_kind.clone());

                data_writer
                    .add_matched_reader(
                        discovered_reader_data.clone(),
                        default_unicast_locator_list.clone(),
                        default_multicast_locator_list.clone(),
                        data_writer_address,
                        PublisherAsync::new(
                            publisher_address.clone(),
                            self.status_condition.address(),
                            participant.clone(),
                        ),
                        self.qos.clone(),
                        publisher_mask_listener,
                        participant_mask_listener.clone(),
                    )
                    .await?;
            }
        }
        Ok(())
    }

    async fn remove_matched_reader(
        &self,
        discovered_reader_handle: InstanceHandle,
        publisher_address: ActorWeakAddress<PublisherActor>,
        participant: DomainParticipantAsync,
        participant_mask_listener: (
            ActorWeakAddress<DomainParticipantListenerActor>,
            Vec<StatusKind>,
        ),
    ) -> DdsResult<()> {
        for data_writer in self.data_writer_list.values() {
            let data_writer_address = data_writer.address();
            let publisher_mask_listener = (self.listener.address(), self.status_kind.clone());
            data_writer
                .remove_matched_reader(
                    discovered_reader_handle,
                    data_writer_address,
                    PublisherAsync::new(
                        publisher_address.clone(),
                        self.status_condition.address(),
                        participant.clone(),
                    ),
                    publisher_mask_listener,
                    participant_mask_listener.clone(),
                )
                .await?;
        }
        Ok(())
    }

    pub fn get_statuscondition(&self) -> ActorWeakAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    #[allow(clippy::unused_unit)]
    fn set_listener(
        &mut self,
        listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        runtime_handle: tokio::runtime::Handle,
    ) -> () {
        self.listener = Actor::spawn(
            PublisherListenerActor::new(listener),
            &runtime_handle,
            DEFAULT_ACTOR_BUFFER_SIZE,
        );
        self.status_kind = status_kind;
    }
}

impl PublisherActor {
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

impl PublisherQos {
    fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.presentation != other.presentation {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}
