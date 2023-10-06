use std::collections::HashMap;

use dust_dds_derive::actor_interface;
use fnmatch_regex::glob_to_regex;
use tracing::warn;

use crate::{
    implementation::{
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        dds::dds_data_writer_listener::DdsDataWriterListener,
        rtps::{
            endpoint::RtpsEndpoint,
            group::RtpsGroup,
            messages::overall_structure::{RtpsMessageHeader, RtpsMessageRead},
            types::{
                EntityId, Guid, Locator, TopicKind, USER_DEFINED_WRITER_NO_KEY,
                USER_DEFINED_WRITER_WITH_KEY,
            },
            writer::RtpsWriter,
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{spawn_actor, Actor, ActorAddress},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        qos_policy::PartitionQosPolicy,
        status::StatusKind,
        time::{Duration, Time, DURATION_ZERO},
    },
};

use super::{
    dds_data_writer::{self, DdsDataWriter},
    dds_domain_participant::DdsDomainParticipant,
    dds_domain_participant_listener::DdsDomainParticipantListener,
    dds_publisher_listener::DdsPublisherListener,
};

pub struct DdsPublisher {
    qos: PublisherQos,
    rtps_group: RtpsGroup,
    data_writer_list: HashMap<InstanceHandle, Actor<DdsDataWriter>>,
    enabled: bool,
    user_defined_data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
    listener: Option<Actor<DdsPublisherListener>>,
    status_kind: Vec<StatusKind>,
}

impl DdsPublisher {
    pub fn new(
        qos: PublisherQos,
        rtps_group: RtpsGroup,
        listener: Option<Actor<DdsPublisherListener>>,
        status_kind: Vec<StatusKind>,
    ) -> Self {
        Self {
            qos,
            rtps_group,
            data_writer_list: HashMap::new(),
            enabled: false,
            user_defined_data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
            listener,
            status_kind,
        }
    }
}

#[actor_interface]
impl DdsPublisher {
    #[allow(clippy::too_many_arguments)]
    async fn create_datawriter(
        &mut self,
        type_name: String,
        topic_name: String,
        has_key: bool,
        data_max_size_serialized: usize,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Actor<DdsDataWriterListener>>,
        mask: Vec<StatusKind>,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
    ) -> DdsResult<ActorAddress<DdsDataWriter>> {
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
            self.get_unique_writer_id().await,
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
            Duration::new(0, 200_000_000),
            DURATION_ZERO,
            DURATION_ZERO,
            data_max_size_serialized,
        );

        let data_writer = DdsDataWriter::new(
            rtps_writer_impl,
            type_name,
            topic_name,
            a_listener,
            mask,
            qos,
        );
        let data_writer_actor = spawn_actor(data_writer);
        let data_writer_address = data_writer_actor.address();
        self.data_writer_list.insert(guid.into(), data_writer_actor);

        Ok(data_writer_address)
    }

    async fn lookup_datawriter(&self, topic_name: String) -> Option<ActorAddress<DdsDataWriter>> {
        for dw in self.data_writer_list.values() {
            if dw
                .send_mail_and_await_reply(dds_data_writer::get_topic_name::new())
                .await
                == topic_name
            {
                return Some(dw.address());
            }
        }
        None
    }

    async fn enable(&mut self) {
        self.enabled = true;
    }

    async fn is_enabled(&self) -> bool {
        self.enabled
    }

    async fn is_empty(&self) -> bool {
        self.data_writer_list.is_empty()
    }

    async fn get_unique_writer_id(&mut self) -> u8 {
        let counter = self.user_defined_data_writer_counter;
        self.user_defined_data_writer_counter += 1;
        counter
    }

    async fn delete_contained_entities(&mut self) {
        self.data_writer_list.clear()
    }

    async fn datawriter_add(
        &mut self,
        instance_handle: InstanceHandle,
        data_writer: Actor<DdsDataWriter>,
    ) {
        self.data_writer_list.insert(instance_handle, data_writer);
    }

    async fn datawriter_delete(&mut self, handle: InstanceHandle) {
        self.data_writer_list.remove(&handle);
    }

    async fn set_default_datawriter_qos(&mut self, qos: DataWriterQos) {
        self.default_datawriter_qos = qos;
    }

    async fn get_default_datawriter_qos(&self) -> DataWriterQos {
        self.default_datawriter_qos.clone()
    }

    async fn set_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
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

    async fn guid(&self) -> Guid {
        self.rtps_group.guid()
    }

    async fn get_instance_handle(&self) -> InstanceHandle {
        self.rtps_group.guid().into()
    }

    async fn get_status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }

    async fn get_listener(&self) -> Option<ActorAddress<DdsPublisherListener>> {
        self.listener.as_ref().map(|l| l.address())
    }

    async fn get_qos(&self) -> PublisherQos {
        self.qos.clone()
    }

    async fn data_writer_list(&self) -> Vec<ActorAddress<DdsDataWriter>> {
        self.data_writer_list
            .values()
            .map(|x| x.address())
            .collect()
    }

    async fn process_rtps_message(&self, message: RtpsMessageRead) {
        for data_writer_address in self.data_writer_list.values().map(|a| a.address()) {
            data_writer_address
                .send_mail(dds_data_writer::process_rtps_message::new(message.clone()))
                .await
                .expect("Should not fail to send command");
        }
    }

    async fn send_message(
        &self,
        header: RtpsMessageHeader,
        udp_transport_write: ActorAddress<UdpTransportWrite>,
        now: Time,
    ) {
        for data_writer_address in self.data_writer_list.values() {
            data_writer_address
                .send_mail(dds_data_writer::send_message::new(
                    header,
                    udp_transport_write.clone(),
                    now,
                ))
                .await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn add_matched_reader(
        &self,
        discovered_reader_data: DiscoveredReaderData,
        default_unicast_locator_list: Vec<Locator>,
        default_multicast_locator_list: Vec<Locator>,
        publisher_address: ActorAddress<DdsPublisher>,
        participant_address: ActorAddress<DdsDomainParticipant>,
        participant_publication_matched_listener: Option<
            ActorAddress<DdsDomainParticipantListener>,
        >,
        offered_incompatible_qos_participant_listener: Option<
            ActorAddress<DdsDomainParticipantListener>,
        >,
    ) {
        if self.is_partition_matched(
            discovered_reader_data
                .subscription_builtin_topic_data()
                .partition(),
        ) {
            for data_writer in self.data_writer_list.values() {
                let data_writer_address = data_writer.address();
                let publisher_publication_matched_listener =
                    if self.status_kind.contains(&StatusKind::PublicationMatched) {
                        self.listener.as_ref().map(|l| l.address())
                    } else {
                        None
                    };
                let offered_incompatible_qos_publisher_listener = if self
                    .status_kind
                    .contains(&StatusKind::OfferedIncompatibleQos)
                {
                    self.listener.as_ref().map(|l| l.address())
                } else {
                    None
                };
                data_writer
                    .send_mail_and_await_reply(dds_data_writer::add_matched_reader::new(
                        discovered_reader_data.clone(),
                        default_unicast_locator_list.clone(),
                        default_multicast_locator_list.clone(),
                        data_writer_address,
                        publisher_address.clone(),
                        participant_address.clone(),
                        self.qos.clone(),
                        publisher_publication_matched_listener,
                        participant_publication_matched_listener.clone(),
                        offered_incompatible_qos_publisher_listener,
                        offered_incompatible_qos_participant_listener.clone(),
                    ))
                    .await;
            }
        }
    }

    async fn remove_matched_reader(
        &self,
        discovered_reader_handle: InstanceHandle,
        publisher_address: ActorAddress<DdsPublisher>,
        participant_address: ActorAddress<DdsDomainParticipant>,
        participant_publication_matched_listener: Option<
            ActorAddress<DdsDomainParticipantListener>,
        >,
    ) {
        for data_writer in self.data_writer_list.values() {
            let data_writer_address = data_writer.address();
            let publisher_publication_matched_listener =
                if self.status_kind.contains(&StatusKind::PublicationMatched) {
                    self.listener.as_ref().map(|l| l.address())
                } else {
                    None
                };
            data_writer
                .send_mail_and_await_reply(dds_data_writer::remove_matched_reader::new(
                    discovered_reader_handle,
                    data_writer_address,
                    publisher_address.clone(),
                    participant_address.clone(),
                    publisher_publication_matched_listener,
                    participant_publication_matched_listener.clone(),
                ))
                .await;
        }
    }
}

impl DdsPublisher {
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
