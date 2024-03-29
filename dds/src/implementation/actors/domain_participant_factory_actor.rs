use crate::{
    configuration::DustDdsConfiguration,
    dds_async::{
        domain_participant::DomainParticipantAsync,
        domain_participant_listener::DomainParticipantListenerAsync,
    },
    domain::domain_participant_factory::DomainId,
    implementation::{
        actors::domain_participant_actor::DomainParticipantActor,
        rtps::{
            messages::overall_structure::RtpsMessageRead,
            participant::RtpsParticipant,
            types::{Locator, LOCATOR_KIND_UDP_V4, PROTOCOLVERSION, VENDOR_ID_S2E},
        },
        rtps_udp_psm::udp_transport::UdpTransportWrite,
        utils::actor::{Actor, ActorAddress},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};
use dust_dds_derive::actor_interface;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};
use socket2::Socket;
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc, OnceLock,
    },
};
use tracing::{info, warn};

pub struct DomainParticipantFactoryActor {
    domain_participant_list: HashMap<InstanceHandle, Actor<DomainParticipantActor>>,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
    configuration: DustDdsConfiguration,
}

impl DomainParticipantFactoryActor {
    pub fn new() -> Self {
        Self {
            domain_participant_list: HashMap::new(),
            qos: DomainParticipantFactoryQos::default(),
            default_participant_qos: DomainParticipantQos::default(),
            configuration: DustDdsConfiguration::default(),
        }
    }

    fn get_unique_participant_id(&mut self) -> u32 {
        static COUNTER: OnceLock<AtomicU32> = OnceLock::new();
        let c = COUNTER.get_or_init(|| AtomicU32::new(0));
        c.fetch_add(1, Ordering::Acquire)
    }
}

pub async fn read_message(socket: &mut tokio::net::UdpSocket) -> DdsResult<RtpsMessageRead> {
    let mut buf = vec![0; 65507];
    let (bytes, _) = socket.recv_from(&mut buf).await?;
    buf.truncate(bytes);
    if bytes > 0 {
        RtpsMessageRead::new(Arc::from(buf.into_boxed_slice()))
    } else {
        Err(DdsError::NoData)
    }
}

#[actor_interface]
impl DomainParticipantFactoryActor {
    async fn create_participant(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        listener: Box<dyn DomainParticipantListenerAsync + Send>,
        status_kind: Vec<StatusKind>,
        runtime_handle: tokio::runtime::Handle,
    ) -> DdsResult<ActorAddress<DomainParticipantActor>> {
        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let interface_address_list =
            get_interface_address_list(self.configuration.interface_name());

        let host_id = if let Some(interface) = interface_address_list.first() {
            match interface.ip() {
                IpAddr::V4(a) => a.octets(),
                IpAddr::V6(_) => unimplemented!("IPv6 not yet implemented"),
            }
        } else {
            warn!("Failed to get Host ID from IP address, use 0 instead");
            [0; 4]
        };

        let app_id = std::process::id().to_ne_bytes();
        let instance_id = self.get_unique_participant_id().to_ne_bytes();

        #[rustfmt::skip]
        let guid_prefix = [
            host_id[0],  host_id[1], host_id[2], host_id[3], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
        ];

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
            .map(|a| Locator::from_ip_and_port(a, user_defined_unicast_locator_port))
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
            .map(|a| Locator::from_ip_and_port(a, metattrafic_unicast_locator_port))
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
            &runtime_handle,
        )
        .await;
        let participant_actor = Actor::spawn(domain_participant, &runtime_handle);
        let participant_address = participant_actor.address();
        self.domain_participant_list.insert(
            InstanceHandle::new(participant_guid.into()),
            participant_actor,
        );
        let status_condition = participant_address.get_statuscondition().await?;
        let builtin_subscriber = participant_address.get_built_in_subscriber().await?;
        let builtin_subscriber_status_condition_address =
            builtin_subscriber.get_statuscondition().await?;
        let participant = DomainParticipantAsync::new(
            participant_address.clone(),
            status_condition.clone(),
            builtin_subscriber,
            builtin_subscriber_status_condition_address,
            domain_id,
            runtime_handle.clone(),
        );

        let participant_address_clone = participant_address.clone();
        let participant_clone = participant.clone();
        let mut socket = get_multicast_socket(
            DEFAULT_MULTICAST_LOCATOR_ADDRESS,
            port_builtin_multicast(domain_id),
            &interface_address_list,
        )
        .map_err(|_| DdsError::Error("Failed to open socket".to_string()))?;
        runtime_handle.spawn(async move {
            loop {
                if let Ok(message) = read_message(&mut socket).await {
                    let r = participant_address_clone
                        .process_metatraffic_rtps_message(message, participant_clone.clone())
                        .await;
                    if r.is_err() {
                        break;
                    }

                    let r = participant_address_clone
                        .process_builtin_discovery(participant_clone.clone())
                        .await;
                    if r.is_err() {
                        break;
                    }
                    let r = participant_address_clone.send_message().await;
                    if r.is_err() {
                        break;
                    }
                }
            }
        });

        let participant_address_clone = participant_address.clone();
        let participant_clone = participant.clone();
        let mut socket =
            tokio::net::UdpSocket::from_std(metattrafic_unicast_socket).map_err(|_| {
                DdsError::Error("Failed to open metattrafic unicast socket".to_string())
            })?;
        runtime_handle.spawn(async move {
            loop {
                if let Ok(message) = read_message(&mut socket).await {
                    let r: DdsResult<()> = async {
                        participant_address_clone
                            .process_metatraffic_rtps_message(message, participant_clone.clone())
                            .await??;
                        participant_address_clone
                            .process_builtin_discovery(participant_clone.clone())
                            .await?;

                        participant_address_clone.send_message().await?;
                        Ok(())
                    }
                    .await;

                    if r.is_err() {
                        break;
                    }
                }
            }
        });

        let participant_address_clone = participant_address.clone();
        let participant_clone = participant.clone();
        let mut socket = tokio::net::UdpSocket::from_std(default_unicast_socket)
            .map_err(|_| DdsError::Error("Failed to open default unicast socket".to_string()))?;
        runtime_handle.spawn(async move {
            loop {
                if let Ok(message) = read_message(&mut socket).await {
                    let r = participant_address_clone
                        .process_user_defined_rtps_message(message, participant_clone.clone())
                        .await;

                    if r.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(participant_address)
    }

    async fn delete_participant(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        let is_participant_empty = self.domain_participant_list[&handle].is_empty().await;
        if is_participant_empty {
            self.domain_participant_list.remove(&handle);
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    async fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> DdsResult<Option<ActorAddress<DomainParticipantActor>>> {
        for dp in self.domain_participant_list.values() {
            if dp.get_domain_id().await == domain_id {
                return Ok(Some(dp.address()));
            }
        }

        Ok(None)
    }

    async fn set_default_participant_qos(
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

    async fn get_default_participant_qos(&self) -> DdsResult<DomainParticipantQos> {
        Ok(self.default_participant_qos.clone())
    }

    async fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;

        Ok(())
    }

    async fn get_qos(&self) -> DdsResult<DomainParticipantFactoryQos> {
        Ok(self.qos.clone())
    }

    async fn set_configuration(&mut self, configuration: DustDdsConfiguration) -> DdsResult<()> {
        self.configuration = configuration;
        Ok(())
    }

    async fn get_configuration(&self) -> DdsResult<DustDdsConfiguration> {
        Ok(self.configuration.clone())
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

fn get_interface_address_list(interface_name: Option<&String>) -> Vec<Addr> {
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
            i.addr.into_iter().filter(|a| match a {
                #[rustfmt::skip]
                Addr::V4(v4) if !v4.ip.is_loopback() => true,
                _ => false,
            })
        })
        .collect()
}

fn get_multicast_socket(
    multicast_address: LocatorAddress,
    port: u16,
    interface_address_list: &[Addr],
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
    for interface_addr in interface_address_list {
        match interface_addr {
            Addr::V4(a) => {
                let r = socket.join_multicast_v4(&addr, &a.ip);
                if let Err(e) = r {
                    info!(
                        "Failed to join multicast group on address {} with error {}",
                        a.ip, e
                    )
                }
            }
            Addr::V6(_) => (),
        }
    }

    socket.set_multicast_loop_v4(true)?;

    tokio::net::UdpSocket::from_std(socket.into())
}
