use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use tokio::runtime::Runtime;

use crate::{
    configuration::DustDdsConfiguration,
    domain::{
        domain_participant::DomainParticipant, domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        actors::domain_participant_actor,
        rtps::{
            participant::RtpsParticipant,
            types::{Locator, LOCATOR_KIND_UDP_V4, PROTOCOLVERSION, VENDOR_ID_S2E},
        },
        rtps_udp_psm::udp_transport::{UdpTransportRead, UdpTransportWrite},
        utils::actor::Actor,
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};

use super::domain_participant_actor::DomainParticipantActor;

pub struct DomainParticipantFactoryActor {
    domain_participant_list: HashMap<InstanceHandle, Actor<DomainParticipantActor>>,
    domain_participant_counter: u32,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
    runtime: Runtime,
}

impl Default for DomainParticipantFactoryActor {
    fn default() -> Self {
        Self::new()
    }
}

impl DomainParticipantFactoryActor {
    pub fn new() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_stack_size(4 * 1024 * 1024)
            .build()
            .expect("Failed to create Tokio runtime");
        Self {
            domain_participant_list: HashMap::new(),
            domain_participant_counter: 0,
            qos: DomainParticipantFactoryQos::default(),
            default_participant_qos: DomainParticipantQos::default(),
            configuration: DustDdsConfiguration::default(),
            runtime,
        }
    }

    pub fn create_participant(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: impl DomainParticipantListener + Send + 'static,
        mask: &[StatusKind],
    ) -> DdsResult<DomainParticipant> {
        // To allow creating actors
        let _runtime_guard = self.runtime.enter();

        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
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
        let instance_id = self.get_unique_participant_id().to_ne_bytes();

        #[rustfmt::skip]
        let guid_prefix = [
            mac_address_octets[2],  mac_address_octets[3], mac_address_octets[4], mac_address_octets[5], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
        ];

        let interface_address_list =
            get_interface_address_list(self.configuration.interface_name());

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
        if let Some(buffer_size) = self.configuration.udp_receive_buffer_size() {
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
            self.configuration.domain_tag().to_string(),
            domain_participant_qos,
            &spdp_discovery_locator_list,
            self.configuration.fragment_size(),
            udp_transport_write,
            listener,
            status_kind,
            self.runtime.handle(),
        );
        let participant_actor = Actor::spawn(domain_participant, self.runtime.handle());
        let participant_address = participant_actor.address();
        self.domain_participant_list.insert(
            InstanceHandle::new(participant_guid.into()),
            participant_actor,
        );

        let domain_participant = DomainParticipant::new(participant_address.clone());

        let participant_address_clone = participant_address.clone();
        self.runtime.spawn(async move {
            let mut metatraffic_multicast_transport = UdpTransportRead::new(
                get_multicast_socket(
                    DEFAULT_MULTICAST_LOCATOR_ADDRESS,
                    port_builtin_multicast(domain_id),
                )
                .expect("Should not fail to open socket"),
            );

            while let Some((_locator, message)) = metatraffic_multicast_transport.read().await {
                let r = participant_address_clone
                    .send_mail_and_await_reply(
                        domain_participant_actor::process_metatraffic_rtps_message::new(
                            message,
                            participant_address_clone.clone(),
                        ),
                    )
                    .await;
                if r.is_err() {
                    break;
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
        self.runtime.spawn(async move {
            let mut metatraffic_unicast_transport = UdpTransportRead::new(
                tokio::net::UdpSocket::from_std(metattrafic_unicast_socket)
                    .expect("Should not fail to open metatraffic unicast transport socket"),
            );

            while let Some((_locator, message)) = metatraffic_unicast_transport.read().await {
                let r: DdsResult<()> = async {
                    participant_address_clone
                        .send_mail_and_await_reply(
                            domain_participant_actor::process_metatraffic_rtps_message::new(
                                message,
                                participant_address_clone.clone(),
                            ),
                        )
                        .await??;
                    participant_address_clone
                        .send_mail_and_await_reply(
                            domain_participant_actor::process_builtin_discovery::new(
                                participant_address_clone.clone(),
                            ),
                        )
                        .await?;

                    participant_address_clone
                        .send_mail(domain_participant_actor::send_message::new())
                        .await?;
                    Ok(())
                }
                .await;

                if r.is_err() {
                    break;
                }
            }
        });

        let participant_address_clone = participant_address;
        self.runtime.spawn(async move {
            let mut default_unicast_transport = UdpTransportRead::new(
                tokio::net::UdpSocket::from_std(default_unicast_socket)
                    .expect("Should not fail to open default unicast socket"),
            );

            while let Some((_locator, message)) = default_unicast_transport.read().await {
                let r = participant_address_clone
                    .send_mail(
                        domain_participant_actor::process_user_defined_rtps_message::new(
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

        if self.qos.entity_factory.autoenable_created_entities {
            domain_participant.enable()?;
        }

        Ok(domain_participant)
    }

    pub fn delete_participant(&mut self, participant: &DomainParticipant) -> DdsResult<()> {
        let handle = participant.get_instance_handle()?;
        let is_participant_empty = self.domain_participant_list[&handle]
            .send_mail_and_await_reply_blocking(domain_participant_actor::is_empty::new());
        if is_participant_empty {
            self.domain_participant_list.remove(&handle);
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    pub fn lookup_participant(&self, domain_id: DomainId) -> DdsResult<Option<DomainParticipant>> {
        Ok(self
            .domain_participant_list
            .values()
            .find(|dp| {
                dp.send_mail_and_await_reply_blocking(
                        domain_participant_actor::get_domain_id::new(),
                    ) == domain_id
            })
            .map(|dp| DomainParticipant::new(dp.address())))
    }

    pub fn set_default_participant_qos(
        &mut self,
        qos: QosKind<DomainParticipantQos>,
    ) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };

        self.default_participant_qos = qos;

        Ok(())
    }

    pub fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        Ok(self.default_participant_qos.clone())
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;

        Ok(())
    }

    pub fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        Ok(self.qos.clone())
    }

    pub fn set_configuration(&mut self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        self.configuration = configuration;
        Ok(())
    }

    pub fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        Ok(self.configuration.clone())
    }

    fn get_unique_participant_id(&mut self) -> u32 {
        let counter = self.domain_participant_counter;
        self.domain_participant_counter += 1;
        counter
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
