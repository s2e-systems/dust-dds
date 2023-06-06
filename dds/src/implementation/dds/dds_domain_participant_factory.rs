use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use jsonschema::JSONSchema;
use lazy_static::lazy_static;
use mac_address::MacAddress;
use schemars::schema_for;
use socket2::Socket;

use crate::{
    domain::{
        domain_participant_factory::DomainId,
        domain_participant_listener::DomainParticipantListener,
    },
    implementation::{
        configuration::DustDdsConfiguration,
        dds::dds_domain_participant::AnnounceKind,
        rtps::{
            messages::overall_structure::{RtpsMessageRead, RtpsMessageWrite},
            participant::RtpsParticipant,
            transport::TransportWrite,
            types::{
                GuidPrefix, Locator, LocatorAddress, LocatorPort, LOCATOR_KIND_UDP_V4,
                PROTOCOLVERSION, VENDOR_ID_S2E,
            },
        },
        rtps_udp_psm::udp_transport::{UdpTransportRead, UdpTransportWrite},
        utils::{
            actor::{spawn_actor, Actor, ActorAddress, THE_RUNTIME},
            condvar::DdsCondvar,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DomainParticipantFactoryQos, DomainParticipantQos, QosKind},
        status::StatusKind,
    },
};

use super::dds_domain_participant::DdsDomainParticipant;

lazy_static! {
    static ref THE_DDS_CONFIGURATION: DustDdsConfiguration =
        if let Ok(configuration_json) = std::env::var("DUST_DDS_CONFIGURATION") {
            configuration_try_from_str(configuration_json.as_str()).unwrap()
        } else {
            DustDdsConfiguration::default()
        };
}

pub struct DdsDomainParticipantFactory {
    domain_participant_list: Vec<Actor<DdsDomainParticipant>>,
    domain_participant_counter: u32,
    qos: DomainParticipantFactoryQos,
    default_participant_qos: DomainParticipantQos,
}

impl Default for DdsDomainParticipantFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl DdsDomainParticipantFactory {
    pub fn new() -> Self {
        Self {
            domain_participant_list: Vec::new(),
            domain_participant_counter: 0,
            qos: DomainParticipantFactoryQos::default(),
            default_participant_qos: DomainParticipantQos::default(),
        }
    }

    pub fn create_participant(
        &mut self,
        domain_id: DomainId,
        qos: QosKind<DomainParticipantQos>,
        a_listener: Option<Box<dyn DomainParticipantListener + Send + Sync>>,
        mask: &[StatusKind],
    ) -> DdsResult<ActorAddress<DdsDomainParticipant>> {
        async fn task_send_entity_announce(
            mut announce_receiver: tokio::sync::mpsc::Receiver<AnnounceKind>,
            domain_participant_address: ActorAddress<DdsDomainParticipant>,
        ) {
            loop {
                if let Some(announce_kind) = announce_receiver.recv().await {
                    tokio::task::block_in_place(|| {
                        domain_participant_address
                            .announce_entity(announce_kind)
                            .unwrap()
                    });
                }
            }
        }

        async fn task_metatraffic_multicast_receive(
            mut metatraffic_multicast_transport: UdpTransportRead,
            builtin_message_broadcast_sender: tokio::sync::broadcast::Sender<(
                Locator,
                RtpsMessageRead,
            )>,
        ) {
            loop {
                if let Some((locator, message)) = metatraffic_multicast_transport.read().await {
                    builtin_message_broadcast_sender
                        .send((locator, message))
                        .ok();
                    // tokio::task::block_in_place(|| {
                    //     domain_participant_address
                    //         .receive_builtin_message(locator, message)
                    //         .unwrap()
                    // });
                }
            }
        }

        async fn task_metatraffic_unicast_receive(
            mut metatraffic_unicast_transport: UdpTransportRead,
            domain_participant_address: ActorAddress<DdsDomainParticipant>,
        ) {
            loop {
                if let Some((locator, message)) = metatraffic_unicast_transport.read().await {
                    tokio::task::block_in_place(|| {
                        domain_participant_address
                            .receive_builtin_message(locator, message)
                            .unwrap()
                    });
                }
            }
        }

        async fn task_user_defined_receive(
            mut default_unicast_transport: UdpTransportRead,
            domain_participant_address: ActorAddress<DdsDomainParticipant>,
        ) {
            loop {
                if let Some((locator, message)) = default_unicast_transport.read().await {
                    tokio::task::block_in_place(|| {
                        domain_participant_address
                            .receive_user_defined_message(locator, message)
                            .unwrap()
                    });
                }
            }
        }

        async fn task_unicast_metatraffic_communication_send(
            mut rtps_message_channel_receiver: tokio::sync::mpsc::Receiver<(
                RtpsMessageWrite,
                Vec<Locator>,
            )>,
        ) {
            let socket = std::net::UdpSocket::bind("0.0.0.0:0000").unwrap();

            let metatraffic_unicast_transport_send = UdpTransportWrite::new(socket);

            loop {
                if let Some((message, destination_locator_list)) =
                    rtps_message_channel_receiver.recv().await
                {
                    metatraffic_unicast_transport_send.write(&message, &destination_locator_list);
                }
            }
        }

        async fn task_unicast_user_defined_communication_send(
            mut rtps_message_channel_receiver: tokio::sync::mpsc::Receiver<(
                RtpsMessageWrite,
                Vec<Locator>,
            )>,
        ) {
            let socket = std::net::UdpSocket::bind("0.0.0.0:0000").unwrap();
            let default_unicast_transport_send = UdpTransportWrite::new(socket);

            loop {
                if let Some((message, destination_locator_list)) =
                    rtps_message_channel_receiver.recv().await
                {
                    default_unicast_transport_send.write(&message, &destination_locator_list);
                }
            }
        }

        async fn task_announce_participant(
            domain_participant_address: ActorAddress<DdsDomainParticipant>,
        ) {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            loop {
                tokio::task::block_in_place(|| {
                    domain_participant_address.announce_participant().unwrap()
                });

                interval.tick().await;
            }
        }

        let domain_participant_qos = match qos {
            QosKind::Default => self.default_participant_qos.clone(),
            QosKind::Specific(q) => q,
        };

        let mac_address = ifcfg::IfCfg::get()
            .expect("Could not scan interfaces")
            .into_iter()
            .filter_map(|i| MacAddress::from_str(&i.mac).ok())
            .find(|&mac| mac != MacAddress::new([0, 0, 0, 0, 0, 0]))
            .expect("Could not find any mac address")
            .bytes();

        let app_id = std::process::id().to_ne_bytes();
        let instance_id = self.domain_participant_counter.to_ne_bytes();
        self.domain_participant_counter += 1;

        #[rustfmt::skip]
        let guid_prefix = GuidPrefix::new([
            mac_address[2],  mac_address[3], mac_address[4], mac_address[5], // Host ID
            app_id[0], app_id[1], app_id[2], app_id[3], // App ID
            instance_id[0], instance_id[1], instance_id[2], instance_id[3], // Instance ID
        ]);

        let interface_address_list =
            get_interface_address_list(THE_DDS_CONFIGURATION.interface_name.as_ref());

        let default_unicast_socket =
            std::net::UdpSocket::bind(SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0)))
                .map_err(|_| DdsError::Error)?;
        default_unicast_socket.set_nonblocking(true).unwrap();
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
        metattrafic_unicast_socket.set_nonblocking(true).unwrap();

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

        let sedp_condvar = DdsCondvar::new();
        let user_defined_data_send_condvar = DdsCondvar::new();
        let (announce_sender, announce_receiver) = tokio::sync::mpsc::channel(500);

        let rtps_participant = RtpsParticipant::new(
            guid_prefix,
            default_unicast_locator_list,
            default_multicast_locator_list,
            metatraffic_unicast_locator_list,
            metatraffic_multicast_locator_list,
            PROTOCOLVERSION,
            VENDOR_ID_S2E,
        );

        let (builtin_rtps_message_channel_sender, builtin_rtps_message_channel_receiver) =
            tokio::sync::mpsc::channel(10);

        let (user_defined_rtps_message_channel_sender, user_defined_rtps_message_channel_receiver) =
            tokio::sync::mpsc::channel(10);

        let domain_participant = DdsDomainParticipant::new(
            rtps_participant,
            domain_id,
            THE_DDS_CONFIGURATION.domain_tag.clone(),
            domain_participant_qos,
            &spdp_discovery_locator_list,
            user_defined_data_send_condvar.clone(),
            THE_DDS_CONFIGURATION.fragment_size,
            announce_sender,
            sedp_condvar.clone(),
            builtin_rtps_message_channel_sender,
            user_defined_rtps_message_channel_sender,
        );

        let participant_actor = spawn_actor(domain_participant);
        let participant_address = participant_actor.address();
        self.domain_participant_list.push(participant_actor);

        let metatraffic_multicast_transport = UdpTransportRead::new(
            get_multicast_socket(
                DEFAULT_MULTICAST_LOCATOR_ADDRESS,
                port_builtin_multicast(domain_id),
            )
            .unwrap(),
        );

        THE_RUNTIME.spawn(task_send_entity_announce(
            announce_receiver,
            participant_address.clone(),
        ));

        let (builtin_message_broadcast_sender, _builtin_message_broadcast_receiver) =
            tokio::sync::broadcast::channel(10);
        THE_RUNTIME.spawn(task_metatraffic_multicast_receive(
            metatraffic_multicast_transport,
            builtin_message_broadcast_sender,
        ));

        let metatraffic_unicast_transport = UdpTransportRead::new(
            tokio::net::UdpSocket::from_std(metattrafic_unicast_socket).unwrap(),
        );

        THE_RUNTIME.spawn(task_metatraffic_unicast_receive(
            metatraffic_unicast_transport,
            participant_address.clone(),
        ));

        let default_unicast_transport =
            UdpTransportRead::new(tokio::net::UdpSocket::from_std(default_unicast_socket).unwrap());

        THE_RUNTIME.spawn(task_user_defined_receive(
            default_unicast_transport,
            participant_address.clone(),
        ));

        THE_RUNTIME.spawn(task_unicast_metatraffic_communication_send(
            builtin_rtps_message_channel_receiver,
        ));

        THE_RUNTIME.spawn(task_unicast_user_defined_communication_send(
            user_defined_rtps_message_channel_receiver,
        ));

        THE_RUNTIME.spawn(task_announce_participant(participant_address.clone()));

        if self.qos.entity_factory.autoenable_created_entities {
            participant_address.enable()?;
        }

        Ok(participant_address)
    }

    pub fn delete_participant(&mut self, handle: InstanceHandle) -> DdsResult<()> {
        let idx = self
            .domain_participant_list
            .iter()
            .position(|dp| {
                dp.address()
                    .get_instance_handle()
                    .expect("Should not fail to send message")
                    == handle
            })
            .ok_or(DdsError::BadParameter)?;

        let is_participant_empty = self.domain_participant_list[idx].address().is_empty()?;
        if is_participant_empty {
            self.domain_participant_list.remove(idx);
            Ok(())
        } else {
            Err(DdsError::PreconditionNotMet(
                "Domain participant still contains other entities".to_string(),
            ))
        }
    }

    pub fn lookup_participant(
        &self,
        domain_id: DomainId,
    ) -> Option<ActorAddress<DdsDomainParticipant>> {
        self.domain_participant_list
            .iter()
            .map(|dp| dp.address())
            .find(|a| a.get_domain_id().expect("Should not fail to send message") == domain_id)
    }

    pub fn get_qos(&self) -> &DomainParticipantFactoryQos {
        &self.qos
    }

    pub fn set_qos(&mut self, qos: QosKind<DomainParticipantFactoryQos>) {
        let qos = match qos {
            QosKind::Default => DomainParticipantFactoryQos::default(),
            QosKind::Specific(q) => q,
        };

        self.qos = qos;
    }

    pub fn get_default_participant_qos(&self) -> &DomainParticipantQos {
        &self.default_participant_qos
    }

    pub fn set_default_participant_qos(&mut self, qos: QosKind<DomainParticipantQos>) {
        let qos = match qos {
            QosKind::Default => DomainParticipantQos::default(),
            QosKind::Specific(q) => q,
        };
        self.default_participant_qos = qos;
    }
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
    ifcfg::IfCfg::get()
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
            i.addresses.into_iter().filter_map(|a| match a.address? {
                #[rustfmt::skip]
                SocketAddr::V4(v4) if !v4.ip().is_loopback() => Some(
                    LocatorAddress::new([0, 0, 0, 0,
                        0, 0, 0, 0,
                        0, 0, 0, 0,
                        v4.ip().octets()[0], v4.ip().octets()[1], v4.ip().octets()[2], v4.ip().octets()[3]])
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
