use super::domain_participant::DomainParticipant;
use crate::{
    configuration::DustDdsConfiguration,
    domain::domain_participant_listener::DomainParticipantListener,
    implementation::{
        actors::{
            domain_participant_actor::{self, DomainParticipantActor},
            domain_participant_factory_actor::{self, DomainParticipantFactoryActor},
        },
        rtps::{
            message_receiver::MessageReceiver,
            messages::overall_structure::RtpsSubmessageReadKind,
            participant::RtpsParticipant,
            types::{Locator, LOCATOR_KIND_UDP_V4, PROTOCOLVERSION, VENDOR_ID_S2E},
        },
        rtps_udp_psm::udp_transport::{UdpTransportRead, UdpTransportWrite},
        utils::actor::{spawn_actor, Actor, THE_RUNTIME},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};
use lazy_static::lazy_static;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, RwLock},
};
use tracing::warn;

pub type DomainId = i32;

lazy_static! {
    /// This value can be used as an alias for the singleton factory returned by the operation
    /// [`DomainParticipantFactory::get_instance()`].
    pub static ref THE_PARTICIPANT_FACTORY: DomainParticipantFactory = {
        let participant_factory_actor = spawn_actor(DomainParticipantFactoryActor::new());
        DomainParticipantFactory(participant_factory_actor)
    };

    static ref THE_DDS_CONFIGURATION: RwLock<DustDdsConfiguration> = RwLock::new(DustDdsConfiguration::default());

}

/// The sole purpose of this class is to allow the creation and destruction of [`DomainParticipant`] objects.
/// [`DomainParticipantFactory`] itself has no factory. It is a pre-existing singleton object that can be accessed by means of the
/// [`DomainParticipantFactory::get_instance`] operation.
pub struct DomainParticipantFactory(Actor<DomainParticipantFactoryActor>);

impl DomainParticipantFactory {
    /// This operation creates a new [`DomainParticipant`] object. The [`DomainParticipant`] signifies that the calling application intends
    /// to join the Domain identified by the `domain_id` argument.
    /// If the specified QoS policies are not consistent, the operation will fail and no [`DomainParticipant`] will be created.
    /// The value [`QosKind::Default`] can be used to indicate that the [`DomainParticipant`] should be created
    /// with the default DomainParticipant QoS set in the factory. The use of this value is equivalent to the application obtaining the
    /// default DomainParticipant QoS by means of the operation [`DomainParticipantFactory::get_default_participant_qos`] and using the resulting
    /// QoS to create the [`DomainParticipant`].
    #[tracing::instrument(skip(self, a_listener))]
    pub fn create_participant(
        &self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: impl DomainParticipantListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.0.address().send_mail_and_await_reply_blocking(
                domain_participant_factory_actor::get_default_participant_qos::new(),
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
                domain_participant_factory_actor::get_unique_participant_id::new(),
            )?
            .to_ne_bytes();

        #[rustfmt::skip]
        let guid_prefix = [
            mac_address_octets[2],  mac_address_octets[3], mac_address_octets[4], mac_address_octets[5], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
        ];

        let interface_address_list =
            get_interface_address_list(THE_DDS_CONFIGURATION.read().unwrap().interface_name());

        let default_unicast_socket =
            socket2::Socket::new(socket2::Domain::IPV4, socket2::Type::DGRAM, None).map_err(
                |_| DdsError::Error("Failed to create default unicast socket".to_string()),
            )?;
        default_unicast_socket
            .bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)).into())
            .map_err(|_| DdsError::Error("Failed to bind to default unicast socket".to_string()))?;
        default_unicast_socket
            .set_nonblocking(true)
            .map_err(|_| DdsError::Error("Failed to set socket non-blocking".to_string()))?;
        if let Some(buffer_size) = THE_DDS_CONFIGURATION
            .read()
            .unwrap()
            .udp_receive_buffer_size()
        {
            default_unicast_socket
                .set_recv_buffer_size(buffer_size)
                .map_err(|_| {
                    DdsError::Error(
                        "Failed to set default unicast socket receive buffer size".to_string(),
                    )
                })?;
        }
        let default_unicast_socket = std::net::UdpSocket::from(default_unicast_socket);

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
        let udp_transport_write = Arc::new(UdpTransportWrite::new(socket));

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

        let listener = Box::new(a_listener);
        let status_kind = mask.to_vec();

        let domain_participant = DomainParticipantActor::new(
            rtps_participant,
            domain_id,
            THE_DDS_CONFIGURATION
                .read()
                .unwrap()
                .domain_tag()
                .to_string(),
            domain_participant_qos,
            &spdp_discovery_locator_list,
            THE_DDS_CONFIGURATION.read().unwrap().fragment_size(),
            udp_transport_write,
            listener,
            status_kind,
        );

        let participant_actor = spawn_actor(domain_participant);
        let participant_address = participant_actor.address();
        self.0.address().send_mail_and_await_reply_blocking(
            domain_participant_factory_actor::add_participant::new(
                InstanceHandle::new(participant_guid.into()),
                participant_actor,
            ),
        )?;
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
                let mut message_receiver = MessageReceiver::new(&message);
                while let Some(submessage) = message_receiver.next() {
                    match submessage {
                        RtpsSubmessageReadKind::AckNack(ack_nack_submessage) => participant_address_clone
                            .send_mail_and_await_reply(
                            domain_participant_actor::process_metatraffic_acknack_submessage::new(
                                ack_nack_submessage,
                                message_receiver.source_guid_prefix(),
                            ),
                        ).await.unwrap(),
                        RtpsSubmessageReadKind::Data(data_submessage) => participant_address_clone.send_mail_and_await_reply(domain_participant_actor::process_metatraffic_data_submessage::new(data_submessage, message_receiver.source_guid_prefix(), message_receiver.source_timestamp(), participant_address_clone.clone())).await.unwrap(),
                        RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => participant_address_clone.send_mail_and_await_reply(domain_participant_actor::process_metatraffic_data_frag_submessage::new(data_frag_submessage, message_receiver.source_guid_prefix(), message_receiver.source_timestamp(), participant_address_clone.clone())).await.unwrap(),
                        RtpsSubmessageReadKind::Gap(gap_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_gap_submessage::new(
                            gap_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_heartbeat_submessage::new(
                            heartbeat_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) =>  participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_heartbeat_frag_submessage::new(
                            heartbeat_frag_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::NackFrag(nack_frag_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_nack_frag_submessage::new(
                            nack_frag_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        _ => (),
                    }
                }

                let r = participant_address_clone
                    .send_mail_and_await_reply(
                        domain_participant_actor::process_builtin_discovery::new(
                            participant_address_clone.clone(),
                        ),
                    )
                    .await;
                if r.is_err() {
                    break;
                }
                let r = participant_address_clone
                    .send_mail(domain_participant_actor::send_message::new())
                    .await;
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
                let mut message_receiver = MessageReceiver::new(&message);
                while let Some(submessage) = message_receiver.next() {
                    match submessage {
                        RtpsSubmessageReadKind::AckNack(ack_nack_submessage) => participant_address_clone
                            .send_mail_and_await_reply(
                            domain_participant_actor::process_metatraffic_acknack_submessage::new(
                                ack_nack_submessage,
                                message_receiver.source_guid_prefix(),
                            ),
                        ).await.unwrap(),
                        RtpsSubmessageReadKind::Data(data_submessage) => participant_address_clone.send_mail_and_await_reply(domain_participant_actor::process_metatraffic_data_submessage::new(data_submessage, message_receiver.source_guid_prefix(), message_receiver.source_timestamp(), participant_address_clone.clone())).await.unwrap(),
                        RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => participant_address_clone.send_mail_and_await_reply(domain_participant_actor::process_metatraffic_data_frag_submessage::new(data_frag_submessage, message_receiver.source_guid_prefix(), message_receiver.source_timestamp(), participant_address_clone.clone())).await.unwrap(),
                        RtpsSubmessageReadKind::Gap(gap_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_gap_submessage::new(
                            gap_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_heartbeat_submessage::new(
                            heartbeat_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) =>  participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_heartbeat_frag_submessage::new(
                            heartbeat_frag_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::NackFrag(nack_frag_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_nack_frag_submessage::new(
                            nack_frag_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        _ => (),
                    }
                }
                participant_address_clone
                    .send_mail_and_await_reply(
                        domain_participant_actor::process_builtin_discovery::new(
                            participant_address_clone.clone(),
                        ),
                    )
                    .await.unwrap();

                participant_address_clone
                    .send_mail(domain_participant_actor::send_message::new())
                    .await.unwrap();


                    }
        });

        let participant_address_clone = participant_address;
        THE_RUNTIME.spawn(async move {
            let mut default_unicast_transport = UdpTransportRead::new(
                tokio::net::UdpSocket::from_std(default_unicast_socket)
                    .expect("Should not fail to open default unicast socket"),
            );

            while let Some((_locator, message)) = default_unicast_transport.read().await {
                let mut message_receiver = MessageReceiver::new(&message);
                while let Some(submessage) = message_receiver.next() {
                    match submessage {
                        RtpsSubmessageReadKind::AckNack(ack_nack_submessage) => participant_address_clone
                            .send_mail_and_await_reply(
                            domain_participant_actor::process_user_defined_acknack_submessage::new(
                                ack_nack_submessage,
                                message_receiver.source_guid_prefix(),
                            ),
                        ).await.unwrap(),
                        RtpsSubmessageReadKind::Data(data_submessage) => participant_address_clone.send_mail_and_await_reply(domain_participant_actor::process_user_defined_data_submessage::new(data_submessage, message_receiver.source_guid_prefix(), message_receiver.source_timestamp(), participant_address_clone.clone())).await.unwrap(),
                        RtpsSubmessageReadKind::DataFrag(data_frag_submessage) => participant_address_clone.send_mail_and_await_reply(domain_participant_actor::process_user_defined_data_frag_submessage::new(data_frag_submessage, message_receiver.source_guid_prefix(), message_receiver.source_timestamp(), participant_address_clone.clone())).await.unwrap(),
                        RtpsSubmessageReadKind::Gap(gap_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_user_defined_gap_submessage::new(
                            gap_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::Heartbeat(heartbeat_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_user_defined_heartbeat_submessage::new(
                            heartbeat_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::HeartbeatFrag(heartbeat_frag_submessage) =>  participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_user_defined_heartbeat_frag_submessage::new(
                            heartbeat_frag_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        RtpsSubmessageReadKind::NackFrag(nack_frag_submessage) => participant_address_clone
                        .send_mail_and_await_reply(
                        domain_participant_actor::process_user_defined_nack_frag_submessage::new(
                            nack_frag_submessage,
                            message_receiver.source_guid_prefix(),
                        )
                    ).await.unwrap(),
                        _ => (),
                    }
                }
                participant_address_clone
                        .send_mail(domain_participant_actor::send_message::new()).await.unwrap();
            }
        });

        if self
            .0
            .address()
            .send_mail_and_await_reply_blocking(domain_participant_factory_actor::get_qos::new())?
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
            domain_participant_factory_actor::get_participant_list::new(),
        )?;
        let participant = participant_list
            .iter()
            .find(|x| {
                if let Ok(h) = x.send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_instance_handle::new(),
                ) {
                    h == handle
                } else {
                    false
                }
            })
            .ok_or(DdsError::BadParameter)?;

        if participant
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_empty::new())?
        {
            self.0.address().send_mail_and_await_reply_blocking(
                domain_participant_factory_actor::delete_participant::new(handle),
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
                domain_participant_factory_actor::get_participant_list::new(),
            )?
            .iter()
            .find(|&a| {
                if let Ok(id) = a.send_mail_and_await_reply_blocking(
                    domain_participant_actor::get_domain_id::new(),
                ) {
                    id == domain_id
                } else {
                    false
                }
            })
            .cloned()
            .map(DomainParticipant::new))
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
            domain_participant_factory_actor::set_default_participant_qos::new(qos),
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
            domain_participant_factory_actor::get_default_participant_qos::new(),
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
            .send_mail_and_await_reply_blocking(domain_participant_factory_actor::set_qos::new(qos))
    }

    /// This operation returns the value of the [`DomainParticipantFactoryQos`] policies.
    #[tracing::instrument(skip(self))]
    pub fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        self.0
            .address()
            .send_mail_and_await_reply_blocking(domain_participant_factory_actor::get_qos::new())
    }
}

impl DomainParticipantFactory {
    /// Set the configuration of the [`DomainParticipantFactory`] singleton
    pub fn set_configuration(&self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        *THE_DDS_CONFIGURATION.write().unwrap() = configuration;
        Ok(())
    }

    /// Get the current configuration of the [`DomainParticipantFactory`] singleton
    pub fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        Ok(THE_DDS_CONFIGURATION.read().unwrap().clone())
    }
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
