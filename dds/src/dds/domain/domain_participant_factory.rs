use super::domain_participant::DomainParticipant;
use crate::{
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        configuration::DustDdsConfiguration,
        data_representation_builtin_endpoints::{
            discovered_reader_data::{DiscoveredReaderData, DCPS_SUBSCRIPTION},
            discovered_topic_data::{DiscoveredTopicData, DCPS_TOPIC},
            discovered_writer_data::{DiscoveredWriterData, DCPS_PUBLICATION},
            spdp_discovered_participant_data::SpdpDiscoveredParticipantData,
        },
        dds::{
            dds_data_reader::DdsDataReader,
            dds_data_writer::DdsDataWriter,
            dds_domain_participant::{
                DdsDomainParticipant, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            },
            dds_domain_participant_factory::DdsDomainParticipantFactory,
            dds_domain_participant_listener::DdsDomainParticipantListener,
        },
        rtps::{
            discovery_types::BuiltinEndpointSet,
            messages::overall_structure::{RtpsMessageHeader, RtpsMessageRead},
            participant::RtpsParticipant,
            reader_proxy::RtpsReaderProxy,
            stateful_reader::RtpsStatefulReader,
            stateful_writer::RtpsStatefulWriter,
            types::{
                DurabilityKind, Guid, GuidPrefix, Locator, LocatorAddress, LocatorPort,
                ReliabilityKind, ENTITYID_PARTICIPANT, ENTITYID_UNKNOWN, LOCATOR_KIND_UDP_V4,
                PROTOCOLVERSION, VENDOR_ID_S2E,
            },
            writer_proxy::RtpsWriterProxy,
        },
        rtps_udp_psm::udp_transport::{UdpTransportRead, UdpTransportWrite},
        utils::actor::{spawn_actor, Actor, ActorAddress, THE_RUNTIME},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{
            InstanceStateKind, SampleStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
        },
    },
    DdsType,
};
use fnmatch_regex::glob_to_regex;
use jsonschema::JSONSchema;
use lazy_static::lazy_static;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use schemars::schema_for;
use socket2::Socket;
use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

pub type DomainId = i32;

lazy_static! {
    /// This value can be used as an alias for the singleton factory returned by the operation
    /// [`DomainParticipantFactory::get_instance()`].
    pub static ref THE_PARTICIPANT_FACTORY: DomainParticipantFactory = {
        let participant_factory_actor = spawn_actor(DdsDomainParticipantFactory::new());
        DomainParticipantFactory(participant_factory_actor)
    };

    static ref THE_DDS_CONFIGURATION: DustDdsConfiguration =
        if let Ok(configuration_json) = std::env::var("DUST_DDS_CONFIGURATION") {
            configuration_try_from_str(configuration_json.as_str()).unwrap()
        } else {
            DustDdsConfiguration::default()
        };
}

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// [`DomainParticipantFactory`] itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory(Actor<DdsDomainParticipantFactory>);

impl DomainParticipantFactory {
    /// This operation creates a new [`DomainParticipant`] object. The [`DomainParticipant`] signifies that the calling application intends
    /// to join the Domain identified by the `domain_id` argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no [`DomainParticipant`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`DomainParticipant`] should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation [`DomainParticipantFactory::get_default_participant_qos`] and using the resulting
    /// QoS to create the [`DomainParticipant`].
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.0.address().get_default_participant_qos()?,
            QosKind::Specific(q) => q,
        };

        let mac_address = NetworkInterface::show()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter_map(|i| i.mac_addr)
            .next()
            .expect("Could not find any mac address");

        let app_id = std::process::id().to_ne_bytes();
        let instance_id = self.0.address().get_unique_participant_id()?.to_ne_bytes();

        #[rustfmt::skip]
        let guid_prefix = GuidPrefix::new([
            mac_address.as_bytes()[2],  mac_address.as_bytes()[3], mac_address.as_bytes()[4], mac_address.as_bytes()[5], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
        ]);

        let interface_address_list =
            get_interface_address_list(THE_DDS_CONFIGURATION.interface_name.as_ref());

        let default_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
                .map_err(|_| DdsError::Error)?;
        default_unicast_socket
            .set_nonblocking(true)
            .map_err(|_| DdsError::Error)?;
        let user_defined_unicast_port = default_unicast_socket
            .local_addr()
            .map_err(|_| DdsError::Error)?
            .port();
        let user_defined_unicast_locator_port = LocatorPort::new(user_defined_unicast_port.into());

        let default_unicast_locator_list: Vec<Locator> = interface_address_list
            .iter()
            .map(|a| Locator::new(LOCATOR_KIND_UDP_V4, user_defined_unicast_locator_port, *a))
            .collect();

        let default_multicast_locator_list = vec![];

        let metattrafic_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
                .map_err(|_| DdsError::Error)?;
        metattrafic_unicast_socket
            .set_nonblocking(true)
            .map_err(|_| DdsError::Error)?;

        let metattrafic_unicast_locator_port = LocatorPort::new(
            metattrafic_unicast_socket
                .local_addr()
                .map_err(|_| DdsError::Error)?
                .port()
                .into(),
        );
        let metatraffic_unicast_locator_list: Vec<Locator> = interface_address_list
            .iter()
            .map(|a| Locator::new(LOCATOR_KIND_UDP_V4, metattrafic_unicast_locator_port, *a))
            .collect();

        let metatraffic_multicast_locator_list = vec![Locator::new(
            LOCATOR_KIND_UDP_V4,
            port_builtin_multicast(domain_id),
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
        )];

        let spdp_discovery_locator_list = metatraffic_multicast_locator_list.clone();

        let socket = std::net::UdpSocket::bind("0.0.0.0:0000").unwrap();
        let udp_transport_write = spawn_actor(UdpTransportWrite::new(socket));

        let rtps_participant = RtpsParticipant::new(
            guid_prefix,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
        );

        let listener = a_listener.map(|l| spawn_actor(DdsDomainParticipantListener::new(l)));
        let status_kind = mask.to_vec();

        let domain_participant = DdsDomainParticipant::new(
            rtps_participant,
            domain_id,
            THE_DDS_CONFIGURATION.domain_tag.clone(),
            domain_participant_qos,
            &spdp_discovery_locator_list,
            THE_DDS_CONFIGURATION.fragment_size,
            udp_transport_write,
            listener,
            status_kind,
        );

        let participant_actor = spawn_actor(domain_participant);
        let participant_address = participant_actor.address();
        self.0.address().add_participant(participant_actor)?;
        let domain_participant = DomainParticipant::new(participant_address.clone());

        let participant_address_clone = participant_address.clone();
        THE_RUNTIME.spawn(async move {
            let mut metatraffic_multicast_transport = UdpTransportRead::new(
                get_multicast_socket(
                    DEFAULT_MULTICAST_LOCATOR_ADDRESS,
                    port_builtin_multicast(domain_id),
                )
                .expect("Should not fail to open socket"),
            );

            while let Some((_locator, message)) = metatraffic_multicast_transport.read().await {
                let r = tokio::task::block_in_place(|| {
                    process_spdp_metatraffic(&participant_address_clone, message)
                });

                if r.is_err() {
                    break;
                }
            }
        });

        let participant_address_clone = participant_address.clone();
        THE_RUNTIME.spawn(async move {
            let mut metatraffic_unicast_transport = UdpTransportRead::new(
                tokio::net::UdpSocket::from_std(metattrafic_unicast_socket)
                    .expect("Should not fail to open metatraffic unicast transport socket"),
            );

            while let Some((_locator, message)) = metatraffic_unicast_transport.read().await {
                let r: DdsResult<()> = tokio::task::block_in_place(|| {
                    process_sedp_metatraffic(&participant_address_clone, message)?;
                    process_sedp_discovery(&participant_address_clone)?;
                    Ok(())
                });

                if r.is_err() {
                    break;
                }
            }
        });

        let participant_address_clone = participant_address;
        THE_RUNTIME.spawn(async move {
            let mut default_unicast_transport = UdpTransportRead::new(
                tokio::net::UdpSocket::from_std(default_unicast_socket)
                    .expect("Should not fail to open default unicast socket"),
            );

            while let Some((_locator, message)) = default_unicast_transport.read().await {
                let r = tokio::task::block_in_place(|| {
                    process_user_defined_data(&participant_address_clone, message)
                });

                if r.is_err() {
                    break;
                }
            }
        });

        if self
            .0
            .address()
            .get_qos()?
            .entity_factory
            .autoenable_created_entities
        {
            domain_participant.enable()?;
        }

        Ok(domain_participant)
    }

    /// This operation deletes an existing [`DomainParticipant`]. This operation can only be invoked if all domain entities belonging to
    /// the participant have already been deleted otherwise the error [`DdsError::PreconditionNotMet`] is returned. If the
    /// participant has been previously deleted this operation returns the error [`DdsError::AlreadyDeleted`].
    pub fn delete_participant(&self, participant: &DomainParticipant) -> DdsResult<()> {
        let handle = participant.get_instance_handle()?;
        let participant_list = self.0.address().get_participant_list()?;
        let participant = participant_list
            .iter()
            .find(|x| {
                if let Ok(h) = x.get_instance_handle() {
                    h == handle
                } else {
                    false
                }
            })
            .ok_or(DdsError::BadParameter)?;

        if participant.is_empty()? {
            self.0.address().delete_participant(handle)?;
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    /// This operation returns the [`DomainParticipantFactory`] singleton. The operation is idempotent, that is, it can be called multiple
    /// times without side-effects and it will return the same [`DomainParticipantFactory`] instance.
    /// The pre-defined value [`struct@THE_PARTICIPANT_FACTORY`] can also be used as an alias for the singleton factory returned by this operation.
    pub fn get_instance() -> &'static Self {
        &THE_PARTICIPANT_FACTORY
    }

    /// This operation retrieves a previously created [`DomainParticipant`] belonging to the specified domain_id. If no such
    /// [`DomainParticipant`] exists, the operation will return a [`None`] value.
    /// If multiple [`DomainParticipant`] entities belonging to that domain_id exist, then the operation will return one of them. It is not
    /// specified which one.
    pub fn lookup_participant(&self, domain_id: DomainId) -> DdsResult<Option<DomainParticipant>> {
        Ok(self
            .0
            .address()
            .get_participant_list()?
            .iter()
            .find(|&a| {
                if let Ok(id) = a.get_domain_id() {
                    id == domain_id
                } else {
                    false
                }
            })
            .map(|dp| DomainParticipant::new(dp.clone())))
    }

    /// This operation sets a default value of the [`DomainParticipantQos`] policies which will be used for newly created
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_default_participant_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.0.address().set_default_participant_qos(qos)
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`]
    /// operation.
    /// The values retrieved by [`DomainParticipantFactory::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`DomainParticipantFactory::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    pub fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.0.address().get_default_participant_qos()
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    pub fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.0.address().set_qos(qos)
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    pub fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        self.0.address().get_qos()
    }
}

fn lookup_data_writer_by_topic_name(
    stateful_writer_list: &[ActorAddress<DdsDataWriter<RtpsStatefulWriter>>],
    topic_name: &str,
) -> Option<ActorAddress<DdsDataWriter<RtpsStatefulWriter>>> {
    stateful_writer_list
        .iter()
        .find(|dw| {
            if let Ok(t) = dw.get_topic_name() {
                t == topic_name
            } else {
                false
            }
        })
        .cloned()
}

fn lookup_data_reader_by_topic_name(
    stateful_reader_list: &[ActorAddress<DdsDataReader<RtpsStatefulReader>>],
    topic_name: &str,
) -> Option<ActorAddress<DdsDataReader<RtpsStatefulReader>>> {
    stateful_reader_list
        .iter()
        .find(|dw| {
            if let Ok(t) = dw.get_topic_name() {
                t == topic_name
            } else {
                false
            }
        })
        .cloned()
}

fn add_matched_publications_detector(
    writer: &ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_DETECTOR)
    {
        let remote_reader_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let expects_inline_qos = false;
        let proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            expects_inline_qos,
            true,
            ReliabilityKind::Reliable,
            DurabilityKind::TransientLocal,
        );
        writer.matched_reader_add(proxy).unwrap();
    }
}

fn add_matched_publications_announcer(
    reader: &ActorAddress<DdsDataReader<RtpsStatefulReader>>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_PUBLICATIONS_ANNOUNCER)
    {
        let remote_writer_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let data_max_size_serialized = None;

        let proxy = RtpsWriterProxy::new(
            remote_writer_guid,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            data_max_size_serialized,
            remote_group_entity_id,
        );

        reader.matched_writer_add(proxy).unwrap();
    }
}

fn add_matched_subscriptions_detector(
    writer: &ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_DETECTOR)
    {
        let remote_reader_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let expects_inline_qos = false;
        let proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            expects_inline_qos,
            true,
            ReliabilityKind::Reliable,
            DurabilityKind::TransientLocal,
        );
        writer.matched_reader_add(proxy).unwrap();
    }
}

fn add_matched_subscriptions_announcer(
    reader: &ActorAddress<DdsDataReader<RtpsStatefulReader>>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_SUBSCRIPTIONS_ANNOUNCER)
    {
        let remote_writer_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let data_max_size_serialized = None;

        let proxy = RtpsWriterProxy::new(
            remote_writer_guid,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            data_max_size_serialized,
            remote_group_entity_id,
        );
        reader.matched_writer_add(proxy).unwrap();
    }
}

fn add_matched_topics_detector(
    writer: &ActorAddress<DdsDataWriter<RtpsStatefulWriter>>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_DETECTOR)
    {
        let remote_reader_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let expects_inline_qos = false;
        let proxy = RtpsReaderProxy::new(
            remote_reader_guid,
            remote_group_entity_id,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            expects_inline_qos,
            true,
            ReliabilityKind::Reliable,
            DurabilityKind::TransientLocal,
        );
        writer.matched_reader_add(proxy).unwrap();
    }
}

fn add_matched_topics_announcer(
    reader: &ActorAddress<DdsDataReader<RtpsStatefulReader>>,
    discovered_participant_data: &SpdpDiscoveredParticipantData,
) {
    if discovered_participant_data
        .participant_proxy()
        .available_builtin_endpoints()
        .has(BuiltinEndpointSet::BUILTIN_ENDPOINT_TOPICS_ANNOUNCER)
    {
        let remote_writer_guid = Guid::new(
            discovered_participant_data
                .participant_proxy()
                .guid_prefix(),
            ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER,
        );
        let remote_group_entity_id = ENTITYID_UNKNOWN;
        let data_max_size_serialized = None;

        let proxy = RtpsWriterProxy::new(
            remote_writer_guid,
            discovered_participant_data
                .participant_proxy()
                .metatraffic_unicast_locator_list(),
            discovered_participant_data
                .participant_proxy()
                .metatraffic_multicast_locator_list(),
            data_max_size_serialized,
            remote_group_entity_id,
        );
        reader.matched_writer_add(proxy).unwrap();
    }
}

fn process_user_defined_data(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    message: RtpsMessageRead,
) -> DdsResult<()> {
    for user_defined_subscriber in participant_address.get_user_defined_subscriber_list()? {
        for user_defined_data_reader in user_defined_subscriber.stateful_data_reader_list()? {
            user_defined_data_reader.process_rtps_message(
                message.clone(),
                participant_address.get_current_time()?,
                user_defined_data_reader.clone(),
                user_defined_subscriber.clone(),
                participant_address.clone(),
            )?;
            user_defined_data_reader.send_message(
                RtpsMessageHeader::new(
                    participant_address.get_protocol_version()?,
                    participant_address.get_vendor_id()?,
                    participant_address.get_guid()?.prefix(),
                ),
                participant_address.get_udp_transport_write()?,
            )?;
        }
    }

    for user_defined_publisher in participant_address.get_user_defined_publisher_list()? {
        for user_defined_data_writer in user_defined_publisher.stateful_data_writer_list()? {
            user_defined_data_writer.process_rtps_message(message.clone())?;
            user_defined_data_writer.send_message(
                RtpsMessageHeader::new(
                    participant_address.get_protocol_version()?,
                    participant_address.get_vendor_id()?,
                    participant_address.get_guid()?.prefix(),
                ),
                participant_address.get_udp_transport_write()?,
                participant_address.get_current_time()?,
            )?;
        }
    }
    Ok(())
}

fn process_spdp_metatraffic(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    message: RtpsMessageRead,
) -> DdsResult<()> {
    let builtin_subscriber = participant_address.get_builtin_subscriber()?;

    if let Some(spdp_data_reader) = builtin_subscriber
        .stateless_data_reader_list()?
        .iter()
        .find(|dr| {
            if let Ok(type_name) = dr.get_type_name() {
                type_name == SpdpDiscoveredParticipantData::type_name()
            } else {
                false
            }
        })
    {
        // Receive the data on the builtin spdp reader
        spdp_data_reader.process_rtps_message(message, participant_address.get_current_time()?)?;

        // Read data from each of the readers
        while let Ok(spdp_data_sample_list) = spdp_data_reader
            .read::<SpdpDiscoveredParticipantData>(
                1,
                &[SampleStateKind::NotRead],
                ANY_VIEW_STATE,
                ANY_INSTANCE_STATE,
                None,
            )
        {
            for spdp_data_sample in spdp_data_sample_list {
                let discovered_participant_data =
                    spdp_data_sample.data.expect("Should contain data");

                // Check that the domainId of the discovered participant equals the local one.
                // If it is not equal then there the local endpoints are not configured to
                // communicate with the discovered participant.
                // AND
                // Check that the domainTag of the discovered participant equals the local one.
                // If it is not equal then there the local endpoints are not configured to
                // communicate with the discovered participant.
                let is_domain_id_matching =
                    discovered_participant_data.participant_proxy().domain_id()
                        == participant_address.get_domain_id()?;
                let is_domain_tag_matching =
                    discovered_participant_data.participant_proxy().domain_tag()
                        == participant_address.get_domain_tag()?;
                let is_participant_ignored = participant_address.is_participant_ignored(
                    discovered_participant_data.get_serialized_key().into(),
                )?;

                if is_domain_id_matching && is_domain_tag_matching && !is_participant_ignored {
                    // Process any new participant discovery (add/remove matched proxies)
                    let builtin_data_writer_list = participant_address
                        .get_builtin_publisher()?
                        .stateful_data_writer_list()?;
                    let builtin_data_reader_list = participant_address
                        .get_builtin_subscriber()?
                        .stateful_data_reader_list()?;

                    if let Some(sedp_publications_announcer) = lookup_data_writer_by_topic_name(
                        &builtin_data_writer_list,
                        DCPS_PUBLICATION,
                    ) {
                        add_matched_publications_detector(
                            &sedp_publications_announcer,
                            &discovered_participant_data,
                        );

                        sedp_publications_announcer.send_message(
                            RtpsMessageHeader::new(
                                participant_address.get_protocol_version()?,
                                participant_address.get_vendor_id()?,
                                participant_address.get_guid()?.prefix(),
                            ),
                            participant_address.get_udp_transport_write()?,
                            participant_address.get_current_time()?,
                        )?;
                    }

                    if let Some(sedp_publications_detector) = lookup_data_reader_by_topic_name(
                        &builtin_data_reader_list,
                        DCPS_PUBLICATION,
                    ) {
                        add_matched_publications_announcer(
                            &sedp_publications_detector,
                            &discovered_participant_data,
                        );
                    }

                    if let Some(sedp_subscriptions_announcer) = lookup_data_writer_by_topic_name(
                        &builtin_data_writer_list,
                        DCPS_SUBSCRIPTION,
                    ) {
                        add_matched_subscriptions_detector(
                            &sedp_subscriptions_announcer,
                            &discovered_participant_data,
                        );
                        sedp_subscriptions_announcer.send_message(
                            RtpsMessageHeader::new(
                                participant_address.get_protocol_version()?,
                                participant_address.get_vendor_id()?,
                                participant_address.get_guid()?.prefix(),
                            ),
                            participant_address.get_udp_transport_write()?,
                            participant_address.get_current_time()?,
                        )?;
                    }

                    if let Some(sedp_subscriptions_detector) = lookup_data_reader_by_topic_name(
                        &builtin_data_reader_list,
                        DCPS_SUBSCRIPTION,
                    ) {
                        add_matched_subscriptions_announcer(
                            &sedp_subscriptions_detector,
                            &discovered_participant_data,
                        );
                    }

                    if let Some(sedp_topics_announcer) =
                        lookup_data_writer_by_topic_name(&builtin_data_writer_list, DCPS_TOPIC)
                    {
                        add_matched_topics_detector(
                            &sedp_topics_announcer,
                            &discovered_participant_data,
                        );

                        sedp_topics_announcer.send_message(
                            RtpsMessageHeader::new(
                                participant_address.get_protocol_version()?,
                                participant_address.get_vendor_id()?,
                                participant_address.get_guid()?.prefix(),
                            ),
                            participant_address.get_udp_transport_write()?,
                            participant_address.get_current_time()?,
                        )?;
                    }

                    if let Some(sedp_topics_detector) =
                        lookup_data_reader_by_topic_name(&builtin_data_reader_list, DCPS_TOPIC)
                    {
                        add_matched_topics_announcer(
                            &sedp_topics_detector,
                            &discovered_participant_data,
                        );
                    }

                    participant_address.discovered_participant_add(
                        discovered_participant_data.get_serialized_key().into(),
                        discovered_participant_data,
                    )?;
                }
            }
        }
    }

    Ok(())
}

fn process_sedp_metatraffic(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    message: RtpsMessageRead,
) -> DdsResult<()> {
    let builtin_subscriber = participant_address.get_builtin_subscriber()?;
    let builtin_publisher = participant_address.get_builtin_publisher()?;

    for stateful_builtin_writer in builtin_publisher.stateful_data_writer_list()? {
        stateful_builtin_writer.process_rtps_message(message.clone())?;
        stateful_builtin_writer.send_message(
            RtpsMessageHeader::new(
                participant_address.get_protocol_version()?,
                participant_address.get_vendor_id()?,
                participant_address.get_guid()?.prefix(),
            ),
            participant_address.get_udp_transport_write()?,
            participant_address.get_current_time()?,
        )?;
    }

    for stateful_builtin_reader in builtin_subscriber.stateful_data_reader_list()? {
        stateful_builtin_reader.process_rtps_message(
            message.clone(),
            participant_address.get_current_time()?,
            stateful_builtin_reader.clone(),
            builtin_subscriber.clone(),
            participant_address.clone(),
        )?;
        stateful_builtin_reader.send_message(
            RtpsMessageHeader::new(
                participant_address.get_protocol_version()?,
                participant_address.get_vendor_id()?,
                participant_address.get_guid()?.prefix(),
            ),
            participant_address.get_udp_transport_write()?,
        )?;
    }

    Ok(())
}

fn process_sedp_discovery(
    participant_address: &ActorAddress<DdsDomainParticipant>,
) -> DdsResult<()> {
    let builtin_subscriber = participant_address.get_builtin_subscriber()?;

    for stateful_builtin_reader in builtin_subscriber.stateful_data_reader_list()? {
        match stateful_builtin_reader.get_topic_name()?.as_str() {
            DCPS_PUBLICATION => {
                if let Ok(mut discovered_writer_sample_list) = stateful_builtin_reader
                    .read::<DiscoveredWriterData>(
                    i32::MAX,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    ANY_INSTANCE_STATE,
                    None,
                ) {
                    for discovered_writer_sample in discovered_writer_sample_list.drain(..) {
                        discover_matched_writers(participant_address, &discovered_writer_sample)?;
                    }
                }
            }
            DCPS_SUBSCRIPTION => {
                if let Ok(mut discovered_reader_sample_list) = stateful_builtin_reader
                    .read::<DiscoveredReaderData>(
                    i32::MAX,
                    ANY_SAMPLE_STATE,
                    ANY_VIEW_STATE,
                    ANY_INSTANCE_STATE,
                    None,
                ) {
                    for discovered_reader_sample in discovered_reader_sample_list.drain(..) {
                        discover_matched_readers(participant_address, &discovered_reader_sample)?;
                    }
                }
            }
            DCPS_TOPIC => {
                if let Ok(discovered_topic_sample_list) = stateful_builtin_reader
                    .read::<DiscoveredTopicData>(
                        i32::MAX,
                        ANY_SAMPLE_STATE,
                        ANY_VIEW_STATE,
                        ANY_INSTANCE_STATE,
                        None,
                    )
                {
                    for discovered_topic_sample in discovered_topic_sample_list {
                        discover_matched_topics(participant_address, &discovered_topic_sample)?;
                    }
                }
            }
            _ => (),
        };
    }

    Ok(())
}

fn discover_matched_writers(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    discovered_writer_sample: &Sample<DiscoveredWriterData>,
) -> DdsResult<()> {
    match discovered_writer_sample.sample_info.instance_state {
        InstanceStateKind::Alive => {
            if let Some(discovered_writer_data) = &discovered_writer_sample.data {
                if !participant_address.is_publication_ignored(
                    discovered_writer_data
                        .writer_proxy()
                        .remote_writer_guid()
                        .into(),
                )? {
                    let remote_writer_guid_prefix = discovered_writer_data
                        .writer_proxy()
                        .remote_writer_guid()
                        .prefix();
                    let writer_parent_participant_guid =
                        Guid::new(remote_writer_guid_prefix, ENTITYID_PARTICIPANT);

                    if let Some(spdp_discovered_participant_data) = participant_address
                        .discovered_participant_get(InstanceHandle::from(
                            writer_parent_participant_guid,
                        ))?
                    {
                        let default_unicast_locator_list = spdp_discovered_participant_data
                            .participant_proxy()
                            .default_unicast_locator_list()
                            .to_vec();
                        let default_multicast_locator_list = spdp_discovered_participant_data
                            .participant_proxy()
                            .default_multicast_locator_list()
                            .to_vec();
                        for user_defined_subscriber_address in
                            participant_address.get_user_defined_subscriber_list()?
                        {
                            let is_discovered_writer_regex_matched_to_subscriber = if let Ok(d) =
                                glob_to_regex(
                                    discovered_writer_data
                                        .clone()
                                        .dds_publication_data()
                                        .partition()
                                        .name
                                        .as_str(),
                                ) {
                                d.is_match(
                                    &user_defined_subscriber_address.get_qos()?.partition.name,
                                )
                            } else {
                                false
                            };

                            let is_subscriber_regex_matched_to_discovered_writer = if let Ok(d) =
                                glob_to_regex(
                                    &user_defined_subscriber_address.get_qos()?.partition.name,
                                ) {
                                d.is_match(
                                    &discovered_writer_data
                                        .clone()
                                        .dds_publication_data()
                                        .partition()
                                        .name,
                                )
                            } else {
                                false
                            };

                            let is_partition_string_matched = discovered_writer_data
                                .clone()
                                .dds_publication_data()
                                .partition()
                                .name
                                == user_defined_subscriber_address.get_qos()?.partition.name;

                            if is_discovered_writer_regex_matched_to_subscriber
                                || is_subscriber_regex_matched_to_discovered_writer
                                || is_partition_string_matched
                            {
                                let user_defined_subscriber_qos =
                                    user_defined_subscriber_address.get_qos()?;
                                for data_reader_address in
                                    user_defined_subscriber_address.stateful_data_reader_list()?
                                {
                                    data_reader_address.add_matched_writer(
                                        discovered_writer_data.clone(),
                                        default_unicast_locator_list.clone(),
                                        default_multicast_locator_list.clone(),
                                        user_defined_subscriber_qos.clone(),
                                        data_reader_address.clone(),
                                        user_defined_subscriber_address.clone(),
                                        participant_address.clone(),
                                    )?;
                                    data_reader_address.send_message(
                                        RtpsMessageHeader::new(
                                            participant_address.get_protocol_version()?,
                                            participant_address.get_vendor_id()?,
                                            participant_address.get_guid()?.prefix(),
                                        ),
                                        participant_address.get_udp_transport_write()?,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        InstanceStateKind::NotAliveDisposed => {
            for subscriber in participant_address.get_user_defined_subscriber_list()? {
                for data_reader in subscriber.stateful_data_reader_list()? {
                    data_reader.remove_matched_writer(
                        discovered_writer_sample.sample_info.instance_handle,
                        data_reader.clone(),
                        subscriber.clone(),
                        participant_address.clone(),
                    )?;
                }
            }
        }
        InstanceStateKind::NotAliveNoWriters => todo!(),
    }

    Ok(())
}

pub fn discover_matched_readers(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    discovered_reader_sample: &Sample<DiscoveredReaderData>,
) -> DdsResult<()> {
    match discovered_reader_sample.sample_info.instance_state {
        InstanceStateKind::Alive => {
            if let Some(discovered_reader_data) = &discovered_reader_sample.data {
                if !participant_address.is_subscription_ignored(
                    discovered_reader_data
                        .reader_proxy()
                        .remote_reader_guid()
                        .into(),
                )? {
                    let remote_reader_guid_prefix = discovered_reader_data
                        .reader_proxy()
                        .remote_reader_guid()
                        .prefix();
                    let reader_parent_participant_guid =
                        Guid::new(remote_reader_guid_prefix, ENTITYID_PARTICIPANT);

                    if let Some(spdp_discovered_participant_data) = participant_address
                        .discovered_participant_get(InstanceHandle::from(
                            reader_parent_participant_guid,
                        ))?
                    {
                        let default_unicast_locator_list = spdp_discovered_participant_data
                            .participant_proxy()
                            .default_unicast_locator_list()
                            .to_vec();
                        let default_multicast_locator_list = spdp_discovered_participant_data
                            .participant_proxy()
                            .default_multicast_locator_list()
                            .to_vec();
                        for user_defined_publisher_address in
                            participant_address.get_user_defined_publisher_list()?
                        {
                            let publisher_qos = user_defined_publisher_address.get_qos()?;
                            let is_discovered_reader_regex_matched_to_publisher = if let Ok(d) =
                                glob_to_regex(
                                    &discovered_reader_data
                                        .subscription_builtin_topic_data()
                                        .partition()
                                        .name,
                                ) {
                                d.is_match(&publisher_qos.partition.name)
                            } else {
                                false
                            };

                            let is_publisher_regex_matched_to_discovered_reader =
                                if let Ok(d) = glob_to_regex(&publisher_qos.partition.name) {
                                    d.is_match(
                                        &discovered_reader_data
                                            .subscription_builtin_topic_data()
                                            .partition()
                                            .name,
                                    )
                                } else {
                                    false
                                };

                            let is_partition_string_matched = discovered_reader_data
                                .subscription_builtin_topic_data()
                                .partition()
                                .name
                                == publisher_qos.partition.name;

                            if is_discovered_reader_regex_matched_to_publisher
                                || is_publisher_regex_matched_to_discovered_reader
                                || is_partition_string_matched
                            {
                                for data_writer in
                                    user_defined_publisher_address.stateful_data_writer_list()?
                                {
                                    data_writer.add_matched_reader(
                                        discovered_reader_data.clone(),
                                        default_unicast_locator_list.clone(),
                                        default_multicast_locator_list.clone(),
                                        publisher_qos.clone(),
                                        data_writer.clone(),
                                        user_defined_publisher_address.clone(),
                                        participant_address.clone(),
                                    )?;
                                    data_writer.send_message(
                                        RtpsMessageHeader::new(
                                            participant_address.get_protocol_version()?,
                                            participant_address.get_vendor_id()?,
                                            participant_address.get_guid()?.prefix(),
                                        ),
                                        participant_address.get_udp_transport_write()?,
                                        participant_address.get_current_time()?,
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        InstanceStateKind::NotAliveDisposed => {
            for publisher in participant_address.get_user_defined_publisher_list()? {
                for data_writer in publisher.stateful_data_writer_list()? {
                    data_writer.remove_matched_reader(
                        discovered_reader_sample.sample_info.instance_handle,
                        data_writer.clone(),
                        publisher.clone(),
                        participant_address.clone(),
                    )?;
                }
            }
        }

        InstanceStateKind::NotAliveNoWriters => todo!(),
    }

    Ok(())
}

fn discover_matched_topics(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    discovered_topic_sample: &Sample<DiscoveredTopicData>,
) -> DdsResult<()> {
    match discovered_topic_sample.sample_info.instance_state {
        InstanceStateKind::Alive => {
            if let Some(topic_data) = discovered_topic_sample.data.as_ref() {
                for topic in participant_address.get_user_defined_topic_list()? {
                    topic.process_discovered_topic(topic_data.clone())?;
                }

                participant_address.discovered_topic_add(
                    topic_data.get_serialized_key().into(),
                    topic_data.topic_builtin_topic_data().clone(),
                )?;
            }
        }
        InstanceStateKind::NotAliveDisposed => todo!(),
        InstanceStateKind::NotAliveNoWriters => todo!(),
    }

    Ok(())
}

fn configuration_try_from_str(configuration_json: &str) -> Result<DustDdsConfiguration, String> {
    let root_schema = schema_for!(DustDdsConfiguration);
    let json_schema_str =
        serde_json::to_string(&root_schema).expect("Json schema could not be created");

    let schema = serde_json::value::Value::from_str(json_schema_str.as_str())
        .expect("Json schema not valid");
    let compiled_schema = JSONSchema::compile(&schema).expect("Json schema could not be compiled");

    let instance =
        serde_json::value::Value::from_str(configuration_json).map_err(|e| e.to_string())?;
    compiled_schema
        .validate(&instance)
        .map_err(|errors| errors.map(|e| e.to_string()).collect::<String>())?;
    serde_json::from_value(instance).map_err(|e| e.to_string())
}

// As of 9.6.1.4.1  Default multicast address
const DEFAULT_MULTICAST_LOCATOR_ADDRESS: LocatorAddress =
    LocatorAddress::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1]);

const PB: i32 = 7400;
const DG: i32 = 250;
#[allow(non_upper_case_globals)]
const d0: i32 = 0;

fn port_builtin_multicast(domain_id: DomainId) -> LocatorPort {
    LocatorPort::new((PB + DG * domain_id + d0) as u32)
}

fn get_interface_address_list(interface_name: Option<&String>) -> Vec<LocatorAddress> {
    NetworkInterface::show()
        .expect("Could not scan interfaces")
        .into_iter()
        .filter(|x| {
            if let Some(if_name) = interface_name {
                &x.name == if_name
            } else {
                true
            }
        })
        .flat_map(|i| {
            i.addr.into_iter().filter_map(|a| match a {
                #[rustfmt::skip]
                Addr::V4(v4) if !v4.ip.is_loopback() => Some(
                    LocatorAddress::new([0, 0, 0, 0,
                        0, 0, 0, 0,
                        0, 0, 0, 0,
                        v4.ip.octets()[0], v4.ip.octets()[1], v4.ip.octets()[2], v4.ip.octets()[3]])
                    ),
                _ => None,
            })
        })
        .collect()
}

fn get_multicast_socket(
    multicast_address: LocatorAddress,
    port: LocatorPort,
) -> std::io::Result<tokio::net::UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, <u32>::from(port) as u16));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;

    socket.bind(&socket_addr.into())?;
    let multicast_addr_bytes: [u8; 16] = multicast_address.into();
    let addr = Ipv4Addr::new(
        multicast_addr_bytes[12],
        multicast_addr_bytes[13],
        multicast_addr_bytes[14],
        multicast_addr_bytes[15],
    );
    socket.join_multicast_v4(&addr, &Ipv4Addr::UNSPECIFIED)?;
    socket.set_multicast_loop_v4(true)?;

    tokio::net::UdpSocket::from_std(socket.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_configuration_json() {
        let configuration = configuration_try_from_str(
            r#"{"domain_tag" : "from_configuration_json", "interface_name": "Wi-Fi"}"#,
        )
        .unwrap();
        assert_eq!(
            configuration,
            DustDdsConfiguration {
                domain_tag: "from_configuration_json".to_string(),
                interface_name: Some("Wi-Fi".to_string()),
                fragment_size: 1344
            }
        );
    }
}
