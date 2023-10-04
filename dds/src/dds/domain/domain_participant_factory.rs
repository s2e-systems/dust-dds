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
            dds_data_reader::{self, DdsDataReader},
            dds_data_writer::{self, DdsDataWriter},
            dds_domain_participant::{
                self, DdsDomainParticipant, ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER,
                ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR,
                ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER, ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR,
            },
            dds_domain_participant_factory::{self, DdsDomainParticipantFactory},
            dds_domain_participant_listener::DdsDomainParticipantListener,
            dds_publisher, dds_subscriber, dds_topic,
            nodes::DomainParticipantNode,
        },
        rtps::{
            discovery_types::BuiltinEndpointSet,
            messages::overall_structure::RtpsMessageRead,
            participant::RtpsParticipant,
            reader_proxy::RtpsReaderProxy,
            types::{
                Guid, Locator, ReliabilityKind, SequenceNumber, ENTITYID_PARTICIPANT,
                ENTITYID_UNKNOWN, LOCATOR_KIND_UDP_V4, PROTOCOLVERSION, VENDOR_ID_S2E,
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
        qos_policy::PartitionQosPolicy,
        status::StatusKind,
    },
    subscription::{
        data_reader::Sample,
        sample_info::{
            InstanceStateKind, SampleStateKind, ANY_INSTANCE_STATE, ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
        },
    },
    topic_definition::type_support::{dds_deserialize_from_bytes, dds_serialize_key},
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
use tracing::warn;

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
    #[tracing::instrument(skip(self, a_listener), fields(with_listener = a_listener.is_some()))]
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.0.address().send_mail_and_await_reply_blocking(
                dds_domain_participant_factory::get_default_participant_qos::new(),
            )?,
            QosKind::Specific(q) => q,
        };

        let mac_address = NetworkInterface::show()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter_map(|i| i.mac_addr)
            .find(|m| m != "00:00:00:00:00:00")
            .expect("Could not find any mac address");
        let mut mac_address_octets = [0u8; 6];
        for (index, octet_str) in mac_address.split(|c| c == ':' || c == '-').enumerate() {
            mac_address_octets[index] =
                u8::from_str_radix(octet_str, 16).expect("All octet strings should be valid");
        }

        let app_id = std::process::id().to_ne_bytes();
        let instance_id = self
            .0
            .address()
            .send_mail_and_await_reply_blocking(
                dds_domain_participant_factory::get_unique_participant_id::new(),
            )?
            .to_ne_bytes();

        #[rustfmt::skip]
        let guid_prefix = [
            mac_address_octets[2],  mac_address_octets[3], mac_address_octets[4], mac_address_octets[5], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
        ];

        let interface_address_list =
            get_interface_address_list(THE_DDS_CONFIGURATION.interface_name.as_ref());

        let default_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0))).map_err(
                |_| DdsError::Error("Failed to bind to default unicast socket".to_string()),
            )?;
        default_unicast_socket
            .set_nonblocking(true)
            .map_err(|_| DdsError::Error("Failed to set socket non-blocking".to_string()))?;
        let user_defined_unicast_port = default_unicast_socket
            .local_addr()
            .map_err(|_| DdsError::Error("Failed to get socket address".to_string()))?
            .port();
        let user_defined_unicast_locator_port = user_defined_unicast_port.into();

        let default_unicast_locator_list: Vec<Locator> = interface_address_list
            .iter()
            .map(|a| Locator::new(LOCATOR_KIND_UDP_V4, user_defined_unicast_locator_port, *a))
            .collect();

        let default_multicast_locator_list = vec![];

        let metattrafic_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
                .map_err(|_| DdsError::Error("Failed to open metatraffic socket".to_string()))?;
        metattrafic_unicast_socket
            .set_nonblocking(true)
            .map_err(|_| {
                DdsError::Error("Failed to set metatraffic socket non-blocking".to_string())
            })?;

        let metattrafic_unicast_locator_port = metattrafic_unicast_socket
            .local_addr()
            .map_err(|_| DdsError::Error("Failed to get metatraffic socket address".to_string()))?
            .port()
            .into();
        let metatraffic_unicast_locator_list: Vec<Locator> = interface_address_list
            .iter()
            .map(|a| Locator::new(LOCATOR_KIND_UDP_V4, metattrafic_unicast_locator_port, *a))
            .collect();

        let metatraffic_multicast_locator_list = vec![Locator::new(
            LOCATOR_KIND_UDP_V4,
            port_builtin_multicast(domain_id) as u32,
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
        let participant_guid = rtps_participant.guid();

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
        let participant_address = participant_actor.address().clone();
        self.0.address().send_mail_and_await_reply_blocking(
            dds_domain_participant_factory::add_participant::new(
                participant_guid.into(),
                participant_actor,
            ),
        )?;
        let domain_participant =
            DomainParticipant::new(DomainParticipantNode::new(participant_address.clone()));

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
                let r = process_spdp_metatraffic(&participant_address_clone, message).await;

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
                let r: DdsResult<()> = async {
                    process_sedp_metatraffic(&participant_address_clone, message).await?;
                    process_sedp_discovery(&participant_address_clone).await?;
                    Ok(())
                }
                .await;

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
                let r = participant_address_clone
                    .send_mail(
                        dds_domain_participant::process_user_defined_rtps_message::new(
                            message,
                            participant_address_clone.clone(),
                        ),
                    )
                    .await;

                if r.is_err() {
                    break;
                }
            }
        });

        if self
            .0
            .address()
            .send_mail_and_await_reply_blocking(dds_domain_participant_factory::get_qos::new())?
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
    #[tracing::instrument(skip(self, participant))]
    pub fn delete_participant(&self, participant: &DomainParticipant) -> DdsResult<()> {
        let handle = participant.get_instance_handle()?;
        let participant_list = self.0.address().send_mail_and_await_reply_blocking(
            dds_domain_participant_factory::get_participant_list::new(),
        )?;
        let participant = participant_list
            .iter()
            .find(|x| {
                if let Ok(h) = x.send_mail_and_await_reply_blocking(
                    dds_domain_participant::get_instance_handle::new(),
                ) {
                    h == handle
                } else {
                    false
                }
            })
            .ok_or(DdsError::BadParameter)?;

        if participant.send_mail_and_await_reply_blocking(dds_domain_participant::is_empty::new())?
        {
            self.0.address().send_mail_and_await_reply_blocking(
                dds_domain_participant_factory::delete_participant::new(handle),
            )?;
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
    #[tracing::instrument]
    pub fn get_instance() -> &'static Self {
        &THE_PARTICIPANT_FACTORY
    }

    /// This operation retrieves a previously created [`DomainParticipant`] belonging to the specified domain_id. If no such
    /// [`DomainParticipant`] exists, the operation will return a [`None`] value.
    /// If multiple [`DomainParticipant`] entities belonging to that domain_id exist, then the operation will return one of them. It is not
    /// specified which one.
    #[tracing::instrument(skip(self))]
    pub fn lookup_participant(&self, domain_id: DomainId) -> DdsResult<Option<DomainParticipant>> {
        Ok(self
            .0
            .address()
            .send_mail_and_await_reply_blocking(
                dds_domain_participant_factory::get_participant_list::new(),
            )?
            .iter()
            .find(|&a| {
                if let Ok(id) =
                    a.send_mail_and_await_reply_blocking(
                        dds_domain_participant::get_domain_id::new(),
                    )
                {
                    id == domain_id
                } else {
                    false
                }
            })
            .map(|dp| DomainParticipant::new(DomainParticipantNode::new(dp.clone()))))
    }

    /// This operation sets a default value of the [`DomainParticipantQos`] policies which will be used for newly created
    /// [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`] operation.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    #[tracing::instrument(skip(self))]
    pub fn set_default_participant_qos(&self, qos: QosKind<DomainParticipantQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.0.address().send_mail_and_await_reply_blocking(
            dds_domain_participant_factory::set_default_participant_qos::new(qos),
        )
    }

    /// This operation retrieves the default value of the [`DomainParticipantQos`], that is, the QoS policies which will be used for
    /// newly created [`DomainParticipant`] entities in the case where the QoS policies are defaulted in the [`DomainParticipantFactory::create_participant`]
    /// operation.
    /// The values retrieved by [`DomainParticipantFactory::get_default_participant_qos`] will match the set of values specified on the last successful call to
    /// [`DomainParticipantFactory::set_default_participant_qos`], or else, if the call was never made, the default value of [`DomainParticipantQos`].
    #[tracing::instrument(skip(self))]
    pub fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        self.0.address().send_mail_and_await_reply_blocking(
            dds_domain_participant_factory::get_default_participant_qos::new(),
        )
    }

    /// This operation sets the value of the [`DomainParticipantFactoryQos`] policies. These policies control the behavior of the object
    /// a factory for entities.
    /// Note that despite having QoS, the [`DomainParticipantFactory`] is not an Entity.
    /// This operation will check that the resulting policies are self consistent; if they are not, the operation will have no effect and
    /// return a [`DdsError::InconsistentPolicy`].
    #[tracing::instrument(skip(self))]
    pub fn set_qos(&self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.0
            .address()
            .send_mail_and_await_reply_blocking(dds_domain_participant_factory::set_qos::new(qos))
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    #[tracing::instrument(skip(self))]
    pub fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        self.0
            .address()
            .send_mail_and_await_reply_blocking(dds_domain_participant_factory::get_qos::new())
    }
}

async fn lookup_data_writer_by_topic_name(
    writer_list: &[ActorAddress<DdsDataWriter>],
    topic_name: &str,
) -> Option<ActorAddress<DdsDataWriter>> {
    for data_writer in writer_list {
        if let Ok(t) = data_writer
            .send_mail_and_await_reply(dds_data_writer::get_topic_name::new())
            .await
        {
            if t == topic_name {
                return Some(data_writer.clone());
            }
        }
    }
    None
}

async fn lookup_data_reader_by_topic_name(
    stateful_reader_list: &[ActorAddress<DdsDataReader>],
    topic_name: &str,
) -> Option<ActorAddress<DdsDataReader>> {
    for data_reader in stateful_reader_list {
        if data_reader
            .send_mail_and_await_reply(dds_data_reader::get_topic_name::new())
            .await
            == Ok(topic_name.to_string())
        {
            return Some(data_reader.clone());
        }
    }
    None
}

async fn add_matched_publications_detector(
    writer: &ActorAddress<DdsDataWriter>,
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
            SequenceNumber::from(0),
        );
        writer
            .send_mail_and_await_reply(dds_data_writer::matched_reader_add::new(proxy))
            .await
            .unwrap();
    }
}

async fn add_matched_publications_announcer(
    reader: &ActorAddress<DdsDataReader>,
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

        reader
            .send_mail_and_await_reply(dds_data_reader::matched_writer_add::new(proxy))
            .await
            .unwrap();
    }
}

async fn add_matched_subscriptions_detector(
    writer: &ActorAddress<DdsDataWriter>,
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
            SequenceNumber::from(0),
        );
        writer
            .send_mail_and_await_reply(dds_data_writer::matched_reader_add::new(proxy))
            .await
            .unwrap();
    }
}

async fn add_matched_subscriptions_announcer(
    reader: &ActorAddress<DdsDataReader>,
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
        reader
            .send_mail_and_await_reply(dds_data_reader::matched_writer_add::new(proxy))
            .await
            .unwrap();
    }
}

async fn add_matched_topics_detector(
    writer: &ActorAddress<DdsDataWriter>,
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
            SequenceNumber::from(0),
        );
        writer
            .send_mail_and_await_reply(dds_data_writer::matched_reader_add::new(proxy))
            .await
            .unwrap();
    }
}

async fn add_matched_topics_announcer(
    reader: &ActorAddress<DdsDataReader>,
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
        reader
            .send_mail_and_await_reply(dds_data_reader::matched_writer_add::new(proxy))
            .await
            .unwrap();
    }
}

async fn process_spdp_metatraffic(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    message: RtpsMessageRead,
) -> DdsResult<()> {
    let builtin_subscriber = participant_address
        .send_mail_and_await_reply(dds_domain_participant::get_built_in_subscriber::new())
        .await?;

    let participant_mask_listener = (
        participant_address
            .send_mail_and_await_reply(dds_domain_participant::get_listener::new())
            .await?,
        participant_address
            .send_mail_and_await_reply(dds_domain_participant::get_status_kind::new())
            .await?,
    );

    // Receive the data on the builtin spdp reader
    builtin_subscriber
        .send_mail(dds_subscriber::process_rtps_message::new(
            message,
            participant_address
                .send_mail_and_await_reply(dds_domain_participant::get_current_time::new())
                .await?,
            participant_address.clone(),
            builtin_subscriber.clone(),
            participant_mask_listener,
        ))
        .await
        .expect("Should not fail to send command");

    let data_reader_list = builtin_subscriber
        .send_mail_and_await_reply(dds_subscriber::data_reader_list::new())
        .await?;
    for data_reader in data_reader_list {
        if data_reader
            .send_mail_and_await_reply(dds_data_reader::get_type_name::new())
            .await
            == Ok("SpdpDiscoveredParticipantData".to_string())
        {
            // Read data from each of the readers
            while let Ok(spdp_data_sample_list) = data_reader
                .send_mail_and_await_reply(dds_data_reader::read::new(
                    1,
                    vec![SampleStateKind::NotRead],
                    ANY_VIEW_STATE.to_vec(),
                    ANY_INSTANCE_STATE.to_vec(),
                    None,
                ))
                .await?
            {
                for (spdp_data_sample, _) in spdp_data_sample_list {
                    let discovered_participant_data =
                        dds_deserialize_from_bytes::<SpdpDiscoveredParticipantData>(
                            spdp_data_sample.expect("Should contain data").as_ref(),
                        )?;

                    // Check that the domainId of the discovered participant equals the local one.
                    // If it is not equal then there the local endpoints are not configured to
                    // communicate with the discovered participant.
                    // AND
                    // Check that the domainTag of the discovered participant equals the local one.
                    // If it is not equal then there the local endpoints are not configured to
                    // communicate with the discovered participant.
                    // IN CASE no domain id was transmitted the a local domain id is assumed
                    // (as specified in Table 9.19 - ParameterId mapping and default values)
                    let is_domain_id_matching = if let Some(domain_id) =
                        discovered_participant_data.participant_proxy().domain_id()
                    {
                        domain_id
                            == participant_address
                                .send_mail_and_await_reply(
                                    dds_domain_participant::get_domain_id::new(),
                                )
                                .await?
                    } else {
                        true
                    };
                    let is_domain_tag_matching =
                        discovered_participant_data.participant_proxy().domain_tag()
                            == participant_address
                                .send_mail_and_await_reply(
                                    dds_domain_participant::get_domain_tag::new(),
                                )
                                .await?;
                    let is_participant_ignored = participant_address
                        .send_mail_and_await_reply(
                            dds_domain_participant::is_participant_ignored::new(
                                dds_serialize_key(&discovered_participant_data)?.into(),
                            ),
                        )
                        .await?;

                    if is_domain_id_matching && is_domain_tag_matching && !is_participant_ignored {
                        // Process any new participant discovery (add/remove matched proxies)
                        let builtin_data_writer_list = participant_address
                            .send_mail_and_await_reply(
                                dds_domain_participant::get_builtin_publisher::new(),
                            )
                            .await?
                            .send_mail_and_await_reply(dds_publisher::data_writer_list::new())
                            .await?;
                        let builtin_data_reader_list = participant_address
                            .send_mail_and_await_reply(
                                dds_domain_participant::get_built_in_subscriber::new(),
                            )
                            .await?
                            .send_mail_and_await_reply(dds_subscriber::data_reader_list::new())
                            .await?;

                        if let Some(sedp_publications_announcer) = lookup_data_writer_by_topic_name(
                            &builtin_data_writer_list,
                            DCPS_PUBLICATION,
                        )
                        .await
                        {
                            add_matched_publications_detector(
                                &sedp_publications_announcer,
                                &discovered_participant_data,
                            )
                            .await;

                            participant_address
                                .send_mail_blocking(dds_domain_participant::send_message::new())?;
                        }

                        if let Some(sedp_publications_detector) = lookup_data_reader_by_topic_name(
                            &builtin_data_reader_list,
                            DCPS_PUBLICATION,
                        )
                        .await
                        {
                            add_matched_publications_announcer(
                                &sedp_publications_detector,
                                &discovered_participant_data,
                            )
                            .await;
                        }

                        if let Some(sedp_subscriptions_announcer) =
                            lookup_data_writer_by_topic_name(
                                &builtin_data_writer_list,
                                DCPS_SUBSCRIPTION,
                            )
                            .await
                        {
                            add_matched_subscriptions_detector(
                                &sedp_subscriptions_announcer,
                                &discovered_participant_data,
                            )
                            .await;

                            participant_address
                                .send_mail_blocking(dds_domain_participant::send_message::new())?;
                        }

                        if let Some(sedp_subscriptions_detector) = lookup_data_reader_by_topic_name(
                            &builtin_data_reader_list,
                            DCPS_SUBSCRIPTION,
                        )
                        .await
                        {
                            add_matched_subscriptions_announcer(
                                &sedp_subscriptions_detector,
                                &discovered_participant_data,
                            )
                            .await;
                        }

                        if let Some(sedp_topics_announcer) =
                            lookup_data_writer_by_topic_name(&builtin_data_writer_list, DCPS_TOPIC)
                                .await
                        {
                            add_matched_topics_detector(
                                &sedp_topics_announcer,
                                &discovered_participant_data,
                            )
                            .await;

                            participant_address
                                .send_mail_blocking(dds_domain_participant::send_message::new())?;
                        }

                        if let Some(sedp_topics_detector) =
                            lookup_data_reader_by_topic_name(&builtin_data_reader_list, DCPS_TOPIC)
                                .await
                        {
                            add_matched_topics_announcer(
                                &sedp_topics_detector,
                                &discovered_participant_data,
                            )
                            .await;
                        }

                        participant_address
                            .send_mail_and_await_reply(
                                dds_domain_participant::discovered_participant_add::new(
                                    dds_serialize_key(&discovered_participant_data)?.into(),
                                    discovered_participant_data,
                                ),
                            )
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_sedp_metatraffic(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    message: RtpsMessageRead,
) -> DdsResult<()> {
    let builtin_subscriber = participant_address
        .send_mail_and_await_reply(dds_domain_participant::get_built_in_subscriber::new())
        .await?;
    let builtin_publisher = participant_address
        .send_mail_and_await_reply(dds_domain_participant::get_builtin_publisher::new())
        .await?;
    let participant_mask_listener = (
        participant_address
            .send_mail_and_await_reply(dds_domain_participant::get_listener::new())
            .await?,
        participant_address
            .send_mail_and_await_reply(dds_domain_participant::get_status_kind::new())
            .await?,
    );

    for stateful_builtin_writer in builtin_publisher
        .send_mail_and_await_reply(dds_publisher::data_writer_list::new())
        .await?
    {
        stateful_builtin_writer
            .send_mail_and_await_reply(dds_data_writer::process_rtps_message::new(message.clone()))
            .await?;

        participant_address.send_mail_blocking(dds_domain_participant::send_message::new())?;
    }

    builtin_subscriber
        .send_mail_and_await_reply(dds_subscriber::process_rtps_message::new(
            message,
            participant_address
                .send_mail_and_await_reply(dds_domain_participant::get_current_time::new())
                .await?,
            participant_address.clone(),
            builtin_subscriber.clone(),
            participant_mask_listener,
        ))
        .await??;

    participant_address.send_mail_blocking(dds_domain_participant::send_message::new())?;

    Ok(())
}

async fn process_sedp_discovery(
    participant_address: &ActorAddress<DdsDomainParticipant>,
) -> DdsResult<()> {
    let builtin_subscriber = participant_address
        .send_mail_and_await_reply(dds_domain_participant::get_built_in_subscriber::new())
        .await?;

    for stateful_builtin_reader in builtin_subscriber
        .send_mail_and_await_reply(dds_subscriber::data_reader_list::new())
        .await?
    {
        match stateful_builtin_reader
            .send_mail_and_await_reply(dds_data_reader::get_topic_name::new())
            .await?
            .as_str()
        {
            DCPS_PUBLICATION => {
                //::<DiscoveredWriterData>
                if let Ok(mut discovered_writer_sample_list) = stateful_builtin_reader
                    .send_mail_and_await_reply(dds_data_reader::read::new(
                        i32::MAX,
                        ANY_SAMPLE_STATE.to_vec(),
                        ANY_VIEW_STATE.to_vec(),
                        ANY_INSTANCE_STATE.to_vec(),
                        None,
                    ))
                    .await?
                {
                    for (discovered_writer_data, discovered_writer_sample_info) in
                        discovered_writer_sample_list.drain(..)
                    {
                        let discovered_writer_sample =
                            Sample::new(discovered_writer_data, discovered_writer_sample_info);
                        discover_matched_writers(participant_address, &discovered_writer_sample)
                            .await?;
                    }
                }
            }
            DCPS_SUBSCRIPTION => {
                //::<DiscoveredReaderData>
                if let Ok(mut discovered_reader_sample_list) = stateful_builtin_reader
                    .send_mail_and_await_reply(dds_data_reader::read::new(
                        i32::MAX,
                        ANY_SAMPLE_STATE.to_vec(),
                        ANY_VIEW_STATE.to_vec(),
                        ANY_INSTANCE_STATE.to_vec(),
                        None,
                    ))
                    .await?
                {
                    for (discovered_reader_data, discovered_reader_sample_info) in
                        discovered_reader_sample_list.drain(..)
                    {
                        let discovered_reader_sample =
                            Sample::new(discovered_reader_data, discovered_reader_sample_info);
                        discover_matched_readers(participant_address, &discovered_reader_sample)
                            .await?;
                    }
                }
            }
            DCPS_TOPIC => {
                //::<DiscoveredTopicData>
                if let Ok(discovered_topic_sample_list) = stateful_builtin_reader
                    .send_mail_and_await_reply(dds_data_reader::read::new(
                        i32::MAX,
                        ANY_SAMPLE_STATE.to_vec(),
                        ANY_VIEW_STATE.to_vec(),
                        ANY_INSTANCE_STATE.to_vec(),
                        None,
                    ))
                    .await?
                {
                    for (discovered_topic_data, discovered_topic_sample_info) in
                        discovered_topic_sample_list
                    {
                        let discovered_topic_sample =
                            Sample::new(discovered_topic_data, discovered_topic_sample_info);
                        discover_matched_topics(participant_address, &discovered_topic_sample)
                            .await?;
                    }
                }
            }
            _ => (),
        };
    }

    Ok(())
}

async fn discover_matched_writers(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    discovered_writer_sample: &Sample<DiscoveredWriterData>,
) -> DdsResult<()> {
    let participant_mask_listener = (
        participant_address
            .send_mail_and_await_reply(dds_domain_participant::get_listener::new())
            .await?,
        participant_address
            .send_mail_and_await_reply(dds_domain_participant::get_status_kind::new())
            .await?,
    );
    match discovered_writer_sample.sample_info().instance_state {
        InstanceStateKind::Alive => {
            if let Some(discovered_writer_data) = discovered_writer_sample.data() {
                if !participant_address
                    .send_mail_and_await_reply(dds_domain_participant::is_publication_ignored::new(
                        discovered_writer_data
                            .writer_proxy()
                            .remote_writer_guid()
                            .into(),
                    ))
                    .await?
                {
                    let remote_writer_guid_prefix = discovered_writer_data
                        .writer_proxy()
                        .remote_writer_guid()
                        .prefix();
                    let writer_parent_participant_guid =
                        Guid::new(remote_writer_guid_prefix, ENTITYID_PARTICIPANT);

                    if let Some(spdp_discovered_participant_data) = participant_address
                        .send_mail_and_await_reply(
                            dds_domain_participant::discovered_participant_get::new(
                                InstanceHandle::from(writer_parent_participant_guid),
                            ),
                        )
                        .await?
                    {
                        let default_unicast_locator_list = spdp_discovered_participant_data
                            .participant_proxy()
                            .default_unicast_locator_list()
                            .to_vec();
                        let default_multicast_locator_list = spdp_discovered_participant_data
                            .participant_proxy()
                            .default_multicast_locator_list()
                            .to_vec();
                        for user_defined_subscriber_address in participant_address
                            .send_mail_and_await_reply(
                                dds_domain_participant::get_user_defined_subscriber_list::new(),
                            )
                            .await?
                        {
                            let subscriber_qos = user_defined_subscriber_address
                                .send_mail_and_await_reply(dds_subscriber::get_qos::new())
                                .await?;

                            if is_partition_matched(
                                discovered_writer_data.dds_publication_data().partition(),
                                &subscriber_qos.partition,
                            ) {
                                for data_reader_address in user_defined_subscriber_address
                                    .send_mail_and_await_reply(
                                        dds_subscriber::data_reader_list::new(),
                                    )
                                    .await?
                                {
                                    let subscriber_mask_listener = (
                                        user_defined_subscriber_address
                                            .send_mail_and_await_reply(
                                                dds_subscriber::get_listener::new(),
                                            )
                                            .await?,
                                        user_defined_subscriber_address
                                            .send_mail_and_await_reply(
                                                dds_subscriber::get_status_kind::new(),
                                            )
                                            .await?,
                                    );

                                    data_reader_address
                                        .send_mail_and_await_reply(
                                            dds_data_reader::add_matched_writer::new(
                                                discovered_writer_data.clone(),
                                                default_unicast_locator_list.clone(),
                                                default_multicast_locator_list.clone(),
                                                data_reader_address.clone(),
                                                user_defined_subscriber_address.clone(),
                                                participant_address.clone(),
                                                user_defined_subscriber_address
                                                    .send_mail_and_await_reply(
                                                        dds_subscriber::get_qos::new(),
                                                    )
                                                    .await?,
                                                subscriber_mask_listener.clone(),
                                                participant_mask_listener.clone(),
                                            ),
                                        )
                                        .await??;

                                    participant_address.send_mail_blocking(
                                        dds_domain_participant::send_message::new(),
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        InstanceStateKind::NotAliveDisposed => {
            for subscriber in participant_address
                .send_mail_and_await_reply(
                    dds_domain_participant::get_user_defined_subscriber_list::new(),
                )
                .await?
            {
                for data_reader in subscriber
                    .send_mail_and_await_reply(dds_subscriber::data_reader_list::new())
                    .await?
                {
                    let subscriber_mask_listener = (
                        subscriber
                            .send_mail_and_await_reply(dds_subscriber::get_listener::new())
                            .await?,
                        subscriber
                            .send_mail_and_await_reply(dds_subscriber::get_status_kind::new())
                            .await?,
                    );

                    data_reader
                        .send_mail_and_await_reply(dds_data_reader::remove_matched_writer::new(
                            discovered_writer_sample.sample_info().instance_handle,
                            data_reader.clone(),
                            subscriber.clone(),
                            participant_address.clone(),
                            subscriber_mask_listener.clone(),
                            participant_mask_listener.clone(),
                        ))
                        .await??;
                }
            }
        }
        InstanceStateKind::NotAliveNoWriters => todo!(),
    }

    Ok(())
}

pub fn is_partition_matched(lhs: &PartitionQosPolicy, rhs: &PartitionQosPolicy) -> bool {
    fn compare_regex_with_partition(
        regex: &PartitionQosPolicy,
        partition: &PartitionQosPolicy,
    ) -> bool {
        for partition_regex in regex.name.iter() {
            match glob_to_regex(partition_regex) {
                Ok(d) => {
                    let is_any_partition_matched = partition.name.iter().any(|p| d.is_match(p));
                    if is_any_partition_matched {
                        return true;
                    }
                }
                Err(e) => {
                    warn!(
                        "Received invalid partition regex name {:?}. Error {:?}",
                        partition_regex, e
                    );
                }
            }
        }

        false
    }

    fn compare_partition_names(lhs: &PartitionQosPolicy, rhs: &PartitionQosPolicy) -> bool {
        for partition_name in lhs.name.iter() {
            let is_any_partition_matched = rhs.name.contains(partition_name);
            if is_any_partition_matched {
                return true;
            }
        }

        false
    }

    (lhs.name.is_empty() && rhs.name.is_empty())
        || compare_regex_with_partition(lhs, rhs)
        || compare_regex_with_partition(rhs, lhs)
        || compare_partition_names(lhs, rhs)
}

pub async fn discover_matched_readers(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    discovered_reader_sample: &Sample<DiscoveredReaderData>,
) -> DdsResult<()> {
    match discovered_reader_sample.sample_info().instance_state {
        InstanceStateKind::Alive => {
            if let Some(discovered_reader_data) = discovered_reader_sample.data() {
                if !participant_address
                    .send_mail_and_await_reply(
                        dds_domain_participant::is_subscription_ignored::new(
                            discovered_reader_data
                                .reader_proxy()
                                .remote_reader_guid()
                                .into(),
                        ),
                    )
                    .await?
                {
                    let remote_reader_guid_prefix = discovered_reader_data
                        .reader_proxy()
                        .remote_reader_guid()
                        .prefix();
                    let reader_parent_participant_guid =
                        Guid::new(remote_reader_guid_prefix, ENTITYID_PARTICIPANT);

                    if let Some(spdp_discovered_participant_data) = participant_address
                        .send_mail_and_await_reply(
                            dds_domain_participant::discovered_participant_get::new(
                                InstanceHandle::from(reader_parent_participant_guid),
                            ),
                        )
                        .await?
                    {
                        let default_unicast_locator_list = spdp_discovered_participant_data
                            .participant_proxy()
                            .default_unicast_locator_list()
                            .to_vec();
                        let default_multicast_locator_list = spdp_discovered_participant_data
                            .participant_proxy()
                            .default_multicast_locator_list()
                            .to_vec();
                        for user_defined_publisher_address in participant_address
                            .send_mail_and_await_reply(
                                dds_domain_participant::get_user_defined_publisher_list::new(),
                            )
                            .await?
                        {
                            let publisher_qos = user_defined_publisher_address
                                .send_mail_and_await_reply(dds_publisher::get_qos::new())
                                .await?;

                            if is_partition_matched(
                                discovered_reader_data
                                    .subscription_builtin_topic_data()
                                    .partition(),
                                &publisher_qos.partition,
                            ) {
                                for data_writer in user_defined_publisher_address
                                    .send_mail_and_await_reply(
                                        dds_publisher::data_writer_list::new(),
                                    )
                                    .await?
                                {
                                    let publisher_publication_matched_listener =
                                        match user_defined_publisher_address
                                            .send_mail_and_await_reply(
                                                dds_publisher::get_listener::new(),
                                            )
                                            .await?
                                        {
                                            Some(l)
                                                if user_defined_publisher_address
                                                    .send_mail_and_await_reply(
                                                        dds_publisher::get_status_kind::new(),
                                                    )
                                                    .await?
                                                    .contains(&StatusKind::PublicationMatched) =>
                                            {
                                                Some(l)
                                            }
                                            _ => None,
                                        };

                                    let participant_publication_matched_listener =
                                        match participant_address
                                            .send_mail_and_await_reply(
                                                dds_domain_participant::get_listener::new(),
                                            )
                                            .await?
                                        {
                                            Some(l)
                                                if participant_address
                                                    .send_mail_and_await_reply(
                                                        dds_domain_participant::get_status_kind::new(),
                                                    )
                                                    .await?
                                                    .contains(&StatusKind::PublicationMatched) =>
                                            {
                                                Some(l)
                                            }
                                            _ => None,
                                        };
                                    let offered_incompatible_qos_publisher_listener =
                                        match user_defined_publisher_address
                                            .send_mail_and_await_reply(
                                                dds_publisher::get_listener::new(),
                                            )
                                            .await?
                                        {
                                            Some(l)
                                                if user_defined_publisher_address
                                                    .send_mail_and_await_reply(
                                                        dds_publisher::get_status_kind::new(),
                                                    )
                                                    .await?
                                                    .contains(
                                                        &StatusKind::OfferedIncompatibleQos,
                                                    ) =>
                                            {
                                                Some(l)
                                            }
                                            _ => None,
                                        };
                                    let offered_incompatible_qos_participant_listener =
                                        match participant_address
                                            .send_mail_and_await_reply(
                                                dds_domain_participant::get_listener::new(),
                                            )
                                            .await?
                                        {
                                            Some(l)
                                                if participant_address
                                                    .send_mail_and_await_reply(
                                                        dds_domain_participant::get_status_kind::new(),
                                                    )
                                                    .await?
                                                    .contains(
                                                        &StatusKind::OfferedIncompatibleQos,
                                                    ) =>
                                            {
                                                Some(l)
                                            }
                                            _ => None,
                                        };
                                    data_writer
                                        .send_mail_and_await_reply(
                                            dds_data_writer::add_matched_reader::new(
                                                discovered_reader_data.clone(),
                                                default_unicast_locator_list.clone(),
                                                default_multicast_locator_list.clone(),
                                                data_writer.clone(),
                                                user_defined_publisher_address.clone(),
                                                participant_address.clone(),
                                                publisher_qos.clone(),
                                                publisher_publication_matched_listener,
                                                participant_publication_matched_listener,
                                                offered_incompatible_qos_publisher_listener,
                                                offered_incompatible_qos_participant_listener,
                                            ),
                                        )
                                        .await??;

                                    participant_address.send_mail_blocking(
                                        dds_domain_participant::send_message::new(),
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }
        InstanceStateKind::NotAliveDisposed => {
            for publisher in participant_address
                .send_mail_and_await_reply(
                    dds_domain_participant::get_user_defined_publisher_list::new(),
                )
                .await?
            {
                for data_writer in publisher
                    .send_mail_and_await_reply(dds_publisher::data_writer_list::new())
                    .await?
                {
                    let publisher_publication_matched_listener = match publisher
                        .send_mail_and_await_reply(dds_publisher::get_listener::new())
                        .await?
                    {
                        Some(l)
                            if publisher
                                .send_mail_and_await_reply(dds_publisher::get_status_kind::new())
                                .await?
                                .contains(&StatusKind::PublicationMatched) =>
                        {
                            Some(l)
                        }
                        _ => None,
                    };
                    let participant_publication_matched_listener = match participant_address
                        .send_mail_and_await_reply(dds_domain_participant::get_listener::new())
                        .await?
                    {
                        Some(l)
                            if participant_address
                                .send_mail_and_await_reply(
                                    dds_domain_participant::get_status_kind::new(),
                                )
                                .await?
                                .contains(&StatusKind::PublicationMatched) =>
                        {
                            Some(l)
                        }
                        _ => None,
                    };
                    data_writer
                        .send_mail_and_await_reply(dds_data_writer::remove_matched_reader::new(
                            discovered_reader_sample.sample_info().instance_handle,
                            data_writer.clone(),
                            publisher.clone(),
                            participant_address.clone(),
                            publisher_publication_matched_listener,
                            participant_publication_matched_listener,
                        ))
                        .await??;
                }
            }
        }

        InstanceStateKind::NotAliveNoWriters => todo!(),
    }

    Ok(())
}

async fn discover_matched_topics(
    participant_address: &ActorAddress<DdsDomainParticipant>,
    discovered_topic_sample: &Sample<DiscoveredTopicData>,
) -> DdsResult<()> {
    match discovered_topic_sample.sample_info().instance_state {
        InstanceStateKind::Alive => {
            if let Some(topic_data) = discovered_topic_sample.data() {
                for topic in participant_address
                    .send_mail_and_await_reply(
                        dds_domain_participant::get_user_defined_topic_list::new(),
                    )
                    .await?
                {
                    topic
                        .send_mail_and_await_reply(dds_topic::process_discovered_topic::new(
                            topic_data.clone(),
                        ))
                        .await??;
                }

                participant_address
                    .send_mail_and_await_reply(dds_domain_participant::discovered_topic_add::new(
                        dds_serialize_key(&topic_data)?.into(),
                        topic_data.topic_builtin_topic_data().clone(),
                    ))
                    .await?;
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

type LocatorAddress = [u8; 16];
// As of 9.6.1.4.1  Default multicast address
const DEFAULT_MULTICAST_LOCATOR_ADDRESS: LocatorAddress =
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1];

const PB: i32 = 7400;
const DG: i32 = 250;
#[allow(non_upper_case_globals)]
const d0: i32 = 0;

fn port_builtin_multicast(domain_id: DomainId) -> u16 {
    (PB + DG * domain_id + d0) as u16
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
                    [0, 0, 0, 0,
                        0, 0, 0, 0,
                        0, 0, 0, 0,
                        v4.ip.octets()[0], v4.ip.octets()[1], v4.ip.octets()[2], v4.ip.octets()[3]]
                    ),
                _ => None,
            })
        })
        .collect()
}

fn get_multicast_socket(
    multicast_address: LocatorAddress,
    port: u16,
) -> std::io::Result<tokio::net::UdpSocket> {
    let socket_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));

    let socket = Socket::new(
        socket2::Domain::IPV4,
        socket2::Type::DGRAM,
        Some(socket2::Protocol::UDP),
    )?;

    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;

    socket.bind(&socket_addr.into())?;
    let addr = Ipv4Addr::new(
        multicast_address[12],
        multicast_address[13],
        multicast_address[14],
        multicast_address[15],
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
